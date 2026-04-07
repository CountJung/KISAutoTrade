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

import { useRecentLogs } from '../api/hooks'
import type { AppLogEntry } from '../api/types'

type LogLevel = 'ALL' | 'DEBUG' | 'INFO' | 'WARN' | 'ERROR' | 'TRACE'

const LEVEL_COLORS: Record<string, string> = {
  ALL:   '#888',
  DEBUG: '#888',
  TRACE: '#888',
  INFO:  '#2196f3',
  WARN:  '#ff9800',
  ERROR: '#f44336',
}

export default function Log() {
  const [level, setLevel]   = useState<LogLevel>('ALL')
  const [search, setSearch] = useState('')
  const [logHeight, setLogHeight] = useState(480)
  const bottomRef = useRef<HTMLDivElement>(null)
  const dragStartRef = useRef<{ y: number; h: number } | null>(null)

  const handleDragStart = useCallback((e: React.MouseEvent) => {
    dragStartRef.current = { y: e.clientY, h: logHeight }
    const onMove = (mv: MouseEvent) => {
      if (!dragStartRef.current) return
      const delta = mv.clientY - dragStartRef.current.y
      setLogHeight(Math.max(160, dragStartRef.current.h + delta))
    }
    const onUp = () => {
      dragStartRef.current = null
      window.removeEventListener('mousemove', onMove)
      window.removeEventListener('mouseup', onUp)
    }
    window.addEventListener('mousemove', onMove)
    window.addEventListener('mouseup', onUp)
  }, [logHeight])

  const { data: logs = [], isLoading } = useRecentLogs(300)

  // 새 로그가 올 때 스크롤 하단 이동
  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: 'smooth' })
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
      <Stack direction="row" spacing={2} mb={2} alignItems="center" flexWrap="wrap">
        <ToggleButtonGroup
          value={level}
          exclusive
          onChange={(_, v) => v && setLevel(v)}
          size="small"
        >
          {(['ALL', 'DEBUG', 'INFO', 'WARN', 'ERROR'] as LogLevel[]).map((l) => (
            <ToggleButton key={l} value={l} sx={{ minWidth: 64 }}>
              {l}
            </ToggleButton>
          ))}
        </ToggleButtonGroup>
        <TextField
          placeholder="검색..."
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          size="small"
          sx={{ width: 200 }}
        />
        <Chip label={`${filtered.length}건`} size="small" />
        {isLoading && <CircularProgress size={16} />}
      </Stack>

      {/* 로그 뷰어 */}
      <Paper
        sx={{
          flex: 1,
          p: 1.5,
          overflow: 'auto',
          bgcolor: 'background.paper',
          fontFamily: 'monospace',
          fontSize: '0.78rem',
          height: logHeight,
          minHeight: 160,
        }}
      >
        {filtered.map((log: AppLogEntry, i: number) => (
          <Box
            key={i}
            sx={{
              display: 'flex',
              gap: 1.5,
              py: 0.2,
              borderBottom: '1px solid',
              borderBottomColor: 'divider',
            }}
          >
            <Box sx={{ color: 'text.secondary', whiteSpace: 'nowrap', minWidth: 90 }}>
              {log.timestamp.length > 23 ? log.timestamp.slice(11, 23) : log.timestamp}
            </Box>
            <Box
              sx={{
                color: LEVEL_COLORS[log.level] ?? '#888',
                fontWeight: 700,
                minWidth: 46,
                whiteSpace: 'nowrap',
              }}
            >
              {log.level}
            </Box>
            <Box sx={{ color: 'text.secondary', minWidth: 160, whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis' }}>
              {log.target}
            </Box>
            <Box sx={{ flex: 1 }}>{log.message}</Box>
          </Box>
        ))}
        <div ref={bottomRef} />
      </Paper>

      {/* 높이 조절 핸들 */}
      <Box
        onMouseDown={handleDragStart}
        sx={{
          height: 6,
          cursor: 'ns-resize',
          bgcolor: 'divider',
          borderRadius: 1,
          mt: 0.5,
          '&:hover': { bgcolor: 'action.selected' },
        }}
      />
    </Box>
  )
}

