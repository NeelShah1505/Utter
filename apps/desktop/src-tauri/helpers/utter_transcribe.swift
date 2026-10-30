// utter_transcribe.swift
// 
// Streaming Speech Recognition using SFSpeechAudioBufferRecognitionRequest
import Foundation
import Speech
import AVFoundation
import AppKit

public typealias TranscriptCallback = @convention(c) (UnsafeMutableRawPointer?, UnsafePointer<CChar>?, Bool) -> Void

class StreamingRecognizer {
    private let recognizer: SFSpeechRecognizer?
    private var request: SFSpeechAudioBufferRecognitionRequest?
    private var task: SFSpeechRecognitionTask?

    private let sampleRate: Double
    private let contextPtr: UnsafeMutableRawPointer?
    private let callback: TranscriptCallback

    private var stitcher = TranscriptStitcher()
    private var isManuallyEnded = false

    private let syncQueue = DispatchQueue(label: "com.utter.transcriber.sync")

    init?(sampleRate: Double, contextPtr: UnsafeMutableRawPointer?, callback: @escaping TranscriptCallback) {
        self.sampleRate = sampleRate
        self.contextPtr = contextPtr
        self.callback = callback

        let locale = Locale(identifier: "en-US")
        self.recognizer = SFSpeechRecognizer(locale: locale)
        guard let recognizer = self.recognizer, recognizer.isAvailable else {
            fputs("error: Speech Recognizer not available for en-US\n", stderr)
            return nil
        }
        // Offline claim: never fall back to Apple's servers.
        guard recognizer.supportsOnDeviceRecognition else {
            fputs("error: on-device speech recognition is unavailable for en-US. Download Dictation language data in System Settings.\n", stderr)
            return nil
        }
    }

    func start() -> Bool {
        var authStatus = SFSpeechRecognizer.authorizationStatus()
        if authStatus == .notDetermined {
            let authSema = DispatchSemaphore(value: 0)
            SFSpeechRecognizer.requestAuthorization { status in
                authStatus = status
                authSema.signal()
            }
            _ = authSema.wait(timeout: .now() + 10)
        }

        guard authStatus == .authorized else {
            fputs("error: Speech Recognition permission not authorized\n", stderr)
            return false
        }

        return syncQueue.sync { startTaskLocked() }
    }

    /// Emit the full transcript so far (finalized + in-flight segment).
    /// Must be called on `syncQueue`.
    private func emit(final: Bool) {
        stitcher.combined.withCString { callback(contextPtr, $0, final) }
    }

    /// Handle a task that has just ended, for any reason. Must be called on
    /// `syncQueue`. The segment is already committed by the caller; here we
    /// either finish (manual stop) or restart to keep dictating.
    private func handleTaskEnded() {
        if isManuallyEnded {
            emit(final: true)
        } else {
            _ = startTaskLocked()
        }
    }

    /// Start (or restart) a recognition task. Must be called on `syncQueue`.
    /// A single implementation drives both the first start and every restart,
    /// so the final/error handling can never drift between copies.
    private func startTaskLocked() -> Bool {
        if isManuallyEnded { return false }

        request = Self.makeRequest()
        guard let request = request else { return false }

        task = recognizer?.recognitionTask(with: request) { [weak self] result, error in
            guard let self = self else { return }

            self.syncQueue.async {
                if let result = result {
                    self.stitcher.update(result.bestTranscription.formattedString)
                    if result.isFinal {
                        // Task ended cleanly (silence or duration cap): keep the
                        // words, then restart or finish.
                        self.stitcher.commit()
                        self.handleTaskEnded()
                    } else {
                        self.emit(final: false)
                    }
                } else if error != nil {
                    // Task died with an error (commonly code 203 "no speech" or
                    // the on-device duration cap near ~10s). Commit the in-flight
                    // segment BEFORE restarting — without this the words spoken
                    // in this segment are lost. This was the "long dictation cuts
                    // off earlier words" bug.
                    self.stitcher.commit()
                    self.handleTaskEnded()
                }
            }
        }
        return true
    }

