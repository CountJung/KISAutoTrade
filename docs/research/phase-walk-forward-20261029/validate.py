"""Independent recorded-fill and phase arithmetic; no strategy signal model.

Reads only the frozen research input and this declared research's results.
Writes summaries, lossless archives and hashes in this folder; never runs Rust.
"""
import collections
import csv
import gzip
import hashlib
import json
import math
from pathlib import Path
import statistics

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
SOURCE = ROOT / 'docs/research/strategy-review-20261018/input.json'
source_bytes = SOURCE.read_bytes()
design = json.loads((HERE / 'design.json').read_text())
assert hashlib.sha256(source_bytes).hexdigest() == design['sourceSha256']
source = json.loads(source_bytes)
run = json.loads((HERE / 'run-summary.json').read_text())
assert run['design'] == design, 'Executed design differs from fixed declaration'
assert (run['contextRuns'], run['evaluationRuns'], run['totalWalkForwardRuns'], run['pcCompatibilityRuns'], run['partitionCompatibilityRuns']) == (792, 792, 1584, 78, 6)
datasets = {d['symbol']: d['candles'] for d in source['datasets']}
folds = {(f['id'], s['symbol']): s for f in design['folds'] for s in f['symbols']}
strategies = {s['strategyId']: s['params'] for s in design['strategies']}
checks = collections.Counter()
maximum_money_error = 0.0
phase_count = 0


def near(actual, expected, label='money', tolerance=0.031):
    global maximum_money_error
    assert math.isfinite(actual) and math.isfinite(expected), (label, actual, expected)
    error = abs(actual - expected)
    assert error <= tolerance, (label, actual, expected, error)
    checks[label] += 1
    if tolerance == 0.031:
        maximum_money_error = max(maximum_money_error, error)


def load(name, folder=HERE):
    plain = folder / name
    if plain.exists():
        with plain.open() as f:
            for line in f:
                yield json.loads(line)
    else:
        with gzip.open(folder / (name + '.gz'), 'rt') as f:
            for line in f:
                yield json.loads(line)


def strip_interpretation(result):
    report = json.loads(json.dumps(result['backtest']))
    report.pop('overfitWarning', None)
    report.pop('executionWarnings', None)
    for p in report['phases']:
        p.pop('diagnostics', None)
    return {'backtest': report, 'signals': result['signals'], 'preparation': result['preparation']}


