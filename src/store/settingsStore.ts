import { create } from 'zustand'
import { persist } from 'zustand/middleware'
import type { ThemeMode } from '../theme'

interface SettingsState {
  theme: ThemeMode
  discordEnabled: boolean
  notificationLevels: string[]
  setTheme: (theme: ThemeMode) => void
  setDiscordEnabled: (enabled: boolean) => void
  setNotificationLevels: (levels: string[]) => void
}

export const useSettingsStore = create<SettingsState>()(
  persist(
    (set) => ({
      theme: 'system',
      discordEnabled: false,
      notificationLevels: ['CRITICAL', 'ERROR', 'TRADE'],
      setTheme: (theme) => set({ theme }),
      setDiscordEnabled: (discordEnabled) => set({ discordEnabled }),
      setNotificationLevels: (notificationLevels) => set({ notificationLevels }),
    }),
    { name: 'act-settings' },
  ),
)
