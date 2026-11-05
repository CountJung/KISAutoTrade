use std::path::Path;

use anyhow::{Context, Result};

pub(super) async fn atomic_write(path: &Path, content: &str) -> Result<()> {
    let path = path.to_path_buf();
    let content = content.to_string();
    tokio::task::spawn_blocking(move || durable_atomic_write(&path, &content, false))
        .await
        .context("JSON 원자 저장 task 실패")?
}

/// 동기 컨텍스트(앱 시작 migration)용 민감 설정 저장.
pub(super) fn atomic_write_private_sync(path: &Path, content: &str) -> Result<()> {
    durable_atomic_write(path, content, true)
}

/// Make the original path's absence durable before a dependent transaction marker is removed.
pub(super) fn durable_remove_file(path: &Path) -> Result<()> {
    #[cfg(not(windows))]
    {
        remove_file_using_sync(path, sync_parent)
    }
    #[cfg(windows)]
    {
        // Windows has no Unix directory fsync. A write-through rename commits the original
        // name's absence; a leftover private tombstone cannot be loaded as active config.
        let tombstone = path.with_file_name(format!(".database-removed-{}", uuid::Uuid::new_v4()));
        replace_and_sync(path, &tombstone)?;
        std::fs::remove_file(&tombstone).context("Windows DB 삭제 tombstone 정리 실패")
    }
}

#[cfg(not(windows))]
fn remove_file_using_sync(path: &Path, sync: impl FnOnce(&Path) -> Result<()>) -> Result<()> {
    std::fs::remove_file(path)?;
    sync(path)
}

#[cfg(not(windows))]
fn sync_parent(path: &Path) -> Result<()> {
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    std::fs::File::open(parent)?.sync_all()?;
    Ok(())
}

