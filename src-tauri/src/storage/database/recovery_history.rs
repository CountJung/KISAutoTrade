//! Bounded recovery-only history scan; never credentials, backup files, or JSON fallback.
use super::*;
use serde::de::DeserializeOwned;

const MAX_HISTORY_DOCUMENTS: usize = 2_048;
const MAX_HISTORY_BYTES: u64 = 16 * 1024 * 1024;
const MAX_HISTORY_RECORDS: usize = 50_000;
const MAX_ENTRIES: usize = 20_000;

fn history_key(key: &str, category: &str) -> Result<bool> {
    let filename = format!("{category}.json");
    if !key.ends_with(&format!("/{filename}")) {
        return Ok(false);
    }
    let parts: Vec<_> = key.split('/').collect();
    anyhow::ensure!(
        parts.len() == 5 && parts[0] == category,
        "복구 이력 경로 구조를 확인할 수 없습니다."
    );
    let date = format!("{}-{}-{}", parts[1], parts[2], parts[3]);
    let parsed = chrono::NaiveDate::parse_from_str(&date, "%Y-%m-%d")?;
    anyhow::ensure!(
        parsed.to_string() == date,
        "복구 이력 날짜 경로가 올바르지 않습니다."
    );
    Ok(true)
}
fn check_size(count: usize, bytes: u64) -> Result<()> {
    anyhow::ensure!(
        count <= MAX_HISTORY_DOCUMENTS && bytes <= MAX_HISTORY_BYTES,
        "보관된 복구 대조 이력이 2,048문서 또는 총 16MiB 한도를 초과하여 보류합니다."
    );
    Ok(())
}
async fn local_paths(base: &Path, category: &str) -> Result<Vec<PathBuf>> {
    let mut stack = vec![base.join(category)];
    let mut paths = Vec::new();
    let mut bytes = 0u64;
    let mut entries = 0;
    while let Some(directory) = stack.pop() {
        let mut reader = match fs::read_dir(&directory).await {
            Ok(reader) => reader,
            Err(e)
                if e.kind() == std::io::ErrorKind::NotFound && directory == base.join(category) =>
            {
                continue
            }
            Err(e) => return Err(e.into()),
        };
        while let Some(entry) = reader.next_entry().await? {
            entries += 1;
            anyhow::ensure!(entries <= MAX_ENTRIES, "복구 이력 디렉터리 항목 한도 초과");
            let kind = entry.file_type().await?;
            anyhow::ensure!(
                !kind.is_symlink(),
                "복구 이력의 symlink는 검증할 수 없습니다."
            );
            if kind.is_dir() {
                anyhow::ensure!(
                    entry.path().strip_prefix(base)?.components().count() <= 4,
                    "복구 이력 디렉터리 깊이 초과"
                );
                stack.push(entry.path());
            } else if kind.is_file() {
                let filename = format!("{category}.json");
                let name = entry.file_name();
                let name = name.to_string_lossy();
                if name == format!("{filename}.bak")
                    || name.starts_with(&format!("{filename}.corrupt-"))
                {
                    // A known history primary existed. Its absence is evidence of loss,
                    // never evidence that this provider ID has not been consumed.
                    let primary = entry.path().with_file_name(&filename);
                    if history_key(&document_key(base, &primary)?, category)? {
                        let metadata = fs::symlink_metadata(&primary).await
                            .map_err(|_| anyhow!("복구 이력의 backup·격리 흔적이 있으나 primary가 유실되어 보류합니다."))?;
                        anyhow::ensure!(
                            metadata.is_file() && !metadata.file_type().is_symlink(),
                            "복구 이력 primary의 파일 유형을 검증할 수 없습니다."
                        );
                    }
                    continue;
                }
                if entry.path().extension().and_then(|s| s.to_str()) != Some("json") {
                    continue;
                }
                let key = document_key(base, &entry.path())?;
                if !history_key(&key, category)? {
                    continue;
                }
                bytes = bytes
                    .checked_add(entry.metadata().await?.len())
                    .ok_or_else(|| anyhow!("복구 이력 크기 초과"))?;
                paths.push(entry.path());
                check_size(paths.len(), bytes)?;
            }
        }
    }
    Ok(paths)
}
async fn managed_paths(base: &Path, category: &str) -> Result<Vec<PathBuf>> {
    let Some(manager) = DATABASE_MANAGER.get() else {
        return local_paths(base, category).await;
    };
    let _operation = manager.operation_lock.lock().await;
    if manager.config.read().await.active_backend == StorageBackend::Json {
        return local_paths(base, category).await;
    }
    anyhow::ensure!(
        base == manager.data_dir,
        "복구 이력의 관리 저장소 경계가 다릅니다."
    );
    let prefix = format!("{category}/%");
    let rows: Vec<(String, i64)> = match manager.pool().await? {
        DatabasePool::Postgresql(pool) => sqlx::query_as(
            "SELECT document_key, size_bytes FROM kisautotrade_documents WHERE document_key LIKE $1 ORDER BY document_key LIMIT 2049")
            .bind(prefix).fetch_all(&pool).await?,
        DatabasePool::Mariadb(pool) => sqlx::query_as(
            "SELECT document_key, size_bytes FROM kisautotrade_documents WHERE document_key LIKE ? ORDER BY document_key LIMIT 2049")
            .bind(prefix).fetch_all(&pool).await?,
    };
    // Count every category document conservatively before excluding live ledgers.
    check_size(rows.len(), 0)?;
    let mut paths = Vec::new();
    let mut bytes = 0u64;
    for (key, size) in rows {
        if !history_key(&key, category)? {
            continue;
        }
        anyhow::ensure!(size >= 0, "복구 이력 크기 메타데이터 오류");
        bytes = bytes
            .checked_add(size as u64)
            .ok_or_else(|| anyhow!("복구 이력 크기 초과"))?;
        paths.push(safe_export_path(base, &key)?);
        check_size(paths.len(), bytes)?;
    }
    Ok(paths)
}
pub(crate) async fn records<T: DeserializeOwned>(base: &Path, category: &str) -> Result<Vec<T>> {
    anyhow::ensure!(
        matches!(category, "orders" | "trades"),
        "복구 조회 category가 아닙니다."
    );
    let paths = managed_paths(base, category).await?;
    let mut result = Vec::new();
    let mut bytes = 0u64;
    for path in paths {
        let content = read_managed_json(&path)
            .await?
            .ok_or_else(|| anyhow!("대조 중 복구 이력이 유실되었습니다."))?;
        bytes = bytes
            .checked_add(content.len() as u64)
            .ok_or_else(|| anyhow!("복구 이력 크기 초과"))?;
        check_size(0, bytes)?;
        let records: Vec<T> = serde_json::from_str(&content)?;
        anyhow::ensure!(
            result.len() + records.len() <= MAX_HISTORY_RECORDS,
            "보관된 복구 대조 이력이 50,000행을 초과하여 보류합니다."
        );
        result.extend(records);
    }
    Ok(result)
}
