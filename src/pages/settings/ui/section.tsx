import type { ReactNode } from 'react'

import Paper from '@mui/material/Paper'
import Stack from '@mui/material/Stack'
import Typography from '@mui/material/Typography'

import { SettingsContentsLink, settingsHeadingFocus } from './settingsNavigation'
import type { SettingsSectionId } from './settingsNavigation'

export function Section({ id, title, children }: { id: SettingsSectionId; title: string; children: ReactNode }) {
  return (
    <Paper component="section" aria-labelledby={id} sx={{ p: 3 }}>
      <Stack direction="row" useFlexGap sx={{ alignItems: 'baseline', justifyContent: 'space-between', flexWrap: 'wrap', gap: 1, mb: 2 }}>
        <Typography id={id} component="h2" variant="subtitle1" tabIndex={-1}
          sx={{ ...settingsHeadingFocus, fontWeight: 600 }}>
          {title}
        </Typography>
        <SettingsContentsLink title={title} />
      </Stack>
      {children}
    </Paper>
  )
}
