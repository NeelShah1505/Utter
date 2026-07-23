#!/usr/bin/env swift
//
// utter-transcribe — macOS Speech Recognition helper.
//
// Protocol:
//   stdin:  raw f32 little-endian PCM, 16 kHz, mono
//   stdout: transcript text (with trailing newline)
//   stderr: warnings / errors
//   exit 0: success (transcript may be empty)
//   exit 1: fatal error
//
// The binary is meant to be called from within the Utter app bundle.
// The app bundle holds the Speech Recognition entitlement; the helper
// inherits the parent process's permission context.

import Foundation
import Speech
import AVFoundation

// MARK: — Authorization (request if not yet determined)

var authStatus = SFSpeechRecognizer.authorizationStatus()

if authStatus == .notDetermined {
    let authSema = DispatchSemaphore(value: 0)
    SFSpeechRecognizer.requestAuthorization { status in
        authStatus = status
        authSema.signal()
    }
    // Wait up to 10 s for the user to respond to the permission dialog
    _ = authSema.wait(timeout: .now() + 10)
}

switch authStatus {
case .denied, .restricted:
    fputs("error: Speech Recognition permission denied. "
        + "Grant access in System Settings › Privacy › Speech Recognition.\n", stderr)
    exit(1)
case .notDetermined:
    fputs("error: Speech Recognition permission not determined (timed out).\n", stderr)
    exit(1)
case .authorized:
    break  // Good to go
@unknown default:
    fputs("warning: unknown authorization status \(authStatus.rawValue)\n", stderr)
}

// MARK: — Read raw f32 PCM from stdin

var pcmData = Data()
var buf = [UInt8](repeating: 0, count: 8192)

while true {
    let n = read(STDIN_FILENO, &buf, buf.count)
    if n <= 0 { break }
    pcmData.append(contentsOf: buf.prefix(n))
}

if pcmData.isEmpty {
    print("")  // empty transcript, not an error
    exit(0)
}

// MARK: — Build AVAudioPCMBuffer from raw f32 PCM

let sampleRate: Double = 16000
let sampleCount = pcmData.count / MemoryLayout<Float>.size

guard
    let format = AVAudioFormat(
        commonFormat: .pcmFormatFloat32,
        sampleRate:   sampleRate,
        channels:     1,
        interleaved:  false
    ),
    let buffer = AVAudioPCMBuffer(
        pcmFormat:     format,
        frameCapacity: AVAudioFrameCount(sampleCount)
    )
else {
    fputs("error: could not create AVAudioPCMBuffer\n", stderr)
    exit(1)
}

buffer.frameLength = AVAudioFrameCount(sampleCount)
pcmData.withUnsafeBytes { rawPtr in
    let floats = rawPtr.bindMemory(to: Float.self)
    guard let base = buffer.floatChannelData?[0] else { return }
    for i in 0 ..< sampleCount { base[i] = floats[i] }
}

// MARK: — Speech recognition

guard
    let recognizer = SFSpeechRecognizer(locale: Locale(identifier: "en-US")),
    recognizer.isAvailable
else {
    fputs("error: SFSpeechRecognizer unavailable (locale en-US)\n", stderr)
    exit(1)
}

recognizer.defaultTaskHint = .dictation

let request = SFSpeechAudioBufferRecognitionRequest()
request.requiresOnDeviceRecognition = true  // fully offline
request.shouldReportPartialResults  = false
request.addsPunctuation             = true  // automatic punctuation (macOS 13+)

// IMPORTANT: start the recognitionTask BEFORE appending audio / calling endAudio().
// Starting after endAudio() races with the internal queue and causes silent timeouts.

let sema = DispatchSemaphore(value: 0)
var transcript      = ""
var recognitionError: Error?

let task = recognizer.recognitionTask(with: request) { result, error in
    if let result = result, result.isFinal {
        transcript = result.bestTranscription.formattedString
        sema.signal()
    } else if let error = error {
        // Error code 203 = "Retry" (no speech detected) — treat as empty, not fatal
        let nsErr = error as NSError
        if nsErr.code == 203 {
            sema.signal()  // transcript stays ""
        } else {
            recognitionError = error
            sema.signal()
        }
    }
}

// Feed audio buffer and signal end-of-stream
request.append(buffer)
request.endAudio()

// Timeout: 30 s should be more than enough for any dictation session
if sema.wait(timeout: .now() + 30) == .timedOut {
    task.cancel()
    fputs("warning: recognition timed out\n", stderr)
}

if let err = recognitionError {
    fputs("error: recognition failed: \(err.localizedDescription)\n", stderr)
    exit(1)
}

print(transcript)
exit(0)
