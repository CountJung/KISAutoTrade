import { useState } from 'react'
import Alert from '@mui/material/Alert'
import Box from '@mui/material/Box'
import Button from '@mui/material/Button'
import Checkbox from '@mui/material/Checkbox'
import FormControlLabel from '@mui/material/FormControlLabel'
import Stack from '@mui/material/Stack'
import TextField from '@mui/material/TextField'
import Typography from '@mui/material/Typography'
import { useApproveBudgetRecovery, useBudgetRecovery } from '../../../api/budgetRecoveryHooks'
import type { RecoveryIdentity } from '../../../api/budgetRecoveryHooks'
import type { AutoTradingBudgetInput } from '../../../api/types'
import type { BudgetRecoveryItem } from '../../../shared/api/budgetRecoveryTypes'

function money(value: number, currency: 'KRW' | 'USD') {
  return new Intl.NumberFormat('ko-KR', { style: 'currency', currency }).format(currency === 'USD' ? value / 100 : value)
}

function RecoveryCard({ item, scope, identity, running, refreshing }: {
  item: BudgetRecoveryItem
  scope: AutoTradingBudgetInput
  identity: RecoveryIdentity
  running: boolean
  refreshing: boolean
}) {
  const resuming = Boolean(item.evidenceReference)
  const [providerId, setProviderId] = useState(item.evidenceReference ? item.providerOrderId ?? '' : '')
  const [clientId, setClientId] = useState(item.evidenceReference ? item.clientOrderId ?? '' : '')
  const [reference, setReference] = useState(item.evidenceReference ?? '')
  const [confirmed, setConfirmed] = useState(false)
  const approval = useApproveBudgetRecovery()
  const identityMatches = item.profileId === identity.profileId && item.isPaper === identity.isPaper
  const retryStored = item.state === 'awaitingReconciliation' && resuming
  const editable = (item.state === 'unresolved' || retryStored) && Boolean(item.snapshotRevision) && !item.blockedReason && identityMatches
  const disabled = !editable || running || refreshing || approval.isPending
  const valid = /^[A-Za-z0-9._:-]{1,128}$/.test(providerId.trim()) && reference.trim().length > 0
    && reference.trim().length <= 120 && new TextEncoder().encode(reference.trim()).length <= 256 && !/[\u0000-\u001f\u007f]/.test(reference)
    && (!item.clientOrderId || clientId.trim() === item.clientOrderId)
  const change = () => { setConfirmed(false); approval.reset() }
  return (
    <Box component="article" aria-label={`${item.symbol} 불명 주문 ${item.reservationId}`}
      sx={{ p: 2, border: 1, borderColor: 'divider', borderRadius: 1, minWidth: 0, overflowWrap: 'anywhere' }}>
      <Stack spacing={1.5}>
        <Typography fontWeight={700}>{item.symbol} · {item.side === 'buy' ? '매수' : '매도'} {item.quantity}주 · {item.currency}</Typography>
        <Typography variant="body2">예약 ID: {item.reservationId}</Typography>
        <Typography variant="body2">원래 프로파일: {item.profileId ?? '미확인'} · 시장: {item.market === 'kr' ? '국내' : item.market === 'us' ? '미국' : '미확인'} · 거래소: {item.exchange ?? '별도 코드 없음'}</Typography>
        <Typography variant="body2">제출 시각: {item.submittedAt || '기존 기록에 없음'} · {item.isPaper === null ? '투자 모드 미확인' : item.isPaper ? '모의' : '실전'}</Typography>
        <Typography variant="body2">주문 가격: {item.orderPrice === 0 ? '시장가' : money(item.orderPrice, item.currency)} · 예약액: {money(item.reservedAmount, item.currency)}</Typography>
        <Typography variant="body2">이미 반영한 체결: {item.filledQuantity}주 · 누적 대금: {money(item.filledNotional, item.currency)}</Typography>
        {item.clientOrderId && <Typography variant="body2">원래 clientOrderId: {item.clientOrderId}</Typography>}
        {item.providerOrderId && <Typography variant="body2">연결한 증권사 주문번호: {item.providerOrderId}</Typography>}
        {item.blockedReason && <Alert severity="warning">{item.blockedReason}</Alert>}
        {!identityMatches && <Alert severity="warning">원래 프로파일 또는 투자 모드와 다릅니다. 보류를 유지합니다.</Alert>}
        {item.state === 'awaitingReconciliation' && <Alert severity="info">미체결 대조 대상으로 연결됐습니다. 실제 체결·종결 확인과 예산 해제는 완료되지 않았습니다.</Alert>}
        {item.state === 'resolved' && <Alert severity="info">복구 승인 기록이 보존된 종결 항목입니다. 다시 연결하지 않습니다.</Alert>}
        {editable && <>
          {resuming && <Alert severity="info">저장된 대조 승인이 있습니다. 아래 원래 근거를 다시 확인한 뒤 같은 승인으로 미체결 복원을 재시도하세요.</Alert>}
          <TextField fullWidth label="대조한 증권사 주문번호" value={providerId} disabled={disabled || resuming}
            inputProps={{ maxLength: 128 }} onChange={(event) => { setProviderId(event.target.value); change() }} />
          {item.clientOrderId && <TextField fullWidth label="원본과 대조한 clientOrderId" value={clientId} disabled={disabled || resuming}
            inputProps={{ maxLength: 128 }} onChange={(event) => { setClientId(event.target.value); change() }} />}
          <TextField fullWidth label="대조 근거 참조" value={reference} disabled={disabled || resuming} inputProps={{ maxLength: 120 }}
            helperText="비민감 자료의 참조명만 적으세요. API 키·계좌 원문·응답 원문은 입력하지 마세요. 최대 120자·UTF-8 256바이트입니다."
            onChange={(event) => { setReference(event.target.value); change() }} />
          <FormControlLabel sx={{ alignItems: 'flex-start', m: 0 }} control={<Checkbox checked={confirmed} disabled={disabled || !valid}
            onChange={(event) => setConfirmed(event.target.checked)} />}
            label="원래 계좌·프로파일·투자 모드·시장·통화·종목·방향·수량·가격·제출 시각과 주문 식별자가 같은 주문임을 근거에서 대조했습니다." />
          <Button variant="outlined" disabled={disabled || !valid || !confirmed} onClick={() => {
            if (!item.snapshotRevision || disabled || !valid || !confirmed) return
            approval.mutate({ ...scope, reservationId: item.reservationId, snapshotRevision: item.snapshotRevision,
              providerOrderId: providerId.trim(), clientOrderId: item.clientOrderId ? clientId.trim() : null,
              evidenceReference: reference.trim(), confirmed: true })
          }}>{approval.isPending ? '대조 승인 저장 중…' : retryStored ? '같은 승인으로 미체결 저장 재시도' : '대조 승인 후 미체결 복원'}</Button>
        </>}
        {approval.error && <Alert severity="error">{approval.error.message || '복구 승인을 저장하지 못했습니다. 보류 상태를 다시 확인하세요.'}</Alert>}
        {approval.isSuccess && <Alert severity="success">대조 승인을 저장했습니다. 주문 재제출이나 체결·종결 확정은 수행하지 않았습니다.</Alert>}
      </Stack>
    </Box>
  )
}

