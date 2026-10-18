"""Summarize production-engine JSONL without implementing trading logic.

Usage: python3 analyze.py <full-results.jsonl> [--check-against other.jsonl]
Full traces remain in the ignored project cache. Tracked compact rows retain
all reported measurements and a SHA-256 of the complete deterministic row.
"""

import argparse
import hashlib
import json
import statistics
from collections import Counter, defaultdict
from pathlib import Path

HERE = Path(__file__).resolve().parent


def canonical(value):
    if isinstance(value, dict):
        return {key: canonical(item) for key, item in value.items() if key != "generatedAt"}
    if isinstance(value, list):
        return [canonical(item) for item in value]
    return value


def sha(value):
    return hashlib.sha256(json.dumps(canonical(value), sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode()).hexdigest()


def load(path):
    with Path(path).open() as source:
        return [json.loads(line) for line in source if line.strip()]


def key(row):
    return (row.get("window", {}).get("id", "minute"), row["kind"], row["strategyId"], row.get("variant", ""), row["costBps"])


def compact(row):
    result = {key: value for key, value in row.items() if key not in ["preview", "backtest", "signals"]}
    result["fullResultSha256ExceptGeneratedAt"] = sha(row)
    if "preview" in row:
        preview = row["preview"]
        result.update(replay=preview["replay"], preparation=preview.get("preparation"),
                      summary=preview["backtest"]["summary"], assumptions=preview["backtest"]["assumptions"])
    elif "backtest" in row:
        result.update(summary=row["backtest"]["summary"], assumptions=row["backtest"]["assumptions"])
    return result


def stats(rows):
    returns = [row["summary"]["cumulativeReturnPct"] for row in rows]
    blocks = Counter()
    for row in rows:
        blocks.update(row["diagnostics"]["blockedReasons"])
    return {
        "windows": len(rows), "positiveWindows": sum(value > 0 for value in returns),
        "medianReturnPct": round(statistics.median(returns), 4),
        "worstReturnPct": min(returns), "bestReturnPct": max(returns),
        "worstMddPct": max(row["summary"]["mddPct"] for row in rows),
        "completedTrades": sum(row["summary"]["completedTrades"] for row in rows),
        "windowsWithNoCompletedTrades": sum(row["summary"]["completedTrades"] == 0 for row in rows),
        "windowsWithOpenPosition": sum(row["diagnostics"]["heldQuantity"] > 0 for row in rows),
        "positiveWindowsWithRealizedLoss": sum(row["summary"]["cumulativeReturnPct"] > 0 and row["diagnostics"]["realizedPnlKrw"] < 0 for row in rows),
        "signalCount": sum(row["summary"]["signalCount"] for row in rows),
        "blockedOrderCount": sum(row["summary"]["blockedOrderCount"] for row in rows),
        "blockedReasons": dict(blocks),
    }


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("full_results")
    parser.add_argument("--check-against")
    args = parser.parse_args()
    rows = load(args.full_results)
    assert len({key(row) for row in rows}) == len(rows), "duplicate result key"
    if args.check_against:
        other = load(args.check_against)
        assert {key(row): sha(row) for row in rows} == {key(row): sha(row) for row in other}
    small = [compact(row) for row in rows]
    with (HERE / "results.jsonl").open("w") as target:
        for row in small:
            target.write(json.dumps(row, ensure_ascii=False, separators=(",", ":")) + "\n")
    groups = defaultdict(list)
    for row in small:
        if row["kind"] == "strategy" and row["window"]["kind"] == "quarter" and not row["window"]["partial"]:
            groups[(row["strategyId"], row["costBps"])].append(row)
    benchmarks = { (row["window"]["id"], row["costBps"]): row for row in small if row["kind"] == "benchmark" }
    baseline = { (row["window"]["id"], row["strategyId"]): row for row in small if row["kind"] == "strategy" and row["costBps"] == 10 }
    summaries = []
    for (strategy, cost), subset in sorted(groups.items()):
        summary = dict(strategyId=strategy, costBps=cost, **stats(subset))
        differences = [row["summary"]["cumulativeReturnPct"] - benchmarks[(row["window"]["id"], cost)]["summary"]["cumulativeReturnPct"] for row in subset]
        summary["medianExcessVsSameQuantityPct"] = round(statistics.median(differences), 4)
        summary["windowsAboveSameQuantityBenchmark"] = sum(value > 0 for value in differences)
        if cost == 20:
            pairs = [(baseline[(row["window"]["id"], strategy)], row) for row in subset]
            summary["positiveToNonpositiveAfterCostStress"] = sum(a["summary"]["cumulativeReturnPct"] > 0 >= b["summary"]["cumulativeReturnPct"] for a, b in pairs)
            summary["medianCostStressDeltaPct"] = round(statistics.median(b["summary"]["cumulativeReturnPct"] - a["summary"]["cumulativeReturnPct"] for a, b in pairs), 4)
        summary["perSymbol"] = {symbol: stats([row for row in subset if row["symbol"] == symbol]) for symbol in sorted({row["symbol"] for row in subset})}
        summaries.append(summary)
    summary = {
        "scope": "45 complete asset-quarter windows (2023Q1-2026Q3); correlated assets, reset portfolios, descriptive statistics only",
        "counts": dict(Counter(row["kind"] for row in small)),
        "repeatMatchesExceptGeneratedAt": bool(args.check_against),
        "inputSha256": hashlib.sha256((HERE / "input.json").read_bytes()).hexdigest(),
        "resultsSha256": hashlib.sha256((HERE / "results.jsonl").read_bytes()).hexdigest(),
        "strategies": summaries,
        "partialQuarterAndLatestLong": [row for row in small if row.get("window", {}).get("partial") or row.get("window", {}).get("kind") == "latestLong"],
        "lthDiagnostics": [row for row in small if row["kind"] == "lthDiagnostic"],
    }
    (HERE / "summary.json").write_text(json.dumps(summary, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps({"counts": summary["counts"], "repeatMatches": summary["repeatMatchesExceptGeneratedAt"]}, ensure_ascii=False))
    for row in summaries:
        if row["costBps"] == 10:
            print(row["strategyId"], row["medianReturnPct"], row["worstReturnPct"], row["worstMddPct"], row["completedTrades"], row["blockedOrderCount"], row["windowsAboveSameQuantityBenchmark"])


if __name__ == "__main__":
    main()
