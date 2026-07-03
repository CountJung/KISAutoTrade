use crate::broker::{BrokerCurrency, BrokerMoney, BrokerOrderSide, BrokerQuantity};

#[derive(Debug, Clone)]
pub struct OrderPreflightInput {
    pub side: BrokerOrderSide,
    pub quantity: BrokerQuantity,
    pub price: BrokerMoney,
}

#[derive(Debug, Clone, Default)]
pub struct OrderPreflightConstraints {
    pub buying_power: Option<BrokerMoney>,
    pub sellable_quantity: Option<BrokerQuantity>,
    /// Toss returns `commissionRate` as a percent string: `0.015` means 0.015%.
    pub commission_rate_percent: Option<String>,
}

#[derive(Debug, Clone)]
pub struct OrderPreflightDecision {
    pub liquidity_ok: bool,
    pub gross_amount: BrokerMoney,
    pub estimated_commission: Option<BrokerMoney>,
    pub required_cash: Option<BrokerMoney>,
    pub blocked_reasons: Vec<String>,
}

pub fn evaluate_order_preflight(
    input: &OrderPreflightInput,
    constraints: &OrderPreflightConstraints,
) -> OrderPreflightDecision {
    let mut blocked_reasons = Vec::new();
    let quantity = parse_decimal_amount(&input.quantity.0).unwrap_or(0.0);
    let price = parse_decimal_amount(&input.price.amount).unwrap_or(0.0);
    if quantity <= 0.0 {
        blocked_reasons.push("주문 수량은 0보다 커야 합니다.".to_string());
    }
    if price <= 0.0 {
        blocked_reasons.push("주문 가격은 0보다 커야 합니다.".to_string());
    }

    let gross = (quantity * price).max(0.0);
    let commission_rate = constraints
        .commission_rate_percent
        .as_deref()
        .and_then(parse_decimal_amount)
        .unwrap_or(0.0)
        .max(0.0)
        / 100.0;
    let commission = gross * commission_rate;
    let required_cash = gross + commission;

    match input.side {
        BrokerOrderSide::Buy => match constraints.buying_power.as_ref() {
            Some(power) if power.currency == input.price.currency => {
                let available = parse_decimal_amount(&power.amount).unwrap_or(0.0);
                if available < required_cash {
                    blocked_reasons.push(format!(
                        "매수가능금액 부족: 필요 {} / 가능 {}",
                        format_money_amount(required_cash, input.price.currency),
                        format_money_amount(available, input.price.currency)
                    ));
                }
            }
            Some(power) => blocked_reasons.push(format!(
                "매수가능금액 통화 불일치: 주문 {:?}, 조회 {:?}",
                input.price.currency, power.currency
            )),
            None => blocked_reasons.push("매수가능금액을 확인하지 못했습니다.".to_string()),
        },
        BrokerOrderSide::Sell => match constraints.sellable_quantity.as_ref() {
            Some(sellable) => {
                let available = parse_decimal_amount(&sellable.0).unwrap_or(0.0);
                if available < quantity {
                    blocked_reasons.push(format!(
                        "매도가능수량 부족: 필요 {} / 가능 {}",
                        format_quantity(quantity),
                        format_quantity(available)
                    ));
                }
            }
            None => blocked_reasons.push("매도가능수량을 확인하지 못했습니다.".to_string()),
        },
    }

    OrderPreflightDecision {
        liquidity_ok: blocked_reasons.is_empty(),
        gross_amount: BrokerMoney {
            amount: format_money_amount(gross, input.price.currency),
            currency: input.price.currency,
        },
        estimated_commission: Some(BrokerMoney {
            amount: format_money_amount(commission, input.price.currency),
            currency: input.price.currency,
        }),
        required_cash: (input.side == BrokerOrderSide::Buy).then(|| BrokerMoney {
            amount: format_money_amount(required_cash, input.price.currency),
            currency: input.price.currency,
        }),
        blocked_reasons,
    }
}

pub fn parse_decimal_amount(value: &str) -> Option<f64> {
    let normalized = value.trim().replace(',', "");
    if normalized.is_empty() {
        return None;
    }
    normalized.parse::<f64>().ok().filter(|v| v.is_finite())
}

pub fn format_money_amount(value: f64, currency: BrokerCurrency) -> String {
    let value = value.max(0.0);
    match currency {
        BrokerCurrency::Krw => format!("{:.0}", value.ceil()),
        BrokerCurrency::Usd => format!("{:.4}", value),
    }
}

fn format_quantity(value: f64) -> String {
    if (value.fract()).abs() < f64::EPSILON {
        format!("{:.0}", value)
    } else {
        format!("{:.6}", value)
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buy_preflight_adds_commission_and_blocks_when_cash_is_short() {
        let input = OrderPreflightInput {
            side: BrokerOrderSide::Buy,
            quantity: BrokerQuantity("10".to_string()),
            price: BrokerMoney::krw("1000"),
        };
        let decision = evaluate_order_preflight(
            &input,
            &OrderPreflightConstraints {
                buying_power: Some(BrokerMoney::krw("10000")),
                sellable_quantity: None,
                commission_rate_percent: Some("0.015".to_string()),
            },
        );

        assert!(!decision.liquidity_ok);
        assert_eq!(decision.gross_amount.amount, "10000");
        assert_eq!(decision.estimated_commission.unwrap().amount, "2");
        assert_eq!(decision.required_cash.unwrap().amount, "10002");
        assert!(decision.blocked_reasons[0].contains("매수가능금액 부족"));
    }

    #[test]
    fn sell_preflight_blocks_when_sellable_quantity_is_short() {
        let input = OrderPreflightInput {
            side: BrokerOrderSide::Sell,
            quantity: BrokerQuantity("5.5".to_string()),
            price: BrokerMoney::usd("20"),
        };
        let decision = evaluate_order_preflight(
            &input,
            &OrderPreflightConstraints {
                buying_power: None,
                sellable_quantity: Some(BrokerQuantity("5.25".to_string())),
                commission_rate_percent: Some("0.1".to_string()),
            },
        );

        assert!(!decision.liquidity_ok);
        assert_eq!(decision.gross_amount.amount, "110.0000");
        assert_eq!(decision.estimated_commission.unwrap().amount, "0.1100");
        assert!(decision.blocked_reasons[0].contains("매도가능수량 부족"));
    }

    #[test]
    fn buy_preflight_passes_when_required_cash_is_available() {
        let input = OrderPreflightInput {
            side: BrokerOrderSide::Buy,
            quantity: BrokerQuantity("2".to_string()),
            price: BrokerMoney::usd("100"),
        };
        let decision = evaluate_order_preflight(
            &input,
            &OrderPreflightConstraints {
                buying_power: Some(BrokerMoney::usd("201")),
                sellable_quantity: None,
                commission_rate_percent: Some("0.1".to_string()),
            },
        );

        assert!(decision.liquidity_ok);
        assert_eq!(decision.required_cash.unwrap().amount, "200.2000");
    }
}
