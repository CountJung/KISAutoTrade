"""Normalize public chart data and declare windows before strategy evaluation.

Run from the repository root; raw files remain in the ignored project cache.
No credentials or account files are used. This is data preparation, not a
second implementation of the trading strategies.
"""

import hashlib
import json
from collections import Counter
from datetime import datetime, time, timezone
from decimal import Decimal, ROUND_HALF_UP
from pathlib import Path
from zoneinfo import ZoneInfo

ROOT = Path(__file__).resolve().parents[3]
CACHE = ROOT / ".cache/research/strategy-review-20261018"
OUTPUT = Path(__file__).parent


def digest(data):
    return hashlib.sha256(data).hexdigest()


def price(value):
    return format(Decimal(str(value)).quantize(Decimal("0.01"), rounding=ROUND_HALF_UP), ".2f")


def read_dataset(symbol, interval):
    path = CACHE / f"{symbol}-{interval}-raw.json"
    raw = path.read_bytes()
    payload = json.loads(raw)
    assert payload["chart"]["error"] is None
    data = payload["chart"]["result"][0]
    meta = data["meta"]
    assert meta["symbol"] == symbol and meta["currency"] == "USD"
    assert meta["exchangeTimezoneName"] == "America/New_York"
    ny = ZoneInfo(meta["exchangeTimezoneName"])
    kst = ZoneInfo("Asia/Seoul")
    quotes = data["indicators"]["quote"][0]
    rows, excluded, session_counts = [], Counter(), Counter()
    for i, epoch in enumerate(data["timestamp"]):
        moment = datetime.fromtimestamp(epoch, timezone.utc)
        local = moment.astimezone(ny)
        # Freeze completed sessions on/before the declared research cutoff.
        if local.strftime("%Y%m%d") > "20261016":
            excluded["afterCutoff"] += 1
            continue
        values = [quotes[key][i] for key in ["open", "high", "low", "close", "volume"]]
        if any(value is None for value in values):
            excluded["nullOHLCV"] += 1
            continue
        o, h, l, c, volume = values
        assert 0 < l <= min(o, c) <= max(o, c) <= h and volume >= 0
        if interval == "1m" and not time(9, 30) <= local.time() < time(16):
            excluded["outsideNormalRegularSession"] += 1
            continue
        stamp = local.strftime("%Y%m%d") if interval == "1d" else moment.astimezone(kst).strftime("%Y%m%d%H%M%S")
        rows.append(dict(date=stamp, open=price(o), high=price(h), low=price(l), close=price(c), volume=str(int(volume))))
        session_counts[local.strftime("%Y%m%d")] += 1
    assert rows and len({row["date"] for row in rows}) == len(rows)
    assert rows == sorted(rows, key=lambda row: row["date"])
    metadata = {
        "sourceUrl": f"https://query1.finance.yahoo.com/v8/finance/chart/{symbol}?range={'5y' if interval == '1d' else '7d'}&interval={interval}" + ("&includePrePost=false" if interval == "1m" else ""),
        "retrievedAtUtc": datetime.fromtimestamp(path.stat().st_mtime, timezone.utc).isoformat(),
        "rawSha256": digest(raw),
        "sourceTimezone": meta["exchangeTimezoneName"],
        "normalizedTimezone": meta["exchangeTimezoneName"] if interval == "1d" else "Asia/Seoul",
        "timestampSemantics": "Provider bar start; daily date is exchange-local date; minute start converted from UTC, no synthetic observations",
        "priceSemantics": "Provider chart OHLC rounded to USD cents; adjusted-close field ignored; corporate-action adjustment and total-return correctness not independently verified",
        "dividendsAndFxChangesIncluded": False,
        "sourceBars": len(data["timestamp"]),
        "retainedBars": len(rows),
        "excluded": dict(excluded),
        "sessionCounts": dict(sorted(session_counts.items())),
        "normalSessionFilter": "09:30 <= America/New_York time < 16:00" if interval == "1m" else None,
        "calendarCoverageVerified": False,
        "normalizedSha256": digest(json.dumps(rows, sort_keys=True, separators=(",", ":")).encode()),
    }
    if interval == "1m":
        metadata["limitations"] = [
            "Seven recent sessions only; does not satisfy SOXQ-P1-05 July-October 3-month coverage",
            "No zero-volume or missing minutes invented; holiday/early-close and source completeness not certified",
            "Engine retains fixed KST session model and minute-start labels; no live 10-second or intrabar execution claim",
        ]
    return {"symbol": symbol, "candles": rows, "metadata": metadata}


def main():
    datasets = [read_dataset(symbol, "1d") for symbol in ["SOXQ", "SPY", "QQQ"]]
    windows = []
    for dataset in datasets:
        rows, symbol = dataset["candles"], dataset["symbol"]
        groups = {}
        for index, row in enumerate(rows):
            year, month = int(row["date"][:4]), int(row["date"][4:6])
            if year < 2023:
                continue
            quarter = (month - 1) // 3 + 1
            groups.setdefault((year, quarter), []).append(index)
        for (year, quarter), indices in sorted(groups.items()):
            start, end = indices[0], indices[-1] + 1
            assert start >= 252 and 0 < end - start <= 248
            windows.append(dict(id=f"{symbol}-{year}Q{quarter}", symbol=symbol,
                                historyStart=start - 252, evaluationStart=start,
                                evaluationEndExclusive=end, kind="quarter",
                                partial=year == 2026 and quarter == 4))
        start, end = len(rows) - 248, len(rows)
        assert start >= 252
        windows.append(dict(id=f"{symbol}-latest248", symbol=symbol,
                            historyStart=start - 252, evaluationStart=start,
                            evaluationEndExclusive=end, kind="latestLong", partial=False))
    minute = read_dataset("SOXQ", "1m")
    first_session = min(minute["metadata"]["sessionCounts"])
    minute["dailyContext"] = [row for row in datasets[0]["candles"] if row["date"] < first_session][-252:]
    assert len(minute["dailyContext"]) == 252
    payload = {
        "researchId": "strategy-review-20261018", "cutoffDate": "2026-10-16",
        "universeDeclaredBeforeRuns": ["SOXQ", "SPY", "QQQ"],
        "datasets": datasets, "windows": windows, "minuteDataset": minute,
        "assumptions": {"targetEntryRatio": 0.15, "costBps": [10, 20],
                        "exchangeRateKrw": 1450, "initialCapitalKrw": 10000000},
        "designLimitations": [
            "Historical diagnostic, not a preregistered prospective or unseen out-of-sample test",
            "Independent cash/position reset per window; no compound annual portfolio return",
            "Universe selected for cross-asset diagnostic, not a representative or survivorship-free stock universe",
            "same-close fills, fixed FX, no dividends, no provider latency or intrabar stops",
            "15% target quantity fixed at prior close including 10bp fee/slippage buffer; same quantity at both costs",
            "Quarter results are summarized separately from overlapping latest248 windows and partial Q4",
        ],
    }
    output = OUTPUT / "input.json"
    output.write_text(json.dumps(payload, ensure_ascii=False, separators=(",", ":")) + "\n")
    print(json.dumps({"windows": len(windows), "dailyBars": {d["symbol"]: len(d["candles"]) for d in datasets},
                      "minuteBars": len(minute["candles"]), "minuteSessions": minute["metadata"]["sessionCounts"],
                      "inputSha256": digest(output.read_bytes())}, indent=2))


if __name__ == "__main__":
    main()
