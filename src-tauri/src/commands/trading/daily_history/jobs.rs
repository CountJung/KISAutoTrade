//! Two bounded background jobs; provider futures are never awaited by quote polling.
use std::{
    collections::BTreeMap,
    future::Future,
    pin::Pin,
    sync::Arc,
    time::{Duration, Instant},
};

use chrono::{DateTime, Utc};
use tokio::{
    sync::{Mutex, RwLock},
    task::JoinHandle,
};

use super::{collect_daily_history, CollectedDailyHistory};
use crate::{
    api::rest::KisRestClient,
    config::ProfilesConfig,
    trading::{
        order::OrderManager,
        strategy::{DailyHistoryTicket, StrategyManager},
    },
};

const MAX_IN_FLIGHT: usize = 2;
const MAX_TARGETS: usize = 512;
type FetchFuture = Pin<Box<dyn Future<Output = Result<CollectedDailyHistory, String>> + Send>>;
type TargetKey = (String, String);

struct JobEntry {
    tickets: Vec<DailyHistoryTicket>,
    attempts: u32,
    retry_at: Instant,
    handle: Option<JoinHandle<Result<CollectedDailyHistory, String>>>,
    satisfied: bool,
}

#[derive(Default)]
pub(in crate::commands::trading) struct DailyHistoryJobs {
    entries: BTreeMap<TargetKey, JobEntry>,
}

impl Drop for DailyHistoryJobs {
    fn drop(&mut self) {
        for entry in self.entries.values_mut() {
            if let Some(handle) = entry.handle.take() {
                handle.abort();
            }
        }
    }
}

fn retry_delay(attempts: u32) -> Duration {
    Duration::from_secs((30_u64.saturating_mul(1 << attempts.saturating_sub(1).min(5))).min(900))
}

impl DailyHistoryJobs {
    pub(in crate::commands::trading) async fn maintain(
        &mut self,
        is_running: &Arc<Mutex<bool>>,
        manager: &Arc<Mutex<StrategyManager>>,
        orders: &Arc<Mutex<OrderManager>>,
        profiles: &Arc<RwLock<ProfilesConfig>>,
        rest: &Arc<RwLock<Arc<KisRestClient>>>,
    ) {
        if !*is_running.lock().await {
            self.clear();
            return;
        }
        let scope = orders.lock().await.execution_scope().clone();
        let identity = manager.lock().await.history_execution_identity();
        let run_profile = identity.and_then(|(_, profile)| profile);
        // Resolve the pinned run profile, never another profile sharing the account.
        let profile = {
            let profiles = profiles.read().await;
            profiles
                .profiles
                .iter()
                .find(|profile| {
                    run_profile.as_ref().is_some_and(|id| id == &profile.id)
                        && profile.broker_id == scope.broker_id
                        && scope
                            .account_id
                            .as_ref()
                            .is_some_and(|id| id.0 == profile.broker_account_id())
                })
                .cloned()
        };
        let profile_id = run_profile.unwrap_or_default();
        let now = Utc::now();
        let targets = {
            let running = is_running.lock().await;
            if !*running {
                self.clear();
                return;
            }
            manager
                .lock()
                .await
                .prepare_daily_history_targets(&scope, &profile_id, now)
        };
        let rest = rest.read().await.clone();
        self.tick(
            targets,
            now,
            Instant::now(),
            is_running,
            manager,
            move |ticket| {
                let profile = profile.clone();
                let rest = rest.clone();
                Box::pin(async move {
                    let profile = profile
                        .ok_or("실행 프로파일·계좌를 찾지 못해 완료 일봉 조회를 중단했습니다.")?;
                    collect_daily_history(ticket, profile, rest).await
                })
            },
        )
        .await;
    }

    fn clear(&mut self) {
        for entry in self.entries.values_mut() {
            if let Some(handle) = entry.handle.take() {
                handle.abort();
            }
        }
        self.entries.clear();
    }

