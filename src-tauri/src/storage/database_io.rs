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

pub(super) async fn atomic_write_private(path: &Path, content: &str) -> Result<()> {
    #[cfg(unix)]
    {
        let path = path.to_path_buf();
        let content = content.to_string();
        tokio::task::spawn_blocking(move || durable_atomic_write(&path, &content, true))
            .await
            .context("민감 설정 저장 task 실패")??;
        Ok(())
    }

    #[cfg(not(unix))]
    atomic_write(path, content).await
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
        let mut file = options.open(&temp)?;
        file.write_all(content.as_bytes())?;
        file.sync_all()?;

        if path.exists() {
            std::fs::copy(path, &backup)?;
            std::fs::File::open(&backup)?.sync_all()?;
        }
        std::fs::rename(&temp, path)?;
        if let Some(parent) = path.parent() {
            std::fs::File::open(parent)?.sync_all()?;
        }
        Ok(())
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&temp);
    }
    result
}
