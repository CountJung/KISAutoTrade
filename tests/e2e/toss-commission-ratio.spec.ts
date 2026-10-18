import { expect, test } from '@playwright/test'

test('official Toss commission ratio is displayed as a percentage beside the corrected cash requirement', async ({ page }) => {
  await page.route('http://127.0.0.1:1430/api/**', route => route.abort())
  await page.route('**/src/main.tsx', async route => {
    const response = await route.fetch()
    const entry = await response.text()
    const dependency = (name: string) => {
      const path = [...entry.matchAll(/(?:from|import)\s*["']([^"']+)["']/g)]
        .map(match => match[1]).find(path => path.includes(`/deps/${name}.js`))
      if (!path) throw new Error(`Vite dependency missing: ${name}`)
      return JSON.stringify(path)
    }
    await route.fulfill({ response, contentType: 'text/javascript', body: `
      import React from ${dependency('react')};
      import ReactDOM from ${dependency('react-dom_client')};
      import { TossOrderPreflightPanel } from '/src/pages/trading/ui/tossPanels.tsx';
      const data = {
        brokerId: 'toss', accountSeq: 'synthetic-account', symbol: '005930', market: 'kr', side: 'buy',
        quantity: '10', price: {amount:'1000',currency:'KRW'}, priceSource: 'input',
        buyingPower: {amount:'10001',currency:'KRW'}, sellableQuantity: null, commissionRate: '0.00015',
        grossAmount: {amount:'10000',currency:'KRW'}, estimatedCommission: {amount:'2',currency:'KRW'},
        requiredCash: {amount:'10002',currency:'KRW'}, liquidityOk:false, safetyOk:true,
        orderAdapterSupported:true, canSubmit:false, blockedReasons:['매수가능금액 부족'], warnings:[]
      };
      ReactDOM.createRoot(document.getElementById('root')).render(
        React.createElement(TossOrderPreflightPanel, { data, isLoading:false, isError:false, error:null })
      );
    ` })
  })
  await page.goto('/')
  await expect(page.getByText('수수료율 0.015% · 추정 수수료 2원')).toBeVisible()
  await expect(page.getByText('주문금액 10,000원 · 필요 10,002원')).toBeVisible()
  await expect(page.getByText('가능금액 부족', { exact: true })).toBeVisible()
  await expect(page.getByText('매수가능금액 부족', { exact: true })).toBeVisible()
})
