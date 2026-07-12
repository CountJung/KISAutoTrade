import DeleteForeverIcon from '@mui/icons-material/DeleteForever'
import DownloadIcon from '@mui/icons-material/Download'
import DnsIcon from '@mui/icons-material/Dns'
import SaveIcon from '@mui/icons-material/Save'
import StorageIcon from '@mui/icons-material/Storage'
import UploadFileIcon from '@mui/icons-material/UploadFile'
import Alert from '@mui/material/Alert'
import Box from '@mui/material/Box'
import Button from '@mui/material/Button'
import Checkbox from '@mui/material/Checkbox'
import Chip from '@mui/material/Chip'
import CircularProgress from '@mui/material/CircularProgress'
import Dialog from '@mui/material/Dialog'
import DialogActions from '@mui/material/DialogActions'
import DialogContent from '@mui/material/DialogContent'
import DialogTitle from '@mui/material/DialogTitle'
import Divider from '@mui/material/Divider'
import FormControl from '@mui/material/FormControl'
import FormControlLabel from '@mui/material/FormControlLabel'
import Grid from '@mui/material/Grid'
import InputLabel from '@mui/material/InputLabel'
import MenuItem from '@mui/material/MenuItem'
import Paper from '@mui/material/Paper'
import Select from '@mui/material/Select'
import Stack from '@mui/material/Stack'
import Table from '@mui/material/Table'
import TableBody from '@mui/material/TableBody'
import TableCell from '@mui/material/TableCell'
import TableHead from '@mui/material/TableHead'
import TableRow from '@mui/material/TableRow'
import TextField from '@mui/material/TextField'
import Typography from '@mui/material/Typography'
import { useEffect, useState } from 'react'

import {
  useClearDatabaseTables,
  useCreateDatabaseTables,
  useDatabaseConfig,
  useDropDatabaseTables,
  useExportDatabaseToJson,
  useImportJsonToDatabase,
  useJsonStorageInventory,
  useSaveDatabaseConfig,
  useSetStorageBackend,
  useTestDatabaseConnection,
  useTradingStatus,
} from '../../../api/hooks'
import type {
  CmdError,
  DatabaseConfigView,
  DatabaseProvider,
  DatabaseStatusView,
  DatabaseTlsMode,
  DatabaseTransferResult,
  SaveDatabaseConfigInput,
} from '../../../api/types'
import { Section } from './section'

type DestructiveAction = 'clear' | 'drop'

const DEFAULT_CONFIG: DatabaseConfigView = {
  provider: 'postgresql',
  host: '127.0.0.1',
  port: 5432,
  database: 'kisautotrade',
  username: 'kisautotrade',
  passwordConfigured: false,
  tlsMode: 'prefer',
  maxConnections: 5,
  activeBackend: 'json',
  configured: false,
}

function errorMessage(error: unknown) {
  if (error && typeof error === 'object' && 'message' in error) {
    return String((error as CmdError).message)
  }
  return String(error)
}

