# Toss Read-Only And Small-Order Verification Checklist

> Source of truth: `https://openapi.tossinvest.com/openapi-docs/latest/openapi.json`  
> Last verified: 2026-07-06, OpenAPI version `1.1.5`

This checklist prevents accidental live trading while operating Toss Securities support. Dashboard small-order verification, Trading manual orders, and auto-trading may submit live Toss orders only after profile diagnostics, order preflight, local/provider pending checks, and explicit `live_trading_consent` are in place.

## 0. Preconditions

- Do not read or paste `.env`, `secure_config.json`, or `profiles.json` into agent context.
- Run `npm run verify:toss-openapi` and confirm the expected title, version, base URL, endpoint count, account header refs, and error schemas.
- Confirm the active profile is `broker_id = "toss"` and the account identifier is Toss `accountSeq`, not a KIS account number.
- Confirm Settings shows the profile as Toss and exposes the `연결 진단` action.
- Confirm `start_trading` rejects Toss profiles without `live_trading_consent` and allows configured Toss profiles with consent.

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

## 2. Order-Preflight Checks

Before any order submission UI or adapter method calls the provider, verify that the preflight adapter methods and Settings diagnostic cover:

| Purpose | Endpoint | Required Evidence |
|---------|----------|-------------------|
| Buying power | `GET /api/v1/buying-power` | requested symbol/side/order type returns available amount |
| Sellable quantity | `GET /api/v1/sellable-quantity` | sell quantity is available for owned symbol |
| Commissions | `GET /api/v1/commissions` | expected commission/tax data is available for market and order shape |

These checks must be called before a Toss manual or automatic order is submitted. Cache nothing that can change intraday unless the official response headers and rate-limit policy make that safe.

Dashboard의 `Toss 소액 수동매매 검증` UI는 검색 종목 1주 시장가 매수 조건으로 현재가 snapshot 기반 사전검증과 최종 확인을 표시한다. Trading은 사용자가 입력한 주문 조건으로 preflight를 표시하고 `canSubmit=true`일 때 일반 주문 버튼을 활성화한다. Strategy/자동매매 화면에는 별도 소액매매 검증 UI를 두지 않는다.

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

Without that approval or stored `live_trading_consent`, order code must fail before provider submission. Dashboard `submit_toss_small_buy_verification` remains limited to a confirmed 1-share market buy for the searched symbol; Trading and auto-trading use their own preflight and pending-conflict gates.

## 4. First Small Order

When approval exists:

1. Generate and store a unique `clientOrderId`.
2. Re-run buying-power, commissions, current price snapshot, and stock-safety checks immediately before submission.
3. Scan current Toss open orders for the same symbol and block if any pending order exists.
4. Submit exactly one share as a market buy from Dashboard. The official `MARKET` schema must send `quantity="1"` and must not send `price` or `orderAmount`.
5. Enforce the user-entered maximum notional amount and hard caps of KRW 1,000,000 or USD 1,000 before calling the provider.
6. Save provider `orderId`, `clientOrderId`, raw status, submitted quantity, estimated gross amount, and provider trace in order history.
7. Poll `GET /api/v1/orders/{orderId}` until the order is filled, rejected, canceled, or clearly pending.
8. If the provider reports filled or partially filled quantity, record the execution in trade history before enabling any broader UI path.

## 5. Auto-Trading Runtime Criteria

Keep Toss auto-trading enabled only while all of these remain true:

- read-only diagnostic passes from Settings
- buying-power, sellable-quantity, and commissions are integrated into order preflight
- create, detail, list, modify, and cancel order APIs are covered by adapter tests
- partial fill, rejection, already-canceled, and opposite pending order states are mapped to common order statuses
- rate-limit backoff honors `Retry-After` and `X-RateLimit-*`
- `TradeGuard` and `RiskManager` aggregate limits by broker/account scope
- History/Log can show Toss `requestId`, `orderId`, and `clientOrderId`

> Last updated: 2026-07-06T22:10:00+09:00
