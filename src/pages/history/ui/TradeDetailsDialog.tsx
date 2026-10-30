import { useId } from 'react'
import Box from '@mui/material/Box'
import Button from '@mui/material/Button'
import Dialog from '@mui/material/Dialog'
import DialogActions from '@mui/material/DialogActions'
import DialogContent from '@mui/material/DialogContent'
import DialogTitle from '@mui/material/DialogTitle'
import Typography from '@mui/material/Typography'
import type { TradeRecord } from '../../../api/types'
import { ProviderTraceChips, type ProviderTrace } from '../../../shared/ui'

interface TradeDetailsDialogProps {
  trade: TradeRecord | null
  price: string
  amount: string
  fee: string
  slippage: string
  quantity: string
  currency: string
  trace: ProviderTrace
  onClose: () => void
}

/** Read-only projection of the selected existing record; no query or money calculation. */
export function TradeDetailsDialog({ trade, price, amount, fee, slippage, quantity, currency, trace, onClose }: TradeDetailsDialogProps) {
  const titleId = useId()
  const fields = trade ? [
    ['체결 ID', trade.id],
    ['일시 (기록값)', trade.timestamp],
    ['종목', `${trade.symbol_name} (${trade.symbol})`],
    ['구분', trade.side === 'buy' ? '매수' : '매도'],
    ['수량', quantity],
    ['통화', currency],
    ['거래소', trade.exchange],
    ['가격', price],
    ['금액', amount],
    ['수수료', fee],
    ['슬리피지', slippage],
    ['전략', trade.strategy_id],
    ['Provider', trace.provider],
    ['TR-ID', trace.providerTrId],
    ['Provider order ID', trace.providerOrderId],
    ['Provider request ID', trace.providerRequestId],
  ] : []

  return (
    <Dialog
      open={trade !== null}
      onClose={onClose}
      aria-labelledby={titleId}
      maxWidth="sm"
      fullWidth
      scroll="paper"
      slotProps={{ paper: { sx: { m: { xs: 1, sm: 4 }, width: { xs: 'calc(100% - 16px)', sm: 'calc(100% - 64px)' }, maxHeight: 'calc(100% - 32px)' } } }}
    >
      <DialogTitle id={titleId} sx={{ overflowWrap: 'anywhere' }}>
        거래 상세 — {trade?.symbol_name} ({trade?.symbol})
      </DialogTitle>
      <DialogContent data-testid="trade-details-content" dividers tabIndex={0}>
        <Box component="dl" sx={{ m: 0, display: 'grid', gap: 1.5, minWidth: 0 }}>
          {fields.map(([label, value]) => (
            <Box key={label} sx={{ minWidth: 0 }}>
              <Typography component="dt" variant="caption" color="text.secondary">{label}</Typography>
              <Typography component="dd" variant="body2" sx={{ m: 0, overflowWrap: 'anywhere' }}>
                {value || '-'}
              </Typography>
            </Box>
          ))}
        </Box>
        <Box sx={{ mt: 2 }}><ProviderTraceChips trace={trace} /></Box>
      </DialogContent>
      <DialogActions><Button autoFocus onClick={onClose}>닫기</Button></DialogActions>
    </Dialog>
  )
}
