use super::*;

pub(crate) async fn fetch_account_risk_balance_krw(
    rest: &Arc<KisRestClient>,
    include_overseas: bool,
    exchange_rate_krw: f64,
) -> Result<i64, String> {
    let domestic = rest
        .get_balance()
        .await
        .map_err(|e| format!("KIS 국내 잔고 조회 실패: {e}"))?;
    let domestic_total = balance_summary_total_krw(domestic.summary.as_ref());

    if !include_overseas {
        return Ok(domestic_total);
    }

    let overseas = rest
        .get_overseas_balance()
        .await
        .map_err(|e| format!("KIS 해외 잔고 조회 실패: {e}"))?;
    let overseas_total = overseas
        .summary
        .as_ref()
        .map(|s| (parse_amount_f64(&s.frcr_evlu_tota) * exchange_rate_krw).round() as i64)
        .unwrap_or(0);

    Ok(domestic_total.saturating_add(overseas_total.max(0)))
}

/// Toss 활성 프로파일 기준 리스크 총잔고(KRW 환산) 조회
///
/// KIS의 `fetch_account_risk_balance_krw`에 대응하는 Toss 경로.
/// 매수가능금액(KRW/USD 현금)과 보유 종목 평가금액(KR/US)을 모두 합산해 KRW로 환산한다.
pub(crate) async fn fetch_toss_risk_balance_krw(
    profile: &AccountProfile,
    exchange_rate_krw: f64,
) -> Result<i64, String> {
    let adapter = TossBrokerAdapter::with_credentials(
        TossBrokerAdapter::DEFAULT_BASE_URL,
        profile.app_key.clone(),
        profile.app_secret.clone(),
        Some(profile.account_no.clone()),
    );
    let account_seq = profile.account_no.as_str();

    let mut total_krw = 0.0f64;

    for currency in [BrokerCurrency::Krw, BrokerCurrency::Usd] {
        let power = adapter
            .get_buying_power(Some(account_seq), currency)
            .await
            .map_err(|e| format!("Toss {currency:?} 매수가능금액 조회 실패: {e}"))?;
        let amount = power
            .cash_buying_power
            .trim()
            .replace(',', "")
            .parse::<f64>()
            .map_err(|_| format!("Toss {currency:?} 매수가능금액 형식 오류"))?;
        total_krw += if currency == BrokerCurrency::Usd {
            amount * exchange_rate_krw
        } else {
            amount
        };
    }

    let account_id = BrokerAccountId(profile.broker_account_id());
    let holdings = adapter
        .list_holdings(Some(&account_id))
        .await
        .map_err(|e| format!("Toss 보유종목 조회 실패: {e}"))?;
    for holding in &holdings {
        let qty = holding
            .quantity
            .0
            .trim()
            .replace(',', "")
            .parse::<f64>()
            .map_err(|_| format!("Toss {} 보유수량 형식 오류", holding.symbol.0))?;
        let price = holding
            .current_price
            .amount
            .trim()
            .replace(',', "")
            .parse::<f64>()
            .map_err(|_| format!("Toss {} 현재가 형식 오류", holding.symbol.0))?;
        let value = qty * price;
        total_krw += if holding.current_price.currency == BrokerCurrency::Usd {
            value * exchange_rate_krw
        } else {
            value
        };
    }

    Ok(total_krw.round() as i64)
}
