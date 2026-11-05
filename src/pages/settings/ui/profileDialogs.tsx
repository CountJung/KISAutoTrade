import { useEffect, useState } from 'react'

import Alert from '@mui/material/Alert'
import Button from '@mui/material/Button'
import Chip from '@mui/material/Chip'
import CircularProgress from '@mui/material/CircularProgress'
import Dialog from '@mui/material/Dialog'
import DialogActions from '@mui/material/DialogActions'
import DialogContent from '@mui/material/DialogContent'
import DialogTitle from '@mui/material/DialogTitle'
import FormControl from '@mui/material/FormControl'
import FormControlLabel from '@mui/material/FormControlLabel'
import InputLabel from '@mui/material/InputLabel'
import MenuItem from '@mui/material/MenuItem'
import Select from '@mui/material/Select'
import Stack from '@mui/material/Stack'
import Switch from '@mui/material/Switch'
import TextField from '@mui/material/TextField'
import Typography from '@mui/material/Typography'
import SyncIcon from '@mui/icons-material/Sync'

import {
  useAddProfile,
  useDetectTradingType,
  useListTossAccounts,
  useListTossProfileAccounts,
  useUpdateProfile,
} from '../../../api/hooks'
import type {
  AccountProfileView,
  AddProfileInput,
  BrokerId,
  TossAccountOptionView,
  UpdateProfileInput,
} from '../../../api/types'

import { brokerLabel, cmdErrMsg } from './profileUtils'
import { useProfileDialogSession } from './useProfileDialogSession'

function brokerProfileLabels(brokerId: BrokerId) {
  if (brokerId === 'toss') {
    return {
      key: 'Client ID',
      secret: 'Client Secret',
      account: 'accountSeq',
      accountPlaceholder: '예: 1',
      accountHelp: '토스증권 accounts 응답의 accountSeq 숫자 값을 입력합니다.',
    }
  }
  return {
    key: 'APP KEY',
    secret: 'APP SECRET',
    account: '계좌번호 (10자리 입력, 예: 12345678-01)',
    accountPlaceholder: '',
    accountHelp: '',
  }
}

// ── 프로파일 입력 폼 상태 ──────────────────────────────────────────
interface ProfileFormState {
  broker_id: BrokerId
  name: string
  is_paper_trading: boolean
  live_trading_consent: boolean
  app_key: string
  app_secret: string
  account_no: string
}

type DetectStatus = 'idle' | 'detecting' | 'detected' | 'failed'

const emptyForm = (brokerId: BrokerId = 'kis'): ProfileFormState => ({
  broker_id: brokerId,
  name: '',
  is_paper_trading: false,
  live_trading_consent: false,
  app_key: '',
  app_secret: '',
  account_no: '',
})

function TossAccountSeqField({
  idPrefix,
  value,
  onChange,
  accounts,
  loading,
  onLookup,
  lookupDisabled,
  message,
  helperText,
}: {
  idPrefix: string
  value: string
  onChange: (value: string) => void
  accounts: TossAccountOptionView[]
  loading: boolean
  onLookup: () => void
  lookupDisabled: boolean
  message: string
  helperText: string
}) {
  const selectedFromLookup = accounts.some((account) => account.account_seq === value)

  return (
    <Stack spacing={0.75}>
      <Stack direction={{ xs: 'column', sm: 'row' }} spacing={1}>
        {accounts.length > 0 ? (
          <FormControl fullWidth size="small">
            <InputLabel id={`${idPrefix}-toss-account-label`}>accountSeq</InputLabel>
            <Select
              labelId={`${idPrefix}-toss-account-label`}
              label="accountSeq"
              value={value}
              onChange={(e) => onChange(String(e.target.value))}
            >
              {value && !selectedFromLookup && (
                <MenuItem value={value}>현재 저장값: {value}</MenuItem>
              )}
              {accounts.map((account) => (
                <MenuItem key={account.account_seq} value={account.account_seq}>
                  {account.label}
                </MenuItem>
              ))}
            </Select>
          </FormControl>
        ) : (
          <TextField
            label="accountSeq"
            placeholder="계좌 조회 후 선택 또는 숫자 입력"
            value={value}
            onChange={(e) => onChange(e.target.value)}
            fullWidth
            size="small"
          />
        )}
        <Button
          size="small"
          variant="outlined"
          startIcon={loading ? <CircularProgress size={14} color="inherit" /> : <SyncIcon />}
          onClick={onLookup}
          disabled={lookupDisabled || loading}
          sx={{ minWidth: 112 }}
        >
          {loading ? '조회 중' : '계좌 조회'}
        </Button>
      </Stack>
      <Typography variant="caption" color="text.secondary">
        {message || helperText}
      </Typography>
    </Stack>
  )
}

