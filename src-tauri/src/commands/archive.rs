use super::*;

// ────────────────────────────────────────────────────────────────────
// 체결 기록 보관 설정
// ────────────────────────────────────────────────────────────────────

/// 체결 기록 보관 설정 (보관 기간, 최대 저장 용량)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeArchiveConfig {
    pub retention_days: u32, // 보관 기간 (일), 기본 90
    pub max_size_mb: u64,    // 최대 저장 용량 (MB), 기본 500
}

impl Default for TradeArchiveConfig {
    fn default() -> Self {
        Self {
            retention_days: 90,
            max_size_mb: 500,
        }
    }
}

impl TradeArchiveConfig {
    /// 저장 파일에서 로드, 없으면 기본값
    pub fn load_or_default(data_dir: &Path) -> Self {
        let path = data_dir.join("trade_archive_config.json");
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    /// 파일에 동기 저장
    pub fn save_sync(&self, data_dir: &Path) -> std::result::Result<(), String> {
        if let Some(parent) = data_dir.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        std::fs::create_dir_all(data_dir).map_err(|e| e.to_string())?;
        let path = data_dir.join("trade_archive_config.json");
        let content = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(&path, content).map_err(|e| e.to_string())
    }
}

/// 체결 기록 저장소 통계
#[derive(Debug, Serialize)]
pub struct TradeArchiveStats {
    pub total_files: u64,
    pub size_bytes: u64,
    pub oldest_date: Option<String>,
    pub newest_date: Option<String>,
}

pub fn collect_trade_archive_stats(data_dir: &Path) -> TradeArchiveStats {
    let day_dirs = collect_trade_day_dirs(data_dir);
    let mut total_files: u64 = 0;
    let mut size_bytes: u64 = 0;
    for (_, dir) in &day_dirs {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                if entry.path().is_file() {
                    total_files += 1;
                    size_bytes += entry.path().metadata().map(|m| m.len()).unwrap_or(0);
                }
            }
        }
    }
    let oldest_date = day_dirs
        .first()
        .map(|(d, _)| d.format("%Y-%m-%d").to_string());
    let newest_date = day_dirs
        .last()
        .map(|(d, _)| d.format("%Y-%m-%d").to_string());
    TradeArchiveStats {
        total_files,
        size_bytes,
        oldest_date,
        newest_date,
    }
}

/// 날짜별 trades 디렉토리 목록 수집 (trades/YYYY/MM/DD/)
fn collect_trade_day_dirs(data_dir: &Path) -> Vec<(chrono::NaiveDate, PathBuf)> {
    let trades_dir = data_dir.join("trades");
    if !trades_dir.exists() {
        return vec![];
    }
    let mut result = Vec::new();
    let Ok(year_entries) = std::fs::read_dir(&trades_dir) else {
        return result;
    };
    for year_entry in year_entries.flatten() {
        let year_path = year_entry.path();
        if !year_path.is_dir() {
            continue;
        }
        let Some(year_str) = year_entry.file_name().into_string().ok() else {
            continue;
        };
        let Ok(year) = year_str.parse::<i32>() else {
            continue;
        };
        let Ok(month_entries) = std::fs::read_dir(&year_path) else {
            continue;
        };
        for month_entry in month_entries.flatten() {
            let month_path = month_entry.path();
            if !month_path.is_dir() {
                continue;
            }
            let Some(month_str) = month_entry.file_name().into_string().ok() else {
                continue;
            };
            let Ok(month) = month_str.parse::<u32>() else {
                continue;
            };
            let Ok(day_entries) = std::fs::read_dir(&month_path) else {
                continue;
            };
            for day_entry in day_entries.flatten() {
                let day_path = day_entry.path();
                if !day_path.is_dir() {
                    continue;
                }
                let Some(day_str) = day_entry.file_name().into_string().ok() else {
                    continue;
                };
                let Ok(day) = day_str.parse::<u32>() else {
                    continue;
                };
                if let Some(date) = chrono::NaiveDate::from_ymd_opt(year, month, day) {
                    result.push((date, day_path));
                }
            }
        }
    }
    result.sort_by_key(|(d, _)| *d);
    result
}

