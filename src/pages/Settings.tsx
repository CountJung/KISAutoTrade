import { useState, useEffect, useRef } from 'react'
import InputAdornment from '@mui/material/InputAdornment'
import Typography from '@mui/material/Typography'
import Paper from '@mui/material/Paper'
import Box from '@mui/material/Box'
import Stack from '@mui/material/Stack'
import Divider from '@mui/material/Divider'
import ToggleButton from '@mui/material/ToggleButton'
import ToggleButtonGroup from '@mui/material/ToggleButtonGroup'
import Slider from '@mui/material/Slider'
import FormControlLabel from '@mui/material/FormControlLabel'
import Switch from '@mui/material/Switch'
import Button from '@mui/material/Button'
import Alert from '@mui/material/Alert'
import Chip from '@mui/material/Chip'
import CircularProgress from '@mui/material/CircularProgress'
import Collapse from '@mui/material/Collapse'
import LinearProgress from '@mui/material/LinearProgress'
import Dialog from '@mui/material/Dialog'
import DialogTitle from '@mui/material/DialogTitle'
import DialogContent from '@mui/material/DialogContent'
import DialogActions from '@mui/material/DialogActions'
import TextField from '@mui/material/TextField'
import IconButton from '@mui/material/IconButton'
import Tooltip from '@mui/material/Tooltip'

import LightModeIcon from '@mui/icons-material/LightMode'
import DarkModeIcon from '@mui/icons-material/DarkMode'
import SettingsBrightnessIcon from '@mui/icons-material/SettingsBrightness'
import NotificationsActiveIcon from '@mui/icons-material/NotificationsActive'
import CheckCircleIcon from '@mui/icons-material/CheckCircle'
import ErrorIcon from '@mui/icons-material/Error'
import AddIcon from '@mui/icons-material/Add'
import EditIcon from '@mui/icons-material/Edit'
import DeleteIcon from '@mui/icons-material/Delete'
import RadioButtonCheckedIcon from '@mui/icons-material/RadioButtonChecked'
import RadioButtonUncheckedIcon from '@mui/icons-material/RadioButtonUnchecked'
import SyncIcon from '@mui/icons-material/Sync'

import Select from '@mui/material/Select'
import MenuItem from '@mui/material/MenuItem'
import InputLabel from '@mui/material/InputLabel'
import FormControl from '@mui/material/FormControl'

import StorageIcon from '@mui/icons-material/Storage'

import { useSettingsStore } from '../store/settingsStore'
import {
  useAppConfig,
  useCheckConfig,
  useSendTestDiscord,
  useProfiles,
  useAddProfile,
  useUpdateProfile,
  useDeleteProfile,
  useSetActiveProfile,
  useLogConfig,
  useSetLogConfig,
  useTradeArchiveConfig,
  useSetTradeArchiveConfig,
  useTradeArchiveStats,
  useWebConfig,
  useSaveWebConfig,
  useDetectTradingType,
  useDetectProfileTradingType,
  useStockListStats,
  useSetStockUpdateInterval,
  useTradingStatus,
  useRiskConfig,
  useUpdateRiskConfig,
} from '../api/hooks'
import type { AccountProfileView, AddProfileInput, UpdateProfileInput, UpdateRiskConfigInput } from '../api/types'
import type { ThemeMode } from '../theme'

const fmt = (n: number) => n.toLocaleString('ko-KR')

// ── 공통 섹션 래퍼 ─────────────────────────────────────────────────
function Section({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <Paper sx={{ p: 3 }}>
      <Typography variant="subtitle1" fontWeight={600} mb={2}>
        {title}
      </Typography>
      {children}
    </Paper>
  )
}

// ── 슬라이더 + 텍스트 입력 복합 컴포넌트 ─────────────────────────
interface SliderWithInputProps {
  label: string
  value: number
  min: number
  max: number
  step: number
  unit?: string
  disabled?: boolean
  onChange: (v: number) => void
  onChangeCommitted: (v: number) => void
}

function SliderWithInput({
  label, value, min, max, step, unit, disabled,
  onChange, onChangeCommitted,
}: SliderWithInputProps) {
  const [inputVal, setInputVal] = useState(String(value))
  const prevValue = useRef(value)

  // 외부 value 변경 시 텍스트 필드 동기화 (슬라이더 드래그 완료 후)
  useEffect(() => {
    if (prevValue.current !== value) {
      prevValue.current = value
      setInputVal(String(value))
    }
  }, [value])

  const handleInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const raw = e.target.value.replace(/[^\d.]/g, '')
    setInputVal(raw)
    const n = parseFloat(raw)
    if (!isNaN(n) && n >= min && n <= max) {
      onChange(n)
    }
  }

  const handleInputBlur = () => {
    const n = parseFloat(inputVal)
    if (!isNaN(n)) {
      const clamped = Math.max(min, Math.min(max, Math.round(n / step) * step))
      prevValue.current = clamped
      setInputVal(String(clamped))
      onChange(clamped)
      onChangeCommitted(clamped)
    } else {
      setInputVal(String(value))
    }
  }

  const handleInputKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') (e.target as HTMLInputElement).blur()
  }

  return (
    <Box>
      <Typography variant="body2" gutterBottom>
        {label}: <strong>{value}{unit}</strong>
      </Typography>
      <Stack direction="row" alignItems="center" spacing={2}>
        <Slider
          value={value}
          min={min}
          max={max}
          step={step}
          onChange={(_, v) => onChange(v as number)}
          onChangeCommitted={(_, v) => onChangeCommitted(v as number)}
          valueLabelDisplay="auto"
          disabled={disabled}
          sx={{ flex: 1, maxWidth: 260 }}
        />
        <TextField
          value={inputVal}
          onChange={handleInputChange}
          onBlur={handleInputBlur}
          onKeyDown={handleInputKeyDown}
          size="small"
          disabled={disabled}
          sx={{ width: 100 }}
          slotProps={{
            input: unit ? {
              endAdornment: <InputAdornment position="end">{unit}</InputAdornment>,
            } : {},
          }}
        />
      </Stack>
    </Box>
  )
}

