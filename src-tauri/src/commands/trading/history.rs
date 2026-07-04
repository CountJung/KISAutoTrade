use super::*;

pub(super) async fn initialize_active_strategy_history(state: &AppState) {
    // 활성 전략의 종목별 일봉 차트 데이터 로드 → 히스토리 기반 전략 초기화 (52주 신고가 등)
    // 국내 종목: get_chart_data (KRW 정수 가격)
    // 해외 종목: get_overseas_chart_data (USD float → ×100 센트로 정수화)
    let active_symbols: Vec<String> = state.strategy_manager.lock().await.active_symbols();
    if !active_symbols.is_empty() {
        let rest = state.rest_client.read().await.clone();
        let today = chrono::Local::now();
        let end_date = today.format("%Y%m%d").to_string();
        // 400일치 조회 (52주 = 252거래일 + 여유분)
        let start_date = (today - chrono::Duration::days(400))
            .format("%Y%m%d")
            .to_string();

        for symbol in &active_symbols {
            if is_domestic_symbol(symbol) {
                // ── 국내 종목 초기화 ──
                match rest
                    .get_chart_data(symbol, "D", &start_date, &end_date)
                    .await
                {
                    Ok(candles) if !candles.is_empty() => {
                        let highs: Vec<u64> = candles
                            .iter()
                            .filter_map(|c| c.high.parse::<u64>().ok())
                            .collect();
                        if !highs.is_empty() {
                            state
                                .strategy_manager
                                .lock()
                                .await
                                .initialize_historical(symbol, &highs);
                            tracing::info!(
                                "전략 히스토리 초기화 완료: {} ({}봉)",
                                symbol,
                                highs.len()
                            );
                        }
                        let high_close: Vec<(u64, u64)> = candles
                            .iter()
                            .filter_map(|c| {
                                let h = c.high.parse::<u64>().ok()?;
                                let cl = c.close.parse::<u64>().ok()?;
                                Some((h, cl))
                            })
                            .collect();
                        if !high_close.is_empty() {
                            state
                                .strategy_manager
                                .lock()
                                .await
                                .initialize_candles(symbol, &high_close);
                        }
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
                        if !ohlc.is_empty() {
                            state
                                .strategy_manager
                                .lock()
                                .await
                                .initialize_ohlc(symbol, &ohlc);
                            if let Some(atr) = calculate_atr(&ohlc, 14) {
                                state.risk_manager.lock().await.set_symbol_atr(symbol, atr);
                                tracing::info!("리스크 ATR 초기화 완료: {} ATR14={}", symbol, atr);
                            }
                        }
                        let ranges: Vec<u64> = candles
                            .iter()
                            .filter_map(|c| {
                                let h = c.high.parse::<u64>().ok()?;
                                let l = c.low.parse::<u64>().ok()?;
                                Some(h.saturating_sub(l))
                            })
                            .collect();
                        if !ranges.is_empty() {
                            state
                                .strategy_manager
                                .lock()
                                .await
                                .initialize_range_data(symbol, &ranges);
                        }
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
                            // USD float 문자열 → ×100 센트(u64)로 변환하여 전략 히스토리 초기화
                            let highs: Vec<u64> = candles
                                .iter()
                                .filter_map(|c| {
                                    c.high
                                        .parse::<f64>()
                                        .ok()
                                        .map(|v| (v * 100.0).round() as u64)
                                })
                                .filter(|&v| v > 0)
                                .collect();
                            if !highs.is_empty() {
                                state
                                    .strategy_manager
                                    .lock()
                                    .await
                                    .initialize_historical(symbol, &highs);
                                tracing::info!(
                                    "해외 전략 히스토리 초기화: {} @ {} ({}봉, 센트 단위)",
                                    symbol,
                                    exchange,
                                    highs.len()
                                );
                            }
                            let high_close: Vec<(u64, u64)> = candles
                                .iter()
                                .filter_map(|c| {
                                    let h = c
                                        .high
                                        .parse::<f64>()
                                        .ok()
                                        .map(|v| (v * 100.0).round() as u64)?;
                                    let cl = c
                                        .close
                                        .parse::<f64>()
                                        .ok()
                                        .map(|v| (v * 100.0).round() as u64)?;
                                    if h > 0 && cl > 0 {
                                        Some((h, cl))
                                    } else {
                                        None
                                    }
                                })
                                .collect();
                            if !high_close.is_empty() {
                                state
                                    .strategy_manager
                                    .lock()
                                    .await
                                    .initialize_candles(symbol, &high_close);
                            }
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
                            if !ohlc.is_empty() {
                                state
                                    .strategy_manager
                                    .lock()
                                    .await
                                    .initialize_ohlc(symbol, &ohlc);
                                if let Some(atr) = calculate_atr(&ohlc, 14) {
                                    state.risk_manager.lock().await.set_symbol_atr(symbol, atr);
                                    tracing::info!(
                                        "해외 리스크 ATR 초기화: {} @ {} ATR14={} cents",
                                        symbol,
                                        exchange,
                                        atr
                                    );
                                }
                            }
                            let ranges: Vec<u64> = candles
                                .iter()
                                .filter_map(|c| {
                                    let h = c.high.parse::<f64>().ok()?;
                                    let l = c.low.parse::<f64>().ok()?;
                                    let diff = ((h - l) * 100.0).round() as u64;
                                    if diff > 0 {
                                        Some(diff)
                                    } else {
                                        None
                                    }
                                })
                                .collect();
                            if !ranges.is_empty() {
                                state
                                    .strategy_manager
                                    .lock()
                                    .await
                                    .initialize_range_data(symbol, &ranges);
                            }
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
