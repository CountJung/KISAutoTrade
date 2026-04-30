pub mod api;
pub mod commands;
pub mod config;
pub mod logging;
pub mod market;
pub mod market_hours;
pub mod notifications;
pub mod server;
pub mod storage;
pub mod trading;
pub mod updater;

use std::path::PathBuf;
use tauri::{Emitter, Manager};

use commands::{AppState, RefreshConfig};
use config::{AppConfig, DiscordConfig, ProfilesConfig};
use logging::LogConfig;

/// 디렉토리 재귀 복사 (cross-filesystem 이전 시 rename 대신 사용)
/// macOS ._* 리소스 포크 파일은 건너뜀
fn copy_dir_all(
    src: &std::path::Path,
    dst: &std::path::Path,
) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let file_name = entry.file_name();
        // macOS exFAT 리소스 포크(._*) 제외
        if file_name.to_str().map(|n| n.starts_with("._")).unwrap_or(false) {
            continue;
        }
        let src_path = entry.path();
        let dst_path = dst.join(&file_name);
        if src_path.is_dir() {
            copy_dir_all(&src_path, &dst_path)?;
        } else if src_path.is_file() {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let app_data_dir: PathBuf = app
                .path()
                .app_data_dir()
                .expect("앱 데이터 디렉토리를 가져올 수 없습니다");

            // 로그 디렉토리: {cwd}/logs/
            let log_dir = logging::default_log_dir();

            // 로그 설정 로드 (없으면 기본값 5일/100MB)
            let log_cfg = LogConfig::load_sync(&log_dir);

            // 로그 시스템 초기화 (daily rolling + 시작 시 정리)
            logging::init(&log_dir, &log_cfg)?;

            tracing::info!("KISAutoTrade 시작 - 데이터 경로: {:?}", app_data_dir);
            tracing::info!("로그 경로: {:?}, 보관 {}일, 최대 {}MB", log_dir, log_cfg.retention_days, log_cfg.max_size_mb);

            // profiles.json 경로: 항상 app_data_dir 사용 (실행 방식과 무관하게 일관된 경로)
            let profiles_path = app_data_dir.join("profiles.json");

            // 비동기 초기화
            let (config, discord_config, profiles) =
                tauri::async_runtime::block_on(async {
                    // Discord 설정 로드 (secure_config.json)
                    let discord = DiscordConfig::load(&app_data_dir).await;
                    let discord = std::sync::Arc::new(discord);

                    // 프로파일 목록 로드
                    let profiles = ProfilesConfig::load(&profiles_path).await;

                    // 활성 프로파일에서 AppConfig 생성
                    let config = match profiles.get_active() {
                        Some(p) => {
                            tracing::info!("활성 프로파일 로드: {} ({})",
                                p.name,
                                if p.is_paper_trading { "모의투자" } else { "실전투자" }
                            );
                            AppConfig::from_profile(p, &discord)
                        }
                        None => {
                            tracing::warn!("등록된 계좌 프로파일이 없습니다. Settings에서 계좌를 추가하세요.");
                            AppConfig::empty(&discord)
                        }
                    };

                    (config, discord, profiles)
                });

            // 데이터 저장 경로: 로그처럼 실행 위치 기준 ./data/
            // (기존 앱 데이터 디렉토리 경로에서 최초 1회 이전 시도)
            let preferred_data_dir = std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .join("data");
            let legacy_data_dir = app_data_dir.join("data");

            let data_dir = if preferred_data_dir.exists() {
                // 이미 실행 위치에 데이터 존재 → 그대로 사용
                preferred_data_dir
            } else if legacy_data_dir.exists() {
                // 기존 앱 데이터 디렉토리에만 있음 → 이전 시도
                // exFAT 외장 드라이브 등 크로스 파일시스템이면 rename 실패 → copy 시도
                if std::fs::rename(&legacy_data_dir, &preferred_data_dir).is_ok() {
                    tracing::info!("데이터 이전(rename) 완료: {:?} → {:?}", legacy_data_dir, preferred_data_dir);
                    preferred_data_dir
                } else {
                    match copy_dir_all(&legacy_data_dir, &preferred_data_dir) {
                        Ok(_) => {
                            let _ = std::fs::remove_dir_all(&legacy_data_dir);
                            tracing::info!("데이터 이전(copy) 완료: {:?} → {:?}", legacy_data_dir, preferred_data_dir);
                            preferred_data_dir
                        }
                        Err(e) => {
                            tracing::warn!(
                                "데이터 이전 실패 — 기존 위치 계속 사용 ({:?}): {}",
                                legacy_data_dir, e
                            );
                            legacy_data_dir
                        }
                    }
                }
            } else {
                // 신규 설치 — 실행 위치에 새로 생성
                preferred_data_dir
            };

            // 웹 서버 포트 (WEB_PORT 환경변수, 기본 7474)
            let web_port: u16 = std::env::var("WEB_PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(7474);

            // 공통 데이터 갱신 주기 환경변수 폴백 (UI 저장 없으면 이 값 사용)
            let refresh_interval_env: u64 = std::env::var("REFRESH_INTERVAL_SEC")
                .ok()
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or(30)
                .max(5);
            // 데이터 갱신 주기 설정 로드 (.env REFRESH_INTERVAL_SEC 우선, 없으면 환경변수 폴백)
            let refresh_config = RefreshConfig::load_from_env(refresh_interval_env);
            let (interval_tx, _) = tokio::sync::watch::channel(refresh_config.interval_sec);
            tracing::info!(
                "환경 설정: WEB_PORT={}, 갱신 주기={}s (.env 또는 기본값)",
                web_port, refresh_config.interval_sec
            );

            // AppState 초기화 및 등록
            let state = AppState::new(config, discord_config, profiles, profiles_path, data_dir.clone(), log_dir.clone(), log_cfg, web_port, refresh_config, interval_tx);
            app.manage(state);

            // 시작 시 체결 기록 즉시 정리 (로그와 동일하게 시작 시 1회 실행)
            {
                let st: tauri::State<AppState> = app.state();
                // setup 클로저는 동기 컨텍스트이므로 blocking_read() + std::thread::spawn 사용
                // tokio::task::spawn_blocking은 runtime handle이 없으면 패닉하므로 사용 불가
                let trade_cfg  = st.trade_archive_config.blocking_read().clone();
                let data_dir_c = st.data_dir.clone();
                std::thread::spawn(move || {
                    commands::purge_old_trade_files(&data_dir_c, &trade_cfg);
                    tracing::info!("시작 시 체결 기록 정리 완료 (보관 {}일)", trade_cfg.retention_days);
                });
            }

            // ── 비동기 백그라운드 작업 ──────────────────────────
            // 1) KRX 종목 목록 로드 (캐시 우선, 없으면 다운로드)
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let items = market::StockList::load_or_fetch(&data_dir).await;
                let st: tauri::State<AppState> = app_handle.state();
                // KRX 결과가 있으면 stock_list(레거시) + stock_store(영구) 양쪽 저장
                if !items.is_empty() {
                    st.stock_store.upsert_many(
                        items.iter().map(|i| (i.pdno.clone(), i.prdt_name.clone()))
                    ).await;
                    *st.stock_list.write().await = items;
                }
            });

            // 2) 자동매매 폴링 데몬 영구 spawn (is_trading 플래그로 활성/비활성 제어)
            {
                let st: tauri::State<AppState> = app.state();
                let is_trading   = st.is_trading.clone();
                let strategy_mgr = st.strategy_manager.clone();
                let order_mgr    = st.order_manager.clone();
                let risk_mgr     = st.risk_manager.clone();
                let rest_arc     = st.rest_client.clone();
                let stock_store  = st.stock_store.clone();
                tauri::async_runtime::spawn(commands::run_trading_daemon(
                    is_trading, strategy_mgr, order_mgr, risk_mgr, rest_arc, stock_store,
                ));
            }

            // 3) 모바일 웹 서버 시작 (포트 web_port) — React 앱(dist/) 서비스
            {
                let st: tauri::State<AppState> = app.state();
                let rest_client          = st.rest_client.clone();
                let stock_list           = st.stock_list.clone();
                let port                 = st.web_port;
                let is_trading           = st.is_trading.clone();
                let strategy_manager     = st.strategy_manager.clone();
                let position_tracker     = st.position_tracker.clone();
                let config               = st.config.clone();
                let profiles             = st.profiles.clone();
                let trade_store          = st.trade_store.clone();
                let stats_store          = st.stats_store.clone();
                let log_config           = st.log_config.clone();
                let log_dir              = st.log_dir.clone();
                let trade_archive_config = st.trade_archive_config.clone();
                let data_dir             = st.data_dir.clone();
                let risk_manager         = st.risk_manager.clone();
                let order_manager        = st.order_manager.clone();
                let stock_store          = st.stock_store.clone();
                let strategy_store       = st.strategy_store.clone();
                let profiles_path        = st.profiles_path.clone();
                let discord              = st.discord.clone();
                let exchange_rate_krw    = st.exchange_rate_krw.clone();
                let refresh_config       = st.refresh_config.clone();
                tauri::async_runtime::spawn(async move {
                    server::start(
                        rest_client, stock_list, port,
                        is_trading, strategy_manager, position_tracker,
                        config, profiles, trade_store, stats_store,
                        log_config, log_dir, trade_archive_config, data_dir,
                        risk_manager, order_manager, stock_store, strategy_store,
                        profiles_path, discord, exchange_rate_krw, refresh_config,
                    ).await;
                });
            }

            // 4) 환율 갱신 데몬 — refresh_config.interval_sec마다 USD/KRW 환율 업데이트 + 이벤트 발행
            {
                let st: tauri::State<AppState> = app.state();
                let exchange_rate = st.exchange_rate_krw.clone();
                let interval_tx   = st.refresh_interval_tx.clone();
                let app_handle    = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    let mut interval_rx = interval_tx.subscribe();
                    // 시작 시 즉시 1회 갱신
                    match crate::api::rest::fetch_usd_krw_rate().await {
                        Ok(rate) => {
                            *exchange_rate.write().await = rate;
                            let _ = app_handle.emit("exchange-rate-updated", rate);
                            tracing::info!("USD/KRW 환율 초기값: {:.2}원", rate);
                        }
                        Err(e) => tracing::warn!("환율 초기 조회 실패 (기본값 1450원 사용): {}", e),
                    }
                    let mut current_interval = *interval_rx.borrow_and_update();
                    loop {
                        tokio::select! {
                            _ = tokio::time::sleep(tokio::time::Duration::from_secs(current_interval)) => {
                                match crate::api::rest::fetch_usd_krw_rate().await {
                                    Ok(rate) => {
                                        *exchange_rate.write().await = rate;
                                        let _ = app_handle.emit("exchange-rate-updated", rate);
                                        tracing::debug!("USD/KRW 환율 갱신: {:.2}원", rate);
                                    }
                                    Err(e) => tracing::warn!("환율 갱신 실패 (이전 값 유지): {}", e),
                                }
                            }
                            _ = interval_rx.changed() => {
                                current_interval = *interval_rx.borrow_and_update();
                                tracing::info!("환율 갱신 주기 변경: {}초", current_interval);
                            }
                        }
                    }
                });
            }

            // 5) 일일 로그/체결 기록 정리 데몬 — 24시간마다 retention 설정대로 파일 삭제
            // (앱 시작 시 cleanup 이외에, 장기 실행 시에도 자동 정리 보장)
            {
                let st: tauri::State<AppState> = app.state();
                let log_config_arc           = st.log_config.clone();
                let trade_archive_config_arc = st.trade_archive_config.clone();
                let log_dir_d  = st.log_dir.clone();
                let data_dir_d = st.data_dir.clone();
                tauri::async_runtime::spawn(async move {
                    loop {
                        // 24시간 대기 후 정리
                        tokio::time::sleep(tokio::time::Duration::from_secs(24 * 3600)).await;

                        let log_cfg   = log_config_arc.read().await.clone();
                        crate::logging::cleanup(&log_dir_d, &log_cfg);
                        tracing::info!("일일 로그 정리 완료 (보관 {}일)", log_cfg.retention_days);

                        let trade_cfg  = trade_archive_config_arc.read().await.clone();
                        let data_dir_c = data_dir_d.clone();
                        tokio::task::spawn_blocking(move || {
                            commands::purge_old_trade_files(&data_dir_c, &trade_cfg);
                            tracing::info!("일일 체결 기록 정리 완료 (보관 {}일)", trade_cfg.retention_days);
                        });
                    }
                });
            }
            // 6) 잠고 백그라운드 갱신 데몬 — refresh_config.interval_sec마다 잔고 조회 후 이벤트 발행
            // 프론트엔드는 balance-updated / overseas-balance-updated 이벤트를 리신하여
            // TanStack Query 캐시를 직접 갱신합니다 (폴링 전환 가능).
            {
                let st: tauri::State<AppState> = app.state();
                let rest_arc         = st.rest_client.clone();
                let interval_tx      = st.refresh_interval_tx.clone();
                let stock_store_arc  = st.stock_store.clone();
                let position_tracker = st.position_tracker.clone();
                let app_handle       = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    let mut interval_rx = interval_tx.subscribe();
                    // 앱 초기화 완료 대기 (5초)
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    let mut current_interval = *interval_rx.borrow_and_update();
                    loop {
                        let client = rest_arc.read().await.clone();
                        if !client.app_key().is_empty() {
                            // 국내 잔고
                            match client.get_balance().await {
                                Ok(resp) => {
                                    stock_store_arc.upsert_many(
                                        resp.items.iter().map(|i| (i.pdno.clone(), i.prdt_name.clone()))
                                    ).await;
                                    {
                                        let mut tracker = position_tracker.lock().await;
                                        tracker.load_if_empty(
                                            resp.items.iter().map(|i| (
                                                i.pdno.clone(),
                                                i.prdt_name.clone(),
                                                i.hldg_qty.parse::<u64>().unwrap_or(0),
                                                i.pchs_avg_pric.parse::<f64>().unwrap_or(0.0) as u64,
                                                i.prpr.parse::<u64>().unwrap_or(0),
                                            ))
                                        );
                                    }
                                    let payload = serde_json::json!({
                                        "items": resp.items,
                                        "summary": resp.summary,
                                    });
                                    let _ = app_handle.emit("balance-updated", payload);
                                }
                                Err(e) => tracing::debug!("잔고 백그라운드 조회 실패: {}", e),
                            }
                            // 해외 잔고
                            match client.get_overseas_balance().await {
                                Ok(resp) => {
                                    let payload = serde_json::json!({
                                        "items": resp.items,
                                        "summary": resp.summary,
                                    });
                                    let _ = app_handle.emit("overseas-balance-updated", payload);
                                }
                                Err(e) => tracing::debug!("해외 잔고 백그라운드 조회 실패: {}", e),
                            }
                        }
                        tokio::select! {
                            _ = tokio::time::sleep(tokio::time::Duration::from_secs(current_interval)) => {}
                            _ = interval_rx.changed() => {
                                current_interval = *interval_rx.borrow_and_update();
                                tracing::info!("잔고 갱신 주기 변경: {}초", current_interval);
                            }
                        }
                    }
                });
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_app_config,
            commands::check_config,
            commands::list_profiles,
            commands::add_profile,
            commands::update_profile,
            commands::delete_profile,
            commands::set_active_profile,
            commands::get_balance,
            commands::get_overseas_balance,
            commands::get_chart_data,
            commands::get_price,
            commands::place_order,
            commands::get_today_executed,
            commands::get_today_trades,
            commands::get_trades_by_range,
            commands::get_today_stats,
            commands::get_stats_by_range,
            commands::send_test_discord,
            commands::save_trade,
            commands::upsert_daily_stats,
            commands::get_trading_status,
            commands::start_trading,
            commands::stop_trading,
            commands::get_positions,
            commands::get_strategies,
            commands::update_strategy,
            commands::get_log_config,
            commands::set_log_config,
            commands::get_trade_archive_config,
            commands::set_trade_archive_config,
            commands::get_trade_archive_stats,
            commands::write_frontend_log,
            commands::search_stock,
            commands::refresh_stock_list,
            commands::get_stock_list_stats,
            commands::set_stock_update_interval,
            commands::get_kis_executed_by_range,
            commands::get_recent_logs,
            commands::check_for_update,
            commands::get_web_config,
            commands::save_web_config,
            commands::detect_trading_type,
            commands::detect_profile_trading_type,
            commands::get_overseas_price,
            commands::get_overseas_chart_data,
            commands::place_overseas_order,
            commands::get_risk_config,
            commands::update_risk_config,
            commands::clear_emergency_stop,
            commands::activate_emergency_stop,
            commands::get_pending_orders,
            commands::get_exchange_rate,
            commands::get_refresh_interval,
            commands::get_refresh_config,
            commands::set_refresh_config,
            commands::clear_buy_suspension,
        ])
        .on_window_event(|window, event| {
            // 앱 종료 요청 시 자동매매 정지 신호 전송 (트레이딩 데몬 루프 안전 종료)
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                let state: tauri::State<commands::AppState> = window.state();
                let is_trading = state.is_trading.clone();
                tauri::async_runtime::spawn(async move {
                    let mut guard = is_trading.lock().await;
                    if *guard {
                        *guard = false;
                        tracing::info!("앱 종료 — 자동매매 정지 신호 전송");
                    }
                });
                tracing::info!("앱 종료 요청");
            }
        })
        .run(tauri::generate_context!())
        .expect("Tauri 애플리케이션 실행 중 오류 발생");
}
