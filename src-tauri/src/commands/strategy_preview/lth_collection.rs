//! Bounded research collection; the runtime supplies the reservation absolute deadline.
use super::*;
use crate::broker::{BrokerMarket, CandleCursor, CandlePageRequest};
use chrono::{DateTime, Utc};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::time::Duration;

#[derive(Clone, Copy)]
pub(super) struct Context {
    pub as_of: DateTime<Utc>,
    pub count: u16,
    pub max_pages: usize,
    pub max_rows: usize,
    pub deadline: tokio::time::Instant,
    pub page_size: u16,
}

impl Context {
    #[cfg(test)]
    pub fn new(as_of: DateTime<Utc>, count: u16) -> Self {
        Self::with_deadline(
            as_of,
            count,
            tokio::time::Instant::now() + Duration::from_secs(30),
        )
    }

    pub fn with_deadline(as_of: DateTime<Utc>, count: u16, deadline: tokio::time::Instant) -> Self {
        Self {
            as_of,
            count,
            max_pages: 16,
            max_rows: 3200,
            deadline,
            page_size: 200,
        }
    }
}

#[derive(Default)]
struct Budget {
    pages: usize,
    raw: usize,
    admitted: usize,
}

impl Budget {
    fn error(&self, reason: &'static str, context: Context) -> CmdError {
        CmdError {
            code: "LTH_COLLECTION_INCOMPLETE".into(),
            message: format!(
                "US 분봉 연구 수집 중단: {reason}; admitted={} missing={} pages={} raw={}",
                self.admitted,
                usize::from(context.count).saturating_sub(self.admitted),
                self.pages,
                self.raw
            ),
        }
    }
}

struct Stream {
    interval: &'static str,
    cursor: CandleCursor,
    ended: bool,
    rows: BTreeMap<DateTime<Utc>, BrokerCandle>,
    stats: StreamStats,
}

fn cursor_time(cursor: &CandleCursor) -> Option<DateTime<Utc>> {
    match cursor {
        CandleCursor::Toss { before } if before.len() <= 64 => DateTime::parse_from_rfc3339(before)
            .ok()
            .map(|t| t.to_utc()),
        _ => None,
    }
}

impl Stream {
    fn new(interval: &'static str, cutoff: &str) -> Self {
        Self {
            interval,
            cursor: CandleCursor::Toss {
                before: cutoff.into(),
            },
            ended: false,
            rows: BTreeMap::new(),
            stats: StreamStats::default(),
        }
    }

    fn values(&self) -> Vec<BrokerCandle> {
        self.rows.values().cloned().collect()
    }

