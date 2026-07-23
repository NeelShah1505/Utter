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
use enigo::{Enigo, Keyboard, Settings as EnigoSettings};

/// Insert `text` at the current cursor position in the focused application.
pub fn insert_text(text: &str) -> Result<()> {
    if text.is_empty() {
        return Ok(());
    }

    let mut enigo = Enigo::new(&EnigoSettings::default()).map_err(|e| {
        AppError::Internal(format!("enigo init: {e}"))
    })?;

    enigo.text(text).map_err(|_| AppError::InsertFailed)?;

    log::debug!("insert_text: inserted {} chars", text.len());
    Ok(())
}
