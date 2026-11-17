import { createContext, useContext, useEffect, useId, useRef, useState, type ReactNode, type KeyboardEvent } from 'react'
import { useQueryClient } from '@tanstack/react-query'
import { useRouter } from '@tanstack/react-router'
import { Alert, Button, Dialog, DialogActions, DialogContent, DialogContentText, DialogTitle, Typography } from '@mui/material'
import { useStopTrading, useActivateEmergencyStop } from '../../api/tradingHooks'
import { KEYS } from '../../api/queryKeys'
import type { AppConfigView } from '../../api/types'
import { BrokerScopeIndicator } from '../../shared/ui/BrokerScopeIndicator'

type Action = {
  title: string
  impact: string
  summary?: string
  execute: () => Promise<unknown>
  valid: () => boolean
  allowPaperStart?: boolean
}
type Ticket = Action & { config?: AppConfigView; scope: string; invalid: boolean; consumed: boolean; pending: boolean; error?: string }
type Controller = { request: (action: Action) => void; busy: boolean; invalidate: () => void }
const Context = createContext<Controller | null>(null)

/** No form/default/global submission; only an intentional focused button key. */
export function financialActionKeyDown(event: KeyboardEvent) {
  if ((event.key === 'Enter' || event.key === ' ') && (event.repeat || event.nativeEvent.isComposing
    || event.nativeEvent.keyCode === 229 || event.ctrlKey || event.metaKey || event.altKey || event.shiftKey)) {
    event.preventDefault(); event.stopPropagation()
  }
}

