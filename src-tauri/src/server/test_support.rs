//! Isolated production-router fixtures. No app startup, environment/config load,
//! keychain, global database installation or external provider connections.
use super::*;
use axum::{
    body::Body,
    extract::MatchedPath,
    http::{HeaderMap, Request, StatusCode},
    middleware::Next,
};
use serde_json::{json, Value};
use tower::ServiceExt;

pub(super) const TOKEN: &str = "0123456789abcdef0123456789abcdef";
pub(super) const HOST: &str = "fixture.test:7474";
pub(super) type WireRequests = Arc<Mutex<Vec<(Uri, HeaderMap)>>>;

async fn mock_provider(
    State(requests): State<WireRequests>,
    uri: Uri,
    headers: HeaderMap,
) -> Response {
    requests.lock().await.push((uri.clone(), headers));
    if uri.query().is_some_and(|query| query.contains("FAIL")) {
        return (StatusCode::BAD_GATEWAY, "synthetic provider failure").into_response();
    }
    let body = match uri.path() {
        "/uapi/domestic-stock/v1/quotations/inquire-price" => {
            json!({"rt_cd":"0","msg1":"fixture","output":{"stck_prpr":"10000","hts_kor_isnm":"합성 종목"}})
        }
        "/uapi/overseas-price/v1/quotations/price" => {
            json!({"rt_cd":"0","msg1":"fixture","output":{"last":"100.25","name":"Synthetic ETF"}})
        }
        "/uapi/domestic-stock/v1/quotations/inquire-daily-itemchartprice" => {
            json!({"rt_cd":"0","msg1":"fixture","output2":[{"stck_bsop_date":"20261002","stck_clpr":"102","stck_oprc":"100","stck_hgpr":"103","stck_lwpr":"99","acml_vol":"12"},{"stck_bsop_date":"20261001","stck_clpr":"100"}]})
        }
        "/uapi/overseas-price/v1/quotations/dailyprice" => {
            json!({"rt_cd":"0","msg1":"fixture","output2":[{"xymd":"20261002","clos":"100.25","open":"100","high":"101","low":"99","tvol":"12"}]})
        }
        _ => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "unexpected fixture provider request",
            )
                .into_response()
        }
    };
    Json(body).into_response()
}

async fn mark_route(request: Request<Body>, next: Next) -> Response {
    let matched = request
        .extensions()
        .get::<MatchedPath>()
        .map(|path| path.as_str().to_string());
    let mut response = next.run(request).await;
    response.headers_mut().insert(
        "x-fixture-route",
        matched.as_deref().unwrap_or("<fallback>").parse().unwrap(),
    );
    response
}

pub(super) struct Fixture {
    pub state: ServerState,
    pub requests: WireRequests,
    pub path: PathBuf,
    pub profile: crate::config::AccountProfile,
    task: tokio::task::JoinHandle<()>,
}

