/// 모바일/외부 접속용 경량 웹 서버 (axum)
///
/// 엔드포인트:
///   GET /                    → React 앱(dist/) 또는 모바일 대시보드 HTML
///   GET /api/info            → 앱 정보 JSON
///   GET /api/balance         → 잔고 JSON
///   GET /api/price/:symbol   → 현재가 JSON
///   GET /api/executed        → 당일 체결 JSON
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use axum::{
    extract::{Path, State},
    response::{Html, IntoResponse, Response},
    routing::get,
    Json, Router,
};
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;

use crate::api::rest::KisRestClient;

#[derive(Clone)]
struct ServerState {
    rest_client: Arc<RwLock<Arc<KisRestClient>>>,
    web_port: u16,
    dist_path: PathBuf,
}

/// 서버 시작 (포트 바인드 실패 시 경고만 내고 종료)
pub async fn start(rest_client: Arc<RwLock<Arc<KisRestClient>>>, port: u16) {
    let dist_path = web_dist_path();

    if dist_path.join("index.html").exists() {
        tracing::info!("웹 모드: React 앱을 {:?} 에서 서비스합니다", dist_path);
    } else {
        tracing::info!("웹 모드: dist/ 없음 — 모바일 대시보드 HTML로 서비스합니다");
        tracing::info!(
            "React 앱을 서비스하려면 프로젝트 루트에서 'npm run build:web' 을 실행하세요"
        );
    }

    let state = ServerState {
        rest_client,
        web_port: port,
        dist_path,
    };

    let app = Router::new()
        .route("/api/info", get(info_handler))
        .route("/api/balance", get(balance_handler))
        .route("/api/price/:symbol", get(price_handler))
        .route("/api/executed", get(executed_handler))
        .fallback(get(spa_handler))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    match tokio::net::TcpListener::bind(addr).await {
        Ok(listener) => {
            tracing::info!(
                "웹 서버 시작: http://0.0.0.0:{} (같은 네트워크에서 접속 가능)",
                port
            );
            if let Err(e) = axum::serve(listener, app).await {
                tracing::error!("웹 서버 종료: {}", e);
            }
        }
        Err(e) => {
            tracing::warn!("웹 서버 포트 {} 바인드 실패: {} — 웹 접속 비활성", port, e);
        }
    }
}

/// dist/ 폴더 경로 탐색: 바이너리 옆 dist/ → 현재 작업 디렉토리의 dist/
fn web_dist_path() -> PathBuf {
    if let Some(p) = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("dist")))
    {
        if p.exists() {
            return p;
        }
    }
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("dist")
}

// ── 핸들러 ──────────────────────────────────────────────────────────

/// SPA fallback: dist/index.html 서비스 또는 모바일 대시보드 HTML 반환
async fn spa_handler(State(s): State<ServerState>) -> Response {
    let index_path = s.dist_path.join("index.html");
    match tokio::fs::read_to_string(&index_path).await {
        Ok(content) => Html(content).into_response(),
        Err(_) => Html(MOBILE_HTML).into_response(),
    }
}

async fn info_handler(State(s): State<ServerState>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "app": "KISAutoTrade",
        "version": env!("CARGO_PKG_VERSION"),
        "port": s.web_port,
        "mode": "web",
    }))
}

