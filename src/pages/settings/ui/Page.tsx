import { useState, useEffect, useRef } from 'react'
import InputAdornment from '@mui/material/InputAdornment'
import Typography from '@mui/material/Typography'
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
import TextField from '@mui/material/TextField'

import LightModeIcon from '@mui/icons-material/LightMode'
import DarkModeIcon from '@mui/icons-material/DarkMode'
import SettingsBrightnessIcon from '@mui/icons-material/SettingsBrightness'
import NotificationsActiveIcon from '@mui/icons-material/NotificationsActive'

import Select from '@mui/material/Select'
import MenuItem from '@mui/material/MenuItem'
import InputLabel from '@mui/material/InputLabel'
import FormControl from '@mui/material/FormControl'

import StorageIcon from '@mui/icons-material/Storage'

import { useSettingsStore } from '../../../entities/settings'
import {
  useAppConfig,
  useSendTestDiscord,
  useLogConfig,
  useSetLogConfig,
  useTradeArchiveConfig,
  useSetTradeArchiveConfig,
  useTradeArchiveStats,
  useWebConfig,
  useSaveWebConfig,
  useStockListStats,
  useSetStockUpdateInterval,
  useRiskConfig,
  useUpdateRiskConfig,
  useRefreshConfig,
  useSetRefreshConfig,
} from '../../../api/hooks'
import type { UpdateRiskConfigInput } from '../../../api/types'
import type { ThemeMode } from '../../../shared/config/theme'
import { fmtNumber } from '../../../shared/lib'

import { AccountProfilesSection } from './accountProfiles'
import { Section } from './section'

