"""Compare frozen v7 traces with v8; never rewrite the original study.

Usage: python3 compare.py <v7.jsonl> <v8.jsonl>
Traces stay in .cache; this directory retains hashes and measured differences.
"""

import copy
import hashlib
import json
import statistics
import sys
from collections import Counter
from pathlib import Path


def load(path):
    return [json.loads(line) for line in Path(path).open() if line.strip()]


def key(row):
    return (row.get("window", {}).get("id", "minute"), row["kind"],
            row["strategyId"], row.get("variant", ""), row["costBps"])


def report(row):
    return row.get("preview", {}).get("backtest", row.get("backtest"))


def comparable(row):
    result = copy.deepcopy(row)
    preview = result.get("preview")
    if preview:
        preview.pop("generatedAt", None)
        for field in ("engineVersion", "strategyVersion", "inputHash"):
            preview.get("replay", {}).pop(field, None)
    backtest = report(result)
    if backtest:
        backtest.pop("atrPreparation", None)
    return result


def without_generation_time(value):
    if isinstance(value, dict):
        return {k: without_generation_time(v) for k, v in value.items() if k != "generatedAt"}
    if isinstance(value, list):
        return [without_generation_time(v) for v in value]
    return value


def main():
    before_path, after_path = map(Path, sys.argv[1:3])
    before_rows, after_rows = load(before_path), load(after_path)
    before = {key(row): row for row in before_rows}
    after = {key(row): row for row in after_rows}
    assert len(before) == len(before_rows) and len(after) == len(after_rows)
    assert before.keys() == after.keys(), "research coverage changed"
    changes = []
    for identity, previous in before.items():
        current = after[identity]
        if comparable(previous) == comparable(current):
            continue
        assert identity[2] == "fifty_two_week_high", identity
        changes.append({
            "key": identity,
            "beforeSummary": report(previous)["summary"],
            "afterSummary": report(current)["summary"],
            "summaryChanged": report(previous)["summary"] != report(current)["summary"],
            "diagnosticsChanged": previous["diagnostics"] != current["diagnostics"],
        })
    complete = [row for row in after_rows if row["kind"] == "strategy"
                and row["strategyId"] == "fifty_two_week_high"
                and row["window"]["kind"] == "quarter" and not row["window"]["partial"]]
    stats = []
    for cost in (10, 20):
        for label, mapping in (("v7", before), ("v8", after)):
            rows = [mapping[key(row)] for row in complete if row["costBps"] == cost]
            summaries = [report(row)["summary"] for row in rows]
            stats.append({"engine": label, "costBpsEachFeeAndSlippage": cost,
                          "windows": len(rows),
                          "medianReturnPct": statistics.median(s["cumulativeReturnPct"] for s in summaries),
                          "worstReturnPct": min(s["cumulativeReturnPct"] for s in summaries),
                          "worstMddPct": max(s["mddPct"] for s in summaries),
                          "completedTrades": sum(s["completedTrades"] for s in summaries),
                          "windowsWithOpenPosition": sum(row["diagnostics"]["heldQuantity"] > 0 for row in rows)})
    output = {
        "scope": "same frozen input; volatility sizing disabled; reset portfolios; descriptive comparison",
        "beforeSha256": hashlib.sha256(before_path.read_bytes()).hexdigest(),
        "afterSha256": hashlib.sha256(after_path.read_bytes()).hexdigest(),
        "counts": dict(Counter(row["kind"] for row in after_rows)),
        "sameComparableRows": len(before) - len(changes),
        "changedComparableRows": len(changes),
        "changedSummaryRows": sum(change["summaryChanged"] for change in changes),
        "excludedFields": ["preview.generatedAt", "preview.replay.engineVersion",
                           "preview.replay.strategyVersion", "preview.replay.inputHash",
                           "backtest.atrPreparation"],
        "highStrategyCompleteQuarterStats": stats,
        "changes": changes,
    }
    if len(sys.argv) > 3:
        repeat_path = Path(sys.argv[3])
        repeat = load(repeat_path)
        assert without_generation_time(after_rows) == without_generation_time(repeat)
        output["repeatMatchesExceptGeneratedAt"] = True
        output["repeatSha256"] = hashlib.sha256(repeat_path.read_bytes()).hexdigest()
    target = Path(__file__).with_name("comparison.json")
    target.write_text(json.dumps(output, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps({k: output[k] for k in ("counts", "sameComparableRows", "changedComparableRows",
                                            "highStrategyCompleteQuarterStats")}, ensure_ascii=False))


if __name__ == "__main__":
    main()
