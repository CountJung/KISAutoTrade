import { create } from 'zustand'

export type TradingStatus = 'idle' | 'running' | 'stopping' | 'error'

interface TradingState {
  status: TradingStatus
  activeStrategies: string[]
  errorMessage: string | null
  setStatus: (status: TradingStatus) => void
  setActiveStrategies: (strategies: string[]) => void
  setError: (msg: string | null) => void
}

export const useTradingStore = create<TradingState>((set) => ({
  status: 'idle',
  activeStrategies: [],
  errorMessage: null,
  setStatus: (status) => set({ status }),
  setActiveStrategies: (activeStrategies) => set({ activeStrategies }),
  setError: (errorMessage) => set({ errorMessage }),
}))
