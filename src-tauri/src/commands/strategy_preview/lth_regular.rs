//! US regular-only preparation and the shared LTH replay driver. No provider I/O.
use super::*;
use crate::trading::simulation::minute_time::parse_minute_instant;
use crate::trading::strategy::lth_research_session::{self as clock, Exclusion, RegularClock};
use chrono::{DateTime, Utc};

/// Provenance is retained internally; wire exposure is a separate B pass.
#[derive(Debug)]
pub(super) struct Prepared {
    pub timed: Vec<LeveragedTrendHoldTimedCandle>,
    pub warmup: Vec<BrokerCandle>,
    pub sessions: Vec<RegularClock>,
    pub excluded: Vec<(usize, Exclusion)>,
    pub as_of: DateTime<Utc>,
    pub calendar_source: &'static str,
}

pub(super) fn prepare(
    source: &[BrokerCandle],
    daily: &[BrokerCandle],
    count: u16,
    as_of: DateTime<Utc>,
) -> CmdResult<Prepared> {
    let mut prepared = Prepared {
        timed: Vec::new(),
        warmup: Vec::new(),
        sessions: Vec::new(),
        excluded: Vec::new(),
        as_of,
        calendar_source: clock::CALENDAR_SOURCE,
    };
    let mut rows = Vec::new();
    for (index, row) in source.iter().enumerate() {
        let Some(instant) = parse_minute_instant(&row.date) else {
            prepared.excluded.push((index, Exclusion::InvalidTime));
            continue;
        };
        rows.push((instant, index, row));
    }
    rows.sort_by_key(|(instant, _, _)| *instant);
    // Reject every ambiguous instant (including identical duplicate rows), never
    // silently choose one price/label or let ordering decide the execution path.
    for (i, (instant, index, row)) in rows.iter().enumerate() {
        if i.checked_sub(1)
            .is_some_and(|prev| rows[prev].0 == *instant)
            || rows.get(i + 1).is_some_and(|next| next.0 == *instant)
        {
            prepared
                .excluded
                .push((*index, Exclusion::DuplicateInstant));
            continue;
        }
        let session = match clock::regular_clock(*instant) {
            Ok(session) => session,
            Err(reason) => {
                prepared.excluded.push((*index, reason));
                continue;
            }
        };
        if *instant > as_of {
            prepared.excluded.push((*index, Exclusion::Future));
            continue;
        }
        // Reuse the existing positive-price conversion, after instant validation.
        let mut converted = broker_candles_to_timed_ohlc(&[(*row).clone()], 1, "1m", 0, 0, 0);
        let Some(timed) = converted.pop() else {
            prepared.excluded.push((*index, Exclusion::InvalidPrice));
            continue;
        };
        prepared.timed.push(timed);
        prepared.sessions.push(session);
    }
    let skip = prepared.timed.len().saturating_sub(count as usize);
    // Keep source indices even for valid bars removed by the requested tail.
    for row in &prepared.timed[..skip] {
        let index = source
            .iter()
            .position(|source| source.date == row.time)
            .unwrap_or(0);
        prepared.excluded.push((index, Exclusion::SelectionLimit));
    }
    prepared.timed.drain(..skip);
    prepared.sessions.drain(..skip);
    if let Some(first) = prepared.sessions.first() {
        let cutoff = first.date.format("%Y%m%d").to_string();
        // Daily labels are trading dates, not midnight instants to shift to NY.
        prepared.warmup = daily
            .iter()
            .filter(|row| {
                let date = normalize_preview_chart_time(&row.date, "1d");
                chrono::NaiveDate::parse_from_str(&date, "%Y%m%d").is_ok() && date < cutoff
            })
            .cloned()
            .collect();
        prepared.warmup.sort_by(|a, b| a.date.cmp(&b.date));
    }
    // Read retained metadata here as an invariant; do not promote verification flags.
    debug_assert_eq!(prepared.as_of, as_of);
    debug_assert!(!prepared.calendar_source.is_empty());
    debug_assert_eq!(prepared.sessions.len(), prepared.timed.len());
    Ok(prepared)
}

