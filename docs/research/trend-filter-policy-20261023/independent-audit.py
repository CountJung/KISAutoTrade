"""독립 체결 장부 감사. 신호 생성·전략 재구현·Rust 실행은 하지 않는다."""
import gzip
import hashlib
import itertools
import json
import math
import subprocess
from collections import Counter, defaultdict
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
CHECKS = Counter()
ERRORS = []
CONTEXT = "입력"


def check(ok, label, detail=None):
    CHECKS[label] += 1
    if not ok:
        ERRORS.append({"run": CONTEXT, "check": label, "detail": detail})


def close(actual, expected, label, tolerance=0.021):
    check(math.isfinite(actual) and abs(actual - expected) <= tolerance,
          label, {"reported": actual, "independent": expected, "tolerance": tolerance})


def read_rows(stem):
    path = HERE / (stem + ".jsonl")
    raw = path.read_bytes() if path.exists() else gzip.decompress(
        (HERE / (stem + ".jsonl.gz")).read_bytes())
    return [json.loads(line) for line in raw.splitlines()], hashlib.sha256(raw).hexdigest()


design_bytes = (HERE / "design.json").read_bytes()
source_bytes = (HERE.parent / "strategy-review-20261018/input.json").read_bytes()
check(hashlib.sha256(design_bytes).hexdigest() ==
      "f2a3f801da05d77544580b3fabc265197bdc26cd7072859df9f2f4428262040f", "설계 SHA")
design = json.loads(design_bytes)
check(hashlib.sha256(source_bytes).hexdigest() == design["sourceSha256"], "동결 입력 SHA")
source = json.loads(source_bytes)
datasets = {d["symbol"]: d["candles"] for d in source["datasets"]}
continuous, continuous_sha = read_rows("continuous-results")
resets, resets_sha = read_rows("reset-results")
rows = continuous + resets
variants = {v["id"]: v for v in design["variants"]}
configs = {c["symbol"]: c for c in design["configs"]}
quarters = [f"{y}Q{q}" for y in range(2023, 2026) for q in range(1, 5)]
key = lambda r: (r["kind"], r["variant"], r["config"]["symbol"], r["costBps"], r.get("quarter"))
expected = set(itertools.product(["continuous"], variants, configs, design["costBps"], [None]))
expected |= set(itertools.product(["quarterReset"], variants, configs, design["costBps"], quarters))
check(len(rows) == len(set(map(key, rows))) == 390 and set(map(key, rows)) == expected,
      "30연속·360별도분기 유일 조합")
run = json.loads((HERE / "run-summary.json").read_text())
check((run["continuousRuns"], run["partitionEquivalenceRuns"],
       run["independentQuarterResetRuns"], run["unchangedStrategyFullReportAndSignalParityRuns"])
      == (30, 30, 360, 1122), "실행 요약 범위")
partition = {(p["variant"], p["symbol"], p["costBps"]): p for p in run["partitions"]}
check(len(partition) == len(run["partitions"]) == 30, "대체 분할 유일 조합")

# 부모의 archive 검증 주장을 Git object bytes와 독립 대조한다.
archive = json.loads((HERE / "baseline-source-validation.json").read_text())
check(archive["commit"] == design["baselineCommit"], "기준 commit")
for entry in archive["files"]:
    original = subprocess.check_output(["git", "show", f'{archive["commit"]}:{entry["path"]}'], cwd=ROOT)
    check(hashlib.sha256(original).hexdigest() == entry["sha256"], "기준 Git object SHA", entry["path"])

