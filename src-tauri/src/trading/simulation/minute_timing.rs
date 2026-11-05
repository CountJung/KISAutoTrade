//! Bounded, descriptive minute-label diagnostics. Never filters or changes replay input.
use std::collections::{BTreeMap, BTreeSet};

use chrono::{DateTime, Datelike, Duration, Offset, SecondsFormat, Timelike, Utc, Weekday};
use chrono_tz::Tz;
use serde::Serialize;

use super::minute_time::parse_minute_instant;

const INPUT_LIMIT: usize = 512;
const DAY_LIMIT: usize = 8;
const EXAMPLE_LIMIT: usize = 6;
const LABEL_LIMIT: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum MinuteLabelConvention {
    BarClose,
    BarStart,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TimingMarket {
    Us,
    Kr,
}

impl TimingMarket {
    fn timezone(self) -> Tz {
        match self {
            Self::Us => chrono_tz::America::New_York,
            Self::Kr => chrono_tz::Asia::Seoul,
        }
    }

    fn ordinary_window(self) -> (u32, u32) {
        match self {
            Self::Us => (9 * 60 + 30, 16 * 60),
            Self::Kr => (9 * 60, 15 * 60 + 30),
        }
    }
}

/// Counts describe the provider result, legacy lexical tail selection, and executed rows.
#[derive(Debug, Clone, Copy)]
pub(crate) struct MinuteTimingInput<'a> {
    pub labels: &'a [&'a str],
    pub source_rows: usize,
    pub selected_rows: usize,
    pub market: TimingMarket,
    pub label_convention: MinuteLabelConvention,
    pub reference_as_of: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MinuteTimingExample {
    pub kind: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MinuteTimingDayView {
    pub market_date: String,
    pub utc_offset_minutes: i32,
    pub regular_unique_bars: usize,
    pub first_close_market_local: String,
    pub last_close_market_local: String,
    pub ordinary_window_expected_slots: usize,
    pub interior_unobserved_slots: usize,
    pub max_interior_gap_slots: usize,
    pub leading_unobserved_window_slots: usize,
    pub trailing_unobserved_window_slots: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LthMinuteTimingView {
    pub diagnostic_version: String,
    pub label_convention: MinuteLabelConvention,
    pub market: TimingMarket,
    pub exchange_timezone: String,
    pub reference_as_of: String,
    pub legacy_session_policy: String,
    pub source_rows: usize,
    pub selected_rows: usize,
    pub used_rows: usize,
    pub price_excluded_rows: usize,
    pub input_limit: usize,
    pub processed_rows: usize,
    pub input_truncated: bool,
    pub parsed_rows: usize,
    pub invalid_rows: usize,
    pub assumed_kst_rows: usize,
    pub duplicate_instants: usize,
    pub out_of_order_rows: usize,
    pub non_minute_aligned_rows: usize,
    pub regular_rows: usize,
    pub off_session_rows: usize,
    pub weekend_rows: usize,
    pub unfinished_rows: usize,
    pub market_utc_offsets_minutes: Vec<i32>,
    pub first_bar_start_utc: Option<String>,
    pub last_bar_start_utc: Option<String>,
    pub first_close_available_at_utc: Option<String>,
    pub last_close_available_at_utc: Option<String>,
    /// Chronological bounds of parsed original labels converted to the market timezone.
    pub first_market_local: Option<String>,
    pub last_market_local: Option<String>,
    pub observed_days_count: usize,
    pub day_row_limit: usize,
    pub omitted_day_rows: usize,
    pub interior_unobserved_slots: usize,
    pub max_interior_gap_slots: usize,
    pub days: Vec<MinuteTimingDayView>,
    pub examples: Vec<MinuteTimingExample>,
    pub calendar_verified: bool,
    pub provider_coverage_verified: bool,
    pub requested_period_coverage_verified: bool,
    pub actual_publication_verified: bool,
    pub limitations: Vec<String>,
}

impl LthMinuteTimingView {
    fn example(&mut self, kind: &str, label: &str) {
        if self.examples.len() >= EXAMPLE_LIMIT {
            return;
        }
        let mut end = label.len().min(LABEL_LIMIT);
        while !label.is_char_boundary(end) {
            end -= 1;
        }
        self.examples.push(MinuteTimingExample {
            kind: kind.into(),
            label: label[..end].into(),
        });
    }
}

fn utc(value: DateTime<Utc>) -> String {
    value.to_rfc3339_opts(SecondsFormat::AutoSi, true)
}

fn local(value: DateTime<Utc>, timezone: Tz) -> String {
    value.with_timezone(&timezone).to_rfc3339()
}

/// Pure timing interpretation. The caller's signals, candles and hash remain untouched.
pub(crate) fn diagnose_minute_timing(input: MinuteTimingInput<'_>) -> LthMinuteTimingView {
    let timezone = input.market.timezone();
    let mut view = LthMinuteTimingView {
        diagnostic_version: "lth-minute-timing-v1".into(),
        label_convention: input.label_convention,
        market: input.market,
        exchange_timezone: timezone.name().into(),
        reference_as_of: utc(input.reference_as_of),
        legacy_session_policy: "legacyFixedKstUnverified".into(),
        source_rows: input.source_rows,
        selected_rows: input.selected_rows,
        used_rows: input.labels.len(),
        price_excluded_rows: input.selected_rows.saturating_sub(input.labels.len()),
        input_limit: INPUT_LIMIT,
        processed_rows: input.labels.len().min(INPUT_LIMIT),
        input_truncated: input.labels.len() > INPUT_LIMIT,
        parsed_rows: 0,
        invalid_rows: 0,
        assumed_kst_rows: 0,
        duplicate_instants: 0,
        out_of_order_rows: 0,
        non_minute_aligned_rows: 0,
        regular_rows: 0,
        off_session_rows: 0,
        weekend_rows: 0,
        unfinished_rows: 0,
        market_utc_offsets_minutes: vec![],
        first_bar_start_utc: None,
        last_bar_start_utc: None,
        first_close_available_at_utc: None,
        last_close_available_at_utc: None,
        first_market_local: None,
        last_market_local: None,
        observed_days_count: 0,
        day_row_limit: DAY_LIMIT,
        omitted_day_rows: 0,
        interior_unobserved_slots: 0,
        max_interior_gap_slots: 0,
        days: vec![],
        examples: vec![],
        calendar_verified: false,
        provider_coverage_verified: false,
        requested_period_coverage_verified: false,
        actual_publication_verified: false,
        limitations: vec![
            "준비된 실행 label만 진단합니다. 기존 lexical tail 선택과 가격 제외, 실행 순서·시그널·체결은 바꾸지 않습니다.".into(),
            "compact12/14는 KST 가정입니다. 원본 표기의 시간대나 실제 발행·수신 시각을 검증한 결과가 아닙니다.".into(),
            "일반 평일 정규 시간창만 산술 비교합니다. 휴장·조기마감·거래정지·무거래봉 생략 및 provider/요청 기간 coverage는 미검증입니다.".into(),
            "미관측 슬롯은 관측된 같은 날짜 내부만 합산합니다. tail 앞뒤·없는 날짜·야간·주말의 자료 누락을 확정하지 않습니다.".into(),
            "현재 LTH 실행 시계는 legacyFixedKstUnverified이며 시간대 진단으로 성과 적합성이나 lookAheadSafe를 승격하지 않습니다.".into(),
        ],
    };
    if input.label_convention == MinuteLabelConvention::Unknown {
        view.limitations.push("봉 시작/종료 표기가 unknown이므로 세션·공개 가능 시각·미관측 슬롯을 판정하지 않습니다.".into());
    }
    let mut seen = BTreeSet::new();
    let mut offsets = BTreeSet::new();
    let mut previous = None;
    let mut first = None;
    let mut last = None;
    let mut start_bounds: Option<(DateTime<Utc>, DateTime<Utc>)> = None;
    let mut close_bounds: Option<(DateTime<Utc>, DateTime<Utc>)> = None;
    let mut days: BTreeMap<String, BTreeMap<u32, DateTime<Utc>>> = BTreeMap::new();
    let (open_minute, close_minute) = input.market.ordinary_window();
    for label in input.labels.iter().take(INPUT_LIMIT) {
        let instant = (label.len() <= LABEL_LIMIT)
            .then(|| parse_minute_instant(label))
            .flatten();
        let Some(instant) = instant else {
            view.invalid_rows += 1;
            view.example("invalid", label);
            continue;
        };
        view.parsed_rows += 1;
        if matches!(label.len(), 12 | 14) && label.bytes().all(|byte| byte.is_ascii_digit()) {
            view.assumed_kst_rows += 1;
        }
        if !seen.insert(instant) {
            view.duplicate_instants += 1;
            view.example("duplicateInstant", label);
        }
        if previous.is_some_and(|previous| instant < previous) {
            view.out_of_order_rows += 1;
            view.example("outOfOrder", label);
        }
        previous = Some(instant);
        first = Some(first.map_or(instant, |old: DateTime<Utc>| old.min(instant)));
        last = Some(last.map_or(instant, |old: DateTime<Utc>| old.max(instant)));
        let market_label = instant.with_timezone(&timezone);
        offsets.insert(market_label.offset().fix().local_minus_utc() / 60);
        let aligned = instant.second() == 0 && instant.nanosecond() == 0;
        if !aligned {
            view.non_minute_aligned_rows += 1;
            view.example("nonMinuteAligned", label);
        }
        let pair = match input.label_convention {
            MinuteLabelConvention::BarClose => instant
                .checked_sub_signed(Duration::minutes(1))
                .map(|start| (start, instant)),
            MinuteLabelConvention::BarStart => instant
                .checked_add_signed(Duration::minutes(1))
                .map(|close| (instant, close)),
            MinuteLabelConvention::Unknown => None,
        };
        let Some((start, close)) = pair else {
            continue;
        };
        start_bounds =
            Some(start_bounds.map_or((start, start), |(a, b)| (a.min(start), b.max(start))));
        close_bounds =
            Some(close_bounds.map_or((close, close), |(a, b)| (a.min(close), b.max(close))));
        if close > input.reference_as_of {
            view.unfinished_rows += 1;
            view.example("unfinished", label);
        }
        let market_start = start.with_timezone(&timezone);
        let market_close = close.with_timezone(&timezone);
        if matches!(market_start.weekday(), Weekday::Sat | Weekday::Sun) {
            view.weekend_rows += 1;
            view.example("weekend", label);
            continue;
        }
        let start_second = market_start.num_seconds_from_midnight();
        let close_second = market_close.num_seconds_from_midnight();
        let regular = market_start.date_naive() == market_close.date_naive()
            && start_second >= open_minute * 60
            && (close_second < close_minute * 60
                || (close_second == close_minute * 60 && market_close.nanosecond() == 0));
        if !regular {
            view.off_session_rows += 1;
            view.example("offSession", label);
            continue;
        }
        view.regular_rows += 1;
        if aligned {
            days.entry(market_start.format("%Y-%m-%d").to_string())
                .or_default()
                .entry(start_second / 60 - open_minute)
                .or_insert(close);
        }
    }
    view.market_utc_offsets_minutes = offsets.into_iter().collect();
    view.first_market_local = first.map(|value| local(value, timezone));
    view.last_market_local = last.map(|value| local(value, timezone));
    if let Some((first, last)) = start_bounds {
        view.first_bar_start_utc = Some(utc(first));
        view.last_bar_start_utc = Some(utc(last));
    }
    if let Some((first, last)) = close_bounds {
        view.first_close_available_at_utc = Some(utc(first));
        view.last_close_available_at_utc = Some(utc(last));
    }
    view.observed_days_count = days.len();
    view.omitted_day_rows = days.len().saturating_sub(DAY_LIMIT);
    for (date, slots) in days {
        let mut gaps = 0;
        let mut max_gap = 0;
        let mut previous_slot = None;
        for slot in slots.keys().copied() {
            if let Some(previous) = previous_slot {
                let gap = slot - previous - 1;
                gaps += gap as usize;
                max_gap = max_gap.max(gap as usize);
            }
            previous_slot = Some(slot);
        }
        view.interior_unobserved_slots += gaps;
        view.max_interior_gap_slots = view.max_interior_gap_slots.max(max_gap);
        if view.days.len() >= DAY_LIMIT {
            continue;
        }
        if let (Some((first_slot, first_close)), Some((last_slot, last_close))) =
            (slots.first_key_value(), slots.last_key_value())
        {
            view.days.push(MinuteTimingDayView {
                market_date: date,
                utc_offset_minutes: first_close
                    .with_timezone(&timezone)
                    .offset()
                    .fix()
                    .local_minus_utc()
                    / 60,
                regular_unique_bars: slots.len(),
                first_close_market_local: local(*first_close, timezone),
                last_close_market_local: local(*last_close, timezone),
                ordinary_window_expected_slots: (close_minute - open_minute) as usize,
                interior_unobserved_slots: gaps,
                max_interior_gap_slots: max_gap,
                leading_unobserved_window_slots: *first_slot as usize,
                trailing_unobserved_window_slots: (close_minute - open_minute - 1 - last_slot)
                    as usize,
            });
        }
    }
    view
}

#[cfg(test)]
#[path = "minute_timing_tests.rs"]
mod tests;
