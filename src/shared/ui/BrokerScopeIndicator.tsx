import Stack from '@mui/material/Stack'
import Chip from '@mui/material/Chip'
import Tooltip from '@mui/material/Tooltip'
import AccountBalanceIcon from '@mui/icons-material/AccountBalance'
import BadgeIcon from '@mui/icons-material/Badge'
import BusinessCenterIcon from '@mui/icons-material/BusinessCenter'
import SecurityIcon from '@mui/icons-material/Security'
import type { SxProps, Theme } from '@mui/material/styles'

import type { AppConfigView, BrokerId } from '../api/types'

interface BrokerScopeIndicatorProps {
  appConfig?: AppConfigView | null
  compact?: boolean
  sx?: SxProps<Theme>
}

function brokerLabel(brokerId?: BrokerId) {
  switch (brokerId) {
    case 'kis':
      return 'KIS'
    case 'toss':
      return 'Toss'
    default:
      return 'Broker'
  }
}

export function BrokerScopeIndicator({ appConfig, compact = false, sx }: BrokerScopeIndicatorProps) {
  const brokerId = appConfig?.active_broker_id
  const profileName = appConfig?.active_profile_name ?? '프로파일 미선택'
  const accountId = appConfig?.active_broker_account_id ?? '계좌 미설정'
  const size = compact ? 'small' : 'medium'

  return (
    <Stack
      direction="row"
      flexWrap="wrap"
      alignItems="center"
      gap={0.75}
      sx={sx}
      aria-label="현재 broker scope"
    >
      <Tooltip title="현재 broker">
        <Chip
          icon={<BusinessCenterIcon />}
          label={brokerLabel(brokerId)}
          color={brokerId === 'toss' ? 'info' : brokerId === 'kis' ? 'primary' : 'default'}
          size={size}
          variant={brokerId ? 'filled' : 'outlined'}
        />
      </Tooltip>
      <Tooltip title="활성 프로파일">
        <Chip
          icon={<BadgeIcon />}
          label={profileName}
          size={size}
          variant="outlined"
          color={appConfig?.active_profile_id ? 'default' : 'warning'}
        />
      </Tooltip>
      <Tooltip title="활성 broker 계좌">
        <Chip
          icon={<AccountBalanceIcon />}
          label={accountId}
          size={size}
          variant="outlined"
          color={appConfig?.active_broker_account_id ? 'default' : 'warning'}
        />
      </Tooltip>
      {brokerId === 'kis' && appConfig && (
        <Tooltip title="KIS 거래 모드">
          <Chip
            icon={<SecurityIcon />}
            label={appConfig.kis_is_paper_trading ? '모의투자' : '실전투자'}
            color={appConfig.kis_is_paper_trading ? 'default' : 'warning'}
            size={size}
            variant="outlined"
          />
        </Tooltip>
      )}
    </Stack>
  )
}
