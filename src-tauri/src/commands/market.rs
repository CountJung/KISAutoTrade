use super::*;

// ────────────────────────────────────────────────────────────────────
// 차트 데이터 조회
// ────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct ChartDataInput {
    pub symbol: String,
    /// "D"=일봉, "W"=주봉, "M"=월봉
    pub period_code: String,
    pub start_date: String, // YYYYMMDD
    pub end_date: String,   // YYYYMMDD
}

#[tauri::command]
pub async fn get_chart_data(
    input: ChartDataInput,
    state: State<'_, AppState>,
) -> CmdResult<Vec<ChartCandle>> {
    let client = state.rest_client.read().await.clone();
    client
        .get_chart_data(
            &input.symbol,
            &input.period_code,
            &input.start_date,
            &input.end_date,
        )
        .await
        .map_err(CmdError::from)
}

// ────────────────────────────────────────────────────────────────────
// 현재가 조회
// ────────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_price(symbol: String, state: State<'_, AppState>) -> CmdResult<PriceResponse> {
    let client = state.rest_client.read().await.clone();
    let result = client.get_price(&symbol).await.map_err(CmdError::from)?;
    // 현재가 응답에서 종목명 자동 수집
    if !result.hts_kor_isnm.is_empty() {
        state
            .stock_store
            .upsert(&symbol, &result.hts_kor_isnm)
            .await;
    }
    Ok(result)
}

// ── 종목 검색 ─────────────────────────────────────────────────────────
#[tauri::command]
pub async fn search_stock(
    query: String,
    state: State<'_, AppState>,
) -> CmdResult<Vec<StockSearchItem>> {
    if query.len() < 2 {
        return Ok(vec![]);
    }

    // ① 6자리 영숫자 코드 입력 → KIS 현재가에서 이름 확인 (0005A0 등 ETF 코드 포함)
    if query.len() == 6 && query.chars().all(|c| c.is_ascii_alphanumeric()) {
        let code = query.to_uppercase();
        // StockStore에 이미 있으면 빠르게 반환
        if let Some(name) = state.stock_store.get_name(&code).await {
            return Ok(vec![StockSearchItem {
                pdno: code,
                prdt_name: name,
                market: None,
            }]);
        }
        // 없으면 KIS get_price로 확인
        let client = state.rest_client.read().await.clone();
        if let Ok(p) = client.get_price(&code).await {
            if !p.hts_kor_isnm.is_empty() {
                state.stock_store.upsert(&code, &p.hts_kor_isnm).await;
                return Ok(vec![StockSearchItem {
                    pdno: code,
                    prdt_name: p.hts_kor_isnm,
                    market: None,
                }]);
            }
        }
        // KIS 실패 시 Yahoo Finance로 이름 조회 (설정 없이도 동작)
        tracing::debug!("KIS 현재가 실패 → Yahoo Finance로 종목명 조회: {}", code);
        match crate::market::lookup_name_by_code(&code).await {
            Ok(name) => {
                tracing::info!("Yahoo Finance 이름 조회 성공: {} → {}", code, name);
                state.stock_store.upsert(&code, &name).await;
                return Ok(vec![StockSearchItem {
                    pdno: code,
                    prdt_name: name,
                    market: None,
                }]);
            }
            Err(e) => {
                tracing::warn!("Yahoo Finance 이름 조회 실패: {} — {}", code, e);
                return Ok(vec![]);
            }
        }
    }

    // ② StockStore(영구 캐시) 검색 — 우선순위 최상
    let local_results = state.stock_store.search(&query, 20).await;
    if !local_results.is_empty() {
        tracing::debug!(
            "StockStore 검색: query={:?}, {}개 결과",
            query,
            local_results.len()
        );
        return Ok(local_results);
    }

    // ③ KRX 레거시 캐시 검색 (stock_list — KRX 다운로드 성공 시에만 유효)
    {
        let stock_list = state.stock_list.read().await;
        if !stock_list.is_empty() {
            let results = crate::market::search_local(&stock_list, &query, 20);
            if !results.is_empty() {
                tracing::debug!("KRX 캐시 검색: query={:?}, {}개 결과", query, results.len());
                return Ok(results);
            }
        }
    }

    // ④ KRX 프록시 검색 (k-skill-proxy — 공식 KRX 데이터, API 키 불필요, 시장구분 포함)
    tracing::info!(
        "search_stock: 로컬 캐시 miss → KRX 프록시 검색 (query={:?})",
        query
    );
    match crate::market::search_krx_proxy(&query, 20).await {
        Ok(results) if !results.is_empty() => {
            tracing::info!(
                "KRX 프록시 검색 성공: {}개 결과 (query={:?})",
                results.len(),
                query
            );
            // 결과를 StockStore에 캐시
            state
                .stock_store
                .upsert_many(
                    results
                        .iter()
                        .map(|r| (r.pdno.clone(), r.prdt_name.clone())),
                )
                .await;
            return Ok(results);
        }
        Ok(_) => tracing::debug!("KRX 프록시 결과 없음 (query={:?}), NAVER 폴백 시도", query),
        Err(e) => tracing::warn!(
            "KRX 프록시 검색 실패: {} (query={:?}), NAVER 폴백 시도",
            e,
            query
        ),
    }

    // ⑤ NAVER Finance 실시간 검색 폴백 (최후 수단)
    tracing::info!(
        "search_stock: KRX 프록시 miss → NAVER 실시간 검색 (query={:?})",
        query
    );
    match crate::market::search_naver_live(&query).await {
        Ok(results) if !results.is_empty() => {
            tracing::info!(
                "NAVER 검색 성공: {}개 결과 (query={:?})",
                results.len(),
                query
            );
            // NAVER 결과도 StockStore에 캐시
            state
                .stock_store
                .upsert_many(
                    results
                        .iter()
                        .map(|r| (r.pdno.clone(), r.prdt_name.clone())),
                )
                .await;
            Ok(results)
        }
        Ok(_) => {
            tracing::debug!("NAVER 검색 결과 없음 (query={:?})", query);
            Ok(vec![])
        }
        Err(e) => {
            tracing::warn!("NAVER 검색 실패: {} (query={:?})", e, query);
            Err(CmdError {
                code: "STOCK_LIST_EMPTY".into(),
                message: "종목 검색에 실패했습니다. 네트워크 연결을 확인하거나 '종목 목록 새로고침'을 눌러주세요.".into(),
            })
        }
    }
}

