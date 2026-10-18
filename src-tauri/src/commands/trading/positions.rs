use super::*;
use crate::trading::order::{HoldingsMarkets, HoldingsUpdate};

pub(super) async fn sync_strategy_positions_from_active_broker(
    state: &AppState,
    current_cfg: &AppConfig,
) -> CmdResult<usize> {
    let active_profile = {
        let profiles = state.profiles.read().await;
        profiles.get_active().cloned()
    };
    match active_profile
        .as_ref()
        .map(|profile| profile.broker_id)
        .unwrap_or(current_cfg.broker_id)
    {
        BrokerId::Kis => sync_kis_strategy_positions(state).await,
        BrokerId::Toss => match active_profile {
            Some(profile) => sync_toss_strategy_positions(state, profile).await,
            None => Err(CmdError {
                code: "ACCOUNT_SYNC_FAILED".into(),
                message: "활성 Toss 프로파일이 없어 잔고를 동기화할 수 없습니다.".into(),
            }),
        },
    }
}

async fn sync_kis_strategy_positions(state: &AppState) -> CmdResult<usize> {
    let query = state
        .order_manager
        .lock()
        .await
        .begin_holdings_query(BrokerId::Kis, HoldingsMarkets::Both)
        .await
        .map_err(|error| CmdError {
            code: "ACCOUNT_SYNC_FAILED".into(),
            message: error.to_string(),
        })?;
    let rest = query.client();
    let mut synced = 0usize;
    let domestic = rest.get_balance().await.map_err(|e| CmdError {
        code: "ACCOUNT_SYNC_FAILED".into(),
        message: format!("자동매매 시작 전 KIS 국내 잔고 동기화 실패: {e}"),
    })?;
    let overseas = rest.get_overseas_balance().await.map_err(|e| CmdError {
        code: "ACCOUNT_SYNC_FAILED".into(),
        message: format!("자동매매 시작 전 KIS 해외 잔고 동기화 실패: {e}"),
    })?;
    let domestic_positions: Vec<(String, String, u64, u64, u64)> = domestic
        .items
        .iter()
        .map(|item| {
            Ok((
                item.pdno.clone(),
                item.prdt_name.clone(),
                parse_holding_u64(&item.hldg_qty, &item.pdno, "보유수량")?,
                parse_holding_price(&item.pchs_avg_pric, &item.pdno, "평균매입가")?,
                parse_holding_u64(&item.prpr, &item.pdno, "현재가")?,
            ))
        })
        .collect::<CmdResult<_>>()?;
    let overseas_positions: Vec<(String, String, String, u64, u64, u64)> = overseas
        .items
        .iter()
        .map(|item| {
            Ok((
                item.ovrs_pdno.clone(),
                item.ovrs_item_name.clone(),
                normalize_overseas_order_exchange(&item.ovrs_excg_cd),
                parse_holding_u64(&item.ovrs_cblc_qty, &item.ovrs_pdno, "해외 보유수량")?,
                parse_holding_usd_cents(&item.pchs_avg_pric, &item.ovrs_pdno, "해외 평균매입가")?,
                parse_holding_usd_cents(&item.now_pric2, &item.ovrs_pdno, "해외 현재가")?,
            ))
        })
        .collect::<CmdResult<_>>()?;

    state
        .stock_store
        .upsert_many(
            domestic
                .items
                .iter()
                .map(|item| (item.pdno.clone(), item.prdt_name.clone())),
        )
        .await;
    let mut mgr = state.strategy_manager.lock().await;
    let mut orders = state.order_manager.lock().await;
    orders
        .publish_holdings(
            query,
            HoldingsUpdate::Both {
                domestic: domestic_positions.clone(),
                overseas: overseas_positions.clone(),
            },
        )
        .await
        .map_err(|error| CmdError {
            code: "ACCOUNT_SYNC_FAILED".into(),
            message: error.to_string(),
        })?;
    for symbol in mgr.active_symbols() {
        if is_domestic_symbol(&symbol) {
            mgr.sync_position_for_broker(&BrokerPositionSnapshot {
                broker_id: BrokerId::Kis,
                market: BrokerMarket::Kr,
                symbol,
                quantity: 0,
                avg_price: 0,
            });
        }
    }
    for (symbol, _, qty, avg, _) in &domestic_positions {
        if *qty > 0 {
            synced += 1;
        }
        mgr.sync_position_for_broker(&BrokerPositionSnapshot {
            broker_id: BrokerId::Kis,
            market: BrokerMarket::Kr,
            symbol: symbol.clone(),
            quantity: *qty,
            avg_price: *avg,
        });
    }
    for symbol in mgr.active_symbols() {
        if !is_domestic_symbol(&symbol) {
            mgr.sync_position_for_broker(&BrokerPositionSnapshot {
                broker_id: BrokerId::Kis,
                market: BrokerMarket::Us,
                symbol,
                quantity: 0,
                avg_price: 0,
            });
        }
    }
    for (symbol, _, _, qty, avg, _) in &overseas_positions {
        if *qty > 0 {
            synced += 1;
        }
        mgr.sync_position_for_broker(&BrokerPositionSnapshot {
            broker_id: BrokerId::Kis,
            market: BrokerMarket::Us,
            symbol: symbol.clone(),
            quantity: *qty,
            avg_price: *avg,
        });
    }

    Ok(synced)
}

