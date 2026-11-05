import Button from '@mui/material/Button'
import Typography from '@mui/material/Typography'

interface RowSelectionButtonProps {
  label: string
  detail: string
  onSelect: () => void
  disabled?: boolean
}

/** A native selection control inside a row; the row keeps its pointer action. */
export function RowSelectionButton({ label, detail, onSelect, disabled = false }: RowSelectionButtonProps) {
  return (
    <Button
      size="small"
      color="inherit"
      disabled={disabled}
      aria-label={`${label} (${detail}) 선택`}
      onClick={(event) => { event.stopPropagation(); onSelect() }}
      sx={{
        minWidth: 0,
        maxWidth: '100%',
        justifyContent: 'flex-start',
        textAlign: 'left',
        textTransform: 'none',
        px: 0.5,
        '&:focus-visible, &.Mui-focusVisible': {
          outline: '2px solid',
          outlineColor: 'primary.main',
          outlineOffset: -2,
        },
      }}
    >
      <Typography component="span" variant="body2" noWrap>{label}</Typography>
    </Button>
  )
}
