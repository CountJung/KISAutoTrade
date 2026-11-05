#!/usr/bin/env python3
"""기존 추세 필터 기록 체결의 독립 정산 감사. 생산 전략은 실행하지 않는다."""

from collections import Counter, defaultdict
from decimal import Decimal, getcontext
from hashlib import sha256
from pathlib import Path
import csv
import gzip
import json
import re
import subprocess


getcontext().prec = 40
ROOT = Path(__file__).resolve().parents[3]
OUT = Path(__file__).with_suffix('.json')
SOURCE = ROOT / 'docs/research/continuous-replay-20261018'
INPUT = ROOT / 'docs/research/strategy-review-20261018/input.json'
EXPECTED_INPUT_SHA = 'd48ce5b59b7876c0a1457121e877fdc1117a77f990b4ec07ce649c92fe153317'
BASE = '81fa00d3aa50bc59606667568f36c4b686902412'
D = lambda v: Decimal(str(v))
ZERO = D(0)
HUNDRED = D(100)
checks = Counter()
errors = []
largest = defaultdict(lambda: ZERO)


def check(condition, category, identity, detail=''):
    checks[category] += 1
    if not condition:
        errors.append({'category': category, 'identity': identity, 'detail': detail})


def close(actual, expected, category, identity, tolerance='0.0051'):
    error = abs(D(actual) - D(expected))
    largest[category] = max(largest[category], error)
    check(error <= D(tolerance), category, identity,
          f'기록={actual}, 재계산={expected}, 차이={error}, 허용={tolerance}')


def pct(n, denominator):
    return n / denominator * HUNDRED if denominator else ZERO


def native(candle, key):
    # 이 고정 원본은 소수 둘째 자리 가격이다. 생산 native cent 변환을 재모델링하지 않는다.
    return D(candle[key])


def digest(path):
    return sha256(path.read_bytes()).hexdigest()


def quarter(day):
    return f'{day[:4]}Q{(int(day[4:6]) - 1) // 3 + 1}'


def numbers(value):
    if isinstance(value, Decimal):
        return float(value)
    if isinstance(value, dict):
        return {k: numbers(v) for k, v in value.items()}
    if isinstance(value, (list, tuple)):
        return [numbers(v) for v in value]
    return value


def audit_benchmark(row, bars, identity):
    """기록된 실제 진입 수량에 조건부인 보유 산술. guard 결정은 재구현하지 않는다."""
    b = row['result']['backtest']
    a = b['assumptions']
    ref = b['comparison']['sameInputQuantityBuyHold']
    capital, fx, rate = D(a['initialCapitalKrw']), D(a['exchangeRateKrw']), D(row['costBps']) / D(10000)
    qty = ref['actualEntryQuantity']
    check(ref['requestedQuantity'] == row['config']['orderQuantity'], '보유 비교 요청 수량', identity)
    check((qty == 0 and ref['filledOrderCount'] == 0 and ref['blockedOrderCount'] == 1)
          or (qty > 0 and ref['filledOrderCount'] == 1 and ref['blockedOrderCount'] == 0),
          '보유 비교 기록 형태', identity)
    gross = native(bars[1], 'open') * (1 + rate) * qty * fx if qty else ZERO
    fee = gross * rate
    cash = capital - gross - fee
    equities, values, equity_ratios = [], [], []
    peak, mdd = capital, ZERO
    for i, bar in enumerate(bars):
        value = native(bar, 'close') * qty * fx if i >= 1 else ZERO
        equity = cash + value if i >= 1 else capital
        peak = max(peak, equity)
        mdd = max(mdd, pct(peak - equity, peak))
        equities.append(equity)
        values.append(value)
        equity_ratios.append(pct(value, equity))
    final = equities[-1]
    liquidation_gross = native(bars[-1], 'close') * (1 - rate) * qty * fx
    liquidation = cash + liquidation_gross * (1 - rate)
    remaining = final - capital
    calculations = {
        'finalEquityKrw': final,
        'cumulativeReturnPct': pct(final - capital, capital),
        'realizedPnlKrw': ZERO,
        'residualUnrealizedPnlKrw': remaining,
        'remainingQuantity': qty,
        'mddPct': mdd,
        'averageInvestedCapitalPct': pct(sum(values) / len(bars), capital),
        'averageInvestedEquityPct': sum(equity_ratios) / len(bars),
        'peakInvestedCapitalPct': pct(max(values), capital),
        'peakInvestedEquityPct': max(equity_ratios),
        'holdingBarPct': pct(D(len(bars) - 1 if qty else 0), D(len(bars))),
        'terminalLiquidationReturnPct': pct(liquidation - capital, capital),
        'dividendNetKrw': ZERO,
    }
    for key, value in calculations.items():
        close(ref[key], value, '보유 비교 재정산 ' + key, identity,
              '0.000051' if key.endswith('Pct') else '0.0051')
    return {'scope': '기록된 실제 진입 수량 조건부 산술; 별도 보유 주문 원장은 없음',
            'requestedQuantity': ref['requestedQuantity'], 'actualEntryQuantity': qty,
            'entryTime': bars[1]['date'] if qty else None, **calculations}