impl Fixture {
    pub async fn new(token: &str) -> Self {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let requests: WireRequests = Arc::default();
        let provider = Router::new()
            .fallback(mock_provider)
            .with_state(requests.clone());
        let url = format!("http://{}", listener.local_addr().unwrap());
        let task = tokio::spawn(async move { axum::serve(listener, provider).await.unwrap() });
        let profile = crate::trading::order::test_support::profile("1234567801");
        let rest = crate::trading::order::test_support::client(url, &profile).await;
        let path =
            std::env::temp_dir().join(format!("kis-server-contract-{}", uuid::Uuid::new_v4()));
        let data_dir = path.join("data");
        std::fs::create_dir_all(&data_dir).unwrap();
        let rest_client = Arc::new(RwLock::new(rest));
        let profiles = Arc::new(RwLock::new(ProfilesConfig {
            active_id: None,
            profiles: vec![],
        }));
        let position_tracker = Arc::new(Mutex::new(PositionTracker::new()));
        let trade_store = Arc::new(TradeStore::new(data_dir.clone()));
        let stats_store = Arc::new(StatsStore::new(data_dir.clone()));
        let risk_manager = Arc::new(Mutex::new(RiskManager::default()));
        let risk_store = Arc::new(RiskStore::new(data_dir.clone()));
        let exchange_rate_krw = Arc::new(RwLock::new(1450.0));
        let orders = OrderManager::new(
            rest_client.clone(),
            profiles.clone(),
            Arc::new(crate::storage::OrderStore::new(data_dir.clone())),
            Arc::new(crate::storage::PendingOrderStore::new(data_dir.clone())),
            trade_store.clone(),
            position_tracker.clone(),
            Arc::new(Mutex::new(
                crate::trading::position::OverseasPositionTracker::new(),
            )),
            stats_store.clone(),
            exchange_rate_krw.clone(),
            risk_manager.clone(),
            risk_store.clone(),
            None,
        );
        let mut rate = ExchangeRateView::default_krw();
        rate.updated_at = "2026-10-29T00:00:00Z".into();
        let state = ServerState {
            rest_client,
            stock_list: Arc::new(RwLock::new(vec![])),
            web_port: 7474,
            dist_path: path.join("dist"),
            dist_found: false,
            is_trading: Arc::new(Mutex::new(false)),
            storage_maintenance: Arc::new(Mutex::new(())),
            database_manager: Arc::new(DatabaseManager::isolated_json_for_tests(&path)),
            strategy_manager: Arc::new(Mutex::new(StrategyManager::new())),
            strategy_update_lock: Arc::new(Mutex::new(())),
            position_tracker,
            config: Arc::new(RwLock::new(AppConfig::empty(
                &crate::config::DiscordConfig::default(),
            ))),
            profiles,
            trade_store,
            stats_store,
            log_config: Arc::new(RwLock::new(LogConfig::default())),
            log_dir: path.join("logs"),
            trade_archive_config: Arc::new(RwLock::new(TradeArchiveConfig::default())),
            data_dir: data_dir.clone(),
            risk_manager,
            risk_store,
            order_manager: Arc::new(Mutex::new(orders)),
            stock_store: Arc::new(StockStore::new(&data_dir).await),
            strategy_store: Arc::new(StrategyStore::new(&data_dir)),
            profiles_path: path.join("fixture-accounts.json"),
            discord: None,
            exchange_rate_krw,
            exchange_rate_status: Arc::new(RwLock::new(rate)),
            refresh_config: Arc::new(RwLock::new(RefreshConfig::default())),
            web_security: WebSecurity::fixture(token),
        };
        Self {
            state,
            requests,
            path,
            profile,
            task,
        }
    }

    pub fn app(&self) -> Router {
        router::build_router(self.state.clone()).layer(middleware::from_fn(mark_route))
    }

    pub async fn send(&self, request: Request<Body>) -> Response {
        self.app().oneshot(request).await.unwrap()
    }

    pub async fn assert_no_effects(&self) {
        assert!(
            self.requests.lock().await.is_empty(),
            "rejected request reached provider"
        );
        assert!(!*self.state.is_trading.lock().await);
        assert_eq!(self.state.order_manager.lock().await.pending_count(), 0);
        assert_eq!(self.state.position_tracker.lock().await.count(), 0);
        assert_eq!(
            std::fs::read_dir(&self.state.data_dir).unwrap().count(),
            0,
            "rejected request wrote fixture data"
        );
        assert!(!self.state.profiles_path.exists());
        assert!(!self.path.join("fixture-database.json").exists());
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        self.task.abort();
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

pub(super) fn request(method: &str, uri: &str) -> axum::http::request::Builder {
    Request::builder()
        .method(method)
        .uri(uri)
        .header(header::HOST, HOST)
}

pub(super) fn authorized(method: &str, uri: &str) -> axum::http::request::Builder {
    request(method, uri).header(header::AUTHORIZATION, format!("Bearer {TOKEN}"))
}

pub(super) async fn body(response: Response) -> Vec<u8> {
    axum::body::to_bytes(response.into_body(), 4 * 1024 * 1024)
        .await
        .unwrap()
        .to_vec()
}

pub(super) async fn json_body(response: Response) -> Value {
    assert_eq!(response.headers()[header::CONTENT_TYPE], "application/json");
    serde_json::from_slice(&body(response).await).unwrap()
}

pub(super) fn canonical(pattern: &str) -> String {
    pattern
        .split('/')
        .map(|part| {
            part.strip_prefix(':')
                .map_or_else(|| part.to_string(), |name| format!("{{{name}}}"))
        })
        .collect::<Vec<_>>()
        .join("/")
}
