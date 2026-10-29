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
    
    private var finalizedBuffer: String = ""
    private var currentSegment: String = ""
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
        
        return startTask()
    }
    
    private func startTask() -> Bool {
        syncQueue.sync {
            if isManuallyEnded { return false }
            
            request = Self.makeRequest()
            guard let request = request else { return false }
            
            let ctx = self.contextPtr
            let cb = self.callback
            
            task = recognizer?.recognitionTask(with: request) { [weak self] result, error in
                guard let self = self else { return }
                
                self.syncQueue.async {
                    if let result = result {
                        self.currentSegment = result.bestTranscription.formattedString
                        let isFinal = result.isFinal
                        
                        let combined = self.finalizedBuffer + (self.finalizedBuffer.isEmpty || self.currentSegment.isEmpty ? "" : " ") + self.currentSegment
                        
                        if isFinal {
                            // Task ended naturally (e.g. silence timeout). Stitch the text.
                            self.finalizedBuffer = combined
                            self.currentSegment = ""
                            
                            if !self.isManuallyEnded {
                                // Restart task seamlessly
                                _ = self.startTaskWithoutSync()
                            } else {
                                combined.withCString { cb(ctx, $0, true) }
                            }
                        } else {
                            // Still transcribing...
                            combined.withCString { cb(ctx, $0, false) }
                        }
                    } else if error != nil {
                        // Error occurred (usually code 203: no speech). Task dies.
                        if !self.isManuallyEnded {
                            // Restart seamlessly
                            _ = self.startTaskWithoutSync()
                        } else {
                            self.finalizedBuffer.withCString { cb(ctx, $0, true) }
                        }
                    }
                }
            }
            return true
        }
    }
    
    // Internal version of startTask that doesn't deadlock inside syncQueue
    private func startTaskWithoutSync() -> Bool {
        if isManuallyEnded { return false }
        
        request = Self.makeRequest()
        guard let request = request else { return false }
        
        let ctx = self.contextPtr
        let cb = self.callback
        
        task = recognizer?.recognitionTask(with: request) { [weak self] result, error in
            guard let self = self else { return }
            
            self.syncQueue.async {
                if let result = result {
                    self.currentSegment = result.bestTranscription.formattedString
                    let isFinal = result.isFinal
                    
                    let combined = self.finalizedBuffer + (self.finalizedBuffer.isEmpty || self.currentSegment.isEmpty ? "" : " ") + self.currentSegment
                    
                    if isFinal {
                        self.finalizedBuffer = combined
                        self.currentSegment = ""
                        
                        if !self.isManuallyEnded {
                            _ = self.startTaskWithoutSync()
                        } else {
                            combined.withCString { cb(ctx, $0, true) }
                        }
                    } else {
                        combined.withCString { cb(ctx, $0, false) }
                    }
                } else if error != nil {
                    if !self.isManuallyEnded {
                        _ = self.startTaskWithoutSync()
                    } else {
                        self.finalizedBuffer.withCString { cb(ctx, $0, true) }
                    }
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
            request?.endAudio()
            
            // If the request was nil or already died, we force the final callback ourselves
            if request == nil {
                let combined = self.finalizedBuffer
                combined.withCString { self.callback(self.contextPtr, $0, true) }
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
