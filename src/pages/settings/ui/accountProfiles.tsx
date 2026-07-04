import { useState } from 'react'

import Alert from '@mui/material/Alert'
import Box from '@mui/material/Box'
import Button from '@mui/material/Button'
import Chip from '@mui/material/Chip'
import CircularProgress from '@mui/material/CircularProgress'
import Dialog from '@mui/material/Dialog'
import DialogActions from '@mui/material/DialogActions'
import DialogContent from '@mui/material/DialogContent'
import DialogTitle from '@mui/material/DialogTitle'
import Divider from '@mui/material/Divider'
import IconButton from '@mui/material/IconButton'
import Paper from '@mui/material/Paper'
import Stack from '@mui/material/Stack'
import Tooltip from '@mui/material/Tooltip'
import Typography from '@mui/material/Typography'
import AddIcon from '@mui/icons-material/Add'
import CheckCircleIcon from '@mui/icons-material/CheckCircle'
import DeleteIcon from '@mui/icons-material/Delete'
import EditIcon from '@mui/icons-material/Edit'
import ErrorIcon from '@mui/icons-material/Error'
import RadioButtonCheckedIcon from '@mui/icons-material/RadioButtonChecked'
import RadioButtonUncheckedIcon from '@mui/icons-material/RadioButtonUnchecked'
import SyncIcon from '@mui/icons-material/Sync'

import {
  useAppConfig,
  useCheckConfig,
  useCheckTossProfileConnection,
  useDeleteProfile,
  useDetectProfileTradingType,
  useProfiles,
  useSetActiveProfile,
  useTradingStatus,
  useUpdateProfile,
} from '../../../api/hooks'
import type {
  AccountProfileView,
  BrokerId,
  TossConnectionDiagnostic,
} from '../../../api/types'

import { AddProfileDialog, EditProfileDialog } from './profileDialogs'
import { brokerLabel, cmdErrMsg } from './profileUtils'
import { Section } from './section'

