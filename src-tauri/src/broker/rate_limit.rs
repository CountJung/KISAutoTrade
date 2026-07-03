use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use reqwest::header::HeaderMap;
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
}

impl Default for RateLimitGroupState {
    fn default() -> Self {
        Self {
            min_interval: Duration::ZERO,
            next_available_at: Instant::now(),
            paused_until: None,
        }
    }
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
}