function formatBytes(bytes: number) {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / 1024 / 1024).toFixed(2)} MB`
}

function providerLabel(provider: DatabaseProvider) {
  return provider === 'postgresql' ? 'PostgreSQL' : 'MariaDB'
}

export function DatabaseManagementSection() {
  const isDesktop = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
  const configQuery = useDatabaseConfig(isDesktop)
  const inventoryQuery = useJsonStorageInventory(isDesktop)
  const { data: tradingStatus } = useTradingStatus()
  const saveMutation = useSaveDatabaseConfig()
  const testMutation = useTestDatabaseConnection()
  const createMutation = useCreateDatabaseTables()
  const clearMutation = useClearDatabaseTables()
  const dropMutation = useDropDatabaseTables()
  const importMutation = useImportJsonToDatabase()
  const exportMutation = useExportDatabaseToJson()
  const backendMutation = useSetStorageBackend()

  const [form, setForm] = useState<SaveDatabaseConfigInput>({
    ...DEFAULT_CONFIG,
    password: '',
  })
  const [status, setStatus] = useState<DatabaseStatusView | null>(null)
  const [transfer, setTransfer] = useState<DatabaseTransferResult | null>(null)
  const [result, setResult] = useState<{ severity: 'success' | 'error' | 'info'; message: string } | null>(null)
  const [destructiveAction, setDestructiveAction] = useState<DestructiveAction | null>(null)
  const [confirmation, setConfirmation] = useState('')
  const [confirmed, setConfirmed] = useState(false)

  const config = configQuery.data ?? DEFAULT_CONFIG
  useEffect(() => {
    if (!configQuery.data) return
    setForm({
      provider: configQuery.data.provider,
      host: configQuery.data.host,
      port: configQuery.data.port,
      database: configQuery.data.database,
      username: configQuery.data.username,
      password: '',
      tlsMode: configQuery.data.tlsMode,
      maxConnections: configQuery.data.maxConnections,
    })
  }, [configQuery.data])

  const busy = saveMutation.isPending
    || testMutation.isPending
    || createMutation.isPending
    || clearMutation.isPending
    || dropMutation.isPending
    || importMutation.isPending
    || exportMutation.isPending
    || backendMutation.isPending
  const activeBackend = status?.activeBackend ?? config.activeBackend
  const connectionLocked = activeBackend === 'database'
  const configDirty = form.provider !== config.provider
    || form.host !== config.host
    || form.port !== config.port
    || form.database !== config.database
    || form.username !== config.username
    || form.tlsMode !== config.tlsMode
    || form.maxConnections !== config.maxConnections
    || Boolean(form.password)
  const schemaReady = status?.tables.every((table) => table.exists)
    && status.schemaVersion === status.requiredSchemaVersion
  const tradingRunning = tradingStatus?.isRunning ?? false
  const requiredConfirmation = destructiveAction === 'drop'
    ? 'DROP KISAUTOTRADE TABLES'
    : 'CLEAR KISAUTOTRADE DATA'

  const fail = (error: unknown) => setResult({ severity: 'error', message: errorMessage(error) })

  const saveConfig = () => {
    setResult(null)
    saveMutation.mutate(form, {
      onSuccess: (saved) => {
        setStatus(null)
        setResult({ severity: 'success', message: 'DB 연결 설정을 저장했습니다. 저장 backend는 안전하게 JSON으로 유지됩니다.' })
        setForm((current) => ({ ...current, password: '' }))
        if (saved.passwordConfigured) {
          setResult({ severity: 'success', message: 'DB 연결 설정과 password를 안전한 앱 설정 경로에 저장했습니다.' })
        }
      },
      onError: fail,
    })
  }

  const testConnection = () => {
    setResult(null)
    setStatus(null)
    testMutation.mutate(undefined, {
      onSuccess: (next) => {
        setStatus(next)
        setResult({ severity: 'success', message: next.message })
      },
      onError: (error) => {
        setStatus(null)
        fail(error)
      },
    })
  }

  const createTables = () => {
    setResult(null)
    createMutation.mutate(undefined, {
      onSuccess: (next) => {
        setStatus(next)
        setResult({ severity: 'success', message: 'KISAutoTrade 관리 테이블을 생성·검증했습니다.' })
      },
      onError: fail,
    })
  }

  const importJson = () => {
    setResult(null)
    importMutation.mutate(undefined, {
      onSuccess: (next) => {
        setTransfer(next)
        setResult({ severity: 'success', message: next.message })
        testConnection()
      },
      onError: fail,
    })
  }

  const exportJson = () => {
    setResult(null)
    exportMutation.mutate(undefined, {
      onSuccess: (next) => {
        setTransfer(next)
        setResult({ severity: 'success', message: next.message })
      },
      onError: fail,
    })
  }

  const setBackend = (backend: 'json' | 'database') => {
    setResult(null)
    backendMutation.mutate(backend, {
      onSuccess: (next) => {
        setStatus(next)
        setResult({
          severity: 'success',
          message: backend === 'database'
            ? 'DB 저장을 활성화했습니다. 이후 공통 저장소 읽기/쓰기는 DB를 사용합니다.'
            : 'JSON 파일 저장을 활성화했습니다.',
        })
      },
      onError: fail,
    })
  }

  const runDestructiveAction = () => {
    if (!destructiveAction || !confirmed || confirmation !== requiredConfirmation) return
    const mutation = destructiveAction === 'drop' ? dropMutation : clearMutation
    mutation.mutate(confirmation, {
      onSuccess: (next) => {
        setStatus(next)
        setDestructiveAction(null)
        setConfirmation('')
        setConfirmed(false)
        setResult({ severity: 'success', message: destructiveAction === 'drop' ? '앱 관리 테이블을 삭제했습니다.' : 'DB 문서 데이터를 비웠습니다.' })
      },
      onError: fail,
    })
  }

  const closeDestructiveDialog = () => {
    setDestructiveAction(null)
    setConfirmation('')
    setConfirmed(false)
  }

  return (
    <Section title="데이터베이스 및 데이터 이관">
      {!isDesktop ? (
        <Alert severity="warning">
          DB password와 테이블 삭제 기능은 보안을 위해 Tauri 데스크톱 앱 Settings에서만 사용할 수 있습니다.
        </Alert>
      ) : (
        <Stack spacing={2.5}>
          {configQuery.isError && <Alert severity="error">DB 설정을 불러오지 못했습니다: {errorMessage(configQuery.error)}</Alert>}
          {inventoryQuery.isError && <Alert severity="error">JSON 저장 현황을 불러오지 못했습니다: {errorMessage(inventoryQuery.error)}</Alert>}
          <Stack direction="row" spacing={1} alignItems="center" flexWrap="wrap" useFlexGap>
            <Chip
              icon={<StorageIcon />}
              label={`현재 저장: ${activeBackend === 'database' ? 'Database' : 'JSON'}`}
              color={activeBackend === 'database' ? 'success' : 'default'}
              variant="outlined"
            />
            <Chip
              label={config.passwordConfigured ? 'DB password 저장됨' : 'DB password 미설정'}
              color={config.passwordConfigured ? 'default' : 'warning'}
              variant="outlined"
            />
            {tradingRunning && <Chip label="자동매매 실행 중 · 관리 작업 잠금" color="warning" />}
          </Stack>

          <Box>
            <Typography variant="subtitle2" fontWeight={700} mb={1}>1. 연결 설정</Typography>
            <Grid container spacing={1.5}>
              <Grid item xs={12} sm={6} md={3}>
                <FormControl size="small" fullWidth>
                  <InputLabel>DB 종류</InputLabel>
                  <Select
                    label="DB 종류"
                    value={form.provider}
                    disabled={busy || tradingRunning}
                    onChange={(event) => {
                      const provider = event.target.value as DatabaseProvider
                      setForm((current) => ({
                        ...current,
                        provider,
                        port: provider === 'postgresql' ? 5432 : 3306,
                      }))
                      setStatus(null)
                    }}
                  >
                    <MenuItem value="postgresql">PostgreSQL</MenuItem>
                    <MenuItem value="mariadb">MariaDB</MenuItem>
                  </Select>
                </FormControl>
              </Grid>
              <Grid item xs={12} sm={6} md={5}>
                <TextField label="Host" size="small" fullWidth value={form.host} disabled={busy || tradingRunning} onChange={(event) => setForm((current) => ({ ...current, host: event.target.value }))} />
              </Grid>
              <Grid item xs={12} sm={6} md={2}>
                <TextField label="Port" type="number" size="small" fullWidth value={form.port} disabled={busy || tradingRunning} inputProps={{ min: 1, max: 65535 }} onChange={(event) => setForm((current) => ({ ...current, port: Number(event.target.value) }))} />
              </Grid>
              <Grid item xs={12} sm={6} md={2}>
                <TextField label="최대 연결" type="number" size="small" fullWidth value={form.maxConnections} disabled={busy || tradingRunning} inputProps={{ min: 1, max: 20 }} onChange={(event) => setForm((current) => ({ ...current, maxConnections: Number(event.target.value) }))} />
              </Grid>
              <Grid item xs={12} sm={6} md={4}>
                <TextField
                  label="Database"
                  size="small"
                  fullWidth
                  value={form.database}
                  disabled={busy || tradingRunning}
                  helperText="입력한 사용자/암호로 접속은 되지만 DB가 없으면 연결 테스트 시 자동 생성합니다."
                  onChange={(event) => setForm((current) => ({ ...current, database: event.target.value }))}
                />
              </Grid>
              <Grid item xs={12} sm={6} md={4}>
                <TextField label="Username" size="small" fullWidth value={form.username} disabled={busy || tradingRunning} autoComplete="username" onChange={(event) => setForm((current) => ({ ...current, username: event.target.value }))} />
              </Grid>
              <Grid item xs={12} sm={6} md={4}>
                <TextField
                  label="Password"
                  type="password"
                  size="small"
                  fullWidth
                  value={form.password ?? ''}
                  disabled={busy || tradingRunning}
                  autoComplete="new-password"
                  placeholder={config.passwordConfigured ? '비워두면 기존 password 유지' : 'DB password 입력'}
                  onChange={(event) => setForm((current) => ({ ...current, password: event.target.value }))}
                />
              </Grid>
              <Grid item xs={12} sm={6} md={4}>
                <FormControl size="small" fullWidth>
                  <InputLabel>TLS</InputLabel>
                  <Select label="TLS" value={form.tlsMode} disabled={busy || tradingRunning} onChange={(event) => setForm((current) => ({ ...current, tlsMode: event.target.value as DatabaseTlsMode }))}>
                    <MenuItem value="disable">사용 안 함</MenuItem>
                    <MenuItem value="prefer">가능하면 사용</MenuItem>
                    <MenuItem value="require">필수</MenuItem>
                  </Select>
                </FormControl>
              </Grid>
            </Grid>
            <Stack direction={{ xs: 'column', sm: 'row' }} spacing={1} mt={1.5}>
              <Button variant="contained" startIcon={saveMutation.isPending ? <CircularProgress size={16} color="inherit" /> : <SaveIcon />} disabled={busy || tradingRunning || connectionLocked} onClick={saveConfig}>설정 저장</Button>
              <Button variant="outlined" startIcon={testMutation.isPending ? <CircularProgress size={16} /> : <DnsIcon />} disabled={busy || tradingRunning || !config.configured || configDirty} onClick={testConnection}>연결 테스트</Button>
            </Stack>
            {configDirty && <Alert severity="info" sx={{ mt: 1.5 }}>변경한 연결 설정을 저장한 뒤 연결 테스트와 테이블 관리를 실행하세요.</Alert>}
            {connectionLocked && <Alert severity="warning" sx={{ mt: 1.5 }}>DB가 현재 원본입니다. 연결 설정을 바꾸려면 먼저 “JSON 저장 사용”으로 전환해 최신 데이터를 복구하세요.</Alert>}
          </Box>

          <Divider />

          <Box>
            <Typography variant="subtitle2" fontWeight={700} mb={1}>2. 앱 테이블 관리</Typography>
            {status ? (
              <Stack spacing={1.25}>
                <Alert severity={schemaReady ? 'success' : 'warning'}>
                  {status.message} · {providerLabel(status.provider)} {status.serverVersion ?? ''} · {status.latencyMs ?? '-'}ms
                </Alert>
                <Paper variant="outlined" sx={{ overflowX: 'auto' }}>
                  <Table size="small">
                    <TableHead><TableRow><TableCell>테이블</TableCell><TableCell>용도</TableCell><TableCell align="center">상태</TableCell><TableCell align="right">행</TableCell></TableRow></TableHead>
                    <TableBody>{status.tables.map((table) => (
                      <TableRow key={table.name}>
                        <TableCell><code>{table.name}</code></TableCell>
                        <TableCell>{table.purpose}</TableCell>
                        <TableCell align="center"><Chip size="small" label={table.exists ? '생성됨' : '없음'} color={table.exists ? 'success' : 'warning'} variant="outlined" /></TableCell>
                        <TableCell align="right">{table.rowCount.toLocaleString('ko-KR')}</TableCell>
                      </TableRow>
                    ))}</TableBody>
                  </Table>
                </Paper>
              </Stack>
            ) : (
              <Alert severity="info">연결 테스트를 실행하면 서버·스키마·테이블 상태를 표시합니다.</Alert>
            )}
            <Stack direction={{ xs: 'column', sm: 'row' }} spacing={1} mt={1.5} flexWrap="wrap" useFlexGap>
              <Button variant="outlined" disabled={busy || tradingRunning || configDirty || !status?.connected} onClick={createTables}>테이블 생성/업그레이드</Button>
              <Button color="warning" variant="outlined" disabled={busy || tradingRunning || configDirty || activeBackend === 'database' || !status?.tables.some((table) => table.exists)} onClick={() => setDestructiveAction('clear')}>데이터 비우기</Button>
              <Button color="error" variant="outlined" startIcon={<DeleteForeverIcon />} disabled={busy || tradingRunning || configDirty || activeBackend === 'database' || !status?.tables.some((table) => table.exists)} onClick={() => setDestructiveAction('drop')}>앱 테이블 삭제</Button>
            </Stack>
          </Box>

          <Divider />

          <Box>
            <Typography variant="subtitle2" fontWeight={700} mb={1}>3. JSON 가져오기·DB 반출·저장 전환</Typography>
            {inventoryQuery.data ? (
              <Stack direction="row" spacing={1} flexWrap="wrap" useFlexGap mb={1.5}>
                <Chip label={`JSON ${inventoryQuery.data.fileCount}개`} variant="outlined" />
                <Chip label={formatBytes(inventoryQuery.data.sizeBytes)} variant="outlined" />
                {inventoryQuery.data.categories.map((category) => (
                  <Chip key={category.category} size="small" label={`${category.category} ${category.fileCount}`} />
                ))}
              </Stack>
            ) : <Alert severity="info" sx={{ mb: 1.5 }}>JSON 저장 현황을 확인 중입니다.</Alert>}
            <Stack direction={{ xs: 'column', md: 'row' }} spacing={1} flexWrap="wrap" useFlexGap>
              <Button variant="outlined" startIcon={importMutation.isPending ? <CircularProgress size={16} /> : <UploadFileIcon />} disabled={busy || tradingRunning || configDirty || !schemaReady || activeBackend === 'database'} onClick={importJson}>JSON → DB 가져오기</Button>
              <Button variant="outlined" startIcon={exportMutation.isPending ? <CircularProgress size={16} /> : <DownloadIcon />} disabled={busy || tradingRunning || configDirty || !schemaReady} onClick={exportJson}>DB → JSON 스냅샷 반출</Button>
              <Button variant={activeBackend === 'json' ? 'contained' : 'outlined'} disabled={busy || tradingRunning || activeBackend === 'json'} onClick={() => setBackend('json')}>JSON 저장 사용</Button>
              <Button color="success" variant={activeBackend === 'database' ? 'contained' : 'outlined'} disabled={busy || tradingRunning || configDirty || !schemaReady || activeBackend === 'database'} onClick={() => setBackend('database')}>DB 저장 활성화</Button>
            </Stack>
            {transfer && (
              <Alert severity="success" sx={{ mt: 1.5 }}>
                {transfer.message} · {formatBytes(transfer.sizeBytes)} · SHA-256 {transfer.checksum.slice(0, 16)}…
                {transfer.outputPath && <Typography variant="caption" display="block">반출 경로: <code>{transfer.outputPath}</code></Typography>}
              </Alert>
            )}
          </Box>

          {result && <Alert severity={result.severity}>{result.message}</Alert>}
        </Stack>
      )}

      <Dialog open={destructiveAction !== null} onClose={() => !busy && closeDestructiveDialog()} maxWidth="sm" fullWidth>
        <DialogTitle>{destructiveAction === 'drop' ? 'KISAutoTrade 앱 테이블 삭제' : 'DB 문서 데이터 비우기'}</DialogTitle>
        <DialogContent>
          <Stack spacing={2} mt={1}>
            <Alert severity="error">
              이 작업은 되돌릴 수 없습니다. 먼저 “DB → JSON 스냅샷 반출”을 실행하세요. 계좌 자격증명과 로그는 대상에 포함되지 않습니다.
            </Alert>
            <TextField label="확인 문구" value={confirmation} onChange={(event) => setConfirmation(event.target.value)} helperText={requiredConfirmation} fullWidth autoComplete="off" />
            <FormControlLabel control={<Checkbox checked={confirmed} onChange={(event) => setConfirmed(event.target.checked)} />} label="삭제 범위와 복구 불가 경고를 확인했습니다." />
            {result?.severity === 'error' && <Alert severity="error">{result.message}</Alert>}
          </Stack>
        </DialogContent>
        <DialogActions>
          <Button onClick={closeDestructiveDialog} disabled={busy}>취소</Button>
          <Button color="error" variant="contained" disabled={busy || !confirmed || confirmation !== requiredConfirmation} onClick={runDestructiveAction}>실행</Button>
        </DialogActions>
      </Dialog>
    </Section>
  )
}
