pub mod api;
pub mod commands;
pub mod config;
pub mod logging;
pub mod market;
pub mod notifications;
pub mod server;
pub mod storage;
pub mod trading;

use std::path::PathBuf;
use tauri::Manager;

use commands::AppState;
use config::{AppConfig, DiscordConfig, ProfilesConfig};
use logging::LogConfig;

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

            tracing::info!("AutoConditionTrade 시작 - 데이터 경로: {:?}", app_data_dir);
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

            // 데이터 저장 경로
            let data_dir = app_data_dir.join("data");

            // AppState 초기화 및 등록
            let state = AppState::new(config, discord_config, profiles, profiles_path, data_dir.clone(), log_dir.clone(), log_cfg);
            app.manage(state);

            // ── 비동기 백그라운드 작업 ──────────────────────────
            // 1) KRX 종목 목록 로드 (캐시 우선, 없으면 다운로드)
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let items = market::StockList::load_or_fetch(&data_dir).await;
                let st: tauri::State<AppState> = app_handle.state();
                *st.stock_list.write().await = items;
            });

            // 2) 모바일 웹 서버 시작 (포트 7474)
            {
                let st: tauri::State<AppState> = app.state();
                let rest_client = st.rest_client.clone();
                tauri::async_runtime::spawn(async move {
                    server::start(rest_client, 7474).await;
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
            commands::write_frontend_log,
            commands::search_stock,
            commands::refresh_stock_list,
            commands::get_kis_executed_by_range,
            commands::get_recent_logs,
        ])
        .run(tauri::generate_context!())
        .expect("Tauri 애플리케이션 실행 중 오류 발생");
}
