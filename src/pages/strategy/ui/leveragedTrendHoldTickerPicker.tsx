import type { KeyboardEvent as ReactKeyboardEvent } from 'react'
import { useRef } from 'react'
import type { StockSearchItem } from '../../../api/types'
import AddIcon from '@mui/icons-material/Add'
import FlagIcon from '@mui/icons-material/Flag'
import PublicIcon from '@mui/icons-material/Public'
import SearchIcon from '@mui/icons-material/Search'
import Box from '@mui/material/Box'
import Button from '@mui/material/Button'
import Chip from '@mui/material/Chip'
import CircularProgress from '@mui/material/CircularProgress'
import IconButton from '@mui/material/IconButton'
import InputAdornment from '@mui/material/InputAdornment'
import Paper from '@mui/material/Paper'
import Stack from '@mui/material/Stack'
import Table from '@mui/material/Table'
import TableBody from '@mui/material/TableBody'
import TableCell from '@mui/material/TableCell'
import TableRow from '@mui/material/TableRow'
import TextField from '@mui/material/TextField'
import ToggleButton from '@mui/material/ToggleButton'
import ToggleButtonGroup from '@mui/material/ToggleButtonGroup'
import Typography from '@mui/material/Typography'
import { focusResultButton, isSearchEnter } from '../../../shared/lib'
import { RowSelectionButton } from '../../../shared/ui'
import type { LeveragedTrendHoldEditorModel, Market } from './useLeveragedTrendHoldEditorModel'

