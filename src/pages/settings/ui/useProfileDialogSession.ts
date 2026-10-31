import { useRef } from 'react'
import { useKeyedRequestGate } from '../../../shared/lib'

// UI lifetime only. Values, credentials, validation, mutations and close/focus callbacks stay with callers.
export function useProfileDialogSession(open: boolean, identity: unknown) {
  const session = useRef({ open, identity, epoch: 0, enabled: open })
  if (session.current.open !== open || !Object.is(session.current.identity, identity)) {
    session.current = { open, identity, epoch: session.current.epoch + 1, enabled: open }
  }
  const epoch = session.current.epoch
  const key = String(epoch)
  const main = useKeyedRequestGate(key)
  const detect = useKeyedRequestGate(key)
  const lookup = useKeyedRequestGate(key)
  const guard = (gate: ReturnType<typeof useKeyedRequestGate>) => ({
    ...gate,
    begin: () => session.current.enabled && session.current.epoch === epoch ? gate.begin() : null,
  })
  const invalidateAux = () => {
    detect.invalidate()
    lookup.invalidate()
  }
  return {
    main: guard(main),
    detect: guard(detect),
    lookup: guard(lookup),
    invalidateAux,
    close() {
      // Reject blur/queued activation during close, even before parent open=false commits.
      session.current.enabled = false
      main.invalidate()
      invalidateAux()
    },
  }
}
