# -*- coding: utf-8 -*-
"""Independent recorded-fill arithmetic; never generates strategy decisions."""
import csv
import gzip
import hashlib
import json
import statistics
import itertools
import math
from collections import Counter
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
SOURCE = ROOT / "docs/research/strategy-review-20261018/input.json"
assert hashlib.sha256(SOURCE.read_bytes()).hexdigest() == "d48ce5b59b7876c0a1457121e877fdc1117a77f990b4ec07ce649c92fe153317"
source = json.loads(SOURCE.read_text())
design = json.loads((HERE / "design.json").read_text())
run = json.loads((HERE / "run-summary.json").read_text())
assert (run["unchangedStrategyFullReportAndSignalParityRuns"], run["continuousRuns"], run["partitionEquivalenceRuns"], run["independentQuarterResetRuns"]) == (1224, 48, 48, 576)
def load_lines(name):
    plain = HERE / name
    payload = plain.read_bytes() if plain.exists() else gzip.decompress((HERE / (name + ".gz")).read_bytes())
    return [json.loads(line) for line in payload.decode("utf-8").splitlines()]


rows = [row for name in ["continuous-results.jsonl", "reset-results.jsonl"] for row in load_lines(name)]
assert Counter(r["kind"] for r in rows) == {"continuous": 48, "quarterReset": 576}

# Verify exact factorial coverage, not merely a row count that permits duplicates.
variants={v["id"] for v in design["variants"]}
symbols={c["symbol"] for c in design["configs"]}
costs=set(design["costBps"])
quarters={f"{year}Q{quarter}" for year in range(2023,2026) for quarter in range(1,5)}
full_key=lambda r:(r["kind"],r["variant"],r["config"]["symbol"],r["costBps"],r.get("quarter"))
expected={("continuous",v,s,c,None) for v,s,c in itertools.product(variants,symbols,costs)}
expected|={("quarterReset",v,s,c,q) for v,s,c,q in itertools.product(variants,symbols,costs,quarters)}
assert len({full_key(r) for r in rows})==len(rows)
assert {full_key(r) for r in rows}==expected
partition_key=lambda r:(r["variant"],r["symbol"],r["costBps"])
partitions={partition_key(p):p for p in run["partitions"]}
assert len(partitions)==len(run["partitions"])==48
assert set(partitions)==set(itertools.product(variants,symbols,costs))

assert design["declaredBeforeCandidateRuns"] is False
assert design["followup"]["declaredAfterStageA"] and design["followup"]["declaredBeforeItsCandidateRuns"]
original=json.dumps(design["stageA"]["originalDesign"],indent=2,ensure_ascii=False)+"\n"
assert hashlib.sha256(original.encode()).hexdigest()==design["stageA"]["originalDesignSha256"]
assert run["stageAControlEquivalenceRuns"]==36
for old in design["stageA"]["runSummary"]["partitions"]:
    now=partitions[partition_key(old)]
    assert now["economicHash"]==old["economicHash"] and now["inputHash"]==old["inputHash"]

for r in (r for r in rows if r["kind"]=="continuous"):
    assert partitions[(r["variant"],r["config"]["symbol"],r["costBps"])]["inputHash"]==r["result"]["replay"]["inputHash"]

