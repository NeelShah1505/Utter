// persist.rs — Atomic config/history writes.
//
// Write a temp file in the same directory, fsync, then rename over the target.
// On Unix the file mode is 0600 so other local users cannot read transcripts.

use std::io::Write;
use std::path::Path;

pub fn atomic_write(path: &Path, data: &str) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut tmp_name = path
        .file_name()
        .map(|s| s.to_os_string())
        .unwrap_or_else(|| std::ffi::OsString::from("utter"));
    tmp_name.push(".tmp");
    let tmp = path.with_file_name(tmp_name);

    {
        let mut file = std::fs::File::create(&tmp)?;
        file.write_all(data.as_bytes())?;
        file.sync_all()?;
    }

    std::fs::rename(&tmp, path)?;
    restrict_mode(path)?;
    Ok(())
}

fn restrict_mode(path: &Path) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
    }
    #[cfg(not(unix))]
    {
        let _ = path;
    }
    Ok(())
}