export function FinancialConfirmationProvider({ children }: { children: ReactNode }) {
  const qc = useQueryClient()
  const router = useRouter()
  const navigation = useRef(0)
  const stop = useStopTrading(), emergency = useActivateEmergencyStop()
  const quickPending = useRef({ stop: false, emergency: false })
  const active = useRef<Ticket | null>(null) // synchronous, shared across every entry point
  const opener = useRef<HTMLElement | null>(null)
  const [ticket, setTicket] = useState<Ticket | null>(null)
  const titleId = useId(), descriptionId = useId()
  const scope = () => {
    const state = qc.getQueryState<AppConfigView>(KEYS.appConfig), c = state?.data
    if (!state || state.status !== 'success' || state.fetchStatus !== 'idle' || state.isInvalidated
      || qc.isMutating({ mutationKey: ['account-profile-publication'] }) > 0
      || !c?.active_profile_id || !c.active_broker_id || !c.active_broker_account_id
      || !c.active_broker_configured || typeof c.kis_is_paper_trading !== 'boolean') return ''
    return JSON.stringify([navigation.current, state.dataUpdateCount, c.active_profile_id, c.active_profile_name,
      c.active_broker_id, c.active_broker_account_id, c.kis_is_paper_trading])
  }
  const valid = (t: Ticket) => !t.invalid && t.scope !== '' && t.scope === scope() && t.valid()
  const publish = (t: Ticket) => { if (active.current === t) setTicket({ ...t }) }
  const invalidate = () => {
    const t = active.current
    if (t && !t.consumed && !t.invalid && !valid(t)) { t.invalid = true; publish(t) }
  }
  // Observe transient publication/fetch/error and ABA even if React batches renders.
  useEffect(() => {
    const queryOff = qc.getQueryCache().subscribe(invalidate)
    const mutationOff = qc.getMutationCache().subscribe(invalidate)
    const navigationOff = router.subscribe('onBeforeNavigate', () => { navigation.current++; invalidate() })
    return () => { queryOff(); mutationOff(); navigationOff() }
  }, [qc, router])
  const execute = async (t: Ticket) => {
    if (active.current !== t || t.consumed || !valid(t)) {
      t.invalid = true; publish(t); return
    }
    t.consumed = true; t.pending = true; publish(t)
    try {
      await t.execute()
      if (active.current === t) { active.current = null; setTicket(null) }
    } catch (error) {
      t.error = error instanceof Error ? error.message
        : (error as { message?: string } | null)?.message ?? String(error)
    } finally { t.pending = false; publish(t) }
  }
  const request = (action: Action) => {
    if (active.current) return
    // The shared busy state disables the initiating button before Modal can
    // remember it. Capture it now and restore after controls are re-enabled.
    opener.current = document.activeElement instanceof HTMLElement ? document.activeElement : null
    stop.reset(); emergency.reset()
    const t: Ticket = { ...action, scope: scope(), config: qc.getQueryData(KEYS.appConfig),
      invalid: false, consumed: false, pending: false }
    active.current = t
    if (!valid(t)) t.invalid = true
    setTicket({ ...t })
    // Only the same trusted snapshot captured above may waive KIS paper start.
    // A previously rendered handler must never supply a stale bypass decision.
    if (action.allowPaperStart && t.config?.active_broker_id === 'kis'
      && t.config.kis_is_paper_trading === true && !t.invalid) void execute(t)
  }
  const close = () => {
    if (active.current?.pending) return
    active.current = null; setTicket(null)
  }
  const current = active.current
  const quickStop = (kind: 'stop' | 'emergency') => {
    if (quickPending.current[kind]) return
    quickPending.current[kind] = true
    if (current) { current.invalid = true; publish(current) }
    const operation = kind === 'stop' ? stop.mutateAsync() : emergency.mutateAsync()
    void operation.catch(() => { /* Existing mutation.error is displayed below. */ })
      .finally(() => { quickPending.current[kind] = false })
  }
  return <Context.Provider value={{ request, busy: ticket !== null, invalidate }}>
    {children}
    <Dialog open={ticket !== null} onClose={close} aria-labelledby={titleId} aria-describedby={descriptionId}
      disableRestoreFocus slotProps={{ transition: { onExited: () => {
        if (active.current) return
        const element = opener.current
        opener.current = null
        if (element?.isConnected) element.focus()
      } } }}
      fullWidth maxWidth="sm" onKeyDownCapture={event => {
        financialActionKeyDown(event)
        if ((event.key === 'Enter' || event.key === ' ') && event.target === event.currentTarget) event.preventDefault()
      }} onKeyDown={event => { if (event.key === 'Enter' || event.key === ' ') event.stopPropagation() }}>
      <DialogTitle id={titleId}>{ticket?.title}</DialogTitle>
      <DialogContent sx={{ overflowWrap: 'anywhere' }}>
        <BrokerScopeIndicator appConfig={ticket?.config} compact />
        <Typography sx={{ mt: 1, fontWeight: 700 }}>{ticket?.config?.active_broker_id === 'toss' ? '실전투자' : ticket?.config?.kis_is_paper_trading ? '모의투자' : '실전투자'}</Typography>
        <DialogContentText id={descriptionId} sx={{ mt: 1, whiteSpace: 'pre-line' }}>{ticket?.summary}{ticket?.summary ? '\n' : ''}{ticket?.impact}</DialogContentText>
        <Typography variant="caption" component="p" sx={{ mt: 1 }}>정지는 접수 주문 취소나 포지션 청산이 아닙니다. 아래 정지 요청은 즉시 전송되며 이미 전송한 요청을 취소하지 않습니다.</Typography>
        {current && !current.consumed && !valid(current) && <Alert severity="warning" sx={{ mt: 2 }}>계좌·모드·프로필 공개 상태 또는 주문 입력이 바뀌었거나 미확인입니다. 취소 후 다시 확인하세요.</Alert>}
        {ticket?.pending && <Alert severity="info" sx={{ mt: 2 }}>요청 처리 중입니다. 중복 실행할 수 없습니다.</Alert>}
        {(stop.error || emergency.error) && <Alert severity="error" sx={{ mt: 2 }}>정지 요청 실패: {String((stop.error || emergency.error)?.message ?? stop.error ?? emergency.error)}</Alert>}
        {ticket?.error && <Alert severity="error" sx={{ mt: 2 }}>{ticket.error} — 반영 여부를 현재 상태에서 확인하세요. 닫은 뒤 새 확인으로 명시적으로 재시도하세요.</Alert>}
      </DialogContent>
      <DialogActions sx={{ flexWrap: 'wrap', gap: 1, px: 3 }}>
        <Button type="button" color="error" onKeyDown={financialActionKeyDown} disabled={stop.isPending}
          onClick={() => quickStop('stop')}>자동매매 즉시 정지</Button>
        <Button type="button" color="error" onKeyDown={financialActionKeyDown} disabled={emergency.isPending}
          onClick={() => quickStop('emergency')}>비상정지 즉시 발동</Button>
      </DialogActions>
      <DialogActions sx={{ flexWrap: 'wrap', gap: 1 }}>
        <Button type="button" autoFocus onClick={close} disabled={ticket?.pending}>{ticket?.error ? '닫기' : '취소'}</Button>
        <Button type="button" variant="contained" color="warning" disabled={!current || current.consumed || !valid(current)}
          onClick={() => { if (current) void execute(current) }}>최종 확인</Button>
      </DialogActions>
    </Dialog>
  </Context.Provider>
}

/** Local draft generation invalidates A→B→A and unmounted action closures. */
export function useFinancialConfirmation(draft = '') {
  const context = useContext(Context)
  if (!context) throw new Error('FinancialConfirmationProvider is required')
  const version = useRef({ draft, generation: 0, mounted: true })
  if (version.current.draft !== draft) { version.current.draft = draft; version.current.generation++ }
  useEffect(() => { version.current.mounted = true; return () => { version.current.mounted = false; context.invalidate() } }, [])
  useEffect(() => { context.invalidate() }, [draft, context])
  return {
    busy: context.busy,
    request: (action: Omit<Action, 'valid'>) => {
      const generation = version.current.generation
      context.request({ ...action, valid: () => version.current.mounted && version.current.generation === generation })
    },
  }
}
