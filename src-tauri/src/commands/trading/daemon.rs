use super::*;

mod fence;
use crate::trading::order::ExecutionRunContext;
use fence::{capture_cycle, RunFence};

// ────────────────────────────────────────────────────────────────────
// 자동매매 폴링 데몬 (lib.rs 에서 앱 시작 시 영구 spawn)
//
// is_trading 플래그가 false 이면 5초마다 재확인하며 대기.
// true 로 바뀌면 즉시 폴링 재개. start_trading / web API 모두 이 방식으로 제어.
//
// 설계 원칙:
//   - 레이블 루프(labeled loop)·goto 유사 패턴 사용 금지
//   - 제어 흐름은 항상 위→아래 순차 실행
//   - 내부 루프에서 외부 루프로 점프하는 break 'label / continue 'label 금지
//   - 내부 루프 조기 탈출이 필요한 경우 별도 함수로 추출 후 return 사용
// ────────────────────────────────────────────────────────────────────

/// 종목 폴링 한 사이클(틱)의 처리 결과
///
/// `poll_symbols_tick` 반환값으로 사용. 호출자가 결과에 따라
/// 시장 대기 여부를 결정한다.
#[derive(Debug, PartialEq)]
enum TickCycleResult {
    /// 모든 종목 정상 처리 완료
    Done,
    /// 장 마감 / 장외 시간 감지 → 호출자는 market_pause_until 설정 후 다음 이터레이션으로
    MarketClosed,
    /// is_trading 플래그가 false 로 바뀜 → 이번 사이클 조기 종료
    Stopped,
}

/// 자동매매 시세 폴링에 사용할 broker 소스.
///
/// 실행 scope가 Toss면 해당 프로파일 자격증명으로 Toss 시세를 조회하고,
/// 그 외(KIS)에는 기존 `KisRestClient` 경로를 그대로 사용한다.
enum PriceSource {
    Kis,
    Toss(AccountProfile),
}

fn toss_us_session_policy_from_params(params: &serde_json::Value) -> UsTradingSessionPolicy {
    let value = params
        .get("toss_us_session")
        .or_else(|| params.get("tossUsSession"))
        .and_then(|value| value.as_str());
    UsTradingSessionPolicy::parse(value)
}

fn strategy_market_open_for_symbol(
    config: &crate::trading::strategy::StrategyConfig,
    symbol: &str,
    price_source: &PriceSource,
    market_calendar: Option<&MarketCalendarOverride>,
) -> bool {
    if matches!(price_source, PriceSource::Toss(_)) && !is_domestic_symbol(symbol) {
        is_market_open_for_with_calendar_policy(
            symbol,
            market_calendar,
            toss_us_session_policy_from_params(&config.params),
        )
    } else {
        is_market_open_for_with_calendar(symbol, market_calendar)
    }
}

/// Toss 종목 현재가 조회 → (가격, 거래량) 튜플로 변환
///
/// 가격 단위는 KIS 경로와 동일하게 맞춘다: KRW는 원 단위 정수, USD는 센트(×100) 정수.
/// Toss `/api/v1/prices`는 거래량을 제공하지 않아 volume은 항상 0으로 반환한다
/// (leveraged_trend_hold 등 volume을 사용하지 않는 전략에는 영향 없음).
async fn fetch_toss_tick(profile: &AccountProfile, symbol: &str) -> anyhow::Result<(u64, u64)> {
    let normalized_symbol = symbol.trim().to_uppercase();
    if normalized_symbol.is_empty() {
        anyhow::bail!("Toss 현재가 조회 symbol이 비어 있습니다.");
    }
    let adapter = TossBrokerAdapter::with_credentials(
        TossBrokerAdapter::DEFAULT_BASE_URL,
        profile.app_key.clone(),
        profile.app_secret.clone(),
        Some(profile.account_no.clone()),
    );
    let quote = adapter
        .get_price(&BrokerSymbol(normalized_symbol.clone()))
        .await
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;
    let amount = quote.last.amount.trim().replace(',', "");
    let price_amount = amount.parse::<f64>().map_err(|e| {
        anyhow::anyhow!("Toss 현재가 파싱 실패({normalized_symbol}): {amount} ({e})")
    })?;
    if price_amount <= 0.0 {
        anyhow::bail!("Toss 현재가가 0 이하입니다: {normalized_symbol}={amount}");
    }
    let price = match quote.last.currency {
        BrokerCurrency::Krw => price_amount.round() as u64,
        BrokerCurrency::Usd => (price_amount * 100.0).round() as u64,
    };
    let volume = quote
        .volume
        .and_then(|q| q.0.trim().replace(',', "").parse::<u64>().ok())
        .unwrap_or(0);
    Ok((price, volume))
}

