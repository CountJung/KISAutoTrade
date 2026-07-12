import Alert from '@mui/material/Alert'
import Button from '@mui/material/Button'
import Chip from '@mui/material/Chip'
import Divider from '@mui/material/Divider'
import Paper from '@mui/material/Paper'
import Stack from '@mui/material/Stack'
import Typography from '@mui/material/Typography'
import LaunchIcon from '@mui/icons-material/Launch'
import { useNavigate } from '@tanstack/react-router'

import { useProfiles } from '../../../api/hooks'
import type { AppConfigView } from '../../../api/types'

export function DashboardTossVerificationPanel({
  appConfig,
}: {
  appConfig: AppConfigView | undefined
}) {
  const navigate = useNavigate()
  const isTossActive = appConfig?.active_broker_id === 'toss'
  const { data: profiles = [] } = useProfiles({ enabled: isTossActive })

  if (!isTossActive) return null

  const activeProfile = profiles.find((profile) => profile.id === appConfig?.active_profile_id) ?? null
  const consentReady = activeProfile?.live_trading_consent ?? false

  return (
    <Paper sx={{ p: 2.5, mb: 2 }}>
      <Stack direction="row" alignItems="center" spacing={1} mb={1.5} flexWrap="wrap">
        <Typography variant="subtitle1" fontWeight={600}>
          Toss 소액매매 검증
        </Typography>
        <Chip
          size="small"
          label={consentReady ? '실거래 동의 저장됨' : '실거래 동의 필요'}
          color={consentReady ? 'success' : 'warning'}
          variant="outlined"
          sx={{ height: 20, fontSize: '0.7rem' }}
        />
        {appConfig?.active_broker_account_id && (
          <Typography variant="caption" color="text.secondary">
            accountSeq {appConfig.active_broker_account_id}
          </Typography>
        )}
      </Stack>
      <Divider sx={{ mb: 1.5 }} />

      <Alert severity="info" sx={{ mb: 1.5 }}>
        소액매매 검증(1주 시장가 매수 등)은 수동거래 페이지에서 진행합니다. 종목 검색, 주문 전
        사전검증, 실거래 동의 확인을 통과하면 실제 주문이 제출됩니다.
        {!consentReady && ' 실거래 동의는 Settings의 프로파일 편집에서 저장할 수 있습니다.'}
      </Alert>

      <Button
        variant="outlined"
        size="small"
        startIcon={<LaunchIcon />}
        onClick={() => void navigate({ to: '/trading' })}
      >
        수동거래 페이지로 이동
      </Button>
    </Paper>
  )
}
