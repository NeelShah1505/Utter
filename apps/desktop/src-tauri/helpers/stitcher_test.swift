// stitcher_test.swift
//
// Standalone test for TranscriptStitcher. Run with:
//   swiftc helpers/stitcher.swift helpers/stitcher_test.swift -o /tmp/stitcher_test && /tmp/stitcher_test
//
// Not part of the app build; it only links stitcher.swift.

import Foundation

var failures = 0
func check(_ cond: Bool, _ msg: String) {
    if cond {
        print("  ok: \(msg)")
    } else {
        print("FAIL: \(msg)")
        failures += 1
    }
}

@main
enum StitcherTest {
  static func main() {

// 1. A single task producing partials, then stopping.
do {
    var s = TranscriptStitcher()
    s.update("hello")
    s.update("hello world")
    check(s.combined == "hello world", "partials replace, not append")
    s.commit()
    check(s.combined == "hello world", "final commit keeps the words")
}

// 2. THE BUG: a task that ends with an error mid-sentence must not lose words.
do {
    var s = TranscriptStitcher()
    s.update("book an appointment")   // task 1 partials
    s.commit()                        // task 1 errors out (~10s) -> commit, restart
    s.update("for ten pm")            // task 2 partials
    check(s.combined == "book an appointment for ten pm",
          "words before an error-triggered restart survive")
    s.commit()
    check(s.combined == "book an appointment for ten pm", "nothing lost at stop")
}

// 3. Many restarts over a long dictation accumulate.
do {
    var s = TranscriptStitcher()
    for i in 1...20 {
        s.update("segment\(i)")
        s.commit()
    }
    let parts = s.combined.split(separator: " ")
    check(parts.count == 20, "20 restarts -> 20 segments, none dropped")
    check(parts.first == "segment1" && parts.last == "segment20", "order preserved")
}

// 4. Empty segments never create stray spaces.
do {
    check(TranscriptStitcher.join("", "a") == "a", "join empty-left")
    check(TranscriptStitcher.join("a", "") == "a", "join empty-right")
    check(TranscriptStitcher.join("", "") == "", "join empty-both")
    var s = TranscriptStitcher()
    s.commit()                 // a task that produced nothing
    s.update("first words")
    check(s.combined == "first words", "leading empty commit adds no space")
}

print(failures == 0 ? "\nALL PASSED" : "\n\(failures) FAILED")
exit(failures == 0 ? 0 : 1)
  }
}