async fn balance_handler(State(s): State<ServerState>) -> Json<serde_json::Value> {
    let client = s.rest_client.read().await.clone();
    match client.get_balance().await {
        Ok(b) => Json(serde_json::to_value(b).unwrap_or_default()),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn price_handler(
    State(s): State<ServerState>,
    Path(symbol): Path<String>,
) -> Json<serde_json::Value> {
    let client = s.rest_client.read().await.clone();
    match client.get_price(&symbol).await {
        Ok(p) => Json(serde_json::to_value(p).unwrap_or_default()),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn executed_handler(State(s): State<ServerState>) -> Json<serde_json::Value> {
    let client = s.rest_client.read().await.clone();
    match client.get_today_executed_orders().await {
        Ok(e) => Json(serde_json::to_value(e).unwrap_or_default()),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

// ── 모바일 대시보드 HTML (dist/index.html 없을 때 폴백) ────────────

static MOBILE_HTML: &str = r#"<!DOCTYPE html>
<html lang="ko">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>KISAutoTrade</title>
  <style>
    * { box-sizing: border-box; margin: 0; padding: 0; }
    body { font-family: -apple-system, sans-serif; background: #121212; color: #e0e0e0; padding: 16px; }
    h1 { font-size: 18px; color: #90caf9; margin-bottom: 4px; }
    .subtitle { font-size: 12px; color: #757575; margin-bottom: 16px; }
    h2 { font-size: 14px; color: #90caf9; margin-bottom: 8px; }
    .card { background: #1e1e1e; border-radius: 8px; padding: 14px; margin-bottom: 12px; }
    .row { display: flex; justify-content: space-between; padding: 6px 0; border-bottom: 1px solid #2a2a2a; font-size: 13px; }
    .row:last-child { border-bottom: none; }
    .label { color: #9e9e9e; }
    .value { font-weight: 600; }
    .green { color: #4caf50; }
    .red { color: #ef5350; }
    table { width: 100%; border-collapse: collapse; font-size: 12px; }
    th { color: #9e9e9e; text-align: left; padding: 4px 0; border-bottom: 1px solid #333; }
    td { padding: 6px 0; border-bottom: 1px solid #2a2a2a; vertical-align: top; }
    .chip { display: inline-block; padding: 2px 6px; border-radius: 4px; font-size: 11px; }
    .chip-buy { background: #1565c0; color: #fff; }
    .chip-sell { background: #b71c1c; color: #fff; }
    .sub { font-size: 11px; color: #757575; }
    .refresh { margin-top: 8px; text-align: right; font-size: 11px; color: #757575; }
    button { background: #1565c0; color: #fff; border: none; padding: 8px 16px; border-radius: 6px; font-size: 13px; cursor: pointer; }
    .notice { background: #1a237e; border-radius: 6px; padding: 10px 12px; margin-bottom: 12px; font-size: 12px; color: #90caf9; }
  </style>
</head>
<body>
  <h1>KISAutoTrade</h1>
  <div class="subtitle">모바일 대시보드 (읽기 전용)</div>
  <div class="notice">
    💡 전체 UI는 데스크탑 앱에서 사용하거나, 프로젝트 루트에서
    <code>npm run build:web</code> 실행 후 접속하세요.
  </div>

  <div class="card">
    <h2>잔고</h2>
    <div id="balance-body"><div class="label">로딩 중...</div></div>
  </div>

  <div class="card">
    <h2>보유 종목</h2>
    <table>
      <thead><tr><th>종목</th><th style="text-align:right">수량</th><th style="text-align:right">손익률</th></tr></thead>
      <tbody id="holdings"></tbody>
    </table>
  </div>

  <div class="card">
    <h2>당일 체결</h2>
    <table>
      <thead><tr><th>종목</th><th>구분</th><th style="text-align:right">체결가</th></tr></thead>
      <tbody id="executed"></tbody>
    </table>
  </div>

  <div class="refresh" id="refresh-time"></div>
  <br>
  <button onclick="load()">새로고침</button>

  <script>
    function fmt(n) { return Number(n || 0).toLocaleString('ko-KR'); }

    async function load() {
      try {
        const bal = await fetch('/api/balance').then(r => r.json());
        const s = bal.summary || {};
        document.getElementById('balance-body').innerHTML =
          `<div class="row"><span class="label">예수금</span><span class="value">${fmt(s.dnca_tot_amt)}원</span></div>` +
          `<div class="row"><span class="label">총평가금액</span><span class="value">${fmt(s.tot_evlu_amt)}원</span></div>` +
          `<div class="row"><span class="label">순자산</span><span class="value">${fmt(s.nass_amt)}원</span></div>`;

        const tbody = document.getElementById('holdings');
        tbody.innerHTML = (bal.items || []).map(i => {
          const pf = parseFloat(i.evlu_pfls_rt || 0);
          const cls = pf >= 0 ? 'green' : 'red';
          const sign = pf >= 0 ? '+' : '';
          return `<tr>
            <td>${i.prdt_name}<div class="sub">${i.pdno}</div></td>
            <td style="text-align:right">${fmt(i.hldg_qty)}주</td>
            <td style="text-align:right" class="${cls}">${sign}${pf.toFixed(2)}%<div class="sub ${cls}">${sign}${fmt(i.evlu_pfls_amt)}원</div></td>
          </tr>`;
        }).join('') || '<tr><td colspan="3" class="label" style="text-align:center;padding:8px">보유 종목 없음</td></tr>';
      } catch(e) { console.error('balance', e); }

      try {
        const ex = await fetch('/api/executed').then(r => r.json());
        const etbody = document.getElementById('executed');
        etbody.innerHTML = (Array.isArray(ex) ? ex : []).map(o => {
          const isSell = o.sll_buy_dvsn_cd === '01';
          return `<tr>
            <td>${o.prdt_name}<div class="sub">${o.pdno}</div></td>
            <td><span class="chip ${isSell ? 'chip-sell' : 'chip-buy'}">${isSell ? '매도' : '매수'}</span></td>
            <td style="text-align:right">${fmt(o.ord_unpr)}원<div class="sub">${fmt(o.tot_ccld_qty)}주</div></td>
          </tr>`;
        }).join('') || '<tr><td colspan="3" class="label" style="text-align:center;padding:8px">체결 내역 없음</td></tr>';
      } catch(e) { console.error('executed', e); }

      document.getElementById('refresh-time').textContent =
        '최종 업데이트: ' + new Date().toLocaleTimeString('ko-KR');
    }

    load();
    setInterval(load, 30000);
  </script>
</body>
</html>"#;