summaries = []
for row in rows:
    CONTEXT = "|".join(map(str, key(row)))
    result = row["result"]
    b = result["backtest"]
    sm, ac, assumptions = b["summary"], b["accounting"], b["assumptions"]
    cfg = configs[row["config"]["symbol"]]
    params = dict(cfg["params"], **variants[row["variant"]]["params"])
    check(row["config"]["params"] == params and
          row["config"]["orderQuantity"] == cfg["orderQuantity"], "선언 수량·단독 후보")
    start = row.get("evaluationStart", cfg["evaluationStart"])
    end = row.get("evaluationEndExclusive", cfg["evaluationEndExclusive"])
    history = row.get("historyStart", cfg["historyStart"])
    bars = datasets[cfg["symbol"]][start:end]
    check(result["candles"] == bars and start - history == 252, "동결 평가·이전252봉")
    check(result["replay"]["warmupCount"] == 252 and result["preparation"]["readyAtStart"] and
          result["preparation"]["unreadyEvaluationBars"] == 0, "준비·warmup")
    check(result["replay"]["engineVersion"] ==
          ("strategy-replay-v21" if row["variant"] == "legacy" else "strategy-replay-v22"), "실행 engine")
    if row["kind"] == "continuous":
        check(len(bars) == 752 and bars[0]["date"] == "20230103" and bars[-1]["date"] == "20251231",
              "연속 평가 경계")
        check(partition[(row["variant"], cfg["symbol"], row["costBps"])]["inputHash"] ==
              result["replay"]["inputHash"], "분할 inputHash 기록")
    capital, fx = assumptions["initialCapitalKrw"], assumptions["exchangeRateKrw"]
    rate = row["costBps"] / 10000
    check(capital == 10000000 and fx == 1450 and assumptions["feeBps"] == row["costBps"] and
          assumptions["slippageBps"] == row["costBps"] and assumptions["taxBps"] == 0 and
          assumptions["executionModel"] == "nextOpen" and assumptions["maxPositionRatio"] == 0.3 and
          not assumptions["fxScenario"] and not assumptions["dividendScenario"] and
          assumptions["stopLossPct"] == assumptions["takeProfitPct"] == 0, "고정 비용·FX·체결·보호")
    fills = [t for t in b["trades"] if t["status"] == "filled"]
    by_date = defaultdict(list)
    for fill in fills:
        by_date[fill["fillAt"]["barTime"]].append(fill)
    check(set(by_date).issubset({c["date"] for c in bars}), "체결 평가 범위")
    qty = 0
    cash = capital
    basis = buy_fee_basis = realized = fees = slippage = gross_turnover = 0.0
    realized_price = path_market = 0.0
    qty_before = {}
    qty_after = {}
    values = []
    weighted_equity = []
    equities = []
    drawdowns = []
    completed = []
    peak = capital
    prev = float(datasets[cfg["symbol"]][start - 1]["close"])
    check(len(b["equityCurve"]) == len(bars), "곡선 관측수")
    for index, candle in enumerate(bars):
        date = candle["date"]
        op, cp = float(candle["open"]), float(candle["close"])
        qty_before[date] = qty
        path_market += qty * (op - prev) * fx
        for t in by_date[date]:
            q = t["quantity"]
            check(isinstance(q, int) and q > 0, "실체결 정수 수량")
            check(index > 0 and t["signalAvailableAt"] == {"barTime": bars[index - 1]["date"], "phase": "close"}
                  and t["fillAt"] == {"barTime": date, "phase": "open"} and t["time"] == date,
                  "직전 종가 공개→다음 관측 시가")
            fill_price = op * (1 + rate if t["side"] == "buy" else 1 - rate)
            gross = q * fill_price * fx
            fee = gross * rate
            slip = q * abs(fill_price - op) * fx
            close(t["executionReferencePrice"], op, "체결 기준 시가")
            close(t["fillPrice"], fill_price, "방향별 slippage", 1e-8)
            close(t["grossAmountKrw"], gross, "실수량 거래금액")
            close(t["feeKrw"], fee, "방향별 fee")
            close(t["slippageKrw"], slip, "slippage 원가")
            close(t["taxKrw"], 0, "세금0")
            close(t["costKrw"], fee + slip, "기록 총비용")
            if t["side"] == "buy":
                cash -= gross + fee
                qty += q
                basis += gross
                buy_fee_basis += fee
            else:
                check(0 < q <= qty, "잔량 이내 청산")
                fraction = q / qty
                pnl = gross - fee - (basis + buy_fee_basis) * fraction
                price_pnl = gross - basis * fraction
                close(t["realizedPnlKrw"], pnl, "체결 실현 순손익")
                completed.append(pnl)
                realized += pnl
                realized_price += price_pnl
                cash += gross - fee
                basis *= 1 - fraction
                buy_fee_basis *= 1 - fraction
                qty -= q
            fees += fee
            slippage += slip
            gross_turnover += gross
        qty_after[date] = qty
        path_market += qty * (cp - op) * fx
        notional = qty * cp * fx
        equity = cash + notional
        point = b["equityCurve"][index]
        check(point["time"] == date, "평가 곡선 날짜")
        close(point["equityKrw"], equity, "봉별 현금+평가액")
        # 생산 peak는 직전까지 반올림해 게시한 equity를 사용한다.
        peak = max(peak, point["equityKrw"])
        dd = (peak - point["equityKrw"]) / peak * 100
        close(point["drawdownPct"], dd, "봉별 drawdown", 0.00011)
        drawdowns.append(dd)
        values.append(notional)
        weighted_equity.append(notional / equity * 100)
        equities.append(equity)
        prev = cp
    last_value = qty * float(bars[-1]["close"]) * fx
    residual = last_value - basis - buy_fee_basis
    final = cash + last_value
    terminal_slip = last_value * rate
    terminal_fee = (last_value - terminal_slip) * rate
    terminal = final - terminal_slip - terminal_fee
    for field, value in {"realizedPnlKrw": realized, "realizedPricePnlKrw": realized_price,
                         "realizedFxPnlKrw": 0, "residualFxPnlKrw": 0,
                         "residualPricePnlKrw": last_value - basis, "residualUnrealizedPnlKrw": residual,
                         "remainingCostBasisKrw": basis, "remainingBuyFeeKrw": buy_fee_basis,
                         "totalFeeKrw": fees, "totalSlippageKrw": slippage, "totalTaxKrw": 0,
                         "terminalExitSlippageKrw": terminal_slip, "terminalExitFeeKrw": terminal_fee,
                         "terminalExitTaxKrw": 0, "terminalLiquidationEquityKrw": terminal}.items():
        close(ac[field], value, "장부:" + field)
    check(ac["remainingQuantity"] == qty, "기말 실제 잔량")
    close(final - capital, realized + residual, "총자산=실현+잔여")
    close(sm["finalEquityKrw"], final, "기말 순자산")
    close(sm["cumulativeReturnPct"], (final / capital - 1) * 100, "순수익률", 0.00011)
    close(ac["terminalLiquidationReturnPct"], (terminal / capital - 1) * 100, "기말 청산 순수익률", 0.00011)
    close(sm["mddPct"], max(drawdowns), "최대 drawdown", 0.00011)
    close(sm["turnoverPct"], gross_turnover / capital * 100, "실체결 회전율", 0.00011)
    held = sum(v > 0 for v in values)
    close(sm["exposurePct"], held / len(bars) * 100, "보유 봉 비율", 0.00011)
    close(ac["averageInvestedCapitalPct"], sum(values) / len(values) / capital * 100, "평균 자본 노출", 0.00011)
    close(ac["averageInvestedNotionalKrw"], sum(values) / len(values), "평균 투자금액")
    close(ac["averageInvestedEquityPct"], sum(weighted_equity) / len(values), "평균 equity 노출", 0.00011)
    close(ac["peakInvestedCapitalPct"], max(values) / capital * 100, "최대 자본 노출", 0.00011)
    close(ac["peakInvestedEquityPct"], max(weighted_equity), "최대 equity 노출", 0.00011)
    check((sm["completedTrades"], sm["winningTrades"], sm["losingTrades"]) ==
          (len(completed), sum(p > 0 for p in completed), sum(p < 0 for p in completed)), "청산·승리·손실수")
    statuses = Counter(t["status"] for t in b["trades"])
    check(sm["filledOrderCount"] == statuses["filled"] and sm["blockedOrderCount"] == statuses["blocked"]
          and sm["unfilledOrderCount"] == statuses["unfilled"] and sm["signalCount"] == len(result["signals"]),
          "신호·체결·차단·미체결수")
    for t in b["trades"]:
        if t["status"] != "filled":
            check(t.get("fillAt") is None and t["fillPrice"] is None and t["grossAmountKrw"] == 0 and
                  t["feeKrw"] == t["slippageKrw"] == 0, "비체결 장부 불변")
        if t["status"] == "unfilled":
            check(t["signalAvailableAt"] == {"barTime": bars[-1]["date"], "phase": "close"}, "마지막 pending")
    for signal in result["signals"]:
        check((signal["side"] == "buy") == (qty_after[signal["time"]] == 0), "실소유 방향 신호")
    reference = b["comparison"]["sameExecutionPathCostReference"]
    close(reference["restoredCostKrw"], fees + slippage, "경로 비용 환급")
    close(reference["finalEquityKrw"], final + fees + slippage, "경로 gross 참조")
    close(path_market, final - capital + fees + slippage, "실수량 gap+봉내 시장손익")
    # 단순 보유는 기록된 실제 체결량으로 독립 정산한다. 0주는 노출 비교 불가다.
    bh = b["comparison"]["sameInputQuantityBuyHold"]
    bhq = bh["actualEntryQuantity"]
    check(bhq in (0, cfg["orderQuantity"]) and bh["requestedQuantity"] == cfg["orderQuantity"] and
          bh["remainingQuantity"] == bhq, "단순보유 요청·실체결량")
    bh_cost = bhq * float(bars[1]["open"]) * (1 + rate) * fx * (1 + rate)
    bh_values = [0] + [bhq * float(c["close"]) * fx for c in bars[1:]]
    bh_curve = [capital] + [capital - bh_cost + value for value in bh_values[1:]]
    bh_final = bh_curve[-1]
    close(bh["finalEquityKrw"], bh_final, "단순보유 실수량 순자산")
    close(bh["cumulativeReturnPct"], (bh_final / capital - 1) * 100, "단순보유 순수익", 0.00011)
    close(bh["residualUnrealizedPnlKrw"], bh_final - capital, "단순보유 잔여 손익")
    close(bh["holdingBarPct"], (len(bars) - 1) / len(bars) * 100 if bhq else 0,
          "단순보유 보유 봉", 0.00011)
    close(bh["averageInvestedCapitalPct"], sum(bh_values) / len(bars) / capital * 100,
          "단순보유 실제 노출", 0.00011)
    bh_peak = capital
    bh_dd = []
    for value in bh_curve:
        bh_peak = max(bh_peak, value)
        bh_dd.append((bh_peak - value) / bh_peak * 100)
    close(bh["mddPct"], max(bh_dd), "단순보유 MDD", 0.00011)
    close(bh["terminalLiquidationReturnPct"],
          (bh_final - bh_values[-1] * (2 * rate - rate * rate) - capital) / capital * 100,
          "단순보유 기말 청산 비용", 0.00011)
    summaries.append({"kind": row["kind"], "variant": row["variant"], "symbol": cfg["symbol"],
                      "costBps": row["costBps"], "quarter": row.get("quarter"),
                      "netReturnPct": sm["cumulativeReturnPct"],
                      "grossPathReferenceReturnPct": path_market / capital * 100,
                      "mddPct": sm["mddPct"], "realizedPnlKrw": realized, "residualPnlKrw": residual,
                      "completedTrades": len(completed), "losingTrades": sum(p < 0 for p in completed),
                      "remainingQuantity": qty, "remainingCostBasisKrw": basis,
                      "remainingBuyFeeKrw": buy_fee_basis,
                      "remainingNativeAverageFromLedger": basis / qty / fx if qty else None,
                      "terminalExitCostKrw": terminal_slip + terminal_fee,
                      "terminalNetReturnPct": (terminal / capital - 1) * 100,
                      "holdingBarPct": sm["exposurePct"], "turnoverPct": sm["turnoverPct"],
                      "averageInvestedCapitalPct": ac["averageInvestedCapitalPct"],
                      "blockedReasons": dict(Counter(t["blockedReason"] for t in b["trades"] if t["status"] == "blocked")),
                      "profitProtectionFills": sum(t["side"] == "sell" and "수익 보호" in t["reason"] for t in fills),
                      "holdCapFills": sum(t["side"] == "sell" and "최대 보유 관측" in t["reason"] for t in fills),
                      "buyHoldActualQuantity": bhq, "buyHoldNetReturnPct": bh["cumulativeReturnPct"]})

