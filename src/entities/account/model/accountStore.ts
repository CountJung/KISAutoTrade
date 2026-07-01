import { create } from 'zustand'

interface AccountState {
  totalBalance: number
  availableCash: number
  stockValue: number
  totalProfitLoss: number
  totalProfitRate: number
  isLoading: boolean
  lastUpdatedAt: string | null
  setBalance: (data: Partial<Omit<AccountState, 'setBalance' | 'isLoading'>>) => void
  setLoading: (loading: boolean) => void
}

export const useAccountStore = create<AccountState>((set) => ({
  totalBalance: 0,
  availableCash: 0,
  stockValue: 0,
  totalProfitLoss: 0,
  totalProfitRate: 0,
  isLoading: false,
  lastUpdatedAt: null,
  setBalance: (data) => set((state) => ({ ...state, ...data })),
  setLoading: (isLoading) => set({ isLoading }),
}))