const fmt = (n: number) => fmtNumber(n)

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
// ── 리스크 관리 섹션 ──────────────────────────────────────────────
function RiskSection() {
  const { data: risk, isLoading } = useRiskConfig()
  const { mutate: updateRisk, isPending: saving } = useUpdateRiskConfig()

  const [lossLimit, setLossLimit] = useState<number>(0)
  const [posRatio, setPosRatio]   = useState<number>(0)
  const [sellLimit, setSellLimit] = useState<number>(1)
  const [lossBlockLimit, setLossBlockLimit] = useState<number>(3)
  const [volatilitySizing, setVolatilitySizing] = useState<boolean>(false)
  const [riskPerTradePct, setRiskPerTradePct] = useState<number>(1)
  const [atrStopMultiplier, setAtrStopMultiplier] = useState<number>(2)
  const [dirty, setDirty]         = useState(false)

  // risk 데이터가 처음 로드되거나 외부에서 변경됐을 때 로컬 상태 초기화
  useEffect(() => {
    if (risk && !dirty) {
      setLossLimit(risk.dailyLossLimit)
      setPosRatio(Math.round(risk.maxPositionRatio * 100))
      setSellLimit(risk.maxDailySellOrdersPerSymbol)
      setLossBlockLimit(risk.maxConsecutiveLossesPerStrategySymbol)
      setVolatilitySizing(risk.volatilitySizingEnabled)
      setRiskPerTradePct(risk.riskPerTradeBps / 100)
      setAtrStopMultiplier(risk.atrStopMultiplier)
    }
  }, [risk, dirty])

  const handleToggleEnabled = (enabled: boolean) => {
    updateRisk({ enabled })
  }

  const handleSave = () => {
    const input: UpdateRiskConfigInput = {
      dailyLossLimit: lossLimit,
      maxPositionRatio: posRatio / 100,
      maxDailyBuyOrdersPerSymbol: 0,
      maxDailySellOrdersPerSymbol: sellLimit,
      maxConsecutiveLossesPerStrategySymbol: lossBlockLimit,
      volatilitySizingEnabled: volatilitySizing,
      riskPerTradeBps: Math.round(riskPerTradePct * 100),
      atrStopMultiplier,
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
              {risk.blockedStrategySymbolCount > 0 && (
                <Alert severity="warning" sx={{ mt: 1 }}>
                  연속 손실로 신규 진입이 차단된 전략/종목 조합 {risk.blockedStrategySymbolCount}개
                </Alert>
              )}
              {risk.volatilitySizingEnabled && (
                <Alert severity={risk.atrSymbolCount > 0 ? 'info' : 'warning'} sx={{ mt: 1 }}>
                  변동성 기반 수량 산정 활성 — ATR 준비 종목 {risk.atrSymbolCount}개
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
                <Alert severity="info" sx={{ py: 0.5 }}>
                  전략/종목별 일일 매수 제한은 해제되어 있습니다. 재진입은 연속 손실 차단, 비상정지, 포지션 비중, ATR 수량 산정으로 관리합니다.
                </Alert>
                <SliderWithInput
                  label="전략/종목별 일일 매도 제한"
                  value={sellLimit}
                  min={0}
                  max={10}
                  step={1}
                  unit="회"
                  disabled={saving}
                  onChange={(v) => { setSellLimit(v); setDirty(true) }}
                  onChangeCommitted={(v) => { setSellLimit(v); setDirty(true) }}
                />
                <SliderWithInput
                  label="전략/종목별 연속 손실 차단"
                  value={lossBlockLimit}
                  min={0}
                  max={10}
                  step={1}
                  unit="회"
                  disabled={saving}
                  onChange={(v) => { setLossBlockLimit(v); setDirty(true) }}
                  onChangeCommitted={(v) => { setLossBlockLimit(v); setDirty(true) }}
                />
                <Divider />
                <Stack direction="row" alignItems="flex-start" justifyContent="space-between">
                  <Box>
                    <Typography variant="body2" fontWeight={600}>
                      변동성 기반 주문 수량
                    </Typography>
                    <Typography variant="caption" color="text.secondary">
                      자동매매 시작 시 일봉 ATR을 읽고 계좌 위험 한도와 손절폭으로 매수 수량을 계산합니다.
                    </Typography>
                  </Box>
                  <Switch
                    checked={volatilitySizing}
                    onChange={(e) => { setVolatilitySizing(e.target.checked); setDirty(true) }}
                    disabled={saving}
                    color="success"
                  />
                </Stack>
                <SliderWithInput
                  label="거래당 위험 한도"
                  value={riskPerTradePct}
                  min={0}
                  max={10}
                  step={0.1}
                  unit="%"
                  disabled={saving || !volatilitySizing}
                  onChange={(v) => { setRiskPerTradePct(v); setDirty(true) }}
                  onChangeCommitted={(v) => { setRiskPerTradePct(v); setDirty(true) }}
                />
                <SliderWithInput
                  label="ATR 손절 배수"
                  value={atrStopMultiplier}
                  min={0.5}
                  max={10}
                  step={0.5}
                  unit="배"
                  disabled={saving || !volatilitySizing}
                  onChange={(v) => { setAtrStopMultiplier(v); setDirty(true) }}
                  onChangeCommitted={(v) => { setAtrStopMultiplier(v); setDirty(true) }}
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

  // 데이터 갱신 주기 설정 (IPC)
  const { data: refreshCfg } = useRefreshConfig()
  const { mutate: saveRefreshConfig, isPending: refreshSaving } = useSetRefreshConfig()
  const [localIntervalSec, setLocalIntervalSec] = useState<number>(30)

  // refreshCfg 로드 시 로컈 동기화
  const prevRefreshCfgRef = useState<typeof refreshCfg>(undefined)
  if (refreshCfg && refreshCfg !== prevRefreshCfgRef[0]) {
    prevRefreshCfgRef[1](refreshCfg)
    setLocalIntervalSec(refreshCfg.interval_sec)
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

        {/* ── 데이터 갱신 주기 ──────────────────────────────────── */}
        <Section title="데이터 갱신 주기">
          <Stack spacing={2}>
            <SliderWithInput
              label="갱신 주기"
              value={localIntervalSec}
              min={5} max={300} step={5}
              unit="초"
              disabled={refreshSaving}
              onChange={(v) => setLocalIntervalSec(v)}
              onChangeCommitted={(v) => saveRefreshConfig(v)}
            />
            <Typography variant="caption" color="text.secondary">
              잔고, 환율 등 주요 데이터를 백그라운드에서 이 주기마다 자동 갱신합니다. 변경 후 재시작 없이 즉시 적용됩니다.
              <br />
              최소 5초, 최대 300초. 모의투자 모드에서는 KIS API 제한(2 calls/s)을 고려하여 60초 이상을 권장합니다.
            </Typography>
          </Stack>
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

        <AccountProfilesSection />

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
    </Box>
  )
}
