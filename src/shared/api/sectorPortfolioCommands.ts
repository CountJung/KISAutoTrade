import { invoke } from './transport'
import type { SectorPortfolioInput, SectorPortfolioReport } from './sectorPortfolioTypes'

/** Pure calculation: never invokes a broker, account, or trading command. */
export const previewSectorPortfolio = (input: SectorPortfolioInput) =>
  invoke<SectorPortfolioReport>('preview_sector_portfolio', { input })
