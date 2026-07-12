use std::{
    collections::HashMap,
    sync::{Arc, Mutex as StdMutex, OnceLock},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use reqwest::header::HeaderMap;
use serde::Serialize;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Default)]
pub struct RateLimitScheduler {
    groups: Arc<Mutex<HashMap<String, RateLimitGroupState>>>,
}

#[derive(Debug, Clone)]
struct RateLimitGroupState {
    min_interval: Duration,
    next_available_at: Instant,
    paused_until: Option<Instant>,
    /// 서버 rate limit 응답(Retry-After 등)으로 pause가 적용된 누적 횟수
    rate_limited_count: u64,
    last_success_epoch_ms: Option<u64>,
    last_failure_epoch_ms: Option<u64>,
    consecutive_failures: u64,
}

impl Default for RateLimitGroupState {
    fn default() -> Self {
        Self {
            min_interval: Duration::ZERO,
            next_available_at: Instant::now(),
            paused_until: None,
            rate_limited_count: 0,
            last_success_epoch_ms: None,
            last_failure_epoch_ms: None,
            consecutive_failures: 0,
        }
    }
}

/// 운영 가시성용 그룹별 rate limit 상태 스냅샷.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RateLimitGroupStatus {
    pub group: String,
    pub min_interval_ms: u64,
    /// 현재 서버 요청 pause가 남아 있으면 잔여 ms, 없으면 0
    pub paused_remaining_ms: u64,
    pub rate_limited_count: u64,
    pub last_success_epoch_ms: Option<u64>,
    pub last_failure_epoch_ms: Option<u64>,
    pub consecutive_failures: u64,
}

fn epoch_ms_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_millis() as u64)
        .unwrap_or(0)
}

impl RateLimitScheduler {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_min_intervals(
        intervals: impl IntoIterator<Item = (impl Into<String>, Duration)>,
    ) -> Self {
        let groups = intervals
            .into_iter()
            .map(|(group, min_interval)| {
                (
                    group.into(),
                    RateLimitGroupState {
                        min_interval,
                        ..RateLimitGroupState::default()
                    },
                )
            })
            .collect();
        Self {
            groups: Arc::new(Mutex::new(groups)),
        }
    }

    pub async fn set_min_interval(&self, group: impl Into<String>, interval: Duration) {
        let mut groups = self.groups.lock().await;
        groups.entry(group.into()).or_default().min_interval = interval;
    }

    pub async fn wait(&self, group: &str) {
        let delay = {
            let mut groups = self.groups.lock().await;
            let state = groups.entry(group.to_string()).or_default();
            let now = Instant::now();
            let available_at = state
                .paused_until
                .unwrap_or(state.next_available_at)
                .max(state.next_available_at);
            state.next_available_at = available_at + state.min_interval;
            available_at.saturating_duration_since(now)
        };

        if !delay.is_zero() {
            tokio::time::sleep(delay).await;
        }
    }

    pub async fn apply_response_headers(&self, group: &str, headers: &HeaderMap) {
        let delay = retry_after_delay(headers).or_else(|| rate_limit_reset_delay(headers));
        let exhausted = headers
            .get("X-RateLimit-Remaining")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.parse::<u64>().ok())
            == Some(0);

        let Some(delay) = delay.filter(|delay| exhausted || !delay.is_zero()) else {
            return;
        };

        self.pause(group, delay).await;
    }

    pub async fn pause(&self, group: &str, delay: Duration) {
        let mut groups = self.groups.lock().await;
        let state = groups.entry(group.to_string()).or_default();
        let paused_until = Instant::now() + delay;
        state.paused_until = Some(state.paused_until.unwrap_or(paused_until).max(paused_until));
        state.rate_limited_count = state.rate_limited_count.saturating_add(1);
    }

    /// 요청 결과를 운영 상태에 반영한다. 성공 시 연속 실패 카운터가 초기화된다.
    pub async fn record_outcome(&self, group: &str, success: bool) {
        let mut groups = self.groups.lock().await;
        let state = groups.entry(group.to_string()).or_default();
        let now = epoch_ms_now();
        if success {
            state.last_success_epoch_ms = Some(now);
            state.consecutive_failures = 0;
        } else {
            state.last_failure_epoch_ms = Some(now);
            state.consecutive_failures = state.consecutive_failures.saturating_add(1);
        }
    }

    /// 그룹별 운영 상태 스냅샷 (그룹 이름 정렬).
    pub async fn status_snapshot(&self) -> Vec<RateLimitGroupStatus> {
        let groups = self.groups.lock().await;
        let now = Instant::now();
        let mut statuses: Vec<RateLimitGroupStatus> = groups
            .iter()
            .map(|(group, state)| RateLimitGroupStatus {
                group: group.clone(),
                min_interval_ms: state.min_interval.as_millis() as u64,
                paused_remaining_ms: state
                    .paused_until
                    .map(|until| until.saturating_duration_since(now).as_millis() as u64)
                    .unwrap_or(0),
                rate_limited_count: state.rate_limited_count,
                last_success_epoch_ms: state.last_success_epoch_ms,
                last_failure_epoch_ms: state.last_failure_epoch_ms,
                consecutive_failures: state.consecutive_failures,
            })
            .collect();
        statuses.sort_by(|a, b| a.group.cmp(&b.group));
        statuses
    }

    #[cfg(test)]
    async fn snapshot_delay_ms(&self, group: &str) -> Option<u128> {
        let groups = self.groups.lock().await;
        let state = groups.get(group)?;
        state
            .paused_until
            .map(|until| until.saturating_duration_since(Instant::now()).as_millis())
    }
}

