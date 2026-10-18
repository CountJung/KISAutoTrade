# -*- coding: utf-8 -*-
"""Independent recorded-fill arithmetic; never generates strategy decisions."""
import csv
import gzip
import hashlib
import json
import statistics
from collections import Counter
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
SOURCE = ROOT / "docs/research/strategy-review-20261018/input.json"
assert hashlib.sha256(SOURCE.read_bytes()).hexdigest() == "d48ce5b59b7876c0a1457121e877fdc1117a77f990b4ec07ce649c92fe153317"
source = json.loads(SOURCE.read_text())
design = json.loads((HERE / "design.json").read_text())
run = json.loads((HERE / "run-summary.json").read_text())
assert (run["flatFullReportAndSignalParityRuns"], run["continuousRuns"], run["partitionEquivalenceRuns"], run["independentQuarterResetRuns"]) == (1224, 72, 72, 864)
def load_lines(name):
    plain = HERE / name
    payload = plain.read_bytes() if plain.exists() else gzip.decompress((HERE / (name + ".gz")).read_bytes())
    return [json.loads(line) for line in payload.decode("utf-8").splitlines()]


rows = [row for name in ["continuous-results.jsonl", "reset-results.jsonl"] for row in load_lines(name)]
assert Counter(r["kind"] for r in rows) == {"continuous": 72, "quarterReset": 864}
checks = 0


def near(left, right, tolerance=0.031):
    global checks
    assert abs(left - right) <= tolerance, (left, right)
    checks += 1


summaries = []
for row in rows:
    config, result = row["config"], row["result"]
    report = result["backtest"]
    summary, account, assumptions = report["summary"], report["accounting"], report["assumptions"]
    dataset = next(d for d in source["datasets"] if d["symbol"] == config["symbol"])["candles"]
    start = row.get("evaluationStart", config["evaluationStart"])
    end = row.get("evaluationEndExclusive", config["evaluationEndExclusive"])
    bars = dataset[start:end]
    assert result["candles"] == bars
    assert assumptions["executionModel"] == "nextOpen"
    assert not assumptions["fxScenario"] and not assumptions["dividendScenario"]
    assert assumptions["stopLossPct"] == assumptions["takeProfitPct"] == 0
    assert result["replay"]["engineVersion"] == design["plannedEngineVersion"]
    assert result["replay"]["warmupCount"] == 252
    assert result["preparation"]["readyAtStart"] is True
    capital, fx = assumptions["initialCapitalKrw"], assumptions["exchangeRateKrw"]
    assert capital == 10_000_000 and fx == 1450
    qty = 0
    cash = capital
    fees = taxes = slippage = market_path_pnl = invested_sum = equity_pct_sum = 0
    peak_cap = peak_eq = held_bars = 0
    previous_close = float(dataset[start - 1]["close"])
    quantities_before, quantities_after = {}, {}
    filled = [t for t in report["trades"] if t["status"] == "filled"]
    for index, bar in enumerate(bars):
        date = bar["date"]
        open_price, close_price = float(bar["open"]), float(bar["close"])
        quantities_before[date] = qty
        # q_before is known before this open, q_after only after accepted open fills.
        market_path_pnl += qty * (open_price - previous_close) * fx
        for trade in (t for t in filled if t["fillAt"]["barTime"] == date):
            assert trade["fillAt"]["phase"] == "open"
            q = trade["quantity"]
            assert q > 0
            gross = trade["fillPrice"] * q * fx
            fee = gross * row["costBps"] / 10000
            slip = abs(trade["fillPrice"] - open_price) * q * fx
            near(trade["grossAmountKrw"], gross)
            near(trade["executionReferencePrice"], open_price)
            near(trade["feeKrw"], fee)
            near(trade["taxKrw"], 0)
            near(trade["slippageKrw"], slip)
            fees += fee
            taxes += trade["taxKrw"]
            slippage += slip
            if trade["side"] == "buy":
                qty += q
                cash -= gross + fee
            else:
                assert q <= qty
                qty -= q
                cash += gross - fee
        market_path_pnl += qty * (close_price - open_price) * fx
        previous_close = close_price
        quantities_after[date] = qty
        market_value = close_price * fx * qty
        equity = cash + market_value
        near(report["equityCurve"][index]["equityKrw"], equity)
        invested_sum += market_value
        equity_pct = market_value / equity * 100 if equity > 0 else 0
        equity_pct_sum += equity_pct
        peak_cap = max(peak_cap, market_value / capital * 100)
        peak_eq = max(peak_eq, equity_pct)
        held_bars += qty > 0
    near(summary["finalEquityKrw"], equity)
    near(account["realizedPnlKrw"] + account["residualUnrealizedPnlKrw"] + account["dividendNetKrw"], equity - capital)
    near(account["remainingQuantity"], qty)
    near(account["totalFeeKrw"], fees)
    near(account["totalTaxKrw"], taxes)
    near(account["totalSlippageKrw"], slippage)
    near(account["averageInvestedNotionalKrw"], invested_sum / len(bars))
    near(account["averageInvestedCapitalPct"], invested_sum / len(bars) / capital * 100, 0.00011)
    near(account["averageInvestedEquityPct"], equity_pct_sum / len(bars), 0.00011)
    near(account["peakInvestedCapitalPct"], peak_cap, 0.00011)
    near(account["peakInvestedEquityPct"], peak_eq, 0.00011)
    near(summary["exposurePct"], held_bars / len(bars) * 100, 0.00011)
    terminal_slippage = market_value * row["costBps"] / 10000
    terminal_fee = (market_value - terminal_slippage) * row["costBps"] / 10000
    near(account["terminalLiquidationEquityKrw"], equity - terminal_slippage - terminal_fee)
    reference = report["comparison"]["sameExecutionPathCostReference"]
    near(reference["restoredCostKrw"], fees + taxes + slippage)
    near(reference["finalEquityKrw"], equity + fees + taxes + slippage)
    near(reference["finalEquityKrw"], capital + market_path_pnl)
    if row["kind"] == "continuous":
        continuity = result["continuity"]
        assert continuity["mode"] == "continuous" and continuity["chunkCount"] == 12
        assert continuity["evaluationBars"] == 752 and continuity["historyBars"] == 252
        signals_at = {signal["time"] for signal in result["signals"]}
        for chunk in continuity["chunks"]:
            near(chunk["startQuantity"], quantities_before[chunk["start"]])
            near(chunk["endQuantity"], quantities_after[chunk["end"]])
            assert chunk["pendingAtEnd"] == (chunk["end"] in signals_at)
    summaries.append({
        "kind":row["kind"], "symbol":config["symbol"], "strategyId":config["strategyId"],
        "costBps":row["costBps"], "quarter":row.get("quarter", "2023-2025"),
        "orderQuantity":config["orderQuantity"], "returnPct":summary["cumulativeReturnPct"],
        "terminalReturnPct":account["terminalLiquidationReturnPct"], "mddPct":summary["mddPct"],
        "realizedPnlKrw":account["realizedPnlKrw"], "residualPnlKrw":account["residualUnrealizedPnlKrw"],
        "completedTrades":summary["completedTrades"], "blockedOrders":summary["blockedOrderCount"],
        "turnoverPct":summary["turnoverPct"], "averageInvestedCapitalPct":account["averageInvestedCapitalPct"],
        "holdingBarPct":summary["exposurePct"],
        "sameRequestedQuantityBuyHoldReturnPct":report["comparison"]["sameInputQuantityBuyHold"]["cumulativeReturnPct"],
        "sameActualQuantityPathReferenceReturnPct":reference["cumulativeReturnPct"],
    })

