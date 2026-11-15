//! 시장 개장 시간 사전 검사. 국내 기존 KST 규칙은 유지한다.
//! 미국 기본 경로는 계획 달력(2023–2028), NY 정규장/DST/조기종료를 적용한다.
//! 달력 범위 밖은 차단한다. 임시휴장 전체를 실시간 인증하는 달력은 아니다.
//! Toss 명시 세션은 공급자 달력대로 유지하며 누락 시 임의 시간표로 대체하지 않는다.

use chrono::{DateTime, Datelike, FixedOffset, Timelike, Weekday};

/// 시스템 타임존에 무관하게 현재 KST 시각 반환 (UTC+9 고정)
fn now_kst() -> chrono::DateTime<chrono::FixedOffset> {
    let kst = chrono::FixedOffset::east_opt(9 * 3600).expect("KST FixedOffset 생성 실패");
    chrono::Utc::now().with_timezone(&kst)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarketSessionWindow {
    pub start_at: DateTime<FixedOffset>,
    pub end_at: DateTime<FixedOffset>,
}

impl MarketSessionWindow {
    pub fn parse(start_at: &str, end_at: &str) -> Option<Self> {
        let start_at = DateTime::parse_from_rfc3339(start_at).ok()?;
        let end_at = DateTime::parse_from_rfc3339(end_at).ok()?;
        (start_at < end_at).then_some(Self { start_at, end_at })
    }

    pub fn contains(&self, now: DateTime<FixedOffset>) -> bool {
        now >= self.start_at && now < self.end_at
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarketDayCalendar {
    pub regular_session: Option<MarketSessionWindow>,
}

impl MarketDayCalendar {
    pub fn regular(start_at: &str, end_at: &str) -> Option<Self> {
        Some(Self {
            regular_session: Some(MarketSessionWindow::parse(start_at, end_at)?),
        })
    }

    pub fn closed() -> Self {
        Self {
            regular_session: None,
        }
    }

    pub fn is_open_at(&self, now: DateTime<FixedOffset>) -> bool {
        self.regular_session
            .as_ref()
            .is_some_and(|session| session.contains(now))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsTradingSessionPolicy {
    Auto,
    Day,
    Pre,
    Regular,
    After,
}

impl UsTradingSessionPolicy {
    pub fn parse(value: Option<&str>) -> Self {
        match value.unwrap_or("auto").trim().to_ascii_lowercase().as_str() {
            "auto" => Self::Auto,
            "day" | "daymarket" | "day_market" => Self::Day,
            "pre" | "premarket" | "pre_market" => Self::Pre,
            "after" | "aftermarket" | "after_market" => Self::After,
            _ => Self::Regular,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UsMarketSessionCalendar {
    pub day_session: Option<MarketSessionWindow>,
    pub pre_session: Option<MarketSessionWindow>,
    pub regular_session: Option<MarketSessionWindow>,
    pub after_session: Option<MarketSessionWindow>,
}

impl UsMarketSessionCalendar {
    pub fn is_open_at(&self, policy: UsTradingSessionPolicy, now: DateTime<FixedOffset>) -> bool {
        let contains = |session: &Option<MarketSessionWindow>| {
            session.as_ref().is_some_and(|s| s.contains(now))
        };
        match policy {
            UsTradingSessionPolicy::Auto => {
                contains(&self.day_session)
                    || contains(&self.pre_session)
                    || contains(&self.regular_session)
                    || contains(&self.after_session)
            }
            UsTradingSessionPolicy::Day => contains(&self.day_session),
            UsTradingSessionPolicy::Pre => contains(&self.pre_session),
            UsTradingSessionPolicy::Regular => contains(&self.regular_session),
            UsTradingSessionPolicy::After => contains(&self.after_session),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MarketCalendarOverride {
    pub kr: Option<MarketDayCalendar>,
    pub us: Option<MarketDayCalendar>,
    pub us_sessions: Option<UsMarketSessionCalendar>,
}

impl MarketCalendarOverride {
    pub fn is_kr_open_at(&self, now: DateTime<FixedOffset>) -> Option<bool> {
        self.kr.as_ref().map(|day| day.is_open_at(now))
    }

    pub fn is_us_open_at(&self, now: DateTime<FixedOffset>) -> Option<bool> {
        self.us.as_ref().map(|day| day.is_open_at(now))
    }

    pub fn is_us_open_at_with_policy(
        &self,
        now: DateTime<FixedOffset>,
        policy: UsTradingSessionPolicy,
    ) -> Option<bool> {
        self.us_sessions
            .as_ref()
            .map(|sessions| sessions.is_open_at(policy, now))
            .or_else(|| match policy {
                UsTradingSessionPolicy::Auto | UsTradingSessionPolicy::Regular => {
                    self.is_us_open_at(now)
                }
                _ => None,
            })
    }
}

/// 국내 주식 종목코드 판별
///
/// KRX 종목코드: 6자리, 첫 글자가 ASCII 숫자
/// - 일반 주식: `005930` (숫자 6자리)
/// - ETF 등:   `0005A0` (숫자+알파벳 혼합)
pub fn is_domestic_symbol(symbol: &str) -> bool {
    symbol.len() == 6 && symbol.chars().next().is_some_and(|c| c.is_ascii_digit())
}

/// KRX 정규 세션 개장 여부 (KST 기준)
///
/// - 월~금, **09:00 ~ 15:30 KST**
/// - 한국 공휴일은 미포함 → KIS API `장종료` 에러 응답으로 사후 보완
pub fn is_krx_open() -> bool {
    is_krx_open_at(now_kst())
}

pub fn is_krx_open_at(now: DateTime<FixedOffset>) -> bool {
    if matches!(now.weekday(), Weekday::Sat | Weekday::Sun) {
        return false;
    }
    let mins = now.hour() * 60 + now.minute();
    // 09:00 = 540분, 15:30 = 930분
    (540..930).contains(&mins)
}

/// 미국 계획 정규장: 개장 포함/종료 제외. 출처·범위 한계는 공유 달력 참조.
pub fn is_us_open() -> bool {
    is_us_open_at(now_kst())
}

pub fn is_us_open_at(now: DateTime<FixedOffset>) -> bool {
    crate::trading::strategy::lth_research_session::quote_clock(now.to_utc()).is_ok()
}

/// Local automatic KIS rejection: no provider request was sent in this attempt.
#[derive(Debug, thiserror::Error)]
#[error("US_SESSION_CLOSED: 미국 정규장 장운영시간 밖이거나 계획 달력 미확인으로 자동 주문을 보류합니다.")]
pub(crate) struct UsSessionClosed;

pub(crate) fn ensure_kis_automatic_us_session(
    enforce: bool,
    now: chrono::DateTime<chrono::Utc>,
) -> Result<(), UsSessionClosed> {
    if enforce && !is_us_open_at(now.fixed_offset()) {
        Err(UsSessionClosed)
    } else {
        Ok(())
    }
}

/// 해당 종목의 시장이 현재 개장 중인지 여부
///
/// - 국내 종목(6자리, 첫 글자 숫자): [`is_krx_open`] 반환
/// - 해외 종목: [`is_us_open`] 반환
pub fn is_market_open_for(symbol: &str) -> bool {
    if is_domestic_symbol(symbol) {
        is_krx_open()
    } else {
        is_us_open()
    }
}

pub fn is_market_open_for_with_calendar(
    symbol: &str,
    calendar: Option<&MarketCalendarOverride>,
) -> bool {
    is_market_open_for_with_calendar_at(symbol, calendar, now_kst())
}

pub fn is_market_open_for_with_calendar_at(
    symbol: &str,
    calendar: Option<&MarketCalendarOverride>,
    now: DateTime<FixedOffset>,
) -> bool {
    if is_domestic_symbol(symbol) {
        calendar
            .and_then(|override_calendar| override_calendar.is_kr_open_at(now))
            .unwrap_or_else(|| is_krx_open_at(now))
    } else {
        is_us_open_at(now)
            && calendar
                .and_then(|override_calendar| override_calendar.is_us_open_at(now))
                .unwrap_or(true)
    }
}

pub fn is_market_open_for_with_calendar_policy(
    symbol: &str,
    calendar: Option<&MarketCalendarOverride>,
    us_policy: UsTradingSessionPolicy,
) -> bool {
    is_market_open_for_with_calendar_policy_at(symbol, calendar, us_policy, now_kst())
}

pub fn is_market_open_for_with_calendar_policy_at(
    symbol: &str,
    calendar: Option<&MarketCalendarOverride>,
    us_policy: UsTradingSessionPolicy,
    now: DateTime<FixedOffset>,
) -> bool {
    if is_domestic_symbol(symbol) {
        calendar
            .and_then(|override_calendar| override_calendar.is_kr_open_at(now))
            .unwrap_or_else(|| is_krx_open_at(now))
    } else {
        calendar
            .and_then(|override_calendar| {
                override_calendar.is_us_open_at_with_policy(now, us_policy)
            })
            .unwrap_or(false)
    }
}

/// 현재 개장 중인 시장 요약 문자열 (로그 출력용)
pub fn open_markets_summary() -> &'static str {
    match (is_krx_open(), is_us_open()) {
        (true, true) => "KRX 개장 / US 개장",
        (true, false) => "KRX 개장 / US 폐장",
        (false, true) => "KRX 폐장 / US 개장",
        (false, false) => "KRX 폐장 / US 폐장",
    }
}

pub fn open_markets_summary_with_calendar(calendar: Option<&MarketCalendarOverride>) -> String {
    let now = now_kst();
    let kr_open = calendar
        .and_then(|override_calendar| override_calendar.is_kr_open_at(now))
        .unwrap_or_else(|| is_krx_open_at(now));
    let us_open = calendar
        .and_then(|override_calendar| override_calendar.is_us_open_at(now))
        .unwrap_or_else(|| is_us_open_at(now));
    match (kr_open, us_open) {
        (true, true) => "KRX 개장 / US 개장",
        (true, false) => "KRX 개장 / US 폐장",
        (false, true) => "KRX 폐장 / US 개장",
        (false, false) => "KRX 폐장 / US 폐장",
    }
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(s: &str) -> DateTime<FixedOffset> {
        DateTime::parse_from_rfc3339(s).unwrap()
    }

    #[test]
    fn planned_us_session_blocks_holidays_unknown_year_and_observes_dst_and_early_close() {
        for open in [
            "2026-07-06T13:30:00Z",
            "2026-11-09T14:30:00Z",
            "2026-11-27T17:59:59Z",
        ] {
            assert!(is_us_open_at(at(open)), "{open}");
        }
        for closed in [
            "2026-11-09T14:29:59Z",
            "2026-11-09T21:00:00Z",
            "2026-07-06T20:00:00Z",
            "2026-11-26T15:00:00Z",
            "2026-11-27T18:00:00Z",
            "2026-11-08T15:00:00Z",
            "2029-11-09T15:00:00Z",
        ] {
            assert!(!is_us_open_at(at(closed)), "{closed}");
        }
    }

    #[test]
    fn kis_none_uses_strict_calendar_but_toss_missing_calendar_fails_closed() {
        let now = at("2026-11-09T15:00:00Z");
        assert!(is_market_open_for_with_calendar_at("AAPL", None, now));
        assert!(!is_market_open_for_with_calendar_policy_at(
            "AAPL",
            None,
            UsTradingSessionPolicy::Auto,
            now
        ));
        let calendar = MarketCalendarOverride {
            kr: None,
            us: Some(MarketDayCalendar::closed()),
            us_sessions: None,
        };
        assert!(!is_market_open_for_with_calendar_at(
            "AAPL",
            Some(&calendar),
            now
        ));
        let holiday = at("2026-11-26T15:00:00Z");
        let calendar = MarketCalendarOverride {
            kr: None,
            us: MarketDayCalendar::regular("2026-11-26T14:30:00Z", "2026-11-26T21:00:00Z"),
            us_sessions: None,
        };
        assert!(!is_market_open_for_with_calendar_at(
            "AAPL",
            Some(&calendar),
            holiday
        ));
    }

    #[test]
    fn explicit_toss_extended_calendar_is_not_narrowed_to_regular_session() {
        let now = at("2026-11-09T22:00:00Z");
        let calendar = MarketCalendarOverride {
            kr: None,
            us: None,
            us_sessions: Some(UsMarketSessionCalendar {
                after_session: MarketSessionWindow::parse(
                    "2026-11-09T21:00:00Z",
                    "2026-11-10T01:00:00Z",
                ),
                ..Default::default()
            }),
        };
        assert!(!is_us_open_at(now));
        assert!(is_market_open_for_with_calendar_policy_at(
            "AAPL",
            Some(&calendar),
            UsTradingSessionPolicy::After,
            now
        ));
        assert!(is_market_open_for_with_calendar_policy_at(
            "AAPL",
            Some(&calendar),
            UsTradingSessionPolicy::Auto,
            now
        ));
        assert!(!is_market_open_for_with_calendar_policy_at(
            "AAPL",
            Some(&calendar),
            UsTradingSessionPolicy::Regular,
            now
        ));
    }

    #[test]
    fn incomplete_legacy_calendar_cannot_substitute_regular_for_extended_policy() {
        let now = at("2026-11-09T15:00:00Z");
        let calendar = MarketCalendarOverride {
            kr: None,
            us: MarketDayCalendar::regular("2026-11-09T14:30:00Z", "2026-11-09T21:00:00Z"),
            us_sessions: None,
        };
        assert!(is_market_open_for_with_calendar_policy_at(
            "AAPL",
            Some(&calendar),
            UsTradingSessionPolicy::Regular,
            now
        ));
        for policy in [
            UsTradingSessionPolicy::Day,
            UsTradingSessionPolicy::Pre,
            UsTradingSessionPolicy::After,
        ] {
            assert!(!is_market_open_for_with_calendar_policy_at(
                "AAPL",
                Some(&calendar),
                policy,
                now
            ));
        }
    }

    #[test]
    fn domestic_symbol_detection() {
        assert!(is_domestic_symbol("005930")); // 삼성전자
        assert!(is_domestic_symbol("069500")); // KODEX 200
        assert!(is_domestic_symbol("0005A0")); // ETF 알파벳 포함
        assert!(!is_domestic_symbol("AAPL")); // 해외
        assert!(!is_domestic_symbol("QQQM")); // 해외
        assert!(!is_domestic_symbol("12345")); // 5자리 (잘못된 코드)
        assert!(!is_domestic_symbol("A05930")); // 첫 글자 알파벳
    }

    #[test]
    fn calendar_session_window_checks_time_range() {
        let session =
            MarketSessionWindow::parse("2026-03-25T09:00:00+09:00", "2026-03-25T15:30:00+09:00")
                .unwrap();
        let open = DateTime::parse_from_rfc3339("2026-03-25T10:00:00+09:00").unwrap();
        let closed = DateTime::parse_from_rfc3339("2026-03-25T16:00:00+09:00").unwrap();

        assert!(session.contains(open));
        assert!(!session.contains(closed));
    }

    #[test]
    fn calendar_override_represents_holiday_without_fallback() {
        let calendar = MarketCalendarOverride {
            kr: Some(MarketDayCalendar::closed()),
            us: None,
            us_sessions: None,
        };
        let open = DateTime::parse_from_rfc3339("2026-03-25T10:00:00+09:00").unwrap();

        assert_eq!(calendar.is_kr_open_at(open), Some(false));
        assert_eq!(calendar.is_us_open_at(open), None);
    }

    #[test]
    fn us_session_policy_checks_extended_sessions() {
        let calendar = MarketCalendarOverride {
            kr: None,
            us: None,
            us_sessions: Some(UsMarketSessionCalendar {
                day_session: MarketSessionWindow::parse(
                    "2026-03-25T09:00:00+09:00",
                    "2026-03-25T16:50:00+09:00",
                ),
                pre_session: None,
                regular_session: MarketSessionWindow::parse(
                    "2026-03-25T22:30:00+09:00",
                    "2026-03-26T05:00:00+09:00",
                ),
                after_session: MarketSessionWindow::parse(
                    "2026-03-26T05:00:00+09:00",
                    "2026-03-26T07:00:00+09:00",
                ),
            }),
        };
        let day = DateTime::parse_from_rfc3339("2026-03-25T10:00:00+09:00").unwrap();
        let regular = DateTime::parse_from_rfc3339("2026-03-25T23:00:00+09:00").unwrap();
        let after = DateTime::parse_from_rfc3339("2026-03-26T06:00:00+09:00").unwrap();

        assert_eq!(
            calendar.is_us_open_at_with_policy(day, UsTradingSessionPolicy::Day),
            Some(true)
        );
        assert_eq!(
            calendar.is_us_open_at_with_policy(day, UsTradingSessionPolicy::Regular),
            Some(false)
        );
        assert_eq!(
            calendar.is_us_open_at_with_policy(regular, UsTradingSessionPolicy::Auto),
            Some(true)
        );
        assert_eq!(
            calendar.is_us_open_at_with_policy(after, UsTradingSessionPolicy::After),
            Some(true)
        );
    }
}
