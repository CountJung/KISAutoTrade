import type { KeyboardEvent as ReactKeyboardEvent } from 'react'
import Box from '@mui/material/Box'
import CircularProgress from '@mui/material/CircularProgress'
import Paper from '@mui/material/Paper'
import Table from '@mui/material/Table'
import TableBody from '@mui/material/TableBody'
import TableCell from '@mui/material/TableCell'
import TableRow from '@mui/material/TableRow'
import Typography from '@mui/material/Typography'
import type { StockSearchItem } from '../../../api/types'
import { RowSelectionButton } from '../../../shared/ui'
import { focusResultButton } from '../../../shared/lib'

interface StockSearchResultsProps {
  open: boolean
  items: StockSearchItem[]
  loading: boolean
  onPointerInteraction: () => void
  onSelect: (item: StockSearchItem) => void
}

/** Route-local result display; query, selection and strategy updates stay in Page. */
export function StockSearchResults({ open, items, loading, onPointerInteraction, onSelect }: StockSearchResultsProps) {
  if (!open || (items.length === 0 && !loading)) return null
  return (
    <Paper
      elevation={8}
      onMouseDown={(event) => { event.preventDefault(); onPointerInteraction() }}
      sx={{ mt: 0.5, maxHeight: 240, overflow: 'auto', border: 1, borderColor: 'divider', zIndex: 1400, position: 'absolute', width: '100%' }}
    >
      {loading && items.length === 0 ? (
        <Box sx={{ p: 1.5, display: 'flex', alignItems: 'center', gap: 1 }}>
          <CircularProgress size={14} />
          <Typography variant="caption" color="text.secondary">검색 중...</Typography>
        </Box>
      ) : (
        <Table size="small">
          <TableBody onKeyDown={(event: ReactKeyboardEvent<HTMLTableSectionElement>) => {
            if (event.altKey || event.ctrlKey || event.metaKey || event.shiftKey) return
            if (!focusResultButton(event.currentTarget, event.target, event.key)) return
            event.preventDefault()
            event.stopPropagation()
          }}>
            {items.map((item) => (
              <TableRow key={item.pdno} hover sx={{ cursor: 'pointer' }} onClick={() => onSelect(item)}>
                <TableCell sx={{ py: 0.75, maxWidth: { xs: 180, sm: 400 } }}>
                  <RowSelectionButton label={item.prdt_name} detail={item.pdno} onSelect={() => onSelect(item)} />
                </TableCell>
                <TableCell sx={{ py: 0.75, width: 80 }}>
                  <Typography variant="caption" color="text.secondary">{item.pdno}</Typography>
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      )}
    </Paper>
  )
}
