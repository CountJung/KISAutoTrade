# Toss Read-Only And Small-Order Verification Checklist

> Source of truth: `https://openapi.tossinvest.com/openapi-docs/latest/openapi.json`  
> Last verified: 2026-07-03, OpenAPI version `1.1.5`

This checklist prevents accidental live trading while adding Toss Securities support. Do not connect Toss order APIs to manual order buttons or auto-trading until every read-only step passes and the user gives explicit approval for a small live-order test.

## 0. Preconditions

- Do not read or paste `.env`, `secure_config.json`, or `profiles.json` into agent context.
- Run `npm run verify:toss-openapi` and confirm the expected title, version, base URL, endpoint count, account header refs, and error schemas.
- Confirm the active profile is `broker_id = "toss"` and the account identifier is Toss `accountSeq`, not a KIS account number.
- Confirm Settings shows the profile as Toss and exposes the `연결 진단` action.
- Confirm `start_trading` still rejects Toss profiles with `BROKER_NOT_SUPPORTED`.

## 1. Read-Only Connection

Run Settings → Toss profile → `연결 진단`.

Expected diagnostic steps:

| Step | Endpoint | Required Evidence |
|------|----------|-------------------|
| OpenAPI spec | `GET /openapi-docs/latest/openapi.json` | version and path count are displayed |
| Token | `POST /oauth2/token` | token type and expiry are displayed; token value is never displayed |
| Accounts | `GET /api/v1/accounts` | accounts count is displayed and saved `accountSeq` matches an account |
| Holdings | `GET /api/v1/holdings` | holdings count is displayed; empty holdings is valid if the request succeeds |
| Buying power | `GET /api/v1/buying-power` | KRW and USD cash buying power are displayed |
| Commissions | `GET /api/v1/commissions` | commissions policy count is displayed |
| Sellable quantity | `GET /api/v1/sellable-quantity` | first holding symbol is checked when holdings exist; skipped when holdings are empty |

Failure handling:

- Preserve `X-Request-Id` and `Retry-After` in diagnostics/logs when Toss returns them.
- Fix profile configuration before retrying if token or accountSeq checks fail.
- Do not proceed to order validation if any read-only step fails.

## 2. Order-Preflight Read-Only Checks

Before enabling any order submission UI or adapter method, verify that the read-only adapter methods and Settings diagnostic cover:

| Purpose | Endpoint | Required Evidence |
|---------|----------|-------------------|
| Buying power | `GET /api/v1/buying-power` | requested symbol/side/order type returns available amount |
| Sellable quantity | `GET /api/v1/sellable-quantity` | sell quantity is available for owned symbol |
| Commissions | `GET /api/v1/commissions` | expected commission/tax data is available for market and order shape |

These checks must be called before `TradeGuard` allows a Toss order. Cache nothing that can change intraday unless the official response headers and rate-limit policy make that safe.

Trading 화면의 `Toss 소액 수동매매 검증` UI는 이 단계의 진행 상태를 보여준다. 이 UI는 활성 Toss `accountSeq`, 종목, 지정가, 수량, 가격, 실거래 동의 저장 상태, read-only 사전검증 결과를 표시하며, `TossOrderPreflightView.canSubmit=true`가 되면 숨김 처리된다. 현재 주문 adapter가 gate 뒤에 있으면 실제 주문 버튼은 계속 차단되어야 한다.

## 3. Small Live-Order Approval Gate

Live order testing requires a separate user approval that states:

- broker: Toss Securities
- accountSeq
- market: KR or US
- symbol
- side: buy or sell
- order type and price
- quantity
- maximum notional amount
- confirmation that the order may execute in a real account

Without that approval, order code may be implemented behind tests, but it must not be reachable from Settings, Trading, Strategy, Dashboard, or auto-trading flows.

## 4. First Small Order

When approval exists:

1. Generate and store a unique `clientOrderId`.
2. Re-run buying-power or sellable-quantity immediately before submission.
3. Submit the smallest practical limit order, not a market order.
4. Save provider `orderId`, `clientOrderId`, `requestId` if available, raw status, submitted price, submitted quantity, and provider timestamp.
5. Poll `GET /api/v1/orders/{orderId}` until the order is filled, rejected, canceled, or clearly pending.
6. If the order remains pending longer than the test window, cancel it with `POST /api/v1/orders/{orderId}/cancel`.
7. Record final state in order history before enabling any broader UI path.

## 5. Auto-Trading Unlock Criteria

Do not remove `BROKER_NOT_SUPPORTED` for Toss until all of these are true:

- read-only diagnostic passes from Settings
- buying-power, sellable-quantity, and commissions are integrated into order preflight
- create, detail, list, modify, and cancel order APIs are covered by adapter tests
- partial fill, rejection, already-canceled, and opposite pending order states are mapped to common order statuses
- rate-limit backoff honors `Retry-After` and `X-RateLimit-*`
- `TradeGuard` and `RiskManager` aggregate limits by broker/account scope
- History/Log can show Toss `requestId`, `orderId`, and `clientOrderId`

> Last updated: 2026-07-04T11:57:39+09:00
