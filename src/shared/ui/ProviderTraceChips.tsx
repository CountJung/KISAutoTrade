import Box from '@mui/material/Box'
import Chip from '@mui/material/Chip'
import Tooltip from '@mui/material/Tooltip'

export interface ProviderTrace {
  provider?: string | null
  providerOrderId?: string | null
  providerRequestId?: string | null
  providerTrId?: string | null
}

const TRACE_PATTERN =
  /\b(?:provider=(?<provider>[A-Za-z0-9_-]+)|tr_id=(?<trId>[A-Z0-9_-]+)|TR-ID[:=](?<trIdAlt>[A-Z0-9_-]+)|odno=(?<orderId>[A-Za-z0-9_-]+)|order_id=(?<orderIdAlt>[A-Za-z0-9_-]+)|request_id=(?<requestId>[A-Za-z0-9_.:-]+)|requestId=(?<requestIdAlt>[A-Za-z0-9_.:-]+)|X-Request-Id[:=](?<requestHeaderId>[A-Za-z0-9_.:-]+))/g

function shortValue(value: string) {
  return value.length > 18 ? `${value.slice(0, 8)}...${value.slice(-6)}` : value
}

function chip(label: string, value: string, title: string) {
  return (
    <Tooltip key={`${label}:${value}`} title={`${title}: ${value}`} arrow>
      <Chip
        label={`${label} ${shortValue(value)}`}
        size="small"
        variant="outlined"
        sx={{ height: 22, maxWidth: 180, '& .MuiChip-label': { px: 0.75 } }}
      />
    </Tooltip>
  )
}

export function parseProviderTraceText(message: string): ProviderTrace {
  const trace: ProviderTrace = {}

  for (const match of message.matchAll(TRACE_PATTERN)) {
    const groups = match.groups ?? {}
    trace.provider ??= groups.provider
    trace.providerTrId ??= groups.trId ?? groups.trIdAlt
    trace.providerOrderId ??= groups.orderId ?? groups.orderIdAlt
    trace.providerRequestId ??= groups.requestId ?? groups.requestIdAlt ?? groups.requestHeaderId
  }

  return trace
}

export function hasProviderTrace(trace: ProviderTrace) {
  return Boolean(
    trace.provider ||
      trace.providerOrderId ||
      trace.providerRequestId ||
      trace.providerTrId
  )
}

export function ProviderTraceChips({ trace }: { trace: ProviderTrace }) {
  if (!hasProviderTrace(trace)) return null

  return (
    <Box sx={{ display: 'flex', gap: 0.5, flexWrap: 'wrap', alignItems: 'center' }}>
      {trace.provider && chip('Provider', trace.provider, 'Provider')}
      {trace.providerTrId && chip('TR', trace.providerTrId, 'TR-ID')}
      {trace.providerOrderId && chip('Order', trace.providerOrderId, 'Provider order ID')}
      {trace.providerRequestId && chip('Req', trace.providerRequestId, 'Provider request ID')}
    </Box>
  )
}