    private static func makeRequest() -> SFSpeechAudioBufferRecognitionRequest? {
        let request = SFSpeechAudioBufferRecognitionRequest()
        request.shouldReportPartialResults = true
        request.requiresOnDeviceRecognition = true
        if #available(macOS 13.0, *) {
            request.addsPunctuation = true
        }
        return request
    }

    func appendAudio(samples: UnsafePointer<Float>, count: Int) {
        guard count > 0 else { return }
        
        guard
            let format = AVAudioFormat(commonFormat: .pcmFormatFloat32, sampleRate: sampleRate, channels: 1, interleaved: false),
            let buffer = AVAudioPCMBuffer(pcmFormat: format, frameCapacity: AVAudioFrameCount(count))
        else {
            return
        }
        
        buffer.frameLength = AVAudioFrameCount(count)
        let channelData = buffer.floatChannelData![0]
        channelData.update(from: samples, count: count)
        
        syncQueue.sync {
            if !isManuallyEnded {
                request?.append(buffer)
            }
        }
    }
    
    func endAudio() {
        syncQueue.sync {
            isManuallyEnded = true
            if let request = request {
                // The recognizer will fire one last result; the task handler then
                // commits and emits the final transcript because isManuallyEnded.
                request.endAudio()
            } else {
                // No live task (it already died): emit whatever we have now so the
                // Rust side does not wait out its timeout for a callback.
                emit(final: true)
            }
        }
    }
}

// ----------------------------------------------------------------------------
// C-API Exposed to Rust
// ----------------------------------------------------------------------------

@_cdecl("speech_init_stream")
public func speechInitStream(sampleRate: Double, contextPtr: UnsafeMutableRawPointer?, callback: TranscriptCallback) -> UnsafeMutableRawPointer? {
    guard let manager = StreamingRecognizer(sampleRate: sampleRate, contextPtr: contextPtr, callback: callback) else {
        return nil
    }
    if !manager.start() {
        return nil
    }
    return Unmanaged.passRetained(manager).toOpaque()
}

@_cdecl("speech_append_audio")
public func speechAppendAudio(managerPtr: UnsafeMutableRawPointer, samples: UnsafePointer<Float>, count: Int) {
    let manager = Unmanaged<StreamingRecognizer>.fromOpaque(managerPtr).takeUnretainedValue()
    manager.appendAudio(samples: samples, count: count)
}

@_cdecl("speech_end_stream")
public func speechEndStream(managerPtr: UnsafeMutableRawPointer) {
    let manager = Unmanaged<StreamingRecognizer>.fromOpaque(managerPtr).takeUnretainedValue()
    manager.endAudio()
}

@_cdecl("speech_free_stream")
public func speechFreeStream(managerPtr: UnsafeMutableRawPointer) {
    // takeRetainedValue transfers ownership back to ARC, which then immediately deallocates it when scope exits.
    _ = Unmanaged<StreamingRecognizer>.fromOpaque(managerPtr).takeRetainedValue()
}

@_cdecl("free_transcript")
public func freeTranscript(ptr: UnsafeMutablePointer<CChar>?) {
    if let ptr = ptr { free(ptr) }
}

@_cdecl("get_frontmost_app_info")
public func get_frontmost_app_info(
    nameBuf: UnsafeMutablePointer<CChar>,
    nameMax: Int,
    bundleBuf: UnsafeMutablePointer<CChar>,
    bundleMax: Int
) -> Bool {
    guard let app = NSWorkspace.shared.frontmostApplication else {
        return false
    }
    let name = app.localizedName ?? "Desktop"
    let bundle = app.bundleIdentifier ?? "com.apple.finder"
    
    _ = name.withCString { src in
        strncpy(nameBuf, src, nameMax - 1)
        nameBuf[nameMax - 1] = 0
    }
    _ = bundle.withCString { src in
        strncpy(bundleBuf, src, bundleMax - 1)
        bundleBuf[bundleMax - 1] = 0
    }
    return true
}

@_cdecl("activate_app_to_front")
public func activate_app_to_front() {
    DispatchQueue.main.async {
        NSApplication.shared.setActivationPolicy(.regular)
        NSApplication.shared.activate(ignoringOtherApps: true)
    }
}
