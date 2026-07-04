use std::time::Duration as StdDuration;

use crate::broker::{
    domain::{BrokerCurrency, BrokerMarket},
    rate_limit::RateLimitScheduler,
};
use anyhow::anyhow;

pub(super) fn toss_rate_limiter() -> RateLimitScheduler {
    RateLimitScheduler::with_min_intervals([
        ("toss:auth", StdDuration::from_millis(500)),
        ("toss:account", StdDuration::from_millis(200)),
        ("toss:order", StdDuration::from_millis(500)),
        ("toss:order_history", StdDuration::from_millis(200)),
        ("toss:market", StdDuration::from_millis(100)),
    ])
}

pub(super) fn toss_rate_limit_group(method: &str, path: &str) -> &'static str {
    if path.starts_with("/oauth2/") {
        "toss:auth"
    } else if path.starts_with("/api/v1/orders") && method.eq_ignore_ascii_case("GET") {
        "toss:order_history"
    } else if path.starts_with("/api/v1/orders") {
        "toss:order"
    } else if path.starts_with("/api/v1/accounts")
        || path.starts_with("/api/v1/holdings")
        || path.starts_with("/api/v1/buying-power")
        || path.starts_with("/api/v1/sellable-quantity")
        || path.starts_with("/api/v1/commissions")
    {
        "toss:account"
    } else {
        "toss:market"
    }
}

pub(super) fn toss_currency(value: &str) -> anyhow::Result<BrokerCurrency> {
    match value {
        "KRW" => Ok(BrokerCurrency::Krw),
        "USD" => Ok(BrokerCurrency::Usd),
        other => Err(anyhow!("지원하지 않는 Toss currency: {other}")),
    }
}

pub(super) fn toss_currency_code(value: BrokerCurrency) -> &'static str {
    match value {
        BrokerCurrency::Krw => "KRW",
        BrokerCurrency::Usd => "USD",
    }
}

pub(super) fn toss_market(value: &str) -> anyhow::Result<BrokerMarket> {
    match value {
        "KR" => Ok(BrokerMarket::Kr),
        "US" => Ok(BrokerMarket::Us),
        other => Err(anyhow!("지원하지 않는 Toss marketCountry: {other}")),
    }
}

pub(super) fn market_from_currency(currency: BrokerCurrency) -> BrokerMarket {
    match currency {
        BrokerCurrency::Krw => BrokerMarket::Kr,
        BrokerCurrency::Usd => BrokerMarket::Us,
    }
}

pub(super) fn validate_toss_symbol(symbol: &str) -> anyhow::Result<()> {
    if symbol.is_empty() || symbol.len() > 32 {
        return Err(anyhow!("토스증권 symbol은 1~32자여야 합니다: {symbol}"));
    }
    if !symbol
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-')
    {
        return Err(anyhow!(
            "토스증권 symbol은 영문/숫자/./- 문자만 허용합니다: {symbol}"
        ));
    }
    Ok(())
}

pub(super) fn validate_toss_order_id(order_id: &str) -> anyhow::Result<()> {
    if order_id.is_empty() {
        return Err(anyhow!("토스증권 orderId는 비어 있을 수 없습니다"));
    }
    if !order_id
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(anyhow!(
            "토스증권 orderId는 영문/숫자/-/_ 문자만 허용합니다: {order_id}"
        ));
    }
    Ok(())
}

pub(super) fn validate_client_order_id(value: Option<&str>) -> anyhow::Result<()> {
    let Some(value) = value else {
        return Ok(());
    };
    if value.is_empty() || value.len() > 36 {
        return Err(anyhow!(
            "토스증권 clientOrderId는 1~36자여야 합니다: {value}"
        ));
    }
    if !value
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(anyhow!(
            "토스증권 clientOrderId는 영문/숫자/-/_ 문자만 허용합니다: {value}"
        ));
    }
    Ok(())
}

pub(super) fn new_toss_client_order_id() -> String {
    format!("ka-{}", uuid::Uuid::new_v4().simple())
}

pub(super) fn validate_order_side(value: &str) -> anyhow::Result<()> {
    if matches!(value, "BUY" | "SELL") {
        Ok(())
    } else {
        Err(anyhow!(
            "토스증권 주문 방향은 BUY 또는 SELL만 허용합니다: {value}"
        ))
    }
}

pub(super) fn validate_order_type(value: &str) -> anyhow::Result<()> {
    if matches!(value, "LIMIT" | "MARKET") {
        Ok(())
    } else {
        Err(anyhow!(
            "토스증권 주문 유형은 LIMIT 또는 MARKET만 허용합니다: {value}"
        ))
    }
}

pub(super) fn validate_time_in_force(value: &str) -> anyhow::Result<()> {
    if matches!(value, "DAY" | "CLS") {
        Ok(())
    } else {
        Err(anyhow!(
            "토스증권 timeInForce는 DAY 또는 CLS만 허용합니다: {value}"
        ))
    }
}

pub(super) fn validate_optional_decimal(field: &str, value: Option<&str>) -> anyhow::Result<()> {
    let Some(value) = value else {
        return Ok(());
    };
    if value.is_empty()
        || value.len() > 30
        || !value.chars().all(|c| c.is_ascii_digit() || c == '.')
        || value.matches('.').count() > 1
        || value == "."
    {
        return Err(anyhow!(
            "토스증권 {field}는 양수 decimal 문자열이어야 합니다: {value}"
        ));
    }
    Ok(())
}

pub(super) fn validate_iso_date(date: &str) -> anyhow::Result<()> {
    let is_valid = date.len() == 10
        && date.chars().enumerate().all(|(index, ch)| {
            matches!(index, 4 | 7) && ch == '-' || !matches!(index, 4 | 7) && ch.is_ascii_digit()
        });
    if !is_valid {
        return Err(anyhow!(
            "토스증권 market-calendar date는 YYYY-MM-DD 형식이어야 합니다: {date}"
        ));
    }
    Ok(())
}