/// 종목 목록을 순차적으로 순회하여 현재가 조회 + 전략 신호 처리
///
/// `break 'label` / `continue 'label` 없이 `return`으로 조기 탈출한다.
/// 장 마감 감지 시 `TickCycleResult::MarketClosed` 를 반환하고,
/// 호출자(run_trading_daemon)가 market_pause_until 을 설정한다.
#[allow(clippy::too_many_arguments)]
async fn poll_symbols_tick(
    symbols: &[String],
    is_trading: &Arc<Mutex<bool>>,
    strategy_mgr: &Arc<Mutex<crate::trading::strategy::StrategyManager>>,
    order_mgr: &Arc<Mutex<crate::trading::order::OrderManager>>,
    stock_store: &Arc<crate::storage::stock_store::StockStore>,
    rest: &Arc<KisRestClient>,
    price_source: &PriceSource,
    delay_ms: u64,
    total_balance_krw: i64,
    market_calendar: Option<&MarketCalendarOverride>,
    expected: &ExecutionRunContext,
) -> TickCycleResult {
    let fence = RunFence {
        running: is_trading,
        strategies: strategy_mgr,
        orders: order_mgr,
        expected,
    };
    for symbol in symbols {
        // 종료 플래그 선행 확인
        if !fence.is_current().await {
            return TickCycleResult::Stopped;
        }

        // 해당 종목을 사용하는 활성 전략 중 현재 허용된 세션이 있는지 확인한다.
        // Toss 미국 전략은 params.toss_us_session에 따라 day/pre/regular/after/auto gate를 적용한다.
        let Some(symbol_has_open_strategy) = fence
            .with_current(|manager| {
                manager.any_active_config_for_symbol(symbol, |config| {
                    strategy_market_open_for_symbol(config, symbol, price_source, market_calendar)
                })
            })
            .await
        else {
            return TickCycleResult::Stopped;
        };
        if !symbol_has_open_strategy {
            tracing::debug!(
                "시장 폐장 — 건너뜀: {} ({})",
                symbol,
                if is_domestic_symbol(symbol) {
                    "KRX"
                } else {
                    "US"
                }
            );
            continue;
        }

        // 현재가 조회 + 해외 주문용 거래소 코드 캡처 (활성 broker 프로파일에 맞는 소스로 조회)
        let Some((tick, exchange_opt)) = fence
            .after(async {
                match price_source {
                    PriceSource::Toss(profile) => match fetch_toss_tick(profile, symbol).await {
                        Ok((price, volume)) => {
                            let exch = (!is_domestic_symbol(symbol)).then(|| "TOSS_US".to_string());
                            (Ok((price, volume)), exch)
                        }
                        Err(e) => (Err(e.to_string()), None),
                    },
                    PriceSource::Kis if is_domestic_symbol(symbol) => {
                        let t = rest
                            .get_price(symbol)
                            .await
                            .map(|p| {
                                let price = p.stck_prpr.parse::<u64>().unwrap_or(0);
                                let volume = p.acml_vol.parse::<u64>().unwrap_or(0);
                                (price, volume)
                            })
                            .map_err(|e| e.to_string());
                        (t, None)
                    }
                    PriceSource::Kis => match fetch_overseas_tick(rest, symbol).await {
                        Ok((price, volume, exch)) => (Ok((price, volume)), Some(exch)),
                        Err(e) => (Err(e.to_string()), None),
                    },
                }
            })
            .await
        else {
            return TickCycleResult::Stopped;
        };

        // 틱 처리: 현재가 기반 전략 신호 생성 → 주문
        match tick {
            Ok((price, volume)) if price > 0 => {
                let Some(signals) = fence
                    .with_current(|manager| {
                        manager
                            .invalidate_stale_daily_history_for_symbol(symbol, chrono::Utc::now());
                        manager.on_tick_filtered(symbol, price, volume, |config| {
                            strategy_market_open_for_symbol(
                                config,
                                symbol,
                                price_source,
                                market_calendar,
                            )
                        })
                    })
                    .await
                else {
                    return TickCycleResult::Stopped;
                };
                for strategy_signal in signals {
                    if !fence.is_current().await {
                        return TickCycleResult::Stopped;
                    }
                    let Some(symbol_name) = fence.after(stock_store.get_name(symbol)).await else {
                        return TickCycleResult::Stopped;
                    };
                    let symbol_name = symbol_name.unwrap_or_else(|| symbol.clone());
                    let rollback_snapshot = crate::trading::order::OrderManager::current_position_snapshot_for_signal_shared(
                        order_mgr,
                        &strategy_signal.signal,
                        exchange_opt.clone(),
                        price,
                    )
                    .await;
                    if !fence.is_current().await {
                        return TickCycleResult::Stopped;
                    }
                    let submit_result = crate::trading::order::OrderManager::submit_signal_shared(
                        order_mgr,
                        expected.clone(),
                        strategy_signal,
                        symbol_name,
                        total_balance_krw,
                        exchange_opt.clone(),
                        price,
                    )
                    .await;
                    match submit_result {
                        Ok(crate::trading::order::SubmissionOutcome::Submitted {
                            provider_order_id,
                        }) => {
                            tracing::info!(
                                "전략 주문 제출 완료: symbol={} providerOrderId={}",
                                symbol,
                                provider_order_id
                            );
                            tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
                            if !fence.is_current().await {
                                return TickCycleResult::Stopped;
                            }
                        }
                        Ok(crate::trading::order::SubmissionOutcome::StaleRun { reason }) => {
                            tracing::debug!("이전 자동매매 실행의 신호를 폐기합니다: {}", reason);
                            return TickCycleResult::Stopped;
                        }
                        Ok(crate::trading::order::SubmissionOutcome::Skipped {
                            reason,
                            held_quantity,
                            avg_price,
                        }) => {
                            // 전략이 raw signal 생성 시 내부 포지션 상태를 먼저 바꿨더라도
                            // guard/risk/provider가 주문을 거부하면 실제 tracker 상태로 되돌린다.
                            if fence
                                .with_current(|manager| {
                                    manager.sync_position(
                                        symbol,
                                        held_quantity,
                                        avg_price.unwrap_or(0),
                                    )
                                })
                                .await
                                .is_none()
                            {
                                return TickCycleResult::Stopped;
                            }
                            tracing::debug!("전략 주문 스킵: symbol={} reason={}", symbol, reason);
                        }
                        Err(e) => {
                            if fence
                                .with_current(|manager| {
                                    manager.sync_position(
                                        symbol,
                                        rollback_snapshot.0,
                                        rollback_snapshot.1.unwrap_or(0),
                                    )
                                })
                                .await
                                .is_none()
                            {
                                return TickCycleResult::Stopped;
                            }
                            let msg = e.to_string();
                            if is_market_closed_error(&msg) {
                                tracing::info!(
                                    "장 마감/장외 시간 감지 (주문, {}) — 5분 대기: {}",
                                    symbol,
                                    msg
                                );
                                return TickCycleResult::MarketClosed;
                            }
                            tracing::warn!("신호 처리 실패 ({}): {}", symbol, msg);
                        }
                    }
                }
            }
            Ok(_) => {
                tracing::debug!("현재가 0 — 건너뜀: {}", symbol);
            }
            Err(e) => {
                if is_market_closed_error(&e) {
                    tracing::info!(
                        "장 마감/장외 시간 감지 (현재가, {}) — 5분 대기: {}",
                        symbol,
                        e
                    );
                    return TickCycleResult::MarketClosed;
                }
                tracing::warn!("현재가 조회 실패 ({}): {}", symbol, e);
            }
        }

        // 종목 간 API 호출 딜레이
        tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;

        // 딜레이 후 종료 플래그 재확인
        if !fence.is_current().await {
            return TickCycleResult::Stopped;
        }
    }
    TickCycleResult::Done
}

