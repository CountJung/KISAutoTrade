import Drawer from '@mui/material/Drawer'
import List from '@mui/material/List'
import ListItem from '@mui/material/ListItem'
import ListItemButton from '@mui/material/ListItemButton'
import ListItemIcon from '@mui/material/ListItemIcon'
import ListItemText from '@mui/material/ListItemText'
import Divider from '@mui/material/Divider'
import Typography from '@mui/material/Typography'
import Box from '@mui/material/Box'
import Stack from '@mui/material/Stack'
import Chip from '@mui/material/Chip'
import Tooltip from '@mui/material/Tooltip'
import DashboardIcon from '@mui/icons-material/Dashboard'
import TrendingUpIcon from '@mui/icons-material/TrendingUp'
import AutoAwesomeIcon from '@mui/icons-material/AutoAwesome'
import HistoryIcon from '@mui/icons-material/History'
import ArticleIcon from '@mui/icons-material/Article'
import SettingsIcon from '@mui/icons-material/Settings'
import { useNavigate, useLocation } from '@tanstack/react-router'
import { useAppConfig } from '../../api/hooks'

interface NavItem {
  label: string
  path: string
  icon: React.ReactNode
}

const NAV_ITEMS: NavItem[] = [
  { label: 'Dashboard', path: '/', icon: <DashboardIcon /> },
  { label: 'Trading', path: '/trading', icon: <TrendingUpIcon /> },
  { label: 'Strategy', path: '/strategy', icon: <AutoAwesomeIcon /> },
  { label: 'History', path: '/history', icon: <HistoryIcon /> },
  { label: 'Log', path: '/log', icon: <ArticleIcon /> },
]

interface SidebarProps {
  drawerWidth: number
  mobileOpen: boolean
  onMobileClose: () => void
}

function DrawerContent({ drawerWidth }: { drawerWidth: number }) {
  const navigate = useNavigate()
  const location = useLocation()
  const { data: appConfig } = useAppConfig()

  return (
    <Box sx={{ width: drawerWidth, display: 'flex', flexDirection: 'column', height: '100%', overflow: 'hidden' }}>
      {/* 로고 영역 */}
      <Box sx={{ px: 2, py: 2.5, overflow: 'hidden' }}>
        <Typography variant="subtitle2" color="primary" fontWeight={700} noWrap>
          AutoConditionTrade
        </Typography>
        {appConfig?.active_profile_name ? (
          <Stack direction="row" spacing={0.5} alignItems="center" mt={0.5}>
            <Typography variant="caption" color="text.secondary" noWrap sx={{ flex: 1, minWidth: 0 }}>
              {appConfig.active_profile_name}
            </Typography>
            <Chip
              size="small"
              label={appConfig.kis_is_paper_trading ? '모의' : '실전'}
              color={appConfig.kis_is_paper_trading ? 'warning' : 'success'}
              sx={{ height: 16, fontSize: '0.6rem', flexShrink: 0 }}
            />
          </Stack>
        ) : (
          <Typography variant="caption" color="text.secondary">
            개인용 자동매매
          </Typography>
        )}
      </Box>
      <Divider />

      {/* 주요 메뉴 */}
      <List dense sx={{ flexGrow: 1, pt: 1 }}>
        {NAV_ITEMS.map((item) => {
          const isActive = location.pathname === item.path
          return (
            <ListItem key={item.path} disablePadding>
              <Tooltip title={item.label} placement="right" arrow>
                <ListItemButton
                  selected={isActive}
                  onClick={() => navigate({ to: item.path })}
                  sx={{
                    mx: 1,
                    borderRadius: 1,
                    '&.Mui-selected': {
                      bgcolor: 'primary.main',
                      color: 'primary.contrastText',
                      '& .MuiListItemIcon-root': { color: 'primary.contrastText' },
                      '&:hover': { bgcolor: 'primary.dark' },
                    },
                  }}
                >
                  <ListItemIcon sx={{ minWidth: 36 }}>{item.icon}</ListItemIcon>
                  <ListItemText primary={item.label} />
                </ListItemButton>
              </Tooltip>
            </ListItem>
          )
        })}
      </List>

      <Divider />

      {/* 설정 */}
      <List dense>
        <ListItem disablePadding>
          <ListItemButton
            selected={location.pathname === '/settings'}
            onClick={() => navigate({ to: '/settings' })}
            sx={{ mx: 1, borderRadius: 1 }}
          >
            <ListItemIcon sx={{ minWidth: 36 }}>
              <SettingsIcon />
            </ListItemIcon>
            <ListItemText primary="Settings" />
          </ListItemButton>
        </ListItem>
      </List>
    </Box>
  )
}

export function Sidebar({ drawerWidth, mobileOpen, onMobileClose }: SidebarProps) {
  return (
    <Box
      component="nav"
      sx={{ width: { md: drawerWidth }, flexShrink: { md: 0 } }}
    >
      {/* 모바일용 임시 Drawer */}
      <Drawer
        variant="temporary"
        open={mobileOpen}
        onClose={onMobileClose}
        ModalProps={{ keepMounted: true }}
        sx={{
          display: { xs: 'block', md: 'none' },
          '& .MuiDrawer-paper': { width: drawerWidth, boxSizing: 'border-box', overflowX: 'hidden' },
        }}
      >
        <DrawerContent drawerWidth={drawerWidth} />
      </Drawer>

      {/* 데스크탑용 영구 Drawer */}
      <Drawer
        variant="permanent"
        sx={{
          display: { xs: 'none', md: 'block' },
          '& .MuiDrawer-paper': {
            width: drawerWidth,
            boxSizing: 'border-box',
            borderRight: '1px solid',
            borderColor: 'divider',
            overflowX: 'hidden',
          },
        }}
        open
      >
        <DrawerContent drawerWidth={drawerWidth} />
      </Drawer>
    </Box>
  )
}
