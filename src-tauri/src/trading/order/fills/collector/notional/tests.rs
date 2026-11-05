use super::*;

#[test]
fn exact_notional_decimal_rounds_native_total_once_and_checks_bounds() {
    assert_eq!(
        OrderManager::parse_fill_notional("301", BrokerCurrency::Krw).unwrap(),
        301
    );
    assert_eq!(
        OrderManager::parse_fill_notional("3.01", BrokerCurrency::Usd).unwrap(),
        301
    );
    assert_eq!(
        scaled_total("1.003333333333333333", BrokerCurrency::Usd, 3).unwrap(),
        301
    );
    assert_eq!(
        scaled_total("100.333333333333333333", BrokerCurrency::Krw, 3).unwrap(),
        301
    );
    assert_eq!(
        OrderManager::parse_fill_notional("0.005", BrokerCurrency::Usd).unwrap(),
        1
    );
    for invalid in [
        "",
        "NaN",
        "Infinity",
        "-1",
        "+1",
        "1e3",
        "1.2.3",
        "1,00",
        "1.0,1",
        "1,,000",
        ",123",
        "340282366920938463463374607431768211456",
    ] {
        assert!(
            OrderManager::parse_fill_notional(invalid, BrokerCurrency::Usd).is_err(),
            "{invalid}"
        );
    }
    assert!(execution_quantity("0.5").is_err());
    assert_eq!(execution_quantity("3.000").unwrap(), 3);
}

fn toss(
    currency: &str,
    amount: Option<&str>,
    average: Option<&str>,
) -> crate::broker::toss::TossOrder {
    serde_json::from_value(serde_json::json!({
        "orderId":"fixture","symbol":if currency=="KRW" {"005930"} else {"SOXQ"},
        "side":"BUY","orderType":"LIMIT","timeInForce":"DAY","status":"FILLED",
        "price":"1.00","quantity":"3","orderAmount":null,"currency":currency,
        "orderedAt":"2026-10-18T12:00:00+09:00","canceledAt":null,
        "execution":{"filledQuantity":"3","averageFilledPrice":average,
            "filledAmount":amount,"commission":null,"tax":null,"filledAt":null,"settlementDate":null}
    })).unwrap()
}

#[test]
fn exact_notional_toss_explicit_amount_wins_and_explicit_zero_never_falls_back() {
    assert_eq!(
        toss_amount(&toss("USD", Some("3.01"), Some("1.00")), "SOXQ", 3).unwrap(),
        301
    );
    assert_eq!(
        toss_amount(&toss("USD", None, Some("1.003333333333333333")), "SOXQ", 3).unwrap(),
        301
    );
    assert!(toss_amount(&toss("USD", Some("0"), Some("1.00")), "SOXQ", 3).is_err());
    assert!(toss_amount(&toss("EUR", Some("3.01"), Some("1.00")), "SOXQ", 3).is_err());
    assert!(toss_amount(&toss("KRW", Some("301"), Some("100")), "SOXQ", 3).is_err());
}

#[tokio::test]
async fn exact_notional_kis_collectors_preserve_original_301_for_three_shares() {
    let date = chrono::Local::now().format("%Y%m%d").to_string();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let domestic = serde_json::json!({"rt_cd":"0","msg1":"synthetic domestic","output1":[{
        "pdno":"005930","odno":"dom","ord_dt":date,"ord_qty":"3","tot_ccld_qty":"3",
        "tot_ccld_amt":"301","ord_unpr":"101","rmn_qty":"0"
    }]});
    let overseas = serde_json::json!({"rt_cd":"0","msg1":"synthetic overseas","output":[{
        "pdno":"SOXQ","odno":"us","ord_dt":date,"ft_ord_qty":"3","ft_ccld_qty":"3",
        "ft_ccld_amt3":"3.01","ft_ccld_unpr3":"1.00","nccs_qty":"0","tr_crcy_cd":"USD",
        "ovrs_excg_cd":"NASD"
    }]});
    let router = axum::Router::new()
        .route(
            "/uapi/domestic-stock/v1/trading/inquire-daily-ccld",
            axum::routing::get(move || {
                let body = domestic.clone();
                async move { axum::Json(body) }
            }),
        )
        .route(
            "/uapi/overseas-stock/v1/trading/inquire-ccnl",
            axum::routing::get(move || {
                let body = overseas.clone();
                async move { axum::Json(body) }
            }),
        );
    let server = tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });
    let profile = crate::trading::order::test_support::profile("1234567801");
    let client = crate::trading::order::test_support::client(base, &profile).await;
    let fills = super::super::collect_kis_pending_fills(
        client,
        &[
            ("dom".into(), "005930".into(), date.clone()),
            ("us".into(), "SOXQ".into(), date),
        ],
    )
    .await
    .unwrap();
    assert_eq!(fills.len(), 2);
    for fill in fills {
        assert_eq!(fill.filled_qty, 3);
        assert_eq!(fill.avg_price, 100);
        assert_eq!(fill.cumulative_notional, Some(301));
    }
    server.abort();
}