pub(super) fn execute(
    interval: &str,
    symbol: &str,
    params: LeveragedTrendHoldParams,
    ohlc: &[OhlcCandle],
    timed: &[LeveragedTrendHoldTimedCandle],
    base_execution_events: &[SimulationEvent],
    entry_is_overseas: bool,
    assumptions: SimulationAssumptions,
) -> (Vec<LeveragedTrendHoldPreviewSignal>, ReplaySession) {
    let mut params = params;
    if interval == "1m" && entry_is_overseas {
        params.toss_us_session = "regular".into();
    }
    let mut session = ReplaySession::new(
        "leveraged_trend_hold",
        &symbol,
        entry_is_overseas,
        assumptions,
        &[],
        if interval == "1m" {
            base_execution_events.len()
        } else {
            0
        },
    );
    let mut observation_index = 0;
    let mut close_processed = false;
    let preview_signals =
        lth_preview_signals_for_interval(interval, &symbol, params, &ohlc, &timed, |step| {
            let Some(event) = base_execution_events.get(observation_index) else {
                return session.position_snapshot();
            };
            match step {
                LeveragedTrendHoldPreviewStep::Open(_) => {
                    session.begin_bar(event);
                    close_processed = false;
                }
                LeveragedTrendHoldPreviewStep::Signal(preview_signal, _) => {
                    let mut event = event.clone();
                    event.signal = Some(if preview_signal.side == "buy" {
                        Signal::Buy {
                            symbol: symbol.to_string(),
                            quantity: preview_signal.quantity,
                            reason: preview_signal.reason.clone(),
                        }
                    } else {
                        Signal::Sell {
                            symbol: symbol.to_string(),
                            quantity: preview_signal.quantity,
                            reason: preview_signal.reason.clone(),
                        }
                    });
                    session.process_close(&event);
                    close_processed = true;
                }
                LeveragedTrendHoldPreviewStep::Close(_) => {
                    if !close_processed {
                        session.process_close(event);
                    }
                    observation_index += 1;
                }
            }
            session.position_snapshot()
        });
    debug_assert_eq!(
        observation_index,
        base_execution_events
            .len()
            .min(if interval == "1m" { timed.len() } else { 0 })
    );
    (preview_signals, session)
}

pub(super) const CLOCK_VERSION: &str = "us-regular-close-v1";
pub(super) const STRATEGY_VERSION: &str = "leveraged-trend-hold-v5-us-regular";

fn reason_name(reason: Exclusion) -> &'static str {
    match reason {
        Exclusion::InvalidTime => "invalidTime",
        Exclusion::CalendarUnknown => "calendarUnknown",
        Exclusion::Weekend => "weekend",
        Exclusion::PlannedHoliday => "plannedHoliday",
        Exclusion::ExtraordinaryClosure => "extraordinaryClosure",
        Exclusion::OutsideRegular => "outsideRegular",
        Exclusion::Future => "future",
        Exclusion::DuplicateInstant => "duplicateInstant",
        Exclusion::InvalidPrice => "invalidPrice",
        Exclusion::SelectionLimit => "selectionLimit",
    }
}

/// Join by the original unique label; never sort or tail the admitted sequence again.
pub(super) fn chart(source: &[BrokerCandle], prepared: &Prepared) -> Vec<ChartCandle> {
    prepared
        .timed
        .iter()
        .filter_map(|timed| {
            let row = source.iter().find(|row| row.date == timed.time)?;
            Some(ChartCandle {
                date: row.date.clone(),
                open: broker_money_amount(&row.open)?.to_string(),
                high: broker_money_amount(&row.high)?.to_string(),
                low: broker_money_amount(&row.low)?.to_string(),
                close: broker_money_amount(&row.close)?.to_string(),
                volume: row.volume.0.clone(),
            })
        })
        .collect()
}

