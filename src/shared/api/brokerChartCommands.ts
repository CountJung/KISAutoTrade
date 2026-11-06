import { invoke } from './transport'
import type { BrokerChartCollection, BrokerChartInput, BrokerChartReport, BrokerChartScope, CollectBrokerChartInput } from './brokerChartTypes'

/** Pure price research, no authenticated provider request. */
export const previewBrokerChartPortfolio = (input: BrokerChartInput) => invoke<BrokerChartReport>('preview_broker_chart_portfolio', { input })
/** Explicit normal-native scope lookup only; never scheduled on mount. */
export const getBrokerChartScope = () => invoke<BrokerChartScope>('get_broker_chart_scope')
/** Explicit authenticated chart request; unavailable in the research runtime. */
export const collectBrokerChartData = (input: CollectBrokerChartInput) => invoke<BrokerChartCollection>('collect_broker_chart_data', { input })
