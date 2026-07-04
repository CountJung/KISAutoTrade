import { alpha, createTheme, Theme } from '@mui/material/styles'

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
      MuiCssBaseline: {
        styleOverrides: (theme) => {
          const track = theme.palette.background.default
          const thumb = alpha(theme.palette.text.primary, theme.palette.mode === 'dark' ? 0.42 : 0.32)
          const thumbHover = alpha(theme.palette.text.primary, theme.palette.mode === 'dark' ? 0.58 : 0.46)
          const border = theme.palette.background.paper

          return {
            html: {
              height: '100%',
              overflow: 'hidden',
              scrollbarColor: `${thumb} ${track}`,
              scrollbarGutter: 'stable both-edges',
            },
            body: {
              height: '100%',
              overflow: 'hidden',
              scrollbarColor: `${thumb} ${track}`,
              scrollbarGutter: 'stable both-edges',
            },
            '#root': {
              height: '100%',
              overflow: 'hidden',
            },
            '*': {
              scrollbarWidth: 'thin',
              scrollbarColor: `${thumb} ${track}`,
            },
            '*::-webkit-scrollbar': {
              width: 10,
              height: 10,
            },
            '*::-webkit-scrollbar-track': {
              backgroundColor: track,
            },
            '*::-webkit-scrollbar-thumb': {
              backgroundColor: thumb,
              borderRadius: 8,
              border: `2px solid ${border}`,
              backgroundClip: 'padding-box',
            },
            '*::-webkit-scrollbar-thumb:hover': {
              backgroundColor: thumbHover,
            },
            '*::-webkit-scrollbar-corner': {
              backgroundColor: track,
            },
          }
        },
      },
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
