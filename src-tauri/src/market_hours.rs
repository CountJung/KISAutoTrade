//! 시장 개장 시간 판별 모듈
//!
//! KST(UTC+9) 기준으로 KRX(국내)와 NYSE/NASDAQ(미국) 정규 세션 개장 여부를 판별한다.
//! API 호출 전 사전 체크로 불필요한 API 호출을 줄여 EGW00201 레이트 리밋을 방지한다.
//! 공휴일은 별도 달력 없이 KIS API 에러 응답(`장종료`, `장시작전` 등)으로 보완한다.
//!
//! ## 시간표 요약
//!
//! | 시장 | 정규 세션 (KST) | 비고 |
//! |------|---------------|------|
//! | KRX (국내) | 월~금 09:00 ~ 15:30 | 한국 공휴일 제외(API 에러로 보완) |
//! | NYSE/NASDAQ (해외) | 22:00 ~ 07:00 (야간) | 하계 22:30~05:00, 동계 23:30~06:00 기준 + 버퍼 |

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
        match value
            .unwrap_or("regular")
            .trim()
            .to_ascii_lowercase()
            .as_str()
        {
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
            .or_else(|| self.is_us_open_at(now))
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

/// 미국 주식 시장(NYSE/NASDAQ) 세션 개장 여부 (KST 기준 야간)
///
/// ## 변환 기준 (정규장 09:30 ~ 16:00 ET)
///
/// | DST 구분 | ET 오프셋 | KST 변환 |
/// |---------|---------|---------|
/// | 하계(EDT, UTC−4) | −4h | **22:30 ~ 05:00 KST** |
/// | 동계(EST, UTC−5) | −5h | **23:30 ~ 06:00 KST** |
///
/// 버퍼 포함 범위: **KST 22:00 ~ 07:00** (하계/동계 모두 커버)
///
/// ## 유효 요일 (KST 기준)
/// - `22:00 ~ 24:00`: 월~금 (당일 미국 시장 개장)
/// - `00:00 ~ 07:00`: 화~토 (전날 미국 시장 연속)
///
/// 미국 공휴일(추수감사절·크리스마스 등)은 KIS API 에러로 보완
pub fn is_us_open() -> bool {
    is_us_open_at(now_kst())
}

pub fn is_us_open_at(now: DateTime<FixedOffset>) -> bool {
    let weekday = now.weekday();
    let mins = now.hour() * 60 + now.minute();

    if mins >= 22 * 60 {
        // 22:00 이후: 월~금 야간 → 미국 당일 시장 개장
        matches!(
            weekday,
            Weekday::Mon | Weekday::Tue | Weekday::Wed | Weekday::Thu | Weekday::Fri
        )
    } else if mins < 7 * 60 {
        // 00:00 ~ 07:00: 화~토 새벽 → 전날 미국 야간 시장 연속
        matches!(
            weekday,
            Weekday::Tue | Weekday::Wed | Weekday::Thu | Weekday::Fri | Weekday::Sat
        )
    } else {
        false
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
    let now = now_kst();
    if is_domestic_symbol(symbol) {
        calendar
            .and_then(|override_calendar| override_calendar.is_kr_open_at(now))
            .unwrap_or_else(|| is_krx_open_at(now))
    } else {
        calendar
            .and_then(|override_calendar| override_calendar.is_us_open_at(now))
            .unwrap_or_else(|| is_us_open_at(now))
    }
}

pub fn is_market_open_for_with_calendar_policy(
    symbol: &str,
    calendar: Option<&MarketCalendarOverride>,
    us_policy: UsTradingSessionPolicy,
) -> bool {
    let now = now_kst();
    if is_domestic_symbol(symbol) {
        calendar
            .and_then(|override_calendar| override_calendar.is_kr_open_at(now))
            .unwrap_or_else(|| is_krx_open_at(now))
    } else {
        calendar
            .and_then(|override_calendar| {
                override_calendar.is_us_open_at_with_policy(now, us_policy)
            })
            .unwrap_or_else(|| is_us_open_at(now))
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
