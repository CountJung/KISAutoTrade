import { useState, type MouseEvent } from 'react'

import Box from '@mui/material/Box'
import Button from '@mui/material/Button'
import Chip from '@mui/material/Chip'
import CircularProgress from '@mui/material/CircularProgress'
import FormControl from '@mui/material/FormControl'
import IconButton from '@mui/material/IconButton'
import InputLabel from '@mui/material/InputLabel'
import MenuItem from '@mui/material/MenuItem'
import Select from '@mui/material/Select'
import Stack from '@mui/material/Stack'
import Table from '@mui/material/Table'
import TableBody from '@mui/material/TableBody'
import TableCell from '@mui/material/TableCell'
import TableContainer from '@mui/material/TableContainer'
import TableHead from '@mui/material/TableHead'
import TableRow from '@mui/material/TableRow'
import TableSortLabel from '@mui/material/TableSortLabel'
import TextField from '@mui/material/TextField'
import ToggleButton from '@mui/material/ToggleButton'
import ToggleButtonGroup from '@mui/material/ToggleButtonGroup'
import Tooltip from '@mui/material/Tooltip'
import Typography from '@mui/material/Typography'
import RefreshIcon from '@mui/icons-material/Refresh'

import { usePendingOrders, useTradesByRange } from '../../../api/hooks'
import { fmtNumber } from '../../../shared/lib'

function fmt(n: number) {
  return fmtNumber(n)
}

function todayStr() {
  return new Date().toISOString().slice(0, 10)
}

type TradeSort = 'timestamp' | 'symbol_name' | 'side' | 'quantity' | 'price' | 'total_amount'
type SortDir = 'asc' | 'desc'

