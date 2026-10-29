# Claims vs reality

Checked against the code as it was at the start of the 2026-10-29 audit, then updated where this branch changed the claim.

| Claim | Verdict | Evidence |
|-------|---------|----------|
| "Everywhere" | Was false. Removed from the homepage and README. | Only the macOS path has an engine that does not return `EngineUnavailable`. |
| "Offline" / audio never leaves | Was conditional, now required in code, not proven on the wire. | `setRequiresOnDeviceRecognition:` is in the compiled `utter_transcribe.o`. No packet capture. |
| Windows and Intel downloads | False. The CI jobs wrote placeholder text files. | Old `build-release.yml` `echo "placeholder - not a real installer"`. Jobs removed. |
| FluidAudio + Parakeet | False. | `apple_silicon.rs` returns `NotImplemented`. The live engine is `macos_speech.rs`. |
| whisper.cpp on Intel and Windows | False. | `whisper_cpp.rs` returns `NotImplemented`. |
| No account, no telemetry | True for the desktop app. | No analytics SDK. Panic hook writes a local file only (`main.rs`). |
| Transcripts not saved unless you opt in | False. | `history.rs` `add_entry` on every non-empty final. Docs now say history is on. |
| Cleanup off by default | True. | `CleanupConfig::None` in `Settings::default`. |
| API keys not in the config file | True. | `set_keychain_secret` uses keyring service `com.utter.app`. |
| Text appears as you speak, under 300ms | Unmeasured, and live typing is now off. | The 300ms line was deleted from the homepage. No timing command was run. |
| ~10 MB install | Unmeasured. | Removed from the homepage. |
| Works in any app | Unverified. | `enigo` types into the focused app. Password fields are refused in code. Notes, Slack, Terminal, and the rest of the matrix were not run. |
| JS-disabled download lists four installers | Was true and harmful. | `noscript` now links only `app-macos-arm64.dmg`. |
