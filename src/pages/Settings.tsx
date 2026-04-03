import { useState } from 'react'
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
  useWebConfig,
  useSaveWebConfig,
} from '../api/hooks'
import type { AccountProfileView, AddProfileInput, UpdateProfileInput } from '../api/types'
import type { ThemeMode } from '../theme'

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

// ── 프로파일 입력 폼 상태 ──────────────────────────────────────────
interface ProfileFormState {
  name: string
  is_paper_trading: boolean
  app_key: string
  app_secret: string
  account_no: string
}

const emptyForm = (): ProfileFormState => ({
  name: '',
  is_paper_trading: true,
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
  const { mutate: addProfile, isPending } = useAddProfile()

  const handleSubmit = () => {
    if (!form.name.trim()) { setError('프로파일 이름을 입력하세요.'); return }
    if (!form.app_key.trim()) { setError('APP KEY를 입력하세요.'); return }
    if (!form.app_secret.trim()) { setError('APP SECRET을 입력하세요.'); return }
    if (!form.account_no.trim()) { setError('계좌번호를 입력하세요.'); return }

    const input: AddProfileInput = { ...form }
    addProfile(input, {
      onSuccess: () => { setForm(emptyForm()); setError(null); onClose() },
      onError: (e) => setError(String(e)),
    })
  }

  const handleClose = () => { setForm(emptyForm()); setError(null); onClose() }

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
            placeholder="예: 모의투자 1호 계좌"
            fullWidth size="small"
          />
          <FormControlLabel
            control={
              <Switch
                checked={form.is_paper_trading}
                onChange={(e) => setForm({ ...form, is_paper_trading: e.target.checked })}
              />
            }
            label={
              <Chip
                size="small"
                label={form.is_paper_trading ? '모의투자' : '실전투자'}
                color={form.is_paper_trading ? 'warning' : 'primary'}
              />
            }
          />
          <TextField
            label="APP KEY"
            value={form.app_key}
            onChange={(e) => setForm({ ...form, app_key: e.target.value })}
            fullWidth size="small" autoComplete="off"
          />
          <TextField
            label="APP SECRET"
            value={form.app_secret}
            onChange={(e) => setForm({ ...form, app_secret: e.target.value })}
            type="password"
            fullWidth size="small" autoComplete="new-password"
          />
          <TextField
            label="계좌번호 (10자리 입력, 예: 12345678-01)"
            value={form.account_no}
            onChange={(e) => setForm({ ...form, account_no: e.target.value })}
            fullWidth size="small"
          />
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
  const { mutate: updateProfile, isPending } = useUpdateProfile()

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
      onSuccess: () => { setError(null); onClose() },
      onError: (e) => setError(String(e)),
    })
  }

  const handleClose = () => { setError(null); onClose() }

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
          <FormControlLabel
            control={
              <Switch
                checked={form.is_paper_trading}
                onChange={(e) => setForm({ ...form, is_paper_trading: e.target.checked })}
              />
            }
            label={
              <Chip
                size="small"
                label={form.is_paper_trading ? '모의투자' : '실전투자'}
                color={form.is_paper_trading ? 'warning' : 'primary'}
              />
            }
          />
          <TextField
            label="APP KEY (변경 시에만 입력)"
            value={form.app_key}
            onChange={(e) => setForm({ ...form, app_key: e.target.value })}
            placeholder={profile?.app_key_masked ?? ''}
            fullWidth size="small" autoComplete="off"
          />
          <TextField
            label="APP SECRET (변경 시에만 입력)"
            value={form.app_secret}
            onChange={(e) => setForm({ ...form, app_secret: e.target.value })}
            type="password"
            placeholder="변경 시에만 입력"
            fullWidth size="small" autoComplete="new-password"
          />
          <TextField
            label="계좌번호"
            value={form.account_no}
            onChange={(e) => setForm({ ...form, account_no: e.target.value })}
            fullWidth size="small"
          />
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
}: {
  profile: AccountProfileView
  onEdit: (p: AccountProfileView) => void
  onDelete: (p: AccountProfileView) => void
  onSetActive: (id: string) => void
}) {
  const { isPending: activating } = useSetActiveProfile()

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
          <Stack direction="row" spacing={1} alignItems="center">
            {profile.is_active ? (
              <RadioButtonCheckedIcon color="primary" fontSize="small" />
            ) : (
              <RadioButtonUncheckedIcon color="disabled" fontSize="small" />
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
          </Stack>
          <Typography variant="body2" color="text.secondary" pl={3.5}>
            KEY: <code>{profile.app_key_masked}</code>
            &nbsp;&nbsp;계좌: <code>{profile.account_no || '(미설정)'}</code>
          </Typography>
        </Stack>

        <Stack direction="row" spacing={0.5} alignItems="center">
          {!profile.is_active && (
            <Tooltip title="이 프로파일로 전환">
              <span>
                <Button
                  size="small"
                  variant="outlined"
                  onClick={() => onSetActive(profile.id)}
                  disabled={activating}
                  sx={{ whiteSpace: 'nowrap' }}
                >
                  활성화
                </Button>
              </span>
            </Tooltip>
          )}
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
export default function Settings() {
  const {
    theme, discordEnabled,
    setTheme, setDiscordEnabled,
  } = useSettingsStore()

  const { data: appConfig } = useAppConfig()
  const { data: diag, refetch: recheckConfig, isFetching: diagFetching } = useCheckConfig()
  const { mutate: sendTestDiscord, isPending: discordPending } = useSendTestDiscord()
  const [discordResult, setDiscordResult] = useState<{ ok: boolean; msg: string } | null>(null)

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
  const [webSaveResult, setWebSaveResult] = useState<{ ok: boolean; msg: string } | null>(null)
  // webConfig가 로드되면 입력칼 동기화
  const prevWebConfigRef = useState<typeof webConfig>(undefined)
  if (webConfig && webConfig !== prevWebConfigRef[0]) {
    prevWebConfigRef[1](webConfig)
    setWebPortInput(String(webConfig.runningPort))
  }

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
            <Box>
              <Typography variant="body2" gutterBottom>
                보관 기간: {localRetentionDays}일
              </Typography>
              <Slider
                value={localRetentionDays}
                min={1} max={30} step={1}
                onChange={(_, v) => setLocalRetentionDays(v as number)}
                onChangeCommitted={(_, v) => saveLogConfig({
                  retention_days: v as number,
                  max_size_mb: localMaxSizeMb,
                })}
                sx={{ maxWidth: 300 }}
                valueLabelDisplay="auto"
                disabled={logSaving}
              />
            </Box>
            <Box>
              <Typography variant="body2" gutterBottom>
                최대 용량: {localMaxSizeMb}MB
              </Typography>
              <Slider
                value={localMaxSizeMb}
                min={10} max={500} step={10}
                onChange={(_, v) => setLocalMaxSizeMb(v as number)}
                onChangeCommitted={(_, v) => saveLogConfig({
                  retention_days: localRetentionDays,
                  max_size_mb: v as number,
                })}
                sx={{ maxWidth: 300 }}
                valueLabelDisplay="auto"
                disabled={logSaving}
              />
            </Box>
            <Typography variant="caption" color="text.secondary">
              로그 파일 위치: <code>./logs/</code> (앱 실행 폴더 기준)
              <br />
              설정은 즉시 적용되며 초과 파일은 자동 정리됩니다.
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
            <Box display="flex" alignItems="center" gap={1}>
              <TextField
                label="웹 서버 포트"
                value={webPortInput}
                onChange={(e) => { setWebPortInput(e.target.value); setWebSaveResult(null) }}
                size="small"
                type="number"
                inputProps={{ min: 1024, max: 65535 }}
                sx={{ width: 140 }}
                helperText="기본값: 7474 (재시작 후 적용)"
              />
              <Button
                variant="outlined"
                onClick={() => {
                  const port = parseInt(webPortInput, 10)
                  if (isNaN(port)) return
                  setWebSaveResult(null)
                  saveWebConfig(port, {
                    onSuccess: (msg) => setWebSaveResult({ ok: true, msg }),
                    onError: (err) => setWebSaveResult({ ok: false, msg: String(err) }),
                  })
                }}
                disabled={webSaving || !webPortInput}
                startIcon={webSaving ? <CircularProgress size={16} /> : null}
              >
                저장
              </Button>
            </Box>
            {webSaveResult && (
              <Alert severity={webSaveResult.ok ? 'success' : 'error'}>
                {webSaveResult.msg}
                {webSaveResult.ok && ' — 앱을 재시작하면 새 포트가 적용됩니다.'}
              </Alert>
            )}
          </Stack>
        </Section>

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