export function BudgetRecoverySection({ scope, identity, running }: {
  scope: AutoTradingBudgetInput | undefined
  identity: RecoveryIdentity
  running: boolean
}) {
  const desktop = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
  const recovery = useBudgetRecovery(scope, identity, desktop)
  const sameScope = scope && recovery.data?.scope.brokerId === scope.brokerId && recovery.data.scope.accountId === scope.brokerAccountId
  return (
    <Stack spacing={2}>
      <Typography variant="subtitle1" fontWeight={700}>응답 불명 주문 대조·복구</Typography>
      <Typography variant="body2">예약을 자동 해제하거나 주문을 다시 제출하지 않습니다. 운영자가 원래 주문 근거를 대조한 뒤 미체결 처리 경로에 연결합니다. 이 승인만으로 증권사 사실 확인이나 체결 정산이 완료되지는 않습니다.</Typography>
      {!desktop ? <Alert severity="info">불명 주문 복구는 Tauri 데스크톱 앱에서만 사용할 수 있습니다.</Alert> : <>
        {!scope && <Alert severity="info">계좌 프로파일을 먼저 선택하세요.</Alert>}
        {scope && <Button variant="text" disabled={recovery.isFetching} onClick={() => { void recovery.refetch() }}>불명 주문 다시 조회</Button>}
        {recovery.isLoading && scope && <Typography>복구 기록을 불러오는 중…</Typography>}
        {recovery.error && <Alert severity="error">{recovery.error.message || '복구 기록을 불러오지 못했습니다.'}</Alert>}
        {running && <Alert severity="info">자동매매가 정지된 상태에서만 대조를 승인할 수 있습니다.</Alert>}
        {sameScope && recovery.data && <>
          <Typography variant="body2">전체 {recovery.data.totalCount}개 중 {recovery.data.items.length}개 표시</Typography>
          {recovery.data.totalCount > recovery.data.items.length && <Alert severity="warning">표시 한도를 넘는 기록이 있습니다. 표시되지 않은 예약도 보류 대상이며 누락을 해제로 해석하지 마세요.</Alert>}
          {recovery.data.items.length === 0 && <Typography>표시할 불명 주문·복구 승인 기록이 없습니다.</Typography>}
          {recovery.data.items.map((item) => <RecoveryCard key={`${scope.brokerId}:${scope.brokerAccountId}:${identity.profileId}:${identity.isPaper}:${item.reservationId}:${item.snapshotRevision}:${item.state}:${item.evidenceReference ?? ''}`}
            item={item} scope={scope} identity={identity} running={running} refreshing={recovery.isFetching || recovery.isError} />)}
        </>}
      </>}
    </Stack>
  )
}
