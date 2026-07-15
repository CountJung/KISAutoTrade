import type { BrokerId, StrategyExperimentSnapshot } from '../../../api/types'

const STORAGE_VERSION = 'v1'
const KEY_PREFIX = `act:strategy-experiments:${STORAGE_VERSION}`
const INDEX_KEY = `${KEY_PREFIX}:index`
const MAX_SCOPE_KEYS = 12

export type ExperimentScope = {
  brokerId: BrokerId
  brokerAccountId: string | null
  strategyId: string
  symbol: string
}

function normalizePart(value: string | null) {
  return encodeURIComponent((value ?? 'none').trim().toLowerCase())
}

function storageKey(scope: ExperimentScope) {
  return [
    KEY_PREFIX,
    normalizePart(scope.brokerId),
    normalizePart(scope.brokerAccountId),
    normalizePart(scope.strategyId),
    normalizePart(scope.symbol),
  ].join(':')
}

export function loadExperimentSlots(scope: ExperimentScope): Partial<Record<'A' | 'B', StrategyExperimentSnapshot>> {
  if (typeof window === 'undefined') return {}
  try {
    const raw = window.localStorage.getItem(storageKey(scope))
    if (!raw) return {}
    const parsed = JSON.parse(raw) as Partial<Record<'A' | 'B', StrategyExperimentSnapshot>>
    return parsed && typeof parsed === 'object' ? parsed : {}
  } catch {
    return {}
  }
}

function compactSnapshot(snapshot: StrategyExperimentSnapshot): StrategyExperimentSnapshot {
  return {
    ...snapshot,
    backtest: {
      ...snapshot.backtest,
      trades: snapshot.backtest.trades.slice(0, 100),
      equityCurve: snapshot.backtest.equityCurve.filter((_, index, rows) => (
        rows.length <= 200 || index % Math.ceil(rows.length / 200) === 0 || index === rows.length - 1
      )),
    },
  }
}

function readIndex(): string[] {
  try {
    const parsed = JSON.parse(window.localStorage.getItem(INDEX_KEY) ?? '[]')
    return Array.isArray(parsed) ? parsed.filter((key): key is string => typeof key === 'string') : []
  } catch {
    return []
  }
}

export function saveExperimentSlot(scope: ExperimentScope, snapshot: StrategyExperimentSnapshot): boolean {
  if (typeof window === 'undefined') return false
  const key = storageKey(scope)
  const current = loadExperimentSlots(scope)
  current[snapshot.slot] = compactSnapshot(snapshot)
  try {
    const index = readIndex().filter((item) => item !== key)
    index.push(key)
    while (index.length > MAX_SCOPE_KEYS) {
      const evicted = index.shift()
      if (evicted) window.localStorage.removeItem(evicted)
    }
    window.localStorage.setItem(key, JSON.stringify(current))
    window.localStorage.setItem(INDEX_KEY, JSON.stringify(index))
    return true
  } catch {
    return false
  }
}

export function clearExperimentSlots(scope: ExperimentScope) {
  if (typeof window === 'undefined') return
  const key = storageKey(scope)
  window.localStorage.removeItem(key)
  try {
    window.localStorage.setItem(INDEX_KEY, JSON.stringify(readIndex().filter((item) => item !== key)))
  } catch {
    // 저장소가 비활성화된 환경에서는 현재 scope 제거만으로 충분하다.
  }
}
