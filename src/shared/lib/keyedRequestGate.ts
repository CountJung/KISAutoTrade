import { useEffect, useRef, useState } from 'react'

declare const ticketBrand: unique symbol
export type RequestTicket = { readonly [ticketBrand]: true }

// Opaque key only: callers retain validation, network, broker and UI result callbacks.
export function createKeyedRequestGate(initialKey: string) {
  let key = initialKey
  let epoch = 0
  let active = true
  let pending: { ticket: RequestTicket; epoch: number } | null = null
  const invalidate = () => { epoch++; pending = null }
  const current = (ticket: RequestTicket | null) => active && ticket !== null
    && pending !== null && pending.ticket === ticket && pending.epoch === epoch
  return {
    setKey(nextKey: string) { if (nextKey !== key) { key = nextKey; invalidate() } },
    activate() { active = true },
    dispose() { active = false; invalidate() },
    invalidate,
    pending: () => active && pending !== null,
    current,
    begin(): RequestTicket | null {
      if (!active || pending) return null
      const ticket = {} as RequestTicket
      pending = { ticket, epoch }
      return ticket
    },
    finish(ticket: RequestTicket): boolean {
      if (!current(ticket)) return false
      pending = null
      return true
    },
  }
}

export function useKeyedRequestGate(key: string) {
  const [, render] = useState(0)
  const ref = useRef<ReturnType<typeof createKeyedRequestGate> | null>(null)
  if (!ref.current) ref.current = createKeyedRequestGate(key)
  const gate = ref.current
  // Invalidate synchronously, including A -> B -> A, before an old Promise can apply.
  gate.setKey(key)
  useEffect(() => {
    gate.activate() // StrictMode effect re-entry must accept new tickets.
    return () => gate.dispose()
  }, [gate])
  return {
    current: gate.current,
    pending: gate.pending,
    begin() {
      const ticket = gate.begin()
      if (ticket) render(revision => revision + 1)
      return ticket
    },
    finish(ticket: RequestTicket) {
      const finished = gate.finish(ticket)
      if (finished) render(revision => revision + 1)
      return finished
    },
    invalidate() { gate.invalidate(); render(revision => revision + 1) },
  }
}
