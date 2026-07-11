import AddIcon from '@mui/icons-material/Add'
import DeleteIcon from '@mui/icons-material/Delete'
import InfoOutlinedIcon from '@mui/icons-material/InfoOutlined'
import Box from '@mui/material/Box'
import Button from '@mui/material/Button'
import IconButton from '@mui/material/IconButton'
import InputAdornment from '@mui/material/InputAdornment'
import Stack from '@mui/material/Stack'
import Table from '@mui/material/Table'
import TableBody from '@mui/material/TableBody'
import TableCell from '@mui/material/TableCell'
import TableContainer from '@mui/material/TableContainer'
import TableHead from '@mui/material/TableHead'
import TableRow from '@mui/material/TableRow'
import TextField from '@mui/material/TextField'
import Tooltip from '@mui/material/Tooltip'
import Typography from '@mui/material/Typography'

import type { PriceConditionSymbolConfig, StockSearchItem } from '../../../api/types'

type Props = {
  stratEnabled: boolean
  initialSymbols: PriceConditionSymbolConfig[]
  editedSymbols: PriceConditionSymbolConfig[] | undefined
  selectedStock: StockSearchItem | null
  market: 'KR' | 'US'
  onUpdate: (symbols: PriceConditionSymbolConfig[]) => void
}

export function PriceConditionEditorPanel({
  stratEnabled,
  initialSymbols,
  editedSymbols,
  selectedStock,
  market,
  onUpdate,
}: Props) {
  const symbols = editedSymbols ?? initialSymbols

  const handleAdd = () => {
    if (!selectedStock || symbols.some((symbol) => symbol.symbol === selectedStock.pdno)) return
    onUpdate([
      ...symbols,
      {
        symbol: selectedStock.pdno,
        symbol_name: selectedStock.prdt_name,
        quantity: 1,
        buy_trigger_price: 0,
        sell_trigger_price: 0,
        take_profit_pct: 5,
        stop_loss_pct: 3,
        is_overseas: market === 'US',
      },
    ])
  }

  const handleFieldChange = (
    symbol: string,
    field: keyof PriceConditionSymbolConfig,
    value: number,
  ) => {
    onUpdate(symbols.map((item) => (item.symbol === symbol ? { ...item, [field]: value } : item)))
  }

  const numericFields: Array<{
    key: keyof PriceConditionSymbolConfig
    label: string
    isPrice: boolean
  }> = [
    { key: 'quantity', label: '수량', isPrice: false },
    { key: 'buy_trigger_price', label: '매수가', isPrice: true },
    { key: 'sell_trigger_price', label: '익절가', isPrice: true },
    { key: 'take_profit_pct', label: '익절%', isPrice: false },
    { key: 'stop_loss_pct', label: '손절%', isPrice: false },
  ]

  return (
    <Stack spacing={1.5}>
      <Stack direction="row" alignItems="center" justifyContent="space-between" flexWrap="wrap" gap={1}>
        <Stack direction="row" alignItems="center" gap={0.5}>
          <Typography variant="caption" color="text.secondary" fontWeight={600}>
            대상 종목 ({symbols.length}개)
          </Typography>
          <Tooltip
            title="종목별로 매수가·익절가·손절%·익절%를 독립 설정. 매수가 ≤ 현재가이면 매수, 매수 후 지정가/비율 조건 달성 시 매도. 0은 해당 조건 비활성."
            arrow
          >
            <InfoOutlinedIcon sx={{ fontSize: 13, color: 'text.disabled', cursor: 'help' }} />
          </Tooltip>
        </Stack>
        <Button
          size="small"
          variant="outlined"
          startIcon={<AddIcon />}
          disabled={!selectedStock || stratEnabled || symbols.some((symbol) => symbol.symbol === selectedStock.pdno)}
          onClick={handleAdd}
          sx={{ fontSize: '0.7rem', py: 0.3 }}
        >
          {selectedStock ? `${selectedStock.prdt_name} 추가` : '위에서 종목 선택'}
        </Button>
      </Stack>

      {symbols.length === 0 ? (
        <Typography variant="caption" color="text.disabled" sx={{ pl: 0.5 }}>
          추가된 종목이 없습니다
        </Typography>
      ) : (
        <TableContainer sx={{ border: 1, borderColor: 'divider', borderRadius: 1, overflowX: 'auto' }}>
          <Table size="small" sx={{ minWidth: 750 }}>
            <TableHead>
              <TableRow>
                <TableCell sx={{ fontSize: '0.7rem', py: 0.75, minWidth: 110 }}>종목</TableCell>
                {numericFields.map((field) => (
                  <TableCell
                    key={field.key}
                    sx={{ fontSize: '0.7rem', py: 0.75, minWidth: field.isPrice ? 130 : 90 }}
                    align="center"
                  >
                    {field.isPrice ? `${field.label}(원/$)` : field.label}
                  </TableCell>
                ))}
                <TableCell sx={{ width: 36, py: 0.75 }} />
              </TableRow>
            </TableHead>
            <TableBody>
              {symbols.map((symbol) => (
                <TableRow key={symbol.symbol}>
                  <TableCell sx={{ py: 0.5 }}>
                    <Stack direction="row" alignItems="center" gap={0.5}>
                      {symbol.is_overseas && (
                        <Typography variant="caption" color="primary.main" fontWeight={700} sx={{ fontSize: '0.6rem' }}>
                          $
                        </Typography>
                      )}
                      <Box>
                        <Typography variant="caption" fontWeight={600}>{symbol.symbol}</Typography>
                        <Typography variant="caption" color="text.secondary" display="block" noWrap sx={{ maxWidth: 80 }}>
                          {symbol.symbol_name}
                        </Typography>
                      </Box>
                    </Stack>
                  </TableCell>
                  {numericFields.map((field) => {
                    const step = field.isPrice ? (symbol.is_overseas ? 0.01 : 100) : 0.5
                    const fieldStep = field.key === 'quantity' ? 1 : step
                    return (
                      <TableCell key={field.key} sx={{ py: 0.25 }} align="center">
                        <TextField
                          type="number"
                          value={symbol[field.key] as number}
                          disabled={stratEnabled}
                          size="small"
                          onChange={(event) => handleFieldChange(symbol.symbol, field.key, Number(event.target.value))}
                          inputProps={{
                            'aria-label': `${symbol.symbol} ${field.label}`,
                            min: field.key === 'quantity' ? 1 : 0,
                            step: fieldStep,
                            style: { padding: '4px 4px', fontSize: '0.75rem', textAlign: 'right' },
                          }}
                          InputProps={field.isPrice ? {
                            endAdornment: (
                              <InputAdornment position="end">
                                <Typography variant="caption" color="text.secondary" sx={{ fontSize: '0.65rem', lineHeight: 1 }}>
                                  {symbol.is_overseas ? '$' : '원'}
                                </Typography>
                              </InputAdornment>
                            ),
                          } : undefined}
                          sx={{ width: '100%', '& .MuiInputBase-root': { fontSize: '0.75rem' } }}
                        />
                      </TableCell>
                    )
                  })}
                  <TableCell sx={{ py: 0.5 }}>
                    <IconButton
                      size="small"
                      disabled={stratEnabled}
                      aria-label={`${symbol.symbol} 삭제`}
                      onClick={() => onUpdate(symbols.filter((item) => item.symbol !== symbol.symbol))}
                    >
                      <DeleteIcon sx={{ fontSize: 14 }} />
                    </IconButton>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </TableContainer>
      )}
    </Stack>
  )
}