def audit_row(row, source_path, line_number, datasets, design, csv_rows, partitions):
    r, cfg = row['result'], row['config']
    symbol, cost = cfg['symbol'], row['costBps']
    identity = f"{row['kind']}/{symbol}/{cost}/{row.get('quarter', '2023-2025')}"
    b, bars = r['backtest'], r['candles']
    a, summary, accounting = b['assumptions'], b['summary'], b['accounting']
    s = row.get('evaluationStart', cfg['evaluationStart'])
    e = row.get('evaluationEndExclusive', cfg['evaluationEndExclusive'])
    h = row.get('historyStart', cfg['historyStart'])
    frozen = datasets[symbol]
    check(bars == frozen[s:e], '평가 원본 일치', identity)
    check(s - h == 252 and frozen[s-1]['date'] < bars[0]['date'], '사전 자료 분리', identity)
    check(cfg['params'] == {'long_period': 200, 'mid_period': 20, 'short_period': 5}, '원래 설정 고정', identity)
    check(cfg in design['configs'], '기존 설계 설정 일치', identity)
    check(len(bars) == e - s and len({x['date'] for x in bars}) == len(bars), '관측 날짜 중복 없음', identity)
    check([x['date'] for x in bars] == sorted(x['date'] for x in bars), '관측 날짜 순서', identity)
    check(r['strategyId'] == 'trend_filter' and r['symbol'] == symbol, '전략 격리', identity)
    for key, value in {
        'executionModel': 'nextOpen', 'initialCapitalKrw': 10000000.0,
        'exchangeRateKrw': 1450.0, 'feeBps': cost, 'slippageBps': cost,
        'taxBps': 0.0, 'stopLossPct': 0.0, 'takeProfitPct': 0.0,
        'fxScenario': [], 'dividendScenario': [], 'volatilitySizingEnabled': False,
        'minimumFeeNative': 0.0, 'buyFeeBps': None, 'sellFeeBps': None,
        'maxPositionRatio': 0.3, 'dailyLossLimitKrw': 1000000.0,
    }.items():
        check(a[key] == value, '기록 가정 고정', identity, key)
    prep = r['preparation']
    for key, value in {'providedHistoryBars': 252, 'requiredHistoryBars': 200,
                       'availableHistoryBars': 200, 'readyAtStart': True,
                       'readyAtEnd': True, 'evaluatedBars': len(bars),
                       'unreadyEvaluationBars': 0}.items():
        check(prep[key] == value, '준비 진단', identity, key)
    replay = r['replay']
    check(replay['engineVersion'] == 'strategy-replay-v12', '기존 엔진 기록', identity)
    check(replay['sourceInterval'] == '1d' and replay['dataStart'] == bars[0]['date']
          and replay['dataEnd'] == bars[-1]['date'] and replay['warmupCount'] == 252,
          '실행 기간 기록', identity)
    check(bool(re.fullmatch('[0-9a-f]{16}', replay['inputHash'])), '기록 inputHash 형식', identity)
    if row['kind'] == 'continuous':
        part = partitions[(symbol, cost)]
        check(part['inputHash'] == replay['inputHash'], '분할 기록 inputHash 일치', identity)
        check(part['quarterChunks'] == 12 and part['alternativeChunks'] == 2, '분할 기록 수', identity)
        check(len(bars) == 752 and bars[0]['date'] == '20230103'
              and bars[-1]['date'] == '20251231', '연속 기간 고정', identity)
    else:
        check(all(quarter(x['date']) == row['quarter'] for x in bars), '별도 분기 경계', identity)

    capital, fx, rate = D(a['initialCapitalKrw']), D(a['exchangeRateKrw']), D(cost) / D(10000)
    by_day = defaultdict(list)
    indexes = {bar['date']: i for i, bar in enumerate(bars)}
    ts = b['trades']
    signatures = Counter((x['time'], x['side'], D(x['price']), x['quantity']) for x in r['signals'])
    check(len(r['signals']) == summary['signalCount'], '신호 수', identity)
    for trade in ts:
        status, day = trade['status'], trade['time']
        check(day in indexes and status in {'filled', 'blocked', 'unfilled'}, '주문 기록 범위', identity)
        available = trade['signalAvailableAt']
        signature = (available['barTime'], trade['side'], D(trade['signalPrice']), trade['quantity'])
        check(signatures[signature] > 0, '주문과 원래 신호 대응', identity)
        signatures[signature] -= 1
        check(available['phase'] == 'close', '신호 공개 단계', identity)
        si = indexes[available['barTime']]
        close(trade['signalPrice'], native(bars[si], 'close'), '신호 관측 가격', identity, '0.00000001')
        if status == 'unfilled':
            check(si == len(bars)-1 and day == available['barTime'] and trade.get('fillAt') is None,
                  '마지막 미체결', identity)
        else:
            check(indexes[day] == si + 1, '다음 관측 봉 인과성', identity)
        if status == 'filled':
            check(trade['fillAt'] == {'barTime': day, 'phase': 'open'}, '체결 관측 단계', identity)
            by_day[day].append(trade)
        else:
            for key in ['grossAmountKrw', 'feeKrw', 'slippageKrw', 'taxKrw', 'costKrw']:
                close(trade[key], ZERO, '미체결 현금 영향 없음', identity, '0')
            check(trade['fillPrice'] is None and trade['realizedPnlKrw'] is None,
                  '미체결 원가 영향 없음', identity)
    check(all(n == 0 for n in signatures.values()), '누락 또는 중복 주문 대응 없음', identity)

    cash, basis, buy_fees, realized, realized_price = capital, ZERO, ZERO, ZERO, ZERO
    qty = 0
    fees, slippage, taxes, turnover, market_pnl = ZERO, ZERO, ZERO, ZERO, ZERO
    values, equity_ratios, equities, owned_quantities = [], [], [], []
    peak, mdd, previous_close = capital, ZERO, None
    sale_pnls, sales, purchases = [], [], []
    signals_by_day = defaultdict(list)
    for signal in r['signals']:
        signals_by_day[signal['time']].append(signal)
    flat_sells = held_buys = 0
    chunks_by_end = {x['end']: x for x in r.get('continuity', {}).get('chunks', [])}
    chunks_by_start = {x['start']: x for x in r.get('continuity', {}).get('chunks', [])}
    for i, candle in enumerate(bars):
        day, op, cl = candle['date'], native(candle, 'open'), native(candle, 'close')
        if day in chunks_by_start:
            check(chunks_by_start[day]['startQuantity'] == qty, '연속 경계 실제 수량', identity)
        if previous_close is not None:
            market_pnl += (op - previous_close) * qty * fx
        for t in by_day[day]:
            q, side = t['quantity'], t['side']
            check(q > 0, '실체결 양수 수량', identity)
            price = op * (1 + rate if side == 'buy' else 1 - rate)
            gross, fee = price * q * fx, price * q * fx * rate
            slip = abs(price - op) * q * fx
            close(t['executionReferencePrice'], op, '원본 시가 체결 기준', identity, '0.00000001')
            close(t['fillPrice'], price, '슬리피지 체결 가격', identity, '0.00000001')
            close(t['exchangeRateKrw'], fx, '체결 환율', identity, '0')
            close(t['grossAmountKrw'], gross, '체결 총액', identity, '0.00001')
            close(t['feeKrw'], fee, '체결 수수료', identity, '0.00001')
            close(t['slippageKrw'], slip, '체결 슬리피지', identity, '0.00001')
            close(t['taxKrw'], ZERO, '체결 세금', identity, '0')
            close(t['costKrw'], fee + slip, '체결 비용 합', identity, '0.00001')
            fees += fee
            slippage += slip
            turnover += gross
            if side == 'buy':
                cash -= gross + fee
                basis += gross
                buy_fees += fee
                qty += q
                purchases.append({'time': day, 'quantity': q, 'price': price})
                check(t['realizedPnlKrw'] is None, '매수 실현 없음', identity)
            else:
                check(q <= qty, '실제 보유 이내 청산', identity)
                share = D(q) / D(qty)
                allocated_basis, allocated_fee = basis * share, buy_fees * share
                pnl = gross - fee - allocated_basis - allocated_fee
                realized += pnl
                realized_price += gross - allocated_basis
                cash += gross - fee
                basis -= allocated_basis
                buy_fees -= allocated_fee
                qty -= q
                sale_pnls.append(pnl)
                sales.append({'time': day, 'signalTime': t['signalAvailableAt']['barTime'],
                              'quantity': q, 'realizedPnlKrw': pnl})
                close(t['realizedPnlKrw'], pnl, '체결별 실현 정산', identity)
            check(cash >= 0 and qty >= 0, '현금과 보유 비음수', identity)
        market_pnl += (cl - op) * qty * fx
        value = cl * qty * fx
        equity = cash + value
        peak = max(peak, equity)
        dd = pct(peak - equity, peak)
        mdd = max(mdd, dd)
        close(b['equityCurve'][i]['equityKrw'], equity, '봉별 자산 재정산', identity)
        close(b['equityCurve'][i]['drawdownPct'], dd, '봉별 낙폭 재정산', identity, '0.000051')
        check(b['equityCurve'][i]['time'] == day, '봉별 자산 시각', identity)
        if day in chunks_by_end:
            check(chunks_by_end[day]['endQuantity'] == qty, '연속 경계 실제 수량', identity)
        for signal in signals_by_day[day]:
            flat_sells += int(signal['side'] == 'sell' and qty == 0)
            held_buys += int(signal['side'] == 'buy' and qty > 0)
        values.append(value)
        equity_ratios.append(pct(value, equity))
        equities.append(equity)
        owned_quantities.append(qty)
        previous_close = cl
    check(len(b['equityCurve']) == len(bars), '봉별 자산 개수', identity)
    residual = values[-1] - basis - buy_fees
    terminal_gross = native(bars[-1], 'close') * (1 - rate) * qty * fx
    terminal_fee = terminal_gross * rate
    terminal_slip = native(bars[-1], 'close') * rate * qty * fx
    terminal = cash + terminal_gross - terminal_fee
    final = equities[-1]
    profit, loss = sum((x for x in sale_pnls if x > 0), ZERO), -sum((x for x in sale_pnls if x < 0), ZERO)
    counts = Counter(t['status'] for t in ts)
    nwin, nloss = sum(x > 0 for x in sale_pnls), sum(x < 0 for x in sale_pnls)
    summary_calc = {
        'initialCapitalKrw': capital, 'finalEquityKrw': final,
        'cumulativeReturnPct': pct(final - capital, capital), 'mddPct': mdd,
        'turnoverPct': pct(turnover, capital),
        'exposurePct': pct(D(sum(q > 0 for q in owned_quantities)), D(len(bars))),
        'filledOrderCount': counts['filled'], 'blockedOrderCount': counts['blocked'],
        'unfilledOrderCount': counts['unfilled'], 'completedTrades': len(sale_pnls),
        'winningTrades': nwin, 'losingTrades': nloss,
        'winRatePct': pct(D(nwin), D(len(sale_pnls))),
    }
    for key, value in summary_calc.items():
        close(summary[key], value, '요약 독립 정산 ' + key, identity,
              '0.000051' if key.endswith('Pct') else '0.0051')
    if loss > 0:
        close(summary['profitFactor'], profit / loss, '완료 손익 비율', identity, '0.000051')
    else:
        check(summary['profitFactor'] is None, '완료 손실 없음 표기', identity)
    accounting_calc = {
        'realizedPnlKrw': realized, 'realizedPricePnlKrw': realized_price,
        'realizedFxPnlKrw': ZERO, 'remainingQuantity': qty,
        'remainingCostBasisKrw': basis, 'remainingBuyFeeKrw': buy_fees,
        'residualUnrealizedPnlKrw': residual, 'residualPricePnlKrw': values[-1] - basis,
        'residualFxPnlKrw': ZERO, 'totalFeeKrw': fees, 'totalSlippageKrw': slippage,
        'totalTaxKrw': ZERO, 'dividendGrossKrw': ZERO, 'dividendTaxKrw': ZERO,
        'dividendNetKrw': ZERO, 'terminalExitFeeKrw': terminal_fee,
        'terminalExitSlippageKrw': terminal_slip, 'terminalExitTaxKrw': ZERO,
        'terminalLiquidationEquityKrw': terminal,
        'terminalLiquidationReturnPct': pct(terminal - capital, capital),
        'averageInvestedNotionalKrw': sum(values) / len(bars),
        'averageInvestedCapitalPct': pct(sum(values) / len(bars), capital),
        'averageInvestedEquityPct': sum(equity_ratios) / len(bars),
        'peakInvestedCapitalPct': pct(max(values), capital),
        'peakInvestedEquityPct': max(equity_ratios),
    }
    for key, value in accounting_calc.items():
        close(accounting[key], value, '회계 독립 정산 ' + key, identity,
              '0.000051' if key.endswith('Pct') else '0.0051')
    close(final - capital, realized + residual, '실현 잔여 자산 항등식', identity, '0.00001')
    close(final + fees + slippage, capital + market_pnl, '원본 시가 종가 실제 경로 항등식', identity, '0.00001')
    comparison = b['comparison']
    pathref = comparison['sameExecutionPathCostReference']
    for key, value in {'restoredCostKrw': fees + slippage,
                       'finalEquityKrw': capital + market_pnl,
                       'cumulativeReturnPct': pct(market_pnl, capital)}.items():
        close(pathref[key], value, '동일 실제 경로 비용 환급 ' + key, identity,
              '0.000051' if key.endswith('Pct') else '0.0151')
    # 기존 참조는 이미 0.01원으로 반올림한 자산·수수료·슬리피지를 합친다.
    # 원장 정밀도 정산과는 최대 0.015원 차이가 가능하며 기록끼리 항등식도 별도 확인한다.
    close(pathref['restoredCostKrw'], D(accounting['totalFeeKrw']) + D(accounting['totalSlippageKrw']),
          '반올림 비용 환급 항등식', identity, '0.000001')
    close(pathref['finalEquityKrw'], D(summary['finalEquityKrw']) + D(pathref['restoredCostKrw']),
          '반올림 자산 비용 환급 항등식', identity, '0.000001')
    benchmark = audit_benchmark(row, bars, identity)
    close(comparison['capitalExposureDifferencePct'],
          accounting_calc['averageInvestedCapitalPct'] - benchmark['averageInvestedCapitalPct'],
          '보유 비교 자본 노출 차이', identity, '0.000151')
    close(comparison['holdingBarDifferencePct'],
          summary_calc['exposurePct'] - benchmark['holdingBarPct'],
          '보유 비교 보유 봉 차이', identity, '0.000151')
    # 두 입력을 각각 소수 넷째 자리로 반올림한 뒤 차이를 기록하므로 독립 정밀도와 분리한다.
    close(comparison['capitalExposureDifferencePct'],
          D(accounting['averageInvestedCapitalPct']) - D(b['comparison']['sameInputQuantityBuyHold']['averageInvestedCapitalPct']),
          '반올림 자본 노출 차이 항등식', identity, '0.000001')
    close(comparison['holdingBarDifferencePct'],
          D(summary['exposurePct']) - D(b['comparison']['sameInputQuantityBuyHold']['holdingBarPct']),
          '반올림 보유 봉 차이 항등식', identity, '0.000001')
    csv_row = csv_rows[(row['kind'], symbol, cost, row.get('quarter', '2023-2025'))]
    for csv_key, value in {
        'returnPct': summary['cumulativeReturnPct'],
        'terminalReturnPct': accounting['terminalLiquidationReturnPct'],
        'mddPct': summary['mddPct'], 'realizedPnlKrw': accounting['realizedPnlKrw'],
        'residualPnlKrw': accounting['residualUnrealizedPnlKrw'],
        'completedTrades': summary['completedTrades'], 'blockedOrders': summary['blockedOrderCount'],
        'turnoverPct': summary['turnoverPct'],
        'averageInvestedCapitalPct': accounting['averageInvestedCapitalPct'],
        'holdingBarPct': summary['exposurePct'],
    }.items():
        close(csv_row[csv_key], value, 'CSV 원장 요약 교차검증', identity, '0')
    return {
        'identity': identity, 'sourcePath': str(source_path.relative_to(ROOT)),
        'sourceLine': line_number, 'sourceRowSha256': sha256(json.dumps(row, sort_keys=True, ensure_ascii=False,
                                                                      separators=(',', ':')).encode()).hexdigest(),
        'symbol': symbol, 'costBpsEachFeeAndSlippage': cost, 'kind': row['kind'],
        'quarter': row.get('quarter'), 'orderQuantity': cfg['orderQuantity'],
        'evaluationStart': bars[0]['date'], 'evaluationEnd': bars[-1]['date'],
        'evaluationBars': len(bars), 'historyStart': frozen[h]['date'],
        'historyEnd': frozen[s-1]['date'], 'historyBars': s-h,
        'recordedInputHash': replay['inputHash'],
        'independentSettlement': {'finalCashKrw': cash, **summary_calc, **accounting_calc,
                                  'remainingNativeAverageFromRecordedFills': basis / (qty * fx) if qty else None,
                                  'marketPathPnlBeforeRecordedCostsKrw': market_pnl,
                                  'grossRecordedExecutionPathReturnPct': pct(market_pnl, capital),
                                  'netReturnPct': pct(final - capital, capital),
                                  'explicitRecordedCostKrw': fees + slippage},
        'confirmedSales': sales, 'confirmedPurchases': purchases,
        'sameInputQuantityBuyHold': benchmark,
        'excessReturnToRecordedQuantityBuyHoldPct': summary_calc['cumulativeReturnPct'] - benchmark['cumulativeReturnPct'],
        'flatSellSignals': flat_sells, 'heldBuySignals': held_buys,
        'realizedLossWithPositiveEquity': realized < 0 and final > capital,
        'positiveEquityWithoutCompletedSale': final > capital and not sales,
    }


