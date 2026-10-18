//! Offline production-engine comparison. No credentials, provider or order APIs.
use kis_auto_trade_lib::{
    api::rest::ChartCandle,
    commands::{preview_strategy_from_candles, StrategyPreviewInput},
    trading::simulation::{replay_input_hash, ExecutionModel, SimulationAssumptions},
};
use serde_json::{json, Value};
use std::{error::Error, fs};

fn main() -> Result<(), Box<dyn Error>> {
    let root = std::env::args().nth(1).ok_or("repository path required")?;
    let frozen = format!("{root}/docs/research/strategy-review-20261018");
    let input: Value = serde_json::from_str(&fs::read_to_string(format!("{frozen}/input.json"))?)?;
    let datasets = input["datasets"].as_array().ok_or("datasets")?;
    let baseline: Vec<Value> = serde_json::from_str(&fs::read_to_string(format!(
        "{root}/docs/research/replay-accounting-20261018/baseline-hashes.json"
    ))?)?;
    let mut identity_checks = 0;
    let mut comparisons = Vec::new();
    let mut parity = 0;
    for line in fs::read_to_string(format!("{frozen}/results.jsonl"))?.lines() {
        let row: Value = serde_json::from_str(line)?;
        if row["kind"] != "strategy" {
            continue;
        }
        let symbol = row["symbol"].as_str().ok_or("symbol")?;
        let strategy_id = row["strategyId"].as_str().ok_or("strategy")?;
        let data = datasets
            .iter()
            .find(|data| data["symbol"] == symbol)
            .ok_or("dataset")?;
        let candles: Vec<ChartCandle> = serde_json::from_value(data["candles"].clone())?;
        let window = &row["window"];
        let h = window["historyStart"].as_u64().ok_or("history")? as usize;
        let start = window["evaluationStart"].as_u64().ok_or("start")? as usize;
        let end = window["evaluationEndExclusive"].as_u64().ok_or("end")? as usize;
        let quantity = row["orderQuantity"].as_u64().ok_or("quantity")?;
        let assumptions: SimulationAssumptions =
            serde_json::from_value(row["assumptions"].clone())?;
        let run = |model| {
            let mut assumptions = assumptions.clone();
            assumptions.execution_model = model;
            preview_strategy_from_candles(StrategyPreviewInput {
                strategy_id: strategy_id.into(),
                strategy_name: strategy_id.into(),
                symbol: symbol.into(),
                is_overseas: true,
                order_quantity: quantity,
                params: row["params"].clone(),
                candles: candles[start..end].to_vec(),
                history_candles: candles[h..start].to_vec(),
                warmup_count: None,
                interval: Some("1d".into()),
                data_source: Some("frozen strategy-review-20261018".into()),
                strategy_version: None,
                broker_id: None,
                broker_account_id: None,
                assumptions,
            })
            .map_err(|error| format!("{}: {}", error.code, error.message))
        };
        let same_close = run(ExecutionModel::SameClose)?;
        let key = format!("{}|{strategy_id}|{}", window["id"], row["costBps"]);
        let prior = baseline
            .iter()
            .find(|record| record["key"] == key)
            .ok_or("HEAD baseline missing")?;
        let mut report = serde_json::to_value(&same_close.backtest)?;
        report.as_object_mut().unwrap().remove("accounting");
        report.as_object_mut().unwrap().remove("comparison");
        for field in [
            "costSource",
            "costNote",
            "buyFeeBps",
            "sellFeeBps",
            "minimumFeeNative",
            "fxScenario",
            "dividendScenario",
            "dividendPriceBasis",
        ] {
            report["assumptions"].as_object_mut().unwrap().remove(field);
        }
        for trade in report["trades"].as_array_mut().unwrap() {
            for field in [
                "executionReferencePrice",
                "exchangeRateKrw",
                "feeKrw",
                "taxKrw",
                "slippageKrw",
            ] {
                trade.as_object_mut().unwrap().remove(field);
            }
        }
        let report_hash = replay_input_hash(&[&serde_json::to_string(&report)?]);
        let signal_hash = replay_input_hash(&[&serde_json::to_string(&same_close.signals)?]);
        if prior["reportHash"] != report_hash || prior["signalsHash"] != signal_hash {
            return Err(format!("HEAD sameClose accounting/signal regression {key}").into());
        }
        parity += 1;
        if window["id"] != "QQQ-2026Q3" || !matches!(strategy_id, "mean_reversion" | "strong_close")
        {
            continue;
        }
        for model in [ExecutionModel::SameClose, ExecutionModel::NextOpen] {
            for scenario in [
                "fixed",
                "fxOnly",
                "dividendOnly",
                "fxAndDividend",
                "syntheticFeeSchedule",
            ] {
                let mut changed = assumptions.clone();
                changed.execution_model = model;
                if matches!(scenario, "fxOnly" | "fxAndDividend") {
                    changed.fx_scenario = serde_json::from_value(json!([
                        {"barTime":"20260803","rateKrw":1300.0},
                        {"barTime":"20260901","rateKrw":1600.0}
                    ]))?;
                }
                if matches!(scenario, "dividendOnly" | "fxAndDividend") {
                    changed.dividend_scenario = serde_json::from_value(json!([
                        {"barTime":"20260803","amountPerShare":1.0,"withholdingTaxBps":1500.0}
                    ]))?;
                }
                if scenario == "syntheticFeeSchedule" {
                    changed.buy_fee_bps = Some(8.0);
                    changed.sell_fee_bps = Some(12.0);
                    changed.minimum_fee_native = 1.0;
                    changed.cost_note =
                        "Synthetic research fee schedule; not a broker quote".into();
                }
                let preview = preview_strategy_from_candles(StrategyPreviewInput {
                    strategy_id: strategy_id.into(),
                    strategy_name: strategy_id.into(),
                    symbol: symbol.into(),
                    is_overseas: true,
                    order_quantity: quantity,
                    params: row["params"].clone(),
                    candles: candles[start..end].to_vec(),
                    history_candles: candles[h..start].to_vec(),
                    warmup_count: None,
                    interval: Some("1d".into()),
                    data_source: Some(
                        "frozen strategy-review-20261018; synthetic cash-flow scenarios".into(),
                    ),
                    strategy_version: None,
                    broker_id: None,
                    broker_account_id: None,
                    assumptions: changed,
                })
                .map_err(|error| format!("{}: {}", error.code, error.message))?;
                let r = &preview.backtest;
                let a = &r.accounting;
                let near = |left: f64, right: f64| -> Result<(), Box<dyn Error>> {
                    if (left - right).abs() > 0.031 {
                        return Err(format!("Accounting identity {left} != {right}").into());
                    }
                    Ok(())
                };
                near(
                    r.summary.final_equity_krw - r.summary.initial_capital_krw,
                    a.realized_pnl_krw + a.residual_unrealized_pnl_krw + a.dividend_net_krw,
                )?;
                near(
                    a.realized_pnl_krw + a.residual_unrealized_pnl_krw,
                    a.realized_price_pnl_krw
                        + a.realized_fx_pnl_krw
                        + a.residual_price_pnl_krw
                        + a.residual_fx_pnl_krw
                        - a.total_fee_krw
                        - a.total_tax_krw,
                )?;
                near(
                    a.terminal_liquidation_equity_krw,
                    r.summary.final_equity_krw
                        - a.terminal_exit_fee_krw
                        - a.terminal_exit_tax_krw
                        - a.terminal_exit_slippage_krw,
                )?;
                identity_checks += 3;
                comparisons.push(json!({"window":window,"strategyId":strategy_id,"symbol":symbol,
                    "costBps":row["costBps"],"orderQuantity":quantity,"params":row["params"],
                    "scenario":scenario,"historyBars":start-h,"evaluationBars":end-start,
                    "replay":preview.replay,"backtest":preview.backtest,"preparation":preview.preparation
                }));
            }
        }
    }

    if parity != 1224 || comparisons.len() != 40 || identity_checks != 120 {
        return Err(format!(
            "Unexpected coverage: {parity}/{} /{identity_checks}",
            comparisons.len()
        )
        .into());
    }
    fs::write(
        format!("{root}/docs/research/replay-accounting-20261018/results.json"),
        serde_json::to_string_pretty(&json!({
            "source":"../strategy-review-20261018/input.json","baselineCommit":"1192673","sameCloseFullReportAndSignalParityRuns":parity,
            "accountingIdentityChecks":identity_checks,
            "comparisons":comparisons,
            "limitations":["Frozen daily bars; calendar/corporate-action coverage unverified", "Requested quantity fixed. FX/dividend/fee overrides are hypothetical inputs, not actual QQQ or broker data", "OHLC path and actual market fills unverified; no research overlay enabled in these comparisons"]
        }))? + "\n",
    )?;
    println!(
        "sameClose full accounting report and signal parity {parity}; production comparisons {}",
        comparisons.len()
    );
    Ok(())
}