// ── 종목 목록 새로고침 ─────────────────────────────────────────────
#[tauri::command]
pub async fn refresh_stock_list(state: State<'_, AppState>) -> CmdResult<usize> {
    tracing::info!("수동 종목 목록 새로고침 시작 (KRX 다운로드 시도)...");
    let items = crate::market::StockList::fetch_from_krx()
        .await
        .map_err(CmdError::from)?;

    if items.is_empty() {
        tracing::warn!(
            "KRX 다운로드 결과가 0개입니다. \
             KRX 데이터 포털(data.krx.co.kr)이 봇 차단(WAF)을 적용 중이거나 \
             네트워크 문제일 수 있습니다. \
             종목 검색은 NAVER Finance 실시간 검색으로 자동 대체됩니다."
        );
        return Err(CmdError {
            code: "KRX_EMPTY".into(),
            message: "KRX에서 종목 목록을 가져오지 못했습니다 (0개). 종목 검색은 실시간 검색으로 동작합니다.".into(),
        });
    }

    let count = items.len();

    // 메모리 갱신
    *state.stock_list.write().await = items.clone();

    // 캐시 파일 갱신
    let cache_path = state.data_dir.join("stock_list.json");
    crate::storage::write_json(&cache_path, &items)
        .await
        .map_err(|error| CmdError {
            code: "PERSISTENCE_ERROR".into(),
            message: format!("종목 목록 캐시 저장 실패: {error}"),
        })?;

    tracing::info!("종목 목록 수동 갱신 완료: {}개", count);
    Ok(count)
}

// ── 종목 목록 통계 조회 ────────────────────────────────────────────
#[tauri::command]
pub async fn get_stock_list_stats(state: State<'_, AppState>) -> CmdResult<StockListStats> {
    let count = state.stock_store.size().await;
    let last_updated_at = state.stock_store.last_updated_at().await;
    let update_interval_hours = state.stock_store.get_interval_hours().await;
    let file_path = state
        .data_dir
        .join("stocklist")
        .join("stocklist.json")
        .to_string_lossy()
        .to_string();
    Ok(StockListStats {
        count,
        last_updated_at,
        file_path,
        update_interval_hours,
    })
}

// ── 종목 목록 자동 갱신 간격 설정 ────────────────────────────────
#[tauri::command]
pub async fn set_stock_update_interval(hours: u32, state: State<'_, AppState>) -> CmdResult<()> {
    state
        .stock_store
        .set_interval_hours(hours)
        .await
        .map_err(CmdError::from)?;
    tracing::info!("종목 목록 갱신 간격 변경: {}시간", hours);
    Ok(())
}