pub(super) fn execution_clock(
    source: &[BrokerCandle],
    prepared: &Prepared,
    blackout_windows: &[String],
) -> crate::trading::simulation::ExecutionClockView {
    use crate::trading::simulation::{
        ExecutionClockView, ExecutionDayView, ExecutionExclusionCount, ExecutionExclusionExample,
    };
    use chrono::TimeZone;
    use chrono_tz::America::New_York;
    use std::collections::BTreeMap;
    let mut counts = BTreeMap::<&str, usize>::new();
    for (_, reason) in &prepared.excluded {
        *counts.entry(reason_name(*reason)).or_default() += 1;
    }
    let mut days = BTreeMap::<String, ExecutionDayView>::new();
    for session in &prepared.sessions {
        let date = session.date.to_string();
        let day = days.entry(date.clone()).or_insert_with(|| {
            // Calendar policy supplies an exact 13:00/16:00 close. The two
            // minute counters truncate independently for non-aligned labels.
            let open = New_York
                .from_local_datetime(
                    &session
                        .date
                        .and_hms_opt(9, 30, 0)
                        .expect("valid session open"),
                )
                .single()
                .expect("unambiguous regular open");
            let close = New_York
                .from_local_datetime(
                    &session
                        .date
                        .and_hms_opt(if session.early_close { 13 } else { 16 }, 0, 0)
                        .expect("valid session close"),
                )
                .single()
                .expect("unambiguous regular close");
            ExecutionDayView {
                date,
                open: open.to_rfc3339(),
                close: close.to_rfc3339(),
                early_close: session.early_close,
                adopted_rows: 0,
            }
        });
        day.adopted_rows += 1;
    }
    // No raw asOf: identical policy and admitted observations remain the same model.
    // Full windows/clock rows participate before the bounded wire projection.
    let admission = prepared
        .timed
        .iter()
        .zip(&prepared.sessions)
        .map(|(row, session)| {
            let volume = source
                .iter()
                .find(|source| source.date == row.time)
                .map(|source| source.volume.0.as_str())
                .unwrap_or("");
            format!(
                "{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}",
                row.time,
                row.candle.open,
                row.candle.high,
                row.candle.low,
                row.candle.close,
                session.date,
                session.elapsed,
                session.to_close,
                session.blackout_kst_minute,
                session.early_close,
                volume
            )
        })
        .collect::<Vec<_>>()
        .join("|");
    let blackout_identity = serde_json::to_string(blackout_windows).unwrap_or_default();
    let identity = replay_input_hash(&[
        CLOCK_VERSION,
        chrono_tz::IANA_TZDB_VERSION,
        "America/New_York",
        "barClose:[close-1m,close);close<=asOf",
        "Asia/Seoul",
        &blackout_identity,
        prepared.calendar_source,
        "2023-2028",
        &admission,
    ]);
    let day_count = days.len();
    ExecutionClockView {
        version: CLOCK_VERSION.into(),
        timezone: "America/New_York".into(),
        tzdb_version: chrono_tz::IANA_TZDB_VERSION.into(),
        label_policy: "barClose:[close-1m,close);close<=asOf".into(),
        blackout_timezone: "Asia/Seoul".into(),
        blackout_windows: blackout_windows.to_vec(),
        calendar_source: prepared.calendar_source.into(),
        calendar_horizon: "2023-2028".into(),
        as_of: prepared.as_of.to_rfc3339(),
        admission_identity: identity,
        source_rows: source.len(),
        adopted_rows: prepared.timed.len(),
        warmup_rows: prepared.warmup.len(),
        unknown_rows: *counts.get("calendarUnknown").unwrap_or(&0),
        excluded: counts
            .into_iter()
            .map(|(reason, count)| ExecutionExclusionCount {
                reason: reason.into(),
                count,
            })
            .collect(),
        examples: prepared
            .excluded
            .iter()
            .take(6)
            .map(|(index, reason)| ExecutionExclusionExample {
                input_index: *index,
                label: source[*index].date.chars().take(128).collect(),
                label_truncated: source[*index].date.chars().count() > 128,
                reason: reason_name(*reason).into(),
            })
            .collect(),
        omitted_examples: prepared.excluded.len().saturating_sub(6),
        days: days.into_values().take(8).collect(),
        day_count,
        omitted_days: day_count.saturating_sub(8),
    }
}

/// Production assembly seam shared by the provider caller and pure fixtures.
pub(super) fn assessment(
    source: &[BrokerCandle],
    prepared: &Prepared,
    count: u16,
    daily_bars: usize,
    blackout_windows: &[String],
) -> ReplayAssessmentView {
    let mut assessment = lth_preview_assessment(
        "1m",
        daily_bars,
        source.len(),
        count,
        &prepared.timed,
        true,
        prepared.as_of,
    );
    let clock = execution_clock(source, prepared, blackout_windows);
    if let Some(timing) = &mut assessment.minute_timing {
        // For this path selection means the full raw preparation cohort, not a lexical tail.
        timing.selected_rows = source.len();
        timing.price_excluded_rows = prepared
            .excluded
            .iter()
            .filter(|(_, reason)| *reason == Exclusion::InvalidPrice)
            .count();
        timing.limitations.retain(|text| {
            !text.contains("legacyFixedKstUnverified") && !text.contains("lexical tail")
        });
        timing.limitations.push("실행 채택은 executionClock의 NY 계획 달력/단일 cutoff를 따릅니다. 위 일반 평일창 진단은 별도이며 실제 공급/발행/coverage 검증이 아닙니다. blackout은 레거시 Asia/Seoul 분입니다. inputHash는 params·비용·scope를 포함하며 순수 dataset hash가 아닙니다.".into());
        timing.execution_clock = Some(clock);
    }
    assessment
}
