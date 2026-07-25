# KISAutoTrade — 프로젝트 맵

> 이 문서는 `AGENTS.md` 에서 분리된 상세 디렉토리 맵 및 아키텍처 참조 문서입니다.

---

## 1. 전체 디렉토리 맵

아래 블록은 Git이 추적하는 파일과 `.gitignore`에 걸리지 않은 새 파일을 기준으로
자동 생성한다. `npm run project-map:update`로 갱신하고
`npm run check:project-map`으로 drift를 검사한다.

<!-- project-map:generated:start -->
<!-- 이 블록은 scripts/project-map.mjs가 생성합니다. 직접 편집하지 마세요. -->
```text
KISAutoTrade/
├── .claude/
│   ├── skills/
│   │   ├── kisautotrade-frontend-fsd/
│   │   │   └── SKILL.md
│   │   ├── kisautotrade-kis-api/
│   │   │   └── SKILL.md
│   │   ├── kisautotrade-project/
│   │   │   └── SKILL.md
│   │   ├── kisautotrade-react/
│   │   │   └── SKILL.md
│   │   ├── kisautotrade-rust/
│   │   │   └── SKILL.md
│   │   ├── kisautotrade-toss-api/
│   │   │   └── SKILL.md
│   │   └── kisautotrade-ui/
│   │       └── SKILL.md
│   └── README.md
├── .codex/
│   ├── agents/
│   │   ├── helper_curator.toml
│   │   ├── project_mapper.toml
│   │   └── symbol_navigator.toml
│   ├── skills/
│   │   ├── kisautotrade-frontend-fsd/
│   │   │   └── SKILL.md
│   │   ├── kisautotrade-kis-api/
│   │   │   └── SKILL.md
│   │   ├── kisautotrade-project/
│   │   │   └── SKILL.md
│   │   ├── kisautotrade-react/
│   │   │   └── SKILL.md
│   │   ├── kisautotrade-rust/
│   │   │   └── SKILL.md
│   │   ├── kisautotrade-toss-api/
│   │   │   └── SKILL.md
│   │   └── kisautotrade-ui/
│   │       └── SKILL.md
│   ├── README.md
│   └── config.toml
├── .github/
│   ├── agents/
│   │   ├── helper-curator.agent.md
│   │   ├── project-map-maintainer.agent.md
│   │   └── symbol-navigator.agent.md
│   ├── skills/
│   │   ├── frontend-fsd/
│   │   │   └── SKILL.md
│   │   ├── kis-api/
│   │   │   └── SKILL.md
│   │   ├── react-best-practices/
│   │   │   └── SKILL.md
│   │   ├── rust-skills/
│   │   │   └── SKILL.md
│   │   ├── toss-api/
│   │   │   └── SKILL.md
│   │   └── ui-conventions/
│   │       └── SKILL.md
│   ├── workflows/
│   │   └── release.yml
│   ├── codex-instructions.md
│   └── copilot-instructions.md
├── .serena/
│   ├── memories/
│   │   ├── backend/
│   │   │   └── core.md
│   │   ├── frontend/
│   │   │   └── core.md
│   │   ├── conventions.md
│   │   ├── core.md
│   │   ├── memory_maintenance.md
│   │   ├── suggested_commands.md
│   │   ├── task_completion.md
│   │   └── tech_stack.md
│   ├── .gitignore
│   └── project.yml
├── .vscode/
│   ├── extensions.json
│   ├── launch.json
│   └── tasks.json
├── docs/
│   ├── MasterPlan.md
│   ├── agent-tooling.md
│   ├── coding-guide.md
│   ├── discord-setup-guide.md
│   ├── ipc-commands.md
│   ├── leveraged-trend-hold-parameter-guide.md
│   ├── mock-trading-e2e-checklist.md
│   ├── project-map.md
│   ├── toss-openapi.md
│   ├── toss-readonly-small-order-checklist.md
│   └── user-guide.md
├── scripts/
│   ├── check-fsd-imports.mjs
│   ├── graphify.mjs
│   ├── project-map.mjs
│   ├── release-version.mjs
│   ├── setup-local.sh
│   ├── sync-codex-skills.ps1
│   └── verify-toss-openapi.mjs
├── src/
│   ├── api/
│   │   ├── backendEvents.ts
│   │   ├── commands.ts
│   │   ├── databaseHooks.ts
│   │   ├── hooks.ts
│   │   ├── queryKeys.ts
│   │   ├── transport.ts
│   │   └── types.ts
│   ├── components/
│   │   ├── chart/
│   │   │   ├── OverseasStockChart.tsx
│   │   │   └── StockChart.tsx
│   │   ├── layout/
│   │   │   ├── AppShell.tsx
│   │   │   └── Sidebar.tsx
│   │   └── LayoutResizer.tsx
│   ├── entities/
│   │   ├── account/
│   │   │   ├── model/
│   │   │   │   └── accountStore.ts
│   │   │   └── index.ts
│   │   ├── settings/
│   │   │   ├── model/
│   │   │   │   └── settingsStore.ts
│   │   │   └── index.ts
│   │   └── trading/
│   │       ├── model/
│   │       │   └── tradingStore.ts
│   │       └── index.ts
│   ├── features/
│   │   ├── discord-notification-config/
│   │   │   └── index.ts
│   │   ├── log-filter/
│   │   │   └── index.ts
│   │   ├── strategy-configure/
│   │   │   └── index.ts
│   │   ├── strategy-toggle/
│   │   │   └── index.ts
│   │   ├── symbol-search/
│   │   │   └── index.ts
│   │   ├── trading-start-stop/
│   │   │   └── index.ts
│   │   └── README.md
│   ├── pages/
│   │   ├── dashboard/
│   │   │   ├── ui/
│   │   │   │   ├── Page.tsx
│   │   │   │   ├── orderPanels.tsx
│   │   │   │   └── tossVerificationPanel.tsx
│   │   │   └── index.ts
│   │   ├── history/
│   │   │   ├── ui/
│   │   │   │   └── Page.tsx
│   │   │   └── index.ts
│   │   ├── log/
│   │   │   ├── ui/
│   │   │   │   └── Page.tsx
│   │   │   └── index.ts
│   │   ├── settings/
│   │   │   ├── ui/
│   │   │   │   ├── Page.tsx
│   │   │   │   ├── accountProfiles.tsx
│   │   │   │   ├── brokerRateLimitSection.tsx
│   │   │   │   ├── databaseManagementSection.tsx
│   │   │   │   ├── profileDialogs.tsx
│   │   │   │   ├── profileUtils.ts
│   │   │   │   └── section.tsx
│   │   │   └── index.ts
│   │   ├── strategy/
│   │   │   ├── model/
│   │   │   │   └── experimentStore.ts
│   │   │   ├── ui/
│   │   │   │   ├── Page.tsx
│   │   │   │   ├── leveragedTrendHoldEditorPanel.tsx
│   │   │   │   ├── leveragedTrendHoldPreviewChart.tsx
│   │   │   │   ├── priceConditionEditorPanel.tsx
│   │   │   │   ├── strategyMetadata.ts
│   │   │   │   ├── strategyPreviewPanel.tsx
│   │   │   │   └── strategyResearchPanel.tsx
│   │   │   └── index.ts
│   │   └── trading/
│   │       ├── ui/
│   │       │   ├── Page.tsx
│   │       │   ├── kisPanels.tsx
│   │       │   └── tossPanels.tsx
│   │       └── index.ts
│   ├── router/
│   │   └── index.ts
│   ├── scheduler/
│   │   └── index.ts
│   ├── shared/
│   │   ├── api/
│   │   │   ├── commands.ts
│   │   │   ├── databaseCommands.ts
│   │   │   ├── databaseTypes.ts
│   │   │   ├── index.ts
│   │   │   ├── strategyResearchTypes.ts
│   │   │   ├── transport.ts
│   │   │   └── types.ts
│   │   ├── config/
│   │   │   ├── scheduler/
│   │   │   │   └── index.ts
│   │   │   └── theme/
│   │   │       └── index.ts
│   │   ├── lib/
│   │   │   ├── formatters.ts
│   │   │   ├── index.ts
│   │   │   └── persistentLayout.ts
│   │   └── ui/
│   │       ├── BrokerScopeIndicator.tsx
│   │       ├── LayoutResizer.tsx
│   │       ├── ProviderTraceChips.tsx
│   │       ├── TradingHealthPanel.tsx
│   │       └── index.ts
│   ├── store/
│   │   ├── accountStore.ts
│   │   ├── settingsStore.ts
│   │   └── tradingStore.ts
│   ├── theme/
│   │   └── index.ts
│   ├── widgets/
│   │   ├── app-shell/
│   │   │   ├── ui/
│   │   │   │   └── AppShell.tsx
│   │   │   └── index.ts
│   │   ├── sidebar/
│   │   │   ├── ui/
│   │   │   │   └── Sidebar.tsx
│   │   │   └── index.ts
│   │   └── stock-chart/
│   │       ├── ui/
│   │       │   ├── OverseasStockChart.tsx
│   │       │   └── StockChart.tsx
│   │       └── index.ts
│   └── main.tsx
├── src-tauri/
│   ├── icons/
│   │   ├── android/
│   │   │   ├── mipmap-anydpi-v26/
│   │   │   │   └── ic_launcher.xml
│   │   │   ├── mipmap-hdpi/
│   │   │   │   ├── ic_launcher.png
│   │   │   │   ├── ic_launcher_foreground.png
│   │   │   │   └── ic_launcher_round.png
│   │   │   ├── mipmap-mdpi/
│   │   │   │   ├── ic_launcher.png
│   │   │   │   ├── ic_launcher_foreground.png
│   │   │   │   └── ic_launcher_round.png
│   │   │   ├── mipmap-xhdpi/
│   │   │   │   ├── ic_launcher.png
│   │   │   │   ├── ic_launcher_foreground.png
│   │   │   │   └── ic_launcher_round.png
│   │   │   ├── mipmap-xxhdpi/
│   │   │   │   ├── ic_launcher.png
│   │   │   │   ├── ic_launcher_foreground.png
│   │   │   │   └── ic_launcher_round.png
│   │   │   ├── mipmap-xxxhdpi/
│   │   │   │   ├── ic_launcher.png
│   │   │   │   ├── ic_launcher_foreground.png
│   │   │   │   └── ic_launcher_round.png
│   │   │   └── values/
│   │   │       └── ic_launcher_background.xml
│   │   ├── ios/
│   │   │   ├── AppIcon-20x20@1x.png
│   │   │   ├── AppIcon-20x20@2x-1.png
│   │   │   ├── AppIcon-20x20@2x.png
│   │   │   ├── AppIcon-20x20@3x.png
│   │   │   ├── AppIcon-29x29@1x.png
│   │   │   ├── AppIcon-29x29@2x-1.png
│   │   │   ├── AppIcon-29x29@2x.png
│   │   │   ├── AppIcon-29x29@3x.png
│   │   │   ├── AppIcon-40x40@1x.png
│   │   │   ├── AppIcon-40x40@2x-1.png
│   │   │   ├── AppIcon-40x40@2x.png
│   │   │   ├── AppIcon-40x40@3x.png
│   │   │   ├── AppIcon-512@2x.png
│   │   │   ├── AppIcon-60x60@2x.png
│   │   │   ├── AppIcon-60x60@3x.png
│   │   │   ├── AppIcon-76x76@1x.png
│   │   │   ├── AppIcon-76x76@2x.png
│   │   │   └── AppIcon-83.5x83.5@2x.png
│   │   ├── 128x128.png
│   │   ├── 128x128@2x.png
│   │   ├── 32x32.png
│   │   ├── 64x64.png
│   │   ├── Square107x107Logo.png
│   │   ├── Square142x142Logo.png
│   │   ├── Square150x150Logo.png
│   │   ├── Square284x284Logo.png
│   │   ├── Square30x30Logo.png
│   │   ├── Square310x310Logo.png
│   │   ├── Square44x44Logo.png
│   │   ├── Square71x71Logo.png
│   │   ├── Square89x89Logo.png
│   │   ├── StoreLogo.png
│   │   ├── icon.icns
│   │   ├── icon.ico
│   │   ├── icon.png
│   │   └── icon.svg
│   ├── src/
│   │   ├── api/
│   │   │   ├── rest/
│   │   │   │   ├── exchange.rs
│   │   │   │   └── types.rs
│   │   │   ├── detect.rs
│   │   │   ├── mod.rs
│   │   │   ├── rest.rs
│   │   │   ├── token.rs
│   │   │   └── websocket.rs
│   │   ├── broker/
│   │   │   ├── toss/
│   │   │   │   ├── adapter.rs
│   │   │   │   ├── client.rs
│   │   │   │   ├── error.rs
│   │   │   │   ├── http.rs
│   │   │   │   ├── mod.rs
│   │   │   │   ├── orders.rs
│   │   │   │   ├── support.rs
│   │   │   │   ├── tests.rs
│   │   │   │   └── types.rs
│   │   │   ├── adapter.rs
│   │   │   ├── domain.rs
│   │   │   ├── kis.rs
│   │   │   ├── mod.rs
│   │   │   └── rate_limit.rs
│   │   ├── commands/
│   │   │   ├── strategy_preview/
│   │   │   │   └── tests.rs
│   │   │   ├── toss/
│   │   │   │   └── small_order.rs
│   │   │   ├── trading/
│   │   │   │   ├── history.rs
│   │   │   │   └── risk_restore_tests.rs
│   │   │   ├── accounts.rs
│   │   │   ├── archive.rs
│   │   │   ├── database.rs
│   │   │   ├── market.rs
│   │   │   ├── orders.rs
│   │   │   ├── records.rs
│   │   │   ├── risk.rs
│   │   │   ├── settings.rs
│   │   │   ├── strategy.rs
│   │   │   ├── strategy_preview.rs
│   │   │   ├── toss.rs
│   │   │   ├── toss_market.rs
│   │   │   └── trading.rs
│   │   ├── config/
│   │   │   └── mod.rs
│   │   ├── logging/
│   │   │   └── mod.rs
│   │   ├── market/
│   │   │   └── mod.rs
│   │   ├── notifications/
│   │   │   ├── discord.rs
│   │   │   ├── mod.rs
│   │   │   └── types.rs
│   │   ├── server/
│   │   │   ├── market.rs
│   │   │   ├── mod.rs
│   │   │   ├── profiles.rs
│   │   │   ├── records.rs
│   │   │   ├── security.rs
│   │   │   ├── toss.rs
│   │   │   └── trading.rs
│   │   ├── storage/
│   │   │   ├── balance_store.rs
│   │   │   ├── database.rs
│   │   │   ├── database_archive.rs
│   │   │   ├── database_contract_tests.rs
│   │   │   ├── database_io.rs
│   │   │   ├── database_keychain.rs
│   │   │   ├── database_projection.rs
│   │   │   ├── database_schema.rs
│   │   │   ├── database_types.rs
│   │   │   ├── mod.rs
│   │   │   ├── order_store.rs
│   │   │   ├── pending_order_store.rs
│   │   │   ├── risk_store.rs
│   │   │   ├── stats_store.rs
│   │   │   ├── stock_store.rs
│   │   │   ├── strategy_store.rs
│   │   │   └── trade_store.rs
│   │   ├── trading/
│   │   │   ├── order/
│   │   │   │   ├── conflicts.rs
│   │   │   │   ├── fills.rs
│   │   │   │   └── submission.rs
│   │   │   ├── strategy/
│   │   │   │   ├── breakout.rs
│   │   │   │   ├── classic.rs
│   │   │   │   ├── core.rs
│   │   │   │   ├── leveraged_trend_hold.rs
│   │   │   │   ├── ma_cross.rs
│   │   │   │   ├── manager.rs
│   │   │   │   ├── mean_trend.rs
│   │   │   │   ├── price_condition.rs
│   │   │   │   ├── state.rs
│   │   │   │   └── tests.rs
│   │   │   ├── guard.rs
│   │   │   ├── mod.rs
│   │   │   ├── order.rs
│   │   │   ├── position.rs
│   │   │   ├── preflight.rs
│   │   │   ├── risk.rs
│   │   │   ├── simulation.rs
│   │   │   ├── strategy.rs
│   │   │   └── views.rs
│   │   ├── updater/
│   │   │   └── mod.rs
│   │   ├── commands.rs
│   │   ├── lib.rs
│   │   ├── main.rs
│   │   └── market_hours.rs
│   ├── Cargo.lock
│   ├── Cargo.toml
│   ├── build.rs
│   └── tauri.conf.json
├── tests/
│   └── e2e/
│       ├── settings-database.spec.ts
│       └── strategy-scrollbar.spec.ts
├── .env.example
├── .gitignore
├── .graphifyignore
├── .nvmrc
├── AGENTS.md
├── CLAUDE.md
├── Cargo.lock
├── Cargo.toml
├── LICENSE
├── README.md
├── index.html
├── package-lock.json
├── package.json
├── playwright.config.ts
├── secure_config.example.json
├── todo.md
├── tsconfig.json
├── tsconfig.node.json
└── vite.config.ts
```
<!-- project-map:generated:end -->

