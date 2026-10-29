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

#[cfg(test)]
mod tests {
    use super::*;

    fn unique_dir() -> std::path::PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir =
            std::env::temp_dir().join(format!("utter-persist-test-{}-{nanos}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn atomic_write_creates_and_overwrites() {
        let dir = unique_dir();
        let path = dir.join("config.toml");

        atomic_write(&path, "first").unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "first");

        // Overwriting replaces the content and leaves no temp file behind.
        atomic_write(&path, "second").unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "second");
        assert!(!path.with_file_name("config.toml.tmp").exists());

        std::fs::remove_dir_all(&dir).ok();
    }

    #[cfg(unix)]
    #[test]
    fn atomic_write_sets_0600() {
        use std::os::unix::fs::PermissionsExt;
        let dir = unique_dir();
        let path = dir.join("history.json");
        atomic_write(&path, "[]").unwrap();
        let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600, "transcripts must not be world/group readable");
        std::fs::remove_dir_all(&dir).ok();
    }
}
