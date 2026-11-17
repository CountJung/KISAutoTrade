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
            place_overseas_with_retry_shared(&deps.rest_client, req, !prepared.submission.is_manual)
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
    enforce_session: bool,
) -> Result<OrderResponse> {
    retry_overseas_request(
        enforce_session,
        || rest_client.automatic_session_now(),
        || async {
            if enforce_session {
                rest_client.place_automatic_overseas_order(req).await
            } else {
                rest_client.place_overseas_order(req).await
            }
        },
    )
    .await
}

async fn retry_overseas_request<F, Fut>(
    enforce_session: bool,
    mut now: impl FnMut() -> chrono::DateTime<chrono::Utc>,
    mut send: F,
) -> Result<OrderResponse>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<OrderResponse>>,
{
    const MAX_RETRIES: u32 = 3;
    let mut last_err = anyhow::anyhow!("해외 주문 최대 재시도 횟수(3회) 초과");
    let mut prior_attempts_definitively_rejected = true;
    for attempt in 0..MAX_RETRIES {
        if let Err(closed) =
            crate::market_hours::ensure_kis_automatic_us_session(enforce_session, now())
        {
            // Never relabel a previous ambiguous provider attempt as unsent.
            return Err(if prior_attempts_definitively_rejected {
                closed.into()
            } else {
                last_err
            });
        }
        match send().await {
            Ok(resp) => return Ok(resp),
            Err(e) => {
                if e.downcast_ref::<crate::market_hours::UsSessionClosed>()
                    .is_some()
                    && !prior_attempts_definitively_rejected
                {
                    return Err(last_err);
                }
                let msg = e.to_string();
                let is_rate_limit = msg.contains("EGW00133") || msg.contains("EGW00201");
                if is_rate_limit && attempt < MAX_RETRIES - 1 {
                    let this_attempt_definitive = e
                        .downcast_ref::<crate::api::rest::KisOrderRejected>()
                        .is_some();
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
                    // Keep the first ambiguous error; a later rejection cannot
                    // prove that the earlier request was never accepted.
                    if prior_attempts_definitively_rejected {
                        last_err = e;
                    }
                    prior_attempts_definitively_rejected &= this_attempt_definitive;
                } else {
                    return Err(if prior_attempts_definitively_rejected {
                        e
                    } else {
                        last_err
                    });
                }
            }
        }
    }
    Err(last_err)
}

#[cfg(test)]
mod session_tests {
    use super::*;
    use std::cell::Cell;

    fn at(s: &str) -> chrono::DateTime<chrono::Utc> {
        chrono::DateTime::parse_from_rfc3339(s).unwrap().to_utc()
    }

    #[tokio::test]
    async fn closed_automatic_us_attempt_never_calls_provider_but_manual_policy_is_unchanged() {
        let calls = Cell::new(0);
        let error = retry_overseas_request(
            true,
            || at("2026-11-26T15:00:00Z"),
            || {
                calls.set(calls.get() + 1);
                async { Err(anyhow::anyhow!("synthetic provider")) }
            },
        )
        .await
        .unwrap_err();
        assert_eq!(calls.get(), 0);
        assert!(error
            .downcast_ref::<crate::market_hours::UsSessionClosed>()
            .is_some());
        let error = retry_overseas_request(
            false,
            || at("2026-11-26T15:00:00Z"),
            || {
                calls.set(calls.get() + 1);
                async { Err(anyhow::anyhow!("synthetic provider")) }
            },
        )
        .await
        .unwrap_err();
        assert_eq!(calls.get(), 1);
        assert!(error
            .downcast_ref::<crate::market_hours::UsSessionClosed>()
            .is_none());
    }

