import { useState } from 'react'
import * as cmd from '../../../api/commands'
import type { AppConfigView, OverseasExchange, StockSearchItem } from '../../../api/types'
import { useKeyedRequestGate } from '../../../shared/lib'

// Page-local lookup lifecycle; no storage, strategy mutation or financial draft ownership.
export function useStrategyUsSearch({ input, market, config, selectedSymbol, onSelect }: {
  input: string
  market: string
  config: AppConfigView | undefined
  selectedSymbol: string | undefined
  onSelect: (item: StockSearchItem) => void
}) {
  const [error, setError] = useState<string | null>(null)
  const gate = useKeyedRequestGate(JSON.stringify([input, market, selectedSymbol, config?.active_broker_id,
    config?.active_profile_id, config?.active_broker_account_id, config?.kis_is_paper_trading]))
  const search = async () => {
    const ticker = input.trim().toUpperCase()
    if (!ticker) return
    const ticket = gate.begin()
    if (!ticket) return
    setError(null)
    try {
      if (config?.active_broker_id === 'toss') {
        try {
          const safety = await cmd.getTossStockSafety(ticker)
          if (!gate.current(ticket)) return
          if (safety.stockInfo) {
            if (!gate.finish(ticket)) return
            onSelect({ pdno: ticker, prdt_name: safety.stockInfo.name || ticker })
            return
          }
        } catch { if (!gate.current(ticket)) return }
        if (gate.current(ticket)) setError(`Toss 종목 정보에서 "${ticker}"을 찾을 수 없습니다.`)
        return
      }
      const exchanges: OverseasExchange[] = ['NAS', 'NYS', 'AMS']
      for (const exchange of exchanges) {
        if (!gate.current(ticket)) return
        try {
          const response = await cmd.getOverseasPrice(ticker, exchange)
          if (!gate.current(ticket)) return
          const validPrice = parseFloat(response.last) > 0
          const hasName = response.name && response.name.trim().length > 0
          if (response && (validPrice || hasName)) {
            if (!gate.finish(ticket)) return
            onSelect({ pdno: ticker, prdt_name: response.name || ticker })
            return
          }
        } catch { if (!gate.current(ticket)) return }
      }
      if (gate.current(ticket)) setError(`"${ticker}"을 NAS·NYS·AMEX에서 찾을 수 없습니다.`)
    } finally { gate.finish(ticket) }
  }
  return { search, searching: gate.pending(), error, setError }
}
