// insert/mod.rs — Text insertion into the focused application.
//
// Uses the `enigo` crate for cross-platform synthetic input:
//   macOS:   CGEvent + Accessibility API (same mechanism as Karabiner-Elements)
//   Windows: SendInput Win32 API
//
// macOS permission requirement:
//   The app must be granted Accessibility access in:
//   System Settings → Privacy & Security → Accessibility
//
//   On first use, macOS will prompt the user automatically (because
//   our entitlements.plist declares the accessibility exception).
//   If the user denies it, insert_text returns AppError::InsertFailed
//   and the transcript is still available via the transcript_final event.
//
// WHY enigo and not clipboard (Cmd+V):
//   Clipboard-based insertion clobbers the user's clipboard.
//   CGEvent synthetic input works in any app without touching the clipboard.

use crate::error::{AppError, Result};
use enigo::{Direction, Enigo, Key, Keyboard, Settings as EnigoSettings};

#[cfg(target_os = "macos")]
pub fn is_trusted() -> bool {
    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        fn AXIsProcessTrusted() -> bool;
    }
    unsafe { AXIsProcessTrusted() }
}

#[cfg(not(target_os = "macos"))]
pub fn is_trusted() -> bool {
    true
}

#[cfg(target_os = "macos")]
pub fn request_permission() -> bool {
    // Open macOS System Settings directly to Privacy & Security -> Accessibility
    let _ = std::process::Command::new("open")
        .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
        .spawn();

    #[link(name = "ApplicationServices", kind = "framework")]
    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
        fn AXIsProcessTrustedWithOptions(options: *const std::ffi::c_void) -> bool;
        static kAXTrustedCheckOptionPrompt: *const std::ffi::c_void;
        static kCFBooleanTrue: *const std::ffi::c_void;
        fn CFDictionaryCreate(
            allocator: *const std::ffi::c_void,
            keys: *const *const std::ffi::c_void,
            values: *const *const std::ffi::c_void,
            numValues: isize,
            keyCallBacks: *const std::ffi::c_void,
            valueCallBacks: *const std::ffi::c_void,
        ) -> *const std::ffi::c_void;
        fn CFRelease(cf: *const std::ffi::c_void);
    }

    unsafe {
        let keys = [kAXTrustedCheckOptionPrompt];
        let values = [kCFBooleanTrue];
        let dict = CFDictionaryCreate(
            std::ptr::null(),
            keys.as_ptr(),
            values.as_ptr(),
            1,
            std::ptr::null(),
            std::ptr::null(),
        );
        let trusted = AXIsProcessTrustedWithOptions(dict);
        if !dict.is_null() {
            CFRelease(dict);
        }
        trusted
    }
}

#[cfg(not(target_os = "macos"))]
pub fn request_permission() -> bool {
    true
}

/// Insert `text` at the current cursor position in the focused application.
pub fn insert_text(text: &str) -> Result<()> {
    if text.is_empty() {
        return Ok(());
    }

    // If accessibility permissions are not granted, copy to clipboard as graceful fallback
    if !is_trusted() {
        log::warn!("Accessibility not granted. Copying final transcript to clipboard as fallback.");
        if let Ok(mut clipboard) = arboard::Clipboard::new() {
            let _ = clipboard.set_text(text);
        }
        return Ok(());
    }

    let mut enigo = Enigo::new(&EnigoSettings::default()).map_err(|e| {
        AppError::Internal(format!("enigo init: {e}"))
    })?;

    enigo.text(text).map_err(|_| AppError::InsertFailed)?;

    log::debug!("insert_text: directly inserted {} chars via keyboard synthetic input", text.len());
    Ok(())
}

/// Real-time streaming insertion: compares `old` vs `new`, sends Backspaces for diverged characters, and types the new delta suffix.
pub fn insert_delta(old: &str, new: &str) -> Result<()> {
    if old == new {
        return Ok(());
    }

    // If accessibility is not granted, DO NOT attempt synthetic key events (prevents pop-up storms)
    if !is_trusted() {
        return Ok(());
    }

    let mut prefix_chars = 0;
    let mut old_chars = old.chars();
    let mut new_chars = new.chars();
    while let (Some(c1), Some(c2)) = (old_chars.next(), new_chars.next()) {
        if c1 == c2 {
            prefix_chars += 1;
        } else {
            break;
        }
    }
    
    let old_total_chars = old.chars().count();
    let num_backspaces = old_total_chars - prefix_chars;
    let suffix: String = new.chars().skip(prefix_chars).collect();

    let mut enigo = Enigo::new(&EnigoSettings::default()).map_err(|e| {
        AppError::Internal(format!("enigo init: {e}"))
    })?;

    for _ in 0..num_backspaces {
        let _ = enigo.key(Key::Backspace, Direction::Click);
    }

    if !suffix.is_empty() {
        let _ = enigo.text(&suffix);
    }

    log::debug!("insert_delta: {} backspaces, typed {} new chars", num_backspaces, suffix.len());
    Ok(())
}