fn parse_holding_u64(value: &str, symbol: &str, field: &str) -> CmdResult<u64> {
    value
        .trim()
        .replace(',', "")
        .parse::<u64>()
        .map_err(|_| CmdError {
            code: "ACCOUNT_SYNC_INVALID_DATA".into(),
            message: format!("{symbol} {field} 응답 형식이 올바르지 않습니다: {value:?}"),
        })
}

fn parse_holding_price(value: &str, symbol: &str, field: &str) -> CmdResult<u64> {
    value
        .trim()
        .replace(',', "")
        .parse::<f64>()
        .map(|parsed| parsed.max(0.0).round() as u64)
        .map_err(|_| CmdError {
            code: "ACCOUNT_SYNC_INVALID_DATA".into(),
            message: format!("{symbol} {field} 응답 형식이 올바르지 않습니다: {value:?}"),
        })
}

fn parse_holding_usd_cents(value: &str, symbol: &str, field: &str) -> CmdResult<u64> {
    value
        .trim()
        .replace(',', "")
        .parse::<f64>()
        .map(|parsed| (parsed.max(0.0) * 100.0).round() as u64)
        .map_err(|_| CmdError {
            code: "ACCOUNT_SYNC_INVALID_DATA".into(),
            message: format!("{symbol} {field} 응답 형식이 올바르지 않습니다: {value:?}"),
        })
}

