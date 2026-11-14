//! US minute research only. Planned NYSE cash-equity dates 2023–2028 plus
//! the known Carter override; not exhaustive extraordinary-closure coverage.
//! Sources: ICE 2022 NYSE 2023/24/25 calendar (eff1f3137780f2e8...),
//! NYSE hours-calendars 2026/27/28 (d2a7bccb73687e31...), Nasdaq 2026
//! (37eacb8257100a0f...), Carter NYSE/Nasdaq ETA2025-1 (df302659fe83210f... /
//! d0b2354c9f6182fb...). Snapshots: P1-SESSION-01-primary-evidence-packet.
//! These dates certify neither provider publication nor actual bar coverage.
//! https://www.nyse.com/trade/hours-calendars
//! https://ir.theice.com/press/news-details/2022/NYSE-Group-Announces-2023-2024-and-2025-Holiday-and-Early-Closings-Calendar/default.aspx
//! https://www.nasdaqtrader.com/TraderNews.aspx?id=ETA2025-1
use crate::trading::simulation::minute_time::parse_minute_instant;
use chrono::{DateTime, Datelike, Duration, NaiveDate, TimeZone, Timelike, Utc, Weekday};
use chrono_tz::{America::New_York, Asia::Seoul};

pub(crate) const CALENDAR_SOURCE: &str = "NYSE planned 2023-2028; Nasdaq 2026; Carter 2025-01-09 override; extraordinary coverage unverified";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Exclusion {
    InvalidTime,
    CalendarUnknown,
    Weekend,
    PlannedHoliday,
    ExtraordinaryClosure,
    OutsideRegular,
    Future,
    DuplicateInstant,
    InvalidPrice,
    SelectionLimit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct RegularClock {
    pub date: NaiveDate,
    pub elapsed: i64,
    pub to_close: i64,
    pub blackout_kst_minute: i64,
    pub early_close: bool,
}

// Explicit published dates, not federal/settlement observance algorithms.
fn schedule(date: NaiveDate) -> Result<i64, Exclusion> {
    let holidays: &[u32] = match date.year() {
        2023 => &[102, 116, 220, 407, 529, 619, 704, 904, 1123, 1225],
        2024 => &[101, 115, 219, 329, 527, 619, 704, 902, 1128, 1225],
        2025 => &[101, 120, 217, 418, 526, 619, 704, 901, 1127, 1225],
        2026 => &[101, 119, 216, 403, 525, 619, 703, 907, 1126, 1225],
        2027 => &[101, 118, 215, 326, 531, 618, 705, 906, 1125, 1224],
        // NYSE explicitly does NOT observe Saturday New Year on 2027-12-31.
        2028 => &[117, 221, 414, 529, 619, 704, 904, 1123, 1225],
        _ => return Err(Exclusion::CalendarUnknown),
    };
    if matches!(date.weekday(), Weekday::Sat | Weekday::Sun) {
        return Err(Exclusion::Weekend);
    }
    let md = date.month() * 100 + date.day();
    if date.year() == 2025 && md == 109 {
        return Err(Exclusion::ExtraordinaryClosure);
    }
    if holidays.contains(&md) {
        return Err(Exclusion::PlannedHoliday);
    }
    let early: &[u32] = match date.year() {
        2023 => &[703, 1124],
        2024 => &[703, 1129, 1224],
        2025 => &[703, 1128, 1224],
        2026 => &[1127, 1224],
        2027 => &[1126],
        2028 => &[703, 1124],
        _ => unreachable!(),
    };
    Ok(if early.contains(&md) {
        13 * 60
    } else {
        16 * 60
    })
}

pub(crate) fn trading_date(label: &str) -> Option<NaiveDate> {
    Some(
        parse_minute_instant(label)?
            .with_timezone(&New_York)
            .date_naive(),
    )
}

pub(crate) fn kst_minute(label: &str) -> Option<i64> {
    let time = parse_minute_instant(label)?.with_timezone(&Seoul);
    Some(i64::from(time.hour() * 60 + time.minute()))
}

/// Full close-labelled [close-1m, close) must be inside the planned session.
/// Completion is owned by the upper preparation seam, not this timed clock.
pub(crate) fn regular_clock(instant: DateTime<Utc>) -> Result<RegularClock, Exclusion> {
    session_clock(instant, true)
}

/// Instantaneous display-price observations: open inclusive, close exclusive.
/// Reuses the research calendar, not a guarantee of extraordinary coverage.
pub(crate) fn quote_clock(instant: DateTime<Utc>) -> Result<RegularClock, Exclusion> {
    session_clock(instant, false)
}

fn session_clock(instant: DateTime<Utc>, close_label: bool) -> Result<RegularClock, Exclusion> {
    let ny = instant.with_timezone(&New_York);
    let date = ny.date_naive();
    let close_minute = schedule(date)?;
    let open = New_York
        .from_local_datetime(&date.and_hms_opt(9, 30, 0).ok_or(Exclusion::InvalidTime)?)
        .single()
        .ok_or(Exclusion::InvalidTime)?
        .to_utc();
    let close = New_York
        .from_local_datetime(
            &date
                .and_hms_opt((close_minute / 60) as u32, 0, 0)
                .ok_or(Exclusion::InvalidTime)?,
        )
        .single()
        .ok_or(Exclusion::InvalidTime)?
        .to_utc();
    if if close_label {
        instant - Duration::minutes(1) < open || instant > close
    } else {
        instant < open || instant >= close
    } {
        return Err(Exclusion::OutsideRegular);
    }
    let kst = instant.with_timezone(&Seoul);
    Ok(RegularClock {
        date,
        elapsed: (instant - open).num_minutes(),
        to_close: (close - instant).num_minutes(),
        blackout_kst_minute: i64::from(kst.hour() * 60 + kst.minute()),
        early_close: close_minute == 13 * 60,
    })
}

pub(crate) fn label_clock(label: &str) -> Result<RegularClock, Exclusion> {
    regular_clock(parse_minute_instant(label).ok_or(Exclusion::InvalidTime)?)
}
