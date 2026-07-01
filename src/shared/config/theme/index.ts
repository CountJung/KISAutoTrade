import { createTheme, Theme } from '@mui/material/styles'

export type ThemeMode = 'light' | 'dark' | 'system'

/** localStorage 키 */
export const THEME_STORAGE_KEY = 'act-theme'

/**
 * system 모드를 실제 light/dark로 해석
 */
export function getResolvedMode(mode: ThemeMode): 'light' | 'dark' {
  if (mode === 'system') {
    return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
  }
  return mode
}

/**
 * MUI 테마 생성
 */
export function createAppTheme(resolvedMode: 'light' | 'dark'): Theme {
  return createTheme({
    palette: {
      mode: resolvedMode,
      primary: {
        main: resolvedMode === 'dark' ? '#90caf9' : '#1565c0',
      },
      secondary: {
        main: resolvedMode === 'dark' ? '#f48fb1' : '#c62828',
      },
      background: {
        default: resolvedMode === 'dark' ? '#121212' : '#f5f5f5',
        paper: resolvedMode === 'dark' ? '#1e1e1e' : '#ffffff',
      },
    },
    typography: {
      fontFamily: '"Noto Sans KR", "Roboto", "Helvetica", "Arial", sans-serif',
      fontSize: 14,
    },
    shape: {
      borderRadius: 8,
    },
    components: {
      MuiDrawer: {
        styleOverrides: {
          paper: {
            backgroundImage: 'none',
          },
        },
      },
    },
  })
}