// ── 프로파일 카드 ──────────────────────────────────────────────────
function ProfileCard({
  profile,
  onEdit,
  onDelete,
  onSetActive,
  isRunning,
  tradingProfileId,
}: {
  profile: AccountProfileView
  onEdit: (p: AccountProfileView) => void
  onDelete: (p: AccountProfileView) => void
  onSetActive: (id: string) => void
  isRunning: boolean
  tradingProfileId: string | null
}) {
  const { isPending: activating } = useSetActiveProfile()
  const { mutate: detectProfile, isPending: detecting } = useDetectProfileTradingType()
  const { mutate: checkTossProfile, isPending: checkingToss } = useCheckTossProfileConnection()
  const { mutate: updateProfile, isPending: updatingMode } = useUpdateProfile()
  const [detectError, setDetectError] = useState<string | null>(null)
  const [tossDiagnostic, setTossDiagnostic] = useState<TossConnectionDiagnostic | null>(null)

  const isActiveTrading = isRunning && tradingProfileId === profile.id
  const isKisProfile = profile.broker_id === 'kis'
  const isTossProfile = profile.broker_id === 'toss'
  const manualTargetIsPaper = !profile.is_paper_trading

  const handleDetect = () => {
    setDetectError(null)
    detectProfile(profile.id, {
      onError: (e) => setDetectError(cmdErrMsg(e)),
    })
  }

  const handleManualModeSwitch = () => {
    setDetectError(null)
    updateProfile(
      {
        id: profile.id,
        is_paper_trading: manualTargetIsPaper,
      },
      {
        onError: (e) => setDetectError(cmdErrMsg(e)),
      },
    )
  }

  const handleTossDiagnostic = () => {
    setDetectError(null)
    setTossDiagnostic(null)
    checkTossProfile(profile.id, {
      onSuccess: setTossDiagnostic,
      onError: (e) => setDetectError(cmdErrMsg(e)),
    })
  }

  return (
    <Paper
      variant="outlined"
      sx={{
        p: 2,
        border: profile.is_active ? '2px solid' : undefined,
        borderColor: profile.is_active ? 'primary.main' : undefined,
        bgcolor: profile.is_active ? 'action.selected' : undefined,
      }}
    >
      <Stack direction="row" alignItems="flex-start" justifyContent="space-between">
        <Stack spacing={0.5} flex={1}>
          <Stack direction="row" spacing={1} alignItems="center" flexWrap="wrap">
            {profile.is_active ? (
              <RadioButtonCheckedIcon color="primary" fontSize="small" />
            ) : (
              <Tooltip title="클릭하여 이 프로파일로 전환">
                <span>
                  <IconButton
                    size="small"
                    onClick={() => onSetActive(profile.id)}
                    disabled={activating}
                    sx={{ p: 0 }}
                  >
                    <RadioButtonUncheckedIcon color="disabled" fontSize="small" />
                  </IconButton>
                </span>
              </Tooltip>
            )}
            <Typography variant="body1" fontWeight={600}>
              {profile.name}
            </Typography>
            <Chip
              size="small"
              label={brokerLabel(profile.broker_id)}
              color={profile.broker_id === 'kis' ? 'info' : 'secondary'}
              variant="outlined"
            />
            {isKisProfile ? (
              <Chip
                size="small"
                label={profile.is_paper_trading ? '모의투자' : '실전투자'}
                color={profile.is_paper_trading ? 'warning' : 'primary'}
              />
            ) : (
              <>
                <Chip size="small" label="read-only 진단" color="secondary" variant="outlined" />
                <Chip
                  size="small"
                  label={profile.live_trading_consent ? '실거래 동의 저장' : '실거래 동의 없음'}
                  color={profile.live_trading_consent ? 'warning' : 'default'}
                  variant="outlined"
                />
              </>
            )}
            {!profile.is_configured && (
              <Chip size="small" label="키 미설정" color="error" variant="outlined" />
            )}
            {isActiveTrading ? (
              <Chip
                size="small"
                label="● 자동매매 실행 중"
                color="success"
                sx={{ fontWeight: 700, letterSpacing: '0.01em' }}
              />
            ) : isRunning ? (
              <Chip
                size="small"
                label="매매 대기"
                color="default"
                variant="outlined"
                sx={{ opacity: 0.55 }}
              />
            ) : null}
            {(()=>{
              if (!isKisProfile) return null
              const digits = (profile.account_no ?? '').replace('-', '')
              const suffix = digits.length >= 10 ? digits.slice(8) : ''
              if (suffix === '22' || suffix === '29') {
                return (
                  <Chip
                    size="small"
                    label={suffix === '22' ? 'IRP/개인연금' : '퇴직연금'}
                    color="warning"
                    variant="outlined"
                    title="퇴직연금 계좌는 KIS Open API 주문이 불가합니다"
                  />
                )
              }
              return null
            })()}
            {/* 실전/모의 자동 감지 버튼 */}
            {isKisProfile && profile.is_configured && (
              <Tooltip title="저장된 키로 실전/모의 여부를 자동 감지하여 즉시 업데이트합니다">
                <span>
                  <Button
                    size="small"
                    variant="text"
                    color="inherit"
                    startIcon={detecting
                      ? <CircularProgress size={12} color="inherit" />
                      : <SyncIcon fontSize="small" />
                    }
                    onClick={handleDetect}
                    disabled={detecting || updatingMode}
                    sx={{ fontSize: '0.7rem', px: 0.5, minWidth: 0 }}
                  >
                    {detecting ? '감지 중...' : '자동 감지'}
                  </Button>
                </span>
              </Tooltip>
            )}
            {isKisProfile && (
              <Tooltip
                title={`감지 결과와 무관하게 ${manualTargetIsPaper ? '모의투자' : '실전투자'} 모드로 즉시 저장합니다`}
              >
                <span>
                  <Button
                    size="small"
                    variant="outlined"
                    color={manualTargetIsPaper ? 'warning' : 'primary'}
                    startIcon={updatingMode
                      ? <CircularProgress size={12} color="inherit" />
                      : <SyncIcon fontSize="small" />
                    }
                    onClick={handleManualModeSwitch}
                    disabled={detecting || updatingMode}
                    sx={{ fontSize: '0.7rem', px: 0.75, minWidth: 0 }}
                  >
                    {updatingMode
                      ? '전환 중...'
                      : manualTargetIsPaper ? '모의 전환' : '실전 전환'}
                  </Button>
                </span>
              </Tooltip>
            )}
            {isTossProfile && (
              <Tooltip title="OpenAPI 스펙, 토큰 발급, 계좌 조회, 잔고 조회를 순서대로 확인합니다">
                <span>
                  <Button
                    size="small"
                    variant="outlined"
                    color={tossDiagnostic?.is_ready ? 'success' : 'secondary'}
                    startIcon={checkingToss
                      ? <CircularProgress size={12} color="inherit" />
                      : <SyncIcon fontSize="small" />
                    }
                    onClick={handleTossDiagnostic}
                    disabled={checkingToss}
                    sx={{ fontSize: '0.7rem', px: 0.75, minWidth: 0 }}
                  >
                    {checkingToss ? '진단 중...' : '연결 진단'}
                  </Button>
                </span>
              </Tooltip>
            )}
          </Stack>
          <Typography variant="body2" color="text.secondary" pl={3.5}>
            KEY: <code>{profile.app_key_masked}</code>
            &nbsp;&nbsp;계좌: <code>{profile.account_no || '(미설정)'}</code>
            &nbsp;&nbsp;Broker 계좌: <code>{profile.broker_account_id || '(미설정)'}</code>
          </Typography>
          {detectError && (
            <Typography variant="caption" color="error" pl={3.5}>
              {detectError}
            </Typography>
          )}
          {tossDiagnostic && (
            <Box pl={3.5} mt={1}>
              <Alert
                severity={tossDiagnostic.is_ready ? 'success' : 'warning'}
                sx={{ py: 0.75 }}
              >
                <Stack spacing={0.75}>
                  <Typography variant="caption">
                    OpenAPI {tossDiagnostic.openapi_version ?? '확인 실패'}
                    {typeof tossDiagnostic.accounts_count === 'number'
                      ? ` · 계좌 ${tossDiagnostic.accounts_count}개`
                      : ''}
                    {typeof tossDiagnostic.holdings_count === 'number'
                      ? ` · 보유 ${tossDiagnostic.holdings_count}개`
                      : ''}
                    {tossDiagnostic.buying_power_krw
                      ? ` · KRW ${Number(tossDiagnostic.buying_power_krw).toLocaleString('ko-KR')}`
                      : ''}
                    {tossDiagnostic.buying_power_usd
                      ? ` · USD ${tossDiagnostic.buying_power_usd}`
                      : ''}
                    {typeof tossDiagnostic.commissions_count === 'number'
                      ? ` · 수수료 ${tossDiagnostic.commissions_count}개`
                      : ''}
                  </Typography>
                  <Stack direction="row" spacing={0.5} flexWrap="wrap" useFlexGap>
                    {tossDiagnostic.steps.map((step) => (
                      <Chip
                        key={step.id}
                        size="small"
                        label={`${step.label}: ${step.ok ? 'OK' : '실패'}`}
                        color={step.ok ? 'success' : 'error'}
                        variant="outlined"
                      />
                    ))}
                  </Stack>
                  {tossDiagnostic.issues[0] && (
                    <Typography variant="caption" color="text.secondary">
                      {tossDiagnostic.issues[0]}
                    </Typography>
                  )}
                </Stack>
              </Alert>
            </Box>
          )}
        </Stack>

        <Stack direction="row" spacing={0.5} alignItems="center">
          <Tooltip title="편집">
            <IconButton size="small" onClick={() => onEdit(profile)}>
              <EditIcon fontSize="small" />
            </IconButton>
          </Tooltip>
          <Tooltip title="삭제">
            <IconButton size="small" color="error" onClick={() => onDelete(profile)}>
              <DeleteIcon fontSize="small" />
            </IconButton>
          </Tooltip>
        </Stack>
      </Stack>
    </Paper>
  )
}

