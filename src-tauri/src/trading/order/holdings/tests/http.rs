//! Real KIS HTTP client, synthetic token and loopback server only.
use super::*;
use axum::{routing::get, Json, Router};
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::{
    sync::Notify,
    time::{timeout, Duration},
};

struct Server {
    url: String,
    arrived: Arc<Notify>,
    resume: Arc<Notify>,
    task: tokio::task::JoinHandle<()>,
}
impl Server {
    async fn new(body: serde_json::Value) -> Self {
        let arrived = Arc::new(Notify::new());
        let resume = Arc::new(Notify::new());
        let (a, r) = (arrived.clone(), resume.clone());
        let router = Router::new().fallback(get(move || {
            let (a, r, body) = (a.clone(), r.clone(), body.clone());
            async move {
                a.notify_one();
                r.notified().await;
                Json(body)
            }
        }));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let task = tokio::spawn(async move {
            axum::serve(listener, router).await.unwrap();
        });
        Self {
            url,
            arrived,
            resume,
            task,
        }
    }
    async fn wait(&self) {
        timeout(Duration::from_secs(3), self.arrived.notified())
            .await
            .unwrap();
    }
}
impl Drop for Server {
    fn drop(&mut self) {
        self.task.abort();
    }
}
fn balance(quantity: &str, avg: &str) -> serde_json::Value {
    serde_json::json!({"rt_cd":"0","msg1":"synthetic fixture","output1":[{
        "pdno":"005930","prdt_name":"synthetic fixture","hldg_qty":quantity,
        "pchs_avg_pric":avg,"prpr":"110"
    }],"output2":[]})
}
fn refresh(
    f: &Fixture,
    notifications: Arc<AtomicUsize>,
) -> tokio::task::JoinHandle<Result<crate::api::rest::BalanceResponse>> {
    let orders = f.manager.clone();
    tokio::spawn(async move {
        OrderManager::refresh_kis_domestic_shared(&orders.clone(), None, move |_| {
            assert!(
                orders.try_lock().is_err(),
                "event must be inside the publication gate"
            );
            notifications.fetch_add(1, Ordering::SeqCst);
        })
        .await
    })
}

#[tokio::test]
async fn delayed_http_a_to_b_to_a_response_never_publishes_or_emits() {
    let server = Server::new(balance("99", "100")).await;
    let f = Fixture::new(server.url.clone()).await;
    f.begin(1).await;
    let notifications = Arc::new(AtomicUsize::new(0));
    let job = refresh(&f, notifications.clone());
    server.wait().await;
    // A real request is suspended while the order/profile publication gate is free.
    {
        let mut manager = timeout(Duration::from_millis(300), f.manager.lock())
            .await
            .unwrap();
        let other = test_support::profile("8765432101");
        manager.set_execution_scope(test_support::scope(&other));
        manager.clear_position_trackers_for_scope_change().await;
        manager.set_execution_scope(test_support::scope(&f.profile));
    }
    f.begin(2).await;
    f.seed_holding("005930", 2, 100).await;
    server.resume.notify_one();
    let error = timeout(Duration::from_secs(3), job)
        .await
        .unwrap()
        .unwrap()
        .unwrap_err();
    assert!(error.to_string().contains("계좌 또는 실행 상태"), "{error}");
    assert_eq!(notifications.load(Ordering::SeqCst), 0);
    assert_eq!(held(&f, "005930").await, 2);
}

#[tokio::test]
async fn delayed_http_response_cannot_undo_a_concurrent_buy_or_sell() {
    for sell in [false, true] {
        let server = Server::new(balance("1", "100")).await;
        let f = Fixture::new(server.url.clone()).await;
        f.begin(1).await;
        f.seed_holding("005930", 2, 100).await;
        let notifications = Arc::new(AtomicUsize::new(0));
        let job = refresh(&f, notifications.clone());
        server.wait().await;
        if sell {
            f.manager
                .lock()
                .await
                .position_tracker
                .lock()
                .await
                .on_sell("005930", 1);
        } else {
            f.seed_holding("005930", 1, 120).await;
        }
        server.resume.notify_one();
        let error = timeout(Duration::from_secs(3), job)
            .await
            .unwrap()
            .unwrap()
            .unwrap_err();
        assert!(error.to_string().contains("포지션 변경"), "{error}");
        assert_eq!(held(&f, "005930").await, if sell { 1 } else { 3 });
        assert_eq!(notifications.load(Ordering::SeqCst), 0);
    }
}

#[tokio::test]
async fn accepted_http_response_publishes_before_its_event_under_the_same_gate() {
    let server = Server::new(balance("3", "100.4")).await;
    let f = Fixture::new(server.url.clone()).await;
    f.begin(1).await;
    let notifications = Arc::new(AtomicUsize::new(0));
    let job = refresh(&f, notifications.clone());
    server.wait().await;
    server.resume.notify_one();
    timeout(Duration::from_secs(3), job)
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    assert_eq!(held(&f, "005930").await, 3);
    assert_eq!(notifications.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn invalid_holdings_do_not_clear_the_previous_position_or_emit_success() {
    for (quantity, avg) in [("not-a-quantity", "100"), ("2", "NaN"), ("2", "-10")] {
        let server = Server::new(balance(quantity, avg)).await;
        let f = Fixture::new(server.url.clone()).await;
        f.begin(1).await;
        f.seed_holding("005930", 2, 100).await;
        let notifications = Arc::new(AtomicUsize::new(0));
        let job = refresh(&f, notifications.clone());
        server.wait().await;
        server.resume.notify_one();
        let error = timeout(Duration::from_secs(3), job)
            .await
            .unwrap()
            .unwrap()
            .unwrap_err();
        assert!(
            error.to_string().contains(if quantity == "not-a-quantity" {
                "보유수량"
            } else {
                "잔고 가격"
            }),
            "{error}"
        );
        assert_eq!(held(&f, "005930").await, 2);
        assert_eq!(notifications.load(Ordering::SeqCst), 0);
    }
}
