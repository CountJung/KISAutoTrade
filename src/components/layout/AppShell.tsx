import { useState, useMemo, useCallback, useRef } from 'react'
import { Outlet } from '@tanstack/react-router'
import Box from '@mui/material/Box'
import CssBaseline from '@mui/material/CssBaseline'
import { ThemeProvider } from '@mui/material/styles'
import { Sidebar } from './Sidebar'
import { LayoutResizer } from '../LayoutResizer'
import { createAppTheme, getResolvedMode, THEME_STORAGE_KEY } from '../../theme'
import { useSettingsStore } from '../../store/settingsStore'

const SIDEBAR_KEY = 'act:panel:sidebar:width'
const SIDEBAR_DEFAULT = 220
const SIDEBAR_MIN = 160
const SIDEBAR_MAX = 400

function readSidebarWidth(): number {
  const raw = localStorage.getItem(SIDEBAR_KEY)
  if (!raw) return SIDEBAR_DEFAULT
  const n = Number(raw)
  return Number.isFinite(n) ? Math.max(SIDEBAR_MIN, Math.min(SIDEBAR_MAX, n)) : SIDEBAR_DEFAULT
}

export function AppShell() {
  const theme = useSettingsStore((s) => s.theme)
  const [mobileOpen, setMobileOpen] = useState(false)
  const [sidebarWidth, setSidebarWidth] = useState(readSidebarWidth)

  // onResizeEnd 클로저에서 최신 width를 읽기 위한 ref
  const sidebarWidthRef = useRef(sidebarWidth)
  sidebarWidthRef.current = sidebarWidth

  const muiTheme = useMemo(() => {
    const resolved = getResolvedMode(theme)
    document.documentElement.dataset.theme = resolved
    localStorage.setItem(THEME_STORAGE_KEY, theme)
    return createAppTheme(resolved)
  }, [theme])

  const handleSidebarResize = useCallback((delta: number) => {
    setSidebarWidth((w) =>
      Math.max(SIDEBAR_MIN, Math.min(SIDEBAR_MAX, w + delta))
    )
  }, [])

  const handleSidebarResizeEnd = useCallback(() => {
    localStorage.setItem(SIDEBAR_KEY, String(sidebarWidthRef.current))
  }, [])

  return (
    <ThemeProvider theme={muiTheme}>
      <CssBaseline />
      <Box sx={{ display: 'flex', height: '100vh', overflow: 'hidden' }}>
        <Sidebar
          drawerWidth={sidebarWidth}
          mobileOpen={mobileOpen}
          onMobileClose={() => setMobileOpen(false)}
        />
        <LayoutResizer
          direction="horizontal"
          onResize={handleSidebarResize}
          onResizeEnd={handleSidebarResizeEnd}
        />
        <Box
          component="main"
          sx={{
            flexGrow: 1,
            overflow: 'auto',
            bgcolor: 'background.default',
            p: 2,
          }}
        >
          <Outlet />
        </Box>
      </Box>
    </ThemeProvider>
  )
}