#[allow(clippy::too_many_arguments)]
pub async fn run_trading_daemon(
    is_trading: Arc<Mutex<bool>>,
    strategy_mgr: Arc<Mutex<crate::trading::strategy::StrategyManager>>,
    order_mgr: Arc<Mutex<crate::trading::order::OrderManager>>,
    risk_mgr: Arc<Mutex<crate::trading::risk::RiskManager>>,
    _rest_arc: Arc<RwLock<Arc<KisRestClient>>>,
    stock_store: Arc<crate::storage::stock_store::StockStore>,
    profiles: Arc<RwLock<ProfilesConfig>>,
    exchange_rate_krw: Arc<RwLock<f64>>,
) {
    tracing::info!("자동매매 폴링 데몬 시작 (is_trading=false 대기 중)");
    let mut last_reset_date = chrono::Local::now().date_naive();
    let mut market_pause_until: Option<tokio::time::Instant> = None;
    let mut last_run: Option<ExecutionRunContext> = None;
    let mut daily_history_jobs = super::daily_history::DailyHistoryJobs::default();

    // 레이블 없는 단순 루프 — 제어 흐름은 위→아래 순차 실행
    loop {
        let is_running = *is_trading.lock().await;

        // pending reconciliation은 자동매매 on/off와 무관하게 계속 수행한다.
        if let Err(error) = OrderManager::confirm_pending_fills_from_broker_shared(&order_mgr).await
        {
            tracing::warn!("provider 미체결 주문 대조 실패: {}", error);
        }
        // Refill completed history even during the market pause. These calls only
        // schedule jobs or drain finished handles; they never await provider data.
        daily_history_jobs
            .maintain(&is_trading, &strategy_mgr, &order_mgr)
            .await;

        // ── Phase 1: 자동매매 비활성 → 5초 대기 후 재확인 ──────────
        if !is_running {
            if last_run.is_some() {
                market_pause_until = None;
                tracing::info!("자동매매 폴링 데몬 일시 정지 (is_trading=false)");
            }
            last_run = None;
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            continue;
        }

        // Capture identity and provider snapshots after reconciliation/history awaits.
        let Some(cycle) = capture_cycle(&is_trading, &strategy_mgr, &order_mgr).await else {
            market_pause_until = None;
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            continue;
        };
        let fence = RunFence {
            running: &is_trading,
            strategies: &strategy_mgr,
            orders: &order_mgr,
            expected: &cycle.context,
        };
        if last_run.as_ref() != Some(&cycle.context) {
            last_run = Some(cycle.context.clone());
            market_pause_until = None;
            last_reset_date = chrono::Local::now().date_naive();
            tracing::info!("자동매매 실행·설정 변경 — 폴링 상태 초기화");
        }

        // ── Phase 3: 장 마감으로 대기 중 → 30초 슬립 후 재확인 ─────
        if let Some(pause_until) = market_pause_until {
            if tokio::time::Instant::now() < pause_until {
                tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
                continue;
            }
            tracing::info!("장 마감 대기 완료 — 폴링 재개");
            market_pause_until = None;
        }

        // ── Phase 4: 날짜 변경 시 일별 초기화 ──────────────────────
        let today = chrono::Local::now().date_naive();
        if today != last_reset_date {
            if !fence.reset_day(&risk_mgr).await {
                continue;
            }
            last_reset_date = today;
            tracing::info!("자동매매 일별 초기화 완료 ({})", today);
        }

        let symbols = cycle.symbols;
        if symbols.is_empty() {
            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
            continue;
        }
        // Profiles sharing an account must never replace the exact run snapshot.
        let price_source = if cycle.context.broker_scope.broker_id == BrokerId::Toss {
            match cycle.profile {
                Some(profile) => PriceSource::Toss(profile),
                None => {
                    tracing::warn!("자동매매 Toss 실행 프로파일 snapshot이 없습니다.");
                    continue;
                }
            }
        } else {
            PriceSource::Kis
        };
        let rest = cycle.rest;
        let delay_ms: u64 = if rest.is_paper() { 700 } else { 150 };

        // ── Phase 7: 전체 시장 폐장 여부 사전 체크 ─────────────────
        let market_calendar = get_active_toss_calendar_override(&profiles).await;

        let Some(all_markets_closed) = fence
            .with_current(|mgr| {
                symbols.iter().all(|symbol| {
                    !mgr.any_active_config_for_symbol(symbol, |config| {
                        strategy_market_open_for_symbol(
                            config,
                            symbol,
                            &price_source,
                            market_calendar.as_ref(),
                        )
                    })
                })
            })
            .await
        else {
            continue;
        };

        if all_markets_closed {
            tracing::info!(
                "모든 시장 폐장 ({}) — 5분 대기 후 재확인",
                open_markets_summary_with_calendar(market_calendar.as_ref())
            );
            market_pause_until =
                Some(tokio::time::Instant::now() + tokio::time::Duration::from_secs(300));
            continue;
        }
        tracing::debug!(
            "시장 상태: {}",
            open_markets_summary_with_calendar(market_calendar.as_ref())
        );

        let exchange_rate = *exchange_rate_krw.read().await;
        let total_balance_result = match &price_source {
            PriceSource::Toss(profile) => fetch_toss_risk_balance_krw(profile, exchange_rate).await,
            PriceSource::Kis => {
                fetch_account_risk_balance_krw(
                    &rest,
                    symbols.iter().any(|s| !is_domestic_symbol(s)),
                    exchange_rate,
                )
                .await
            }
        };
        let Some(total_balance_krw) = fence.apply_balance(total_balance_result).await else {
            continue;
        };

        // ── Phase 8: 종목별 현재가 조회 + 전략 신호 처리 ───────────
        //   내부 루프는 poll_symbols_tick() 으로 분리 — goto 유사 패턴 없음
        let tick_result = poll_symbols_tick(
            &symbols,
            &is_trading,
            &strategy_mgr,
            &order_mgr,
            &stock_store,
            &rest,
            &price_source,
            delay_ms,
            total_balance_krw,
            market_calendar.as_ref(),
            &cycle.context,
        )
        .await;

        if tick_result == TickCycleResult::Stopped {
            continue;
        }
        if tick_result == TickCycleResult::MarketClosed && fence.is_current().await {
            market_pause_until =
                Some(tokio::time::Instant::now() + tokio::time::Duration::from_secs(300));
            continue;
        }

        // ── Phase 9: 다음 틱까지 10초 대기 (100ms 단위 — 종료 신호 즉시 반응) ──
        for _ in 0u32..100 {
            if !fence.is_current().await {
                break;
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    }
}

// ────────────────────────────────────────────────────────────────────
// 자동매매 폴링 루프 헬퍼
// ────────────────────────────────────────────────────────────────────

/// 장 마감 / 장외 시간 오류 여부 감지
///
/// KIS API가 시장 비운영 시간에 반환하는 공통 메시지 패턴을 검사한다.
///
/// ## 실제 KIS 응답 예시 (에러 로그에서 수집)
/// - `"모의투자 장종료 입니다."`
/// - `"모의투자 장시작전 입니다."`
/// - `"장운영시간이 아닙니다."`
/// - `"시간외거래"`
fn is_market_closed_error(msg: &str) -> bool {
    msg.contains("장종료")
        || msg.contains("장마감")
        || msg.contains("장시작전")
        || msg.contains("장운영시간")
        || msg.contains("시간외거래")
        || msg.contains("OPCODE-100")
}

/// 해외 주식 현재가 조회 (NAS → NYS → AMS 순으로 시도)
/// 반환값: (price_cents: u64, volume: u64, exchange: String)
/// - price_cents = USD 현재가 × 100 (정수화하여 on_tick에 전달)
/// - exchange = 성공한 거래소 코드 ("NAS" / "NYS" / "AMS")
async fn fetch_overseas_tick(
    rest: &std::sync::Arc<crate::api::rest::KisRestClient>,
    symbol: &str,
) -> anyhow::Result<(u64, u64, String)> {
    for exchange in &["NAS", "NYS", "AMS"] {
        match rest.get_overseas_price(symbol, exchange).await {
            Ok(p) => {
                let price_f: f64 = p.last.parse().unwrap_or(0.0);
                if price_f > 0.0 {
                    // USD → 센트(×100) 변환으로 u64 정수화
                    let price_cents = (price_f * 100.0).round() as u64;
                    let volume = p.tvol.parse::<u64>().unwrap_or(0);
                    return Ok((price_cents, volume, exchange.to_string()));
                }
            }
            Err(_) => continue,
        }
    }
    anyhow::bail!("해외 현재가 조회 실패: {} (NAS/NYS/AMS 모두 실패)", symbol)
}
