import { useEffect, useRef, useState, useCallback } from 'react'
import Typography from '@mui/material/Typography'
import Paper from '@mui/material/Paper'
import Box from '@mui/material/Box'
import Stack from '@mui/material/Stack'
import ToggleButton from '@mui/material/ToggleButton'
import ToggleButtonGroup from '@mui/material/ToggleButtonGroup'
import TextField from '@mui/material/TextField'
import Chip from '@mui/material/Chip'
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

  const { data: logs = [], isLoading } = useRecentLogs(300)

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

  return (
    <Box sx={{ display: 'flex', flexDirection: 'column', height: '100%' }}>
      <Typography variant="h5" fontWeight={700} mb={2}>
        Log
      </Typography>

      {/* 필터 */}
      <Stack direction="row" spacing={1} mb={2} alignItems="center" flexWrap="wrap" gap={1} useFlexGap>
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
        <TextField
          placeholder="검색..."
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          size="small"
          sx={{ flex: 1, minWidth: 100, maxWidth: { xs: '100%', sm: 200 } }}
        />
        <Chip label={`${filtered.length}건`} size="small" />
        {isLoading && <CircularProgress size={16} />}
      </Stack>

      {/* 로그 뷰어 */}
      <Paper
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
          onResize={handleLogResize}
          onResizeEnd={handleLogResizeEnd}
        />
      )}
    </Box>
  )
}
