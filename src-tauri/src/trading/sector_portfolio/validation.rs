use super::*;
use chrono::{DateTime, Datelike, NaiveDate, Weekday};
use std::collections::BTreeSet;

pub(super) fn date(value: &str) -> Result<NaiveDate, String> {
    let parsed = NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map_err(|_| format!("INVALID_SECTOR_DATE: {value}"))?;
    if parsed.format("%Y-%m-%d").to_string() != value
        || matches!(parsed.weekday(), Weekday::Sat | Weekday::Sun)
    {
        return Err(format!("INVALID_SECTOR_SESSION: {value}"));
    }
    Ok(parsed)
}

pub(super) fn days(rows: &[SectorDay]) -> Result<(), String> {
    let mut previous = None;
    for day in rows {
        let current = date(&day.date)?;
        if previous.is_some_and(|last| current <= last) {
            return Err(
                "NON_MONOTONIC_SECTOR_DATES: 중복·역순 날짜를 정렬로 숨기지 않습니다.".into(),
            );
        }
        previous = Some(current);
        if day.prices.len() != ALL_SYMBOLS.len()
            || ALL_SYMBOLS
                .iter()
                .any(|symbol| !day.prices.contains_key(*symbol))
        {
            return Err(format!(
                "INCOMPLETE_SECTOR_UNIVERSE: {}에는 SPY와 11개 전체 가격이 필요합니다.",
                day.date
            ));
        }
        for (symbol, bar) in &day.prices {
            if [bar.adjusted_close, bar.open, bar.close]
                .iter()
                .any(|value| !value.is_finite() || *value < 0.000_001 || *value > 1_000_000_000.0)
            {
                return Err(format!("INVALID_SECTOR_PRICE: {} {symbol}", day.date));
            }
        }
    }
    Ok(())
}

pub(super) fn history(rows: &[SectorDay]) -> Result<(), String> {
    if !(22..=500).contains(&rows.len()) {
        return Err("INSUFFICIENT_SECTOR_HISTORY: 전체 12종목의 이전 완료 일봉22개 이상·최대500개가 필요합니다.".into());
    }
    days(rows)
}

pub(super) fn input(input: &SectorPortfolioInput) -> Result<(), String> {
    if !input.initial_cash_usd.is_finite()
        || !(1.0..=1_000_000_000.0).contains(&input.initial_cash_usd)
        || !matches!(input.cost_bps, 10.0 | 20.0)
    {
        return Err(
            "INVALID_SECTOR_ASSUMPTIONS: 초기 USD자금1..1e9·편도 총비용10/20bp만 지원합니다."
                .into(),
        );
    }
    if input.history.len().saturating_add(input.evaluation.len()) > 500
        || input.evaluation.is_empty()
    {
        return Err(
            "INVALID_SECTOR_WINDOW: 평가봉1개 이상·사전자료와 합계500봉 이하가 필요합니다.".into(),
        );
    }
    history(&input.history)?;
    days(&input.evaluation)?;
    if input.history.last().expect("validated history").date
        >= input.evaluation.first().expect("validated evaluation").date
    {
        return Err(
            "SECTOR_HISTORY_OVERLAP: 사전자료는 평가 시작보다 엄격하게 이전이어야 합니다.".into(),
        );
    }
    let observed: Vec<_> = input
        .history
        .iter()
        .chain(&input.evaluation)
        .map(|day| day.date.as_str())
        .collect();
    if observed.len() != input.expected_session_dates.len()
        || observed
            .iter()
            .zip(&input.expected_session_dates)
            .any(|(a, b)| *a != b)
    {
        return Err("SECTOR_CALENDAR_MISMATCH: 선언한 거래일과 모든 사전/평가 날짜가 정확히 일치해야 합니다.".into());
    }
    if !input.corporate_actions.is_empty() {
        return Err("UNSUPPORTED_SECTOR_CORPORATE_ACTION: 분할·배당 권리/지급 원장이 아직 없어 해당 입력 전체의 성과를 평가하지 않습니다.".into());
    }
    let source = &input.source;
    if !source.coverage_verified || !source.corporate_actions_verified {
        return Err("UNVERIFIED_SECTOR_SOURCE: 거래일 coverage와 평가창 기업행위 부재 확인이 필요합니다(프로그램 자동 인증 아님).".into());
    }
    if source.adjustment_policy != "adjustedSignalRawExecution" {
        return Err("INVALID_SECTOR_PRICE_BASIS: 신호용 조정종가와 체결/평가용 비조정가격을 구분해야 합니다.".into());
    }
    for value in [&source.name, &source.calendar_source] {
        if value.trim().is_empty() || value.len() > 512 || value.chars().any(char::is_control) {
            return Err(
                "INVALID_SECTOR_SOURCE: 비어 있지 않은 출처/달력 메모(512byte 이하)가 필요합니다."
                    .into(),
            );
        }
    }
    if source.retrieved_at.len() > 64 {
        return Err("INVALID_SECTOR_RETRIEVAL_TIME".into());
    }
    let retrieved = DateTime::parse_from_rfc3339(&source.retrieved_at)
        .map_err(|_| "INVALID_SECTOR_RETRIEVAL_TIME".to_string())?;
    if retrieved.with_timezone(&chrono::Utc) > chrono::Utc::now() {
        return Err("FUTURE_SECTOR_RETRIEVAL: 미래 수집 시각을 허용하지 않습니다.".into());
    }
    let cutoff = retrieved
        .with_timezone(&chrono_tz::America::New_York)
        .date_naive();
    if date(&input.evaluation.last().expect("validated evaluation").date)? >= cutoff {
        return Err(
            "UNFINISHED_SECTOR_DAY: 수집시각의 뉴욕 날짜보다 이전인 완료 일봉만 허용합니다.".into(),
        );
    }
    Ok(())
}

pub(super) fn holdings(rows: &[SectorHolding], cash: f64) -> Result<(), String> {
    if !cash.is_finite() || cash < 0.0 {
        return Err("INVALID_SECTOR_CASH".into());
    }
    let mut seen = BTreeSet::new();
    for row in rows {
        if !ALL_SYMBOLS.contains(&row.symbol.as_str())
            || !seen.insert(&row.symbol)
            || row.quantity > 9_007_199_254_740_991
            || !row.average_cost_usd.is_finite()
            || row.average_cost_usd < 0.0
            || !row.remaining_buy_cost_usd.is_finite()
            || row.remaining_buy_cost_usd < 0.0
        {
            return Err("INVALID_SECTOR_HOLDING".into());
        }
    }
    Ok(())
}
