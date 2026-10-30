import { useEffect, useRef, useState, useCallback } from 'react'
import Typography from '@mui/material/Typography'
import Paper from '@mui/material/Paper'
import Box from '@mui/material/Box'
import Stack from '@mui/material/Stack'
import ToggleButton from '@mui/material/ToggleButton'
import ToggleButtonGroup from '@mui/material/ToggleButtonGroup'
import TextField from '@mui/material/TextField'
import Chip from '@mui/material/Chip'
import Button from '@mui/material/Button'
import CircularProgress from '@mui/material/CircularProgress'
import useMediaQuery from '@mui/material/useMediaQuery'

import { useRecentLogs } from '../../../api/hooks'
import type { AppLogEntry } from '../../../api/types'
import {
  hasProviderTrace,
  LayoutResizer,
  parseProviderTraceText,
  ProviderTraceChips,
} from '../../../shared/ui'
import { clampNumber, readStoredNumber, writeStoredNumber } from '../../../shared/lib'

type LogLevel = 'ALL' | 'DEBUG' | 'INFO' | 'WARN' | 'ERROR' | 'TRACE'

const LOG_HEIGHT_KEY = 'act:panel:log:height'
const LOG_HEIGHT_DEFAULT = 480
const LOG_HEIGHT_MIN = 160
const LOG_HEIGHT_MAX = 900

const LEVEL_COLORS: Record<string, string> = {
  ALL:   '#888',
  DEBUG: '#888',
  TRACE: '#888',
  INFO:  '#2196f3',
  WARN:  '#ff9800',
  ERROR: '#f44336',
}