def validate_row(row):
    global phase_count
    symbol = row.get('symbol', row.get('config', {}).get('symbol'))
    result = row['result']
    report = result['backtest']
    summary = report['summary']
    account = report['accounting']
    assumptions = report['assumptions']
    start, end = row['evaluationStart'], row['evaluationEndExclusive']
    all_bars = datasets[symbol]
    bars = all_bars[start:end]
    assert start >= 252 and len(bars) > 0
    assert result['candles'] == bars
    assert result['replay']['engineVersion'] == design['plannedEngineVersion']
    assert result['replay']['warmupCount'] == 252
    assert result['preparation']['readyAtStart'] is True
    assert assumptions['executionModel'] == 'nextOpen'
    assert assumptions['initialCapitalKrw'] == 10_000_000 and assumptions['exchangeRateKrw'] == 1450
    assert assumptions['feeBps'] == assumptions['slippageBps'] == row['costBps']
    assert not assumptions['fxScenario'] and not assumptions['dividendScenario']
    assert assumptions['stopLossPct'] == assumptions['takeProfitPct'] == 0
    assert not assumptions['volatilitySizingEnabled']
    assert assumptions['minimumFeeNative'] == 0 and assumptions['taxBps'] == 0
    assert len(report['equityCurve']) == len(bars)
    if row['kind'] in ('context', 'evaluation'):
        fold = folds[row['fold'], symbol]
        assert start == fold['contextStart' if row['kind'] == 'context' else 'evaluationStart']
        assert end == fold['contextEndExclusive' if row['kind'] == 'context' else 'evaluationEndExclusive']
        reference = all_bars[fold['contextStart'] - 1]
        assert row['quantityReferenceDate'] == reference['date']
        assert row['quantityReferenceClose'] == reference['close']
        expected_qty = math.floor(.15 * 10_000_000 / (float(reference['close']) * 1450 * 1.002))
        assert row['orderQuantity'] == fold['orderQuantity'] == expected_qty
        assert reference['date'] < bars[0]['date']
        params = strategies[row['strategyId']]
        if row['strategyId'] == 'price_condition':
            params = {'symbols': [{'symbol': symbol, 'quantity': expected_qty,
                     'buy_trigger_price': float(reference['close']), 'take_profit_pct': 5.,
                     'stop_loss_pct': 3., 'is_overseas': True}]}
        assert row['params'] == params
    rate = row['costBps'] / 10_000
    fx = assumptions['exchangeRateKrw']
    capital = assumptions['initialCapitalKrw']
    cash, quantity, basis, buy_fee = capital, 0, 0.0, 0.0
    realized = fees = slippage = turnover = 0.0
    invested_sum = equity_fraction_sum = peak_cap = peak_fraction = 0.0
    held_bars = 0
    snapshots = []
    phase_before = {}
    phase_after = {}
    by_date = collections.defaultdict(list)
    filled = []
    for trade in report['trades']:
        if trade['status'] == 'filled':
            filled.append(trade)
            assert trade['fillAt']['phase'] == 'open'
            by_date[trade['fillAt']['barTime']].append(trade)
            assert trade['signalAvailableAt']['barTime'] < trade['fillAt']['barTime']
    raw_signals = {s['time']: s for s in result['signals']}
    previous_equity = capital
    previous_residual = 0.0
    previous_pending = False
    for i, bar in enumerate(bars):
        phase = report['equityCurve'][i]['phase']
        if phase not in phase_before:
            phase_before[phase] = (quantity, previous_pending, realized, previous_residual, previous_equity, i)
        open_price = float(bar['open'])
        close_price = float(bar['close'])
        for trade in by_date[bar['date']]:
            q = trade['quantity']
            assert q > 0
            fill = open_price * (1 + rate if trade['side'] == 'buy' else 1 - rate)
            gross = fill * fx * q
            fee = gross * rate
            slip = abs(fill - open_price) * fx * q
            near(trade['fillPrice'], fill, 'nativeFill', tolerance=0.000001)
            near(trade['grossAmountKrw'], gross, 'gross')
            near(trade['feeKrw'], fee, 'fee')
            near(trade['slippageKrw'], slip, 'slippage')
            near(trade['taxKrw'], 0., 'tax')
            near(trade['executionReferencePrice'], open_price, 'reference', tolerance=0.000001)
            fees += fee
            slippage += slip
            turnover += gross
            if trade['side'] == 'buy':
                cash -= gross + fee
                basis += gross
                buy_fee += fee
                quantity += q
            else:
                assert 0 < q <= quantity
                sold_basis = basis * q / quantity
                allocated_fee = buy_fee * q / quantity
                pnl = gross - sold_basis - allocated_fee - fee
                realized += pnl
                near(trade['realizedPnlKrw'], pnl, 'realizedTrade')
                cash += gross - fee
                basis -= sold_basis
                buy_fee -= allocated_fee
                quantity -= q
        market_value = close_price * fx * quantity
        residual = market_value - basis - buy_fee
        equity = cash + market_value
        near(report['equityCurve'][i]['equityKrw'], equity, 'equity')
        invested_sum += market_value
        fraction = market_value / equity * 100 if equity > 0 else 0
        equity_fraction_sum += fraction
        peak_cap = max(peak_cap, market_value / capital * 100)
        peak_fraction = max(peak_fraction, fraction)
        held_bars += quantity > 0
        pending = bar['date'] in raw_signals
        snapshots.append((quantity, equity, residual))
        phase_after[phase] = (quantity, pending, realized, residual, equity, i + 1)
        previous_equity, previous_residual, previous_pending = equity, residual, pending
    assert account['remainingQuantity'] == quantity
    near(account['remainingCostBasisKrw'], basis, 'remainingBasis')
    near(account['remainingBuyFeeKrw'], buy_fee, 'remainingBuyFee')
    near(account['realizedPnlKrw'], realized, 'realizedTotal')
    near(account['residualUnrealizedPnlKrw'], residual, 'residualTotal')
    near(summary['finalEquityKrw'], equity, 'finalEquity')
    near(equity - capital, realized + residual, 'equityIdentity')
    near(account['totalFeeKrw'], fees, 'feeTotal')
    near(account['totalSlippageKrw'], slippage, 'slippageTotal')
    near(summary['turnoverPct'], turnover / capital * 100, 'turnover', tolerance=0.00011)
    near(summary['exposurePct'], held_bars / len(bars) * 100, 'exposure', tolerance=0.00011)
    near(account['averageInvestedNotionalKrw'], invested_sum / len(bars), 'invested')
    near(account['averageInvestedCapitalPct'], invested_sum / len(bars) / capital * 100, 'capitalExposure', tolerance=0.00011)
    near(account['averageInvestedEquityPct'], equity_fraction_sum / len(bars), 'equityExposure', tolerance=0.00011)
    near(account['peakInvestedCapitalPct'], peak_cap, 'peakCapitalExposure', tolerance=0.00011)
    near(account['peakInvestedEquityPct'], peak_fraction, 'peakEquityExposure', tolerance=0.00011)
    terminal_slip = market_value * rate
    terminal_fee = (market_value - terminal_slip) * rate
    near(account['terminalLiquidationEquityKrw'], equity - terminal_slip - terminal_fee, 'terminal')
    assert summary['completedTrades'] == sum(t['side'] == 'sell' for t in filled)
    assert summary['filledOrderCount'] == len(filled)
    assert summary['blockedOrderCount'] == sum(t['status'] == 'blocked' for t in report['trades'])
    assert summary['unfilledOrderCount'] == sum(t['status'] == 'unfilled' for t in report['trades'])
    assert summary['signalCount'] == len(result['signals'])
    peak = capital
    max_dd = 0.
    for point in report['equityCurve']:
        dd = max(0., (peak - point['equityKrw']) / peak * 100)
        max_dd = max(max_dd, dd)
        peak = max(peak, point['equityKrw'])
    near(summary['mddPct'], max_dd, 'mdd', tolerance=0.00011)
    for phase in report['phases']:
        name = phase['phase']
        d = phase['diagnostics']
        before = phase_before[name]
        after = phase_after[name]
        count = after[5] - before[5]
        held = sum(q > 0 for q, _, _ in snapshots[before[5]:after[5]])
        assert d['version'] == 1 and d['policy'] == 'sameLedgerObservationSplit'
        assert d['observedBars'] == count and d['heldBars'] == held
        assert (d['startQuantity'], d['pendingAtStart']) == before[:2]
        assert (d['endQuantity'], d['pendingAtEnd']) == after[:2]
        near(d['exposurePct'], held / count * 100, 'phaseExposure', tolerance=0.00011)
        near(d['realizedPnlKrw'], after[2] - before[2], 'phaseRealized')
        near(d['startResidualUnrealizedPnlKrw'], before[3], 'phaseStartResidual')
        near(d['endResidualUnrealizedPnlKrw'], after[3], 'phaseEndResidual')
        near(d['equityChangeKrw'], after[4] - before[4], 'phaseEquity')
        near(d['dividendNetKrw'], 0., 'phaseDividend')
        near(d['equityChangeKrw'], d['realizedPnlKrw'] + d['endResidualUnrealizedPnlKrw'] - d['startResidualUnrealizedPnlKrw'] + d['dividendNetKrw'], 'phaseIdentity')
        assert phase['completedTrades'] == sum(t['side'] == 'sell' and t['phase'] == name for t in filled)
        checks['phaseCounts'] += 1
        phase_count += 1
    assert sum(p['diagnostics']['observedBars'] for p in report['phases']) == len(bars)
    near(sum(p['diagnostics']['equityChangeKrw'] for p in report['phases']), equity-capital, 'phaseTotalIdentity')
    reasons = collections.Counter(t.get('blockedReason', 'unspecified') for t in report['trades'] if t['status'] == 'blocked')
    return {'kind': row['kind'], 'fold': row.get('fold', row.get('quarter', '2023-2025')),
            'symbol': symbol, 'strategyId': row.get('strategyId', row.get('config', {}).get('strategyId')),
            'costBps': row['costBps'], 'orderQuantity': row.get('orderQuantity', row.get('config', {}).get('orderQuantity')),
            'observedBars': len(bars), 'returnPct': summary['cumulativeReturnPct'],
            'mddPct': summary['mddPct'], 'completedTrades': summary['completedTrades'],
            'filledOrders': summary['filledOrderCount'], 'blockedOrders': summary['blockedOrderCount'],
            'unfilledOrders': summary['unfilledOrderCount'], 'remainingQuantity': quantity,
            'realizedPnlKrw': account['realizedPnlKrw'], 'residualPnlKrw': account['residualUnrealizedPnlKrw'],
            'feeKrw': account['totalFeeKrw'], 'slippageKrw': account['totalSlippageKrw'],
            'turnoverPct': summary['turnoverPct'], 'holdingBarPct': summary['exposurePct'],
            'averageInvestedCapitalPct': account['averageInvestedCapitalPct'],
            'averageInvestedEquityPct': account['averageInvestedEquityPct'],
            'buyHoldRequestedQuantity': report['comparison']['sameInputQuantityBuyHold']['requestedQuantity'],
            'buyHoldActualEntryQuantity': report['comparison']['sameInputQuantityBuyHold']['actualEntryQuantity'],
            'buyHoldReturnPct': report['comparison']['sameInputQuantityBuyHold']['cumulativeReturnPct'],
            'blockedReasons': json.dumps(dict(reasons), ensure_ascii=False, sort_keys=True)}