// ────────────────────────────────────────────────────────────────────
// process-wide 공유 registry — 짧게 생성되는 client가 같은 credential
// scope의 pacing/pause/운영 상태를 공유한다.
// ────────────────────────────────────────────────────────────────────

static SHARED_SCHEDULERS: OnceLock<StdMutex<HashMap<String, RateLimitScheduler>>> =
    OnceLock::new();

fn shared_registry() -> &'static StdMutex<HashMap<String, RateLimitScheduler>> {
    SHARED_SCHEDULERS.get_or_init(|| StdMutex::new(HashMap::new()))
}

/// scope key(예: `"toss|{base_url}|{client_id}"`)별로 스케줄러를 재사용한다.
/// 최초 접근 시에만 `init`으로 생성한다.
pub fn shared_scheduler(scope: &str, init: impl FnOnce() -> RateLimitScheduler) -> RateLimitScheduler {
    let mut registry = shared_registry()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    registry.entry(scope.to_string()).or_insert_with(init).clone()
}

/// 운영 상태 노출용: 등록된 모든 scope와 스케줄러 목록.
pub fn shared_scheduler_scopes() -> Vec<(String, RateLimitScheduler)> {
    let registry = shared_registry()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let mut scopes: Vec<(String, RateLimitScheduler)> = registry
        .iter()
        .map(|(scope, scheduler)| (scope.clone(), scheduler.clone()))
        .collect();
    scopes.sort_by(|a, b| a.0.cmp(&b.0));
    scopes
}

fn retry_after_delay(headers: &HeaderMap) -> Option<Duration> {
    headers
        .get("Retry-After")
        .and_then(|value| value.to_str().ok())
        .and_then(parse_delay_header)
}

fn rate_limit_reset_delay(headers: &HeaderMap) -> Option<Duration> {
    headers
        .get("X-RateLimit-Reset")
        .and_then(|value| value.to_str().ok())
        .and_then(parse_delay_header)
}

fn parse_delay_header(value: &str) -> Option<Duration> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }

    let seconds = value.parse::<u64>().ok()?;
    let now_epoch = SystemTime::now().duration_since(UNIX_EPOCH).ok()?.as_secs();

    if seconds > now_epoch {
        Some(Duration::from_secs(seconds - now_epoch))
    } else {
        Some(Duration::from_secs(seconds))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::header::{HeaderMap, HeaderValue};

    #[tokio::test]
    async fn applies_retry_after_to_group_pause() {
        let scheduler = RateLimitScheduler::new();
        let mut headers = HeaderMap::new();
        headers.insert("Retry-After", HeaderValue::from_static("2"));

        scheduler
            .apply_response_headers("toss:account", &headers)
            .await;

        let delay = scheduler.snapshot_delay_ms("toss:account").await.unwrap();
        assert!(delay > 1_000);
        assert!(delay <= 2_000);
    }

    #[tokio::test]
    async fn reserves_min_interval_between_requests() {
        let scheduler = RateLimitScheduler::new();
        scheduler
            .set_min_interval("kis:order", Duration::from_millis(20))
            .await;

        let start = Instant::now();
        scheduler.wait("kis:order").await;
        scheduler.wait("kis:order").await;

        assert!(start.elapsed() >= Duration::from_millis(18));
    }

    #[tokio::test]
    async fn status_snapshot_tracks_outcomes_and_rate_limits() {
        let scheduler = RateLimitScheduler::new();
        scheduler.record_outcome("toss:order", false).await;
        scheduler.record_outcome("toss:order", false).await;
        scheduler
            .pause("toss:order", Duration::from_secs(5))
            .await;

        let snapshot = scheduler.status_snapshot().await;
        let group = snapshot
            .iter()
            .find(|status| status.group == "toss:order")
            .expect("group status");
        assert_eq!(group.consecutive_failures, 2);
        assert_eq!(group.rate_limited_count, 1);
        assert!(group.paused_remaining_ms > 4_000);
        assert!(group.last_failure_epoch_ms.is_some());
        assert!(group.last_success_epoch_ms.is_none());

        scheduler.record_outcome("toss:order", true).await;
        let snapshot = scheduler.status_snapshot().await;
        let group = snapshot
            .iter()
            .find(|status| status.group == "toss:order")
            .expect("group status");
        assert_eq!(group.consecutive_failures, 0);
        assert!(group.last_success_epoch_ms.is_some());
    }

    #[tokio::test]
    async fn shared_scheduler_reuses_same_instance_per_scope() {
        let first = shared_scheduler("test|scope-a", RateLimitScheduler::new);
        first.record_outcome("group", false).await;

        // 같은 scope는 짧게 생성되는 client마다 상태를 공유해야 한다.
        let second = shared_scheduler("test|scope-a", RateLimitScheduler::new);
        let snapshot = second.status_snapshot().await;
        assert_eq!(snapshot.len(), 1);
        assert_eq!(snapshot[0].consecutive_failures, 1);

        // 다른 scope는 독립 상태를 가진다.
        let other = shared_scheduler("test|scope-b", RateLimitScheduler::new);
        assert!(other.status_snapshot().await.is_empty());
    }
}
