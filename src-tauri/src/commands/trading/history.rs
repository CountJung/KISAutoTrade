use super::*;

fn broker_money_to_strategy_units(money: &BrokerMoney) -> Option<u64> {
    let amount = money.amount.trim().replace(',', "").parse::<f64>().ok()?;
    if amount <= 0.0 {
        return None;
    }
    Some(match money.currency {
        BrokerCurrency::Krw => amount.round() as u64,
        BrokerCurrency::Usd => (amount * 100.0).round() as u64,
    })
}

fn broker_candles_to_ohlc(candles: &[BrokerCandle]) -> Vec<OhlcCandle> {
    let mut candles = candles.to_vec();
    candles.sort_by(|a, b| a.date.cmp(&b.date));
    candles
        .iter()
        .filter_map(|c| {
            Some(OhlcCandle {
                open: broker_money_to_strategy_units(&c.open)?,
                high: broker_money_to_strategy_units(&c.high)?,
                low: broker_money_to_strategy_units(&c.low)?,
                close: broker_money_to_strategy_units(&c.close)?,
            })
        })
        .filter(|c| c.open > 0 && c.high > 0 && c.low > 0 && c.close > 0)
        .collect()
}

async fn apply_ohlc_history(
    strategy_manager: &Arc<Mutex<StrategyManager>>,
    risk_manager: &Arc<Mutex<RiskManager>>,
    symbol: &str,
    ohlc: Vec<OhlcCandle>,
    source: &str,
) {
    if ohlc.is_empty() {
        return;
    }
    let candle_count = ohlc.len();
    strategy_manager
        .lock()
        .await
        .initialize_warmup(symbol, &ohlc, &[]);
    tracing::info!(
        "전략 히스토리 초기화 완료: {} @ {} ({}봉)",
        symbol,
        source,
        candle_count
    );
    if let Some(atr) = calculate_atr(&ohlc, 14) {
        risk_manager.lock().await.set_symbol_atr(symbol, atr);
        tracing::info!(
            "리스크 ATR 초기화 완료: {} @ {} ATR14={}",
            symbol,
            source,
            atr
        );
    }
}

async fn apply_intraday_ohlc(
    strategy_manager: &Arc<Mutex<StrategyManager>>,
    symbol: &str,
    ohlc: Vec<OhlcCandle>,
    source: &str,
) {
    if ohlc.is_empty() {
        return;
    }
    strategy_manager
        .lock()
        .await
        .initialize_warmup(symbol, &[], &ohlc);
    tracing::info!(
        "전략 장중 OHLC 초기화 완료: {} @ {} ({}봉)",
        symbol,
        source,
        ohlc.len()
    );
}

pub(super) fn legacy_history_symbols(manager: &StrategyManager) -> Vec<String> {
    manager
        .active_symbols()
        .into_iter()
        .filter(|symbol| {
            manager.any_active_config_for_symbol(symbol, |config| {
                !config.id.starts_with("fifty_two_week_high")
            })
        })
        .collect()
}