summaries = []
keys = set()
for row in load('walk-forward-results.jsonl'):
    key = tuple(row[k] for k in ('kind', 'fold', 'symbol', 'strategyId', 'costBps'))
    assert key not in keys
    keys.add(key)
    summaries.append(validate_row(row))
assert len(keys) == 1584
expected_keys = {(kind, f['id'], symbol, strategy, cost) for kind in ('context', 'evaluation')
                 for f in design['folds'] for symbol in design['universe'] for strategy in strategies for cost in design['costBps']}
assert keys == expected_keys
prior_folder = ROOT / 'docs/research/price-condition-contracts-20261029'
prior = {}
for name in ('continuous-results.jsonl', 'reset-results.jsonl'):
    for row in load(name, prior_folder):
        prior[(row['kind'], row['config']['symbol'], row['costBps'], row.get('quarter'))] = row
compat = 0
for row in load('pc-compatibility-results.jsonl'):
    key = (row['kind'], row['config']['symbol'], row['costBps'], row.get('quarter'))
    assert row['config'] == prior[key]['config']
    assert strip_interpretation(row['result']) == strip_interpretation(prior[key]['result']), (key, 'economic compatibility')
    validate_row(row)
    compat += 1
assert compat == len(prior) == 78
with (HERE / 'summary.csv').open('w', newline='') as f:
    writer = csv.DictWriter(f, fieldnames=list(summaries[0]), lineterminator='\n')
    writer.writeheader()
    writer.writerows(summaries)
