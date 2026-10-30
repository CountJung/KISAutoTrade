import { useRef } from 'react'
import type { StockSearchItem } from '../../../api/types'
import FlagIcon from '@mui/icons-material/Flag'
import PublicIcon from '@mui/icons-material/Public'
import RefreshIcon from '@mui/icons-material/Refresh'
import SearchIcon from '@mui/icons-material/Search'
import Alert from '@mui/material/Alert'
import Box from '@mui/material/Box'
import Button from '@mui/material/Button'
import Chip from '@mui/material/Chip'
import CircularProgress from '@mui/material/CircularProgress'
import Divider from '@mui/material/Divider'
import Grid from '@mui/material/Grid'
import IconButton from '@mui/material/IconButton'
import InputAdornment from '@mui/material/InputAdornment'
import Paper from '@mui/material/Paper'
import Stack from '@mui/material/Stack'
import Table from '@mui/material/Table'
import TableBody from '@mui/material/TableBody'
import TableCell from '@mui/material/TableCell'
import TableContainer from '@mui/material/TableContainer'
import TableHead from '@mui/material/TableHead'
import TableRow from '@mui/material/TableRow'
import TextField from '@mui/material/TextField'
import ToggleButton from '@mui/material/ToggleButton'
import ToggleButtonGroup from '@mui/material/ToggleButtonGroup'
import Tooltip from '@mui/material/Tooltip'
import Typography from '@mui/material/Typography'
import { fmtBrokerMoney, fmtDecimalString } from '../../../shared/lib'
import { BrokerScopeIndicator, RowSelectionButton } from '../../../shared/ui'
import { OverseasStockChart, StockChart } from '../../../widgets/stock-chart'
import {
  HoldingsTable,
  KrStockInfoCard,
  KisHoldingsSection,
  OverseasHoldingsTable,
  UsStockInfoCard,
} from './kisPanels'
import { TossMarketCalendarStrip, TossMarketSnapshotCard, TossStockSafetyCard } from './tossPanels'
import { TradingOrderForm } from './tradingOrderForm'
import { useTradingOrderModel } from './useTradingOrderModel'

