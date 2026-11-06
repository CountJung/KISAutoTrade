import type { BrokerChartInput, BrokerChartReport } from '../../../src/shared/api/brokerChartTypes'
import { brokerChartResultsView } from '../../../src/widgets/sector-portfolio/model/brokerChartInput'
import { fixtureReport } from './sectorPortfolioFixture'

/** Fixed synthetic UI response; does not simulate or verify Rust investment results. */
export function brokerChartFixtureReport(input: BrokerChartInput): BrokerChartReport {
  const result = fixtureReport(brokerChartResultsView(input))
  result.inputHash = 'c'.repeat(64)
  return { mode: 'brokerChartPriceApproximation', signalBasis: 'unadjustedClose', dividendReturnIncluded: false,
    economicReturnVerified: false, inputHash: 'c'.repeat(64), warnings: result.warnings, result }
}