---

## 2. 파일 저장 위치 요약

| 데이터 종류 | 위치 |
|-----------|------|
| `profiles.json` | `~/Library/Application Support/com.countjung.kisautotrade/` (macOS) |
| `database_config.json` | 앱 데이터 폴더 (DB password 포함, IPC에서는 미반환) |
| `database_exports/` | 앱 데이터 폴더 (DB logical JSON snapshot + manifest) |
| `data/` (거래기록 등) | CWD 기준 `./data/` (레거시: app_data_dir, 자동 이전) |
| `logs/` | CWD 기준 `./logs/` |
| `secure_config.json` | 프로젝트 루트 (CWD) |
| `.env` | 프로젝트 루트 (CWD) |

---

## 3. 핵심 모듈 책임 요약

### Frontend

| 모듈 | 책임 |
|------|------|
| `router/` | TanStack Router 기반 라우팅 |
| `shared/api/` | Tauri IPC/Web REST wrapper, Rust 타입 미러 (`BrokerHoldingView` 포함) |
| `shared/ui/` | 공통 UI (`LayoutResizer`, `BrokerScopeIndicator` broker/profile/account scope 표시, `ProviderTraceChips` 원본 요청 trace 표시) |
| `shared/lib/` | 공통 유틸 (`persistentLayout` localStorage 숫자 상태 저장/복원, `formatters` 숫자/decimal/money 표시) |
| `shared/config/theme/` | 앱 테마 생성과 theme mode 타입 |
| `shared/config/scheduler/` | TanStack Query 공통 폴링 주기 |
| `entities/*/model/` | Zustand 전역 상태 (계좌, 매매, 설정) |
| `api/hooks.ts` | TanStack Query 훅 legacy entry (`KEYS`, `useBackendEvents` re-export 유지) |
| `api/queryKeys.ts` | TanStack Query `KEYS` 단일 원천 (`hooks.ts`에서 하위 호환 re-export) |
| `api/backendEvents.ts` | Tauri backend event 구독 → 환율/잔고 Query 캐시 갱신 |
| `api/databaseHooks.ts` | DB 설정/상태/이관 Tauri-only TanStack Query hooks |
| `widgets/app-shell/` | 전체 앱 레이아웃, ThemeProvider, responsive navigation, 좌측 사이드바 자동매매 시작/정지 전역 조작 |
| `widgets/stock-chart/` | 국내/해외/Toss 캔들 차트 |
| `pages/settings/ui/Page.tsx` | 테마, 데이터 갱신 주기, 로그/체결 보관, 웹 포트, 종목 목록, 리스크, Discord 설정 route 조립 |
| `pages/settings/ui/databaseManagementSection.tsx` | PostgreSQL/MariaDB 연결, 고정 앱 테이블 관리, JSON↔DB 이관, backend 전환. 웹 모드에서는 보안 안내만 표시 |
| `pages/settings/ui/accountProfiles.tsx` | 활성 broker/profile 요약, KIS/Toss 프로파일 목록, 연결 진단, 프로파일 삭제 확인 |
| `pages/settings/ui/profileDialogs.tsx` | KIS/Toss 프로파일 추가/편집, 실전/모의 감지, Toss accountSeq 조회. secret 입력 상태는 이 파일 안에만 보관 |
| `pages/settings/ui/profileUtils.ts` | 프로파일 에러 메시지와 broker label 공통 유틸 |
| `pages/settings/ui/section.tsx` | Settings page-local `Section` 래퍼 |
| `pages/dashboard/ui/Page.tsx` | 활성 broker scope, KIS 국내/해외 잔고, 활성 Toss broker 보유 종목/평가 요약, Toss 자동매매 시작, 소액매매 검증 안내, USD/KRW 환율 출처 chip, 수익 카드, 리스크, Dashboard 라우트 조립 |
| `pages/dashboard/ui/orderPanels.tsx` | Dashboard 미체결 주문과 체결 내역 필터/정렬/페이지네이션 패널 |
| `pages/dashboard/ui/tossVerificationPanel.tsx` | 활성 Toss 프로파일의 실거래 동의 상태를 표시하고 실제 주문 검증을 수행하는 수동거래 페이지로 안내 |
| `pages/trading/ui/Page.tsx` | 활성 broker scope, KIS/Toss 수동 주문, Toss 주문 전 검증, 차트와 Trading 라우트 조립 (`kisPanels.tsx`, `tossPanels.tsx`로 세부 패널 분리) |
| `pages/trading/ui/kisPanels.tsx` | KIS 국내/해외 보유 테이블과 KIS 현재가 카드 |
| `pages/trading/ui/tossPanels.tsx` | 활성 Toss 프로파일의 holdings/시세 snapshot/차트/종목 유의사항/장 운영 상태, 주문 전 검증, 접수 주문 목록과 정정 UI |
| `pages/strategy/ui/Page.tsx` | 활성 broker scope, 전략별 저장 broker/account scope 표시, 전략 활성화/대상 종목/전체 너비 카드/카드별 미리보기 route 조립 |
| `pages/strategy/ui/priceConditionEditorPanel.tsx` | 가격조건 전략의 종목별 수량·매수가·익절가·익절률·손절률 편집 테이블 |
| `pages/strategy/ui/strategyMetadata.ts` | 일반 전략 파라미터 입력 메타, 전략 설명, strategy id 기반 타입 판별 |
| `pages/strategy/ui/leveragedTrendHoldEditorPanel.tsx` | 레버리지 추세 보유 전략의 ETF 검색/편집, 진입·반등·청산 파라미터, Toss 1분/일봉·50/100/200봉 preview와 계좌 scope/stale 응답 차단 |
| `pages/strategy/ui/strategyPreviewPanel.tsx` | 일반/가격조건 전략 카드에서 KIS 일/주/월봉 또는 Toss 1분/일봉을 조회하고 편집값·비용 가정을 `preview_strategy`에 전달. 티커/봉/구간/파라미터/수량/broker/account/가정 변경 시 결과 무효화 |
| `pages/strategy/ui/strategyResearchPanel.tsx` | 초기자본·수수료·세금·슬리피지·환율·리스크·학습구간 입력, 수익률/MDD/승률/손익비/turnover/exposure, 원시 신호와 주문 가능/체결 구분, equity curve, 거래 목록, in/out-of-sample, A/B 비교 UI |
| `pages/strategy/model/experimentStore.ts` | credential을 저장하지 않고 broker/account/strategy/symbol scope별 A/B 결과·전략 버전·파라미터·데이터 범위/source·비용 가정·생성 시각을 localStorage에 최대 2개 저장 |
| `pages/strategy/ui/leveragedTrendHoldPreviewChart.tsx` | 레버리지/일반 전략 미리보기의 캔들·종가선·signal marker, 모바일 가로 패닝·핀치 확대/축소와 버튼식 줌 표시 |
| `pages/history/ui/Page.tsx` | 활성 broker scope, 자동매매 체결 기록과 기간별 통계 조회, provider 원본 trace 표시 |
| `pages/log/ui/Page.tsx` | 로그 레벨/검색 필터, provider trace 토큰 chip 표시 |

