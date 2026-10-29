# Changelog

## Unreleased

- macOS speech now requires on-device recognition and refuses to start if English (US) on-device recognition is unavailable.
- Windows and Intel Mac installers are no longer advertised or produced. The old CI jobs wrote text files named like installers.
- Live typing is off by default. The finished transcript is typed once.
- Transcripts are not copied to the clipboard when Accessibility is missing. They stay in History, and the bubble says why.
- Password fields are detected and refused.
- Cleanup calls refuse redirects, non-localhost HTTP, keys inside URLs, oversized replies, and chatbot-style replies. The transcript is delimited so the model is told not to follow instructions inside it.
- History and settings are written atomically. History is mode 0600 and Clear History deletes the file.
- A panic writes `~/Library/Logs/Utter/crash.log` and does not send it anywhere.

## 0.1.0

- First desktop build: hotkey, Apple speech helper, Studio window, optional cleanup, local history.
