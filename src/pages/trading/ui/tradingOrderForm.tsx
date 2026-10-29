import Alert from '@mui/material/Alert'
import Box from '@mui/material/Box'
import Button from '@mui/material/Button'
import Chip from '@mui/material/Chip'
import CircularProgress from '@mui/material/CircularProgress'
import InputAdornment from '@mui/material/InputAdornment'
import Paper from '@mui/material/Paper'
import Stack from '@mui/material/Stack'
import TextField from '@mui/material/TextField'
import ToggleButton from '@mui/material/ToggleButton'
import ToggleButtonGroup from '@mui/material/ToggleButtonGroup'
import Tooltip from '@mui/material/Tooltip'
import Typography from '@mui/material/Typography'
import type { TossManualSession } from '../../../api/types'
import { TossOpenOrdersPanel, TossOrderPreflightPanel } from './tossPanels'
import type { TradingOrderModel } from './useTradingOrderModel'

// Presentation only: the parent retains query/state ownership and callback closures.
export function TradingOrderForm({ model }: { model: TradingOrderModel }) {
  const {
    TOSS_US_SESSION_OPTIONS, availableCash, clearBuySuspension, clearingBuySusp,
    errorMsg, fmt, fmtTossSessionWindow, handleFillMarketPrice,
    handleSubmit, isPaperTrading, isPaperUnsupportedUsSell, isPending,
    isTossActive, isTossPreflightError, isTossPreflightLoading, krCurrentPrice,
    market, normalizedUsSymbol, orderType, overseasOrderExchange,
    price, quantity, result, selectedTossSessionOpen,
    selectedTossSessionWindow, setOrderType, setPrice, setQuantity,
    setSide, setTossSession, showTossUsSessionSelector, side,
    symbol, tossCurrentPrice, tossManualOrderReady, tossPreflight,
    tossPreflightError, tossSafety, tossSession, tossSessionLabel,
    tradingStatus, usCurrentPrice, usExchange,
  } = model
  return (
    <Paper sx={{ p: { xs: 2, sm: 3 }, height: '100%' }}>
      <Stack direction="row" sx={{ alignItems: "center", mb: 2 }} spacing={1}>
        <Typography variant="subtitle1" sx={{ fontWeight: 600 }}>수동 주문</Typography>
        {symbol && (
          <Chip
            label={isTossActive ? `${symbol} (Toss)` : market === 'KR' ? symbol : symbol + ' (' + usExchange + ')'}
            size="small"
            color={isTossActive ? 'secondary' : market === 'US' ? 'info' : 'default'}
            variant="outlined"
          />
        )}
      </Stack>

      <ToggleButtonGroup
        value={side} exclusive onChange={(_, v) => v && setSide(v)}
        fullWidth size="small" sx={{ mb: 2 }}
      >
        <ToggleButton value="Buy" color="primary">매수</ToggleButton>
        <ToggleButton value="Sell" color="error">매도</ToggleButton>
      </ToggleButtonGroup>

      {market === 'KR' && (
        <ToggleButtonGroup
          value={orderType} exclusive onChange={(_, v) => v && setOrderType(v)}
          fullWidth size="small" sx={{ mb: 2 }}
        >
          <ToggleButton value="Limit">지정가</ToggleButton>
          <ToggleButton value="Market">시장가</ToggleButton>
        </ToggleButtonGroup>
      )}

      {showTossUsSessionSelector && (
        <Box
          sx={{
            mb: 1.5,
            p: 1.25,
            border: 1,
            borderColor: 'divider',
            borderRadius: 1,
            bgcolor: 'action.hover',
          }}
        >
          <Stack direction="row" spacing={1} sx={{ alignItems: "center", justifyContent: "space-between", mb: 1  }}>
            <Typography variant="caption" sx={{ fontWeight: 700 }}>
              Toss 미국 거래 세션
            </Typography>
            {tossSession !== 'auto' && (
              <Chip
                size="small"
                label={selectedTossSessionOpen ? '현재 가능' : '현재 아님'}
                color={selectedTossSessionOpen ? 'success' : 'default'}
                variant="outlined"
                sx={{ height: 20, fontSize: '0.68rem' }}
              />
            )}
          </Stack>
          <ToggleButtonGroup
            value={tossSession}
            exclusive
            onChange={(_, v: TossManualSession | null) => v && setTossSession(v)}
            fullWidth
            size="small"
            sx={{ mb: 0.75 }}
          >
            {TOSS_US_SESSION_OPTIONS.map((option) => (
              <ToggleButton key={option.value} value={option.value}>
                {option.label}
              </ToggleButton>
            ))}
          </ToggleButtonGroup>
          <Typography variant="caption" color="text.secondary">
            {tossSession === 'auto'
              ? '세션을 지정하지 않고 Toss 주문 접수 가능 여부를 provider 응답에 맡깁니다.'
              : `${tossSessionLabel(tossSession)} ${fmtTossSessionWindow(selectedTossSessionWindow)} · ` +
                (selectedTossSessionOpen ? '선택 세션이 열려 있습니다.' : '선택 세션 시간이 아닙니다.')}
          </Typography>
        </Box>
      )}

      {isTossActive && !tossManualOrderReady && (
        <Alert severity="info" sx={{ mb: 1.5 }}>
          Toss 수동 주문은 실거래 동의와 주문 전 검증을 통과하면 제출됩니다. 시장가 주문은 가격 없이 수량 기준으로 전송됩니다.
        </Alert>
      )}

      {isTossActive && tossSafety?.buyBlockReason && (
        <Alert severity="warning" sx={{ mb: 1.5 }}>
          {tossSafety.buyBlockReason}
        </Alert>
      )}

      {(market === 'US' || orderType === 'Limit') && (
        <TextField
          label={market === 'US' ? '주문가격 (USD)' : '주문가격'}
          value={price}
          onChange={(e) => setPrice(e.target.value.replace(/[^0-9.]/g, ''))}
          fullWidth size="small" sx={{ mb: 1.5 }}
          slotProps={{ input: {
            endAdornment: (
              <InputAdornment position="end">
                <Tooltip title="현재가 자동 입력">
                  <span>
                    <Button
                      size="small"
                      onClick={handleFillMarketPrice}
                      disabled={isTossActive ? !(tossCurrentPrice && Number.isFinite(tossCurrentPrice)) : !(market === 'KR' ? krCurrentPrice : usCurrentPrice)}
                    >
                      현재가
                    </Button>
                  </span>
                </Tooltip>
              </InputAdornment>
            ),
          } }}
        />
      )}

      <TextField
        label="주문수량"
        value={quantity}
        onChange={(e) => setQuantity(e.target.value.replace(/\D/g, ''))}
        fullWidth size="small" sx={{ mb: 2 }}
      />

      {quantity && price && (
        <Box sx={{ mb: 2, p: 1.5, bgcolor: 'action.hover', borderRadius: 1 }}>
          <Typography variant="body2">
            예상 금액:{' '}
            <strong>
              {market === 'US'
                ? '$' + (parseFloat(quantity || '0') * parseFloat(price || '0')).toFixed(2)
                : fmt(parseInt(quantity || '0') * parseInt(price || '0')) + '원'}
            </strong>
          </Typography>
          {market === 'KR' && (
            <Typography variant="body2" color="text.secondary">
              예수금: {fmt(availableCash)}원
            </Typography>
          )}
        </Box>
      )}

      {isTossActive && symbol && quantity && (
        <TossOrderPreflightPanel
          data={tossPreflight}
          isLoading={isTossPreflightLoading}
          isError={isTossPreflightError}
          error={tossPreflightError}
        />
      )}

      {result   && <Alert severity="success" sx={{ mb: 1.5 }}>{result}</Alert>}
      {errorMsg && <Alert severity="error" sx={{ mb: 1.5 }}>{errorMsg}</Alert>}

      {market === 'US' && isPaperTrading && (
        <Alert severity={isPaperUnsupportedUsSell ? 'warning' : 'info'} sx={{ mb: 1.5 }}>
          {isPaperUnsupportedUsSell
            ? `${normalizedUsSymbol || '선택 종목'} (${overseasOrderExchange}) 매도는 KIS 모의투자에서 제한될 수 있어 주문 전 차단됩니다.`
            : '모의 해외 주문은 USD 지정가로만 전송합니다. 일부 AMEX/ETF 종목은 모의투자 주문 universe에서 제한될 수 있습니다.'}
        </Alert>
      )}

      {/* 잔고 부족 매수 정지 경고 */}
      {tradingStatus?.buySuspended && side === 'Buy' && (
        <Alert
          severity="warning"
          sx={{ mb: 1.5 }}
          action={
            <Button
              size="small"
              color="inherit"
              onClick={() => clearBuySuspension()}
              disabled={clearingBuySusp}
              startIcon={clearingBuySusp ? <CircularProgress size={12} color="inherit" /> : undefined}
            >
              해제
            </Button>
          }
        >
          잔고 부족으로 매수 주문이 정지되었습니다.
          매도 체결로 자본 확보 시 자동 재개됩니다.
        </Alert>
      )}

      <Button
        variant="contained"
        color={side === 'Buy' ? 'primary' : 'error'}
        fullWidth
        onClick={handleSubmit}
        disabled={isPending || !symbol || isPaperUnsupportedUsSell || (isTossActive && !tossManualOrderReady)}
        startIcon={isPending ? <CircularProgress size={16} color="inherit" /> : undefined}
        sx={{ py: 1.2 }}
      >
        {side === 'Buy' ? '매수 주문' : '매도 주문'}
        {market === 'US' && ' (USD 지정가)'}
      </Button>

      {isTossActive && <TossOpenOrdersPanel symbol={symbol} />}
    </Paper>
  )
}