// ── 메인 Settings 페이지 ───────────────────────────────────────────

export function AccountProfilesSection() {
  const { data: appConfig } = useAppConfig()
  const activeBrokerConfigured = appConfig?.active_broker_configured ?? appConfig?.kis_configured ?? false
  const activeBrokerIsKis = appConfig?.active_broker_id === 'kis'
  const activeBrokerModeLabel = activeBrokerIsKis
    ? appConfig?.kis_is_paper_trading ? '모의투자 모드' : '실전투자 모드'
    : 'read-only 진단'
  const { data: diag, refetch: recheckConfig, isFetching: diagFetching } = useCheckConfig()
  const { data: tradingStatus } = useTradingStatus()
  const { data: profiles = [], isLoading: profilesLoading } = useProfiles()
  const { mutate: setActive } = useSetActiveProfile()
  const { mutate: deleteProfile } = useDeleteProfile()
  const [addBroker, setAddBroker] = useState<BrokerId | null>(null)
  const [editProfile, setEditProfile] = useState<AccountProfileView | null>(null)
  const [deleteTarget, setDeleteTarget] = useState<AccountProfileView | null>(null)
  const kisProfiles = profiles.filter((profile) => profile.broker_id === 'kis')
  const tossProfiles = profiles.filter((profile) => profile.broker_id === 'toss')

  const handleDelete = () => {
    if (!deleteTarget) return
    deleteProfile(deleteTarget.id, {
      onSuccess: () => setDeleteTarget(null),
    })
  }

  const renderBrokerProfiles = (brokerId: BrokerId, brokerProfiles: AccountProfileView[]) => (
    <Section title={`${brokerLabel(brokerId)} 계좌 프로파일`}>
      <Stack spacing={2}>
        {profilesLoading ? (
          <CircularProgress size={24} />
        ) : brokerProfiles.length === 0 ? (
          <Alert severity="info">
            등록된 {brokerLabel(brokerId)} 계좌 프로파일이 없습니다.
          </Alert>
        ) : (
          <Stack spacing={1.5}>
            {brokerProfiles.map((profile) => (
              <ProfileCard
                key={profile.id}
                profile={profile}
                onEdit={setEditProfile}
                onDelete={setDeleteTarget}
                onSetActive={(id) => setActive(id)}
                isRunning={tradingStatus?.isRunning ?? false}
                tradingProfileId={tradingStatus?.tradingProfileId ?? null}
              />
            ))}
          </Stack>
        )}

        <Box>
          <Button
            variant="contained"
            startIcon={<AddIcon />}
            onClick={() => setAddBroker(brokerId)}
            size="small"
          >
            {brokerLabel(brokerId)} 계좌 추가
          </Button>
        </Box>
      </Stack>
    </Section>
  )

  return (
    <>
      <Section title="활성 증권사 프로파일">
        <Stack spacing={2}>
          <Stack direction="row" spacing={1} alignItems="center" flexWrap="wrap">
            {appConfig ? (
              <>
                <Chip
                  size="small"
                  label={`Broker: ${brokerLabel(appConfig.active_broker_id)}`}
                  color={appConfig.active_broker_id === 'kis' ? 'info' : 'secondary'}
                  variant="outlined"
                />
                <Chip
                  size="small"
                  label={activeBrokerModeLabel}
                  color={activeBrokerIsKis
                    ? appConfig.kis_is_paper_trading ? 'warning' : 'primary'
                    : 'secondary'}
                  variant={activeBrokerIsKis ? 'filled' : 'outlined'}
                />
                <Chip
                  size="small"
                  icon={activeBrokerConfigured ? <CheckCircleIcon /> : <ErrorIcon />}
                  label={activeBrokerConfigured ? 'API 키 설정됨' : 'API 키 미설정'}
                  color={activeBrokerConfigured ? 'success' : 'error'}
                  variant="outlined"
                />
                {appConfig.active_profile_name && (
                  <Chip
                    size="small"
                    label={`활성: ${appConfig.active_profile_name}`}
                    variant="outlined"
                  />
                )}
                {appConfig.active_broker_account_id && (
                  <Chip
                    size="small"
                    label={`계좌: ${appConfig.active_broker_account_id}`}
                    variant="outlined"
                  />
                )}
              </>
            ) : (
              <CircularProgress size={20} />
            )}
          </Stack>

          {diag && diag.issues.length > 0 && (
            <Alert severity="warning">
              <Typography variant="body2" fontWeight={600} mb={0.5}>
                설정 문제 감지됨
              </Typography>
              {diag.issues.map((issue, i) => (
                <Typography key={i} variant="body2">• {issue}</Typography>
              ))}
            </Alert>
          )}
          {diag && diag.issues.length === 0 && diag.is_ready && (
            <Alert severity="success">
              API 설정이 완료되었습니다. ({diag.active_mode})
            </Alert>
          )}

          <Divider />

          {tradingStatus?.isRunning && (
            <Alert severity="warning">
              자동매매가 실행 중입니다. 프로필을 전환해도 현재 매매에는 영향이 없으며,
              REST 클라이언트는 자동매매 종료 후 전환됩니다.
              <br />
              실행 범위: {brokerLabel(tradingStatus.tradingBrokerId)}
              {tradingStatus.tradingAccountId ? ` / ${tradingStatus.tradingAccountId}` : ''}
            </Alert>
          )}

          <Box>
            <Button
              size="small"
              variant="outlined"
              onClick={() => recheckConfig()}
              disabled={diagFetching}
              startIcon={diagFetching ? <CircularProgress size={16} /> : undefined}
            >
              설정 재점검
            </Button>
          </Box>

          <Alert severity="info" sx={{ mt: 1 }}>
            <Typography variant="body2">
              계좌 정보는 <code>profiles.json</code>에 로컬 저장됩니다 (git 제외).
              APP SECRET은 저장 후 마스킹되어 표시됩니다.
            </Typography>
          </Alert>
        </Stack>
      </Section>

      {renderBrokerProfiles('kis', kisProfiles)}
      {renderBrokerProfiles('toss', tossProfiles)}

      <AddProfileDialog
        open={addBroker !== null}
        brokerId={addBroker ?? 'kis'}
        onClose={() => setAddBroker(null)}
      />
      <EditProfileDialog profile={editProfile} onClose={() => setEditProfile(null)} />

      <Dialog open={!!deleteTarget} onClose={() => setDeleteTarget(null)} maxWidth="xs" fullWidth>
        <DialogTitle>프로파일 삭제</DialogTitle>
        <DialogContent>
          <Typography>
            <strong>{deleteTarget?.name}</strong> 프로파일을 삭제하시겠습니까?
            {deleteTarget?.is_active && (
              <Alert severity="warning" sx={{ mt: 1 }}>
                현재 활성 프로파일입니다. 삭제 시 다른 프로파일로 자동 전환됩니다.
              </Alert>
            )}
          </Typography>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setDeleteTarget(null)}>취소</Button>
          <Button onClick={handleDelete} color="error" variant="contained">
            삭제
          </Button>
        </DialogActions>
      </Dialog>
    </>
  )
}
