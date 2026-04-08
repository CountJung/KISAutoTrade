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

use chrono::{Datelike, Timelike, Weekday};

/// 시스템 타임존에 무관하게 현재 KST 시각 반환 (UTC+9 고정)
fn now_kst() -> chrono::DateTime<chrono::FixedOffset> {
    let kst = chrono::FixedOffset::east_opt(9 * 3600).expect("KST FixedOffset 생성 실패");
    chrono::Utc::now().with_timezone(&kst)
}

/// 국내 주식 종목코드 판별
///
/// KRX 종목코드: 6자리, 첫 글자가 ASCII 숫자
/// - 일반 주식: `005930` (숫자 6자리)
/// - ETF 등:   `0005A0` (숫자+알파벳 혼합)
pub fn is_domestic_symbol(symbol: &str) -> bool {
    symbol.len() == 6 && symbol.chars().next().map_or(false, |c| c.is_ascii_digit())
}

/// KRX 정규 세션 개장 여부 (KST 기준)
///
/// - 월~금, **09:00 ~ 15:30 KST**
/// - 한국 공휴일은 미포함 → KIS API `장종료` 에러 응답으로 사후 보완
pub fn is_krx_open() -> bool {
    let now = now_kst();
    if matches!(now.weekday(), Weekday::Sat | Weekday::Sun) {
        return false;
    }
    let mins = now.hour() * 60 + now.minute();
    // 09:00 = 540분, 15:30 = 930분
    mins >= 540 && mins < 930
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
    let now = now_kst();
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

/// 현재 개장 중인 시장 요약 문자열 (로그 출력용)
pub fn open_markets_summary() -> &'static str {
    match (is_krx_open(), is_us_open()) {
        (true,  true)  => "KRX 개장 / US 개장",
        (true,  false) => "KRX 개장 / US 폐장",
        (false, true)  => "KRX 폐장 / US 개장",
        (false, false) => "KRX 폐장 / US 폐장",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn domestic_symbol_detection() {
        assert!(is_domestic_symbol("005930"));  // 삼성전자
        assert!(is_domestic_symbol("069500"));  // KODEX 200
        assert!(is_domestic_symbol("0005A0"));  // ETF 알파벳 포함
        assert!(!is_domestic_symbol("AAPL"));   // 해외
        assert!(!is_domestic_symbol("QQQM"));   // 해외
        assert!(!is_domestic_symbol("12345"));  // 5자리 (잘못된 코드)
        assert!(!is_domestic_symbol("A05930")); // 첫 글자 알파벳
    }
}