/** Tauri IPC 에러는 `{ code, message }` 객체 → message 를 우선 추출 */
function cmdErrMsg(e: unknown): string {
  if (e && typeof e === 'object' && 'message' in e) {
    return String((e as { message: unknown }).message)
  }
  return String(e)
}

// ── 프로파일 입력 폼 상태 ──────────────────────────────────────────
interface ProfileFormState {
  name: string
  is_paper_trading: boolean
  app_key: string
  app_secret: string
  account_no: string
}

type DetectStatus = 'idle' | 'detecting' | 'detected' | 'failed'

const emptyForm = (): ProfileFormState => ({
  name: '',
  is_paper_trading: false,
  app_key: '',
  app_secret: '',
  account_no: '',
})

// ── 프로파일 추가 다이얼로그 ───────────────────────────────────────
function AddProfileDialog({
  open,
  onClose,
}: {
  open: boolean
  onClose: () => void
}) {
  const [form, setForm] = useState<ProfileFormState>(emptyForm())
  const [error, setError] = useState<string | null>(null)
  const [detectStatus, setDetectStatus] = useState<DetectStatus>('idle')
  const [detectMsg, setDetectMsg] = useState<string>('')
  const { mutate: addProfile, isPending } = useAddProfile()
  const { mutate: detectType, isPending: isDetecting } = useDetectTradingType()

  const canDetect = form.app_key.trim().length > 0 && form.app_secret.trim().length > 0

  const handleDetect = () => {
    if (!canDetect) return
    setDetectStatus('detecting')
    setDetectMsg('')
    detectType(
      { appKey: form.app_key.trim(), appSecret: form.app_secret.trim() },
      {
        onSuccess: (res) => {
          setForm((f) => ({ ...f, is_paper_trading: res.is_paper_trading }))
          setDetectStatus('detected')
          setDetectMsg(res.message)
        },
        onError: (e) => {
          setDetectStatus('failed')
          setDetectMsg(cmdErrMsg(e))
        },
      },
    )
  }

  const handleSubmit = () => {
    if (!form.name.trim()) { setError('프로파일 이름을 입력하세요.'); return }
    if (!form.app_key.trim()) { setError('APP KEY를 입력하세요.'); return }
    if (!form.app_secret.trim()) { setError('APP SECRET을 입력하세요.'); return }
    if (!form.account_no.trim()) { setError('계좌번호를 입력하세요.'); return }

    const input: AddProfileInput = { ...form }
    addProfile(input, {
      onSuccess: () => {
        setForm(emptyForm()); setError(null); setDetectStatus('idle'); setDetectMsg(''); onClose()
      },
      onError: (e) => setError(cmdErrMsg(e)),
    })
  }

  const handleClose = () => {
    setForm(emptyForm()); setError(null); setDetectStatus('idle'); setDetectMsg(''); onClose()
  }

  return (
    <Dialog open={open} onClose={handleClose} maxWidth="sm" fullWidth>
      <DialogTitle>계좌 프로파일 추가</DialogTitle>
      <DialogContent>
        <Stack spacing={2} mt={1}>
          {error && <Alert severity="error">{error}</Alert>}
          <TextField
            label="프로파일 이름"
            value={form.name}
            onChange={(e) => setForm({ ...form, name: e.target.value })}
            placeholder="예: 실전 1호 계좌"
            fullWidth size="small"
          />
          <TextField
            label="APP KEY"
            value={form.app_key}
            onChange={(e) => {
              setForm({ ...form, app_key: e.target.value })
              setDetectStatus('idle')
            }}
            fullWidth size="small" autoComplete="off"
          />
          <TextField
            label="APP SECRET"
            value={form.app_secret}
            onChange={(e) => {
              setForm({ ...form, app_secret: e.target.value })
              setDetectStatus('idle')
            }}
            onBlur={() => { if (canDetect) handleDetect() }}
            type="password"
            fullWidth size="small" autoComplete="new-password"
          />

          {/* ── 자동 감지 영역 ─────────────────────────────── */}
          <Stack direction="row" alignItems="center" spacing={1.5}>
            <Button
              size="small"
              variant="outlined"
              color={detectStatus === 'failed' ? 'error' : 'primary'}
              startIcon={isDetecting
                ? <CircularProgress size={14} color="inherit" />
                : <SyncIcon />
              }
              onClick={handleDetect}
              disabled={!canDetect || isDetecting}
            >
              {isDetecting ? '감지 중...' : '실전/모의 자동 감지'}
            </Button>
            {detectStatus === 'detected' && (
              <Chip
                size="small"
                label={form.is_paper_trading ? '모의투자' : '실전투자'}
                color={form.is_paper_trading ? 'warning' : 'primary'}
              />
            )}
            {detectMsg && (
              <Typography
                variant="caption"
                color={detectStatus === 'failed' ? 'error' : 'text.secondary'}
              >
                {detectMsg}
              </Typography>
            )}
          </Stack>

          {/* ── 수동 override (감지 실패 또는 직접 변경 시) ── */}
          <FormControlLabel
            control={
              <Switch
                checked={form.is_paper_trading}
                onChange={(e) => {
                  setForm({ ...form, is_paper_trading: e.target.checked })
                  setDetectStatus('idle')
                  setDetectMsg('')
                }}
              />
            }
            label={
              <Stack direction="row" spacing={1} alignItems="center">
                <Chip
                  size="small"
                  label={form.is_paper_trading ? '모의투자' : '실전투자'}
                  color={form.is_paper_trading ? 'warning' : 'primary'}
                />
                <Typography variant="caption" color="text.secondary">
                  자동 감지 또는 직접 선택
                </Typography>
              </Stack>
            }
          />

          <TextField
            label="계좌번호 (10자리 입력, 예: 12345678-01)"
            value={form.account_no}
            onChange={(e) => setForm({ ...form, account_no: e.target.value })}
            fullWidth size="small"
          />
          {(()=>{
            const digits = form.account_no.replace('-', '').trim()
            const suffix = digits.length >= 10 ? digits.slice(8) : ''
            if (suffix === '22' || suffix === '29') {
              return (
                <Alert severity="warning" sx={{ py: 0.5 }}>
                  <strong>퇴직연금 계좌({suffix === '22' ? '개인연금·IRP' : '퇴직연금·DC/DB'})은 KIS Open API 주문이 불가합니다.</strong>
                  &nbsp;일반 종합위탁계좌(ACNT_PRDT_CD “01”)만 지원됩니다.
                </Alert>
              )
            }
            return null
          })()}
        </Stack>
      </DialogContent>
      <DialogActions>
        <Button onClick={handleClose} disabled={isPending}>취소</Button>
        <Button onClick={handleSubmit} variant="contained" disabled={isPending}>
          {isPending ? <CircularProgress size={18} /> : '추가'}
        </Button>
      </DialogActions>
    </Dialog>
  )
}

