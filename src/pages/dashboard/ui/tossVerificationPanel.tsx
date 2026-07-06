import { useEffect, useRef, useState } from 'react'

import Alert from '@mui/material/Alert'
import Box from '@mui/material/Box'
import Button from '@mui/material/Button'
import Checkbox from '@mui/material/Checkbox'
import Chip from '@mui/material/Chip'
import CircularProgress from '@mui/material/CircularProgress'
import Divider from '@mui/material/Divider'
import FormControlLabel from '@mui/material/FormControlLabel'
import IconButton from '@mui/material/IconButton'
import Paper from '@mui/material/Paper'
import Stack from '@mui/material/Stack'
import Table from '@mui/material/Table'
import TableBody from '@mui/material/TableBody'
import TableCell from '@mui/material/TableCell'
import TableHead from '@mui/material/TableHead'
import TableRow from '@mui/material/TableRow'
import TextField from '@mui/material/TextField'
import Tooltip from '@mui/material/Tooltip'
import Typography from '@mui/material/Typography'
import SearchIcon from '@mui/icons-material/Search'

import {
  useProfiles,
  useStockSearch,
  useSubmitTossSmallBuyVerification,
  useTossOrderPreflight,
} from '../../../api/hooks'
import type { AppConfigView, CmdError, StockSearchItem } from '../../../api/types'
import { TossManualTradeVerificationPanel } from '../../../features/manual-order'
import { fmtBrokerMoney } from '../../../shared/lib'

const TEST_QUANTITY = '1'

function isDirectSymbol(value: string) {
  return /^[A-Z0-9]{6}$/i.test(value.trim())
}

