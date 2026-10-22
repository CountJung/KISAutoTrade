"""Summarize archived production results; no RSI/strategy signal implementation."""
import csv
import gzip
import json
from collections import Counter
from pathlib import Path

HERE = Path(__file__).resolve().parent
rows = [json.loads(line) for name in ['continuous-results.jsonl.gz', 'reset-results.jsonl.gz']
        for line in gzip.decompress((HERE / name).read_bytes()).splitlines()]
key = lambda row: (row['kind'], row['config']['symbol'], row['costBps'], row.get('quarter'))
owned = {key(row): row for row in rows if row['variant'] == 'owned'}
summary = list(csv.DictReader((HERE / 'summary.csv').open()))
result = {'byVariant': {}, 'longestOwnedCloseObservationsBySymbol': {}, 'inertHold63ExactReportAndSignals': 0}
for variant in sorted({row['variant'] for row in rows}):
    selected = [row for row in rows if row['variant'] == variant]
    counters = Counter()
    for row in selected:
        data = row['result']; report = data['backtest']; sm = report['summary']
        counters['runs'] += 1
        counters['readyAtStart'] += data['preparation']['readyAtStart'] is True
        counters['noSignals'] += not data['signals']
        counters['noCompletedExit'] += sm['completedTrades'] == 0
        counters['terminalHeld'] += report['accounting']['remainingQuantity'] > 0
        counters['stopSignals'] += sum(signal['side'] == 'sell' and 'RSI 손절' in signal['reason'] for signal in data['signals'])
        counters['capSignals'] += sum(signal['side'] == 'sell' and 'RSI 보유 관측' in signal['reason'] for signal in data['signals'])
        if variant == 'hold63':
            prior = owned[key(row)]['result']
            same = all(data[field] == prior[field] for field in ['backtest', 'signals'])
            result['inertHold63ExactReportAndSignals'] += same
        if variant == 'owned':
            qty = age = longest = 0
            fills = [trade for trade in report['trades'] if trade['status'] == 'filled']
            for candle in data['candles']:
                for trade in (trade for trade in fills if trade['fillAt']['barTime'] == candle['date']):
                    if trade['side'] == 'buy':
                        if qty == 0:
                            age = 0
                        qty += trade['quantity']
                    else:
                        qty -= trade['quantity']
                if qty > 0:
                    age += 1
                    longest = max(longest, age)
            symbol = row['config']['symbol']
            result['longestOwnedCloseObservationsBySymbol'][symbol] = max(longest, result['longestOwnedCloseObservationsBySymbol'].get(symbol, 0))
    sr = [row for row in summary if row['variant'] == variant]
    result['byVariant'][variant] = dict(counters, **{field: sum(int(row[field]) for row in sr)
        for field in ['flatSellSignals', 'heldBuySignals', 'rawSignals', 'blockedOrders']})
# Cost sign flips count matching records, including continuous/reset as separate experiment kinds.
comparisons = {}
for row in summary:
    comparisons.setdefault((row['kind'], row['variant'], row['symbol'], row['quarter']), {})[row['costBps']] = float(row['returnPct'])
result['costSignFlipRecords'] = [dict(zip(['kind', 'variant', 'symbol', 'quarter'], k), returns=v)
    for k, v in comparisons.items() if v['10'] * v['20'] < 0]
result['scope'] = 'Recorded production fills and close observations only; no missing bars/calendar inference, no independently generated RSI decisions. Aggregate totals combine continuous/reset experiments for mechanics counts only, never portfolio returns or independent samples.'
(HERE / 'policy-observations.json').write_text(json.dumps(result, indent=2, ensure_ascii=False) + '\n')
print(json.dumps(result, ensure_ascii=False))