    #[tokio::test(start_paused = true)]
    async fn retry_crossing_close_preserves_ambiguous_prior_attempt() {
        for definitive in [false, true] {
            let times = Cell::new(0);
            let calls = Cell::new(0);
            let error = retry_overseas_request(
                true,
                || {
                    let index = times.get();
                    times.set(index + 1);
                    at(if index == 0 {
                        "2026-11-09T20:59:59Z"
                    } else {
                        "2026-11-09T21:00:00Z"
                    })
                },
                || {
                    calls.set(calls.get() + 1);
                    async move {
                        Err(if definitive {
                            crate::api::rest::KisOrderRejected(
                                "EGW00201 synthetic definite rejection".into(),
                            )
                            .into()
                        } else {
                            anyhow::anyhow!("EGW00201 ambiguous synthetic transport")
                        })
                    }
                },
            )
            .await
            .unwrap_err();
            assert_eq!(calls.get(), 1);
            assert_eq!(
                error
                    .downcast_ref::<crate::market_hours::UsSessionClosed>()
                    .is_some(),
                definitive
            );
            if !definitive {
                assert!(error.to_string().contains("ambiguous"));
            }
        }
    }
    #[tokio::test]
    async fn local_session_rejection_releases_only_its_budget_and_uncertain_errors_hold() {
        use crate::trading::order::test_support::{self, Fixture};
        let f = Fixture::new("http://127.0.0.1:1".into()).await;
        let scope = test_support::scope(&f.profile);
        let mut manager = f.manager.lock().await;
        manager.set_execution_scope(scope.clone());
        manager
            .update_auto_trading_budget(&scope, 10_000, 10_000)
            .await
            .unwrap();
        let first = manager
            .reserve_budget(&scope, "AAPL", true, true, 10, 100)
            .await
            .unwrap();
        let before = manager
            .auto_trading_budget(&scope)
            .await
            .unwrap()
            .usd
            .reserved_amount;
        release_definitively_rejected_budget(
            &mut manager,
            Some(&first),
            &anyhow::anyhow!("EGW00201 ambiguous transport"),
        )
        .await
        .unwrap();
        assert_eq!(
            manager
                .auto_trading_budget(&scope)
                .await
                .unwrap()
                .usd
                .reserved_amount,
            before
        );
        release_definitively_rejected_budget(
            &mut manager,
            Some("unrelated-reservation"),
            &crate::market_hours::UsSessionClosed.into(),
        )
        .await
        .unwrap();
        assert_eq!(
            manager
                .auto_trading_budget(&scope)
                .await
                .unwrap()
                .usd
                .reserved_amount,
            before
        );
        release_definitively_rejected_budget(
            &mut manager,
            Some(&first),
            &crate::market_hours::UsSessionClosed.into(),
        )
        .await
        .unwrap();
        let after = manager
            .auto_trading_budget(&scope)
            .await
            .unwrap()
            .usd
            .reserved_amount;
        assert_eq!(after, 0);
    }
    #[tokio::test(start_paused = true)]
    async fn transport_gate_after_ambiguous_attempt_does_not_release_budget() {
        let calls = Cell::new(0);
        let error = retry_overseas_request(
            true,
            || at("2026-11-09T20:59:59Z"),
            || {
                let index = calls.get();
                calls.set(index + 1);
                async move {
                    if index == 0 {
                        Err(anyhow::anyhow!("EGW00201 ambiguous synthetic transport"))
                    } else {
                        Err(crate::market_hours::UsSessionClosed.into())
                    }
                }
            },
        )
        .await
        .unwrap_err();
        assert_eq!(calls.get(), 2);
        assert!(error.to_string().contains("ambiguous"));
        assert!(error
            .downcast_ref::<crate::market_hours::UsSessionClosed>()
            .is_none());
    }
    #[tokio::test(start_paused = true)]
    async fn ambiguous_first_attempt_survives_later_definite_rejections_and_close() {
        for (enforce_session, close_after_second) in [(true, false), (true, true), (false, false)] {
            let calls = Cell::new(0);
            let error = retry_overseas_request(
                enforce_session,
                || {
                    at(if close_after_second && calls.get() == 2 {
                        "2026-11-09T21:00:00Z"
                    } else {
                        "2026-11-09T20:59:59Z"
                    })
                },
                || {
                    let index = calls.get();
                    calls.set(index + 1);
                    async move {
                        if index == 0 {
                            Err(anyhow::anyhow!("EGW00201 first ambiguous attempt"))
                        } else {
                            Err(crate::api::rest::KisOrderRejected(
                                "EGW00201 later rejected attempt".into(),
                            )
                            .into())
                        }
                    }
                },
            )
            .await
            .unwrap_err();
            assert_eq!(calls.get(), if close_after_second { 2 } else { 3 });
            assert!(error.to_string().contains("first ambiguous"));
            assert!(error
                .downcast_ref::<crate::api::rest::KisOrderRejected>()
                .is_none());
            assert!(error
                .downcast_ref::<crate::market_hours::UsSessionClosed>()
                .is_none());
        }
    }
}