export function DashboardTossVerificationPanel({
  appConfig,
}: {
  appConfig: AppConfigView | undefined
}) {
  const [inputValue, setInputValue] = useState('')
  const [searchQuery, setSearchQuery] = useState('')
  const [showResults, setShowResults] = useState(false)
  const [symbol, setSymbol] = useState('')
  const [selectedName, setSelectedName] = useState('')
  const [maxNotionalAmount, setMaxNotionalAmount] = useState('')
  const [confirmed, setConfirmed] = useState(false)
  const closeTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null)

  const isTossActive = appConfig?.active_broker_id === 'toss'
  const { data: profiles = [] } = useProfiles({ enabled: isTossActive })
  const activeProfile = profiles.find((profile) => profile.id === appConfig?.active_profile_id) ?? null
  const {
    data: searchResults = [],
    isFetching: isFetchingSearch,
    isError: isSearchError,
    error: searchError,
  } = useStockSearch(searchQuery)
  const {
    data: preflight,
    isLoading: preflightLoading,
    isError: preflightError,
  } = useTossOrderPreflight(
    {
      symbol,
      side: 'Buy',
      quantity: TEST_QUANTITY,
      price: null,
    },
    { enabled: isTossActive && !!symbol },
  )
  const {
    mutate: submitSmallBuy,
    isPending: submitting,
    data: submitResult,
    error: submitError,
    reset: resetSubmit,
  } = useSubmitTossSmallBuyVerification()

  useEffect(() => {
    if (!isTossActive || !showResults || inputValue.trim().length < 2 || isDirectSymbol(inputValue)) {
      setSearchQuery('')
      return
    }

    const timeout = setTimeout(() => setSearchQuery(inputValue.trim()), 350)
    return () => clearTimeout(timeout)
  }, [inputValue, isTossActive, showResults])

  useEffect(() => {
    const amount = preflight?.requiredCash?.amount ?? preflight?.grossAmount.amount ?? ''
    setMaxNotionalAmount(amount)
    setConfirmed(false)
    resetSubmit()
  }, [preflight?.requiredCash?.amount, preflight?.grossAmount.amount, resetSubmit])

  if (!isTossActive) return null

  const isStockListEmpty =
    isSearchError && (searchError as CmdError | null)?.code === 'STOCK_LIST_EMPTY'
  const selectedLabel = selectedName || symbol
  const market = preflight?.market === 'us' ? 'US' : 'KR'

  const handleInputChange = (value: string) => {
    const normalized = value.trim().toUpperCase()
    setInputValue(value)
    setSelectedName('')
    setConfirmed(false)
    resetSubmit()

    if (isDirectSymbol(normalized)) {
      setSymbol(normalized)
      setShowResults(false)
      setSearchQuery('')
      return
    }

    setSymbol('')
    setShowResults(value.trim().length >= 2)
  }

  const handleSelect = (item: StockSearchItem) => {
    setSymbol(item.pdno)
    setSelectedName(item.prdt_name)
    setInputValue(item.prdt_name)
    setConfirmed(false)
    resetSubmit()
    setShowResults(false)
    setSearchQuery('')
  }

  const consentReady = activeProfile?.live_trading_consent ?? false
  const submitReady =
    !!symbol &&
    !!preflight &&
    preflight.liquidityOk &&
    preflight.safetyOk &&
    !preflightError &&
    consentReady &&
    confirmed &&
    maxNotionalAmount.trim().length > 0

  const handleSubmitSmallBuy = () => {
    if (!preflight || !appConfig?.active_broker_account_id) return
    submitSmallBuy({
      symbol,
      symbolName: selectedName || symbol,
      expectedAccountSeq: appConfig.active_broker_account_id,
      maxNotionalAmount,
      confirmed,
    })
  }

  return (
    <Paper sx={{ p: 2.5, mb: 2 }}>
      <Stack direction="row" alignItems="center" spacing={1} mb={1.5} flexWrap="wrap">
        <Typography variant="subtitle1" fontWeight={600}>
          Toss 소액매매 검증
        </Typography>
        <Chip
          size="small"
          label="검색 종목 1주 시장가 검토"
          color="warning"
          variant="outlined"
          sx={{ height: 20, fontSize: '0.7rem' }}
        />
        {appConfig?.active_broker_account_id && (
          <Typography variant="caption" color="text.secondary">
            accountSeq {appConfig.active_broker_account_id}
          </Typography>
        )}
      </Stack>
      <Divider sx={{ mb: 1.5 }} />

      <Alert severity="warning" sx={{ mb: 1.5 }}>
        이 패널은 Dashboard에서 검색한 종목을 1주 시장가 매수 조건으로 사전검증합니다.
        실거래 동의와 최종 확인 후 실제 계좌에 1주 시장가 매수 주문을 제출할 수 있습니다.
      </Alert>

      <Box sx={{ position: 'relative', mb: 1.5 }}>
        <TextField
          label="검증할 국내 종목명 또는 6자리 코드"
          value={inputValue}
          onChange={(e) => handleInputChange(e.target.value)}
          onBlur={() => {
            closeTimerRef.current = setTimeout(() => setShowResults(false), 180)
          }}
          onFocus={() => {
            if (closeTimerRef.current) clearTimeout(closeTimerRef.current)
            if (inputValue.trim().length >= 2 && !symbol) setShowResults(true)
          }}
          onKeyDown={(e) => {
            if (e.key === 'Escape') {
              setShowResults(false)
              setSearchQuery('')
            }
          }}
          size="small"
          fullWidth
          InputProps={{
            endAdornment: (
              <Tooltip title="검색">
                <span>
                  <IconButton
                    size="small"
                    onClick={() => {
                      if (inputValue.trim().length >= 2) {
                        setSearchQuery(inputValue.trim())
                        setShowResults(true)
                      }
                    }}
                    disabled={inputValue.trim().length < 2}
                  >
                    {isFetchingSearch ? <CircularProgress size={16} /> : <SearchIcon fontSize="small" />}
                  </IconButton>
                </span>
              </Tooltip>
            ),
          }}
          helperText={symbol ? `선택됨: ${selectedLabel} (${symbol})` : '검색 후 종목을 선택하면 1주 시장가 기준으로 Toss 주문 전 검증을 실행합니다.'}
        />

        {showResults && (searchResults.length > 0 || isFetchingSearch) && (
          <Paper
            elevation={8}
            onMouseDown={(e) => {
              e.preventDefault()
              if (closeTimerRef.current) clearTimeout(closeTimerRef.current)
            }}
            sx={{
              mt: 0.5,
              maxHeight: 240,
              overflow: 'auto',
              border: 1,
              borderColor: 'divider',
              position: 'absolute',
              left: 0,
              right: 0,
              zIndex: 5,
            }}
          >
            {isFetchingSearch && searchResults.length === 0 ? (
              <Box sx={{ p: 1.5, display: 'flex', alignItems: 'center', gap: 1 }}>
                <CircularProgress size={14} />
                <Typography variant="caption" color="text.secondary">
                  검색 중...
                </Typography>
              </Box>
            ) : (
              <Table size="small">
                <TableHead>
                  <TableRow>
                    <TableCell sx={{ py: 0.75, fontWeight: 700, fontSize: '0.7rem' }}>종목명</TableCell>
                    <TableCell sx={{ py: 0.75, fontWeight: 700, fontSize: '0.7rem', width: 88 }}>코드</TableCell>
                  </TableRow>
                </TableHead>
                <TableBody>
                  {searchResults.map((item) => (
                    <TableRow
                      key={item.pdno}
                      hover
                      sx={{ cursor: 'pointer' }}
                      onClick={() => handleSelect(item)}
                    >
                      <TableCell sx={{ py: 0.75 }}>
                        <Typography variant="body2" noWrap>
                          {item.prdt_name}
                        </Typography>
                      </TableCell>
                      <TableCell sx={{ py: 0.75 }}>
                        <Typography variant="caption" color="text.secondary">
                          {item.pdno}
                        </Typography>
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            )}
          </Paper>
        )}

        {showResults && !isFetchingSearch && searchQuery.length >= 2 && (searchResults.length === 0 || isStockListEmpty) && (
          <Alert severity={isStockListEmpty ? 'warning' : 'info'} sx={{ mt: 0.75, py: 0.5 }}>
            {isStockListEmpty ? '종목 목록이 비어 있습니다. Settings에서 종목 목록 갱신을 먼저 실행하세요.' : '검색 결과가 없습니다.'}
          </Alert>
        )}
      </Box>

      {preflight && (
        <Stack direction="row" spacing={0.75} mb={1.5} flexWrap="wrap" useFlexGap>
          <Chip
            size="small"
            label={`현재가 ${fmtBrokerMoney(preflight.price)}`}
            variant="outlined"
          />
          <Chip
            size="small"
            label={`예상 주문금액 ${fmtBrokerMoney(preflight.grossAmount)}`}
            color={preflight.liquidityOk ? 'success' : 'warning'}
            variant="outlined"
          />
          <Chip
            size="small"
            label={preflight.safetyOk ? '종목 유의사항 통과' : '종목 유의사항 확인 필요'}
            color={preflight.safetyOk ? 'success' : 'warning'}
            variant="outlined"
          />
        </Stack>
      )}

      {preflightError && (
        <Alert severity="warning" sx={{ mb: 1.5 }}>
          Toss 주문 전 사전검증 실패 — 프로파일 연결 진단과 종목 코드를 확인하세요.
        </Alert>
      )}

      <TossManualTradeVerificationPanel
        appConfig={appConfig}
        activeProfile={activeProfile}
        symbol={symbol}
        market={market}
        side="Buy"
        orderType="Market"
        quantity={symbol ? TEST_QUANTITY : ''}
        price=""
        preflight={preflight}
        preflightLoading={preflightLoading}
        preflightError={preflightError}
        adapterBlockedMessage="Dashboard 전용 소액매매 버튼으로만 실제 1주 시장가 매수를 제출할 수 있습니다."
      />

      {preflight && (
        <Box sx={{ p: 1.5, border: 1, borderColor: 'divider', borderRadius: 1 }}>
          <Stack spacing={1.25}>
            <TextField
              label={`최대 허용 주문금액 (${preflight.grossAmount.currency})`}
              value={maxNotionalAmount}
              onChange={(e) => setMaxNotionalAmount(e.target.value.replace(/[^0-9.]/g, ''))}
              size="small"
              fullWidth
              helperText="시장가 주문은 최종 체결가를 보장하지 않습니다. 제출 직전 사전검증 필요금액이 이 값을 넘으면 서버에서 차단합니다."
            />
            <FormControlLabel
              control={
                <Checkbox
                  checked={confirmed}
                  onChange={(e) => setConfirmed(e.target.checked)}
                  color="warning"
                />
              }
              label="실제 Toss 계좌에서 선택 종목 1주 시장가 매수 주문이 실행될 수 있음을 확인합니다."
            />
            {submitError && (
              <Alert severity="error" sx={{ py: 0.5 }}>
                {submitError.message}
              </Alert>
            )}
            {submitResult && (
              <Alert severity={submitResult.status === 'REJECTED' ? 'warning' : 'success'} sx={{ py: 0.5 }}>
                {submitResult.message} orderId {submitResult.orderId}
                {submitResult.clientOrderId ? ` · clientOrderId ${submitResult.clientOrderId}` : ''}
                {submitResult.averageFilledPrice ? ` · 평균 체결가 ${fmtBrokerMoney(submitResult.averageFilledPrice)}` : ''}
              </Alert>
            )}
            <Button
              variant="contained"
              color="warning"
              disabled={!submitReady || submitting}
              onClick={handleSubmitSmallBuy}
              startIcon={submitting ? <CircularProgress size={16} color="inherit" /> : undefined}
            >
              실제 1주 시장가 매수 실행
            </Button>
            {!consentReady && (
              <Typography variant="caption" color="text.secondary">
                먼저 위의 `소액 실거래 검증 동의 저장` 버튼으로 실거래 동의를 저장하세요.
              </Typography>
            )}
          </Stack>
        </Box>
      )}
    </Paper>
  )
}
