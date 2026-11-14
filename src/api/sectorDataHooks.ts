import { useMutation } from '@tanstack/react-query'
import { collectSectorData, getSectorQuoteScope, validateSectorData } from '../shared/api/sectorDataCommands'

export function useSectorQuoteScope() { return useMutation({ mutationFn: getSectorQuoteScope, retry: false, gcTime: 0 }) }
export function useCollectSectorData() { return useMutation({ mutationFn: collectSectorData, retry: false, gcTime: 0 }) }
export function useValidateSectorData() { return useMutation({ mutationFn: validateSectorData, retry: false, gcTime: 0 }) }