// Presentation only: the parent retains query/state ownership and callback closures.
export function LeveragedTrendHoldTickerPicker({ model }: { model: LeveragedTrendHoldEditorModel }) {
  const {
    canAddTarget, draftQuantity, handleAddTarget, handlePickerSelect,
    handlePickerUsSearch, pickerCloseTimer, pickerFetching, pickerInput,
    pickerMarket, pickerOpen, pickerResults, pickerSearching,
    pickerSelection, setDraftQuantity, setPickerError, setPickerInput,
    setPickerMarket, setPickerOpen, setPickerQuery, setPickerSelection,
    stratEnabled,
  } = model
  const pickerRegionRef = useRef<HTMLDivElement>(null)
  const pickerInputRef = useRef<HTMLInputElement>(null)
  const returningPickerFocus = useRef(false)
  const clearPickerCloseTimer = () => {
    if (pickerCloseTimer.current) clearTimeout(pickerCloseTimer.current)
    pickerCloseTimer.current = null
  }
  const returnPickerFocus = () => {
    if (stratEnabled) return
    returningPickerFocus.current = true
    try {
      pickerInputRef.current?.focus()
    } finally {
      returningPickerFocus.current = false
    }
  }
  const selectDomesticResult = (stock: StockSearchItem) => {
    if (stratEnabled) return
    clearPickerCloseTimer()
    handlePickerSelect(stock)
    returnPickerFocus()
  }
  const search = () => {
    if (pickerMarket === 'US') void handlePickerUsSearch()
    else if (pickerInput.trim().length >= 2) {
      setPickerQuery(pickerInput)
      setPickerOpen(true)
    }
  }
  return (
    <>
      <Stack direction={{ xs: 'column', md: 'row' }} spacing={1} sx={{ alignItems: { xs: 'stretch', md: 'center' } }}>
        <ToggleButtonGroup
          value={pickerMarket}
          exclusive
          onChange={(_, v) => { if (v) setPickerMarket(v as Market) }}
          size="small"
          disabled={stratEnabled}
          sx={{ flexShrink: 0 }}
        >
          <ToggleButton value="KR">
            <FlagIcon fontSize="small" sx={{ mr: 0.5 }} />국내
          </ToggleButton>
          <ToggleButton value="US">
            <PublicIcon fontSize="small" sx={{ mr: 0.5 }} />미국
          </ToggleButton>
        </ToggleButtonGroup>

        <Box
          ref={pickerRegionRef}
          sx={{ position: 'relative', flex: 1 }}
          onBlur={(event) => {
            if (pickerMarket !== 'KR' || event.currentTarget.contains(event.relatedTarget)) return
            clearPickerCloseTimer()
            pickerCloseTimer.current = setTimeout(() => setPickerOpen(false), 180)
          }}
          onFocus={clearPickerCloseTimer}
          onKeyDown={(event) => {
            if (pickerMarket !== 'KR' || event.key !== 'Escape') return
            event.preventDefault()
            clearPickerCloseTimer()
            setPickerOpen(false)
            setPickerQuery('')
            returnPickerFocus()
          }}
        >
          <TextField
            inputRef={pickerInputRef}
            label={pickerMarket === 'US' ? 'ETF 티커' : 'ETF 코드 또는 이름'}
            value={pickerInput}
            onChange={(e) => {
              const next = pickerMarket === 'US' ? e.target.value.toUpperCase() : e.target.value
              setPickerInput(next)
              setPickerSelection(null)
              setPickerError(null)
              if (pickerMarket === 'KR') setPickerOpen(next.length >= 2)
            }}
            onFocus={(event) => {
              clearPickerCloseTimer()
              if (!returningPickerFocus.current && pickerMarket === 'KR' && pickerInput.length >= 2
                && !pickerRegionRef.current?.contains(event.relatedTarget)) setPickerOpen(true)
            }}
            size="small"
            fullWidth
            disabled={stratEnabled}
            slotProps={{ htmlInput: {
              style: { textTransform: pickerMarket === 'US' ? 'uppercase' : 'none' },
              onKeyDown: (event: ReactKeyboardEvent<HTMLInputElement>) => {
                if (event.key !== 'Enter') return
                event.stopPropagation()
                if (!isSearchEnter(event)) return
                event.preventDefault()
                search()
              },
            }, input: {
              endAdornment: (
                <InputAdornment position="end">
                  {(pickerFetching || pickerSearching) && <CircularProgress size={14} color="inherit" aria-label="레버리지 대상 조회 중" sx={{ mr: 0.5 }} />}
                  <IconButton
                    size="small"
                    aria-label="레버리지 대상 검색"
                    disabled={stratEnabled || !pickerInput.trim() || (pickerMarket === 'KR' && pickerInput.trim().length < 2)}
                    onClick={search}
                    aria-disabled={stratEnabled || !pickerInput.trim() || (pickerMarket === 'KR' && pickerInput.trim().length < 2) || pickerSearching}
                    aria-busy={pickerFetching || pickerSearching}
                    aria-description={pickerFetching || pickerSearching ? '레버리지 대상 조회 중' : undefined}
                  >
                    <SearchIcon fontSize="small" />
                  </IconButton>
                </InputAdornment>
              ),
            } }}
          />

          {pickerMarket === 'KR' && pickerOpen && (pickerResults.length > 0 || pickerFetching) && (
            <Paper
              elevation={8}
              onMouseDown={(e) => {
                e.preventDefault()
                clearPickerCloseTimer()
              }}
              sx={{ mt: 0.5, maxHeight: 220, overflow: 'auto', border: 1, borderColor: 'divider', zIndex: 1400, position: 'absolute', width: '100%' }}
            >
              {pickerFetching && pickerResults.length === 0 ? (
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
                    {pickerResults.map((r) => (
                      <TableRow
                        key={r.pdno}
                        hover
                        sx={{ cursor: 'pointer' }}
                        onClick={() => selectDomesticResult(r)}
                      >
                        <TableCell sx={{ py: 0.75 }}>
                          <RowSelectionButton label={r.prdt_name} detail={r.pdno}
                            disabled={stratEnabled} onSelect={() => selectDomesticResult(r)} />
                        </TableCell>
                        <TableCell sx={{ py: 0.75, width: 90 }}>
                          <Typography variant="caption" color="text.secondary">{r.pdno}</Typography>
                        </TableCell>
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              )}
            </Paper>
          )}
        </Box>
      </Stack>

      <Stack direction={{ xs: 'column', md: 'row' }} spacing={1} sx={{ alignItems: { xs: 'stretch', md: 'center' } }}>
        <TextField
          label="1회 수량"
          type="number"
          value={draftQuantity}
          disabled={stratEnabled}
          size="small"
          onChange={(e) => setDraftQuantity(Math.max(1, Number(e.target.value)))}
          slotProps={{ htmlInput: { min: 1, step: 1 } }}
          sx={{ width: { xs: '100%', md: 120 } }}
        />
        <Button
          variant="contained"
          size="small"
          startIcon={<AddIcon />}
          disabled={!canAddTarget || stratEnabled}
          onClick={handleAddTarget}
        >
          대상 추가
        </Button>
        {pickerSelection && (
          <Chip
            size="small"
            color="primary"
            label={`${pickerSelection.stock.prdt_name} (${pickerSelection.stock.pdno})`}
            onDelete={stratEnabled ? undefined : () => {
              setPickerSelection(null)
              setPickerInput('')
              setPickerError(null)
            }}
            sx={{ maxWidth: '100%', '& .MuiChip-label': { overflow: 'hidden', textOverflow: 'ellipsis' } }}
          />
        )}
      </Stack>
    </>
  )
}
