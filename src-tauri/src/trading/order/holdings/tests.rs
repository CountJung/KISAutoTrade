use super::*;
use crate::trading::{
    configuration::ConfigurationEditReservation,
    order::test_support::{self, Fixture},
};

mod http;

async fn fixture() -> Fixture {
    let fixture = Fixture::new("http://127.0.0.1:1".into()).await;
    fixture.begin(1).await;
    fixture
}
async fn query(f: &Fixture, markets: HoldingsMarkets) -> HoldingsQuery {
    f.manager
        .lock()
        .await
        .begin_holdings_query(f.profile.broker_id, markets)
        .await
        .unwrap()
}
fn domestic(quantity: u64) -> HoldingsUpdate {
    HoldingsUpdate::Domestic(vec![(
        "005930".into(),
        "fixture".into(),
        quantity,
        100,
        100,
    )])
}
async fn held(f: &Fixture, symbol: &str) -> u64 {
    let manager = f.manager.lock().await;
    if crate::market_hours::is_domestic_symbol(symbol) {
        manager
            .position_tracker
            .lock()
            .await
            .get(symbol)
            .map(|p| p.quantity)
            .unwrap_or(0)
    } else {
        manager
            .overseas_position_tracker
            .lock()
            .await
            .get(symbol)
            .map(|p| p.quantity)
            .unwrap_or(0)
    }
}

#[tokio::test]
async fn newer_request_and_fill_each_reject_a_stale_replacement() {
    let fixture = Fixture::new("http://127.0.0.1:1".into()).await;
    fixture.begin(1).await;
    let old = fixture
        .manager
        .lock()
        .await
        .begin_holdings_query(BrokerId::Kis, HoldingsMarkets::Domestic)
        .await
        .unwrap();
    let latest = fixture
        .manager
        .lock()
        .await
        .begin_holdings_query(BrokerId::Kis, HoldingsMarkets::Domestic)
        .await
        .unwrap();
    assert!(fixture
        .manager
        .lock()
        .await
        .publish_holdings(old, HoldingsUpdate::Domestic(vec![]))
        .await
        .is_err());
    fixture.seed_holding("005930", 1, 100).await;
    assert!(fixture
        .manager
        .lock()
        .await
        .publish_holdings(latest, HoldingsUpdate::Domestic(vec![]))
        .await
        .is_err());
    assert_eq!(
        fixture
            .manager
            .lock()
            .await
            .position_tracker
            .lock()
            .await
            .get("005930")
            .unwrap()
            .quantity,
        1
    );
}

#[tokio::test]
async fn reverse_completion_keeps_the_newest_issued_response() {
    let f = fixture().await;
    let first = query(&f, HoldingsMarkets::Domestic).await;
    let second = query(&f, HoldingsMarkets::Domestic).await;
    f.manager
        .lock()
        .await
        .publish_holdings(second, domestic(2))
        .await
        .unwrap();
    assert!(f
        .manager
        .lock()
        .await
        .publish_holdings(first, domestic(99))
        .await
        .is_err());
    assert_eq!(held(&f, "005930").await, 2);
}

#[tokio::test]
async fn empty_and_zero_holdings_clear_only_the_queried_market() {
    let f = fixture().await;
    f.seed_holding("005930", 2, 100).await;
    f.seed_holding("AAPL", 3, 100).await;
    let kr = query(&f, HoldingsMarkets::Domestic).await;
    let us = query(&f, HoldingsMarkets::Overseas).await;
    f.manager
        .lock()
        .await
        .publish_holdings(kr, domestic(0))
        .await
        .unwrap();
    assert_eq!(held(&f, "005930").await, 0);
    assert_eq!(held(&f, "AAPL").await, 3);
    // An unrelated domestic publication does not supersede a US query.
    f.manager
        .lock()
        .await
        .publish_holdings(us, HoldingsUpdate::Overseas(vec![]))
        .await
        .unwrap();
    assert_eq!(held(&f, "AAPL").await, 0);
}

