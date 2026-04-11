use anyhow::Result;
use std::path::{Path, PathBuf};
use tracing_appender::{non_blocking, rolling};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};

/// 로그 타임스탬프를 시스템 로컬 시간(KST = UTC+9)으로 출력하는 타이머
///
/// 기본 `tracing-subscriber` 타이머는 UTC(`Z`)를 사용하므로
/// `chrono::Local::now()`를 이용한 커스텀 타이머로 교체한다.
#[derive(Clone, Copy)]
struct LocalTimer;

impl tracing_subscriber::fmt::time::FormatTime for LocalTimer {
    fn format_time(
        &self,
        w: &mut tracing_subscriber::fmt::format::Writer<'_>,
    ) -> std::fmt::Result {
        write!(w, "{}", chrono::Local::now().format("%Y-%m-%dT%H:%M:%S%.6f%:z"))
    }
}

/// 로그 엔트리 (IPC로 프론트엔드에 전달)
#[derive(Debug, Clone, serde::Serialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level:     String,
    pub target:    String,
    pub message:   String,
}

/// tracing 포맷 한 줄 파싱
/// 형식: "YYYY-MM-DDTHH:MM:SS.nnnnnZ  LEVEL target: message"
fn parse_log_line(line: &str) -> Option<LogEntry> {
    let mut it = line.splitn(2, "  ");
    let timestamp = it.next()?.trim().to_string();
    let rest = it.next()?.trim();

    let mut it2 = rest.splitn(2, ' ');
    let level = it2.next()?.trim().to_string();
    match level.as_str() {
        "INFO" | "DEBUG" | "WARN" | "ERROR" | "TRACE" => {}
        _ => return None,
    }

    let target_msg = it2.next()?.trim();
    let (target, message) = match target_msg.split_once(": ") {
        Some((t, m)) => (t.trim().to_string(), m.to_string()),
        None => (String::new(), target_msg.to_string()),
    };

    Some(LogEntry { timestamp, level, target, message })
}

/// 오늘 app.log 파일에서 최근 `count`줄 읽기
/// 오늘 파일이 없거나 비어 있으면 가장 최근 app.log.* 파일로 자동 폴백
pub fn read_recent_entries(log_dir: &Path, count: usize) -> Vec<LogEntry> {
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let today_file = log_dir.join(format!("app.log.{}", today));

    // 1) 오늘 파일 시도
    if let Some(entries) = read_log_file(&today_file, count) {
        if !entries.is_empty() {
            return entries;
        }
    }

    // 2) 오늘 파일이 없거나 비어있으면 가장 최근 로그 파일로 폴백
    //    (자정 직후 새 파일 생성 전 공백 상태 방지)
    if let Some(fallback) = find_most_recent_log_file(log_dir) {
        if fallback != today_file {
            tracing::debug!(
                "오늘 로그 파일 없음 — 폴백: {:?}",
                fallback.file_name()
            );
        }
        if let Some(entries) = read_log_file(&fallback, count) {
            return entries;
        }
    }

    vec![]
}

/// 단일 log 파일에서 최근 `count`줄 파싱 (파일 없으면 None, 비어있으면 Some(vec![]))
fn read_log_file(path: &std::path::Path, count: usize) -> Option<Vec<LogEntry>> {
    let content = std::fs::read_to_string(path).ok()?;
    if content.is_empty() {
        return Some(vec![]);
    }
    let mut entries: Vec<LogEntry> = content
        .lines()
        .rev()
        .take(count)
        .filter_map(parse_log_line)
        .collect();
    entries.reverse();
    Some(entries)
}

/// logs 디렉토리에서 수정 시간 기준 가장 최근 app.log.YYYY-MM-DD 파일 반환
fn find_most_recent_log_file(log_dir: &Path) -> Option<std::path::PathBuf> {
    let mut files: Vec<(std::path::PathBuf, std::time::SystemTime)> =
        std::fs::read_dir(log_dir)
            .ok()?
            .filter_map(|e| {
                let e = e.ok()?;
                let path = e.path();
                let name = path.file_name()?.to_str()?;
                if !name.starts_with("app.log.") {
                    return None;
                }
                let modified = e.metadata().ok()?.modified().ok()?;
                Some((path, modified))
            })
            .collect();
    // 수정 시간 내림차순(최신 먼저)
    files.sort_by(|a, b| b.1.cmp(&a.1));
    files.into_iter().map(|(p, _)| p).next()
}

/// 로그 설정 (JSON 직렬화 가능)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LogConfig {
    /// 보관 기간 (일). 기본 5
    pub retention_days: u32,
    /// 파일 최대 합산 용량 (MB). 기본 100
    pub max_size_mb: u64,
    /// KIS API 진단 로그: true 시 요청 파라미터·응답 전체를 INFO로 기록
    #[serde(default)]
    pub api_debug: bool,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self { retention_days: 5, max_size_mb: 100, api_debug: false }
    }
}

