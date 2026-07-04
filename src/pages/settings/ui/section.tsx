import type { ReactNode } from 'react'

import Paper from '@mui/material/Paper'
import Typography from '@mui/material/Typography'

export function Section({ title, children }: { title: string; children: ReactNode }) {
  return (
    <Paper sx={{ p: 3 }}>
      <Typography variant="subtitle1" fontWeight={600} mb={2}>
        {title}
      </Typography>
      {children}
    </Paper>
  )
}