#[tokio::test]
async fn a_change_to_either_market_rejects_both_without_partial_publication() {
    for changed in ["005930", "AAPL"] {
        let f = fixture().await;
        f.seed_holding("005930", 2, 100).await;
        f.seed_holding("AAPL", 3, 100).await;
        let ticket = query(&f, HoldingsMarkets::Both).await;
        f.seed_holding(changed, 1, 100).await;
        assert!(f
            .manager
            .lock()
            .await
            .publish_holdings(
                ticket,
                HoldingsUpdate::Both {
                    domestic: vec![],
                    overseas: vec![]
                }
            )
            .await
            .is_err());
        assert_eq!(
            held(&f, "005930").await,
            if changed == "005930" { 3 } else { 2 }
        );
        assert_eq!(
            held(&f, "AAPL").await,
            if changed == "AAPL" { 4 } else { 3 }
        );
    }
}

#[tokio::test]
async fn quote_updates_do_not_discard_a_valid_holdings_query() {
    let f = fixture().await;
    f.seed_holding("005930", 1, 100).await;
    let ticket = query(&f, HoldingsMarkets::Domestic).await;
    f.manager
        .lock()
        .await
        .position_tracker
        .lock()
        .await
        .update_price("005930", 110);
    f.manager
        .lock()
        .await
        .publish_holdings(ticket, domestic(2))
        .await
        .unwrap();
    assert_eq!(held(&f, "005930").await, 2);
}

#[tokio::test]
async fn sell_clear_replace_and_tracker_recreation_reject_earlier_responses() {
    for change in 0..4 {
        let f = fixture().await;
        f.seed_holding("005930", 2, 100).await;
        let ticket = query(&f, HoldingsMarkets::Domestic).await;
        {
            let manager = f.manager.lock().await;
            let mut tracker = manager.position_tracker.lock().await;
            match change {
                0 => {
                    tracker.on_sell("005930", 1);
                }
                1 => tracker.clear(),
                2 => tracker.replace(vec![("000660".into(), "fixture".into(), 1, 100, 100)]),
                _ => *tracker = PositionTracker::new(),
            }
        }
        assert!(f
            .manager
            .lock()
            .await
            .publish_holdings(ticket, domestic(99))
            .await
            .is_err());
        assert_eq!(held(&f, "005930").await, if change == 0 { 1 } else { 0 });
        assert_eq!(held(&f, "000660").await, if change == 2 { 1 } else { 0 });
    }
}

#[tokio::test]
async fn stopped_and_restarted_same_account_rejects_old_query_without_resetting_holdings() {
    let f = fixture().await;
    f.seed_holding("005930", 2, 100).await;
    let ticket = query(&f, HoldingsMarkets::Domestic).await;
    f.manager.lock().await.end_execution_run();
    f.begin(2).await;
    assert!(f
        .manager
        .lock()
        .await
        .publish_holdings(ticket, domestic(99))
        .await
        .is_err());
    assert_eq!(held(&f, "005930").await, 2);
}

#[tokio::test]
async fn identity_changes_reject_without_mutating_either_tracker() {
    for change in [
        "profile", "account", "key", "secret", "paper", "consent", "broker", "client",
    ] {
        let f = fixture().await;
        f.seed_holding("005930", 2, 100).await;
        let ticket = query(&f, HoldingsMarkets::Domestic).await;
        {
            let manager = f.manager.lock().await;
            if change == "client" {
                *manager.rest_client.write().await =
                    test_support::client("http://127.0.0.1:1".into(), &f.profile).await;
            } else {
                let mut profiles = manager.profiles.write().await;
                let profile = &mut profiles.profiles[0];
                match change {
                    "profile" => {
                        profile.id.push_str("_other");
                        profiles.active_id = Some(profile.id.clone());
                    }
                    "account" => profile.account_no = "8765432101".into(),
                    "key" => profile.app_key.push_str("_other"),
                    "secret" => profile.app_secret.push_str("_other"),
                    "paper" => profile.is_paper_trading = false,
                    "consent" => profile.live_trading_consent = true,
                    _ => profile.broker_id = BrokerId::Toss,
                }
            }
        }
        assert!(
            f.manager
                .lock()
                .await
                .publish_holdings(ticket, domestic(99))
                .await
                .is_err(),
            "{change}"
        );
        assert_eq!(held(&f, "005930").await, 2);
    }
}

