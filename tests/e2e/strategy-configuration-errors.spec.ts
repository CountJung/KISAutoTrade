import { expect, test } from '@playwright/test'
import { mockApi } from './fixtures/strategyResearchFixture'

const failures = [
  { code: 'CONFIGURATION_BUSY', status: 409, message: '전략 신호를 처리 중입니다. 주문 상태가 확정된 뒤 다시 저장하세요.' },
  { code: 'HELD_TARGET_REMOVAL', status: 409, message: '보유 종목은 전략 대상에서 제거할 수 없습니다.' },
  { code: 'HELD_MARKET_CHANGE', status: 409, message: '보유 종목의 국내·해외 구분은 변경할 수 없습니다.' },
  { code: 'PERSISTENCE_ERROR', status: 500, message: '전략 설정을 저장하지 못했습니다.' },
]

for (const failure of failures) {
  test(`Strategy displays ${failure.code}, retains the draft and allows retry`, async ({ page }) => {
    await mockApi(page)
    const requests: unknown[] = []
    await page.route('**/api/strategies/ma_cross_default', async (route) => {
      requests.push(route.request().postDataJSON())
      await route.fulfill({
        status: failure.status,
        json: { code: failure.code, error: failure.message },
      })
    })
    await page.goto('/strategy')
    const card = page.locator('.MuiPaper-root').filter({ hasText: 'MovingAverageCrossStrategy' }).first()
    const quantity = card.getByLabel('1회 수량')
    const save = card.getByRole('button', { name: '변경사항 저장' })
    await quantity.fill('2')
    await save.click()
    await expect(page.getByRole('alert').filter({ hasText: failure.message })).toBeVisible()
    await expect(quantity).toHaveValue('2')
    await expect(save).toBeEnabled()
    await save.click()
    await expect.poll(() => requests.length).toBe(2)
    expect(requests[1]).toMatchObject({
      expectedProfileId: 'paper', expectedBrokerId: 'kis',
      expectedBrokerAccountId: '12345678-01', orderQuantity: 2,
    })
    await page.reload()
    await expect(quantity).toHaveValue('1')
  })
}