    /// Injectable scheduling clock and fetcher allow race/backoff tests without accounts.
    async fn tick<F>(
        &mut self,
        targets: Vec<DailyHistoryTicket>,
        now: DateTime<Utc>,
        instant: Instant,
        is_running: &Arc<Mutex<bool>>,
        manager: &Arc<Mutex<StrategyManager>>,
        fetch: F,
    ) where
        F: Fn(DailyHistoryTicket) -> FetchFuture,
    {
        let mut groups: BTreeMap<TargetKey, Vec<DailyHistoryTicket>> = BTreeMap::new();
        for ticket in targets.into_iter().take(MAX_TARGETS) {
            groups
                .entry((ticket.profile_id.clone(), ticket.symbol.clone()))
                .or_default()
                .push(ticket);
        }
        // Removed targets, new generation, scope/config changes, and a new market day cancel old work.
        self.entries.retain(|key, entry| {
            let keep = groups.get(key) == Some(&entry.tickets);
            if !keep {
                if let Some(handle) = entry.handle.take() {
                    handle.abort();
                }
            }
            keep
        });
        for (key, tickets) in groups {
            self.entries.entry(key).or_insert(JobEntry {
                tickets,
                attempts: 0,
                retry_at: instant,
                handle: None,
                satisfied: false,
            });
        }
        for entry in self.entries.values_mut() {
            if !entry
                .handle
                .as_ref()
                .is_some_and(|handle| handle.is_finished())
            {
                continue;
            }
            let result = entry
                .handle
                .take()
                .expect("finished handle")
                .await
                .unwrap_or_else(|_| Err("완료 일봉 조회 작업이 비정상 종료되었습니다.".into()));
            let running = is_running.lock().await;
            if !*running {
                continue;
            }
            let mut manager = manager.lock().await;
            let delay = retry_delay(entry.attempts);
            let next_retry = (now + chrono::Duration::seconds(delay.as_secs() as i64)).to_rfc3339();
            entry.satisfied = true;
            for ticket in &entry.tickets {
                match &result {
                    Ok(history) => {
                        let accepted = manager.apply_daily_history(
                            ticket,
                            &history.candles,
                            history.source.clone(),
                            history.last_completed_date.clone(),
                            now,
                        );
                        let ready = accepted && manager.is_daily_history_ready(ticket);
                        if accepted && !ready {
                            manager.set_daily_history_collection_reason(
                                ticket,
                                &history.end_reason,
                                now,
                            );
                            manager.set_daily_history_retry(ticket, Some(next_retry.clone()));
                        }
                        entry.satisfied &= ready;
                    }
                    Err(error) => {
                        manager.mark_daily_history_failure(
                            ticket,
                            error,
                            Some(next_retry.clone()),
                            now,
                        );
                        entry.satisfied = false;
                    }
                }
            }
            entry.retry_at = instant + delay;
        }
        let mut active = self
            .entries
            .values()
            .filter(|entry| entry.handle.is_some())
            .count();
        for entry in self.entries.values_mut() {
            if active >= MAX_IN_FLIGHT {
                break;
            }
            if entry.satisfied || entry.handle.is_some() || instant < entry.retry_at {
                continue;
            }
            let running = is_running.lock().await;
            if !*running {
                break;
            }
            let mut manager = manager.lock().await;
            let mut any_valid = false;
            for ticket in &entry.tickets {
                any_valid |= manager.mark_daily_history_pending(ticket, now);
            }
            if !any_valid {
                continue;
            }
            // The same instrument/date/scope is fetched once for all 52-week configurations.
            let ticket = entry
                .tickets
                .iter()
                .max_by_key(|ticket| ticket.required_bars)
                .expect("nonempty target group")
                .clone();
            entry.attempts = entry.attempts.saturating_add(1);
            entry.handle = Some(tokio::spawn(fetch(ticket)));
            active += 1;
        }
        if !*is_running.lock().await {
            self.clear();
        }
    }
}

#[cfg(test)]
mod tests;