async fn sync_toss_strategy_positions(
    state: &AppState,
    profile: AccountProfile,
) -> CmdResult<usize> {
    if !profile.is_configured() {
        return Err(CmdError {
            code: "ACCOUNT_SYNC_FAILED".into(),
            message: "활성 Toss 프로파일 설정이 미완료되어 잔고를 동기화할 수 없습니다.".into(),
        });
    }

    let client = state.rest_client.read().await.clone();
    let scope = BrokerScope::new(
        BrokerId::Toss,
        Some(BrokerAccountId(profile.broker_account_id())),
    );
    let query = state
        .order_manager
        .lock()
        .await
        .begin_holdings_query_for_snapshot(&scope, &client, Some(&profile), HoldingsMarkets::Both)
        .await
        .map_err(|error| CmdError {
            code: "ACCOUNT_SYNC_FAILED".into(),
            message: error.to_string(),
        })?;
    let profile = query.profile().cloned().ok_or_else(|| CmdError {
        code: "ACCOUNT_SYNC_FAILED".into(),
        message: "활성 Toss 프로파일이 없습니다.".into(),
    })?;
    let account_id = BrokerAccountId(profile.broker_account_id());
    let adapter = TossBrokerAdapter::with_credentials(
        TossBrokerAdapter::DEFAULT_BASE_URL,
        profile.app_key,
        profile.app_secret,
        Some(profile.account_no),
    );
    let holdings = adapter
        .list_holdings(Some(&account_id))
        .await
        .map_err(|e| CmdError {
            code: "ACCOUNT_SYNC_FAILED".into(),
            message: format!("자동매매 시작 전 Toss 보유종목 동기화 실패: {e}"),
        })?;
    let parsed_holdings: Vec<(String, String, BrokerMarket, u64, u64, u64)> = holdings
        .iter()
        .map(|holding| {
            let quantity = holding
                .quantity
                .0
                .trim()
                .replace(',', "")
                .parse::<f64>()
                .map_err(|_| CmdError {
                    code: "ACCOUNT_SYNC_INVALID_DATA".into(),
                    message: format!(
                        "{} Toss 보유수량 응답 형식이 올바르지 않습니다.",
                        holding.symbol.0
                    ),
                })?;
            let parse_money = |money: &BrokerMoney, field: &str| -> CmdResult<u64> {
                let value = money
                    .amount
                    .trim()
                    .replace(',', "")
                    .parse::<f64>()
                    .map_err(|_| CmdError {
                        code: "ACCOUNT_SYNC_INVALID_DATA".into(),
                        message: format!(
                            "{} Toss {field} 응답 형식이 올바르지 않습니다.",
                            holding.symbol.0
                        ),
                    })?;
                Ok(match money.currency {
                    BrokerCurrency::Krw => value.max(0.0).round() as u64,
                    BrokerCurrency::Usd => (value.max(0.0) * 100.0).round() as u64,
                })
            };
            Ok((
                holding.symbol.0.clone(),
                holding.symbol_name.clone(),
                holding.market,
                if quantity <= 0.0 {
                    0
                } else {
                    quantity.floor().max(1.0) as u64
                },
                parse_money(&holding.average_price, "평균매입가")?,
                parse_money(&holding.current_price, "현재가")?,
            ))
        })
        .collect::<CmdResult<_>>()?;

    state
        .stock_store
        .upsert_many(
            holdings
                .iter()
                .map(|holding| (holding.symbol.0.clone(), holding.symbol_name.clone())),
        )
        .await;

    let domestic_positions = parsed_holdings
        .iter()
        .filter(|holding| holding.2 == BrokerMarket::Kr)
        .map(|holding| {
            (
                holding.0.clone(),
                holding.1.clone(),
                holding.3,
                holding.4,
                holding.5,
            )
        })
        .collect();
    let overseas_positions = parsed_holdings
        .iter()
        .filter(|holding| holding.2 == BrokerMarket::Us)
        .map(|holding| {
            (
                holding.0.clone(),
                holding.1.clone(),
                "TOSS_US".to_string(),
                holding.3,
                holding.4,
                holding.5,
            )
        })
        .collect();
    let mut synced = 0usize;
    let mut mgr = state.strategy_manager.lock().await;
    let mut orders = state.order_manager.lock().await;
    orders
        .publish_holdings(
            query,
            HoldingsUpdate::Both {
                domestic: domestic_positions,
                overseas: overseas_positions,
            },
        )
        .await
        .map_err(|error| CmdError {
            code: "ACCOUNT_SYNC_FAILED".into(),
            message: error.to_string(),
        })?;
    for symbol in mgr.active_symbols() {
        mgr.sync_position_for_broker(&BrokerPositionSnapshot {
            broker_id: BrokerId::Toss,
            market: if is_domestic_symbol(&symbol) {
                BrokerMarket::Kr
            } else {
                BrokerMarket::Us
            },
            symbol,
            quantity: 0,
            avg_price: 0,
        });
    }
    for holding in &parsed_holdings {
        let quantity = holding.3;
        if quantity > 0 {
            synced += 1;
        }
        mgr.sync_position_for_broker(&BrokerPositionSnapshot {
            broker_id: BrokerId::Toss,
            market: holding.2,
            symbol: holding.0.clone(),
            quantity,
            avg_price: holding.4,
        });
    }
    tracing::info!(
        "Toss holdings 기반 전략 포지션 동기화 완료: {}개 보유 종목",
        synced
    );
    Ok(synced)
}