baseline=json.loads((HERE / "baseline-hashes.json").read_text())
assert len(baseline)==len({r["key"] for r in baseline})==1224
assert sum("|ma_cross|" in r["key"] for r in baseline)==102
assert run["maCrossDefaultReportAndSignalParityRuns"]==102
row_key=lambda r:(r["kind"],r["config"]["symbol"],r["costBps"],r.get("quarter"))
legacy_rows={row_key(r):r for r in rows if r["variant"]=="legacy"}
assert len(legacy_rows)==78
checks = 0
filter_predicate_checks = 0


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
    assert result["replay"]["engineVersion"] == ("strategy-replay-v16" if row["variant"]=="legacy" else design["plannedEngineVersion"])
    assert result["replay"]["warmupCount"] == 252
    assert result["preparation"]["readyAtStart"] is True
    original = next(c for c in design["configs"] if c["symbol"]==config["symbol"])
    variant = next(v for v in design["variants"] if v["id"]==row["variant"])
    assert config["params"] == dict(original["params"], **variant["params"])
    assert config["orderQuantity"] == original["orderQuantity"]
    assert row["costBps"] in design["costBps"]
    assert len(bars)==(752 if row["kind"]=="continuous" else len(bars))
    expected_mdd = 0
    running_peak = capital_for_mdd = assumptions["initialCapitalKrw"]
    for point in report["equityCurve"]:
        running_peak = max(running_peak, point["equityKrw"])
        expected_mdd = max(expected_mdd, (running_peak-point["equityKrw"])/running_peak*100)
    near(summary["mddPct"], expected_mdd, 0.00011)
    near(summary["turnoverPct"], sum(t["grossAmountKrw"] for t in report["trades"] if t["status"]=="filled")/capital_for_mdd*100, 0.00011)
    capital, fx = assumptions["initialCapitalKrw"], assumptions["exchangeRateKrw"]
    assert capital == 10_000_000 and fx == 1450
    qty = 0
    cash = capital
    basis_krw = buy_fees = realized_pnl = 0
    fees = taxes = slippage = market_path_pnl = invested_sum = equity_pct_sum = 0
    peak_cap = peak_eq = held_bars = 0
    previous_close = float(dataset[start - 1]["close"])
    quantities_before, quantities_after = {}, {}
    flat_sells=held_buys=0
    filled = [t for t in report["trades"] if t["status"] == "filled"]
    for index, bar in enumerate(bars):
        date = bar["date"]
        open_price, close_price = float(bar["open"]), float(bar["close"])
        quantities_before[date] = qty
        # q_before is known before this open, q_after only after accepted open fills.
        market_path_pnl += qty * (open_price - previous_close) * fx
        for trade in (t for t in filled if t["fillAt"]["barTime"] == date):
            assert trade["fillAt"]["phase"] == "open"
            signal=trade["signalAvailableAt"]
            assert signal["phase"]=="close"
            signal_index=next(i for i,b in enumerate(bars) if b["date"]==signal["barTime"])
            assert signal_index+1==index, (signal,trade["fillAt"])
            assert trade["time"]==date
            near(trade["signalPrice"],float(bars[signal_index]["close"]),0.0000001)
            expected_fill=open_price*(1+row["costBps"]/10000 if trade["side"]=="buy" else 1-row["costBps"]/10000)
            near(trade["fillPrice"],expected_fill,0.0000001)
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
                basis_krw += gross
                buy_fees += fee
                qty += q
                cash -= gross + fee
            else:
                assert q <= qty
                fraction=q/qty
                realized=gross-fee-(basis_krw+buy_fees)*fraction
                near(trade["realizedPnlKrw"],realized)
                realized_pnl+=realized
                basis_krw*=1-fraction
                buy_fees*=1-fraction
                qty -= q
                cash += gross - fee
        for signal in (x for x in result["signals"] if x["time"]==date):
            flat_sells += signal["side"]=="sell" and qty==0
            held_buys += signal["side"]=="buy" and qty>0
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
    near(account["remainingCostBasisKrw"],basis_krw)
    near(account["remainingBuyFeeKrw"],buy_fees)
    near(account["realizedPnlKrw"],realized_pnl)
    near(account["residualUnrealizedPnlKrw"],market_value-basis_krw-buy_fees)
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
    prep=result.get("entryFilterPreparation")
    if row["variant"] in {"legacy","unfiltered"}:
        assert prep is None
    else:
        assert prep["enabled"] and prep["cadence"]=="priorCompletedDailyOhlc"
        assert prep["candidates"]>=prep["passedCandidates"]
        assert prep["candidates"]>=prep["unreadyCandidates"]
        rejects=sum(prep[k] for k in ["trendRejections","slopeRejections","atrRejections","bandwidthRejections"])
        assert prep["candidates"]==prep["passedCandidates"]+prep["unreadyCandidates"]+rejects
        assert prep["passedCandidates"]==sum(s["side"]=="buy" for s in result["signals"])
        assert prep["requiredCompletedBars"]=={"trend200":200,"slope0_1":21,"atr0_5":15,"bandwidth2":20,"atr2":15,"bandwidth10":20}[row["variant"]]
    # Audit only the declared filter predicates against actual archived cross
    # candidates. This never generates a separate trading/crossover model.
    legacy_signals=legacy_rows[row_key(row)]["result"]["signals"]
    assert [x for x in result["signals"] if x["side"]=="sell"]==[x for x in legacy_signals if x["side"]=="sell"]
    filter_predicate_checks += 1
    if prep:
        candidates=[x for x in legacy_signals if x["side"]=="buy"]
        expected_buys=[]
        for candidate in candidates:
            i=next(i for i,b in enumerate(dataset) if b["date"]==candidate["time"])
            hs=row.get("historyStart",config["historyStart"])
            past=dataset[max(hs,i-512):i]
            units=lambda v:math.floor(float(v)*100.0+0.5)
            closes=[units(b["close"]) for b in past]
            current=units(dataset[i]["close"])
            variant=row["variant"]
            if variant=="trend200":
                allowed=current>sum(closes[-200:])/200
            elif variant=="slope0_1":
                value=((sum(closes[-20:])/20)/(sum(closes[-21:-1])/20)-1)*100
                allowed=value>=0.1
            elif variant in {"atr0_5","atr2"}:
                last=past[-15:]
                tr=[max(units(b["high"])-units(b["low"]),abs(units(b["high"])-units(a["close"])),abs(units(b["low"])-units(a["close"]))) for a,b in zip(last,last[1:])]
                allowed=(sum(tr)/14)/closes[-1]*100>=config["params"]["min_atr_pct"]
            elif variant in {"bandwidth2","bandwidth10"}:
                vals=closes[-20:];mean=sum(vals)/20
                width=4*math.sqrt(sum((v-mean)**2 for v in reversed(vals))/20)/mean*100
                allowed=width>=config["params"]["min_bandwidth_pct"]
            else:
                raise AssertionError(variant)
            if allowed:expected_buys.append(candidate)
            filter_predicate_checks+=1
        assert prep["candidates"]==len(candidates)
        assert [x for x in result["signals"] if x["side"]=="buy"]==expected_buys
        filter_predicate_checks+=2
    old_report=legacy_rows[row_key(row)]["result"]["backtest"]
    old_filled=[t for t in old_report["trades"] if t["status"]=="filled"]
    same_filled_records=old_filled==filled
    # Reason text can differ when a protective trigger coincides with the old
    # original upper-band exit. Keep original records and compare every other field.
    economic_fills=lambda trades:[{k:v for k,v in t.items() if k!="reason"} for t in trades]
    same_path=all(old_report[k]==report[k] for k in ["accounting","equityCurve"]) and economic_fills(old_filled)==economic_fills(filled)
    if row["variant"]=="unfiltered":
        assert old_report==report
        assert legacy_rows[row_key(row)]["result"]["signals"]==result["signals"]
    summaries.append({
        "entryCandidates":prep["candidates"] if prep else None,"entryPassed":prep["passedCandidates"] if prep else None,
        "entryUnready":prep["unreadyCandidates"] if prep else None,
        "entryRejected":sum(prep[k] for k in ["trendRejections","slopeRejections","atrRejections","bandwidthRejections"]) if prep else None,
        "kind":row["kind"], "variant":row["variant"], "symbol":config["symbol"], "strategyId":config["strategyId"],
        "costBps":row["costBps"], "quarter":row.get("quarter", "2023-2025"),
        "orderQuantity":config["orderQuantity"], "returnPct":summary["cumulativeReturnPct"],
        "terminalReturnPct":account["terminalLiquidationReturnPct"], "mddPct":summary["mddPct"],
        "realizedPnlKrw":account["realizedPnlKrw"], "residualPnlKrw":account["residualUnrealizedPnlKrw"],
        "completedTrades":summary["completedTrades"], "blockedOrders":summary["blockedOrderCount"],
        "turnoverPct":summary["turnoverPct"], "averageInvestedCapitalPct":account["averageInvestedCapitalPct"],
        "holdingBarPct":summary["exposurePct"],
        "worstCompletedPnlKrw":min([t["realizedPnlKrw"] for t in report["trades"] if t["status"]=="filled" and t["side"]=="sell"], default=None),
        "rawSignals":len(result["signals"]),
        "filledOrders":summary["filledOrderCount"],
        "blockedReasons":dict(Counter(t["blockedReason"] for t in report["trades"] if t["status"]=="blocked")),
        "flatSellSignals":flat_sells,"heldBuySignals":held_buys,
        "sameAcceptedEconomicPathAsLegacy":same_path,
        "sameAcceptedFilledRecordsAsLegacy":same_filled_records,
        "rawSignalDifferenceFromLegacy":len(result["signals"])-old_report["summary"]["signalCount"],
        "blockedOrderDifferenceFromLegacy":summary["blockedOrderCount"]-old_report["summary"]["blockedOrderCount"],
        "sameRequestedQuantityBuyHoldReturnPct":report["comparison"]["sameInputQuantityBuyHold"]["cumulativeReturnPct"],
        "sameActualQuantityPathReferenceReturnPct":reference["cumulativeReturnPct"],
    })