impl LogConfig {
    const CONFIG_FILE: &'static str = "log_config.json";

    pub fn load_sync(log_dir: &Path) -> Self {
        let path = log_dir.join(Self::CONFIG_FILE);
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save_sync(&self, log_dir: &Path) -> Result<()> {
        let path = log_dir.join(Self::CONFIG_FILE);
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }
}

/// 로그 시스템 초기화
/// - 경로: `{cwd}/logs/`
/// - app.YYYY-MM-DD.log: INFO 이상, daily rolling
/// - error.YYYY-MM-DD.log: WARN 이상, daily rolling
/// - 시작 시 보관 기간 초과 / 용량 초과 파일 자동 정리
pub fn init(log_dir: &Path, cfg: &LogConfig) -> Result<()> {
    std::fs::create_dir_all(log_dir)?;

    // 기존 로그 정리
    cleanup(log_dir, cfg);

    // app.log - INFO 이상
    let app_file = rolling::daily(log_dir, "app.log");
    let (app_writer, _app_guard) = non_blocking(app_file);
    let app_layer = fmt::layer()
        .with_timer(LocalTimer)
        .with_writer(app_writer)
        .with_ansi(false)
        .with_filter(EnvFilter::new("info"));

    // error.log - WARN 이상
    let error_file = rolling::daily(log_dir, "error.log");
    let (error_writer, _error_guard) = non_blocking(error_file);
    let error_layer = fmt::layer()
        .with_timer(LocalTimer)
        .with_writer(error_writer)
        .with_ansi(false)
        .with_filter(EnvFilter::new("warn"));

    // 콘솔 출력 (RUST_LOG 환경변수 또는 debug 기본값)
    let console_layer = fmt::layer()
        .with_timer(LocalTimer)
        .with_filter(EnvFilter::from_default_env().add_directive(
            "auto_condition_trade_lib=debug".parse().unwrap(),
        ));

    tracing_subscriber::registry()
        .with(app_layer)
        .with(error_layer)
        .with(console_layer)
        .init();

    // guard를 leak하여 앱 수명 동안 유지 (단일 프로세스이므로 안전)
    std::mem::forget(_app_guard);
    std::mem::forget(_error_guard);

    Ok(())
}

/// 로그 정리:
/// 1. `retention_days`보다 오래된 `.log` 파일 삭제
/// 2. 전체 합산 용량이 `max_size_mb`를 초과하면 오래된 파일부터 삭제
pub fn cleanup(log_dir: &Path, cfg: &LogConfig) {
    let cutoff = chrono::Local::now()
        - chrono::Duration::days(cfg.retention_days as i64);

    let mut log_files: Vec<(PathBuf, std::fs::Metadata)> = std::fs::read_dir(log_dir)
        .into_iter()
        .flatten()
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            // tracing-appender daily 형식: "app.log.YYYY-MM-DD" / "error.log.YYYY-MM-DD"
            // path.extension()은 마지막 '.' 이후를 반환하므로 날짜 문자열이 나옴 — 사용 불가.
            // 파일명 starts_with 로 정확히 판별 (log_config.json, macOS ._* 리소스 포크 제외)
            let name = path.file_name()?.to_str()?;
            if !(name.starts_with("app.log.") || name.starts_with("error.log.")) {
                return None;
            }
            let meta = std::fs::metadata(&path).ok()?;
            Some((path, meta))
        })
        .collect();

    // 수정 시간 기준 오름차순 정렬 (오래된 것 먼저)
    log_files.sort_by_key(|(_, m)| {
        m.modified().unwrap_or(std::time::UNIX_EPOCH)
    });

    // 1. 보관 기간 초과 파일 삭제
    log_files.retain(|(path, meta)| {
        let modified = meta.modified().unwrap_or(std::time::UNIX_EPOCH);
        let modified_dt: chrono::DateTime<chrono::Local> = modified.into();
        if modified_dt < cutoff {
            let _ = std::fs::remove_file(path);
            return false;
        }
        true
    });

    // 2. 용량 초과 시 오래된 파일부터 삭제
    let max_bytes = cfg.max_size_mb * 1024 * 1024;
    let mut total: u64 = log_files.iter().map(|(_, m)| m.len()).sum();

    for (path, meta) in &log_files {
        if total <= max_bytes { break; }
        if std::fs::remove_file(path).is_ok() {
            total = total.saturating_sub(meta.len());
        }
    }
}

/// 로그 디렉토리 경로 결정
/// - 개발(cargo run): `{cwd}/logs/`
/// - 배포: `{app_data_dir}/logs/` 를 fallback으로 사용할 경우 호출측에서 선택
pub fn default_log_dir() -> PathBuf {
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("logs")
}
