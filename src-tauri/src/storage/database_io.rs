use std::path::Path;

use anyhow::{Context, Result};
use tokio::fs;

pub(super) async fn atomic_write(path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).await?;
    }
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("document.json");
    let temp = path.with_file_name(format!(".{file_name}.{}.tmp", uuid::Uuid::new_v4()));
    fs::write(&temp, content).await?;
    fs::rename(&temp, path).await.inspect_err(|_| {
        let _ = std::fs::remove_file(&temp);
    })?;
    Ok(())
}

pub(super) async fn atomic_write_private(path: &Path, content: &str) -> Result<()> {
    #[cfg(unix)]
    {
        use std::io::Write;
        use std::os::unix::fs::OpenOptionsExt;

        let path = path.to_path_buf();
        let content = content.to_string();
        tokio::task::spawn_blocking(move || -> Result<()> {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let file_name = path
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or("private.json");
            let temp = path.with_file_name(format!(".{file_name}.{}.tmp", uuid::Uuid::new_v4()));
            let result = (|| -> Result<()> {
                let mut file = std::fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .mode(0o600)
                    .open(&temp)?;
                file.write_all(content.as_bytes())?;
                file.sync_all()?;
                std::fs::rename(&temp, &path)?;
                if let Some(parent) = path.parent() {
                    std::fs::File::open(parent)?.sync_all()?;
                }
                Ok(())
            })();
            if result.is_err() {
                let _ = std::fs::remove_file(&temp);
            }
            result
        })
        .await
        .context("민감 설정 저장 task 실패")??;
        Ok(())
    }

    #[cfg(not(unix))]
    atomic_write(path, content).await
}