fn durable_atomic_write(path: &Path, content: &str, private: bool) -> Result<()> {
    use std::io::Write;

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("document.json");
    let temp = path.with_file_name(format!(".{file_name}.{}.tmp", uuid::Uuid::new_v4()));
    let backup = path.with_file_name(format!("{file_name}.bak"));
    let result = (|| -> Result<()> {
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        if private {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        // Windows inherits the parent ACL; Unix alone has a mode argument.
        #[cfg(not(unix))]
        let _ = private;
        let mut file = options.open(&temp)?;
        file.write_all(content.as_bytes())?;
        file.sync_all()?;
        drop(file);

        if path.exists() {
            std::fs::copy(path, &backup)?;
            sync_backup(&backup)?;
        }
        replace_and_sync(&temp, path)?;
        Ok(())
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&temp);
    }
    result
}

fn sync_backup(path: &Path) -> std::io::Result<()> {
    // FlushFileBuffers needs a writable handle on Windows. Unix fsync also
    // supports read-only handles, preserving replacement of read-only files.
    #[cfg(windows)]
    let file = std::fs::OpenOptions::new().write(true).open(path)?;
    #[cfg(not(windows))]
    let file = std::fs::File::open(path)?;
    file.sync_all()
}

#[cfg(not(windows))]
fn replace_and_sync(source: &Path, destination: &Path) -> Result<()> {
    std::fs::rename(source, destination)?;
    sync_parent(destination)
}

#[cfg(windows)]
fn replace_and_sync(source: &Path, destination: &Path) -> Result<()> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::{
        MoveFileExW, MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH,
    };

    fn wide_path(path: &Path) -> Result<Vec<u16>> {
        let parent = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        // Canonicalize only the existing parent, preserving replacement of the
        // destination entry itself and obtaining a verbatim path for long paths.
        let absolute = std::fs::canonicalize(parent)?
            .join(path.file_name().context("JSON 저장 파일 이름이 없습니다")?);
        let mut wide: Vec<u16> = absolute.as_os_str().encode_wide().collect();
        anyhow::ensure!(!wide.contains(&0), "저장 경로에 NUL을 포함할 수 없습니다");
        wide.push(0);
        Ok(wide)
    }
    let source = wide_path(source)?;
    let destination = wide_path(destination)?;
    // Both files are in the same directory. Do not allow copy/delete fallback.
    // Windows has no Unix-style directory fsync; request write-through replacement.
    // SAFETY: both UTF-16 buffers are NUL-terminated and live for the call.
    let moved = unsafe {
        MoveFileExW(
            source.as_ptr(),
            destination.as_ptr(),
            MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
        )
    };
    if moved == 0 {
        return Err(std::io::Error::last_os_error()).context("Windows JSON 원자 교체 실패");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    struct TestDir(PathBuf);

    impl TestDir {
        fn new() -> Self {
            let root = std::env::temp_dir().join(format!("kis-atomic-{}", uuid::Uuid::new_v4()));
            std::fs::create_dir_all(&root).expect("test directory");
            Self(root)
        }

        fn assert_no_temps(&self) {
            for entry in std::fs::read_dir(&self.0).expect("test entries") {
                assert!(!entry
                    .unwrap()
                    .file_name()
                    .to_string_lossy()
                    .ends_with(".tmp"));
            }
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn durable_removal_clears_the_original_path() {
        let dir = TestDir::new();
        let path = dir.0.join("removed.json");
        atomic_write_private_sync(&path, "private synthetic data").unwrap();
        durable_remove_file(&path).unwrap();
        assert!(!path.exists());
        assert!(std::fs::read_dir(&dir.0).unwrap().next().is_none());
    }

    #[cfg(not(windows))]
    #[test]
    fn removal_reports_parent_sync_failure_after_unlink() {
        let dir = TestDir::new();
        let path = dir.0.join("removed.json");
        atomic_write_private_sync(&path, "private synthetic data").unwrap();
        let result = remove_file_using_sync(&path, |deleted| {
            assert_eq!(deleted, path);
            assert!(!deleted.exists(), "unlink must precede the parent flush");
            anyhow::bail!("synthetic parent sync failure")
        });
        assert!(result.is_err());
        assert!(!path.exists());
    }

    #[test]
    fn repeated_replacement_keeps_the_immediately_previous_backup() {
        let dir = TestDir::new();
        // Exercise Windows long-path support as well as Unicode and spaces.
        let nested = dir.0.join("한글 공백").join("nested".repeat(35));
        let path = nested.join("설정.json");
        let backup = nested.join("설정.json.bak");
        durable_atomic_write(&path, "first", false).expect("first save");
        assert!(!backup.exists());
        durable_atomic_write(&path, "second", false).expect("replace first");
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "second");
        assert_eq!(std::fs::read_to_string(&backup).unwrap(), "first");
        durable_atomic_write(&path, "third", false).expect("replace second");
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "third");
        assert_eq!(std::fs::read_to_string(&backup).unwrap(), "second");
        assert!(std::fs::read_dir(nested).unwrap().all(|entry| !entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .ends_with(".tmp")));
    }

    #[test]
    fn backup_failure_preserves_primary_and_cleans_staged_content() {
        let dir = TestDir::new();
        let path = dir.0.join("records.json");
        durable_atomic_write(&path, "original", false).expect("first save");
        std::fs::create_dir(dir.0.join("records.json.bak")).unwrap();
        assert!(durable_atomic_write(&path, "replacement", false).is_err());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "original");
        dir.assert_no_temps();
    }

    #[cfg(windows)]
    #[test]
    fn locked_destination_preserves_content_and_allows_retry_after_release() {
        use std::os::windows::fs::OpenOptionsExt;
        use windows_sys::Win32::Storage::FileSystem::{FILE_SHARE_READ, FILE_SHARE_WRITE};

        let dir = TestDir::new();
        let path = dir.0.join("records.json");
        durable_atomic_write(&path, "original", false).expect("first save");
        // Permit reads/copy but deny delete sharing, forcing replacement to fail.
        let lock = std::fs::OpenOptions::new()
            .read(true)
            .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
            .open(&path)
            .unwrap();
        assert!(durable_atomic_write(&path, "replacement", false).is_err());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "original");
        assert_eq!(
            std::fs::read_to_string(dir.0.join("records.json.bak")).unwrap(),
            "original"
        );
        dir.assert_no_temps();
        drop(lock);
        durable_atomic_write(&path, "replacement", false).expect("retry after unlock");
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "replacement");
    }

    #[cfg(unix)]
    #[test]
    fn read_only_unix_primary_can_be_replaced_in_a_writable_directory() {
        use std::os::unix::fs::PermissionsExt;

        let dir = TestDir::new();
        let path = dir.0.join("read-only.json");
        durable_atomic_write(&path, "original", false).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o444)).unwrap();
        durable_atomic_write(&path, "replacement", false).expect("read-only Unix replacement");
        let backup = dir.0.join("read-only.json.bak");
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "replacement");
        assert_eq!(std::fs::read_to_string(&backup).unwrap(), "original");
        assert_eq!(
            std::fs::metadata(backup).unwrap().permissions().mode() & 0o777,
            0o444
        );
    }

    #[cfg(unix)]
    #[test]
    fn private_sync_replacements_keep_owner_only_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let dir = TestDir::new();
        let path = dir.0.join("private.json");
        atomic_write_private_sync(&path, "first").unwrap();
        atomic_write_private_sync(&path, "second").unwrap();
        for file in [&path, &dir.0.join("private.json.bak")] {
            assert_eq!(
                std::fs::metadata(file).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }
    }
}