/// 디렉토리 내 파일 총 크기 (바이트)
fn dir_size_bytes(path: &Path) -> u64 {
    let mut total = 0u64;
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let ep = entry.path();
            if ep.is_file() {
                total += ep.metadata().map(|m| m.len()).unwrap_or(0);
            } else if ep.is_dir() {
                total += dir_size_bytes(&ep);
            }
        }
    }
    total
}

/// 오래된 체결 기록 파일 정리 (보관 기간 초과 + 용량 초과)
/// lib.rs 시작 시 및 일일 정리 데몬에서 호출 가능하도록 pub
pub fn purge_old_trade_files(data_dir: &Path, cfg: &TradeArchiveConfig) {
    let cutoff =
        chrono::Local::now().date_naive() - chrono::Duration::days(cfg.retention_days as i64);

    let mut day_dirs = collect_trade_day_dirs(data_dir);

    // 보관 기간 초과 삭제
    let mut remaining: Vec<(chrono::NaiveDate, PathBuf)> = Vec::new();
    for (date, dir) in day_dirs.drain(..) {
        if date < cutoff {
            if let Err(e) = std::fs::remove_dir_all(&dir) {
                tracing::warn!("체결 기록 기간 정리 실패 ({:?}): {}", dir, e);
            } else {
                tracing::info!("체결 기록 정리: {} ({} 이전)", date, cutoff);
            }
        } else {
            remaining.push((date, dir));
        }
    }

    // 용량 초과 시 오래된 것부터 삭제
    let max_bytes = cfg.max_size_mb * 1024 * 1024;
    let mut total_size: u64 = remaining.iter().map(|(_, d)| dir_size_bytes(d)).sum();
    for (date, dir) in &remaining {
        if total_size <= max_bytes {
            break;
        }
        let sz = dir_size_bytes(dir);
        if let Err(e) = std::fs::remove_dir_all(dir) {
            tracing::warn!("체결 기록 용량 정리 실패 ({:?}): {}", dir, e);
        } else {
            total_size = total_size.saturating_sub(sz);
            tracing::info!("체결 기록 용량 정리: {} ({}MB 초과)", date, cfg.max_size_mb);
        }
    }
}

// ────────────────────────────────────────────────────────────────────
// 체결 기록 보관 설정 조회 / 변경 / 통계
// ────────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_trade_archive_config(state: State<'_, AppState>) -> CmdResult<TradeArchiveConfig> {
    Ok(state.trade_archive_config.read().await.clone())
}

#[derive(Debug, Deserialize)]
pub struct SetTradeArchiveConfigInput {
    pub retention_days: u32,
    pub max_size_mb: u64,
}

#[tauri::command]
pub async fn set_trade_archive_config(
    input: SetTradeArchiveConfigInput,
    state: State<'_, AppState>,
) -> CmdResult<TradeArchiveConfig> {
    let new_cfg = TradeArchiveConfig {
        retention_days: input.retention_days.clamp(1, 3650),
        max_size_mb: input.max_size_mb.clamp(50, 102400),
    };

    *state.trade_archive_config.write().await = new_cfg.clone();
    new_cfg.save_sync(&state.data_dir).map_err(|e| CmdError {
        code: "SAVE_ERR".into(),
        message: e,
    })?;

    // 즉시 정리 실행
    let data_dir = state.data_dir.clone();
    let cfg_clone = new_cfg.clone();
    tokio::task::spawn_blocking(move || purge_old_trade_files(&data_dir, &cfg_clone));

    tracing::info!(
        "체결 기록 보관 설정 변경: 보관 {}일, 최대 {}MB",
        new_cfg.retention_days,
        new_cfg.max_size_mb
    );

    Ok(new_cfg)
}

#[tauri::command]
pub async fn get_trade_archive_stats(state: State<'_, AppState>) -> CmdResult<TradeArchiveStats> {
    let data_dir = state.data_dir.clone();
    let stats = tokio::task::spawn_blocking(move || collect_trade_archive_stats(&data_dir))
        .await
        .map_err(|e| CmdError {
            code: "TASK_ERR".into(),
            message: e.to_string(),
        })?;

    Ok(stats)
}
