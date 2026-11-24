// stitcher.swift
//
// Pure transcript accumulator, independent of SFSpeechRecognizer so it can be
// unit-tested without a microphone (see stitcher_test.swift).
//
// SFSpeechRecognizer tasks do not run forever: they end on silence, on an
// internal duration cap (often only ~10-60s on device), or with an error such
// as code 203 "no speech detected". To dictate for as long as the user holds
// the hotkey, we restart a task each time one ends and stitch the pieces back
// together. The critical rule: whenever a task ends — final OR error — the
// in-flight segment must be committed to the finalized buffer BEFORE the next
// task starts, or those words are lost.

import Foundation

struct TranscriptStitcher {
    /// Words from tasks that have already ended.
    private(set) var finalized: String = ""
    /// The newest partial from the task that is currently running.
    private(set) var segment: String = ""

    /// Replace the in-flight segment with the latest partial for this task.
    mutating func update(_ newSegment: String) {
        segment = newSegment
    }

    /// A task ended (final or error): fold the in-flight segment into the
    /// finalized buffer and clear it so the next task starts clean.
    mutating func commit() {
        finalized = Self.join(finalized, segment)
        segment = ""
    }

    /// Everything heard so far: finalized words plus the in-flight segment.
    var combined: String {
        Self.join(finalized, segment)
    }

    /// Join two transcript fragments with a single separating space, treating
    /// empty fragments as absent so we never produce leading/trailing spaces.
    static func join(_ a: String, _ b: String) -> String {
        if a.isEmpty { return b }
        if b.isEmpty { return a }
        return a + " " + b
    }
}
