//! Shared syntax only: RFC3339 instants or legacy compact KST minute labels.
use chrono::{DateTime, FixedOffset, NaiveDateTime, TimeZone, Utc};

/// Does not infer market sessions, completion, host timezones, or date-only labels.
pub(crate) fn parse_minute_instant(value: &str) -> Option<DateTime<Utc>> {
    let instant = DateTime::parse_from_rfc3339(value).ok().or_else(|| {
        if !value.bytes().all(|byte| byte.is_ascii_digit()) {
            return None;
        }
        let compact = match value.len() {
            12 => format!("{value}00"),
            14 => value.to_string(),
            _ => return None,
        };
        let naive = NaiveDateTime::parse_from_str(&compact, "%Y%m%d%H%M%S").ok()?;
        FixedOffset::east_opt(9 * 60 * 60)?
            .from_local_datetime(&naive)
            .single()
    })?;
    Some(instant.with_timezone(&Utc))
}
