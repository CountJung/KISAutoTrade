# -*- coding: utf-8 -*-
"""Reconcile recorded production fills; does not regenerate strategy signals."""
import hashlib
import json
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
source = ROOT / "docs/research/strategy-review-20261018/input.json"
assert hashlib.sha256(source.read_bytes()).hexdigest() == "d48ce5b59b7876c0a1457121e877fdc1117a77f990b4ec07ce649c92fe153317"
data = json.loads(source.read_text())
results = json.loads((HERE / "results.json").read_text())
assert results["sameCloseFullReportAndSignalParityRuns"] == 1224
assert results["accountingIdentityChecks"] == 120
assert len(results["comparisons"]) == 40
checks = 0


def near(left, right, tolerance=0.031):
    global checks
    assert abs(left - right) <= tolerance, (left, right)
    checks += 1


for row in results["comparisons"]:
    report = row["backtest"]
    a = report["assumptions"]
    account = report["accounting"]
    summary = report["summary"]
    candles = next(d for d in data["datasets"] if d["symbol"] == row["symbol"])["candles"]
    window = row["window"]
    candles = candles[window["evaluationStart"]:window["evaluationEndExclusive"]]
    fx_points = {p["barTime"]: p["rateKrw"] for p in a["fxScenario"]}
    dividends = {p["barTime"]: p for p in a["dividendScenario"]}
    fills = [t for t in report["trades"] if t["status"] == "filled"]
    cash, qty, native_basis, krw_basis, buy_fees = a["initialCapitalKrw"], 0, 0, 0, 0
    realized = price_pnl = fx_pnl = net_div = gross_div = div_tax = 0
    total_fee = total_tax = total_slip = invested_sum = equity_pct_sum = 0
    peak_cap = peak_eq = held_bars = 0
    fx = a["exchangeRateKrw"]
    for i, candle in enumerate(candles):
        label = candle["date"]
        fx = fx_points.get(label, fx)
        if label in dividends:
            point = dividends[label]
            gross = qty * point["amountPerShare"] * fx
            tax = gross * point["withholdingTaxBps"] / 10000
            gross_div += gross
            div_tax += tax
            net_div += gross - tax
            cash += gross - tax
        for trade in (t for t in fills if t["fillAt"]["barTime"] == label):
            q = trade["quantity"]
            fill = trade["fillPrice"]
            gross = fill * fx * q
            near(gross, trade["grossAmountKrw"])
            near(fx, trade["exchangeRateKrw"])
            bps = a["buyFeeBps"] if trade["side"] == "buy" else a["sellFeeBps"]
            bps = a["feeBps"] if bps is None else bps
            fee = max(gross * bps / 10000, a["minimumFeeNative"] * fx)
            tax = 0 if trade["side"] == "buy" else gross * a["taxBps"] / 10000
            slip = abs(fill - trade["executionReferencePrice"]) * fx * q
            near(fee, trade["feeKrw"])
            near(tax, trade["taxKrw"])
            near(slip, trade["slippageKrw"])
            near(fee + tax + slip, trade["costKrw"])
            total_fee += fee
            total_tax += tax
            total_slip += slip
            if trade["side"] == "buy":
                cash -= gross + fee
                qty += q
                native_basis += fill * q
                krw_basis += gross
                buy_fees += fee
            else:
                assert 0 < q <= qty
                fraction = q / qty
                allocated_fee = buy_fees * fraction
                sold_native = native_basis * fraction
                sold_krw = krw_basis * fraction
                entry_fx = sold_krw / sold_native
                net = gross - sold_krw - fee - tax - allocated_fee
                near(net, trade["realizedPnlKrw"])
                realized += net
                price_pnl += (fill * q - sold_native) * entry_fx
                fx_pnl += fill * q * (fx - entry_fx)
                cash += gross - fee - tax
                native_basis -= sold_native
                krw_basis -= sold_krw
                buy_fees -= allocated_fee
                qty -= q
        mark = float(candle["close"]) * fx * qty
        equity = cash + mark
        near(equity, report["equityCurve"][i]["equityKrw"])
        invested_sum += mark
        pct = mark / equity * 100 if equity > 0 else 0
        equity_pct_sum += pct
        peak_cap = max(peak_cap, mark / a["initialCapitalKrw"] * 100)
        peak_eq = max(peak_eq, pct)
        held_bars += qty > 0
    near(summary["finalEquityKrw"], equity)
    near(account["remainingQuantity"], qty)
    near(account["remainingCostBasisKrw"], krw_basis)
    near(account["remainingBuyFeeKrw"], buy_fees)
    near(account["realizedPnlKrw"], realized)
    near(account["residualUnrealizedPnlKrw"], mark - krw_basis - buy_fees)
    near(account["realizedPricePnlKrw"], price_pnl)
    near(account["realizedFxPnlKrw"], fx_pnl)
    if qty:
        entry_fx = krw_basis / native_basis
        near(account["residualPricePnlKrw"], (float(candles[-1]["close"]) * qty - native_basis) * entry_fx)
        near(account["residualFxPnlKrw"], float(candles[-1]["close"]) * qty * (fx - entry_fx))
    near(account["totalFeeKrw"], total_fee)
    near(account["totalTaxKrw"], total_tax)
    near(account["totalSlippageKrw"], total_slip)
    near(account["dividendGrossKrw"], gross_div)
    near(account["dividendTaxKrw"], div_tax)
    near(account["dividendNetKrw"], net_div)
    near(account["averageInvestedNotionalKrw"], invested_sum / len(candles))
    near(account["averageInvestedCapitalPct"], invested_sum / len(candles) / a["initialCapitalKrw"] * 100, 0.00011)
    near(account["averageInvestedEquityPct"], equity_pct_sum / len(candles), 0.00011)
    near(account["peakInvestedCapitalPct"], peak_cap, 0.00011)
    near(account["peakInvestedEquityPct"], peak_eq, 0.00011)
    near(summary["exposurePct"], held_bars / len(candles) * 100, 0.00011)
    terminal_gross = mark * (1 - a["slippageBps"] / 10000)
    terminal_fee = max(terminal_gross * (a["sellFeeBps"] if a["sellFeeBps"] is not None else a["feeBps"]) / 10000, a["minimumFeeNative"] * fx) if qty else 0
    terminal_tax = terminal_gross * a["taxBps"] / 10000
    near(account["terminalExitFeeKrw"], terminal_fee)
    near(account["terminalExitTaxKrw"], terminal_tax)
    near(account["terminalExitSlippageKrw"], mark - terminal_gross)
    near(account["terminalLiquidationEquityKrw"], equity - (mark - terminal_gross) - terminal_fee - terminal_tax)
    reference = report["comparison"]["sameExecutionPathCostReference"]
    near(reference["restoredCostKrw"], total_fee + total_tax + total_slip + div_tax)
    near(reference["finalEquityKrw"], equity + total_fee + total_tax + total_slip + div_tax)

hashed = [source, HERE / "runner.rs", HERE / "baseline_runner.rs", HERE / "baseline-hashes.json", HERE / "results.json", HERE / "validate.py"]
hashed += list((ROOT / "src-tauri/src/trading/simulation").rglob("*.rs"))
hashed += [ROOT / p for p in ["src-tauri/src/trading/simulation.rs", "src-tauri/src/commands/strategy_preview.rs", "src-tauri/src/commands/strategy_preview/generic.rs"]]
validation = {
    "baselineCommit": "1192673",
    "sameCloseFullReportAndSignalParityRuns": 1224,
    "productionScenarioRuns": 40,
    "productionAccountingIdentityChecks": 120,
    "independentRecordedFillReconciliationChecks": checks,
    "scope": "Frozen daily prices; synthetic FX/dividend/fee inputs. No strategy signal reimplementation or live account access.",
    "sha256": {str(p.relative_to(ROOT)): hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(set(hashed))},
}
(HERE / "validation.json").write_text(json.dumps(validation, indent=2, ensure_ascii=False) + "\n")
print(json.dumps({k: v for k, v in validation.items() if k != "sha256"}))
