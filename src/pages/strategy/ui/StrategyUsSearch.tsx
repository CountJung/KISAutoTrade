import type { KeyboardEvent as ReactKeyboardEvent } from 'react'
import Alert from '@mui/material/Alert'
import Button from '@mui/material/Button'
import CircularProgress from '@mui/material/CircularProgress'
import Stack from '@mui/material/Stack'
import TextField from '@mui/material/TextField'
import Typography from '@mui/material/Typography'
import SearchIcon from '@mui/icons-material/Search'
import { isSearchEnter } from '../../../shared/lib'
import type { StockSearchItem } from '../../../api/types'

// Page-owned input, selection and lookup callbacks; this component only renders the US search.
export function StrategyUsSearch({ value, selection, searching, error, onChange, onSearch }: {
  value: string
  selection: StockSearchItem | null
  searching: boolean
  error: string | null
  onChange: (value: string) => void
  onSearch: () => Promise<void>
}) {
  return (
    <Stack spacing={1}>
      <Stack direction="row" spacing={1}>
        <TextField label="티커 (예: AAPL, SPYM, QQQ)" value={value}
          onChange={event => onChange(event.target.value)} size="small" fullWidth
          slotProps={{ htmlInput: {
            style: { textTransform: 'uppercase' },
            onKeyDown: (event: ReactKeyboardEvent<HTMLInputElement>) => {
              if (event.key !== 'Enter') return
              event.stopPropagation()
              if (!isSearchEnter(event)) return
              event.preventDefault()
              void onSearch()
            },
          } }}
          helperText={selection ? `선택됨: ${selection.prdt_name} (${selection.pdno})` : 'NAS(NASDAQ)·NYS·AMEX 순서로 자동 감지합니다'} />
        <Button variant="contained" size="small" aria-label="미국 종목 검색"
          onClick={onSearch} disabled={!value.trim()}
          aria-disabled={!value.trim() || searching}
          aria-busy={searching} aria-description={searching ? '미국 종목 조회 중' : undefined}
          startIcon={searching ? <CircularProgress size={14} aria-label="미국 종목 조회 중" /> : <SearchIcon />}
          sx={{ minWidth: 80, alignSelf: 'flex-start', mt: 0.5 }}>검색</Button>
      </Stack>
      {error && <Alert severity="warning" sx={{ py: 0.5 }}><Typography variant="caption">{error}</Typography></Alert>}
    </Stack>
  )
}
