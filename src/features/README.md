# Features

사용자 행동 단위 slice를 두는 레이어입니다. 기능 로직을 이동할 때는 아래 slice 이름을 우선 사용합니다.

- `manual-order` — Trading/Dashboard에서 공유하는 수동 주문·Toss 소액 검증 gate
- `symbol-search`
- `strategy-toggle`
- `strategy-configure`
- `trading-start-stop`
- `log-filter`
- `discord-notification-config`

각 slice는 필요할 때 `ui`, `model`, `api`, `lib` 세그먼트와 `index.ts` public API를 둡니다.
