import { useQueryClient } from '@tanstack/react-query'
import { KEYS } from '../../../api/queryKeys'
import { useLayoutEffect, useRef, useState } from 'react'
import * as cmd from '../../../api/commands'
import type { AppConfigView, LthResearchHandle, LthResearchReserveInput, LeveragedTrendHoldPreviewView } from '../../../api/types'

type Phase = 'idle' | 'reserving' | 'running' | 'cancelling' | 'cancelFailed' | 'failed'
type Task = {
  generation: number
  key: string
  cancelled: boolean
  handle?: LthResearchHandle
  cancellation?: Promise<void>
}
type Result = { generation: number; key: string; data: LeveragedTrendHoldPreviewView }

/** One editor owns one capability until completion or a confirmed quiescent cancel. */
export function useLthResearchJob(key: string) {
  const current = useRef({ key, generation: 0 })
  if (current.current.key !== key) {
    current.current = { key, generation: current.current.generation + 1 }
  }
  const renderGeneration = current.current.generation
  const queryClient = useQueryClient()
  const readScope = () => {
    const config = queryClient.getQueryData<AppConfigView>(KEYS.appConfig)
    return JSON.stringify([config?.active_profile_id, config?.active_broker_account_id, config?.active_broker_id, config?.kis_is_paper_trading])
  }
  const renderScope = readScope()
  const observedScope = useRef(renderScope)
  const mounted = useRef(true)
  const task = useRef<Task | null>(null)
  const [phase, setPhase] = useState<Phase>('idle')
  const [error, setError] = useState<string | null>(null)
  const [result, setResult] = useState<Result | null>(null)
  const showPhase = (value: Phase) => { if (mounted.current) setPhase(value) }
  const owns = (value: Task) => task.current === value
  const fresh = (value: Task) => mounted.current && !value.cancelled
    && value.key === current.current.key && value.generation === current.current.generation

  const cancelOwned = (value: Task): Promise<void> => {
    if (value.cancellation) return value.cancellation
    if (!value.handle) return Promise.resolve() // reserve continuation must cancel its late handle
    showPhase('cancelling')
    value.cancellation = (async () => {
      try {
        const ack = await cmd.cancelLthResearch(value.handle!)
        if (ack.status !== 'quiescent') throw new Error('invalid cancellation ACK')
        if (owns(value)) { task.current = null; showPhase('idle'); if (mounted.current) setError(null) }
      } catch {
        if (owns(value)) {
          showPhase('cancelFailed')
          if (mounted.current) setError('취소 확인 실패: 기존 작업의 중단을 확인할 때까지 새 연구를 실행할 수 없습니다. 취소를 다시 확인하세요.')
        }
      } finally { value.cancellation = undefined }
    })()
    return value.cancellation
  }
  const invalidate = () => {
    current.current.generation += 1
    if (mounted.current) setResult(null)
    const value = task.current
    if (!value) {
      if (mounted.current) setError(null)
      showPhase('idle')
      return
    }
    value.cancelled = true
    showPhase('cancelling')
    if (value.handle) void cancelOwned(value)
  }

  useLayoutEffect(() => queryClient.getQueryCache().subscribe(() => {
    const scope = readScope()
    if (scope !== observedScope.current) {
      observedScope.current = scope
      invalidate() // query cache A→B→A is fenced even before React renders B
    }
  }), [queryClient])

  useLayoutEffect(() => {
    const value = task.current
    if (value && !fresh(value)) invalidate()
    else if (!value) { setError(null); showPhase('idle') }
  }, [key]) // key changes are invalidated before browser events; refs also fence old closures
  useLayoutEffect(() => {
    mounted.current = true
    return () => { mounted.current = false; if (task.current) invalidate() }
  }, []) // this editor lifetime owns cancellation, including a still-pending reserve

  const start = async (input: LthResearchReserveInput) => {
    // A closure retained from an earlier render cannot start under the new key.
    if (key !== current.current.key || renderGeneration !== current.current.generation || renderScope !== readScope() || task.current || !mounted.current) return
    current.current.generation += 1
    const value: Task = { ...current.current, cancelled: false }
    task.current = value // same-task double clicks lock synchronously before await/render
    setResult(null); setError(null); showPhase('reserving')
    let failed = false
    try {
      value.handle = await cmd.reserveLthResearch(input)
      if (!fresh(value)) { value.cancelled = true; await cancelOwned(value); return }
      showPhase('running')
      const data = await cmd.runLthResearch(value.handle)
      if (fresh(value)) setResult({ generation: value.generation, key: value.key, data })
    } catch (failure) {
      if (fresh(value)) {
        failed = true
        const message = 'US 분봉 연구 예약·수집·계산에 실패했습니다. 실제 공급·발행·coverage는 미검증입니다.'
        // This specific server error is returned only after runtime terminal/drop.
        // A lost transport response proves no such completion; retain ownership
        // until cancellation confirms quiescence instead of dropping the handle.
        const terminal = typeof failure === 'object' && failure !== null
          && 'code' in failure && failure.code === 'LTH_RESEARCH_FAILED'
        if (value.handle && !terminal) {
          value.cancelled = true
          await cancelOwned(value)
          if (task.current === null && mounted.current && value.generation === current.current.generation && value.key === current.current.key) {
            showPhase('failed'); setError(message)
          }
        } else {
          showPhase('failed'); setError(message)
        }
      }
    } finally {
      if (owns(value)) {
        if (value.cancelled && value.handle) {
          // Failed cancellation deliberately retains the capability and lock.
          if (value.cancellation) await value.cancellation
        } else {
          task.current = null
          if (value.cancelled) showPhase('idle')
          else if (fresh(value) && !failed) showPhase('idle')
        }
      }
    }
  }
  return {
    start, invalidate,
    cancel: () => { invalidate() },
    phase,
    isPending: phase === 'reserving' || phase === 'running' || phase === 'cancelling',
    blocked: task.current !== null || phase === 'cancelFailed',
    error,
    data: result && result.key === current.current.key && result.generation === current.current.generation ? result.data : null,
  }
}