aggregate = []
for kind in ('context', 'evaluation'):
    for strategy in strategies:
        for cost in design['costBps']:
            selected = [r for r in summaries if (r['kind'], r['strategyId'], r['costBps']) == (kind, strategy, cost)]
            assert len(selected) == 33
            aggregate.append({'kind': kind, 'strategyId': strategy, 'costBps': cost, 'separateAccountWindows': len(selected),
            'medianReturnPct': statistics.median(r['returnPct'] for r in selected), 'minReturnPct': min(r['returnPct'] for r in selected),
            'maxReturnPct': max(r['returnPct'] for r in selected), 'worstMddPct': max(r['mddPct'] for r in selected),
            'zeroCompletedWindows': sum(r['completedTrades'] == 0 for r in selected),
            'under20CompletedWindows': sum(r['completedTrades'] < 20 for r in selected),
            'zeroActualBuyHoldWindows': sum(r['buyHoldActualEntryQuantity'] == 0 for r in selected),
            'medianHoldingBarPct': statistics.median(r['holdingBarPct'] for r in selected),
            'medianAverageInvestedCapitalPct': statistics.median(r['averageInvestedCapitalPct'] for r in selected),
            'medianTurnoverPct': statistics.median(r['turnoverPct'] for r in selected),
            'limitations': '33 separate accounts; overlapping contexts, correlated ETFs/costs/strategies are not independent samples. No sum/compounded return or statistical confidence claim.'})