### Backend (Rust)

| 모듈 | 책임 |
|------|------|
| `lib.rs` | Tauri Builder + window-state 플러그인 + 6개 백그라운드 데몬 spawn + `on_window_event` (종료 안전 처리) |
| `commands.rs` | AppState + IPC command facade, 공통 helper |
| `commands/accounts.rs` | 계좌 프로파일 CRUD/활성화, KIS 국내·해외 잔고, broker holdings view. 프로파일 전환 시 strategy scope reset 포함 |
| `commands/archive.rs` | 거래 파일 보관 설정, 보관 통계, 오래된 trade/log 파일 purge IPC |
| `commands/database.rs` | 인증 없는 REST에는 노출하지 않는 Tauri-only DB 관리/이관 IPC |
| `commands/market.rs` | KIS 국내/해외 시세, 차트, 종목 검색, 해외 주문 사전 검증 IPC |
| `commands/orders.rs` | 수동 주문 제출 IPC |
| `commands/records.rs` | 체결/거래/통계 조회, Discord config 저장, frontend log 저장 IPC |
| `commands/settings.rs` | app config/check_config, refresh interval, log/web 설정, USD/KRW 환율 IPC |
| `commands/strategy_preview.rs` | 범용 preview는 최대 500봉·공통 warmup·candle-close replay를 제공하고 broker/account는 결과 scope 메타데이터로 보존한다. Toss 레버리지 preview는 활성 profile/account를 검증하고 1분봉 warmup을 첫 거래일 이전 완료 일봉으로 제한하며 일봉 open/EOD 공개 시점을 분리한다. 두 경로 모두 재현 메타데이터와 비용/리스크 backtest를 반환한다. |
| `commands/strategy_preview/tests.rs` | 일봉 정보 공개 시점, session 경계, 미래 봉 변경이 이전 신호에 영향을 주지 않는 deterministic fixture |
| `commands/toss.rs` | Toss accountSeq 조회, 연결 진단, 주문 전 preflight view facade, 접수 주문 목록 조회와 정정 command |
| `commands/toss/small_order.rs` | 호환용 Toss 1주 소액매매 검증 endpoint. 실거래 동의/최종 확인/최대 허용금액/preflight/open-order scan 후 시장가 매수 제출과 주문·체결 기록 저장. 현재 UI는 일반 수동주문 경로를 사용 |
| `commands/toss_market.rs` | Toss 시세 snapshot, 종목 유의사항, market-calendar override, candles chart command |
| `commands/trading.rs` | 자동매매 상태/시작/정지, broker-aware 포지션 동기화, polling daemon. 히스토리 초기화는 `commands/trading/history.rs`로 분리 |
| `commands/strategy.rs` | 포지션 조회, 전략 목록/수정 IPC. 전략 view는 `trading/views.rs::build_strategy_view()` 재사용 |
| `commands/risk.rs` | 리스크 설정/비상정지 IPC와 pending 주문 view |
| `trading/strategy.rs` | 전략 facade. 공개 import 경로를 유지하면서 하위 전략 모듈을 re-export |
| `trading/strategy/core.rs` | `Signal`, `StrategySignal`, `BrokerPositionSnapshot`, `StrategyConfig`, `Strategy` trait, live/preview 공통 `initialize_strategy_warmup()` |
| `trading/strategy/manager.rs` | `StrategyManager`, `build_strategy()`, 전략 config 재빌드와 공통 warmup dispatch |
| `trading/simulation.rs` | deterministic simulated portfolio, 공통 TradeGuard/RiskManager 기반 주문 가능 판정, 비용·환율·equity·MDD·turnover·exposure·고정 chronological in/out-of-sample 지표. preview command가 engine/strategy/source/scope/params/가정/warmup 경계와 실제 replay OHLCV fingerprint를 hash한다. |
| `trading/strategy/state.rs` | per-symbol 전략 버퍼 상한 helper. user-param 기반 `VecDeque` OOM 방지 |
| `trading/strategy/{classic,breakout,mean_trend}.rs` | MA/RSI/모멘텀/이격도, 돌파 계열, 평균회귀/추세필터 전략 구현 |
| `trading/strategy/{leveraged_trend_hold,price_condition}.rs` | 레버리지 추세 보유와 종목별 가격 조건 전략 구현 |
| `api/detect.rs` | KIS 토큰 응답 기반 실전/모의 앱키 자동 감지 |
| `api/rest.rs` | KIS REST client facade. rate-limit group을 거쳐 잔고/주문/체결/시세/차트 요청 수행 |
| `api/rest/types.rs` | KIS REST 타입, `OrderSide`/`OrderType`, 국내/해외 잔고·체결·시세 응답, 해외 주문 사전 검증 |
| `api/rest/exchange.rs` | 공개 USD/KRW 환율 fallback fetcher (`fetch_usd_krw_rate`) |
| `broker/` | 다중 증권사 공통 타입(`BrokerScope` 포함), adapter trait, `RateLimitScheduler`. KIS 기존 REST 호출을 점진 래핑하고 Toss token/accounts/holdings/market-data/market-info/order client를 수용 |
| `broker/toss/mod.rs` | Toss 공개 surface re-export. 내부 DTO/helper는 외부로 직접 노출하지 않음 |
| `broker/toss/adapter.rs` | `TossBrokerAdapter` 구현과 broker 공통 타입 매핑 |
| `broker/toss/client.rs` | OAuth2 token, accounts/holdings/market/order REST 호출과 401 1회 재시도 |
| `broker/toss/http.rs` | reqwest client timeout, base URL/query encoding, response body streaming cap |
| `broker/toss/error.rs` | Toss error envelope와 provider request id/error snippet formatting |
| `broker/toss/support.rs` | rate-limit group, currency/market 변환, 주문 입력 validation/clientOrderId helper |
| `broker/toss/types.rs` | Toss read-only/account/market DTO와 broker domain 변환 |
| `broker/toss/orders.rs` | Toss 주문 생성/목록/상세/정정/취소 DTO와 주문 validation |
| `api/token.rs` | KIS Access Token 자동 갱신 |
| `api/websocket.rs` | 실시간 시세 수신, 체결 콜백 |
| `trading/mod.rs` | 전략 루프 실행, 장 시간 감지 |
| `trading/views.rs` | `StrategyConfig` → camelCase `StrategyView` 공용 builder. IPC/REST가 같은 view를 사용하며 종목명 조회 전 manager lock을 해제 |
| `trading/order.rs` | `buy_suspended` 플래그, provider trace 캡처, OrderManager facade |
| `trading/order/submission.rs` | `submit_signal_shared()` lock-short 주문 제출, provider 호출 중 `submitting` 예약, 실패 주문 기록 보존 |
| `trading/order/fills.rs` | `OrderManager::on_fill()`, KIS/Toss pending 체결 확인, 체결/수수료/통계/TradeStore 저장. daemon은 shared helper로 provider 체결 조회 네트워크 호출을 `order_manager` mutex 밖에서 수행 |
| `trading/order/conflicts.rs` | 실행 `BrokerScope` 기준 같은 방향/반대 방향 pending 주문 충돌 판정과 provider trace 기반 pending provider 판정 |
| `trading/risk.rs` | 일일 손실 한도, 비상 정지, `record_pnl` |
| `market_hours.rs` | 시장 개장 여부 (KRX 09:00-15:30 / US 22:00-07:00 KST) |
| `server/mod.rs` | axum 웹 서버 route table, ServeDir fallback |
| `server/market.rs` | 웹 REST KIS 잔고/해외잔고, broker holdings, 현재가, 주문, 차트, 종목 검색/갱신 |
| `server/records.rs` | 웹 REST 포지션, 통계/체결 조회, pending 주문, 로그 설정/최근 로그, 체결 보관 설정/통계, 프론트엔드 로그. REST 보관 설정 변경도 IPC와 같이 즉시 purge를 예약 |
| `server/profiles.rs` | 웹 REST 프로파일 CRUD/활성 전환/실전·모의 감지/Toss accountSeq·diagnostic. 프로파일 view는 IPC `profile_to_view()`를 재사용 |
| `server/toss.rs` | 웹 REST Toss 시세 snapshot, 종목 유의사항, 주문 전 preflight, 호환용 1주 소액매매 검증 endpoint, 접수 주문 조회·정정, market-calendar, candles |
| `server/trading.rs` | 웹 REST 자동매매 상태/시작/정지, 전략 목록/수정. 웹 start는 broker별 설정 검증, 실행 scope 설정, KIS/Toss 잔고 기반 전략 포지션 복원을 수행 |
| `storage/trade_store.rs` | `data/trades/YYYY/MM/DD/trades.json` (`provider_*` 원본 요청 trace 포함) |
| `storage/order_store.rs` | `data/orders/YYYY/MM/DD/orders.json` (`provider_*` 원본 주문 trace 포함) |
| `storage/stats_store.rs` | `data/stats/YYYY/MM/daily_stats.json` |
| `storage/strategy_store.rs` | `data/strategies/{profile_id}/strategies.json` (`StrategyConfig`에 broker/account scope 저장) |
| `storage/database.rs` | PostgreSQL/MariaDB 문서 호환 계층, schema v2 정규화 projection, transaction import/retention, JSON export, fail-closed backend 전환 |
| `notifications/discord.rs` | Discord Bot 알림 |
| `config/mod.rs` | `secure_config.json` + `.env` 로드 |