export function FilledOrdersPanel() {
  const [from, setFrom] = useState(todayStr)
  const [to, setTo] = useState(todayStr)
  const [queryFrom, setQueryFrom] = useState(todayStr)
  const [queryTo, setQueryTo] = useState(todayStr)
  const [page, setPage] = useState(0)
  const [pageSize, setPageSize] = useState<25 | 50 | 100>(25)

  const [sideFilter, setSideFilter] = useState<'all' | 'buy' | 'sell'>('all')
  const [symbolSearch, setSymbolSearch] = useState('')

  const [sortBy, setSortBy] = useState<TradeSort>('timestamp')
  const [sortDir, setSortDir] = useState<SortDir>('desc')

  const { data: trades = [], isLoading, dataUpdatedAt, refetch, isFetching } =
    useTradesByRange(queryFrom, queryTo)

  const todayIso = todayStr()
  const isQueryToday = queryTo === todayIso

  const filtered = trades.filter((t) => {
    if (sideFilter !== 'all' && t.side !== sideFilter) return false
    if (symbolSearch) {
      const q = symbolSearch.toLowerCase()
      if (!t.symbol.toLowerCase().includes(q) && !t.symbol_name.toLowerCase().includes(q)) {
        return false
      }
    }
    return true
  })

  const sorted = [...filtered].sort((a, b) => {
    let cmp = 0
    switch (sortBy) {
      case 'timestamp':
        cmp = a.timestamp.localeCompare(b.timestamp)
        break
      case 'symbol_name':
        cmp = a.symbol_name.localeCompare(b.symbol_name)
        break
      case 'side':
        cmp = a.side.localeCompare(b.side)
        break
      case 'quantity':
        cmp = a.quantity - b.quantity
        break
      case 'price':
        cmp = a.price - b.price
        break
      case 'total_amount':
        cmp = a.total_amount - b.total_amount
        break
    }
    return sortDir === 'asc' ? cmp : -cmp
  })

  const totalPages = Math.max(1, Math.ceil(sorted.length / pageSize))
  const pagedTrades = sorted.slice(page * pageSize, (page + 1) * pageSize)

  const handleQuery = () => {
    setQueryFrom(from)
    setQueryTo(to)
    setPage(0)
  }

  const handlePageSizeChange = (newSize: 25 | 50 | 100) => {
    setPageSize(newSize)
    setPage(0)
  }

  const handleSort = (col: TradeSort) => {
    if (sortBy === col) {
      setSortDir((d) => (d === 'asc' ? 'desc' : 'asc'))
    } else {
      setSortBy(col)
      setSortDir('desc')
    }
    setPage(0)
  }

  const handleSideFilter = (_: MouseEvent, v: 'all' | 'buy' | 'sell' | null) => {
    setSideFilter(v ?? 'all')
    setPage(0)
  }

  const lastUpdated = dataUpdatedAt
    ? new Date(dataUpdatedAt).toLocaleTimeString('ko-KR', {
        hour: '2-digit',
        minute: '2-digit',
        second: '2-digit',
      })
    : null

  const SortCell = ({
    col,
    label,
    align,
    sx,
  }: {
    col: TradeSort
    label: string
    align?: 'right' | 'left' | 'center'
    sx?: object
  }) => (
    <TableCell align={align} sx={sx}>
      <TableSortLabel
        active={sortBy === col}
        direction={sortBy === col ? sortDir : 'desc'}
        onClick={() => handleSort(col)}
      >
        {label}
      </TableSortLabel>
    </TableCell>
  )

  return (
    <Box>
      <Stack direction="row" spacing={1} alignItems="center" mb={1} flexWrap="wrap" useFlexGap>
        <TextField
          type="date"
          label="시작일"
          value={from}
          onChange={(e) => setFrom(e.target.value)}
          size="small"
          slotProps={{ inputLabel: { shrink: true } }}
          sx={{ width: 150 }}
        />
        <TextField
          type="date"
          label="종료일"
          value={to}
          onChange={(e) => setTo(e.target.value)}
          size="small"
          slotProps={{ inputLabel: { shrink: true } }}
          sx={{ width: 150 }}
        />
        <Button variant="outlined" size="small" onClick={handleQuery}>
          조회
        </Button>
        <Box sx={{ ml: 'auto', display: 'flex', alignItems: 'center', gap: 1 }}>
          {lastUpdated && (
            <Typography variant="caption" color="text.secondary" sx={{ whiteSpace: 'nowrap' }}>
              {isQueryToday ? '🔄 ' : ''}마지막 갱신 {lastUpdated}
            </Typography>
          )}
          <Tooltip title="수동 새로고침">
            <span>
              <IconButton size="small" onClick={() => void refetch()} disabled={isFetching}>
                {isFetching ? <CircularProgress size={14} /> : <RefreshIcon fontSize="small" />}
              </IconButton>
            </span>
          </Tooltip>
        </Box>
      </Stack>

      {!isLoading && trades.length > 0 && (
        <Stack direction="row" spacing={1} alignItems="center" mb={1.5} flexWrap="wrap" useFlexGap>
          <ToggleButtonGroup
            value={sideFilter}
            exclusive
            onChange={handleSideFilter}
            size="small"
          >
            <ToggleButton value="all" sx={{ px: 1.5, fontSize: '0.75rem' }}>
              전체
            </ToggleButton>
            <ToggleButton
              value="buy"
              sx={{ px: 1.5, fontSize: '0.75rem', '&.Mui-selected': { color: 'primary.main' } }}
            >
              매수
            </ToggleButton>
            <ToggleButton
              value="sell"
              sx={{ px: 1.5, fontSize: '0.75rem', '&.Mui-selected': { color: 'error.main' } }}
            >
              매도
            </ToggleButton>
          </ToggleButtonGroup>

          <TextField
            placeholder="종목코드·종목명 검색"
            value={symbolSearch}
            onChange={(e) => {
              setSymbolSearch(e.target.value)
              setPage(0)
            }}
            size="small"
            sx={{ width: { xs: '100%', sm: 200 } }}
          />

          <Typography variant="caption" color="text.secondary" sx={{ whiteSpace: 'nowrap' }}>
            {filtered.length !== trades.length
              ? `${filtered.length} / ${trades.length}건 표시`
              : `총 ${trades.length}건`}
          </Typography>

          {(sideFilter !== 'all' || symbolSearch) && (
            <Button
              size="small"
              variant="text"
              onClick={() => {
                setSideFilter('all')
                setSymbolSearch('')
                setPage(0)
              }}
              sx={{ fontSize: '0.72rem', px: 1 }}
            >
              필터 초기화
            </Button>
          )}
        </Stack>
      )}

      {isLoading ? (
        <Box sx={{ py: 2, display: 'flex', justifyContent: 'center' }}>
          <CircularProgress size={20} />
        </Box>
      ) : trades.length === 0 ? (
        <Typography variant="body2" color="text.secondary">
          해당 기간에 체결 내역이 없습니다.
        </Typography>
      ) : sorted.length === 0 ? (
        <Typography variant="body2" color="text.secondary">
          필터 조건에 맞는 체결 내역이 없습니다.
        </Typography>
      ) : (
        <>
          <TableContainer sx={{ maxHeight: 400, overflowX: 'auto' }}>
            <Table size="small" stickyHeader>
              <TableHead>
                <TableRow>
                  <SortCell col="symbol_name" label="종목" />
                  <SortCell col="side" label="구분" />
                  <SortCell col="quantity" label="수량" align="right" />
                  <SortCell
                    col="price"
                    label="단가"
                    align="right"
                    sx={{ display: { xs: 'none', sm: 'table-cell' } }}
                  />
                  <SortCell
                    col="total_amount"
                    label="금액"
                    align="right"
                    sx={{ display: { xs: 'none', md: 'table-cell' } }}
                  />
                  <SortCell
                    col="timestamp"
                    label="일시"
                    sx={{ display: { xs: 'none', sm: 'table-cell' } }}
                  />
                  <TableCell sx={{ display: { xs: 'none', md: 'table-cell' } }}>체결사유</TableCell>
                </TableRow>
              </TableHead>
              <TableBody>
                {pagedTrades.map((t) => (
                  <TableRow key={t.id} hover>
                    <TableCell>
                      <Typography variant="body2" component="span" fontWeight={500}>
                        {t.symbol_name}
                      </Typography>
                      <Typography variant="caption" color="text.secondary" component="span" sx={{ ml: 0.5 }}>
                        {t.symbol}
                      </Typography>
                    </TableCell>
                    <TableCell>
                      <Chip
                        label={t.side === 'buy' ? '매수' : '매도'}
                        color={t.side === 'buy' ? 'primary' : 'error'}
                        size="small"
                      />
                    </TableCell>
                    <TableCell align="right">{fmt(t.quantity)}</TableCell>
                    <TableCell align="right" sx={{ display: { xs: 'none', sm: 'table-cell' } }}>
                      {fmt(t.price)}원
                    </TableCell>
                    <TableCell align="right" sx={{ display: { xs: 'none', md: 'table-cell' } }}>
                      {fmt(t.total_amount)}원
                    </TableCell>
                    <TableCell sx={{ display: { xs: 'none', sm: 'table-cell' }, whiteSpace: 'nowrap' }}>
                      {t.timestamp.slice(0, 16).replace('T', ' ')}
                    </TableCell>
                    <TableCell sx={{ display: { xs: 'none', md: 'table-cell' } }}>
                      <Typography
                        variant="caption"
                        color="text.secondary"
                        title={t.signal_reason || undefined}
                        sx={{
                          maxWidth: 160,
                          display: 'block',
                          overflow: 'hidden',
                          textOverflow: 'ellipsis',
                          whiteSpace: 'nowrap',
                        }}
                      >
                        {t.signal_reason || '—'}
                      </Typography>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </TableContainer>

          <Stack direction="row" justifyContent="space-between" alignItems="center" mt={1} flexWrap="wrap" gap={1}>
            <Stack direction="row" spacing={0.5} alignItems="center">
              <Button size="small" variant="outlined" disabled={page === 0} onClick={() => setPage(0)} sx={{ minWidth: 0, px: 1 }}>
                «
              </Button>
              <Button
                size="small"
                variant="outlined"
                disabled={page === 0}
                onClick={() => setPage((p) => Math.max(0, p - 1))}
                sx={{ minWidth: 0, px: 1 }}
              >
                ‹
              </Button>
              <Typography variant="caption" sx={{ px: 1, whiteSpace: 'nowrap' }}>
                {page + 1} / {totalPages} 페이지 · {sorted.length}건
              </Typography>
              <Button
                size="small"
                variant="outlined"
                disabled={page >= totalPages - 1}
                onClick={() => setPage((p) => Math.min(totalPages - 1, p + 1))}
                sx={{ minWidth: 0, px: 1 }}
              >
                ›
              </Button>
              <Button
                size="small"
                variant="outlined"
                disabled={page >= totalPages - 1}
                onClick={() => setPage(totalPages - 1)}
                sx={{ minWidth: 0, px: 1 }}
              >
                »
              </Button>
            </Stack>
            <FormControl size="small" sx={{ minWidth: 90 }}>
              <InputLabel id="dashboard-page-size-label">표시 건수</InputLabel>
              <Select<25 | 50 | 100>
                labelId="dashboard-page-size-label"
                label="표시 건수"
                value={pageSize}
                onChange={(e) => handlePageSizeChange(e.target.value as 25 | 50 | 100)}
              >
                <MenuItem value={25}>25건</MenuItem>
                <MenuItem value={50}>50건</MenuItem>
                <MenuItem value={100}>100건</MenuItem>
              </Select>
            </FormControl>
          </Stack>
        </>
      )}
    </Box>
  )
}

export function PendingOrdersPanel() {
  const { data: orders = [], isLoading } = usePendingOrders()

  if (isLoading) {
    return (
      <Box sx={{ py: 2, display: 'flex', justifyContent: 'center' }}>
        <CircularProgress size={20} />
      </Box>
    )
  }

  if (orders.length === 0) {
    return (
      <Typography variant="body2" color="text.secondary" sx={{ py: 1 }}>
        미체결 주문이 없습니다.
      </Typography>
    )
  }

  return (
    <TableContainer sx={{ maxHeight: 260 }}>
      <Table size="small" stickyHeader>
        <TableHead>
          <TableRow>
            <TableCell>종목</TableCell>
            <TableCell>구분</TableCell>
            <TableCell>상태</TableCell>
            <TableCell align="right">수량</TableCell>
            <TableCell sx={{ display: { xs: 'none', sm: 'table-cell' } }}>주문번호</TableCell>
            <TableCell sx={{ display: { xs: 'none', md: 'table-cell' } }}>신호 이유</TableCell>
          </TableRow>
        </TableHead>
        <TableBody>
          {orders.map((o) => {
            const isPartial = o.status === 'partially_filled'
            const statusLabel = isPartial ? '부분체결' : o.status === 'failed' ? '실패' : '미체결'
            const quantityLabel = isPartial ? `${fmt(o.filledQuantity)} / ${fmt(o.quantity)}` : fmt(o.quantity)

            return (
              <TableRow key={o.odno || o.symbol + o.timestamp}>
                <TableCell>
                  <Typography variant="body2" noWrap>
                    {o.symbolName}
                  </Typography>
                  <Typography variant="caption" color="text.secondary">
                    {o.symbol}
                  </Typography>
                  <Typography variant="caption" color="text.secondary" display="block" noWrap>
                    {o.brokerId.toUpperCase()}{o.brokerAccountId ? ` · ${o.brokerAccountId}` : ''}
                  </Typography>
                </TableCell>
                <TableCell>
                  <Chip
                    label={o.side === 'buy' ? '매수' : '매도'}
                    color={o.side === 'buy' ? 'primary' : 'error'}
                    size="small"
                  />
                </TableCell>
                <TableCell>
                  <Chip
                    label={statusLabel}
                    color={isPartial ? 'warning' : o.status === 'failed' ? 'error' : 'default'}
                    size="small"
                    variant={isPartial ? 'filled' : 'outlined'}
                  />
                </TableCell>
                <TableCell align="right">
                  <Typography variant="body2" noWrap>
                    {quantityLabel}
                  </Typography>
                  {isPartial && (
                    <Typography variant="caption" color="text.secondary" noWrap>
                      잔여 {fmt(o.remainingQuantity)}
                    </Typography>
                  )}
                </TableCell>
                <TableCell sx={{ display: { xs: 'none', sm: 'table-cell' } }}>
                  <Typography variant="caption" color="text.secondary" noWrap>
                    {o.odno || '-'}
                  </Typography>
                </TableCell>
                <TableCell sx={{ display: { xs: 'none', md: 'table-cell' } }}>
                  <Typography variant="caption" color="text.secondary" noWrap>
                    {o.signalReason || '-'}
                  </Typography>
                </TableCell>
              </TableRow>
            )
          })}
        </TableBody>
      </Table>
    </TableContainer>
  )
}