export default function Log() {
  const isMobile = useMediaQuery('(max-width:600px)')
  const [level, setLevel]   = useState<LogLevel>('ALL')
  const [search, setSearch] = useState('')
  const [logHeight, setLogHeight] = useState(() =>
    readStoredNumber(LOG_HEIGHT_KEY, LOG_HEIGHT_DEFAULT, LOG_HEIGHT_MIN, LOG_HEIGHT_MAX)
  )
  const bottomRef = useRef<HTMLDivElement>(null)
  const logPanelRef = useRef<HTMLDivElement>(null)
  const shouldStickToBottomRef = useRef(true)
  const logHeightRef = useRef(logHeight)
  logHeightRef.current = logHeight

  const handleLogResize = useCallback((delta: number) => {
    setLogHeight((height) => clampNumber(height + delta, LOG_HEIGHT_MIN, LOG_HEIGHT_MAX))
  }, [])

  const handleLogResizeEnd = useCallback(() => {
    writeStoredNumber(LOG_HEIGHT_KEY, logHeightRef.current, LOG_HEIGHT_MIN, LOG_HEIGHT_MAX)
  }, [])

  const handleLogValueChange = useCallback((value: number) => {
    const next = clampNumber(value, LOG_HEIGHT_MIN, LOG_HEIGHT_MAX)
    setLogHeight(next)
    writeStoredNumber(LOG_HEIGHT_KEY, next, LOG_HEIGHT_MIN, LOG_HEIGHT_MAX)
  }, [])

  const logsQuery = useRecentLogs(300)
  const { data: logs = [] } = logsQuery

  const handleLogScroll = useCallback(() => {
    const el = logPanelRef.current
    if (!el) return
    const distanceFromBottom = el.scrollHeight - el.scrollTop - el.clientHeight
    shouldStickToBottomRef.current = distanceFromBottom < 48
  }, [])

  // 새 로그가 올 때 사용자가 하단 근처에 있는 경우에만 하단 유지
  useEffect(() => {
    if (shouldStickToBottomRef.current) {
      bottomRef.current?.scrollIntoView({ block: 'end' })
    }
  }, [logs])

  const filtered = logs.filter((log: AppLogEntry) => {
    const levelOk  = level === 'ALL' || log.level === level
    const searchOk = !search || log.message.includes(search) || log.target.includes(search)
    return levelOk && searchOk
  })

  const hasConfirmedLogs = !logsQuery.isPlaceholderData && logsQuery.dataUpdatedAt > 0 && logsQuery.data !== undefined
  const hasReadError = logsQuery.isError || (!hasConfirmedLogs && logsQuery.errorUpdatedAt > 0)
  const statusText = hasReadError
    ? hasConfirmedLogs ? '로그 재조회 실패 — 이전 자료 표시' : '로그 불러오기 실패'
    : hasConfirmedLogs ? '로그 조회 완료' : '로그 불러오는 중'
  const checkedAt = hasConfirmedLogs ? new Date(logsQuery.dataUpdatedAt) : null

  return (
    <Box sx={{ display: 'flex', flexDirection: 'column', height: '100%' }}>
      <Typography variant="h5" sx={{ fontWeight: 700, mb: 2 }}>
        Log
      </Typography>

      <Box data-testid="log-query-status" sx={{ mb: 1.5 }}>
        <Stack direction="row" spacing={1} sx={{ alignItems: 'center', flexWrap: 'wrap' }}>
          <Typography role="status" aria-live="polite" aria-atomic="true" variant="body2" color={hasReadError ? 'warning.main' : 'text.secondary'}>
            {statusText}
          </Typography>
          <Button
            size="small"
            aria-label="로그 다시 불러오기"
            onClick={() => { void logsQuery.refetch() }}
            disabled={logsQuery.isFetching}
            startIcon={logsQuery.isFetching ? <CircularProgress size={12} aria-hidden="true" /> : undefined}
          >
            다시 불러오기
          </Button>
        </Stack>
        <Typography data-testid="log-checked-at" variant="caption" color="text.secondary" sx={{ display: 'block', mt: 0.5 }}>
          마지막 정상 확인: {checkedAt ? <time dateTime={checkedAt.toISOString()}>{checkedAt.toLocaleString('ko-KR')}</time> : '없음'}
        </Typography>
      </Box>

      {/* 필터 */}
      <Stack direction="row" spacing={1} sx={{ mb: 2, alignItems: "center", flexWrap: "wrap", gap: 1 }} useFlexGap>
        <ToggleButtonGroup
          value={level}
          exclusive
          onChange={(_, v) => v && setLevel(v)}
          size="small"
        >
          {(['ALL', 'DEBUG', 'INFO', 'WARN', 'ERROR'] as LogLevel[]).map((l) => (
            <ToggleButton key={l} value={l} sx={{ minWidth: { xs: 42, sm: 56 }, px: { xs: 0.5, sm: 1 }, fontSize: { xs: '0.68rem', sm: '0.8125rem' } }}>
              {l}
            </ToggleButton>
          ))}
        </ToggleButtonGroup>
        <Button size="small" onClick={() => { setLevel('ALL'); setSearch('') }}>필터 지우기</Button>
        <TextField
          label="로그 검색"
          placeholder="검색..."
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          size="small"
          sx={{ flex: 1, minWidth: 100, maxWidth: { xs: '100%', sm: 200 } }}
        />
        <Chip data-testid="log-result-count" label={hasConfirmedLogs ? `표시 ${filtered.length}건 / 조회 ${logs.length}건` : '조회 미확인'} size="small" />
      </Stack>

      {/* 로그 뷰어 */}
      <Paper
        data-testid="log-resizable-panel"
        ref={logPanelRef}
        onScroll={handleLogScroll}
        sx={{
          p: 1,
          overflow: 'auto',
          bgcolor: 'background.paper',
          fontFamily: 'monospace',
          fontSize: { xs: '0.7rem', sm: '0.78rem' },
          // 모바일: flex로 가용 높이 채움 / 데스크탑: 드래그 조절 가능한 고정 높이
          ...(isMobile
            ? { flex: 1, minHeight: 200 }
            : { height: logHeight, minHeight: 160 }
          ),
        }}
      >
        {!hasConfirmedLogs && (
          <Typography variant="body2" color="text.secondary" sx={{ p: 1 }}>
            {hasReadError ? '로그를 불러오지 못했습니다. 다시 불러오기를 눌러 확인하세요.' : '조회가 완료되면 로그를 표시합니다.'}
          </Typography>
        )}
        {hasConfirmedLogs && logs.length === 0 && (
          <Typography variant="body2" color="text.secondary" sx={{ p: 1 }}>조회한 최근 로그가 없습니다.</Typography>
        )}
        {hasConfirmedLogs && logs.length > 0 && filtered.length === 0 && (
          <Typography variant="body2" color="text.secondary" sx={{ p: 1 }}>현재 필터와 일치하는 로그가 없습니다.</Typography>
        )}
        {filtered.map((log: AppLogEntry, i: number) => (
          <Box
            key={i}
            sx={{
              display: 'flex',
              gap: 1,
              py: 0.25,
              borderBottom: '1px solid',
              borderBottomColor: 'divider',
              lineHeight: 1.4,
            }}
          >
            {/* 타임스탬프: 모바일은 시:분:초만 표시 */}
            <Box sx={{ color: 'text.secondary', whiteSpace: 'nowrap', minWidth: { xs: 64, sm: 90 }, flexShrink: 0 }}>
              {isMobile
                ? log.timestamp.slice(11, 19)
                : (log.timestamp.length > 23 ? log.timestamp.slice(11, 23) : log.timestamp)}
            </Box>
            <Box
              sx={{
                color: LEVEL_COLORS[log.level] ?? '#888',
                fontWeight: 700,
                minWidth: { xs: 38, sm: 46 },
                whiteSpace: 'nowrap',
                flexShrink: 0,
              }}
            >
              {isMobile ? log.level.slice(0, 4) : log.level}
            </Box>
            {/* target: 모바일에서 숨김 */}
            {!isMobile && (
              <Box sx={{ color: 'text.secondary', minWidth: 140, whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis', flexShrink: 0 }}>
                {log.target}
              </Box>
            )}
            <Box sx={{ flex: 1, minWidth: 0 }}>
              <Box sx={{ wordBreak: 'break-all' }}>{log.message}</Box>
              {hasProviderTrace(parseProviderTraceText(log.message)) && (
                <Box sx={{ mt: 0.5 }}>
                  <ProviderTraceChips trace={parseProviderTraceText(log.message)} />
                </Box>
              )}
            </Box>
          </Box>
        ))}
        <div ref={bottomRef} />
      </Paper>

      {/* 높이 조절 핸들 — 데스크탑 전용 */}
      {!isMobile && (
        <LayoutResizer
          direction="vertical"
          label="로그 패널 높이 조절"
          value={logHeight}
          min={LOG_HEIGHT_MIN}
          max={LOG_HEIGHT_MAX}
          defaultValue={LOG_HEIGHT_DEFAULT}
          onValueChange={handleLogValueChange}
          onResize={handleLogResize}
          onResizeEnd={handleLogResizeEnd}
        />
      )}
    </Box>
  )
}
