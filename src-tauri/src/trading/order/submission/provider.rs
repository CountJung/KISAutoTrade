//! 준비된 주문 전송과 기존 KIS 재시도.

use super::*;

pub(super) async fn place_prepared_order(
    deps: &SubmissionDeps,
    prepared: &PreparedOrderSubmission,
) -> Result<ProviderOrderResponse> {
    match &prepared.provider_request {
        ProviderOrderRequest::Domestic(req) => {
            tracing::info!(
                "{} 주문 시도: {} {}주 — {}",
                order_side_label(&prepared.submission.side),
                prepared.submission.symbol,
                prepared.quantity,
                prepared.submission.reason
            );
            place_with_retry_shared(&deps.rest_client, req)
                .await
                .map(ProviderOrderResponse::Kis)
        }
        ProviderOrderRequest::Overseas(req) => {
            tracing::info!(
                "해외 {} 주문 시도: {} {}주 @ ${:.2} ({}) — {}",
                order_side_label(&prepared.submission.side),
                prepared.submission.symbol,
                prepared.quantity,
                prepared.submission.tick_price as f64 / 100.0,
                req.exchange,
                prepared.submission.reason
            );
            place_overseas_with_retry_shared(&deps.rest_client, req)
                .await
                .map(ProviderOrderResponse::Kis)
        }
        ProviderOrderRequest::Toss(req) => {
            let profile = deps.active_profile.as_ref().ok_or_else(|| {
                anyhow::anyhow!("활성 Toss 프로파일이 없어 주문을 제출할 수 없습니다.")
            })?;
            if profile.broker_id != BrokerId::Toss {
                return Err(anyhow::anyhow!(
                    "자동매매 실행 scope는 Toss지만 활성 프로파일은 Toss가 아닙니다."
                ));
            }
            if !profile.live_trading_consent {
                return Err(anyhow::anyhow!(
                    "Toss 실거래 동의가 저장되지 않아 자동매매 주문을 차단했습니다."
                ));
            }
            let account_seq = profile.broker_account_id();
            if account_seq.trim().is_empty() {
                return Err(anyhow::anyhow!("Toss accountSeq가 설정되지 않았습니다."));
            }
            tracing::info!(
                "Toss {} 주문 시도: {} {}주 {} — {}",
                order_side_label(&prepared.submission.side),
                prepared.submission.symbol,
                prepared.quantity,
                req.order_type,
                prepared.submission.reason
            );
            let adapter = TossBrokerAdapter::with_credentials(
                TossBrokerAdapter::DEFAULT_BASE_URL,
                profile.app_key.clone(),
                profile.app_secret.clone(),
                Some(account_seq.clone()),
            );
            let response = adapter
                .create_order(Some(&account_seq), req)
                .await
                .map_err(|e| match e {
                    crate::broker::BrokerAdapterError::Provider(error) => error,
                    other => anyhow::anyhow!(other),
                })?;
            Ok(ProviderOrderResponse::Toss {
                order_id: response.order_id,
                client_order_id: response
                    .client_order_id
                    .or_else(|| req.client_order_id.clone()),
            })
        }
    }
}

async fn place_with_retry_shared(
    rest_client: &Arc<KisRestClient>,
    req: &OrderRequest,
) -> Result<OrderResponse> {
    const MAX_RETRIES: u32 = 3;
    let mut last_err = anyhow::anyhow!("주문 최대 재시도 횟수(3회) 초과");

    for attempt in 0..MAX_RETRIES {
        match rest_client.place_order(req).await {
            Ok(resp) => return Ok(resp),
            Err(e) => {
                let msg = e.to_string();
                let is_rate_limit = msg.contains("EGW00133") || msg.contains("EGW00201");
                if is_rate_limit && attempt < MAX_RETRIES - 1 {
                    tracing::warn!(
                        "KIS rate-limit ({}) — 2초 후 재시도 ({}/{})",
                        if msg.contains("EGW00201") {
                            "EGW00201"
                        } else {
                            "EGW00133"
                        },
                        attempt + 1,
                        MAX_RETRIES
                    );
                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                    last_err = e;
                } else {
                    return Err(e);
                }
            }
        }
    }
    Err(last_err)
}

async fn place_overseas_with_retry_shared(
    rest_client: &Arc<KisRestClient>,
    req: &OverseasOrderRequest,
) -> Result<OrderResponse> {
    const MAX_RETRIES: u32 = 3;
    let mut last_err = anyhow::anyhow!("해외 주문 최대 재시도 횟수(3회) 초과");

    for attempt in 0..MAX_RETRIES {
        match rest_client.place_overseas_order(req).await {
            Ok(resp) => return Ok(resp),
            Err(e) => {
                let msg = e.to_string();
                let is_rate_limit = msg.contains("EGW00133") || msg.contains("EGW00201");
                if is_rate_limit && attempt < MAX_RETRIES - 1 {
                    tracing::warn!(
                        "KIS rate-limit (해외, {}) — 2초 후 재시도 ({}/{})",
                        if msg.contains("EGW00201") {
                            "EGW00201"
                        } else {
                            "EGW00133"
                        },
                        attempt + 1,
                        MAX_RETRIES
                    );
                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                    last_err = e;
                } else {
                    return Err(e);
                }
            }
        }
    }
    Err(last_err)
}