CONTEXT = "대응 비교"
lookup = {key(r): r for r in rows}
owned_pairs = []
for r in rows:
    if r["variant"] == "owned":
        k = list(key(r)); k[1] = "legacy"
        prior = lookup[tuple(k)]
        exact = all(r["result"][f] == prior["result"][f] for f in ("backtest", "signals"))
        check(exact, "legacy→owned 전체 report·신호 동일", key(r))
        owned_pairs.append(exact)
deltas = json.loads((HERE / "flat-trend-filter-deltas.json").read_text())
check(len(deltas) == len({d["key"] for d in deltas}) == 102, "flat Trend 유일102창")
for delta in deltas:
    check(all(delta["prior" + part] == delta["current" + part]
              for part in ("ReportHash", "SignalsHash", "Summary", "Accounting")),
          "flat Trend 기준·현재 일치", delta["key"])
policy_activity = {}
for kind in ("continuous", "quarterReset"):
    for variant in variants:
        selected = [s for s in summaries if s["kind"] == kind and s["variant"] == variant]
        policy_activity[kind + "|" + variant] = {
            "runs": len(selected), "profitProtectionFills": sum(s["profitProtectionFills"] for s in selected),
            "holdCapFills": sum(s["holdCapFills"] for s in selected),
            "noCompletedExitRuns": sum(s["completedTrades"] == 0 for s in selected),
            "zeroQuantityBuyHoldRuns": sum(s["buyHoldActualQuantity"] == 0 for s in selected)}
