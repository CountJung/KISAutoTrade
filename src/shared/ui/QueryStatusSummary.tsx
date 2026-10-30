import Box from '@mui/material/Box'
import Button from '@mui/material/Button'
import Chip from '@mui/material/Chip'
import CircularProgress from '@mui/material/CircularProgress'
import Typography from '@mui/material/Typography'
import CheckCircleOutlinedIcon from '@mui/icons-material/CheckCircleOutlined'
import WarningAmberIcon from '@mui/icons-material/WarningAmber'
import RefreshIcon from '@mui/icons-material/Refresh'

interface QueryStatusSummaryProps {
  label: string
  statusText?: string
  statusColor?: 'default' | 'success'
  isError: boolean
  isFetching: boolean
  dataUpdatedAt: number
  onRetry: () => void
  testId: string
}

/** 조회 결과만 표시하며 조회·캐시·동작 정책은 호출부에 둔다. */
export function QueryStatusSummary({
  label,
  statusText,
  statusColor = 'default',
  isError,
  isFetching,
  dataUpdatedAt,
  onRetry,
  testId,
}: QueryStatusSummaryProps) {
  const hasData = statusText !== undefined
  const text = hasData
    ? isError ? `이전 상태: ${statusText}` : statusText
    : isError ? `${label} 확인 실패` : `${label} 확인 중`
  const color = isError ? 'warning' : hasData ? statusColor : 'default'
  const checkedAt = hasData && dataUpdatedAt > 0 ? new Date(dataUpdatedAt) : undefined

  return (
    <Box data-testid={testId} sx={{ minWidth: 0 }}>
      <Box role="status" aria-live="polite" aria-atomic="true">
        <Chip
          data-testid={`${testId}-label`}
          label={text}
          color={color}
          size="small"
          variant={color === 'success' ? 'filled' : 'outlined'}
          icon={isError ? <WarningAmberIcon /> : hasData ? <CheckCircleOutlinedIcon /> : <CircularProgress size={12} color="inherit" />}
          sx={{
            maxWidth: '100%',
            height: 'auto',
            minHeight: 24,
            '& .MuiChip-label': { whiteSpace: 'normal', overflowWrap: 'anywhere', py: 0.25 },
          }}
        />
        {hasData && isError && (
          <Typography variant="caption" color="warning.main" sx={{ display: 'block', mt: 0.5 }}>
            상태 재조회 실패 · 이전 상태 표시
          </Typography>
        )}
        {hasData && isFetching && (
          <Typography variant="caption" color="text.secondary" sx={{ display: 'block', mt: 0.5 }}>
            상태 재확인 중
          </Typography>
        )}
      </Box>
      <Typography
        data-testid={`${testId}-checked-at`}
        variant="caption"
        color="text.secondary"
        sx={{ display: 'block', mt: 0.5, overflowWrap: 'anywhere' }}
      >
        마지막 정상 확인:{' '}
        {checkedAt ? <time dateTime={checkedAt.toISOString()}>{checkedAt.toLocaleString('ko-KR')}</time> : '없음'}
      </Typography>
      <Button
        size="small"
        startIcon={isFetching ? <CircularProgress size={12} color="inherit" /> : <RefreshIcon />}
        aria-label={`${label} 다시 확인`}
        onClick={onRetry}
        disabled={isFetching}
        sx={{ mt: 0.25, px: 0.5, minWidth: 0 }}
      >
        {isFetching ? '확인 중' : '다시 확인'}
      </Button>
    </Box>
  )
}