def main():
    check(digest(INPUT) == EXPECTED_INPUT_SHA, '고정 입력 SHA256', '전체 입력')
    source_input = json.loads(INPUT.read_text())
    datasets = {x['symbol']: x['candles'] for x in source_input['datasets']}
    normalized_sources = []
    current_design_path = Path(__file__).with_name('design.json')
    current_design = json.loads(current_design_path.read_text())
    check(current_design['sourceSha256'] == EXPECTED_INPUT_SHA,
          '이번 설계 원본 SHA256 연결', '고정 입력')
    check(current_design['baselineEngineVersion'] == 'strategy-replay-v21'
          and current_design['plannedEngineVersion'] == 'strategy-replay-v22',
          '이전 원장과 새 후보 버전 분리', 'v12/v21/v22')
    for dataset in source_input['datasets']:
        symbol, bars, metadata = dataset['symbol'], dataset['candles'], dataset['metadata']
        recomputed = sha256(json.dumps(bars, sort_keys=True, separators=(',', ':')).encode()).hexdigest()
        check(recomputed == metadata['normalizedSha256'], '정규화 가격 SHA256 재계산', symbol)
        declared = next(x for x in current_design['provenance'] if x['symbol'] == symbol)
        check(recomputed == declared['normalizedSha256'], '이번 설계 정규화 SHA256 연결', symbol)
        for key in ['sourceUrl', 'retrievedAtUtc', 'rawSha256', 'sourceTimezone',
                    'normalizedTimezone', 'calendarCoverageVerified']:
            check(declared[key] == metadata[key], '이번 설계 출처 메타데이터 승계', symbol, key)
        check(declared['rawBytesRehashedThisStage'] is False, 'raw 원본 재검증 주장 제한', symbol)
        normalized_sources.append({
            'symbol': symbol, 'sourcePath': str(INPUT.relative_to(ROOT)),
            'normalizedSha256Recomputed': recomputed,
            'rawSha256InheritedOnly': metadata['rawSha256'],
            'retrievedAtUtcInheritedOnly': metadata['retrievedAtUtc'],
            'sourceUrlInheritedOnly': metadata['sourceUrl'],
            'rawBytesReadOrRehashedByThisAudit': False,
            'normalizedBars': len(bars), 'firstDate': bars[0]['date'], 'lastDate': bars[-1]['date'],
            'calendarCoverageVerified': False,
        })
    design = json.loads((SOURCE / 'design.json').read_text())
    check(design['sourceSha256'] == EXPECTED_INPUT_SHA, '기존 설계 입력 SHA256', '전체 입력')
    summary = json.loads((SOURCE / 'run-summary.json').read_text())
    partitions = {(x['symbol'], x['costBps']): x for x in summary['partitions'] if x['strategyId'] == 'trend_filter'}
    csv_rows = {(r['kind'], r['symbol'], int(r['costBps']), r['quarter']): r
                for r in csv.DictReader((SOURCE / 'summary.csv').open()) if r['strategyId'] == 'trend_filter'}
    records, inventory = [], []
    for file, expected, kind in [('continuous-results.jsonl.gz', 6, 'continuous'),
                                 ('reset-results.jsonl.gz', 72, 'quarterReset')]:
        path = SOURCE / file
        count, other = 0, 0
        with gzip.open(path, 'rt') as stream:
            for line_number, line in enumerate(stream, 1):
                row = json.loads(line)
                if row['config']['strategyId'] != 'trend_filter':
                    other += 1
                    continue
                count += 1
                check(row['kind'] == kind, '원장 유형 격리', file)
                records.append(audit_row(row, path, line_number, datasets, design, csv_rows, partitions))
        check(count == expected, '추세 필터 원장 개수', file)
        inventory.append({'sourcePath': str(path.relative_to(ROOT)), 'sha256': digest(path),
                          'decompressedSha256': sha256(gzip.decompress(path.read_bytes())).hexdigest(),
                          'trendFilterRowsAudited': count, 'otherStrategyRowsExcluded': other})
    keys = [(r['kind'], r['symbol'], r['costBpsEachFeeAndSlippage'], r['quarter']) for r in records]
    expected_keys = [('continuous', s, c, None) for s in ['SOXQ', 'SPY', 'QQQ'] for c in [10, 20]]
    expected_keys += [('quarterReset', s, c, f'{year}Q{q}') for s in ['SOXQ', 'SPY', 'QQQ']
                     for c in [10, 20] for year in [2023, 2024, 2025] for q in [1, 2, 3, 4]]
    check(Counter(keys) == Counter(expected_keys), '전략별 비용별 기간 전체 조합', '78개 원장')
    grouped = []
    for kind in ['continuous', 'quarterReset']:
        for cost in [10, 20]:
            subset = [r for r in records if r['kind'] == kind and r['costBpsEachFeeAndSlippage'] == cost]
            pnl = [x['realizedPnlKrw'] for r in subset for x in r['confirmedSales']]
            grouped.append({
                'kind': kind, 'costBpsEachFeeAndSlippage': cost, 'rows': len(subset),
                'completedSales': len(pnl), 'profitableSales': sum(x > 0 for x in pnl),
                'losingSales': sum(x < 0 for x in pnl),
                'terminalHeldRows': sum(r['independentSettlement']['remainingQuantity'] > 0 for r in subset),
                'rowsWithoutCompletedSale': sum(not r['confirmedSales'] for r in subset),
                'positiveEquityWithoutCompletedSaleRows': sum(r['positiveEquityWithoutCompletedSale'] for r in subset),
                'realizedLossWithPositiveEquityRows': sum(r['realizedLossWithPositiveEquity'] for r in subset),
                'benchmarkNoEntryRows': sum(r['sameInputQuantityBuyHold']['actualEntryQuantity'] == 0 for r in subset),
                'warning': '같은 자산·기간·두 비용은 상관된 기록이다. 분기 손익을 합산하거나 연속 수익으로 복리화하지 않는다.',
            })
    cost_paths = []
    paired = defaultdict(dict)
    for record in records:
        paired[(record['kind'], record['symbol'], record['quarter'])][record['costBpsEachFeeAndSlippage']] = record
    for (kind, symbol, q), pair in sorted(paired.items(), key=lambda x: str(x[0])):
        low, high = pair[10], pair[20]
        fills = lambda r: sorted([(x['time'], 'buy', x['quantity']) for x in r['confirmedPurchases']]
                                 + [(x['time'], 'sell', x['quantity']) for x in r['confirmedSales']])
        cost_paths.append({
            'kind': kind, 'symbol': symbol, 'quarter': q,
            'sameAcceptedQuantityTimingAtBothCosts': fills(low) == fills(high),
            'netReturn10Bps': low['independentSettlement']['netReturnPct'],
            'netReturn20Bps': high['independentSettlement']['netReturnPct'],
            'netReturnSignFlip': (low['independentSettlement']['netReturnPct'] > 0)
                                 != (high['independentSettlement']['netReturnPct'] > 0),
            'note': '기록된 두 비용 실행의 실제 수량·시점 비교다. 다른 비용이나 새 후보의 경로 동일성을 보장하지 않는다.',
        })
    source_paths = [INPUT, SOURCE/'design.json', SOURCE/'run-summary.json', SOURCE/'summary.csv',
                    current_design_path, Path(__file__)]
    provenance = [{'sourcePath': str(p.relative_to(ROOT)), 'sha256': digest(p)} for p in source_paths]
    old_path = 'src-tauri/src/trading/strategy/mean_trend.rs'
    source_bytes = subprocess.check_output(['git', 'show', BASE + ':' + old_path], cwd=ROOT)
    provenance.append({'sourcePath': f'git:{BASE}:{old_path}', 'sha256': sha256(source_bytes).hexdigest(),
                       'purpose': '이번 수정 직전 구현의 bool 소유·설정 수량 청산 한계 확인용이다. v12 실행 파일 동일성 증명은 아니다.'})
    for text in ['in_position: bool', 'state.in_position = true;', 'state.in_position = false;', 'quantity: qty']:
        check(text.encode() in source_bytes, '수정 전 소유와 청산 구현 근거', old_path, text)
    comparison_path = 'src-tauri/src/trading/simulation/comparison.rs'
    comparison_bytes = subprocess.check_output(['git', 'show', BASE + ':' + comparison_path], cwd=ROOT)
    provenance.append({'sourcePath': f'git:{BASE}:{comparison_path}',
                       'sha256': sha256(comparison_bytes).hexdigest(),
                       'purpose': 'Serena 정의 조회와 함께 공개 비교 필드의 반올림 피연산자·연산 순서를 확인했다. 기존 verifier는 읽거나 복사하지 않았다.'})
    for text in ['report.accounting.total_fee_krw', 'report.summary.final_equity_krw + restored_cost',
                 'round_money(restored_cost)', 'round_money(restored_equity)',
                 'round_metric(exposure_difference)', 'round_metric(holding_difference)']:
        check(text.encode() in comparison_bytes, '공개 비교 필드 반올림 근거', comparison_path, text)
    report = {
        'auditId': 'trend-filter-existing-ledger-audit-20261023',
        'status': '통과' if not errors else '오류',
        'scope': '저장된 strategy-replay-v12 trend_filter 연속6개·별도 분기72개의 독립 체결 정산; 새 생산 실행·전략 재구현 없음',
        'assertions': sum(checks.values()), 'assertionsByCategory': dict(checks),
        'errorCount': len(errors), 'errors': errors,
        'largestAbsoluteDifferenceByCategory': dict(largest),
        'sourceInputSha256': EXPECTED_INPUT_SHA, 'sourceFiles': provenance,
        'normalizedSources': normalized_sources,
        'engineSeparation': {
            'auditedStoredLedgerEngine': 'strategy-replay-v12',
            'prechangeSourceCommit': BASE, 'prechangeSourceEngine': 'strategy-replay-v21',
            'plannedCandidateEngine': 'strategy-replay-v22',
            'candidateEngineRunsIncludedInThisAudit': 0,
            'note': '기존 v12 기록 정산의 통과가 수정 전 v21 또는 미래 v22 후보의 실행·성과 검증을 뜻하지 않는다.',
        },
        'roundingPolicy': {
            'directMoneyToleranceKrw': 0.0051, 'directPercentageTolerancePoints': 0.000051,
            'roundedCostRefundMoneyToleranceKrw': 0.0151,
            'roundedExposureDifferenceTolerancePoints': 0.000151,
            'reason': '비용 환급은 반올림한 자산·수수료·슬리피지의 합이며 노출 차이는 두 반올림 비율의 차이다. 정밀도 재정산과 기록 항등식을 각각 확인한다.',
        },
        'grossDefinition': 'grossRecordedExecutionPathReturnPct는 기록된 실제 수량·시점을 고정한 원본 시가/종가 경로 손익을 자본으로 나눈 값이다. 실제 기록 fee+slippage를 net에 환급하는 산술 참조와 대조한다. 비용 없이 전략을 재실행한 성과가 아니다.',
        'archives': inventory, 'aggregates': grouped, 'costPathComparisons': cost_paths, 'records': records,
        'verified': [
            '추세 필터만 3자산×2비용×연속1개·분기12개로 식별하고 다른 전략은 제외했다.',
            '원본 입력의 암호학적 SHA256과 평가 candles를 대조하고 252 사전자료·200 준비 진단·기간을 확인했다.',
            '기록 주문과 신호 대응, 다음 관측 시가와 고정 비용, 확정 수량·현금·원가·실현·잔여 손익을 독립 재정산했다.',
            '모든 평가 봉의 자산·낙폭과 기말 청산 가정·회전·관측 봉 기준 평균 및 최고 노출을 재계산했다.',
            '이전 종가→시가의 기존 수량 및 시가→종가의 체결 뒤 수량으로 실제 경로 손익을 따로 적분해 비용 환급과 대조했다.',
            '비교 보유는 기록된 실제 진입 수량을 조건으로 첫 다음 관측 시가부터의 현금·잔여·기말 청산·노출을 재계산했다.',
            'CSV는 원장과 교차검증한 요약이며 별도 독립 실행 증거로 세지 않았다.',
        ],
        'notVerified': [
            '전략 신호·guard·risk 결정을 재구현하거나 생산 harness를 재실행하지 않았다. 실제 소유 개선 뒤 성과가 유지된다는 증거가 아니다.',
            '16자리 inputHash는 원래 기록·분할 요약 사이 일치와 형식만 확인했다. 당시 serde 직렬화 및 FNV 해시 전체를 재생성하지 않았다. 원본 파일 SHA256과 candles 일치는 별도로 검증했다.',
            '대체 500+252 분할의 전체 두 번째 원장은 저장되어 있지 않다. 분할 동등성은 기존 run-summary 주장과 inputHash 교차검증만 가능하며 독립 원장 재정산으로 확대하지 않는다.',
            '비교 보유 별도 주문 원장이 없으므로 실제 수량·차단의 guard 결정은 재검증하지 않았다. 기록된 실제 수량에 조건부인 산술 검증이다.',
            '당시 v12 실행 바이너리와 전체 Rust 의존성의 동일성을 재현하지 않았다. 수정 직전 81fa00d 소스는 구현 한계 설명용이다.',
            '달력·가격 조정·배당·기업행사·원본 자료 누락 coverage·실제 발행 및 수신 시각·유동성·장중 순서는 독립 검증하지 않았다.',
            '일봉 관측·고정 FX1450·배당 없음·추정 수수료/슬리피지 모델이며 실제 운용이나 모의 주문 결과가 아니다.',
        ],
        'interpretation': [
            '완료 청산과 기말 보유 평가를 분리한다. SOXQ 연속 총자산 이익은 순실현 손실과 함께 존재하며 잔여 보유 이익 의존을 의미한다.',
            '복수 실제 청산이 있는 세 자산의 연속 원장은 청산 후보 연구 근거가 되지만 세 ETF와 두 비용은 독립 표본이 아니며 이미 관측한 사후 자료다.',
            '분기는 각각 초기 자본 1천만원과 252 사전자료로 새로 시작한다. 동일 가격을 공유하므로 연속 실행의 추가 독립 표본이나 합성 포트폴리오 수익으로 세지 않는다.',
            '비교 보유와 요청 수량이 같더라도 진입 시점·실제 수량·평균 노출이 다르다. 초과수익을 동일 위험으로 보정한 우월성으로 해석하지 않는다.',
            '수정 전 Trend는 bool 소유와 설정 수량 청산으로 실제 부분 보유·원가·수익 보호·보유 나이를 표현할 수 없다. 원장 산술 일치는 이 구현 한계나 운영 안정성을 해소하지 않는다.',
            '현재 원장은 새 확인 청산·수익 보호·최대 보유 후보의 성과 증거가 아니다. 후보별 생산 연속·분기 실행과 독립 감사가 따로 필요하다.',
        ],
    }
    OUT.write_text(json.dumps(numbers(report), ensure_ascii=False, indent=2) + '\n')
    print(json.dumps({'output': str(OUT.relative_to(ROOT)), 'assertions': report['assertions'],
                      'errorCount': report['errorCount'], 'rows': len(records)}, ensure_ascii=False))
    if errors:
        print(json.dumps(numbers(errors[:12]), ensure_ascii=False, indent=2))
        raise SystemExit(1)


if __name__ == '__main__':
    main()
