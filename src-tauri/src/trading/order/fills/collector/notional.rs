//! Checked decimal conversion before rounding a native-currency total once.
use super::*;
use crate::broker::BrokerCurrency;

impl OrderManager {
    pub(crate) fn parse_fill_quantity(value: &str) -> Result<u64> {
        execution_quantity(value)
    }

    pub(crate) fn parse_fill_execution_date(value: &str) -> Option<chrono::NaiveDate> {
        super::parse_provider_execution_date(value)
    }
    pub(crate) fn parse_fill_notional(value: &str, currency: BrokerCurrency) -> Result<u128> {
        scaled_total(value, currency, 1)
    }
}

fn parts(value: &str) -> Result<(u128, u128)> {
    let value = value.trim();
    anyhow::ensure!(
        !value.is_empty() && value.len() <= 128,
        "체결 금액 decimal 길이 오류"
    );
    let mut split = value.split('.');
    let raw_integer = split.next().unwrap_or_default();
    let groups: Vec<_> = raw_integer.split(',').collect();
    anyhow::ensure!(
        groups.len() == 1
            || ((1..=3).contains(&groups[0].len())
                && groups.iter().skip(1).all(|group| group.len() == 3)),
        "체결 금액의 자릿수 구분 형식 오류"
    );
    let integer = groups.join("");
    let fraction = split.next().unwrap_or_default();
    anyhow::ensure!(
        split.next().is_none()
            && !integer.is_empty()
            && integer.bytes().all(|byte| byte.is_ascii_digit())
            && fraction.bytes().all(|byte| byte.is_ascii_digit())
            && fraction.len() <= 38,
        "체결 금액은 음수가 아닌 고정 decimal 문자열이어야 합니다"
    );
    let denominator = 10u128
        .checked_pow(u32::try_from(fraction.len())?)
        .ok_or_else(|| anyhow::anyhow!("체결 금액 소수 정밀도 범위 오류"))?;
    let digits = format!("{integer}{fraction}");
    Ok((digits.parse()?, denominator))
}

pub(super) fn scaled_total(value: &str, currency: BrokerCurrency, quantity: u64) -> Result<u128> {
    let (coefficient, denominator) = parts(value)?;
    let scale = match currency {
        BrokerCurrency::Krw => 1u128,
        BrokerCurrency::Usd => 100u128,
    };
    let numerator = coefficient
        .checked_mul(u128::from(quantity))
        .and_then(|amount| amount.checked_mul(scale))
        .ok_or_else(|| anyhow::anyhow!("체결 누적 금액 범위 오류"))?;
    let whole = numerator / denominator;
    let round_up = numerator % denominator >= denominator.div_ceil(2);
    whole
        .checked_add(u128::from(round_up))
        .ok_or_else(|| anyhow::anyhow!("체결 누적 금액 반올림 범위 오류"))
}

pub(super) fn execution_quantity(value: &str) -> Result<u64> {
    let (numerator, denominator) = parts(value)?;
    anyhow::ensure!(
        numerator % denominator == 0,
        "체결 수량에 정수 주식으로 확인할 수 없는 잔여가 있습니다"
    );
    Ok(u64::try_from(numerator / denominator)?)
}

pub(super) fn toss_amount(
    order: &crate::broker::toss::TossOrder,
    symbol: &str,
    quantity: u64,
) -> Result<u128> {
    let currency = match order.currency.trim().to_ascii_uppercase().as_str() {
        "KRW" if is_domestic_symbol(symbol) => BrokerCurrency::Krw,
        "USD" if !is_domestic_symbol(symbol) => BrokerCurrency::Usd,
        _ => anyhow::bail!("Toss 체결 종목의 시장과 응답 통화가 일치하지 않습니다"),
    };
    let amount = match order.execution.filled_amount.as_deref() {
        Some(amount) => OrderManager::parse_fill_notional(amount, currency)?,
        None => scaled_total(
            order
                .execution
                .average_filled_price
                .as_deref()
                .ok_or_else(|| anyhow::anyhow!("Toss 체결 누적 금액과 실제 평균가가 없습니다"))?,
            currency,
            quantity,
        )?,
    };
    anyhow::ensure!(
        quantity == 0 || amount > 0,
        "Toss 양수 체결 수량의 누적 금액이 0입니다"
    );
    Ok(amount)
}

#[cfg(test)]
mod tests;