with (HERE / "summary.csv").open("w", newline="") as file:
    writer = csv.DictWriter(file, fieldnames=list(summaries[0]), lineterminator="\n")
    writer.writeheader()
    writer.writerows(summaries)
aggregates = []
for continuous in (r for r in summaries if r["kind"] == "continuous"):
    resets = [r for r in summaries if r["kind"] == "quarterReset" and all(r[k] == continuous[k] for k in ["symbol", "strategyId", "costBps"])]
    assert len(resets) == 12
    aggregates.append({"continuous":continuous, "separateQuarterAccountDistribution":{
        "count":12,"medianReturnPct":statistics.median(r["returnPct"] for r in resets),
        "minReturnPct":min(r["returnPct"] for r in resets), "maxReturnPct":max(r["returnPct"] for r in resets),
        "note":"Separate reset accounts. These returns are never summed/compounded into a three-year return."
    }})
(HERE / "aggregate.json").write_text(json.dumps(aggregates, indent=2, ensure_ascii=False) + "\n")
# Preserve full records losslessly; bulky generated plaintext stays in project-local cache.
archives = {}
for name in ["continuous-results.jsonl", "reset-results.jsonl"]:
    plain, packed = HERE / name, HERE / (name + ".gz")
    if plain.exists():
        payload = plain.read_bytes()
        packed.write_bytes(gzip.compress(payload, compresslevel=9, mtime=0))
        assert gzip.decompress(packed.read_bytes()) == payload
        cache = ROOT / ".cache/continuous-research-output"
        cache.mkdir(parents=True, exist_ok=True)
        plain.replace(cache / name)
    payload = gzip.decompress(packed.read_bytes())
    archives[packed.name] = {"uncompressedSha256":hashlib.sha256(payload).hexdigest(),
        "uncompressedBytes":len(payload), "compressedBytes":packed.stat().st_size}
hashed = [SOURCE] + [p for p in HERE.iterdir() if p.is_file() and not p.name.startswith(".") and p.name != "validation.json"]
hashed += list((ROOT / "src-tauri/src/commands/strategy_preview").glob("*.rs"))
hashed += list((ROOT / "src-tauri/src/trading/simulation").rglob("*.rs"))
hashed += [ROOT / "src-tauri/src/trading/simulation.rs", ROOT / "src-tauri/src/commands/strategy_preview.rs"]
validation = {
    "baselineCommit":"cc6f5d0", "flatFullReportAndSignalParityRuns":1224,
    "continuousRuns":72, "alternatePartitionEquivalenceRuns":72, "separateQuarterResetRuns":864,
    "independentArithmeticChecks":checks,
    "archives":archives,
    "scope":"Recorded accepted open quantities, cash/exposure/terminal and causal overnight/intraday market PnL. No strategy signal reimplementation.",
    "sha256":{str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(set(hashed))},
}
(HERE / "validation.json").write_text(json.dumps(validation, indent=2, ensure_ascii=False) + "\n")
print(json.dumps({k:v for k,v in validation.items() if k != "sha256"}))
