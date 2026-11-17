import { invoke } from './transport'
import type { CollectSectorDataInput, SectorDataBundle, SectorDataValidation, SectorQuoteScope, ValidateSectorDataInput } from './sectorDataTypes'

/** Explicit native-only scope lookup, never run on mount. */
export const getSectorQuoteScope = () => invoke<SectorQuoteScope>('get_sector_quote_scope')
/** Authenticated read-only quotes; native normal runtime only. */
export const collectSectorData = (input: CollectSectorDataInput) => invoke<SectorDataBundle>('collect_sector_data', { input })
/** Pure local bundle validation; no provider request. */
export const validateSectorData = (input: ValidateSectorDataInput) => invoke<SectorDataValidation>('validate_sector_data', { input })
