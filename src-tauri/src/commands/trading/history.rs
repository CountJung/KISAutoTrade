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

fn broker_candles_to_close_prices(candles: &[BrokerCandle]) -> Vec<u64> {
    let mut candles = candles.to_vec();
    candles.sort_by(|a, b| a.date.cmp(&b.date));
    candles
        .iter()
        .filter_map(|c| broker_money_to_strategy_units(&c.close))
        .filter(|price| *price > 0)
        .collect()
}

async fn apply_ohlc_history(state: &AppState, symbol: &str, ohlc: Vec<OhlcCandle>, source: &str) {
    if ohlc.is_empty() {
        return;
    }

    let highs: Vec<u64> = ohlc.iter().map(|c| c.high).filter(|v| *v > 0).collect();
    if !highs.is_empty() {
        state
            .strategy_manager
            .lock()
            .await
            .initialize_historical(symbol, &highs);
        tracing::info!(
            "전략 히스토리 초기화 완료: {} @ {} ({}봉)",
            symbol,
            source,
            highs.len()
        );
    }

    let high_close: Vec<(u64, u64)> = ohlc.iter().map(|c| (c.high, c.close)).collect();
    if !high_close.is_empty() {
        state
            .strategy_manager
            .lock()
            .await
            .initialize_candles(symbol, &high_close);
    }

    state
        .strategy_manager
        .lock()
        .await
        .initialize_ohlc(symbol, &ohlc);
    if let Some(atr) = calculate_atr(&ohlc, 14) {
        state.risk_manager.lock().await.set_symbol_atr(symbol, atr);
        tracing::info!(
            "리스크 ATR 초기화 완료: {} @ {} ATR14={}",
            symbol,
            source,
            atr
        );
    }

    let ranges: Vec<u64> = ohlc
        .iter()
        .map(|c| c.high.saturating_sub(c.low))
        .filter(|v| *v > 0)
        .collect();
    if !ranges.is_empty() {
        state
            .strategy_manager
            .lock()
            .await
            .initialize_range_data(symbol, &ranges);
    }
}

async fn apply_intraday_prices(state: &AppState, symbol: &str, prices: Vec<u64>, source: &str) {
    if prices.is_empty() {
        return;
    }
    state
        .strategy_manager
        .lock()
        .await
        .initialize_intraday_prices(symbol, &prices);
    tracing::info!(
        "전략 장중 가격 초기화 완료: {} @ {} ({}개)",
        symbol,
        source,
        prices.len()
    );
}

pub(super) async fn initialize_active_strategy_history(state: &AppState) {
    // 활성 전략의 종목별 일봉 차트 데이터 로드 → 히스토리 기반 전략 초기화 (52주 신고가 등)
    // 국내 종목: get_chart_data (KRW 정수 가격)
    // 해외 종목: get_overseas_chart_data (USD float → ×100 센트로 정수화)
    let active_symbols: Vec<String> = state.strategy_manager.lock().await.active_symbols();
    if !active_symbols.is_empty() {
        let execution_scope = state.order_manager.lock().await.execution_scope().clone();
        let toss_adapter = if execution_scope.broker_id == BrokerId::Toss {
            resolve_scoped_profile(&state.profiles, &execution_scope)
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
        let rest = state.rest_client.read().await.clone();
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
                        apply_ohlc_history(state, symbol, ohlc, "Toss candles").await;
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
                        let prices = broker_candles_to_close_prices(&candles);
                        apply_intraday_prices(state, symbol, prices, "Toss 1m candles").await;
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
                        apply_ohlc_history(state, symbol, ohlc, "KIS domestic chart").await;
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
                                state,
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