// ── 프로파일 편집 다이얼로그 ───────────────────────────────────────
function EditProfileDialog({
  profile,
  onClose,
}: {
  profile: AccountProfileView | null
  onClose: () => void
}) {
  const [form, setForm] = useState<ProfileFormState>(emptyForm())
  const [error, setError] = useState<string | null>(null)
  const [detectStatus, setDetectStatus] = useState<DetectStatus>('idle')
  const [detectMsg, setDetectMsg] = useState<string>('')
  const { mutate: updateProfile, isPending } = useUpdateProfile()
  const { mutate: detectType, isPending: isDetecting } = useDetectTradingType()

  // profile 변경 시 form 동기화
  const prevId = form.name === '' ? null : profile?.id
  if (profile && profile.id !== prevId) {
    setForm({
      name: profile.name,
      is_paper_trading: profile.is_paper_trading,
      app_key: '',         // 보안상 비워둠 (빈 문자열 = 변경 안 함)
      app_secret: '',
      account_no: profile.account_no,
    })
    setDetectStatus('idle')
    setDetectMsg('')
  }

  // 새 키가 양쪽 모두 입력된 경우에만 감지 가능
  const canDetect = form.app_key.trim().length > 0 && form.app_secret.trim().length > 0

  const handleDetect = () => {
    if (!canDetect) return
    setDetectStatus('detecting')
    setDetectMsg('')
    detectType(
      { appKey: form.app_key.trim(), appSecret: form.app_secret.trim() },
      {
        onSuccess: (res) => {
          setForm((f) => ({ ...f, is_paper_trading: res.is_paper_trading }))
          setDetectStatus('detected')
          setDetectMsg(res.message)
        },
        onError: (e) => {
          setDetectStatus('failed')
          setDetectMsg(cmdErrMsg(e))
        },
      },
    )
  }

  const handleSubmit = () => {
    if (!profile) return
    if (!form.name.trim()) { setError('프로파일 이름을 입력하세요.'); return }

    const input: UpdateProfileInput = {
      id: profile.id,
      name: form.name,
      is_paper_trading: form.is_paper_trading,
      app_key: form.app_key || undefined,
      app_secret: form.app_secret || undefined,
      account_no: form.account_no || undefined,
    }
    updateProfile(input, {
      onSuccess: () => { setError(null); setDetectStatus('idle'); setDetectMsg(''); onClose() },
      onError: (e) => setError(cmdErrMsg(e)),
    })
  }

  const handleClose = () => { setError(null); setDetectStatus('idle'); setDetectMsg(''); onClose() }

  return (
    <Dialog open={!!profile} onClose={handleClose} maxWidth="sm" fullWidth>
      <DialogTitle>프로파일 편집 — {profile?.name}</DialogTitle>
      <DialogContent>
        <Stack spacing={2} mt={1}>
          {error && <Alert severity="error">{error}</Alert>}
          <TextField
            label="프로파일 이름"
            value={form.name}
            onChange={(e) => setForm({ ...form, name: e.target.value })}
            fullWidth size="small"
          />
          <TextField
            label="APP KEY (변경 시에만 입력)"
            value={form.app_key}
            onChange={(e) => {
              setForm({ ...form, app_key: e.target.value })
              setDetectStatus('idle')
            }}
            placeholder={profile?.app_key_masked ?? ''}
            fullWidth size="small" autoComplete="off"
          />
          <TextField
            label="APP SECRET (변경 시에만 입력)"
            value={form.app_secret}
            onChange={(e) => {
              setForm({ ...form, app_secret: e.target.value })
              setDetectStatus('idle')
            }}
            onBlur={() => { if (canDetect) handleDetect() }}
            type="password"
            placeholder="변경 시에만 입력"
            fullWidth size="small" autoComplete="new-password"
          />

          {/* ── 자동 감지 영역 (새 키 입력 시) ──────────────── */}
          <Stack direction="row" alignItems="center" spacing={1.5}>
            <Button
              size="small"
              variant="outlined"
              color={detectStatus === 'failed' ? 'error' : 'primary'}
              startIcon={isDetecting
                ? <CircularProgress size={14} color="inherit" />
                : <SyncIcon />
              }
              onClick={handleDetect}
              disabled={!canDetect || isDetecting}
            >
              {isDetecting ? '감지 중...' : '실전/모의 자동 감지'}
            </Button>
            {detectStatus === 'detected' && (
              <Chip
                size="small"
                label={form.is_paper_trading ? '모의투자' : '실전투자'}
                color={form.is_paper_trading ? 'warning' : 'primary'}
              />
            )}
            {detectMsg && (
              <Typography
                variant="caption"
                color={detectStatus === 'failed' ? 'error' : 'text.secondary'}
              >
                {detectMsg}
              </Typography>
            )}
          </Stack>

          {/* ── 실전/모의 수동 선택 ───────────────────────────── */}
          <FormControlLabel
            control={
              <Switch
                checked={form.is_paper_trading}
                onChange={(e) => {
                  setForm({ ...form, is_paper_trading: e.target.checked })
                  setDetectStatus('idle')
                  setDetectMsg('')
                }}
              />
            }
            label={
              <Stack direction="row" spacing={1} alignItems="center">
                <Chip
                  size="small"
                  label={form.is_paper_trading ? '모의투자' : '실전투자'}
                  color={form.is_paper_trading ? 'warning' : 'primary'}
                />
                <Typography variant="caption" color="text.secondary">
                  자동 감지 또는 직접 선택
                </Typography>
              </Stack>
            }
          />

          <TextField
            label="계좌번호"
            value={form.account_no}
            onChange={(e) => setForm({ ...form, account_no: e.target.value })}
            fullWidth size="small"
          />
          {(()=>{
            const digits = form.account_no.replace('-', '').trim()
            const suffix = digits.length >= 10 ? digits.slice(8) : ''
            if (suffix === '22' || suffix === '29') {
              return (
                <Alert severity="warning" sx={{ py: 0.5 }}>
                  <strong>퇴직연금 계좌({suffix === '22' ? '개인연금·IRP' : '퇴직연금·DC/DB'})은 KIS Open API 주문이 불가합니다.</strong>
                  &nbsp;일반 종합위탁계좌(ACNT_PRDT_CD "01")만 지원됩니다.
                </Alert>
              )
            }
            return null
          })()}
        </Stack>
      </DialogContent>
      <DialogActions>
        <Button onClick={handleClose} disabled={isPending}>취소</Button>
        <Button onClick={handleSubmit} variant="contained" disabled={isPending}>
          {isPending ? <CircularProgress size={18} /> : '저장'}
        </Button>
      </DialogActions>
    </Dialog>
  )
}

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
  const [detectError, setDetectError] = useState<string | null>(null)

  const isActiveTrading = isRunning && tradingProfileId === profile.id

  const handleDetect = () => {
    setDetectError(null)
    detectProfile(profile.id, {
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
              label={profile.is_paper_trading ? '모의투자' : '실전투자'}
              color={profile.is_paper_trading ? 'warning' : 'primary'}
            />
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
            {profile.is_configured && (
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
                    disabled={detecting}
                    sx={{ fontSize: '0.7rem', px: 0.5, minWidth: 0 }}
                  >
                    {detecting ? '감지 중...' : '자동 감지'}
                  </Button>
                </span>
              </Tooltip>
            )}
          </Stack>
          <Typography variant="body2" color="text.secondary" pl={3.5}>
            KEY: <code>{profile.app_key_masked}</code>
            &nbsp;&nbsp;계좌: <code>{profile.account_no || '(미설정)'}</code>
          </Typography>
          {detectError && (
            <Typography variant="caption" color="error" pl={3.5}>
              {detectError}
            </Typography>
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
// ── 리스크 관리 섹션 ──────────────────────────────────────────────
function RiskSection() {
  const { data: risk, isLoading } = useRiskConfig()
  const { mutate: updateRisk, isPending: saving } = useUpdateRiskConfig()

  const [lossLimit, setLossLimit] = useState<number>(0)
  const [posRatio, setPosRatio]   = useState<number>(0)
  const [dirty, setDirty]         = useState(false)

  // risk 데이터가 처음 로드되거나 외부에서 변경됐을 때 로컬 상태 초기화
  useEffect(() => {
    if (risk && !dirty) {
      setLossLimit(risk.dailyLossLimit)
      setPosRatio(Math.round(risk.maxPositionRatio * 100))
    }
  }, [risk, dirty])

  const handleToggleEnabled = (enabled: boolean) => {
    updateRisk({ enabled })
  }

  const handleSave = () => {
    const input: UpdateRiskConfigInput = {
      dailyLossLimit: lossLimit,
      maxPositionRatio: posRatio / 100,
    }
    updateRisk(input, { onSuccess: () => setDirty(false) })
  }

  if (isLoading || !risk) return null

  const netLossPct = Math.min(risk.lossRatio * 100, 100)
  const barColor = netLossPct < 50 ? 'success' : netLossPct < 80 ? 'warning' : 'error'

  return (
    <Section title="리스크 관리">
      <Stack spacing={2}>
        {/* 활성화 토글 */}
        <Stack direction="row" alignItems="flex-start" justifyContent="space-between">
          <Box>
            <Typography variant="body2" fontWeight={600}>
              리스크 관리 사용
            </Typography>
            <Typography variant="caption" color="text.secondary">
              비활성화 시 손실 한도 초과 비상정지가 작동하지 않으며 대시보드에 리스크 패널이 표시되지 않습니다.
            </Typography>
          </Box>
          <Switch
            checked={risk.enabled}
            onChange={(e) => handleToggleEnabled(e.target.checked)}
            disabled={saving}
            color="success"
          />
        </Stack>

        {/* 활성화 상태에서만 표시 */}
        <Collapse in={risk.enabled} unmountOnExit>
          <Stack spacing={2}>
            <Divider />

            {/* 오늘 현황 요약 */}
            <Box>
              <Typography variant="body2" fontWeight={600} gutterBottom>
                오늘 현황
              </Typography>
              <Stack direction="row" spacing={1.5}>
                <Box sx={{ p: 1.5, bgcolor: 'action.hover', borderRadius: 1, flex: 1, textAlign: 'center' }}>
                  <Typography variant="caption" color="text.secondary" display="block">총 손실</Typography>
                  <Typography variant="body2" fontWeight={700} color="error.main">
                    -{fmt(Math.abs(risk.currentLoss))}원
                  </Typography>
                </Box>
                <Box sx={{ p: 1.5, bgcolor: 'action.hover', borderRadius: 1, flex: 1, textAlign: 'center' }}>
                  <Typography variant="caption" color="text.secondary" display="block">당일 수익</Typography>
                  <Typography variant="body2" fontWeight={700} color="success.main">
                    +{fmt(risk.dailyProfit)}원
                  </Typography>
                </Box>
                <Box sx={{ p: 1.5, bgcolor: 'action.hover', borderRadius: 1, flex: 1, textAlign: 'center' }}>
                  <Typography variant="caption" color="text.secondary" display="block">순 손실</Typography>
                  <Typography
                    variant="body2"
                    fontWeight={700}
                    color={risk.netLoss > 0 ? 'warning.main' : 'text.primary'}
                  >
                    {fmt(risk.netLoss)}원
                  </Typography>
                </Box>
              </Stack>

              {/* 순손실 진행바 */}
              <Box sx={{ mt: 1.5 }}>
                <Stack direction="row" justifyContent="space-between" mb={0.5}>
                  <Typography variant="caption" color="text.secondary">순손실 소진율</Typography>
                  <Typography variant="caption" fontWeight={700} color={`${barColor}.main`}>
                    {netLossPct.toFixed(1)}%
                  </Typography>
                </Stack>
                <LinearProgress
                  variant="determinate"
                  value={netLossPct}
                  color={barColor}
                  sx={{ borderRadius: 1, height: 6 }}
                />
              </Box>

              {/* 비상정지 상태 알림 */}
              {risk.isEmergencyStop && (
                <Alert severity="error" sx={{ mt: 1 }}>
                  비상정지 활성 — 대시보드의 리스크 관리 패널에서 해제할 수 있습니다.
                </Alert>
              )}
            </Box>

            <Divider />

            {/* 한도 설정 슬라이더 */}
            <Box>
              <Typography variant="body2" fontWeight={600} gutterBottom>
                한도 설정
              </Typography>
              <Stack spacing={2}>
                <SliderWithInput
                  label="일일 순손실 한도"
                  value={lossLimit}
                  min={0}
                  max={5_000_000}
                  step={100_000}
                  unit="원"
                  disabled={saving}
                  onChange={(v) => { setLossLimit(v); setDirty(true) }}
                  onChangeCommitted={(v) => { setLossLimit(v); setDirty(true) }}
                />
                <SliderWithInput
                  label="종목당 최대 비중"
                  value={posRatio}
                  min={1}
                  max={100}
                  step={1}
                  unit="%"
                  disabled={saving}
                  onChange={(v) => { setPosRatio(v); setDirty(true) }}
                  onChangeCommitted={(v) => { setPosRatio(v); setDirty(true) }}
                />
                <Box sx={{ display: 'flex', justifyContent: 'flex-end' }}>
                  <Button
                    variant="contained"
                    size="small"
                    onClick={handleSave}
                    disabled={!dirty || saving}
                    startIcon={saving ? <CircularProgress size={14} color="inherit" /> : undefined}
                  >
                    저장
                  </Button>
                </Box>
              </Stack>
            </Box>
          </Stack>
        </Collapse>
      </Stack>
    </Section>
  )
}

export default function Settings() {
  const {
    theme, discordEnabled,
    setTheme, setDiscordEnabled,
  } = useSettingsStore()

  const { data: appConfig } = useAppConfig()
  const { data: diag, refetch: recheckConfig, isFetching: diagFetching } = useCheckConfig()
  const { mutate: sendTestDiscord, isPending: discordPending } = useSendTestDiscord()
  const [discordResult, setDiscordResult] = useState<{ ok: boolean; msg: string } | null>(null)
  const { data: tradingStatus } = useTradingStatus()

  // 로그 설정 (백엔드 IPC)
  const { data: logCfg } = useLogConfig()
  const { mutate: saveLogConfig, isPending: logSaving } = useSetLogConfig()
  // 슬라이더 드래그 중 로컬 값 (드래그 완료 시 IPC 저장)
  const [localRetentionDays, setLocalRetentionDays] = useState<number>(5)
  const [localMaxSizeMb, setLocalMaxSizeMb] = useState<number>(100)

  // 백엔드 값이 로드되면 로컬 상태 동기화
  const prevLogCfgRef = useState<typeof logCfg>(undefined)
  if (logCfg && logCfg !== prevLogCfgRef[0]) {
    prevLogCfgRef[1](logCfg)
    setLocalRetentionDays(logCfg.retention_days)
    setLocalMaxSizeMb(logCfg.max_size_mb)
  }
  // 체결 기록 보관 설정 (IPC)
  const { data: archiveCfg } = useTradeArchiveConfig()
  const { mutate: saveArchiveConfig, isPending: archiveSaving } = useSetTradeArchiveConfig()
  const { data: archiveStats } = useTradeArchiveStats()
  const [localArchiveRetentionDays, setLocalArchiveRetentionDays] = useState<number>(90)
  const [localArchiveMaxSizeMb, setLocalArchiveMaxSizeMb] = useState<number>(500)

  // archiveCfg 로드 시 로컈 동기화
  const prevArchiveCfgRef = useState<typeof archiveCfg>(undefined)
  if (archiveCfg && archiveCfg !== prevArchiveCfgRef[0]) {
    prevArchiveCfgRef[1](archiveCfg)
    setLocalArchiveRetentionDays(archiveCfg.retention_days)
    setLocalArchiveMaxSizeMb(archiveCfg.max_size_mb)
  }

  // 프로파일 관리 상태
  const { data: profiles = [], isLoading: profilesLoading } = useProfiles()
  const { mutate: setActive } = useSetActiveProfile()
  const { mutate: deleteProfile } = useDeleteProfile()
  const [addOpen, setAddOpen] = useState(false)
  const [editProfile, setEditProfile] = useState<AccountProfileView | null>(null)
  const [deleteTarget, setDeleteTarget] = useState<AccountProfileView | null>(null)

  // 웹 접속 설정
  const { data: webConfig } = useWebConfig()
  const { mutate: saveWebConfig, isPending: webSaving } = useSaveWebConfig()
  const [webPortInput, setWebPortInput] = useState<string>('')
  const [distPathInput, setDistPathInput] = useState<string>('')
  const [webSaveResult, setWebSaveResult] = useState<{ ok: boolean; msg: string } | null>(null)
  // webConfig가 로드되면 입력칸 동기화
  const prevWebConfigRef = useState<typeof webConfig>(undefined)
  if (webConfig && webConfig !== prevWebConfigRef[0]) {
    prevWebConfigRef[1](webConfig)
    setWebPortInput(String(webConfig.runningPort))
    // distPath는 사용자가 직접 입력한 경우에만 동기화
    if (!distPathInput && webConfig.distPath) {
      setDistPathInput(webConfig.distPath)
    }
  }

  // 종목 목록 관리
  const { data: stockStats, isFetching: statsFetching } = useStockListStats()
  const { mutate: doSetInterval } = useSetStockUpdateInterval()

  const handleTestDiscord = () => {
    setDiscordResult(null)
    sendTestDiscord(undefined, {
      onSuccess: (msg) => setDiscordResult({ ok: true, msg }),
      onError: (err) => setDiscordResult({ ok: false, msg: String(err) }),
    })
  }

  const handleDelete = () => {
    if (!deleteTarget) return
    deleteProfile(deleteTarget.id, {
      onSuccess: () => setDeleteTarget(null),
    })
  }

  return (
    <Box>
      <Typography variant="h5" fontWeight={700} mb={3}>
        Settings
      </Typography>

      <Stack spacing={2}>
        {/* ── 테마 ─────────────────────────────────────────────── */}
        <Section title="테마">
          <ToggleButtonGroup
            value={theme}
            exclusive
            onChange={(_, value: ThemeMode | null) => value && setTheme(value)}
            size="small"
          >
            <ToggleButton value="light">
              <LightModeIcon fontSize="small" sx={{ mr: 0.5 }} />
              라이트
            </ToggleButton>
            <ToggleButton value="dark">
              <DarkModeIcon fontSize="small" sx={{ mr: 0.5 }} />
              다크
            </ToggleButton>
            <ToggleButton value="system">
              <SettingsBrightnessIcon fontSize="small" sx={{ mr: 0.5 }} />
              시스템
            </ToggleButton>
          </ToggleButtonGroup>
        </Section>

        {/* ── 로그 설정 ─────────────────────────────────────────── */}
        <Section title="로그 설정">
          <Stack spacing={3}>
            <SliderWithInput
              label="보관 기간"
              value={localRetentionDays}
              min={1} max={30} step={1}
              unit="일"
              disabled={logSaving}
              onChange={(v) => setLocalRetentionDays(v)}
              onChangeCommitted={(v) => saveLogConfig({
                retention_days: v,
                max_size_mb: localMaxSizeMb,
                api_debug: logCfg?.api_debug ?? false,
              })}
            />
            <SliderWithInput
              label="최대 용량"
              value={localMaxSizeMb}
              min={10} max={500} step={10}
              unit="MB"
              disabled={logSaving}
              onChange={(v) => setLocalMaxSizeMb(v)}
              onChangeCommitted={(v) => saveLogConfig({
                retention_days: localRetentionDays,
                max_size_mb: v,
                api_debug: logCfg?.api_debug ?? false,
              })}
            />
            <Box>
              <FormControlLabel
                control={
                  <Switch
                    checked={logCfg?.api_debug ?? false}
                    disabled={logSaving}
                    onChange={(e) => saveLogConfig({
                      retention_days: localRetentionDays,
                      max_size_mb: localMaxSizeMb,
                      api_debug: e.target.checked,
                    })}
                  />
                }
                label="KIS API 진단 로그"
              />
              <Typography variant="caption" color="text.secondary" sx={{ display: 'block', mt: 0.5 }}>
                ON 시 KIS API 요청 파라미터와 응답 JSON 전체를 로그에 기록합니다. 체결 내역 0건 등 문제 진단용입니다. 진단 후 반드시 OFF 하세요.
              </Typography>
            </Box>
            <Typography variant="caption" color="text.secondary">
              로그 파일 위치: <code>./logs/</code> (앱 실행 폴더 기준)
              <br />
              설정은 즉시 적용되며 초과 파일은 자동 정리됩니다.
            </Typography>
          </Stack>
        </Section>

        {/* ── 체결 기록 보관 설정 ─────────────────────────────────── */}
        <Section title="체결 기록 보관">
          <Stack spacing={3}>
            <SliderWithInput
              label="보관 기간"
              value={localArchiveRetentionDays}
              min={1} max={365} step={1}
              unit="일"
              disabled={archiveSaving}
              onChange={(v) => setLocalArchiveRetentionDays(v)}
              onChangeCommitted={(v) => saveArchiveConfig({
                retention_days: v,
                max_size_mb: localArchiveMaxSizeMb,
              })}
            />
            <SliderWithInput
              label="최대 용량"
              value={localArchiveMaxSizeMb}
              min={50} max={2000} step={50}
              unit="MB"
              disabled={archiveSaving}
              onChange={(v) => setLocalArchiveMaxSizeMb(v)}
              onChangeCommitted={(v) => saveArchiveConfig({
                retention_days: localArchiveRetentionDays,
                max_size_mb: v,
              })}
            />
            {archiveStats && (
              <Box>
                <Typography variant="caption" color="text.secondary">
                  현재 저장: {(archiveStats.size_bytes / 1024 / 1024).toFixed(2)}MB
                  {archiveStats.oldest_date && ` · 오래된 날짜 ${archiveStats.oldest_date}`}
                  {archiveStats.newest_date && ` · 최신 날짜 ${archiveStats.newest_date}`}
                </Typography>
              </Box>
            )}
            <Typography variant="caption" color="text.secondary">
              체결 기록 저장 위치: <code>data/trades/YYYY/MM/DD/</code>
              <br />
              설정 저장 시 보관 기간 초과 데이터는 자동 정리됩니다.
            </Typography>
          </Stack>
        </Section>

        {/* ── 계좌 프로파일 관리 ─────────────────────────────────── */}
        <Section title="한국투자증권 계좌 프로파일">
          <Stack spacing={2}>
            {/* 현재 활성 상태 요약 */}
            <Stack direction="row" spacing={1} alignItems="center" flexWrap="wrap">
              {appConfig ? (
                <>
                  <Chip
                    size="small"
                    label={appConfig.kis_is_paper_trading ? '모의투자 모드' : '실전투자 모드'}
                    color={appConfig.kis_is_paper_trading ? 'warning' : 'primary'}
                  />
                  <Chip
                    size="small"
                    icon={appConfig.kis_configured ? <CheckCircleIcon /> : <ErrorIcon />}
                    label={appConfig.kis_configured ? 'API 키 설정됨' : 'API 키 미설정'}
                    color={appConfig.kis_configured ? 'success' : 'error'}
                    variant="outlined"
                  />
                  {appConfig.active_profile_name && (
                    <Chip
                      size="small"
                      label={`활성: ${appConfig.active_profile_name}`}
                      variant="outlined"
                    />
                  )}
                </>
              ) : (
                <CircularProgress size={20} />
              )}
            </Stack>

            {/* 진단 결과 */}
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

            {/* 자동매매 중 프로필 전환 경고 */}
            {tradingStatus?.isRunning && (
              <Alert severity="warning">
                자동매매가 실행 중입니다. 프로필을 전환해도 현재 매매에는 영향이 없으며,
                REST 클라이언트는 자동매매 종료 후 전환됩니다.
              </Alert>
            )}

            {/* 프로파일 목록 */}
            {profilesLoading ? (
              <CircularProgress size={24} />
            ) : profiles.length === 0 ? (
              <Alert severity="info">
                등록된 계좌 프로파일이 없습니다. 아래 버튼으로 추가하세요.
              </Alert>
            ) : (
              <Stack spacing={1.5}>
                {profiles.map((p) => (
                  <ProfileCard
                    key={p.id}
                    profile={p}
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
                onClick={() => setAddOpen(true)}
                size="small"
              >
                계좌 추가
              </Button>
              <Button
                size="small"
                variant="outlined"
                onClick={() => recheckConfig()}
                disabled={diagFetching}
                startIcon={diagFetching ? <CircularProgress size={16} /> : undefined}
                sx={{ ml: 1 }}
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

        {/* ── 웹 접속 설정 ──────────────────────────────────────── */}
        <Section title="웹 접속 설정">
          <Stack spacing={2}>
            <Typography variant="body2" color="text.secondary">
              같은 네트워크의 다른 기기(모바일, 태블릿 등)에서 브라우저로 접속할 수 있습니다.
              <br />
              접속 URL: <code>{webConfig?.accessUrl ?? `http://localhost:7474`}</code>
            </Typography>

            {/* dist/ 빌드 파일 상태 */}
            {webConfig && (
              webConfig.distFound === false ? (
                <Alert severity="warning">
                  <Typography variant="body2" fontWeight={600} mb={0.5}>
                    프론트엔드 빌드 파일 없음
                  </Typography>
                  <Typography variant="body2">
                    탐색 경로: <code>{webConfig.distPath}</code>
                    <br />
                    해결방법: 프로젝트 루트에서 <code>npm run build</code> 실행 후 앱 재시작,
                    또는 아래 dist/ 경로 직접 설정
                  </Typography>
                </Alert>
              ) : (
                <Alert severity="success" sx={{ py: 0.5 }}>
                  React 앱 서비스 중 — <code>{webConfig.distPath}</code>
                </Alert>
              )
            )}

            {/* 포트 설정 */}
            <Box>
              <Stack direction="row" spacing={1} alignItems="center">
                <TextField
                  label="웹 서버 포트"
                  value={webPortInput}
                  onChange={(e) => { setWebPortInput(e.target.value); setWebSaveResult(null) }}
                  size="small"
                  type="number"
                  inputProps={{ min: 1024, max: 65535 }}
                  sx={{ width: 140 }}
                />
                <Button
                  variant="outlined"
                  onClick={() => {
                    const port = parseInt(webPortInput, 10)
                    if (isNaN(port)) return
                    setWebSaveResult(null)
                    saveWebConfig(
                      { newPort: port, distPath: distPathInput.trim() || undefined },
                      {
                        onSuccess: (msg) => setWebSaveResult({ ok: true, msg }),
                        onError: (err) => setWebSaveResult({ ok: false, msg: String(err) }),
                      },
                    )
                  }}
                  disabled={webSaving || !webPortInput}
                  startIcon={webSaving ? <CircularProgress size={16} /> : null}
                >
                  저장
                </Button>
              </Stack>
              <Typography variant="caption" color="text.secondary" sx={{ mt: 0.5, display: 'block' }}>
                기본값: 7474 (재시작 후 적용)
              </Typography>
            </Box>

            {/* dist/ 경로 직접 설정 */}
            <Box>
              <TextField
                label="dist/ 경로 (선택)"
                placeholder="예: /Users/me/KISAutoTrade/dist"
                value={distPathInput}
                onChange={(e) => setDistPathInput(e.target.value)}
                size="small"
                fullWidth
                helperText="비워두면 자동 탐색. 빌드 파일을 찾지 못할 때 절대 경로를 입력하세요. (.env DIST_PATH에 저장됨)"
              />
            </Box>

            {webSaveResult && (
              <Alert severity={webSaveResult.ok ? 'success' : 'error'}>
                {webSaveResult.msg}
                {webSaveResult.ok && ' — 앱을 재시작하면 새 설정이 적용됩니다.'}
              </Alert>
            )}
          </Stack>
        </Section>

        {/* ── 종목 목록 관리 ─────────────────────────────────────── */}
        <Section title="종목 목록 관리">
          <Stack spacing={2}>
            <Typography variant="body2" color="text.secondary">
              종목명 검색은 <strong>로컬 캐시 → NAVER Finance → KIS API</strong> 순서로 동작합니다.
              잔고 조회·현재가 조회·주문 체결 시 종목명이 자동으로 캐시에 누적됩니다.
            </Typography>

            {/* 통계 */}
            <Box sx={{ display: 'flex', gap: 2, flexWrap: 'wrap', alignItems: 'center' }}>
              <Chip
                icon={<StorageIcon />}
                label={statsFetching ? '로딩 중...' : `저장된 종목: ${stockStats?.count ?? 0}개`}
                color={stockStats && stockStats.count > 0 ? 'success' : 'default'}
                variant="outlined"
                size="small"
              />
              {stockStats?.lastUpdatedAt && (
                <Typography variant="caption" color="text.secondary">
                  마지막 갱신: {new Date(stockStats.lastUpdatedAt).toLocaleString('ko-KR')}
                </Typography>
              )}
            </Box>

            <Divider />

            {/* 자동 갱신 간격 */}
            <Box>
              <FormControl size="small" sx={{ minWidth: 200 }}>
                <InputLabel>자동 갱신 간격</InputLabel>
                <Select
                  value={stockStats?.updateIntervalHours ?? 24}
                  label="자동 갱신 간격"
                  onChange={(e) => doSetInterval(Number(e.target.value))}
                >
                  <MenuItem value={0}>수동 전용 (자동 갱신 없음)</MenuItem>
                  <MenuItem value={6}>6시간마다</MenuItem>
                  <MenuItem value={12}>12시간마다</MenuItem>
                  <MenuItem value={24}>매일 (24시간)</MenuItem>
                  <MenuItem value={168}>매주 (7일)</MenuItem>
                </Select>
              </FormControl>
              <Typography variant="caption" color="text.secondary" sx={{ mt: 0.5, display: 'block' }}>
                앱 시작 시 설정된 간격이 지났으면 NAVER Finance 실시간 검색으로 종목 정보를 업데이트합니다.
              </Typography>
            </Box>



            <Alert severity="info">
              <Typography variant="caption">
                파일 경로: <code>{stockStats?.filePath ?? '로딩 중...'}</code>
                <br />
                종목명 검색이 안 될 때: 잔고 조회를 먼저 실행하면 보유 종목이 자동 등록됩니다.
                6자리 코드를 검색창에 입력하면 KIS API로 즉시 확인할 수 있습니다.
              </Typography>
            </Alert>
          </Stack>
        </Section>

        {/* ── 리스크 관리 ───────────────────────────────────────── */}
        <RiskSection />

        {/* ── Discord 알림 ───────────────────────────────────────── */}
        <Section title="Discord 알림">
          <Stack spacing={2}>
            <FormControlLabel
              control={
                <Switch
                  checked={discordEnabled}
                  onChange={(e) => setDiscordEnabled(e.target.checked)}
                />
              }
              label="Discord 알림 활성화"
            />
            <Divider />
            <Typography variant="body2" color="text.secondary">
              Backend Discord 상태:{' '}
              <strong>{appConfig?.discord_enabled ? '✅ 연결됨' : '❌ 미설정'}</strong>
              <br />
              Bot Token 및 채널 ID는 <code>secure_config.json</code>에 저장됩니다.
            </Typography>
            <Box>
              <Button
                variant="outlined"
                startIcon={
                  discordPending
                    ? <CircularProgress size={16} />
                    : <NotificationsActiveIcon />
                }
                onClick={handleTestDiscord}
                disabled={discordPending || !appConfig?.discord_enabled}
              >
                테스트 알림 전송
              </Button>
            </Box>
            {discordResult && (
              <Alert severity={discordResult.ok ? 'success' : 'error'}>
                {discordResult.msg}
              </Alert>
            )}
          </Stack>
        </Section>
      </Stack>

      {/* ── 다이얼로그 ─────────────────────────────────────────── */}
      <AddProfileDialog open={addOpen} onClose={() => setAddOpen(false)} />
      <EditProfileDialog profile={editProfile} onClose={() => setEditProfile(null)} />

      {/* 삭제 확인 다이얼로그 */}
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
    </Box>
  )
}
