import { useMutation } from '@tanstack/react-query'
import { previewSectorPortfolio } from '../shared/api/sectorPortfolioCommands'
import type { SectorPortfolioInput, SectorPortfolioReport } from '../shared/api/sectorPortfolioTypes'
import type { CmdError } from './types'

export function usePreviewSectorPortfolio() {
  return useMutation<SectorPortfolioReport, CmdError, SectorPortfolioInput>({
    mutationFn: previewSectorPortfolio,
    retry: false,
    gcTime: 0,
  })
}
