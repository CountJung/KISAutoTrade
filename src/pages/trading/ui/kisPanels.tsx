import Typography from '@mui/material/Typography'
import Box from '@mui/material/Box'
import Stack from '@mui/material/Stack'
import Table from '@mui/material/Table'
import TableBody from '@mui/material/TableBody'
import TableCell from '@mui/material/TableCell'
import TableContainer from '@mui/material/TableContainer'
import TableHead from '@mui/material/TableHead'
import TableRow from '@mui/material/TableRow'
import TrendingUpIcon from '@mui/icons-material/TrendingUp'
import TrendingDownIcon from '@mui/icons-material/TrendingDown'

import { useOverseasPrice, usePrice } from '../../../api/hooks'
import type { BalanceItem, OverseasBalanceItem, OverseasExchange } from '../../../api/types'
import { fmtNumber } from '../../../shared/lib'

function fmt(n: number) {
  return fmtNumber(n)
}

interface HoldingsTableProps {
  items: BalanceItem[]
  onSelect: (item: BalanceItem) => void
}

export function HoldingsTable({ items, onSelect }: HoldingsTableProps) {
  if (items.length === 0) {
    return (
      <Typography variant="body2" color="text.secondary" sx={{ py: 1 }}>
        보유 종목 없음
      </Typography>
    )
  }
  return (
    <TableContainer sx={{ maxHeight: { xs: 200, sm: 260 }, overflowX: 'auto' }}>
      <Table size="small" stickyHeader>
        <TableHead>
          <TableRow>
            <TableCell sx={{ minWidth: 120 }}>종목</TableCell>
            <TableCell align="right" sx={{ minWidth: 70 }}>수량</TableCell>
            <TableCell align="right" sx={{ minWidth: 90 }}>현재가</TableCell>
            <TableCell align="right" sx={{ minWidth: 90 }}>손익률</TableCell>
          </TableRow>
        </TableHead>
        <TableBody>
          {items.map((item) => {
            const pfls = parseFloat(item.evlu_pfls_rt)
            const positive = pfls >= 0
            return (
              <TableRow
                key={item.pdno}
                hover
                onClick={() => onSelect(item)}
                sx={{ cursor: 'pointer' }}
              >
                <TableCell>
                  <Typography variant="body2" noWrap sx={{ maxWidth: { xs: 130, sm: 'none' } }}>
                    {item.prdt_name}
                  </Typography>
                  <Typography variant="caption" color="text.secondary">{item.pdno}</Typography>
                </TableCell>
                <TableCell align="right">{fmt(parseInt(item.hldg_qty))}주</TableCell>
                <TableCell align="right" sx={{ whiteSpace: 'nowrap' }}>
                  {fmt(parseInt(item.prpr))}원
                </TableCell>
                <TableCell align="right">
                  <Typography
                    variant="body2"
                    color={positive ? 'success.main' : 'error.main'}
                    fontWeight={600}
                    noWrap
                  >
                    {positive ? '+' : ''}{pfls.toFixed(2)}%
                  </Typography>
                  <Typography
                    variant="caption"
                    color={positive ? 'success.main' : 'error.main'}
                    noWrap
                  >
                    {positive ? '+' : ''}{fmt(parseInt(item.evlu_pfls_amt))}원
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

interface OverseasHoldingsTableProps {
  items: OverseasBalanceItem[]
  onSelect: (item: OverseasBalanceItem) => void
}

export function OverseasHoldingsTable({ items, onSelect }: OverseasHoldingsTableProps) {
  if (items.length === 0) {
    return (
      <Typography variant="body2" color="text.secondary" sx={{ py: 1 }}>
        해외 보유 종목 없음
      </Typography>
    )
  }
  return (
    <TableContainer sx={{ maxHeight: { xs: 200, sm: 260 }, overflowX: 'auto' }}>
      <Table size="small" stickyHeader>
        <TableHead>
          <TableRow>
            <TableCell sx={{ minWidth: 120 }}>종목</TableCell>
            <TableCell align="right" sx={{ minWidth: 60 }}>수량</TableCell>
            <TableCell align="right" sx={{ minWidth: 90 }}>현재가(USD)</TableCell>
            <TableCell align="right" sx={{ minWidth: 90 }}>손익률</TableCell>
          </TableRow>
        </TableHead>
        <TableBody>
          {items.map((item) => {
            const pfls = parseFloat(item.evlu_pfls_rt)
            const positive = pfls >= 0
            return (
              <TableRow
                key={item.ovrs_pdno}
                hover
                onClick={() => onSelect(item)}
                sx={{ cursor: 'pointer' }}
              >
                <TableCell>
                  <Typography variant="body2" noWrap sx={{ maxWidth: { xs: 130, sm: 'none' } }}>
                    {item.ovrs_item_name}
                  </Typography>
                  <Typography variant="caption" color="text.secondary">{item.ovrs_pdno}</Typography>
                </TableCell>
                <TableCell align="right">{fmt(parseInt(item.ovrs_cblc_qty))}</TableCell>
                <TableCell align="right" sx={{ whiteSpace: 'nowrap' }}>
                  ${parseFloat(item.now_pric2).toFixed(2)}
                </TableCell>
                <TableCell align="right">
                  <Typography
                    variant="body2"
                    color={positive ? 'success.main' : 'error.main'}
                    fontWeight={600}
                    noWrap
                  >
                    {positive ? '+' : ''}{pfls.toFixed(2)}%
                  </Typography>
                  <Typography
                    variant="caption"
                    color={positive ? 'success.main' : 'error.main'}
                    noWrap
                  >
                    {positive ? '+' : ''}${parseFloat(item.frcr_evlu_pfls_amt).toFixed(2)}
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

export function KrStockInfoCard({ symbol }: { symbol: string }) {
  const { data: p, isLoading } = usePrice(symbol.length === 6 ? symbol : '')
  if (!p && !isLoading) return null
  if (isLoading) return (
    <Box sx={{ p: 2, display: 'flex', alignItems: 'center', gap: 1 }}>
      <Typography variant="body2" color="text.secondary">조회 중</Typography>
    </Box>
  )
  if (!p) return null

  const price = parseInt(p.stck_prpr)
  const change = parseInt(p.prdy_vrss)
  const rt = parseFloat(p.prdy_ctrt)
  const pos = rt >= 0

  return (
    <Box sx={{ px: { xs: 1.5, sm: 2.5 }, py: 1.5 }}>
      <Stack direction="row" spacing={1.5} alignItems="baseline" flexWrap="wrap" mb={1}>
        <Box>
          <Typography variant="subtitle2" fontWeight={700}>{p.hts_kor_isnm}</Typography>
          <Typography variant="caption" color="text.secondary">{symbol}</Typography>
        </Box>
        <Typography variant="h5" fontWeight={700}>{fmt(price)}원</Typography>
        <Stack direction="row" alignItems="center" spacing={0.5}>
          {pos
            ? <TrendingUpIcon fontSize="small" color="success" />
            : <TrendingDownIcon fontSize="small" color="error" />}
          <Typography variant="body2" color={pos ? 'success.main' : 'error.main'} fontWeight={600}>
            {pos ? '+' : ''}{fmt(change)} ({pos ? '+' : ''}{rt.toFixed(2)}%)
          </Typography>
        </Stack>
      </Stack>
      <Stack direction="row" spacing={2} flexWrap="wrap">
        {[
          { label: '시가', value: p.stck_oprc },
          { label: '고가', value: p.stck_hgpr, color: 'error.main' },
          { label: '저가', value: p.stck_lwpr, color: 'primary.main' },
          { label: '거래량', text: fmt(parseInt(p.acml_vol)) },
        ].filter(i => i.value || i.text).map(({ label, value, color, text }) => (
          <Box key={label}>
            <Typography variant="caption" color="text.secondary" display="block">{label}</Typography>
            <Typography variant="caption" fontWeight={600} color={color ?? 'text.primary'}>
              {text ?? (value ? fmt(parseInt(value)) + '원' : '-')}
            </Typography>
          </Box>
        ))}
      </Stack>
    </Box>
  )
}

export function UsStockInfoCard({ symbol, exchange }: { symbol: string; exchange: OverseasExchange }) {
  const { data: p, isLoading, isError } = useOverseasPrice(symbol, exchange)
  if (!symbol) return null
  if (isLoading) return (
    <Box sx={{ p: 2, display: 'flex', alignItems: 'center', gap: 1 }}>
      <Typography variant="body2" color="text.secondary">조회 중</Typography>
    </Box>
  )
  if (isError || !p) return (
    <Box sx={{ px: 2, py: 1 }}>
      <Typography variant="caption" color="error">종목 정보를 불러올 수 없습니다</Typography>
    </Box>
  )

  const price = parseFloat(p.last)
  const change = parseFloat(p.diff)
  const rt = parseFloat(p.rate)
  const pos = rt >= 0

  return (
    <Box sx={{ px: { xs: 1.5, sm: 2.5 }, py: 1.5 }}>
      <Stack direction="row" spacing={1.5} alignItems="baseline" flexWrap="wrap" mb={1}>
        <Box>
          <Typography variant="subtitle2" fontWeight={700}>{p.name ?? symbol}</Typography>
          <Typography variant="caption" color="text.secondary">{symbol} · {exchange}</Typography>
        </Box>
        <Typography variant="h5" fontWeight={700}>${price.toFixed(2)}</Typography>
        <Stack direction="row" alignItems="center" spacing={0.5}>
          {pos
            ? <TrendingUpIcon fontSize="small" color="success" />
            : <TrendingDownIcon fontSize="small" color="error" />}
          <Typography variant="body2" color={pos ? 'success.main' : 'error.main'} fontWeight={600}>
            {pos ? '+' : ''}{change.toFixed(2)} ({pos ? '+' : ''}{rt.toFixed(2)}%)
          </Typography>
        </Stack>
      </Stack>
      <Stack direction="row" spacing={2} flexWrap="wrap">
        {[
          { label: '시가', text: '$' + parseFloat(p.open).toFixed(2) },
          { label: '고가', text: '$' + parseFloat(p.high).toFixed(2), color: 'error.main' },
          { label: '저가', text: '$' + parseFloat(p.low).toFixed(2), color: 'primary.main' },
          { label: '거래량', text: fmt(parseInt(p.tvol)) },
        ]
          .filter(i => i.text && i.text !== '$0.00' && i.text !== '$NaN')
          .map(({ label, text, color }) => (
            <Box key={label}>
              <Typography variant="caption" color="text.secondary" display="block">{label}</Typography>
              <Typography variant="caption" fontWeight={600} color={color ?? 'text.primary'}>{text}</Typography>
            </Box>
          ))}
      </Stack>
    </Box>
  )
}
