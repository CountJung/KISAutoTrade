use super::*;
use crate::broker::{BrokerAccountId, BrokerMoney, BrokerQuantity, BrokerSymbol};

fn scope() -> BrokerScope {
    BrokerScope::new(
        BrokerId::Toss,
        Some(BrokerAccountId("fixture-account".into())),
    )
}

fn holding(market: BrokerMarket, quantity: &str) -> BrokerHolding {
    let currency = if market == BrokerMarket::Kr {
        BrokerCurrency::Krw
    } else {
        BrokerCurrency::Usd
    };
    BrokerHolding {
        broker: BrokerId::Toss,
        account_id: scope().account_id,
        market,
        symbol: BrokerSymbol(
            if market == BrokerMarket::Kr {
                "005930"
            } else {
                "SOXQ"
            }
            .into(),
        ),
        symbol_name: "synthetic fixture".into(),
        quantity: BrokerQuantity(quantity.into()),
        average_price: BrokerMoney {
            amount: "12.345".into(),
            currency,
        },
        current_price: BrokerMoney {
            amount: "14.126".into(),
            currency,
        },
        unrealized_pnl: None,
        raw: serde_json::Value::Null,
    }
}

#[test]
fn toss_fractional_and_zero_quantities_never_create_a_whole_share() {
    for quantity in ["0.5", "0"] {
        let update =
            HoldingsUpdate::from_toss(&scope(), &[holding(BrokerMarket::Kr, quantity)]).unwrap();
        let HoldingsUpdate::Both { domestic, overseas } = update else {
            panic!("Both expected")
        };
        assert_eq!(domestic[0].2, 0);
        assert!(overseas.is_empty());
    }
    let HoldingsUpdate::Both { domestic, overseas } =
        HoldingsUpdate::from_toss(&scope(), &[]).unwrap()
    else {
        panic!("Both expected")
    };
    assert!(domestic.is_empty() && overseas.is_empty());
    let HoldingsUpdate::Both { domestic, .. } =
        HoldingsUpdate::from_toss(&scope(), &[holding(BrokerMarket::Kr, " 1,234.9 ")]).unwrap()
    else {
        panic!("Both expected")
    };
    assert_eq!(domestic[0].2, 1234);
}

#[test]
fn toss_us_prices_round_to_cents_and_kr_prices_round_to_won() {
    let HoldingsUpdate::Both { domestic, overseas } = HoldingsUpdate::from_toss(
        &scope(),
        &[
            holding(BrokerMarket::Kr, "2"),
            holding(BrokerMarket::Us, "3"),
        ],
    )
    .unwrap() else {
        panic!("Both expected")
    };
    assert_eq!((domestic[0].3, domestic[0].4), (12, 14));
    assert_eq!((overseas[0].4, overseas[0].5), (1235, 1413));
    assert_eq!(overseas[0].2, "TOSS_US");
}

#[test]
fn toss_rejects_invalid_quantities_and_prices_including_float_u64_boundary() {
    for invalid in [
        "",
        "-1",
        "NaN",
        "inf",
        "-inf",
        "18446744073709551616",
        "1e999",
    ] {
        assert!(
            HoldingsUpdate::from_toss(&scope(), &[holding(BrokerMarket::Kr, invalid)]).is_err(),
            "quantity {invalid}"
        );
        for average in [false, true] {
            let mut row = holding(BrokerMarket::Kr, "1");
            if average {
                row.average_price.amount = invalid.into();
            } else {
                row.current_price.amount = invalid.into();
            }
            assert!(
                HoldingsUpdate::from_toss(&scope(), &[row]).is_err(),
                "price {invalid}"
            );
        }
    }
    let mut row = holding(BrokerMarket::Us, "1");
    row.current_price.amount = "184467440737095516.16".into();
    assert!(HoldingsUpdate::from_toss(&scope(), &[row]).is_err());
    assert!(money("18446744073709551616", 1.0, "KIS fixture").is_err());
    assert!(money("184467440737095516.16", 100.0, "KIS fixture").is_err());
}

#[test]
fn toss_rejects_wrong_broker_account_currency_and_duplicate_market_symbol() {
    let valid = holding(BrokerMarket::Kr, "1");
    let mut cases = vec![];
    let mut row = valid.clone();
    row.broker = BrokerId::Kis;
    cases.push(row);
    let mut row = valid.clone();
    row.account_id = None;
    cases.push(row);
    let mut row = valid.clone();
    row.account_id = Some(BrokerAccountId("other".into()));
    cases.push(row);
    let mut row = valid.clone();
    row.average_price.currency = BrokerCurrency::Usd;
    cases.push(row);
    let mut row = valid.clone();
    row.current_price.currency = BrokerCurrency::Usd;
    cases.push(row);
    let mut row = valid.clone();
    row.symbol.0.clear();
    cases.push(row);
    for row in cases {
        assert!(HoldingsUpdate::from_toss(&scope(), &[row]).is_err());
    }
    assert!(HoldingsUpdate::from_toss(&scope(), &[valid.clone(), valid]).is_err());
    let mut wrong_scope = scope();
    wrong_scope.broker_id = BrokerId::Kis;
    assert!(HoldingsUpdate::from_toss(&wrong_scope, &[]).is_err());
    wrong_scope = scope();
    wrong_scope.account_id = None;
    assert!(HoldingsUpdate::from_toss(&wrong_scope, &[]).is_err());
}

