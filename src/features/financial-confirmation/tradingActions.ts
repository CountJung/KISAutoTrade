import { useStartTrading, useClearBuySuspension, useClearEmergencyStop } from '../../api/tradingHooks'
import { useFinancialConfirmation } from './index'

export function useConfirmedStartTrading() {
  const mutation = useStartTrading()
  const flow = useFinancialConfirmation()
  return { isPending: mutation.isPending || flow.busy, mutate: () => flow.request({
    title: '자동매매 시작 확인', impact: '선택 계좌의 자동매매를 시작합니다. 전략과 기존 위험·예산 guard에 따라 실제 주문이 제출될 수 있습니다.',
    allowPaperStart: true,
    execute: () => mutation.mutateAsync(),
  }) }
}

export function useConfirmedBuyResume() {
  const mutation = useClearBuySuspension()
  const flow = useFinancialConfirmation()
  return { isPending: mutation.isPending || flow.busy, mutate: () => flow.request({
    title: '매수 재개 확인', impact: '매수 정지와 사유를 해제합니다. 입금이나 잔고 검증이 아니며 비상정지를 해제하거나 자동매매를 시작하지 않습니다. 기존 guard는 유지됩니다.',
    execute: () => mutation.mutateAsync(),
  }) }
}

export function useConfirmedEmergencyRelease() {
  const mutation = useClearEmergencyStop()
  const flow = useFinancialConfirmation()
  return { isPending: mutation.isPending || flow.busy, mutate: () => flow.request({
    title: '비상정지 해제 확인', impact: '비상정지 flag를 해제합니다. 자동매매 시작이나 손실 카운터 초기화가 아닙니다. 손실 한도·매수 정지와 기존 guard는 유지됩니다.',
    execute: () => mutation.mutateAsync(),
  }) }
}
