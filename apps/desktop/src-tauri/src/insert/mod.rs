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
use crate::logic::{self, plan_edit};
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
    extern "C" {
        fn AXIsProcessTrustedWithOptions(options: *const std::ffi::c_void) -> bool;
        static kAXTrustedCheckOptionPrompt: *const std::ffi::c_void;
    }

    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
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

/// True while the user is in a secure input (password field).
/// Typing there would inject keystrokes into a credential.
#[cfg(target_os = "macos")]
pub fn secure_input_active() -> bool {
    #[link(name = "Carbon", kind = "framework")]
    extern "C" {
        fn IsSecureEventInputEnabled() -> bool;
    }
    unsafe { IsSecureEventInputEnabled() }
}

#[cfg(not(target_os = "macos"))]
pub fn secure_input_active() -> bool {
    false
}

fn guard_typing() -> Result<()> {
    if secure_input_active() {
        return Err(AppError::Internal(
            "A password field is focused. Utter will not type into it. Move the cursor to a normal text field and dictate again. The text is in History.".into(),
        ));
    }
    if !is_trusted() {
        return Err(AppError::Accessibility);
    }
    Ok(())
}

/// Insert `text` at the current cursor position in the focused application.
/// Does not touch the clipboard.
pub fn insert_text(text: &str) -> Result<()> {
    let text = logic::sanitize_insertion(text);
    if text.is_empty() {
        return Ok(());
    }
    guard_typing()?;

    let mut enigo = Enigo::new(&EnigoSettings::default())
        .map_err(|e| AppError::Internal(format!("Could not start keyboard input: {e}")))?;

    enigo.text(&text).map_err(|_| AppError::InsertFailed)?;
    log::debug!("insert_text: typed {} chars", text.chars().count());
    Ok(())
}

/// Compare `old` vs `new`, send Backspace for the diverged characters, type the suffix.
pub fn insert_delta(old: &str, new: &str) -> Result<()> {
    let new = logic::sanitize_insertion(new);
    let plan = plan_edit(old, &new);
    if plan.backspaces == 0 && plan.suffix.is_empty() {
        return Ok(());
    }
    guard_typing()?;

    let mut enigo = Enigo::new(&EnigoSettings::default())
        .map_err(|e| AppError::Internal(format!("Could not start keyboard input: {e}")))?;

    for _ in 0..plan.backspaces {
        enigo
            .key(Key::Backspace, Direction::Click)
            .map_err(|_| AppError::InsertFailed)?;
    }

    if !plan.suffix.is_empty() {
        enigo
            .text(&plan.suffix)
            .map_err(|_| AppError::InsertFailed)?;
    }

    log::debug!(
        "insert_delta: {} backspaces, typed {} new chars",
        plan.backspaces,
        plan.suffix.chars().count()
    );
    Ok(())
}
