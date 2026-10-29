# Decisions taken

The audit prompt said to stop here and wait. The same request said to take the product from start to finish and to use "your call" when a decision was needed. These are the calls. Reversal is cheap except for F-007 (Keychain service name) and the bundle id.

## 1. Platform scope

**Call:** 1.0 is macOS Apple Silicon only.

Windows and Intel Mac jobs were uploading files that were not installers. Shipping a smaller true product beats four download buttons. The site and CI match that. `build.rs` can compile the Swift helper for `x86_64-apple-macos13.0` later; that path was not linked or run.

## 2. Speech engine

**Call:** Apple `SFSpeechRecognizer`, on device, English (US). Parakeet and whisper.cpp stay unbuilt.

No benchmark was run. There is no labeled audio set in the repo, and FluidAudio is not a dependency. Inventing a WER number would violate the audit rules. Apple Speech is the engine that already ran in this codebase. Forcing `requiresOnDeviceRecognition` is what makes the offline sentence true. If on-device English is not installed, dictation fails with a sentence that says so.

## 3. Live typing

**Call:** off by default. One insertion when you stop.

Partials from `SFSpeechRecognizer` are rewritten. Typing each rewrite means backspaces in someone else's app: undo stacks, the user's own keystrokes, and terminals. The setting is still there for people who want it. Newlines are never typed, because Return sends a Slack message and submits a shell line.

## 4. History

**Call:** on, local, disclosed, deletable.

A dictation app that forgets every sentence is worse. The bug was the document that claimed history was opt-in. `history.json` is mode 0600. Clear History deletes the file.

## 5. Smart routing and snippets

**Call:** keep smart routing as a cleanup-prompt bias. It does nothing while cleanup is off. Snippets are not a feature. The fake examples are labeled as examples.

## 6. Skills that were not vendored

gstack, rea, and caveman were not copied into this repo. gstack is a Claude Code toolkit, rea is a reverse-engineering toolchain, caveman is a token-compression voice. Vendoring them would add unrelated licenses and tens of thousands of files to a dictation app. The review used their actual method: role-labeled findings, evidence from the built object file, and no padded praise. The report is in normal sentences because the audit specified a report format.