#[tokio::test]
async fn dirty_market_refresh_only_queries_and_replaces_the_affected_kis_market() {
    use crate::trading::order::test_support::Fixture;
    use axum::{routing::get, Json, Router};
    use std::sync::atomic::{AtomicUsize, Ordering};
    let domestic_calls = Arc::new(AtomicUsize::new(0));
    let overseas_calls = Arc::new(AtomicUsize::new(0));
    let (kr, us) = (Arc::clone(&domestic_calls), Arc::clone(&overseas_calls));
    let router = Router::new().fallback(get(move |uri: axum::http::Uri| {
        let (kr, us) = (Arc::clone(&kr), Arc::clone(&us));
        async move {
            if uri.path().contains("overseas-stock") { us.fetch_add(1, Ordering::SeqCst); }
            else { kr.fetch_add(1, Ordering::SeqCst); }
            Json(serde_json::json!({"rt_cd":"0","msg1":"synthetic fixture","output1":[],"output2":null}))
        }
    }));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let server = tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });
    for (mask, expected_kr, expected_us) in [(1, 1, 0), (2, 0, 1), (3, 1, 1), (0, 1, 1)] {
        domestic_calls.store(0, Ordering::SeqCst);
        overseas_calls.store(0, Ordering::SeqCst);
        let fixture = Fixture::new(url.clone()).await;
        fixture.begin(0).await;
        fixture.seed_holding("005930", 10, 100).await;
        fixture.seed_holding("SOXQ", 5, 200).await;
        fixture.manager.lock().await.holdings_refresh_required = mask;
        OrderManager::refresh_active_holdings_shared(&fixture.manager)
            .await
            .unwrap();
        assert_eq!(domestic_calls.load(Ordering::SeqCst), expected_kr);
        assert_eq!(overseas_calls.load(Ordering::SeqCst), expected_us);
        let orders = fixture.manager.lock().await;
        assert_eq!(orders.holdings_refresh_required, 0);
        assert_eq!(
            orders
                .position_tracker
                .lock()
                .await
                .get("005930")
                .map(|p| p.quantity),
            if expected_kr == 0 { Some(10) } else { None }
        );
        assert_eq!(
            orders
                .overseas_position_tracker
                .lock()
                .await
                .get("SOXQ")
                .map(|p| p.quantity),
            if expected_us == 0 { Some(5) } else { None }
        );
    }
    server.abort();
}

#[tokio::test]
async fn malformed_kis_manual_holdings_never_publish_or_send_an_order() {
    use crate::api::rest::{OrderSide as RestOrderSide, OrderType};
    use crate::trading::order::test_support::{scope as fixture_scope, Fixture};
    use axum::{routing::get, Json, Router};
    use std::sync::atomic::{AtomicUsize, Ordering};
    for overseas in [false, true] {
        for invalid in ["NaN", "-1", "18446744073709551616"] {
            let order_calls = Arc::new(AtomicUsize::new(0));
            let count = Arc::clone(&order_calls);
            let body = if overseas {
                serde_json::json!({"rt_cd":"0","msg1":"synthetic fixture", "output1":[{
                    "ovrs_pdno":"SOXQ","ovrs_item_name":"fixture","ovrs_excg_cd":"NASD",
                    "ovrs_cblc_qty":"2","pchs_avg_pric":invalid,"now_pric2":"100",
                    "frcr_evlu_pfls_amt":"0","evlu_pfls_rt":"0"
                }], "output2":null})
            } else {
                serde_json::json!({"rt_cd":"0","msg1":"synthetic fixture", "output1":[{
                    "pdno":"005930","prdt_name":"fixture","hldg_qty":"2",
                    "pchs_avg_pric":invalid,"prpr":"100"
                }], "output2":null})
            };
            let router = Router::new().fallback(
                get(move || {
                    let body = body.clone();
                    async move { Json(body) }
                })
                .post(move || {
                    count.fetch_add(1, Ordering::SeqCst);
                    async { Json(serde_json::json!({"rt_cd":"0","msg1":"unexpected order"})) }
                }),
            );
            let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
            let url = format!("http://{}", listener.local_addr().unwrap());
            let server = tokio::spawn(async move {
                axum::serve(listener, router).await.unwrap();
            });
            let fixture = Fixture::new(url).await;
            fixture.begin(0).await;
            let symbol = if overseas { "SOXQ" } else { "005930" };
            fixture.seed_holding(symbol, 7, 300).await;
            let result = OrderManager::submit_manual_order_shared(
                &fixture.manager,
                symbol.into(),
                "synthetic fixture".into(),
                RestOrderSide::Buy,
                OrderType::Limit,
                1,
                100,
                100,
                100_000,
                overseas.then(|| "NASD".to_string()),
                fixture_scope(&fixture.profile),
            )
            .await;
            let error = result.expect_err("invalid holdings must reject manual order");
            assert!(
                error.to_string().contains("잔고 가격 범위 오류"),
                "{invalid}: {error}"
            );
            assert_eq!(order_calls.load(Ordering::SeqCst), 0);
            let orders = fixture.manager.lock().await;
            assert!(orders.pending.is_empty());
            let quantity = if overseas {
                orders
                    .overseas_position_tracker
                    .lock()
                    .await
                    .get(symbol)
                    .unwrap()
                    .quantity
            } else {
                orders
                    .position_tracker
                    .lock()
                    .await
                    .get(symbol)
                    .unwrap()
                    .quantity
            };
            assert_eq!(quantity, 7);
            assert!(orders
                .order_store
                .get_by_date(chrono::Local::now().date_naive())
                .await
                .unwrap()
                .is_empty());
            server.abort();
        }
    }
}