// ────────────────────────────────────────────────────────────────────
// 해외(미국) 주식 현재가 조회
// ────────────────────────────────────────────────────────────────────

/// 해외 현재가 뷰 (camelCase → TypeScript 1:1)
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OverseasPriceView {
    pub symbol: String,
    pub exchange: String,
    pub name: String,
    pub last: String,
    pub diff: String,
    pub rate: String,
    pub open: String,
    pub high: String,
    pub low: String,
    pub h52p: String,
    pub l52p: String,
    pub tvol: String,
}

/// 해외 주문 입력 (TypeScript PlaceOverseasOrderInput 1:1)
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OverseasOrderInput {
    pub symbol: String,
    pub exchange: String, // NASD / NYSE / AMEX
    pub side: String,
    pub price: f64,
    pub quantity: u64,
}

#[tauri::command]
pub async fn get_overseas_chart_data(
    symbol: String,
    exchange: String,
    period_code: String, // "D", "W", "M"
    base_date: String,   // YYYYMMDD — 비워두면 당일 기준
    state: State<'_, AppState>,
) -> CmdResult<Vec<ChartCandle>> {
    let client = state.rest_client.read().await.clone();
    client
        .get_overseas_chart_data(&symbol, &exchange, &period_code, &base_date)
        .await
        .map_err(CmdError::from)
}

#[tauri::command]
pub async fn get_overseas_price(
    symbol: String,
    exchange: String,
    state: State<'_, AppState>,
) -> CmdResult<OverseasPriceView> {
    let client = state.rest_client.read().await.clone();
    let resp = client
        .get_overseas_price(&symbol, &exchange)
        .await
        .map_err(CmdError::from)?;

    Ok(OverseasPriceView {
        symbol,
        exchange,
        name: resp.name,
        last: resp.last,
        diff: resp.diff,
        rate: resp.rate,
        open: resp.open,
        high: resp.high,
        low: resp.low,
        h52p: resp.h52p,
        l52p: resp.l52p,
        tvol: resp.tvol,
    })
}

#[tauri::command]
pub async fn place_overseas_order(
    input: OverseasOrderInput,
    state: State<'_, AppState>,
) -> CmdResult<OrderResponse> {
    use crate::api::rest::{OrderSide, OrderType};

    tracing::info!(
        "해외 주문 요청: {} {} {} 수량={} 가격={}",
        input.exchange,
        input.symbol,
        input.side,
        input.quantity,
        input.price
    );

    let side = match input.side.trim().to_ascii_lowercase().as_str() {
        "buy" => OrderSide::Buy,
        "sell" => OrderSide::Sell,
        other => {
            return Err(CmdError {
                code: "INVALID_SIDE".into(),
                message: format!("알 수 없는 주문 방향: {other}"),
            })
        }
    };

    let client = state.rest_client.read().await.clone();
    let quote = client
        .get_overseas_price(&input.symbol, &input.exchange)
        .await
        .map_err(CmdError::from)?;
    let quote_price = usd_to_cents(&quote.last);
    if quote_price == 0 {
        return Err(CmdError {
            code: "INVALID_QUOTE".into(),
            message: "해외 현재가 응답을 숫자로 해석할 수 없습니다.".into(),
        });
    }
    let exchange_rate = *state.exchange_rate_krw.read().await;
    let total_balance =
        super::trading::fetch_account_risk_balance_krw(&client, true, exchange_rate)
            .await
            .map_err(|message| CmdError {
                code: "ACCOUNT_SYNC_FAILED".into(),
                message,
            })?;
    let profile = state.profiles.read().await.get_active().cloned();
    let config_account_id = state.config.read().await.broker_account_id.clone();
    let account_id = profile
        .as_ref()
        .map(|value| value.broker_account_id())
        .or_else(|| (!config_account_id.is_empty()).then_some(config_account_id));
    let scope = BrokerScope::new(BrokerId::Kis, account_id.map(BrokerAccountId));
    let symbol_name = if quote.name.trim().is_empty() {
        input.symbol.clone()
    } else {
        quote.name
    };
    let outcome = OrderManager::submit_manual_order_shared(
        &state.order_manager,
        input.symbol,
        symbol_name,
        side,
        OrderType::Limit,
        input.quantity,
        (input.price.max(0.0) * 100.0).round() as u64,
        quote_price,
        total_balance,
        Some(input.exchange),
        scope,
    )
    .await
    .map_err(CmdError::from)?;
    super::orders::manual_submission_response(outcome, "KIS")
}
