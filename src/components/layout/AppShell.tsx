import { useState, useMemo, useCallback, useRef } from 'react'
import { Outlet, useLocation, useNavigate } from '@tanstack/react-router'
import Box from '@mui/material/Box'
import CssBaseline from '@mui/material/CssBaseline'
import Alert from '@mui/material/Alert'
import AlertTitle from '@mui/material/AlertTitle'
import Button from '@mui/material/Button'
import AppBar from '@mui/material/AppBar'
import Toolbar from '@mui/material/Toolbar'
import Typography from '@mui/material/Typography'
import IconButton from '@mui/material/IconButton'
import Paper from '@mui/material/Paper'
import BottomNavigation from '@mui/material/BottomNavigation'
import BottomNavigationAction from '@mui/material/BottomNavigationAction'
import useMediaQuery from '@mui/material/useMediaQuery'
import MenuIcon from '@mui/icons-material/Menu'
import DashboardIcon from '@mui/icons-material/Dashboard'
import TrendingUpIcon from '@mui/icons-material/TrendingUp'
import AutoAwesomeIcon from '@mui/icons-material/AutoAwesome'
import HistoryIcon from '@mui/icons-material/History'
import ArticleIcon from '@mui/icons-material/Article'
import SettingsIcon from '@mui/icons-material/Settings'
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

/** 모바일 하단 내비게이션 항목 */
const BOTTOM_NAV_ITEMS = [
  { label: 'Dashboard', path: '/',         icon: <DashboardIcon /> },
  { label: 'Trading',   path: '/trading',   icon: <TrendingUpIcon /> },
  { label: 'Strategy',  path: '/strategy',  icon: <AutoAwesomeIcon /> },
  { label: 'History',   path: '/history',   icon: <HistoryIcon /> },
  { label: 'Log',       path: '/log',       icon: <ArticleIcon /> },
  { label: 'Settings',  path: '/settings',  icon: <SettingsIcon /> },
]

function readSidebarWidth(): number {
  const raw = localStorage.getItem(SIDEBAR_KEY)
  if (!raw) return SIDEBAR_DEFAULT
  const n = Number(raw)
  return Number.isFinite(n) ? Math.max(SIDEBAR_MIN, Math.min(SIDEBAR_MAX, n)) : SIDEBAR_DEFAULT
}

export function AppShell() {
  // CSS 브레이크포인트 대신 JS window.matchMedia 기반 감지:
  // 브라우저/WebView/줌 레벨 무관하게 정확하게 동작하며, CSS 우선순위 문제 없음.
  // defaultMatches:true → 첫 렌더링 시 데스크탑 레이아웃으로 시작해 flash 방지
  const isDesktop = useMediaQuery('(min-width:900px)', { defaultMatches: true })

  const theme = useSettingsStore((s) => s.theme)
  const [mobileOpen, setMobileOpen] = useState(false)
  const [sidebarWidth, setSidebarWidth] = useState(readSidebarWidth)
  const [updateDismissed, setUpdateDismissed] = useState(false)

  const { data: updateInfo } = useUpdateCheck()
  const showUpdateBanner = !updateDismissed && updateInfo?.hasUpdate === true

  const location = useLocation()
  const navigate = useNavigate()

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

        {/* 모바일 전용 상단 바 — JS 기반으로 isDesktop이 false일 때만 렌더링 */}
        {!isDesktop && (
          <AppBar
            position="static"
            elevation={0}
            sx={{
              bgcolor: 'background.paper',
              borderBottom: '1px solid',
              borderColor: 'divider',
              color: 'text.primary',
              flexShrink: 0,
            }}
          >
            <Toolbar variant="dense" sx={{ minHeight: 48, gap: 1 }}>
              <IconButton
                size="small"
                onClick={() => setMobileOpen(true)}
                aria-label="메뉴 열기"
              >
                <MenuIcon fontSize="small" />
              </IconButton>
              <Typography variant="subtitle2" fontWeight={700} color="primary" noWrap>
                KISAutoTrade
              </Typography>
            </Toolbar>
          </AppBar>
        )}

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

        {/* 사이드바 + 메인 컨텐츠 */}
        <Box sx={{ display: 'flex', flex: 1, overflow: 'hidden' }}>
          <Sidebar
            isDesktop={isDesktop}
            drawerWidth={sidebarWidth}
            mobileOpen={mobileOpen}
            onMobileClose={() => setMobileOpen(false)}
          />
          {/* 리사이저: JS로 데스크탑에서만 렌더링 */}
          {isDesktop && (
            <LayoutResizer
              direction="horizontal"
              onResize={handleSidebarResize}
              onResizeEnd={handleSidebarResizeEnd}
            />
          )}
          <Box
            component="main"
            sx={{
              flexGrow: 1,
              overflow: 'auto',
              bgcolor: 'background.default',
              p: 2,
              // 모바일 하단 내비게이션(60px) 높이만큼 여백 확보
              pb: isDesktop ? 2 : 9,
            }}
          >
            <Outlet />
          </Box>
        </Box>

        {/* 모바일 하단 내비게이션 — JS 기반으로 isDesktop이 false일 때만 렌더링 */}
        {!isDesktop && (
          <Paper
            elevation={8}
            sx={{ position: 'fixed', bottom: 0, left: 0, right: 0, zIndex: 1300 }}
          >
            <BottomNavigation
              value={location.pathname}
              onChange={(_evt, newPath: unknown) => {
                if (typeof newPath === 'string') void navigate({ to: newPath })
              }}
              sx={{ height: 60 }}
            >
              {BOTTOM_NAV_ITEMS.map((item) => (
                <BottomNavigationAction
                  key={item.path}
                  value={item.path}
                  icon={item.icon}
                  label={item.label}
                  sx={{
                    minWidth: 'auto',
                    px: 0.5,
                    '& .MuiBottomNavigationAction-label': { fontSize: '0.6rem' },
                  }}
                />
              ))}
            </BottomNavigation>
          </Paper>
        )}      </Box>
    </ThemeProvider>
  )
}