// ── 프로파일 추가 다이얼로그 ───────────────────────────────────────
export function AddProfileDialog({
  open,
  brokerId,
  onClose,
}: {
  open: boolean
  brokerId: BrokerId
  onClose: () => void
}) {
  const [form, setForm] = useState<ProfileFormState>(() => emptyForm(brokerId))
  const [error, setError] = useState<string | null>(null)
  const [detectStatus, setDetectStatus] = useState<DetectStatus>('idle')
  const [detectMsg, setDetectMsg] = useState<string>('')
  const [tossAccounts, setTossAccounts] = useState<TossAccountOptionView[]>([])
  const [tossAccountMsg, setTossAccountMsg] = useState<string>('')
  const { mutateAsync: addProfile } = useAddProfile()
  const { mutateAsync: detectType } = useDetectTradingType()
  const { mutateAsync: listTossAccounts } = useListTossAccounts()
  const session = useProfileDialogSession(open, brokerId)
  const isPending = session.main.pending()
  const isDetecting = session.detect.pending()
  const isListingTossAccounts = session.lookup.pending()

  useEffect(() => {
    if (!open) return
    setForm(emptyForm(brokerId))
    setError(null)
    setDetectStatus('idle')
    setDetectMsg('')
    setTossAccounts([])
    setTossAccountMsg('')
  }, [open, brokerId])

  const labels = brokerProfileLabels(form.broker_id)
  const isKisProfile = form.broker_id === 'kis'
  const isTossProfile = form.broker_id === 'toss'
  const canDetect = isKisProfile && form.app_key.trim().length > 0 && form.app_secret.trim().length > 0
  const canLookupTossAccounts = isTossProfile
    && form.app_key.trim().length > 0
    && form.app_secret.trim().length > 0

  const handleDetect = async () => {
    if (!canDetect) return
    const ticket = session.detect.begin()
    if (!ticket) return
    setDetectStatus('detecting')
    setDetectMsg('')
    try {
      const res = await detectType({ appKey: form.app_key.trim(), appSecret: form.app_secret.trim() })
      if (!session.detect.current(ticket)) return
      setForm((f) => ({ ...f, is_paper_trading: res.is_paper_trading }))
      setDetectStatus('detected')
      setDetectMsg(res.message)
    } catch (e) {
      if (!session.detect.current(ticket)) return
      setDetectStatus('failed')
      setDetectMsg(cmdErrMsg(e))
    } finally {
      session.detect.finish(ticket)
    }
  }

  const resetTossAccounts = () => {
    setTossAccounts([])
    setTossAccountMsg('')
  }

  const handleLookupTossAccounts = async () => {
    if (!canLookupTossAccounts) {
      setError('토스증권 Client ID와 Client Secret을 먼저 입력하세요.')
      return
    }
    const ticket = session.lookup.begin()
    if (!ticket) return
    setError(null)
    try {
      const accounts = await listTossAccounts({ client_id: form.app_key.trim(), client_secret: form.app_secret.trim() })
      if (!session.lookup.current(ticket)) return
      setTossAccounts(accounts)
      setTossAccountMsg(
        accounts.length > 0
          ? `${accounts.length}개 계좌를 조회했습니다. 저장할 accountSeq를 선택하세요.`
          : '조회된 토스증권 계좌가 없습니다.',
      )
      if (accounts.length === 1) {
        setForm((f) => ({ ...f, account_no: accounts[0].account_seq }))
      }
    } catch (e) {
      if (!session.lookup.current(ticket)) return
      resetTossAccounts()
      setError(cmdErrMsg(e))
    } finally {
      session.lookup.finish(ticket)
    }
  }

  const handleSubmit = async () => {
    if (session.main.pending()) return
    if (!form.name.trim()) { setError('프로파일 이름을 입력하세요.'); return }
    if (!form.app_key.trim()) { setError(`${labels.key}를 입력하세요.`); return }
    if (!form.app_secret.trim()) { setError(`${labels.secret}을 입력하세요.`); return }
    if (!form.account_no.trim()) { setError(`${labels.account}를 입력하세요.`); return }
    if (form.broker_id === 'toss' && !/^\d+$/.test(form.account_no.trim())) {
      setError('토스증권 accountSeq는 숫자여야 합니다.')
      return
    }

    const ticket = session.main.begin()
    if (!ticket) return
    const input: AddProfileInput = { ...form }
    try {
      await addProfile(input)
      if (!session.main.finish(ticket)) return
      session.close()
      setForm(emptyForm(brokerId)); setError(null); setDetectStatus('idle'); setDetectMsg(''); resetTossAccounts(); onClose()
    } catch (e) {
      if (session.main.current(ticket)) setError(cmdErrMsg(e))
    } finally {
      session.main.finish(ticket)
    }
  }

  const handleClose = () => {
    if (session.main.pending()) return
    session.close()
    setForm(emptyForm(brokerId)); setError(null); setDetectStatus('idle'); setDetectMsg(''); resetTossAccounts(); onClose()
  }

  return (
    <Dialog open={open} onClose={handleClose} maxWidth="sm" fullWidth>
      <DialogTitle>{brokerLabel(brokerId)} 계좌 프로파일 추가</DialogTitle>
      <DialogContent>
        <Stack spacing={2} sx={{ mt: 1 }}>
          {error && <Alert severity="error">{error}</Alert>}
          <Chip
            size="small"
            label={brokerLabel(form.broker_id)}
            color={form.broker_id === 'kis' ? 'info' : 'secondary'}
            variant="outlined"
            sx={{ alignSelf: 'flex-start' }}
          />
          {form.broker_id === 'toss' && (
            <>
              <Alert severity="info" sx={{ py: 0.75 }}>
                토스증권 프로파일은 연결 진단과 주문 전 검증을 거친 뒤 실거래 동의가 저장된 경우 실제 주문과 자동매매를 실행할 수 있습니다.
              </Alert>
              <FormControlLabel
                control={
                  <Switch
                    checked={form.live_trading_consent}
                    onChange={(e) => setForm({ ...form, live_trading_consent: e.target.checked })}
                  />
                }
                label={
                  <Stack direction="row" spacing={1} sx={{ alignItems: "center" }}>
                    <Chip
                      size="small"
                      label={form.live_trading_consent ? '실거래 동의 저장' : '실거래 동의 없음'}
                      color={form.live_trading_consent ? 'warning' : 'default'}
                      variant={form.live_trading_consent ? 'filled' : 'outlined'}
                    />
                    <Typography variant="caption" color="text.secondary">
                      향후 토스 주문 연결 전 명시 승인 상태로만 사용
                    </Typography>
                  </Stack>
                }
              />
            </>
          )}
          <TextField
            label="프로파일 이름"
            value={form.name}
            onChange={(e) => setForm({ ...form, name: e.target.value })}
            placeholder="예: 실전 1호 계좌"
            fullWidth size="small"
          />
          <TextField
            label={labels.key}
            value={form.app_key}
            onChange={(e) => {
              session.invalidateAux()
              setForm({ ...form, app_key: e.target.value })
              setDetectStatus('idle')
              if (isTossProfile) resetTossAccounts()
            }}
            fullWidth size="small" autoComplete="off"
          />
          <TextField
            label={labels.secret}
            value={form.app_secret}
            onChange={(e) => {
              session.invalidateAux()
              setForm({ ...form, app_secret: e.target.value })
              setDetectStatus('idle')
              if (isTossProfile) resetTossAccounts()
            }}
            onBlur={() => { if (canDetect) handleDetect() }}
            type="password"
            fullWidth size="small" autoComplete="new-password"
          />

          {/* ── 자동 감지 영역 ─────────────────────────────── */}
          {isKisProfile && (
            <Stack direction="row" sx={{ alignItems: "center" }} spacing={1.5}>
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
          )}

          {/* ── 수동 override (감지 실패 또는 직접 변경 시) ── */}
          {isKisProfile && (
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
                <Stack direction="row" spacing={1} sx={{ alignItems: "center" }}>
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
          )}

          {isTossProfile ? (
            <TossAccountSeqField
              idPrefix="add-profile"
              value={form.account_no}
              onChange={(accountNo) => setForm({ ...form, account_no: accountNo })}
              accounts={tossAccounts}
              loading={isListingTossAccounts}
              onLookup={handleLookupTossAccounts}
              lookupDisabled={!canLookupTossAccounts}
              message={tossAccountMsg}
              helperText={labels.accountHelp}
            />
          ) : (
            <>
              <TextField
                label={labels.account}
                placeholder={labels.accountPlaceholder}
                value={form.account_no}
                onChange={(e) => setForm({ ...form, account_no: e.target.value })}
                fullWidth size="small"
              />
              {labels.accountHelp && (
                <Typography variant="caption" color="text.secondary">
                  {labels.accountHelp}
                </Typography>
              )}
            </>
          )}
          {(()=>{
            if (!isKisProfile) return null
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
        <Button onClick={handleSubmit} variant="contained" aria-label="추가"
          aria-disabled={isPending} aria-busy={isPending} aria-description={isPending ? '추가 중' : undefined}
          startIcon={isPending ? <CircularProgress size={18} color="inherit" aria-label="추가 중" /> : undefined}>
          추가
        </Button>
      </DialogActions>
    </Dialog>
  )
}

// ── 프로파일 편집 다이얼로그 ───────────────────────────────────────
export function EditProfileDialog({
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
  const [tossAccounts, setTossAccounts] = useState<TossAccountOptionView[]>([])
  const [tossAccountMsg, setTossAccountMsg] = useState<string>('')
  const { mutateAsync: updateProfile } = useUpdateProfile()
  const { mutateAsync: detectType } = useDetectTradingType()
  const { mutateAsync: listTossAccounts } = useListTossAccounts()
  const { mutateAsync: listTossProfileAccounts } = useListTossProfileAccounts()
  const session = useProfileDialogSession(!!profile, profile)
  const isPending = session.main.pending()
  const isDetecting = session.detect.pending()

  // profile 변경 시 form 동기화
  useEffect(() => {
    setError(null)
    if (!profile) {
      setForm(emptyForm())
      setTossAccounts([])
      setTossAccountMsg('')
      return
    }
    setForm({
      broker_id: profile.broker_id,
      name: profile.name,
      is_paper_trading: profile.is_paper_trading,
      live_trading_consent: profile.live_trading_consent,
      app_key: '',         // 보안상 비워둠 (빈 문자열 = 변경 안 함)
      app_secret: '',
      account_no: profile.account_no,
    })
    setDetectStatus('idle')
    setDetectMsg('')
    setTossAccounts([])
    setTossAccountMsg('')
  }, [profile])

  // 새 키가 양쪽 모두 입력된 경우에만 감지 가능
  const labels = brokerProfileLabels(form.broker_id)
  const isKisProfile = form.broker_id === 'kis'
  const isTossProfile = form.broker_id === 'toss'
  const canDetect = isKisProfile && form.app_key.trim().length > 0 && form.app_secret.trim().length > 0
  const newTossCredentialsReady = isTossProfile
    && form.app_key.trim().length > 0
    && form.app_secret.trim().length > 0
  const canLookupTossAccounts = isTossProfile && (!!profile || newTossCredentialsReady)
  const isListingAnyTossAccounts = session.lookup.pending()

  const handleDetect = async () => {
    if (!canDetect) return
    const ticket = session.detect.begin()
    if (!ticket) return
    setDetectStatus('detecting')
    setDetectMsg('')
    try {
      const res = await detectType({ appKey: form.app_key.trim(), appSecret: form.app_secret.trim() })
      if (!session.detect.current(ticket)) return
      setForm((f) => ({ ...f, is_paper_trading: res.is_paper_trading }))
      setDetectStatus('detected')
      setDetectMsg(res.message)
    } catch (e) {
      if (!session.detect.current(ticket)) return
      setDetectStatus('failed')
      setDetectMsg(cmdErrMsg(e))
    } finally {
      session.detect.finish(ticket)
    }
  }

  const resetTossAccounts = () => {
    setTossAccounts([])
    setTossAccountMsg('')
  }

  const applyTossAccounts = (accounts: TossAccountOptionView[]) => {
    setTossAccounts(accounts)
    setTossAccountMsg(
      accounts.length > 0
        ? `${accounts.length}개 계좌를 조회했습니다. 저장할 accountSeq를 선택하세요.`
        : '조회된 토스증권 계좌가 없습니다.',
    )
    setForm((f) => {
      if (accounts.some((account) => account.account_seq === f.account_no)) return f
      if (accounts.length === 1) return { ...f, account_no: accounts[0].account_seq }
      return f
    })
  }

  const handleLookupTossAccounts = async () => {
    if (!profile) return
    if (!canLookupTossAccounts) {
      setError('토스증권 Client ID와 Client Secret을 먼저 입력하세요.')
      return
    }
    const ticket = session.lookup.begin()
    if (!ticket) return
    setError(null)
    try {
      const accounts = newTossCredentialsReady
        ? await listTossAccounts({ client_id: form.app_key.trim(), client_secret: form.app_secret.trim() })
        : await listTossProfileAccounts(profile.id)
      if (!session.lookup.current(ticket)) return
      applyTossAccounts(accounts)
    } catch (e) {
      if (!session.lookup.current(ticket)) return
      resetTossAccounts()
      setError(cmdErrMsg(e))
    } finally {
      session.lookup.finish(ticket)
    }
  }

  const handleSubmit = async () => {
    if (session.main.pending() || !profile) return
    if (!form.name.trim()) { setError('프로파일 이름을 입력하세요.'); return }
    if (form.broker_id === 'toss' && form.account_no.trim() && !/^\d+$/.test(form.account_no.trim())) {
      setError('토스증권 accountSeq는 숫자여야 합니다.')
      return
    }

    const ticket = session.main.begin()
    if (!ticket) return
    const input: UpdateProfileInput = {
      id: profile.id,
      broker_id: profile.broker_id,
      name: form.name,
      is_paper_trading: form.is_paper_trading,
      live_trading_consent: form.live_trading_consent,
      app_key: form.app_key || undefined,
      app_secret: form.app_secret || undefined,
      account_no: form.account_no || undefined,
    }
    try {
      await updateProfile(input)
      if (!session.main.finish(ticket)) return
      session.close()
      setError(null); setDetectStatus('idle'); setDetectMsg(''); onClose()
    } catch (e) {
      if (session.main.current(ticket)) setError(cmdErrMsg(e))
    } finally {
      session.main.finish(ticket)
    }
  }

  const handleClose = () => {
    if (session.main.pending()) return
    session.close()
    setError(null); setDetectStatus('idle'); setDetectMsg(''); resetTossAccounts(); onClose()
  }

  return (
    <Dialog open={!!profile} onClose={handleClose} maxWidth="sm" fullWidth>
      <DialogTitle>프로파일 편집 — {profile?.name}</DialogTitle>
      <DialogContent>
        <Stack spacing={2} sx={{ mt: 1 }}>
          {error && <Alert severity="error">{error}</Alert>}
          <Chip
            size="small"
            label={brokerLabel(form.broker_id)}
            color={form.broker_id === 'kis' ? 'info' : 'secondary'}
            variant="outlined"
            sx={{ alignSelf: 'flex-start' }}
          />
          {form.broker_id === 'toss' && (
            <>
              <Alert severity="info" sx={{ py: 0.75 }}>
                토스증권 프로파일은 연결 진단과 주문 전 검증을 거친 뒤 실거래 동의가 저장된 경우 실제 주문과 자동매매를 실행할 수 있습니다.
              </Alert>
              <FormControlLabel
                control={
                  <Switch
                    checked={form.live_trading_consent}
                    onChange={(e) => setForm({ ...form, live_trading_consent: e.target.checked })}
                  />
                }
                label={
                  <Stack direction="row" spacing={1} sx={{ alignItems: "center" }}>
                    <Chip
                      size="small"
                      label={form.live_trading_consent ? '실거래 동의 저장' : '실거래 동의 없음'}
                      color={form.live_trading_consent ? 'warning' : 'default'}
                      variant={form.live_trading_consent ? 'filled' : 'outlined'}
                    />
                    <Typography variant="caption" color="text.secondary">
                      향후 토스 주문 연결 전 명시 승인 상태로만 사용
                    </Typography>
                  </Stack>
                }
              />
            </>
          )}
          <TextField
            label="프로파일 이름"
            value={form.name}
            onChange={(e) => setForm({ ...form, name: e.target.value })}
            fullWidth size="small"
          />
          <TextField
            label={`${labels.key} (변경 시에만 입력)`}
            value={form.app_key}
            onChange={(e) => {
              session.invalidateAux()
              setForm({ ...form, app_key: e.target.value })
              setDetectStatus('idle')
              if (isTossProfile) resetTossAccounts()
            }}
            placeholder={profile?.app_key_masked ?? ''}
            fullWidth size="small" autoComplete="off"
          />
          <TextField
            label={`${labels.secret} (변경 시에만 입력)`}
            value={form.app_secret}
            onChange={(e) => {
              session.invalidateAux()
              setForm({ ...form, app_secret: e.target.value })
              setDetectStatus('idle')
              if (isTossProfile) resetTossAccounts()
            }}
            onBlur={() => { if (canDetect) handleDetect() }}
            type="password"
            placeholder="변경 시에만 입력"
            fullWidth size="small" autoComplete="new-password"
          />

          {/* ── 자동 감지 영역 (새 키 입력 시) ──────────────── */}
          {isKisProfile && (
            <Stack direction="row" sx={{ alignItems: "center" }} spacing={1.5}>
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
          )}

          {/* ── 실전/모의 수동 선택 ───────────────────────────── */}
          {isKisProfile && (
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
                <Stack direction="row" spacing={1} sx={{ alignItems: "center" }}>
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
          )}

          {isTossProfile ? (
            <TossAccountSeqField
              idPrefix="edit-profile"
              value={form.account_no}
              onChange={(accountNo) => setForm({ ...form, account_no: accountNo })}
              accounts={tossAccounts}
              loading={isListingAnyTossAccounts}
              onLookup={handleLookupTossAccounts}
              lookupDisabled={!canLookupTossAccounts}
              message={tossAccountMsg}
              helperText={newTossCredentialsReady
                ? labels.accountHelp
                : '저장된 토스증권 키로 계좌를 조회하거나, 새 Client ID/Secret을 입력한 뒤 조회합니다.'}
            />
          ) : (
            <>
              <TextField
                label={labels.account}
                placeholder={labels.accountPlaceholder}
                value={form.account_no}
                onChange={(e) => setForm({ ...form, account_no: e.target.value })}
                fullWidth size="small"
              />
              {labels.accountHelp && (
                <Typography variant="caption" color="text.secondary">
                  {labels.accountHelp}
                </Typography>
              )}
            </>
          )}
          {(()=>{
            if (!isKisProfile) return null
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
        <Button onClick={handleSubmit} variant="contained" aria-label="저장"
          aria-disabled={isPending} aria-busy={isPending} aria-description={isPending ? '저장 중' : undefined}
          startIcon={isPending ? <CircularProgress size={18} color="inherit" aria-label="저장 중" /> : undefined}>
          저장
        </Button>
      </DialogActions>
    </Dialog>
  )
}