(HERE / 'aggregate.json').write_text(json.dumps(aggregate, ensure_ascii=False, indent=2) + '\n')
archives = {}
for name in ('walk-forward-results.jsonl', 'pc-compatibility-results.jsonl'):
    plain, packed = HERE / name, HERE / (name + '.gz')
    if plain.exists():
        with plain.open('rb') as source_file, packed.open('wb') as dest:
            with gzip.GzipFile(fileobj=dest, mode='wb', compresslevel=9, mtime=0) as zipped:
                while chunk := source_file.read(1024 * 1024):
                    zipped.write(chunk)
        # Retain only lossless archive in tracked research; remove this run's plaintext.
        with gzip.open(packed, 'rb') as a, plain.open('rb') as b:
            while chunk := a.read(1024 * 1024):
                assert chunk == b.read(len(chunk))
            assert not b.read(1)
        plain.unlink()
    h = hashlib.sha256(); size = 0
    with gzip.open(packed, 'rb') as f:
        while chunk := f.read(1024 * 1024):
            h.update(chunk); size += len(chunk)
    archives[packed.name] = {'uncompressedSha256': h.hexdigest(), 'uncompressedBytes': size, 'compressedBytes': packed.stat().st_size}
files = [p for p in HERE.iterdir() if p.is_file() and p.name != 'validation.json' and not p.name.startswith('.')]
files += [SOURCE, ROOT / 'src-tauri/src/trading/simulation.rs', ROOT / 'src-tauri/src/trading/guard.rs', ROOT / 'src-tauri/src/trading/risk.rs', ROOT / 'src-tauri/src/commands/strategy_preview.rs']
for folder in ('src-tauri/src/trading/simulation', 'src-tauri/src/trading/strategy', 'src-tauri/src/commands/strategy_preview'):
    files += list((ROOT / folder).rglob('*.rs'))
validation = {'designSha256': hashlib.sha256((HERE / 'design.json').read_bytes()).hexdigest(),
    'contextRuns': 792, 'evaluationRuns': 792, 'totalWalkForwardRuns': 1584, 'pcCompatibilityRuns': compat,
    'partitionCompatibilityRuns': 6, 'phaseDiagnosticsChecked': phase_count,
    'recordedArithmeticChecks': sum(checks.values()), 'checksByKind': dict(checks), 'maximumMoneyDifferenceKrw': maximum_money_error,
    'archives': archives, 'comparisonPolicy': design['comparisonPolicy'],
    'scope': 'Recorded accepted fills/causal prices, proportional cost basis/buy fees, cash/equity/phase identity, exposure/MDD/turnover and sample counts. No strategy signal reimplementation. No unseen OOS.',
    'sha256': {str(p.relative_to(ROOT)): hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(set(files))}}
(HERE / 'validation.json').write_text(json.dumps(validation, ensure_ascii=False, indent=2) + '\n')
print(json.dumps({k: v for k, v in validation.items() if k not in ('sha256', 'checksByKind')}, ensure_ascii=False))