    async fn page(
        &mut self,
        adapter: &dyn BrokerAdapter,
        symbol: &BrokerSymbol,
        context: Context,
        budget: &mut Budget,
        deadline: tokio::time::Instant,
    ) -> CmdResult<()> {
        if self.ended {
            return Err(budget.error("provider_exhausted", context));
        }
        if budget.pages >= context.max_pages {
            return Err(budget.error("page_limit", context));
        }
        // Never dispatch a request whose maximum response exceeds the remaining raw budget.
        let size = usize::from(context.page_size).min(context.max_rows.saturating_sub(budget.raw));
        if size == 0 {
            return Err(budget.error("row_limit", context));
        }
        let request = CandlePageRequest {
            symbol: symbol.clone(),
            market: BrokerMarket::Us,
            interval: self.interval.into(),
            count: size as u16,
            exchange: None,
            cursor: Some(self.cursor.clone()),
        };
        request
            .validate()
            .map_err(|_| budget.error("invalid_request", context))?;
        let anchor =
            cursor_time(&self.cursor).ok_or_else(|| budget.error("invalid_cursor", context))?;
        // Ready/non-yielding futures can complete before timeout_at polls its timer.
        // Enforce the shared deadline at dispatch, not only around provider awaits.
        if tokio::time::Instant::now() >= deadline {
            return Err(budget.error("total_deadline", context));
        }
        budget.pages += 1;
        self.stats.dispatched_pages += 1;
        let page_deadline = (tokio::time::Instant::now() + Duration::from_secs(15)).min(deadline);
        let page =
            tokio::time::timeout_at(page_deadline, adapter.get_candle_page_with_source(&request))
                .await
                .map_err(|_| {
                    budget.error(
                        if page_deadline == deadline {
                            "total_deadline"
                        } else {
                            "page_timeout"
                        },
                        context,
                    )
                })?
                .map_err(|_| budget.error("provider_error", context))?;
        let source = page.source;
        let page = page.page;
        self.stats.returned_pages += 1;
        self.stats.returned_typed_rows += page.candles.len();
        budget.raw = budget.raw.saturating_add(page.candles.len());
        // Sources describe returned responses, including invalid pages; not validated cohort.
        if let Some(source) = source {
            self.stats.sources.push(PageSource {
                stream: self.interval,
                response_index: self.stats.returned_pages,
                sha256: source.sha256,
                byte_length: source.byte_length,
                collected_at: source.collected_at,
            });
        }
        // A returned invalid page is counted only as attempted/returned, never validated.
        if tokio::time::Instant::now() >= deadline {
            return Err(budget.error("total_deadline", context));
        }
        if page.candles.len() > size || budget.raw > context.max_rows {
            return Err(budget.error("invalid_page_size", context));
        }
        let expected = if self.interval == "D" {
            "tradingDate"
        } else {
            "barClose"
        };
        if page.timestamp_semantics != expected {
            return Err(budget.error("invalid_semantics", context));
        }
        // The adapter normalizes stalled cursors to next=None plus this reason.
        // Preserve that failure instead of accepting a full cohort as ordinary exhaustion.
        if page.end_reason.as_deref() == Some("cursorDidNotAdvance") {
            return Err(budget.error("cursor_no_progress", context));
        }
        // Validate the response cursor even when the requested cohort is already complete.
        if let Some(next) = &page.next_cursor {
            let next_time =
                cursor_time(next).ok_or_else(|| budget.error("invalid_cursor", context))?;
            if next_time >= anchor || next_time > context.as_of {
                return Err(budget.error("cursor_no_progress", context));
            }
        }
        let old_len = self.rows.len();
        let returned_rows = page.candles.len();
        let mut staged = BTreeMap::new();
        let mut duplicates = 0;
        for row in page.candles {
            if row.symbol != *symbol || row.market != BrokerMarket::Us {
                return Err(budget.error("invalid_scope", context));
            }
            let instant = DateTime::parse_from_rfc3339(&row.date)
                .map_err(|_| budget.error("invalid_time", context))?
                .to_utc();
            if instant > context.as_of || instant > anchor {
                return Err(budget.error("time_upper_bound", context));
            }
            if let Some(previous) = self.rows.get(&instant).or_else(|| staged.get(&instant)) {
                // Only the label may differ (offset aliases); keep the original precision/label.
                if previous.symbol != row.symbol
                    || previous.market != row.market
                    || previous.open != row.open
                    || previous.high != row.high
                    || previous.low != row.low
                    || previous.close != row.close
                    || previous.volume != row.volume
                {
                    return Err(budget.error("conflicting_overlap", context));
                }
                duplicates += 1;
            } else {
                staged.insert(instant, row);
            }
        }
        if staged.is_empty() {
            return Err(budget.error(
                if page.next_cursor.is_some() {
                    "no_progress"
                } else {
                    "empty_page"
                },
                context,
            ));
        }
        self.rows.extend(staged);
        debug_assert!(self.rows.len() > old_len);
        self.stats.validated_pages += 1;
        self.stats.validated_typed_rows += returned_rows;
        self.stats.duplicate_rows += duplicates;
        self.stats.unique_rows = self.rows.len();
        self.stats.observed_start = self.rows.first_key_value().map(|(t, _)| *t);
        self.stats.observed_end = self.rows.last_key_value().map(|(t, _)| *t);
        self.ended = page.next_cursor.is_none() || page.end_reason.is_some();
        if let Some(next) = page.next_cursor {
            self.cursor = next;
        }
        if tokio::time::Instant::now() >= deadline {
            return Err(budget.error("total_deadline", context));
        }
        Ok(())
    }
}

