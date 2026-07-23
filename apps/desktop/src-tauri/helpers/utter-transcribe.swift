#!/usr/bin/env swift
//
// utter-transcribe — macOS Speech Recognition helper.
//
// Usage:
//   utter-transcribe <path-to-wav-or-raw-pcm-file>
//
// Stdin protocol (when no file arg):
//   The helper reads raw 16-bit signed little-endian PCM at 16 kHz, mono
//   from stdin until EOF, then runs SFSpeechRecognizer offline and writes
//   the transcript (with trailing newline) to stdout.
//
// Why this approach:
//   - SFSpeechRecognizer with requiresOnDeviceRecognition=true uses the same
//     CoreML/ANE engine as Siri. On Apple Silicon it runs entirely offline.
//   - Zero model download. Zero cmake. Zero FFI glue.
//   - Available since macOS 13 (Ventura); user is on macOS 26.
//
// Output:
//   stdout: transcript text (may be empty if nothing recognised)
//   stderr: errors or debug info
//   exit 0: success (even if transcript is empty)
//   exit 1: fatal error (engine unavailable, permission denied, etc.)

import Foundation
import Speech
import AVFoundation

// MARK: — Permission check

let status = SFSpeechRecognizer.authorizationStatus()
if status == .denied || status == .restricted {
    fputs("error: Speech Recognition permission denied. "
        + "Grant access in System Settings > Privacy > Speech Recognition.\n", stderr)
    exit(1)
}

// MARK: — Read PCM from stdin

var pcmData = Data()
let bufSize = 4096
var buf = [UInt8](repeating: 0, count: bufSize)

while true {
    let n = read(STDIN_FILENO, &buf, bufSize)
    if n <= 0 { break }
    pcmData.append(contentsOf: buf.prefix(n))
}

if pcmData.isEmpty {
    // Nothing recorded — write empty transcript and exit cleanly
    print("")
    exit(0)
}

// MARK: — Build AVAudioPCMBuffer from raw f32 PCM

// Rust sends f32 samples (little-endian 32-bit float) at 16 kHz mono
let sampleRate: Double = 16000
let channelCount: AVAudioChannelCount = 1
let sampleCount = pcmData.count / MemoryLayout<Float>.size

guard
    let format = AVAudioFormat(
        commonFormat: .pcmFormatFloat32,
        sampleRate: sampleRate,
        channels: channelCount,
        interleaved: false
    ),
    let buffer = AVAudioPCMBuffer(pcmFormat: format, frameCapacity: AVAudioFrameCount(sampleCount))
else {
    fputs("error: could not create AVAudioPCMBuffer\n", stderr)
    exit(1)
}

buffer.frameLength = AVAudioFrameCount(sampleCount)
pcmData.withUnsafeBytes { rawPtr in
    let floatPtr = rawPtr.bindMemory(to: Float.self)
    guard let base = buffer.floatChannelData?[0] else { return }
    for i in 0 ..< sampleCount {
        base[i] = floatPtr[i]
    }
}

// MARK: — Speech recognition

guard let recognizer = SFSpeechRecognizer(locale: Locale(identifier: "en-US")),
      recognizer.isAvailable else {
    fputs("error: SFSpeechRecognizer not available (locale: en-US)\n", stderr)
    exit(1)
}

recognizer.defaultTaskHint = .dictation

let request = SFSpeechAudioBufferRecognitionRequest()
request.requiresOnDeviceRecognition = true   // ← fully offline, no Siri calls
request.shouldReportPartialResults = false
request.addsPunctuation = true               // ← automatic punctuation (macOS 13+)
request.append(buffer)
request.endAudio()

// Run on a semaphore so the script doesn't exit before recognition completes
let sema = DispatchSemaphore(value: 0)
var transcript = ""
var recognitionError: Error?

recognizer.recognitionTask(with: request) { result, error in
    if let result = result, result.isFinal {
        transcript = result.bestTranscription.formattedString
        sema.signal()
    } else if let error = error {
        recognitionError = error
        sema.signal()
    }
}

// Timeout: 30 seconds should be more than enough for any dictation session
let timedOut = sema.wait(timeout: .now() + 30) == .timedOut
if timedOut {
    fputs("warning: recognition timed out\n", stderr)
}
if let err = recognitionError {
    fputs("error: recognition failed: \(err.localizedDescription)\n", stderr)
    exit(1)
}

print(transcript)
exit(0)