---

## 4. 백그라운드 데몬 목록 (lib.rs spawn 순서)

| 번호 | 역할 | 제어 방식 |
|------|------|----------|
| 1 | KRX 종목 목록 로드 | 1회성 |
| 2 | 자동매매 폴링 (`run_trading_daemon`) | `is_trading: Arc<Mutex<bool>>` |
| 3 | axum 웹 서버 | 영구 실행 |
| 4 | 환율 갱신 (USD/KRW) | `watch::Receiver` — Toss 우선/공개 환율/fallback 캐시 정책 + interval 변경 즉시 반영 |
| 5 | 로그/체결기록 일일 정리 | 24h 주기 |
| 6 | 잔고 갱신 + 이벤트 발행 | `watch::Receiver` — interval 변경 즉시 반영, 활성 broker가 KIS일 때만 KIS 잔고 조회 |

---

## 5. 데이터 흐름

### 체결 발생 시

```
WebSocket 수신 (체결 이벤트)
    ↓
trading/order.rs — 체결 확인
    ↓
storage/trade_store.rs — 공통 저장 경계 (JSON 또는 DB document, provider/order/request/TR trace 포함)
    ↓
storage/stats_store.rs — 통계 집계 갱신
    ↓
notifications/discord.rs — TRADE 레벨 알림 전송
    ↓
Tauri Event emit → Frontend (실시간 UI 갱신)
```