/// Internal observations only: no publication, adjustment, OHLC or coverage certification.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Summary {
    requested_start: Option<String>,
    as_of: DateTime<Utc>,
    requested_count: u16,
    max_pages: usize,
    max_typed_rows: usize,
    page_size: u16,
    page_timeout_seconds: u8,
    remaining_budget_ms_at_start: u64,
    daily: StreamStats,
    minute: StreamStats,
    admission: Option<Admission>,
    admission_status: &'static str,
    source_availability: &'static str,
    normalized_input_sha256: Option<String>,
    coverage_verified: bool,
    publication_verification: &'static str,
    daily_ohlc_verification: &'static str,
    adjustment_verification: &'static str,
}
#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
struct StreamStats {
    dispatched_pages: usize,
    returned_pages: usize,
    returned_typed_rows: usize,
    validated_pages: usize,
    validated_typed_rows: usize,
    duplicate_rows: usize,
    unique_rows: usize,
    observed_start: Option<DateTime<Utc>>,
    observed_end: Option<DateTime<Utc>>,
    /// Returned-response fingerprints, independent of validated page/row counts.
    sources: Vec<PageSource>,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PageSource {
    stream: &'static str,
    response_index: usize,
    sha256: String,
    byte_length: usize,
    collected_at: DateTime<Utc>,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Admission {
    adopted_rows: usize,
    excluded: BTreeMap<String, usize>,
    used_daily_rows: usize,
    evaluation_start: Option<DateTime<Utc>>,
    evaluation_end: Option<DateTime<Utc>>,
    daily_context_start: Option<String>,
    daily_context_end: Option<String>,
}
impl Summary {
    fn snapshot(
        context: Context,
        remaining_budget_ms_at_start: u64,
        daily: Stream,
        minute: Stream,
        admission: Option<Admission>,
    ) -> Self {
        let responses = daily.stats.returned_pages + minute.stats.returned_pages;
        let sources = daily.stats.sources.len() + minute.stats.sources.len();
        Self {
            requested_start: None,
            as_of: context.as_of,
            requested_count: context.count,
            max_pages: context.max_pages,
            max_typed_rows: context.max_rows,
            page_size: context.page_size,
            page_timeout_seconds: 15,
            remaining_budget_ms_at_start,
            daily: daily.stats,
            minute: minute.stats,
            admission_status: if admission.is_some() {
                "lastCompleted"
            } else {
                "unavailable"
            },
            admission,
            source_availability: if sources == 0 {
                "unavailable"
            } else if sources == responses {
                "complete"
            } else {
                "partial"
            },
            normalized_input_sha256: None,
            coverage_verified: false,
            publication_verification: "unknown",
            daily_ohlc_verification: "unverified",
            adjustment_verification: "unverified",
        }
    }
}

/// Deterministic UTF-8 compact JSON, declaration field order, UTC RFC3339 Z;
/// native money/quantity strings are retained. D includes original join label.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct NormalizedInput<'a> {
    version: &'static str,
    as_of: String,
    requested_count: u16,
    daily: Vec<NormalizedRow<'a>>,
    minute: Vec<NormalizedRow<'a>>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct NormalizedRow<'a> {
    symbol: &'a BrokerSymbol,
    market: BrokerMarket,
    instant: String,
    trading_date: Option<String>,
    source_label: Option<&'a str>,
    open: &'a BrokerMoney,
    high: &'a BrokerMoney,
    low: &'a BrokerMoney,
    close: &'a BrokerMoney,
    volume: &'a str,
}
fn normalized_rows(rows: &[BrokerCandle], daily: bool) -> Result<Vec<NormalizedRow<'_>>, ()> {
    rows.iter()
        .map(|r| {
            let instant = DateTime::parse_from_rfc3339(&r.date)
                .map_err(|_| ())?
                .to_utc();
            let trading_date = daily.then(|| normalize_preview_chart_time(&r.date, "1d"));
            if trading_date
                .as_ref()
                .is_some_and(|d| chrono::NaiveDate::parse_from_str(d, "%Y%m%d").is_err())
            {
                return Err(());
            }
            Ok(NormalizedRow {
                symbol: &r.symbol,
                market: r.market,
                instant: instant.to_rfc3339_opts(chrono::SecondsFormat::AutoSi, true),
                trading_date,
                source_label: daily.then_some(r.date.as_str()),
                open: &r.open,
                high: &r.high,
                low: &r.low,
                close: &r.close,
                volume: &r.volume.0,
            })
        })
        .collect()
}
fn normalized_payload(
    daily: &[BrokerCandle],
    minute: &[BrokerCandle],
    context: Context,
) -> Result<Vec<u8>, ()> {
    serde_json::to_vec(&NormalizedInput {
        version: "lth-typed-collected-v1",
        as_of: context
            .as_of
            .to_rfc3339_opts(chrono::SecondsFormat::AutoSi, true),
        requested_count: context.count,
        daily: normalized_rows(daily, true)?,
        minute: normalized_rows(minute, false)?,
    })
    .map_err(|_| ())
}

#[derive(Debug)]
pub(super) struct Outcome {
    pub daily: Vec<BrokerCandle>,
    pub minute: Vec<BrokerCandle>,
    pub summary: Option<Summary>,
}
#[derive(Debug)]
pub(super) struct Failure {
    pub error: CmdError,
    pub summary: Summary,
}
pub(super) async fn collect(
    adapter: &dyn BrokerAdapter,
    symbol: &BrokerSymbol,
    context: Context,
) -> CmdResult<(Vec<BrokerCandle>, Vec<BrokerCandle>)> {
    match collect_with_summary(adapter, symbol, context).await {
        Ok(Outcome {
            daily,
            minute,
            summary,
        }) => {
            drop(summary);
            Ok((daily, minute))
        }
        Err(Failure { error, summary }) => {
            drop(summary);
            Err(error)
        }
    }
}
pub(super) async fn collect_with_summary(
    adapter: &dyn BrokerAdapter,
    symbol: &BrokerSymbol,
    context: Context,
) -> Result<Outcome, Failure> {
    let remaining_budget_ms_at_start = u64::try_from(
        context
            .deadline
            .saturating_duration_since(tokio::time::Instant::now())
            .as_millis(),
    )
    .unwrap_or(u64::MAX);
    let mut budget = Budget::default();
    let cutoff = context.as_of.to_rfc3339();
    let mut daily = Stream::new("D", &cutoff);
    let mut minute = Stream::new("1m", &cutoff);
    let mut admission = None;
    let deadline = context.deadline;
    let policy_error = if adapter.broker_id() != BrokerId::Toss {
        Some("invalid_provider")
    } else if context.count == 0 || !(1..=200).contains(&context.page_size) {
        Some("invalid_policy")
    } else {
        None
    };
    if let Some(reason) = policy_error {
        return Err(Failure {
            error: budget.error(reason, context),
            summary: Summary::snapshot(context, remaining_budget_ms_at_start, daily, minute, None),
        });
    }
    // State remains outside the timed future so cancellation/timeout retains known observations.
    let result = tokio::time::timeout_at(deadline, async {
        daily
            .page(adapter, symbol, context, &mut budget, deadline)
            .await?;
        loop {
            minute
                .page(adapter, symbol, context, &mut budget, deadline)
                .await?;
            let prepared = prepare_observed(&minute, &daily, context, &mut budget, &mut admission)?;
            budget.admitted = prepared.timed.len();
            if budget.admitted == usize::from(context.count) {
                break;
            }
        }
        loop {
            let prepared = prepare_observed(&minute, &daily, context, &mut budget, &mut admission)?;
            if !prepared.warmup.is_empty() {
                let daily_rows = daily.values();
                let minute_rows = minute.values();
                if tokio::time::Instant::now() >= deadline {
                    return Err(budget.error("total_deadline", context));
                }
                let payload = normalized_payload(&daily_rows, &minute_rows, context)
                    .map_err(|_| budget.error("invalid_normalization", context))?;
                if tokio::time::Instant::now() >= deadline {
                    return Err(budget.error("total_deadline", context));
                }
                #[cfg(test)]
                HASH_CALLS.with(|n| n.set(n.get() + 1));
                let hash = hex::encode(Sha256::digest(payload));
                if tokio::time::Instant::now() >= deadline {
                    return Err(budget.error("total_deadline", context));
                }
                return Ok((daily_rows, minute_rows, hash));
            }
            if daily.ended {
                return Err(budget.error("daily_context_missing", context));
            }
            daily
                .page(adapter, symbol, context, &mut budget, deadline)
                .await?;
        }
    })
    .await;
    let result = if tokio::time::Instant::now() >= deadline {
        Err(budget.error("total_deadline", context))
    } else {
        result.unwrap_or_else(|_| Err(budget.error("total_deadline", context)))
    };
    let mut summary = Summary::snapshot(
        context,
        remaining_budget_ms_at_start,
        daily,
        minute,
        admission,
    );
    if result.is_ok() && tokio::time::Instant::now() >= deadline {
        return Err(Failure {
            error: budget.error("total_deadline", context),
            summary,
        });
    }
    match result {
        Ok((daily, minute, hash)) => {
            summary.normalized_input_sha256 = Some(hash);
            Ok(Outcome {
                daily,
                minute,
                summary: Some(summary),
            })
        }
        Err(error) => Err(Failure { error, summary }),
    }
}

#[cfg(test)]
thread_local! {
    static PREPARE_CALLS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    static HASH_CALLS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

fn prepare_observed(
    minute: &Stream,
    daily: &Stream,
    context: Context,
    budget: &mut Budget,
    admission: &mut Option<Admission>,
) -> CmdResult<lth_regular::Prepared> {
    if tokio::time::Instant::now() >= context.deadline {
        return Err(budget.error("total_deadline", context));
    }
    #[cfg(test)]
    PREPARE_CALLS.with(|n| n.set(n.get() + 1));
    let prepared = lth_regular::prepare(
        &minute.values(),
        &daily.values(),
        context.count,
        context.as_of,
    )?;
    let mut excluded = BTreeMap::new();
    for (_, reason) in &prepared.excluded {
        *excluded.entry(format!("{reason:?}")).or_insert(0) += 1;
    }
    *admission = Some(Admission {
        adopted_rows: prepared.timed.len(),
        excluded,
        used_daily_rows: prepared.warmup.len(),
        evaluation_start: prepared
            .timed
            .first()
            .and_then(|r| DateTime::parse_from_rfc3339(&r.time).ok())
            .map(|t| t.to_utc()),
        evaluation_end: prepared
            .timed
            .last()
            .and_then(|r| DateTime::parse_from_rfc3339(&r.time).ok())
            .map(|t| t.to_utc()),
        daily_context_start: prepared
            .warmup
            .first()
            .map(|r| normalize_preview_chart_time(&r.date, "1d")),
        daily_context_end: prepared
            .warmup
            .last()
            .map(|r| normalize_preview_chart_time(&r.date, "1d")),
    });
    budget.admitted = prepared.timed.len();
    if tokio::time::Instant::now() >= context.deadline {
        return Err(budget.error("total_deadline", context));
    }
    Ok(prepared)
}

#[cfg(test)]
mod tests;