export default function Trading() {
  const model = useTradingOrderModel()
  const searchRegionRef = useRef<HTMLDivElement>(null)
  const searchInputRef = useRef<HTMLInputElement>(null)
  const {
    appConfig, balance, balanceQuery, brokerHoldings, brokerHoldingsError,
    brokerHoldingsLoading, closeTimerRef, doRefreshList, fmt,
    handleSelectBrokerHolding, handleSelectHolding, handleSelectOverseasHolding, handleUsSearch,
    inputValue, isFetchingSearch, isRefreshing, isStockListEmpty,
    isTossActive, isTossCalendarError, isTossCalendarLoading, isTossSafetyError,
    isTossSafetyLoading, market, overseasBalance, overseasBalanceQuery, searchQuery,
    searchResults, setErrorMsg, setInputValue, setMarket,
    setResult, setSearchQuery, setShowResults, setSymbol,
    showResults, stockName, symbol, tossCalendar,
    tossMarketLabel, tossSafety, tossSafetyError, usExchange,
    usSearching,
  } = model
  const selectSearchResult = (r: StockSearchItem) => {
    setSymbol(r.pdno)
    setInputValue(r.prdt_name)
    setShowResults(false)
    setSearchQuery('')
    setResult(null)
    setErrorMsg(null)
    searchInputRef.current?.focus()
  }
  return (
    <Box>
      <Stack direction="row" sx={{ alignItems: "center", gap: 1.5, mb: 2, flexWrap: "wrap" }}>
        <Typography variant="h5" sx={{ fontWeight: 700 }}>Trading</Typography>
        <BrokerScopeIndicator appConfig={appConfig} compact />
      </Stack>

      {/* 1. 보유 종목 */}
      <Paper sx={{ p: { xs: 1.5, sm: 2.5 }, mb: 2 }}>
        <Stack
          direction="row" sx={{ alignItems: "center", justifyContent: "space-between", mb: 1.5, flexWrap: "wrap", gap: 1 }}
        >
          <Stack direction="row" spacing={1} sx={{ alignItems: "center", flexWrap: "wrap" }}>
            <Typography variant="subtitle1" sx={{ fontWeight: 600 }}>보유 종목</Typography>
            {isTossActive && (
              <Chip
                size="small"
                label={appConfig?.active_broker_account_id
                  ? `Toss · accountSeq ${appConfig.active_broker_account_id}`
                  : 'Toss'}
                color="secondary"
                variant="outlined"
              />
            )}
          </Stack>
          {!isTossActive && balance?.summary && (
            <Typography variant="caption" color="text.secondary" noWrap>
              예수금 {fmt(parseInt(balance.summary.dnca_tot_amt))}원
            </Typography>
          )}
        </Stack>
        <Divider sx={{ mb: 1.5 }} />

        {isTossActive ? (
          brokerHoldingsLoading ? (
            <Box sx={{ py: 2, display: 'flex', justifyContent: 'center' }}>
              <CircularProgress size={20} />
            </Box>
          ) : brokerHoldingsError ? (
            <Alert severity="warning" sx={{ py: 0.5 }}>
              Toss 보유종목 조회 실패 — Settings의 Toss Client ID, Client Secret, accountSeq를 확인하세요.
            </Alert>
          ) : brokerHoldings.length === 0 ? (
            <Typography variant="body2" color="text.secondary" sx={{ py: 1 }}>
              보유한 Toss 종목이 없습니다.
            </Typography>
          ) : (
            <TableContainer>
              <Table size="small">
                <TableHead>
                  <TableRow>
                    <TableCell>종목명</TableCell>
                    <TableCell align="right">수량</TableCell>
                    <TableCell align="right" sx={{ display: { xs: 'none', sm: 'table-cell' } }}>평균단가</TableCell>
                    <TableCell align="right" sx={{ display: { xs: 'none', sm: 'table-cell' } }}>현재가</TableCell>
                    <TableCell align="right">평가손익</TableCell>
                  </TableRow>
                </TableHead>
                <TableBody>
                  {brokerHoldings.map((item) => {
                    const pnl = Number(item.unrealizedPnl?.amount.replace(/,/g, '') ?? 0)
                    const pnlPositive = Number.isFinite(pnl) ? pnl >= 0 : true
                    return (
                      <TableRow
                        key={`${item.brokerId}:${item.market}:${item.symbol}`}
                        hover
                        sx={{ cursor: 'pointer' }}
                        onClick={() => handleSelectBrokerHolding(item)}
                      >
                        <TableCell>
                          <RowSelectionButton
                            label={item.symbolName || item.symbol}
                            detail={item.symbol}
                            onSelect={() => handleSelectBrokerHolding(item)}
                          />
                          <Typography variant="caption" color="text.secondary">
                            {item.symbol} · {tossMarketLabel(item.market)}
                          </Typography>
                        </TableCell>
                        <TableCell align="right">{fmtDecimalString(item.quantity, 6)}</TableCell>
                        <TableCell align="right" sx={{ display: { xs: 'none', sm: 'table-cell' } }}>
                          {fmtBrokerMoney(item.averagePrice)}
                        </TableCell>
                        <TableCell align="right" sx={{ display: { xs: 'none', sm: 'table-cell' } }}>
                          {fmtBrokerMoney(item.currentPrice)}
                        </TableCell>
                        <TableCell
                          align="right"
                          sx={{ color: pnlPositive ? 'success.main' : 'error.main', fontWeight: 600 }}
                        >
                          {item.unrealizedPnl ? fmtBrokerMoney(item.unrealizedPnl) : '-'}
                        </TableCell>
                      </TableRow>
                    )
                  })}
                </TableBody>
              </Table>
            </TableContainer>
          )
        ) : (
          <>
            {/* 국내 보유 */}
            <Typography variant="caption" color="text.secondary" sx={{ fontWeight: 600, mb: 0.5, display: 'block'  }}>
              🇰🇷 국내
            </Typography>
            <KisHoldingsSection
              label="국내 잔고"
              hasData={balance !== undefined}
              isError={balanceQuery.isError}
              isFetching={balanceQuery.isFetching}
              dataUpdatedAt={balanceQuery.dataUpdatedAt}
              onRetry={() => { void balanceQuery.refetch() }}
              testId="kis-domestic-holdings"
            >
              {balance && <HoldingsTable items={balance.items} onSelect={handleSelectHolding} />}
            </KisHoldingsSection>

            {/* 해외 보유 */}
            <Box sx={{ mt: 2 }}>
              <Typography variant="caption" color="text.secondary" sx={{ fontWeight: 600, mb: 0.5, display: 'block'  }}>
                🇺🇸 해외 — 클릭하면 US 매도 폼 자동 완성
              </Typography>
              <KisHoldingsSection
                label="해외 잔고"
                hasData={overseasBalance !== undefined}
                isError={overseasBalanceQuery.isError}
                isFetching={overseasBalanceQuery.isFetching}
                dataUpdatedAt={overseasBalanceQuery.dataUpdatedAt}
                onRetry={() => { void overseasBalanceQuery.refetch() }}
                testId="kis-overseas-holdings"
              >
                {overseasBalance && (
                  <OverseasHoldingsTable items={overseasBalance.items} onSelect={handleSelectOverseasHolding} />
                )}
              </KisHoldingsSection>
            </Box>
          </>
        )}
      </Paper>

      {/* 2. 종목 검색 패널 */}
      <Paper sx={{ p: { xs: 1.5, sm: 2.5 }, mb: 2 }}>
        <Stack direction="row" sx={{ alignItems: "center", mb: 1.5, flexWrap: "wrap", gap: 1 }} spacing={1}>
          <Typography variant="subtitle1" sx={{ fontWeight: 600 }}>종목 검색</Typography>
          <ToggleButtonGroup
            value={market} exclusive onChange={(_, v) => v && setMarket(v)} size="small"
          >
            <ToggleButton value="KR" sx={{ px: 1.5, gap: 0.5 }}>
              <FlagIcon sx={{ fontSize: 14 }} /> 국내
            </ToggleButton>
            <ToggleButton value="US" sx={{ px: 1.5, gap: 0.5 }}>
              <PublicIcon sx={{ fontSize: 14 }} /> 미국
            </ToggleButton>
          </ToggleButtonGroup>
        </Stack>
        <Divider sx={{ mb: 1.5 }} />

        {market === 'KR' ? (
          /* ── KR 종목 검색: TextField + 검색결과 드롭다운 테이블 ── */
          <Box
            ref={searchRegionRef}
            onBlur={(event) => {
              if (event.currentTarget.contains(event.relatedTarget)) return
              closeTimerRef.current = setTimeout(() => setShowResults(false), 180)
            }}
            onFocus={() => { if (closeTimerRef.current) clearTimeout(closeTimerRef.current) }}
            onKeyDown={(event) => {
              if (event.key !== 'Escape') return
              event.preventDefault()
              setShowResults(false)
              setSearchQuery('')
              searchInputRef.current?.focus()
            }}
          >
            <TextField
              inputRef={searchInputRef}
              label="6자리 종목코드 (예: 005930, 0005A0)"
              value={inputValue}
              onChange={(e) => {
                const v = e.target.value
                setInputValue(v)
                if (/^[A-Z0-9]{6}$/i.test(v)) {
                  // 6자리 영숫자 코드 직접 입력 (0005A0 등 ETF 포함)
                  setSymbol(v.toUpperCase())
                  setShowResults(false)
                  setSearchQuery('')
                  setResult(null)
                  setErrorMsg(null)
                } else if (!v) {
                  setSymbol('')
                  setShowResults(false)
                  setSearchQuery('')
                } else if (v.length < 6) {
                  // 6자리 미만이면 대기
                  setShowResults(false)
                } else {
                  // 6자리 초과 입력 시 무시
                  setShowResults(false)
                }
              }}
              onFocus={(event) => {
                if (closeTimerRef.current) clearTimeout(closeTimerRef.current)
                if (!searchRegionRef.current?.contains(event.relatedTarget) && inputValue.length >= 2 && !symbol) setShowResults(true)
              }}
              size="small"
              fullWidth
              slotProps={{ input: {
                endAdornment: (
                  <InputAdornment position="end">
                    {isFetchingSearch && (
                      <CircularProgress size={16} color="inherit" sx={{ mr: 0.5 }} />
                    )}
                    <Tooltip title="검색">
                      <span>
                        <IconButton
                          size="small"
                          aria-label="국내 종목 검색"
                          onClick={() => {
                            if (inputValue.length >= 2) {
                              setSearchQuery(inputValue)
                              setShowResults(true)
                            }
                          }}
                          disabled={!inputValue || inputValue.length < 2}
                        >
                          <RefreshIcon fontSize="small" />
                        </IconButton>
                      </span>
                    </Tooltip>
                  </InputAdornment>
                ),
              } }}
              helperText={
                symbol
                  ? `선택됨: ${symbol}`
                  : '국내 주식은 6자리 종목코드로만 검색 가능합니다 (예: 005930, 0005A0)'
              }
            />

            {/* 검색 결과 드롭다운 테이블 */}
            {showResults && (searchResults.length > 0 || isFetchingSearch) && (
              <Paper
                elevation={8}
                onMouseDown={(e) => {
                  // blur 이전에 클릭 이벤트가 발생하도록 preventDefault
                  e.preventDefault()
                  if (closeTimerRef.current) clearTimeout(closeTimerRef.current)
                }}
                sx={{
                  mt: 0.5,
                  maxHeight: 260,
                  overflow: 'auto',
                  border: 1,
                  borderColor: 'divider',
                  zIndex: 1400,
                  position: 'relative',
                }}
              >
                {isFetchingSearch && searchResults.length === 0 ? (
                  <Box sx={{ p: 1.5, display: 'flex', alignItems: 'center', gap: 1 }}>
                    <CircularProgress size={14} />
                    <Typography variant="caption" color="text.secondary">검색 중...</Typography>
                  </Box>
                ) : (
                  <Table size="small">
                    <TableHead>
                      <TableRow>
                        <TableCell sx={{ py: 0.75, fontWeight: 700, fontSize: '0.7rem' }}>종목명</TableCell>
                        <TableCell sx={{ py: 0.75, fontWeight: 700, fontSize: '0.7rem', width: 80 }}>코드</TableCell>
                      </TableRow>
                    </TableHead>
                    <TableBody>
                      {searchResults.map((r) => (
                        <TableRow
                          key={r.pdno}
                          hover
                          sx={{ cursor: 'pointer' }}
                          onClick={() => selectSearchResult(r)}
                        >
                          <TableCell sx={{ py: 0.75, maxWidth: { xs: 180, sm: 400 } }}>
                            <RowSelectionButton
                              label={r.prdt_name}
                              detail={r.pdno}
                              onSelect={() => selectSearchResult(r)}
                            />
                          </TableCell>
                          <TableCell sx={{ py: 0.75 }}>
                            <Typography variant="caption" color="text.secondary">{r.pdno}</Typography>
                          </TableCell>
                        </TableRow>
                      ))}
                    </TableBody>
                  </Table>
                )}
              </Paper>
            )}

            {/* 검색어 입력됐으나 결과 없음 또는 종목 목록 비어있음 */}
            {showResults && !isFetchingSearch && searchQuery.length >= 2 && (searchResults.length === 0 || isStockListEmpty) && (
              <Box sx={{ mt: 0.5, px: 1.5, py: 1 }}>
                {isStockListEmpty ? (
                  <Stack direction="row" sx={{ alignItems: "center", flexWrap: "wrap" }} spacing={1}>
                    <Alert
                      severity="warning"
                      sx={{ flex: 1, py: 0.5, '& .MuiAlert-message': { display: 'flex', alignItems: 'center', gap: 1 } }}
                      action={
                        <Button
                          size="small"
                          color="warning"
                          variant="outlined"
                          onClick={() => doRefreshList()}
                          disabled={isRefreshing}
                          startIcon={isRefreshing ? <CircularProgress size={12} color="inherit" /> : undefined}
                          sx={{ whiteSpace: 'nowrap' }}
                        >
                          {isRefreshing ? '다운로드 중...' : '종목 목록 새로고침'}
                        </Button>
                      }
                    >
                      <Typography variant="caption">
                        종목 목록이 로드되지 않았습니다. KRX 서버 연결을 확인 후 새로고침을 눌러주세요.
                      </Typography>
                    </Alert>
                  </Stack>
                ) : (
                  <Typography variant="caption" color="text.secondary">
                    "{searchQuery}"에 대한 검색 결과가 없습니다
                  </Typography>
                )}
              </Box>
            )}
          </Box>
        ) : (
          <Stack direction="row" spacing={1} sx={{ alignItems: "flex-start" }}>
            <TextField
              label="티커 (AAPL, TSLA …)"
              value={inputValue}
              onChange={(e) => {
                const v = e.target.value.toUpperCase()
                setInputValue(v)
                if (!v) setSymbol('')
              }}
              onKeyDown={(e) => { if (e.key === 'Enter' && inputValue) void handleUsSearch() }}
              size="small"
              fullWidth
              slotProps={{ input: {
                endAdornment: (
                  <InputAdornment position="end">
                    {usSearching
                      ? <CircularProgress size={16} color="inherit" sx={{ mr: 0.5 }} />
                      : (
                        <IconButton
                          size="small"
                          onClick={() => void handleUsSearch()}
                          disabled={!inputValue}
                        >
                          <SearchIcon fontSize="small" />
                        </IconButton>
                      )
                    }
                  </InputAdornment>
                ),
              } }}
              helperText={isTossActive ? 'Enter 또는 검색 버튼 — Toss 시세와 주문 전 검증 조회' : 'Enter 또는 검색 버튼 — 나스닥·뉴욕·AMEX 자동 감지'}
            />
            {symbol && (
              <Chip
                label={isTossActive ? 'Toss' : usExchange}
                size="small"
                color={isTossActive ? 'secondary' : 'info'}
                variant="outlined"
                sx={{ mt: 1, flexShrink: 0 }}
              />
            )}
          </Stack>
        )}

        {symbol && isTossActive && (
          <Box sx={{ mt: 1.5, bgcolor: 'action.hover', borderRadius: 1 }}>
            <TossMarketCalendarStrip
              data={tossCalendar}
              isLoading={isTossCalendarLoading}
              isError={isTossCalendarError}
            />
            <TossMarketSnapshotCard symbol={symbol} />
            <TossStockSafetyCard
              data={tossSafety}
              isLoading={isTossSafetyLoading}
              isError={isTossSafetyError}
              error={tossSafetyError}
            />
          </Box>
        )}
        {symbol && !isTossActive && market === 'KR' && symbol.length === 6 && (
          <Box sx={{ mt: 1.5, bgcolor: 'action.hover', borderRadius: 1 }}>
            <KrStockInfoCard symbol={symbol} />
          </Box>
        )}
        {symbol && !isTossActive && market === 'US' && (
          <Box sx={{ mt: 1.5, bgcolor: 'action.hover', borderRadius: 1 }}>
            <UsStockInfoCard symbol={symbol} exchange={usExchange} />
          </Box>
        )}
      </Paper>

      {/* 3. 주문 + 차트 */}
      <Grid container spacing={2}>
        <Grid size={{ xs: 12, md: 4 }}>
          <TradingOrderForm model={model} />
        </Grid>

        <Grid size={{ xs: 12, md: 8 }}>
          <Paper sx={{ overflow: 'hidden' }}>
            {symbol && (market === 'KR' ? symbol.length === 6 : true) ? (
              <>
                {isTossActive
                  ? <TossMarketSnapshotCard symbol={symbol} />
                  : market === 'KR'
                  ? <KrStockInfoCard symbol={symbol} />
                  : <UsStockInfoCard symbol={symbol} exchange={usExchange} />}
                <Divider />
              </>
            ) : (
              <Box sx={{ px: 2.5, py: 2 }}>
                <Typography variant="body2" color="text.secondary">
                  위 검색창에서 종목을 선택하면 차트가 표시됩니다
                </Typography>
              </Box>
            )}
            {isTossActive && symbol ? (
              <Box sx={{ p: { xs: 1, sm: 2 } }}>
                <StockChart symbol={symbol} stockName={stockName} source="toss" />
              </Box>
            ) : market === 'US' && symbol ? (
              <Box sx={{ p: { xs: 1, sm: 2 } }}>
                <OverseasStockChart symbol={symbol} exchange={usExchange} stockName={stockName} />
              </Box>
            ) : (
              <Box sx={{ p: { xs: 1, sm: 2 } }}>
                <StockChart symbol={market === 'KR' ? symbol : ''} stockName={stockName} />
              </Box>
            )}
          </Paper>
        </Grid>
      </Grid>
    </Box>
  )
}