inert_cap = sum(all(r["result"][f] == lookup[(r["kind"], "owned", r["config"]["symbol"], r["costBps"], r.get("quarter"))]["result"][f]
                    for f in ("backtest", "signals")) for r in rows if r["variant"] == "hold63")
reset_cap_equal = 0
reset_cap_pending = 0
for r in resets:
    if r["variant"] == "hold63":
        prior = lookup[(r["kind"], "owned", r["config"]["symbol"], r["costBps"], r.get("quarter"))]
        accepted = lambda row: [t for t in row["result"]["backtest"]["trades"] if t["status"] == "filled"]
        equal = (r["result"]["backtest"]["equityCurve"] == prior["result"]["backtest"]["equityCurve"]
                 and accepted(r) == accepted(prior))
        check(equal, "분기 보유63 실제 경제경로 동일", key(r))
        reset_cap_equal += equal
        reset_cap_pending += sum(t["status"] == "unfilled" and "최대 보유 관측" in t["reason"]
                                 for t in r["result"]["backtest"]["trades"])
output = {"status": "pass" if not ERRORS else "fail", "checks": sum(CHECKS.values()),
          "checksByCategory": dict(CHECKS), "errors": ERRORS,
          "recordedRuns": len(rows), "legacyOwnedExactPairs": sum(owned_pairs),
          "flatTrendPairs": len(deltas), "hold63ExactToOwnedRuns": inert_cap,
          "hold63ResetAcceptedEconomicPathEqualRuns": reset_cap_equal,
          "hold63ResetTerminalUnfilledCapOrders": reset_cap_pending,
          "decompressedSha256": {"continuous-results.jsonl": continuous_sha, "reset-results.jsonl": resets_sha},
          "baselineDependencyCaveat": archive["dependencyResolution"],
          "scope": "생산 원장 독립 산술 감사. 대체 분할 equality는 부모 Rust 실행기의 기록을 확인했으며 원장을 별도 재실행하지 않았다. 신호·체결 모델을 재구현하지 않았다.",
          "limits": ["과거에 관측한 상관3ETF·겹침 분기이며 독립 OOS가 아니다.",
                     "비용은 방향별 fee/slippage 각각10/20bp, 고정1450FX·배당0이다.",
                     "gross는 실제 경로 비용 환급 참조이며 비용0 전략 재실행이 아니다.",
                     "분기 reset은 별도 계좌이며 수익률을 합산·복리하지 않는다.",
                     "달력 coverage·원본 raw bytes·기업행사/배당 조정·실계좌 체결은 이번 감사에서 검증하지 않았다.",
                     "선택 보호 미발동은 효능 미평가이며0% 효능으로 해석하지 않는다."],
          "policyActivity": policy_activity, "runs": summaries}
(HERE / "independent-audit.json").write_text(json.dumps(output, ensure_ascii=False, indent=2) + "\n")
print(json.dumps({k: output[k] for k in ("status", "checks", "recordedRuns", "legacyOwnedExactPairs", "hold63ExactToOwnedRuns")}, ensure_ascii=False))
if ERRORS:
    print(json.dumps(ERRORS[:10], ensure_ascii=False))
    raise SystemExit(1)