pub(crate) async fn initialize_active_strategy_history(
    strategy_manager: &Arc<Mutex<StrategyManager>>,
    order_manager: &Arc<Mutex<crate::trading::order::OrderManager>>,
    profiles: &Arc<RwLock<ProfilesConfig>>,
    rest_client: &Arc<RwLock<Arc<KisRestClient>>>,
    risk_manager: &Arc<Mutex<RiskManager>>,
) {
    // 활성 전략의 종목별 일봉 차트 데이터 로드 → 히스토리 기반 전략 초기화 (52주 신고가 등)
    // 국내 종목: get_chart_data (KRW 정수 가격)
    // 해외 종목: get_overseas_chart_data (USD float → ×100 센트로 정수화)
    // The 52-week strategy is prepared exclusively by scoped completed-page jobs.
    // A symbol also used by another strategy still receives that strategy's legacy seed.
    let active_symbols = legacy_history_symbols(&*strategy_manager.lock().await);
    if !active_symbols.is_empty() {
        let execution_scope = order_manager.lock().await.execution_scope().clone();
        let toss_adapter = if execution_scope.broker_id == BrokerId::Toss {
            resolve_scoped_profile(profiles, &execution_scope)
                .await
                .map(|profile| {
                    TossBrokerAdapter::with_credentials(
                        TossBrokerAdapter::DEFAULT_BASE_URL,
                        profile.app_key,
                        profile.app_secret,
                        Some(profile.account_no),
                    )
                })
        } else {
            None
        };
        if execution_scope.broker_id == BrokerId::Toss && toss_adapter.is_none() {
            tracing::warn!("Toss 실행 scope의 프로파일을 찾지 못해 히스토리 초기화를 중단합니다.");
            return;
        }
        let rest = rest_client.read().await.clone();
        let today = chrono::Local::now();
        let end_date = today.format("%Y%m%d").to_string();
        // 400일치 조회 (52주 = 252거래일 + 여유분)
        let start_date = (today - chrono::Duration::days(400))
            .format("%Y%m%d")
            .to_string();

        for symbol in &active_symbols {
            if let Some(adapter) = &toss_adapter {
                let broker_symbol = BrokerSymbol(symbol.trim().to_uppercase());
                match adapter.get_candles(&broker_symbol, "D", "", "").await {
                    Ok(candles) if !candles.is_empty() => {
                        let ohlc = broker_candles_to_ohlc(&candles);
                        apply_ohlc_history(
                            strategy_manager,
                            risk_manager,
                            symbol,
                            ohlc,
                            "Toss candles",
                        )
                        .await;
                    }
                    Ok(_) => tracing::debug!(
                        "Toss 차트 데이터 없음 (히스토리 초기화 건너뜀): {}",
                        symbol
                    ),
                    Err(e) => tracing::warn!(
                        "Toss 차트 데이터 조회 실패 (히스토리 초기화 건너뜀): {} — {}",
                        symbol,
                        e
                    ),
                }
                match adapter.get_candles(&broker_symbol, "1m", "", "").await {
                    Ok(candles) if !candles.is_empty() => {
                        let ohlc = broker_candles_to_ohlc(&candles);
                        apply_intraday_ohlc(strategy_manager, symbol, ohlc, "Toss 1m candles")
                            .await;
                    }
                    Ok(_) => tracing::debug!(
                        "Toss 1분봉 데이터 없음 (장중 반동 초기화 건너뜀): {}",
                        symbol
                    ),
                    Err(e) => tracing::warn!(
                        "Toss 1분봉 조회 실패 (장중 반동 초기화 건너뜀): {} — {}",
                        symbol,
                        e
                    ),
                }
                continue;
            }

            if is_domestic_symbol(symbol) {
                // ── 국내 종목 초기화 ──
                match rest
                    .get_chart_data(symbol, "D", &start_date, &end_date)
                    .await
                {
                    Ok(candles) if !candles.is_empty() => {
                        let ohlc: Vec<OhlcCandle> = candles
                            .iter()
                            .filter_map(|c| {
                                Some(OhlcCandle {
                                    open: c.open.parse::<u64>().ok()?,
                                    high: c.high.parse::<u64>().ok()?,
                                    low: c.low.parse::<u64>().ok()?,
                                    close: c.close.parse::<u64>().ok()?,
                                })
                            })
                            .collect();
                        apply_ohlc_history(
                            strategy_manager,
                            risk_manager,
                            symbol,
                            ohlc,
                            "KIS domestic chart",
                        )
                        .await;
                    }
                    Ok(_) => {
                        tracing::debug!("차트 데이터 없음 (히스토리 초기화 건너뜀): {}", symbol)
                    }
                    Err(e) => tracing::warn!(
                        "차트 데이터 조회 실패 (히스토리 초기화 건너뜀): {} — {}",
                        symbol,
                        e
                    ),
                }
            } else {
                // ── 해외 종목 초기화 (NAS → NYS → AMS 순 시도) ──
                let mut initialized = false;
                for exchange in &["NAS", "NYS", "AMS"] {
                    match rest
                        .get_overseas_chart_data(symbol, exchange, "D", &end_date)
                        .await
                    {
                        Ok(candles) if !candles.is_empty() => {
                            let ohlc: Vec<OhlcCandle> = candles
                                .iter()
                                .filter_map(|c| {
                                    Some(OhlcCandle {
                                        open: c
                                            .open
                                            .parse::<f64>()
                                            .ok()
                                            .map(|v| (v * 100.0).round() as u64)?,
                                        high: c
                                            .high
                                            .parse::<f64>()
                                            .ok()
                                            .map(|v| (v * 100.0).round() as u64)?,
                                        low: c
                                            .low
                                            .parse::<f64>()
                                            .ok()
                                            .map(|v| (v * 100.0).round() as u64)?,
                                        close: c
                                            .close
                                            .parse::<f64>()
                                            .ok()
                                            .map(|v| (v * 100.0).round() as u64)?,
                                    })
                                })
                                .filter(|c| c.open > 0 && c.high > 0 && c.low > 0 && c.close > 0)
                                .collect();
                            apply_ohlc_history(
                                strategy_manager,
                                risk_manager,
                                symbol,
                                ohlc,
                                &format!("KIS overseas chart {exchange}"),
                            )
                            .await;
                            initialized = true;
                            break;
                        }
                        Ok(_) => continue,
                        Err(_) => continue,
                    }
                }
                if !initialized {
                    tracing::warn!(
                        "해외 종목 히스토리 초기화 실패: {} (NAS/NYS/AMS 모두 실패, 실시간 틱 누적 모드로 시작)",
                        symbol
                    );
                }
            }
        }
    }
}

fn calculate_atr(candles: &[OhlcCandle], period: usize) -> Option<u64> {
    if candles.len() < 2 || period == 0 {
        return None;
    }
    let start = candles.len().saturating_sub(period).max(1);
    let mut ranges = Vec::with_capacity(candles.len().saturating_sub(start));
    for idx in start..candles.len() {
        let candle = candles[idx];
        let prev_close = candles[idx - 1].close;
        let high_low = candle.high.saturating_sub(candle.low);
        let high_prev = candle.high.abs_diff(prev_close);
        let low_prev = candle.low.abs_diff(prev_close);
        ranges.push(high_low.max(high_prev).max(low_prev));
    }
    if ranges.is_empty() {
        return None;
    }
    Some((ranges.iter().sum::<u64>() as f64 / ranges.len() as f64).round() as u64)
        .filter(|atr| *atr > 0)
}
