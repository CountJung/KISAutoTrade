import { useState, useMemo, useCallback, useRef } from 'react'
import { Outlet } from '@tanstack/react-router'
import Box from '@mui/material/Box'
import CssBaseline from '@mui/material/CssBaseline'
import Alert from '@mui/material/Alert'
import AlertTitle from '@mui/material/AlertTitle'
import Button from '@mui/material/Button'
import { ThemeProvider } from '@mui/material/styles'
import { Sidebar } from './Sidebar'
import { LayoutResizer } from '../LayoutResizer'
import { createAppTheme, getResolvedMode, THEME_STORAGE_KEY } from '../../theme'
import { useSettingsStore } from '../../store/settingsStore'
import { useUpdateCheck } from '../../api/hooks'

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
  const [updateDismissed, setUpdateDismissed] = useState(false)

  const { data: updateInfo } = useUpdateCheck()
  const showUpdateBanner = !updateDismissed && updateInfo?.hasUpdate === true

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
      <Box sx={{ display: 'flex', flexDirection: 'column', height: '100vh', overflow: 'hidden' }}>
        {showUpdateBanner && (
          <Alert
            severity="info"
            sx={{ borderRadius: 0, flexShrink: 0 }}
            action={
              <Box sx={{ display: 'flex', gap: 1, alignItems: 'center' }}>
                <Button
                  size="small"
                  color="inherit"
                  href={updateInfo!.releaseUrl}
                  target="_blank"
                  rel="noopener noreferrer"
                >
                  다운로드 v{updateInfo!.latestVersion}
                </Button>
                <Button size="small" color="inherit" onClick={() => setUpdateDismissed(true)}>
                  닫기
                </Button>
              </Box>
            }
          >
            <AlertTitle>새 버전 출시</AlertTitle>
            현재 v{updateInfo!.currentVersion} → 최신 v{updateInfo!.latestVersion} 로 업데이트가 가능합니다.
          </Alert>
        )}
        <Box sx={{ display: 'flex', flex: 1, overflow: 'hidden' }}>
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
      </Box>
    </ThemeProvider>
  )
}
