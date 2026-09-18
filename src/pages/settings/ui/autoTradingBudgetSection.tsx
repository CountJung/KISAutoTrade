import { useState } from 'react'
import Alert from '@mui/material/Alert'
import Box from '@mui/material/Box'
import Button from '@mui/material/Button'
import Stack from '@mui/material/Stack'
import TextField from '@mui/material/TextField'
import Typography from '@mui/material/Typography'
import { useAppConfig, useAutoTradingBudget, useTradingStatus, useUpdateAutoTradingBudget } from '../../../api/hooks'
import type { AutoTradingBudgetInput, AutoTradingBudgetView, BudgetCurrencyView } from '../../../api/types'
import { Section } from './section'

function amount(text: string, decimals: number): number | undefined {
  if (!(decimals ? /^\d+(\.\d{1,2})?$/ : /^\d+$/).test(text)) return undefined
  const [whole, fraction = ''] = text.split('.')
  const value = Number(whole) * 10 ** decimals + Number(fraction.padEnd(decimals, '0'))
  return Number.isSafeInteger(value) && value >= 0 ? value : undefined
}

function CurrencyStatus({ value, currency }: { value: BudgetCurrencyView; currency: 'KRW' | 'USD' }) {
  const display = (n: number) => new Intl.NumberFormat('ko-KR', { style: 'currency', currency }).format(currency === 'USD' ? n / 100 : n)
  return (
    <Box sx={{ p: 2, bgcolor: 'action.hover', borderRadius: 1, minWidth: 0, flex: 1 }}>
      <Typography fontWeight={600}>{currency === 'KRW' ? '원화' : '달러'}</Typography>
      <Typography variant="body2">배정액: {display(value.allocatedAmount)}</Typography>
      <Typography variant="body2">전용 현금: {display(value.cashAmount)}</Typography>
      <Typography variant="body2">주문 예약액: {display(value.reservedAmount)}</Typography>
      <Typography variant="body2" fontWeight={700}>사용 가능액: {display(value.availableAmount)}</Typography>
      <Typography variant="caption" color="text.secondary">자동매매 보유 {value.ownedPositionCount}종목 · 비용 여유분 {value.feeBufferBps / 100}%</Typography>
      {value.blockedReason && <Alert severity="error" sx={{ mt: 1 }}>{value.blockedReason}</Alert>}
    </Box>
  )
}

function BudgetEditor({ scope, data, running }: { scope: AutoTradingBudgetInput; data: AutoTradingBudgetView; running: boolean }) {
  const [draft, setDraft] = useState<{ krw: string; usd: string } | null>(null)
  const update = useUpdateAutoTradingBudget()
  const krw = draft?.krw ?? String(data.krw.allocatedAmount)
  const usd = draft?.usd ?? (data.usd.allocatedAmount / 100).toFixed(2)
  const krwAmount = amount(krw, 0)
  const usdAmount = amount(usd, 2)
  const disabled = running || update.isPending
  return (
    <Stack spacing={2}>
      <Stack direction={{ xs: 'column', sm: 'row' }} spacing={2}>
        <CurrencyStatus value={data.krw} currency="KRW" />
        <CurrencyStatus value={data.usd} currency="USD" />
      </Stack>
      <Typography variant="body2" color="text.secondary">
        계좌 안에서 자동매매에 사용할 금액을 구분합니다. 실제 이체나 증권사 자금 잠금은 이루어지지 않습니다.
        원화와 달러는 서로 사용하지 않으며, 0으로 설정한 통화는 신규 자동매수가 차단됩니다.
      </Typography>
      <Typography variant="body2" color="text.secondary">
        자동매수는 지정가와 비용 여유분 안에서만 접수됩니다. 매도 대금은 전용 현금으로 돌아오며 손익에 따라 사용 가능액이 변합니다.
        기존 수동 보유 종목은 자동매매가 매도하지 않습니다.
      </Typography>
      <Stack direction={{ xs: 'column', sm: 'row' }} spacing={2}>
        <TextField fullWidth label="자동매매 원화 배정액 (원)" value={krw} disabled={disabled}
          inputProps={{ inputMode: 'numeric' }} error={krwAmount === undefined}
          helperText={krwAmount === undefined ? '0 이상의 정수 금액을 입력하세요.' : '원 단위'}
          onChange={(event) => setDraft({ krw: event.target.value, usd })} />
        <TextField fullWidth label="자동매매 달러 배정액 (USD)" value={usd} disabled={disabled}
          inputProps={{ inputMode: 'decimal' }} error={usdAmount === undefined}
          helperText={usdAmount === undefined ? '소수 둘째 자리까지 입력하세요.' : '달러 단위 (예: 100.50)'}
          onChange={(event) => setDraft({ krw, usd: event.target.value })} />
      </Stack>
      {running && <Alert severity="info">자동매매를 정지한 후 배정액을 변경할 수 있습니다.</Alert>}
      {update.error && <Alert severity="error">{update.error.message || '예산 저장에 실패했습니다.'}</Alert>}
      {update.isSuccess && !draft && <Alert severity="success">자동매매 예산을 저장했습니다.</Alert>}
      <Button variant="contained" disabled={disabled || !draft || krwAmount === undefined || usdAmount === undefined}
        onClick={() => {
          if (krwAmount === undefined || usdAmount === undefined) return
          update.mutate({ ...scope, krwAmount, usdAmount }, { onSuccess: () => setDraft(null) })
        }}>
        {update.isPending ? '저장 중…' : '자동매매 예산 저장'}
      </Button>
    </Stack>
  )
}

export function AutoTradingBudgetSection() {
  const { data: config } = useAppConfig()
  const { data: trading } = useTradingStatus()
  const scope = config?.active_broker_account_id
    ? { brokerId: config.active_broker_id, brokerAccountId: config.active_broker_account_id }
    : undefined
  const budget = useAutoTradingBudget(scope)
  return (
    <Section title="자동매매 전용 예산">
      <Stack spacing={2}>
        <Typography variant="body2">선택 계좌: {config?.active_profile_name || '미선택'}</Typography>
        {!scope && <Alert severity="info">계좌 프로파일을 먼저 선택하세요.</Alert>}
        {scope && budget.isLoading && <Typography>예산을 불러오는 중…</Typography>}
        {budget.error && <Alert severity="error">{budget.error.message || '예산을 불러오지 못했습니다.'}</Alert>}
        {scope && budget.data && budget.data.scope?.brokerId === scope.brokerId && budget.data.scope?.accountId === scope.brokerAccountId &&
          <BudgetEditor key={`${scope.brokerId}:${scope.brokerAccountId}`} scope={scope} data={budget.data} running={trading?.isRunning !== false} />}
      </Stack>
    </Section>
  )
}
