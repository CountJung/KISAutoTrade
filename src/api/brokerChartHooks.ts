import { useMutation } from '@tanstack/react-query'
import { collectBrokerChartData, getBrokerChartScope, previewBrokerChartPortfolio } from '../shared/api/brokerChartCommands'

const options = { retry: false as const, gcTime: 0 }
export const usePreviewBrokerChartPortfolio = () => useMutation({ mutationFn: previewBrokerChartPortfolio, ...options })
export const useBrokerChartScope = () => useMutation({ mutationFn: getBrokerChartScope, ...options })
export const useCollectBrokerChartData = () => useMutation({ mutationFn: collectBrokerChartData, ...options })