#[tokio::test]
async fn configuration_lease_rejects_capture_and_inflight_response_even_after_release() {
    let f = fixture().await;
    f.seed_holding("005930", 2, 100).await;
    let ticket = query(&f, HoldingsMarkets::Domestic).await;
    let edit = Arc::new(ConfigurationEditReservation {
        strategy_id: "fixture".into(),
        profile_id: f.profile.id.clone(),
        scope: test_support::scope(&f.profile),
        symbols: ["005930".into()].into_iter().collect(),
    });
    f.manager.lock().await.configuration_edit = Some(Arc::downgrade(&edit));
    assert!(f
        .manager
        .lock()
        .await
        .begin_holdings_query(BrokerId::Kis, HoldingsMarkets::Domestic)
        .await
        .is_err());
    drop(edit); // A failed save also ends a lease without a new run revision.
    assert!(f
        .manager
        .lock()
        .await
        .publish_holdings(ticket, domestic(99))
        .await
        .is_err());
    assert_eq!(held(&f, "005930").await, 2);
    let fresh = query(&f, HoldingsMarkets::Domestic).await;
    f.manager
        .lock()
        .await
        .publish_holdings(fresh, domestic(3))
        .await
        .unwrap();
    assert_eq!(held(&f, "005930").await, 3);
}

#[tokio::test]
async fn invalid_snapshot_request_cannot_supersede_a_legitimate_inflight_query() {
    let f = fixture().await;
    let ticket = query(&f, HoldingsMarkets::Domestic).await;
    let wrong = test_support::scope(&test_support::profile("8765432101"));
    assert!(f
        .manager
        .lock()
        .await
        .begin_holdings_query_for_snapshot(
            &wrong,
            &f.client,
            Some(&f.profile),
            HoldingsMarkets::Domestic
        )
        .await
        .is_err());
    f.manager
        .lock()
        .await
        .publish_holdings(ticket, domestic(2))
        .await
        .unwrap();
    assert_eq!(held(&f, "005930").await, 2);
}

#[tokio::test]
async fn readonly_response_does_not_supersede_publication_but_rejects_identity_drift() {
    let f = fixture().await;
    let publication = query(&f, HoldingsMarkets::Domestic).await;
    let readonly = query(&f, HoldingsMarkets::ReadOnly).await;
    f.manager
        .lock()
        .await
        .publish_holdings(readonly, HoldingsUpdate::ReadOnly)
        .await
        .unwrap();
    f.manager
        .lock()
        .await
        .publish_holdings(publication, domestic(2))
        .await
        .unwrap();
    let readonly = query(&f, HoldingsMarkets::ReadOnly).await;
    f.manager.lock().await.end_execution_run();
    assert!(f
        .manager
        .lock()
        .await
        .publish_holdings(readonly, HoldingsUpdate::ReadOnly)
        .await
        .is_err());
    assert_eq!(held(&f, "005930").await, 2);
}

#[tokio::test]
async fn overseas_publication_preserves_cents_and_order_exchange_and_rejects_invalid_input() {
    use crate::api::rest::{OverseasBalanceItem, OverseasBalanceResponse};
    let f = fixture().await;
    f.seed_holding("005930", 2, 100).await;
    let response = OverseasBalanceResponse {
        items: vec![OverseasBalanceItem {
            ovrs_pdno: "AAPL".into(),
            ovrs_item_name: "synthetic fixture".into(),
            ovrs_excg_cd: " nasdaq ".into(),
            ovrs_cblc_qty: "2".into(),
            pchs_avg_pric: "123.45".into(),
            now_pric2: "124.50".into(),
            ..Default::default()
        }],
        summary: None,
    };
    let ticket = query(&f, HoldingsMarkets::Overseas).await;
    f.manager
        .lock()
        .await
        .publish_holdings(
            ticket,
            HoldingsUpdate::from_kis_overseas(&response).unwrap(),
        )
        .await
        .unwrap();
    {
        let manager = f.manager.lock().await;
        let tracker = manager.overseas_position_tracker.lock().await;
        let position = tracker.get("AAPL").unwrap();
        assert_eq!(position.quantity, 2);
        assert_eq!(position.avg_price_cents, 12345.0);
        assert_eq!(position.current_price_cents, 12450);
        assert_eq!(position.exchange, "NASD");
    }
    let mut invalid = response;
    invalid.items[0].ovrs_cblc_qty = "invalid".into();
    assert!(HoldingsUpdate::from_kis_overseas(&invalid).is_err());
    assert_eq!(held(&f, "AAPL").await, 2);
    assert_eq!(held(&f, "005930").await, 2);
}