### 실시간 데이터 Push (백그라운드 데몬 → 프론트)

```
lib.rs daemon 4/6 → app_handle.emit("exchange-rate-updated" / "exchange-rate-status-updated" / "balance-updated" / "overseas-balance-updated")
    ↓
AppShell.tsx — useBackendEvents() listen()
    ↓
TanStack Query — setQueryData() (캐시 직접 갱신, 네트워크 요청 없음)
    ↓
관련 컴포넌트 리렌더
```

---

## 6. 설정 파일 레퍼런스

### `.env` (git ignore)

```
KIS_APP_KEY=실전투자_앱키
KIS_APP_SECRET=실전투자_앱시크릿
KIS_ACCOUNT_NO=12345678-01
KIS_IS_PAPER_TRADING=false
WEB_PORT=7474
REFRESH_INTERVAL_SEC=30
```

> `WEB_PORT` / `REFRESH_INTERVAL_SEC` 는 Settings UI에서도 수정 가능 (`.env` 자동 갱신)

### `secure_config.json` (git ignore)

`secure_config.example.json` 참고. Discord 봇 토큰, 모의/실전 듀얼 키 포함.

> **우선순위**: `secure_config.json` > `.env` 환경변수 > 기본값

---

## 7. 외부 의존 서비스

| 서비스 | 용도 | 참고 |
|--------|------|------|
| 한국투자증권 Open API | REST + WebSocket 주식 거래 | [apiportal.koreainvestment.com](https://apiportal.koreainvestment.com) |
| 토스증권 Open API | REST 기반 시세·계좌·주문 확장 후보 | [developers.tossinvest.com](https://developers.tossinvest.com/docs) |
| Discord Bot API | 알림 전송 | `docs/discord-setup-guide.md` |

---

## 8. Codex 프로젝트 브리지 스킬

GitHub Copilot 호환용으로 유지하던 `.github/skills/**/SKILL.md` 원본 스킬은 Codex에서도 자동 트리거될 수 있도록 프로젝트 루트 `.codex/skills/kisautotrade-*`에 얇은 브리지로 연결되어 있다. 브리지는 절대 경로를 저장하지 않고, 현재 작업 저장소에서 `AGENTS.md`와 `.github/skills/**`를 찾아 원본을 읽는다.

| Codex 프로젝트 스킬 | 저장소 원본 |
|-----------------|-------------|
| `.codex/skills/kisautotrade-project` | `AGENTS.md`, `.github/codex-instructions.md` |
| `.codex/skills/kisautotrade-kis-api` | `.github/skills/kis-api/SKILL.md` |
| `.codex/skills/kisautotrade-toss-api` | `.github/skills/toss-api/SKILL.md` |
| `.codex/skills/kisautotrade-rust` | `.github/skills/rust-skills/SKILL.md` |
| `.codex/skills/kisautotrade-react` | `.github/skills/react-best-practices/SKILL.md` |
| `.codex/skills/kisautotrade-frontend-fsd` | `.github/skills/frontend-fsd/SKILL.md` |
| `.codex/skills/kisautotrade-ui` | `.github/skills/ui-conventions/SKILL.md` |

규칙 변경 시 브리지 파일이 아니라 저장소 원본을 수정한다. 프로젝트 위치나 폴더명이 바뀌어도 `AGENTS.md`와 `.github/skills/**` 구조가 유지되면 브리지는 그대로 동작한다. Codex 런타임이 프로젝트 스킬을 직접 읽지 못하는 경우 `scripts/sync-codex-skills.ps1`로 계정 홈에 동기화한 뒤 새 세션을 시작한다.

---

## 9. 프로젝트 맵 유지보수 워크플로

| 목적 | 명령 |
|------|------|
| 전체 저장소 인벤토리 갱신 | `npm run project-map:update` |
| 생성 결과 drift 검사 | `npm run check:project-map` |

소스·설정·문서 파일을 추가, 이동, 삭제한 작업은 완료 전에
`.github/agents/project-map-maintainer.agent.md` 역할로 문서 pass를 위임한다. 이
에이전트는 자동 생성 블록을 갱신한 뒤 현재 diff와 모듈 책임 표, 데몬 목록,
데이터 흐름을 비교한다. 자동 블록 밖의 설명은 실제 코드 책임이 바뀐 경우에만
수정한다.

자동 생성기는 Git 추적 파일과 `.gitignore`에 걸리지 않은 작업 트리 파일을
수집하되 Serena 런타임 로그와 Graphify 생성물은 제외한다. 따라서
`node_modules`, Cargo `target`, 빌드 산출물, 로컬 데이터, 민감 설정은
프로젝트 맵에 포함하지 않는다.

---

## 10. 에이전트 코드 탐색 역할

| 역할 | 프로젝트 파일 | 책임 |
|---|---|---|
| `project_mapper` | `.codex/agents/project_mapper.toml` | 전체 파일 인벤토리와 구조 설명 동기화 |
| `symbol_navigator` | `.codex/agents/symbol_navigator.toml` | Serena 기반 정의·참조·영향 범위 추적 |
| `helper_curator` | `.codex/agents/helper_curator.toml` | Graphify 후보 탐색과 Serena 동등성 검증 후 공용화 |

Serena 설정과 공유 메모리는 `.serena/`, Graphify 코드 그래프는
`graphify-out/`에 둔다. Graphify의 캐시·로컬 경로·분석 중간 산출물은
Git에서 제외하고, 공유 그래프·보고서·manifest·source-hashes를 유지한다. 세 역할의 위임
조건, 민감 파일 제외, 갱신·검증 명령은 `docs/agent-tooling.md`를 따른다.