with (HERE / "summary.csv").open("w", newline="") as file:
    writer = csv.DictWriter(file, fieldnames=list(summaries[0]), lineterminator="\n")
    writer.writeheader()
    writer.writerows(summaries)
aggregates = []
for continuous in (r for r in summaries if r["kind"] == "continuous"):
    resets = [r for r in summaries if r["kind"] == "quarterReset" and all(r[k] == continuous[k] for k in ["symbol", "strategyId", "costBps", "variant"])]
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
        cache = ROOT / ".cache/ma-cross-research-output"
        cache.mkdir(parents=True, exist_ok=True)
        plain.replace(cache / name)
    payload = gzip.decompress(packed.read_bytes())
    archives[packed.name] = {"uncompressedSha256":hashlib.sha256(payload).hexdigest(),
        "uncompressedBytes":len(payload), "compressedBytes":packed.stat().st_size}
for name in ["legacy-continuous.jsonl","legacy-reset.jsonl"]:
    plain=HERE/name
    if plain.exists():
        cache=ROOT/".cache/ma-cross-research-output";cache.mkdir(parents=True,exist_ok=True)
        plain.replace(cache/name)
hashed = [SOURCE] + [p for p in HERE.iterdir() if p.is_file() and not p.name.startswith(".") and p.name != "validation.json"]
hashed += list((ROOT / "src-tauri/src/commands/strategy_preview").glob("*.rs"))
hashed += list((ROOT / "src-tauri/src/trading/simulation").rglob("*.rs"))
hashed += [ROOT / "src-tauri/src/trading/simulation.rs", ROOT / "src-tauri/src/commands/strategy_preview.rs"]
hashed += list((ROOT / "src-tauri/src/trading/strategy").rglob("*.rs"))
hashed += [ROOT / "src-tauri/src/trading/strategy.rs",ROOT / "Cargo.lock",ROOT / "Cargo.toml",ROOT / "src-tauri/Cargo.toml"]
validation = {
    "baselineCommit":"327739a", "unchangedStrategyFullReportAndSignalParityRuns":1224, "maCrossDefaultReportAndSignalParityRuns":102,
    "continuousRuns":48, "alternatePartitionEquivalenceRuns":48, "separateQuarterResetRuns":576,
    "independentArithmeticChecks":checks,
    "priorDailyPredicateChecks":filter_predicate_checks,
    "archives":archives,
    "scope":"Exact factorial coverage, next observed open/directional slippage, recorded accepted quantities, cash/exposure/terminal and causal market PnL. No strategy signal reimplementation.",
    "sha256":{str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(set(hashed))},
}
(HERE / "validation.json").write_text(json.dumps(validation, indent=2, ensure_ascii=False) + "\n")
print(json.dumps({k:v for k,v in validation.items() if k != "sha256"}))
