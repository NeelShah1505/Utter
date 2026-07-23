// insert/mod.rs — Text insertion into the focused application.
//
// After a transcript is finalised (and optionally cleaned), this module
// inserts the text wherever the user's cursor currently is.
//
// Per-OS implementations:
//   macOS:   CGEvent synthetic keyboard input (requires Accessibility permission)
//   Windows: SendInput Win32 API
//
// WHY synthetic keyboard events and not the clipboard:
//   The clipboard approach (copy text, Cmd+V) clobbers the user's clipboard.
//   Synthetic key events simulate typing character-by-character and work in
//   any app without touching the clipboard.
//
// Full implementation: Phase 2 (macOS) / Phase 3 (Windows).
// Stubs are in place below.

use crate::error::{AppError, Result};

/// Insert `text` into the currently focused application.
/// Errors if Accessibility (macOS) or equivalent (Windows) permission is denied.
pub fn insert_text(text: &str) -> Result<()> {
    if text.is_empty() {
        return Ok(());
    }

    #[cfg(target_os = "macos")]
    {
        insert_text_macos(text)
    }

    #[cfg(target_os = "windows")]
    {
        insert_text_windows(text)
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        log::warn!("insert_text: unsupported OS, doing nothing");
        Ok(())
    }
}

#[cfg(target_os = "macos")]
fn insert_text_macos(_text: &str) -> Result<()> {
    // TODO (Phase 2): Implement using CGEvent synthetic keyboard input.
    //
    // Approach:
    //   1. Check AXIsProcessTrustedWithOptions → if false, return AppError::Accessibility.
    //   2. For each Unicode codepoint in `text`, create a CGEventKeyDown/CGEventKeyUp
    //      pair with the CGEventSetString override to inject arbitrary Unicode.
    //   3. Post via CGEventPost(kCGHIDEventTap, event).
    //
    // This is the same approach used by Karabiner-Elements, Hammerspoon, and
    // the original Wispr Flow implementation.
    //
    // Reference: Apple TN2150 "Sending Complex Text Using CGEventKeyboardSetUnicodeString"
    log::error!("insert_text_macos: not yet implemented (Phase 2)");
    Err(AppError::Internal(
        "insert_text_macos not yet implemented — Phase 2".into(),
    ))
}

#[cfg(target_os = "windows")]
fn insert_text_windows(text: &str) -> Result<()> {
    // TODO (Phase 3): Implement using SendInput Win32 API.
    //
    // Approach:
    //   1. For each UTF-16 code unit in `text`, create an INPUT struct with
    //      type = INPUT_KEYBOARD, wVk = 0, wScan = codepoint, dwFlags = KEYEVENTF_UNICODE.
    //   2. Call SendInput() with the INPUT array.
    //
    // Reference: https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-sendinput
    log::error!("insert_text_windows: not yet implemented (Phase 3)");
    Err(AppError::Internal(
        "insert_text_windows not yet implemented — Phase 3".into(),
    ))
}
