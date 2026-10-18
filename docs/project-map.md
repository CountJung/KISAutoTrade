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
├── .cargo/
│   └── config.toml
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
│   │   ├── stock_analyst.toml
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
│   │   ├── release-gate.yml
│   │   ├── release.yml
│   │   └── security.yml
│   ├── codex-instructions.md
│   ├── copilot-instructions.md
│   └── dependabot.yml
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
│   ├── research/
│   │   ├── autonomous-corrections-20261018/
│   │   │   ├── compare.py
│   │   │   ├── comparison.json
│   │   │   └── report.md
│   │   ├── soxq-3m-20261003/
│   │   │   ├── candles.csv
│   │   │   ├── charts.html
│   │   │   ├── p1-01-cost20-results.json
│   │   │   ├── p1-01-results.json
│   │   │   ├── p1-01-validation.md
│   │   │   ├── p1-02-results.json
│   │   │   ├── p1-02-validation.md
│   │   │   ├── p1-03-cost20-results.json
│   │   │   ├── p1-03-results.json
│   │   │   ├── p1-03-validation.md
│   │   │   ├── p1-04-results.json
│   │   │   ├── p1-04-validation.md
│   │   │   ├── p1-05-results.json
│   │   │   ├── p1-05-validation.md
│   │   │   ├── replay-input.json
│   │   │   ├── report.md
│   │   │   ├── results-cost20.json
│   │   │   ├── results.json
│   │   │   ├── runner.rs
│   │   │   └── validation.json
│   │   └── strategy-review-20261018/
│   │       ├── analyze.py
│   │       ├── input.json
│   │       ├── prepare_data.py
│   │       ├── report.md
│   │       ├── results.jsonl
│   │       ├── runner.rs
│   │       ├── summary.json
│   │       └── validation.json
│   ├── tasks/
│   │   ├── TASK_TEMPLATE.md
│   │   ├── appledouble-and-soxq-p1-03.md
│   │   ├── autonomous-backlog-20261018.md
│   │   ├── documentation-layout-20261018.md
│   │   ├── execution-run-reservation-20261018.md
│   │   ├── frontend-hooks-split-20261018.md
│   │   ├── live-completed-daily-history-20261018.md
│   │   ├── project-health-and-soxq-p1-01.md
│   │   ├── provider-candle-contract-20261018.md
│   │   ├── soxq-p1-04.md
│   │   ├── soxq-p1-05.md
│   │   ├── strategy-configuration-preservation-20261018.md
│   │   ├── strategy-review-20261018.md
│   │   └── trading-module-split-20261018.md
│   ├── ARCHITECTURE.md
│   ├── HARNESS_MAP.md
│   ├── MASTER_PLAN.html
│   ├── MasterPlan.md
│   ├── PROJECT_MAP.md
│   ├── agent-tooling.md
│   ├── coding-guide.md
│   ├── discord-setup-guide.md
│   ├── ipc-commands.md
│   ├── leveraged-trend-hold-parameter-guide.md
│   ├── mock-trading-e2e-checklist.md
│   ├── project-map.md
│   ├── release-security.md
│   ├── security-mac-handoff-2026-10-06.md
│   ├── security-storage-handoff-2026-10-06.md
│   ├── todo.html
│   ├── todo.md
│   ├── toss-openapi.md
│   ├── toss-readonly-small-order-checklist.md
│   └── user-guide.md
├── scripts/
│   ├── check-fsd-imports.mjs
│   ├── graphify.mjs
│   ├── project-map.mjs
│   ├── release-version.mjs
│   ├── render-todo.mjs
│   ├── render-todo.test.mjs
│   ├── setup-local.sh
│   ├── sync-codex-skills.ps1
│   ├── verify-lockfiles.mjs
│   ├── verify-release-artifacts.mjs
│   └── verify-toss-openapi.mjs
├── src/
│   ├── api/
│   │   ├── accountHooks.ts
│   │   ├── backendEvents.ts
│   │   ├── budgetHooks.ts
│   │   ├── commands.ts
│   │   ├── databaseHooks.ts
│   │   ├── historyHooks.ts
│   │   ├── hooks.ts
│   │   ├── logHooks.ts
│   │   ├── marketHooks.ts
│   │   ├── orderHooks.ts
│   │   ├── queryKeys.ts
│   │   ├── settingsHooks.ts
│   │   ├── strategyHooks.ts
│   │   ├── tradingHooks.ts
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
│   │   │   │   ├── autoTradingBudgetSection.tsx
│   │   │   │   ├── brokerRateLimitSection.tsx
│   │   │   │   ├── databaseManagementSection.tsx
│   │   │   │   ├── profileDialogs.tsx
│   │   │   │   ├── profileUtils.ts
│   │   │   │   └── section.tsx
│   │   │   └── index.ts
│   │   ├── strategy/
│   │   │   ├── model/
│   │   │   │   ├── candlePageCollector.ts
│   │   │   │   └── experimentStore.ts
│   │   │   ├── ui/
│   │   │   │   ├── Page.tsx
│   │   │   │   ├── bollingerControls.tsx
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
│   │   │   ├── candlePageTypes.ts
│   │   │   ├── commands.ts
│   │   │   ├── databaseCommands.ts
│   │   │   ├── databaseTypes.ts
│   │   │   ├── index.ts
│   │   │   ├── liveDailyHistoryTypes.ts
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
│   │       ├── LiveDailyHistoryPanel.tsx
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
│   │   │   │   ├── chart.rs
│   │   │   │   ├── chart_tests.rs
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
│   │   │   ├── candles.rs
│   │   │   ├── domain.rs
│   │   │   ├── kis.rs
│   │   │   ├── mod.rs
│   │   │   └── rate_limit.rs
│   │   ├── commands/
│   │   │   ├── candles/
│   │   │   │   └── tests.rs
│   │   │   ├── strategy_preview/
│   │   │   │   ├── daily_event_tests.rs
│   │   │   │   ├── generic.rs
│   │   │   │   ├── generic_tests.rs
│   │   │   │   ├── preparation_tests.rs
│   │   │   │   └── tests.rs
│   │   │   ├── toss/
│   │   │   │   └── small_order.rs
│   │   │   ├── trading/
│   │   │   │   ├── daemon/
│   │   │   │   │   ├── fence/
│   │   │   │   │   │   └── tests.rs
│   │   │   │   │   ├── flow/
│   │   │   │   │   │   └── tests.rs
│   │   │   │   │   ├── fence.rs
│   │   │   │   │   └── flow.rs
│   │   │   │   ├── daily_history/
│   │   │   │   │   ├── jobs/
│   │   │   │   │   │   └── tests.rs
│   │   │   │   │   ├── jobs.rs
│   │   │   │   │   └── tests.rs
│   │   │   │   ├── balance.rs
│   │   │   │   ├── daemon.rs
│   │   │   │   ├── daily_history.rs
│   │   │   │   ├── history.rs
│   │   │   │   ├── positions.rs
│   │   │   │   ├── risk_restore.rs
│   │   │   │   └── risk_restore_tests.rs
│   │   │   ├── accounts.rs
│   │   │   ├── archive.rs
│   │   │   ├── budget.rs
│   │   │   ├── candles.rs
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
│   │   │   ├── budget.rs
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
│   │   │   ├── configuration/
│   │   │   │   └── tests.rs
│   │   │   ├── order/
│   │   │   │   ├── configuration/
│   │   │   │   │   └── tests.rs
│   │   │   │   ├── fills/
│   │   │   │   │   ├── reconciliation/
│   │   │   │   │   │   └── tests.rs
│   │   │   │   │   ├── collector.rs
│   │   │   │   │   ├── provider_mode.rs
│   │   │   │   │   └── reconciliation.rs
│   │   │   │   ├── submission/
│   │   │   │   │   ├── positions.rs
│   │   │   │   │   ├── prepare.rs
│   │   │   │   │   ├── provider.rs
│   │   │   │   │   ├── records.rs
│   │   │   │   │   └── run_tests.rs
│   │   │   │   ├── budget.rs
│   │   │   │   ├── configuration.rs
│   │   │   │   ├── conflicts.rs
│   │   │   │   ├── execution_run.rs
│   │   │   │   ├── fills.rs
│   │   │   │   ├── health.rs
│   │   │   │   ├── legacy.rs
│   │   │   │   ├── pending.rs
│   │   │   │   ├── submission.rs
│   │   │   │   └── test_support.rs
│   │   │   ├── simulation/
│   │   │   │   ├── assessment.rs
│   │   │   │   ├── atr.rs
│   │   │   │   └── tests.rs
│   │   │   ├── strategy/
│   │   │   │   ├── breakout/
│   │   │   │   │   ├── day_event_tests.rs
│   │   │   │   │   ├── high_tests.rs
│   │   │   │   │   └── reconfigure_tests.rs
│   │   │   │   ├── classic/
│   │   │   │   │   └── reconfigure_tests.rs
│   │   │   │   ├── leveraged_trend_hold/
│   │   │   │   │   ├── bollinger.rs
│   │   │   │   │   ├── buffer_tests.rs
│   │   │   │   │   ├── exits.rs
│   │   │   │   │   ├── indicators.rs
│   │   │   │   │   ├── live.rs
│   │   │   │   │   ├── preview.rs
│   │   │   │   │   ├── reconfigure_tests.rs
│   │   │   │   │   ├── session.rs
│   │   │   │   │   └── tests.rs
│   │   │   │   ├── ma_cross/
│   │   │   │   │   └── reconfigure_tests.rs
│   │   │   │   ├── manager/
│   │   │   │   │   ├── configuration.rs
│   │   │   │   │   ├── live_history.rs
│   │   │   │   │   └── live_history_tests.rs
│   │   │   │   ├── mean_trend/
│   │   │   │   │   └── reconfigure_tests.rs
│   │   │   │   ├── price_condition/
│   │   │   │   │   └── reconfigure_tests.rs
│   │   │   │   ├── sequence/
│   │   │   │   │   └── reconfigure_tests.rs
│   │   │   │   ├── breakout.rs
│   │   │   │   ├── classic.rs
│   │   │   │   ├── core.rs
│   │   │   │   ├── leveraged_trend_hold.rs
│   │   │   │   ├── ma_cross.rs
│   │   │   │   ├── manager.rs
│   │   │   │   ├── mean_trend.rs
│   │   │   │   ├── price_condition.rs
│   │   │   │   ├── sequence.rs
│   │   │   │   ├── state.rs
│   │   │   │   └── tests.rs
│   │   │   ├── configuration.rs
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
│   ├── tests/
│   │   └── lth_replay_clock.rs
│   ├── Cargo.toml
│   ├── build.rs
│   └── tauri.conf.json
├── tests/
│   └── e2e/
│       ├── fixtures/
│       │   └── strategyResearchFixture.ts
│       ├── candle-pagination.spec.ts
│       ├── live-daily-history.spec.ts
│       ├── settings-database.spec.ts
│       ├── strategy-atr-readiness.spec.ts
│       ├── strategy-configuration-errors.spec.ts
│       ├── strategy-replay-assessment.spec.ts
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
├── tsconfig.json
├── tsconfig.node.json
└── vite.config.ts
```
<!-- project-map:generated:end -->

---

### 문서 위치와 책임

루트에는 `README.md`, `AGENTS.md`, `CLAUDE.md`를 문서 진입점으로 유지한다.
일반 운영·설계·할일 문서는 `docs/`, 작업 기록은 `docs/tasks/`에 모은다.
도구가 요구하는 `.github/`, `.codex/`, `.claude/`, `.serena/` 설정·스킬과
빌드 설정·라이선스·앱 진입 파일은 각 기존 위치를 유지한다.

| 문서 | 책임 |
|---|---|
| `README.md` | 프로젝트 소개·실행과 운영 문서로의 루트 진입점 |
| `AGENTS.md`, `CLAUDE.md` | 에이전트 작업 규칙과 호환 진입점 |
| `docs/MASTER_PLAN.html` | 사람용 단계 계획과 금융 안전 게이트 |
| `docs/PROJECT_MAP.md` | 작업별 수정 시작점과 주요 동기화 묶음의 수동 탐색 지도 |
| `docs/project-map.md` | 이 문서: 자동 생성 전체 인벤토리와 상세 모듈 책임·데이터 흐름 |
| `docs/ARCHITECTURE.md` | 현재 구현된 런타임·주문·리스크·저장 경계 |
| `docs/HARNESS_MAP.md` | 변경 유형별 검증 명령과 완료 게이트 |
| `docs/todo.md` | 상태·근거·위험·완료 조건이 있는 개선 백로그의 원본 |
| `docs/todo.html` | 원본의 열린 항목을 상태별 색상·검색·필터로 읽는 자동 생성 문서 |
| `docs/tasks/` | 작업별 범위·실행·검증·전달 기록, `TASK_TEMPLATE.md` 템플릿 |

`docs/PROJECT_MAP.md`의 밑줄과 `docs/project-map.md`의 하이픈을 구분한다.
문서 정리는 앱 모듈·IPC·데몬·연구 데이터의 책임을 변경하지 않는다.

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
| `api/hooks.ts` | 기존 import를 유지하는 TanStack Query 훅 facade. 도메인별 hook 모듈과 기존 budget/database hook, `KEYS`, `useBackendEvents`를 re-export하며 훅 구현을 직접 소유하지 않음 |
| `api/accountHooks.ts` | 앱 계정 설정·설정 확인·국내/해외/broker 잔고·프로파일 CRUD/전환·거래 환경 감지·Toss 연결/계좌 조회 hook |
| `shared/api/liveDailyHistoryTypes.ts` | `TradingStatus.dailyHistory`의 읽기 전용 준비 진단 타입. 기존 응답의 필드 누락은 준비 완료로 추정하지 않도록 optional 호환 계약 유지 |
| `shared/ui/LiveDailyHistoryPanel.tsx` | Dashboard/Strategy가 기존 상태 query로 공유하는 완료 일봉 준비 패널. 실행/활성 profile·broker/account 범위를 확인하고 상태·확보량·시장 날짜·출처·실패/재시도 사유 표시. 최대 512상태를 요약하고 실제 행은 3개씩 표시하며 프로파일 없는 legacy KIS 실행은 일치 계좌의 실패 진단만 허용 |
| `api/marketHooks.ts` | 국내/해외/Toss 시세·차트·검색·종목 목록 갱신·시장 일정/유의사항·환율 hook과 해외 차트 preset. 갱신 설정은 `settingsHooks.ts::useRefreshConfig`를 직접 참조. `useCandlePage`는 프로파일·broker/account·종목·간격·거래소·커서별 공통 페이지 조회 계약을 사용 |
| `api/orderHooks.ts` | 국내/해외 수동 주문·Toss 주문 전 검증/정정·소액 검증·미체결 조회 hook과 주문 후 관련 query 무효화 |
| `api/strategyHooks.ts` | scope별 전략 목록·수정, 일반 replay와 LTH preview mutation hook |
| `api/settingsHooks.ts` | 업데이트 확인·웹 설정·갱신 주기/상세 설정·broker rate-limit 상태·Discord 테스트 hook |
| `api/historyHooks.ts` | 당일/기간별 provider 체결·로컬 거래·통계와 거래 보관 설정/현황 hook |
| `api/tradingHooks.ts` | 자동매매 상태·시작/정지·매수 정지 해제·포지션·리스크 설정·비상정지 hook |
| `api/logHooks.ts` | 로그 설정·최근 로그 조회·프런트 로그 기록 hook과 `frontendLogger` |
| `api/queryKeys.ts` | TanStack Query `KEYS` 단일 원천 (`hooks.ts`에서 하위 호환 re-export) |
| `api/backendEvents.ts` | Tauri backend event 구독 → 환율/잔고 Query 캐시 갱신 |
| `api/budgetHooks.ts` | broker/account별 자동매매 예산 조회·폴링과 저장 후 해당 scope Query 캐시 갱신 |
| `api/databaseHooks.ts` | DB 설정/상태/이관 Tauri-only TanStack Query hooks |
| `widgets/app-shell/` | 전체 앱 레이아웃, ThemeProvider, responsive navigation, 좌측 사이드바 자동매매 시작/정지 전역 조작 |
| `widgets/stock-chart/` | 국내/해외/Toss 캔들 차트 |
| `pages/settings/ui/Page.tsx` | 테마, 데이터 갱신 주기, 로그/체결 보관, 웹 포트, 종목 목록, 리스크, Discord 설정 route 조립 |
| `pages/settings/ui/autoTradingBudgetSection.tsx` | 선택 계좌의 KRW/USD 자동매매 배정액·현금·예약액·사용 가능액·차단 사유 표시와 정지 상태 예산 편집 |
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
| `pages/strategy/ui/strategyPreviewPanel.tsx` | 일반/가격조건 전략 카드에서 `candlePageCollector`로 KIS 일/주/월봉 또는 Toss 1분/일봉을 순차 조회하고 요청 평가봉과 그 이전 사전자료를 나눠 편집값·비용 가정과 함께 `preview_strategy`에 전달. 실제 페이지 수·종료 사유·미완료/무효/중복 제외 수와 확보한 사전자료를 표시한다. 강한 종가·변동성 확장은 일봉 선택만 제공하고 날짜 이벤트와 종가 체결 근사를 안내한다. 실제 평가 기간·사전자료 수·준비 상태·평가 불가/조건 불충족/무체결을 표시하고 티커/봉/구간/파라미터/수량/broker/account/가정 변경 시 결과 무효화 |
| `pages/strategy/ui/strategyResearchPanel.tsx` | 초기자본·수수료·세금·슬리피지·환율·리스크·학습구간 입력, 수익률/MDD/승률/손익비/turnover/exposure, 원시 신호와 주문 가능/체결 구분, equity curve, 거래 목록, in/out-of-sample, A/B 비교 UI. LTH assessment에 따라 일봉·관측 부족·과거 평가 정보 없는 결과의 성과 표시와 저장/비교를 차단하고, 분봉은 기간 전체 성과와 구분한 표본으로 안내. 연구 전용 ATR 수량 산정 토글·위험 예산·손절 배수 입력과 전략 지표 준비와 별도인 ATR 준비/미준비·과거 응답의 미확인 상태를 표시 |
| `pages/strategy/model/candlePageCollector.ts` | 공통 페이지 API를 최대 8회 요청하고 최대 500봉만 유지한다. 요청 평가봉을 우선 배정한 뒤 이전 최대 252봉을 별도 사전자료로 분리하며 시각 기준 중복 제거·정렬·커서 반복/무진전/제공 자료 끝/일부 오류 종료를 처리한다. 프로파일·계좌 scope와 asOf/source/완료 정책 변경 시 결과를 폐기하고, KIS 해외 거래소 fallback은 첫 성공 페이지 이전 provider 오류에만 허용한다. |
| `shared/api/candlePageTypes.ts` | desktop IPC와 Web GET에 공통인 `CandlePageInput`/`CandlePageView` 계약과 scope·커서·완료 정책·제외 수 타입. 기존 Vec 차트 계약과 구분 |
| `pages/strategy/model/experimentStore.ts` | credential을 저장하지 않고 broker/account/strategy/symbol scope별 A/B 결과·전략 버전·파라미터·데이터 범위/source·비용 가정·생성 시각을 localStorage에 최대 2개 저장 |
| `pages/strategy/ui/bollingerControls.tsx` | 레버리지 추세 보유 전략의 선택적 볼린저 보강 활성화와 기간·배수·스퀴즈·ATR 설정 입력 |
| `pages/strategy/ui/leveragedTrendHoldPreviewChart.tsx` | 레버리지/일반 전략 미리보기의 캔들·종가선·signal marker, 모바일 가로 패닝·핀치 확대/축소와 버튼식 줌 표시. offset 포함 시각은 실제 instant로 파싱하고 compact 분봉은 기존 KST 규칙으로 해석 |
| `pages/history/ui/Page.tsx` | 활성 broker scope, 자동매매 체결 기록과 기간별 통계 조회, provider 원본 trace 표시 |
| `pages/log/ui/Page.tsx` | 로그 레벨/검색 필터, provider trace 토큰 chip 표시 |

### Backend (Rust)

| 모듈 | 책임 |
|------|------|
| `lib.rs` | Tauri Builder + window-state 플러그인 + 6개 백그라운드 데몬 spawn + `on_window_event` (종료 안전 처리) |
| `commands.rs` | AppState + IPC command facade, 공통 helper |
| `commands/accounts.rs` | 계좌 프로파일 CRUD/활성화, KIS 국내·해외 잔고, broker holdings view. 프로파일 적용 전 실행을 무효화하고 자료 로드 뒤 manager → OrderManager 잠금에서 scope·client 교체·tracker 초기화·전략 복원을 함께 게시 |
| `commands/archive.rs` | 거래 파일 보관 설정, 보관 통계, 오래된 trade/log 파일 purge IPC |
| `commands/budget.rs` | 활성 broker/account 검증, 자동매매 전용 예산 조회·변경 IPC. 변경은 자동매매 정지와 저장소/프로파일 변경 잠금 경계에서 수행 |
| `commands/database.rs` | 인증 없는 REST에는 노출하지 않는 Tauri-only DB 관리/이관 IPC |
| `commands/market.rs` | KIS 국내/해외 시세, 차트, 종목 검색, 해외 주문 사전 검증 IPC |
| `commands/candles.rs` | 공통 읽기 전용 페이지 IPC와 Web 공유 서비스. 활성 profile/broker/account와 KIS client 일치, 커서의 scope/asOf/provider를 검증한다. OHLCV·통화를 검증하고 서울/뉴욕 현재 시장 기간보다 이전인 일/주/월봉 또는 asOf 이하 종료 시각의 분봉만 정렬·중복 제거해 반환한다. 완료 정책과 미완료/무효 제외 수를 노출하며 기존 Vec 차트 IPC는 유지 |
| `commands/candles/tests.rs` | 시장 날짜·주/월 경계, DST 날짜와 분봉 offset instant, 미완료/무효/중복 제거, 고정 asOf 커서 및 profile/account/broker 불일치의 fail-closed fixture |
| `commands/orders.rs` | 수동 주문 제출 IPC |
| `commands/records.rs` | 체결/거래/통계 조회, Discord config 저장, frontend log 저장 IPC |
| `commands/settings.rs` | app config/check_config, refresh interval, log/web 설정, USD/KRW 환율 IPC |
| `commands/strategy_preview.rs` | 일반 preview 공개 surface를 `strategy_preview/generic.rs`에서 re-export하며 공통 변환 helper와 Toss 레버리지 preview를 유지한다. 레버리지 preview는 활성 profile/account를 검증하고 분봉 시작 이전 완료 일봉을 별도 맥락으로 전달한다. 일봉 요청은 합성 장중 관측 없이 가격 진단만 반환하고, 분봉 요청만 timed 전략을 실행하고, 주문 시도마다 backtest의 실제 잔여 수량·평균 체결가 snapshot을 피드백한다. `assessment`로 진단·표본·자료 부족과 미검증 시간대/세션을 분리하며 분봉 warmupCount는 0, 일봉 맥락 수는 dailyContextBars로 제공 |
| `commands/strategy_preview/generic.rs` | 사전자료 `historyCandles`와 평가 `candles`를 별도로 정규화하고 합계 최대 500봉·평가 시작 이전 경계를 검증한다. 기본 warmup은 0이며 명시적 `warmupCount` 호환 경로를 유지한다. 일봉은 날짜 시작·종가 OHLC 평가·실행 포지션 피드백·완료봉 순서로 처리하며 강한 종가·변동성 확장의 비일봉 입력은 거부한다. `preparation`은 처리 후 지표 준비 상태·첫 준비 시각과 처리 전 실제 조건 평가 가능 봉 수(`evaluatedBars`)를 구분하며 미지원 값은 `null`이다. generic LTH는 기본/접미 ID 모두 factory 실행 전 `UNSUPPORTED_GENERIC_REPLAY`로 거부하며 지원 12전략 결과는 deterministic이다. 동일 주기의 검증된 사전자료를 `run_backtest_with_history`에도 전달해 ATR을 준비하며 주문·수익률 평가에는 포함하지 않는다. 공통 initializer와 종가 체결 근사, broker/account scope, 비용/리스크 backtest와 사전자료/평가 입력 hash를 보존한다. v9 분봉은 offset 포함 RFC3339 또는 KST compact 시각을 고정 정밀도 UTC instant로 정규화하고 사전자료 겹침·동일 instant 중복을 거부하며 누락 관측은 보간하지 않는다. |
| `commands/strategy_preview/generic_tests.rs` | generic 분봉의 UTC/offset 시각 정렬·동일 instant 중복·사전자료 겹침 거부와 실제 관측 간격 보존 회귀 fixture |
| `commands/strategy_preview/tests.rs` | 일봉 정보 공개 시점, session 경계, 미래 봉 변경이 이전 신호에 영향을 주지 않는 deterministic fixture |
| `commands/strategy_preview/preparation_tests.rs` | 동일 평가 기간, 사전자료 겹침·중복 입력 거부, 지표 준비 상태·실제 조건 평가봉 수와 사전자료 부족 fixture. generic LTH의 기본/접미 ID·간격·history 조합과 malformed 입력의 factory 이전 명시 거부 검증 |
| `commands/strategy_preview/daily_event_tests.rs` | generic 일봉의 전일 조건·이전 범위, 실행 차단 피드백, 평가 중 준비, 미래 봉 변경 불변과 비일봉 거부 회귀 검증 |
| `commands/toss.rs` | Toss accountSeq 조회, 연결 진단, 주문 전 preflight view facade, 접수 주문 목록 조회와 정정 command |
| `commands/toss/small_order.rs` | 호환용 Toss 1주 소액매매 검증 endpoint. 실거래 동의/최종 확인/최대 허용금액/preflight/open-order scan 후 시장가 매수 제출과 주문·체결 기록 저장. 현재 UI는 일반 수동주문 경로를 사용 |
| `commands/toss_market.rs` | Toss 시세 snapshot, 종목 유의사항, market-calendar override, candles chart command |
| `commands/trading.rs` | 자동매매 상태/시작/정지·매수 정지 해제의 4개 IPC와 시작 순서·scope 검증을 조립하는 facade. daemon·잔고·위험 복원·히스토리 initializer의 기존 공개/내부 re-export 경로를 유지. 시작 준비 뒤 검증한 profile/client와 revision으로 실행 epoch를 게시하고 실행 플래그를 켠다. 정지는 실행과 일봉 generation을 무효화하며 예약·pending 추적은 보존. 상태에 완료 일봉 진단 포함 |
| `commands/trading/positions.rs` | 활성 broker를 따라 KIS/Toss 실제 보유 수량·평균가를 전략 상태에 복원하고 broker별 보유 값 파싱을 담당 |
| `commands/trading/balance.rs` | KIS 국내/해외 및 Toss 계좌의 현금·평가잔고를 조회해 리스크 기준 총잔고를 KRW로 환산 |
| `commands/trading/risk_restore.rs` | 실행 broker/account scope의 당일 체결·주문 ledger에서 실현손익·주문 횟수·전략/종목별 손익을 복원하고 runtime을 저장. 조회·저장 실패를 호출자에게 전달 |
| `commands/trading/history.rs` | 활성 전략의 broker별 과거 OHLC 조회·통화 단위 변환과 일봉/분봉 warmup dispatch. live 초기화용 `calculate_atr`를 소유하며 replay 전용 ATR 준비 모듈과 구분. 52주 전용 종목은 legacy seed를 생략하고 공유 종목의 다른 전략은 기존 seed를 받는다. Toss 실행 scope의 프로파일이 없으면 KIS로 대체하지 않고 초기화를 중단 |
| `commands/trading/daemon.rs` | 영구 `run_trading_daemon`과 시세 polling 사이클, KIS/Toss 가격 소스·세션 판단·장 종료 대기·해외 거래소 조회 helper를 소유. 실행에 고정한 provider로 조회하고 fence로 오래된 잔고·quote·종목명·포지션 결과와 전략 복원을 폐기한다. 시장 대기 전 일봉 작업을 관리하고 tick 직전 시장 날짜 변경 재확인. facade의 기존 export를 통해 `lib.rs`에서 실행 |
| `commands/trading/daemon/fence.rs` | running → StrategyManager → OrderManager 잠금 순서의 실행 snapshot 캡처·현재 context 검사. provider await는 잠금 밖에서 수행하고 전략 tick/복원은 현재 epoch·revision일 때만 적용. 잔고 동기화 상태·일 초기화도 실행 범위 검증 후 반영 |
| `commands/trading/daemon/fence/tests.rs` | quote·종목명·포지션 대기, 설정 변경·정지/재시작과 stale 전략 상태 복원 경계의 합성 회귀 검증 |
| `commands/trading/daemon/flow.rs` | 설정 편집 대기와 실제 시장 종료를 구분하고, 장 종료로 제출하지 않은 신호 묶음의 실제 포지션 복원·현재 revision 정산을 연결. 설정 저장 대기는 짧게 재확인하여 장 종료의 긴 대기로 바꾸지 않음 |
| `commands/trading/daily_history.rs` | 공통 완료 페이지로 live 52주 사전자료 수집. 작업당 최대 8회 요청·512봉 유지, 이전 시장 날짜·OHLC·scope/asOf/source 검증과 거래소 fallback·중복·커서 무진전·부분 오류 종료. 마지막 완료봉을 다시 제외하지 않으며 preview의 500봉 상한과 분리 |
| `commands/trading/daily_history/jobs.rs` | 실행 프로파일에 고정한 background scheduler. 같은 profile/종목의 여러 설정을 최대 요구량 1회 조회로 공유하고 동시 2작업만 실행. 완료 handle만 수거하며 제거/새 generation/config/scope/시장 날짜의 과거 작업 취소. 실패/부족은 30~900초 backoff, 정지/drop 시 작업 중단 |
| `commands/trading/daily_history/{tests.rs,jobs/tests.rs}` | 합성 DTO/fetch로 실제 252/512봉·중복/미완료/오류·scope drift·수집 상한, 공유 조회·동시 2작업·backoff·대기 중 손절·정지/재시작 결과 폐기 검증 |
| `commands/strategy.rs` | 포지션 조회, 전략 목록/수정 IPC. 공통 `trading/configuration.rs`에 검증·저장·보유 상태 보존과 revision 게시를 위임하며 전략 view는 `trading/views.rs::build_strategy_view()` 재사용 |
| `commands/risk.rs` | 리스크 설정/비상정지 IPC와 pending 주문 view |
| `trading/strategy.rs` | 전략 facade. 공개 import 경로를 유지하면서 하위 전략 모듈을 re-export |
| `trading/strategy/core.rs` | `Signal`, `StrategySignal`, `BrokerPositionSnapshot`, `StrategyConfig`, `HistoryReadiness`, `Strategy` trait와 live/preview 공통 `initialize_strategy_warmup()`. 일봉 OHLC 초기화 기본 훅은 종가를 historical initializer에 전달한다. replay 날짜 시작·완료봉 훅은 기본 no-op, 종가 평가 훅은 기존 `on_tick()` 위임이다. 준비·평가 가능 상태와 내부 보유 여부의 미지원 값은 `None`이다. 기본 false인 `reconfigure_preserving_position` opt-in으로 같은 ID/broker/account 설정 변경의 보유 상태 보존 계약 제공 |
| `trading/strategy/manager.rs` | `StrategyManager`, `build_strategy()`, config 적용과 live warmup dispatch. legacy warmup의 최신 1봉 제외는 유지하되 새 52주 live 실행은 완료 페이지 경계로 준비. config 변경·복원·reset은 일봉 generation 무효화. 13개 전략은 같은 ID/scope 설정 변경 시 opt-in 훅으로 보유 상태와 유효한 관측을 보존한다. 프로파일 게시 경로는 같은 계좌 문자열이어도 새 인스턴스를 구성 |
| `trading/strategy/manager/configuration.rs` | 설정 편집 Weak 예약의 활성 여부 관리, 현재 revision의 미완료 신호 확인·정산과 미제출 신호 복원. 전략 내부 보유 보호 중인 대상 제거·시장 변경을 저장 전에 거부 |
| `trading/configuration.rs` | IPC/Web 공통 설정 변경 서비스. 작업 소유 task와 `strategy_update_lock`으로 호출자 취소와 동시 저장을 제어한다. 생산 constructor로 대상 정규화 후 manager → OrderManager에서 편집 예약을 만들고 잠금 밖에서 저장. 성공 후 identity 재검증·보유 상태 보존·revision 게시, 사후 검증 실패 시 원본 저장값 복구와 복구 실패의 영속화 차단 |
| `trading/configuration/tests.rs` | 실제 manager/store와 합성 주문 fixture로 저장 실패·지연·호출자 취소·보유 대상 제거·프로파일 변경·사후 복구 경계를 검증 |
| `trading/strategy/{classic,breakout,mean_trend,sequence,ma_cross,price_condition,leveraged_trend_hold}/reconfigure_tests.rs` | 전략별 설정 변경의 보유·손절·관측 보존, 파라미터 재해석과 다른 ID/scope 상태 격리를 검증 |
| `trading/strategy/manager/live_history.rs` | run generation·전체 config fingerprint·profile/broker/account·종목·서울/뉴욕 시장 날짜 ticket 검증과 52주 전용 완료 OHLC 적용. 실제 대상 511개+초과 진단 1행 상한, 준비 무효화·실패/부족/재시도 상태 및 최신 완료봉의 1~10일 freshness 관리. 보유 손절을 유지하고 LTH 분봉 initializer 미호출 |
| `trading/strategy/manager/live_history_tests.rs` | 마지막 완료 high/close·다음날 고가 만료·252/512봉·대상 cap·재시작/config ABA·scope/profile 변경·quote 중 자정·DST·실패 중 손절·동일 scope 설정/rollback 보유 보존·무관 전략 불변 fixture |
| `trading/simulation.rs` | deterministic simulated portfolio, 공통 TradeGuard/RiskManager 기반 주문 가능 판정, 비용·환율·equity·MDD·turnover·exposure·고정 chronological in/out-of-sample 지표. `run_backtest_with_history`는 동일 주기 과거 관측으로 위험 지표만 준비하고 주문·equity는 평가 구간에서만 만든다. ATR 미준비·변동폭 0 또는 거래당 위험 예산 0이면 선택적 ATR 수량 산정의 신규 매수를 차단하고 보유분 매도는 유지한다. preview command가 engine/strategy/source/scope/params/가정/warmup 경계와 실제 replay OHLCV fingerprint를 hash한다. v9 replay 시각은 RFC3339 offset의 실제 instant를 우선 사용한다. |
| `trading/simulation/assessment.rs` | optional `ReplayAssessmentView`로 LTH의 dailyDiagnostic/notEvaluable과 intradaySample/sampleOnly를 구분한다. 분봉 관측 0개도 notEvaluable이며 일봉 맥락·장중 봉 수와 미검증 시각/세션·모델 한계를 제공 |
| `trading/simulation/atr.rs` | 이전 종가를 포함한 True Range 14개를 bounded ring에 보관하고 단순 평균 ATR을 계산한다. 같은 주기 관측 15봉을 요구하며 갭을 보존하고 누락 봉을 보간하지 않는다. `AtrPreparationView`로 사전 관측 수·시작/종료 준비·최초 준비 시각·미준비 평가봉 수를 제공 |
| `trading/simulation/tests.rs` | 비용·리스크·실행 차단, 날짜별 손실 상태, ATR 준비·위험 예산·실제 포지션과 동일 replay/report/hash 회귀 fixture |
| `trading/strategy/state.rs` | per-symbol 전략 버퍼 상한 helper. user-param 기반 `VecDeque` OOM 방지 |
| `trading/strategy/{classic,breakout,mean_trend}.rs` | RSI/모멘텀/이격도, 돌파 계열, 평균회귀/추세필터 전략 구현. `breakout.rs`의 52주 신고가는 완료 일봉 고가의 bounded ring에서 이전 기간 최고가를 구하며 날짜 경계 뒤 완료봉 1개만 적재해 오래된 고가를 만료한다. 반복 tick·분봉 초기화로 일봉을 전진시키지 않고, 강한 종가는 완료일 조건을 다음 평가일에 사용하며 변동성 확장은 보유 상태를 유지한 채 당일 OHLC와 이전 N개 완료 범위(0 포함)의 bounded ring을 구분한다. `mean_trend.rs`의 평균회귀 밴드·추세필터 MA는 기본 훅이 전달한 종가로 초기화한다. |
| `trading/strategy/breakout/high_tests.rs` | 52주 고가의 전체 준비 창·오래된 최고가 만료·신규 완료 고가·미래 참조 방지·반복 tick/분봉 초기화 불변·종목별 경계·보유 손절 보존 회귀 검증 |
| `trading/strategy/breakout/day_event_tests.rs` | 강한 종가·변동성 확장 event 훅의 날짜별 독립 상태, 차단 피드백·실제 보유 손절, 범위 ring 상한과 flat-day 회귀 검증 |
| `trading/strategy/ma_cross.rs` | 이동평균 교차 전략. historical initializer에서 종가 버퍼와 직전 MA를 복원하고 첫 평가봉의 교차 조건과 실제 준비 상태를 제공한다. |
| `trading/strategy/sequence.rs` | 연속 상승/하락·돌파 실패 전략. 사전 종가 버퍼와 준비 상태를 복원하며 initializer에서 `on_tick()`을 호출해 가상 매매 상태를 만들지 않는다. |
| `trading/strategy/leveraged_trend_hold.rs` | 레버리지 전략 공개 타입·config·상태와 bounded buffer 상한을 소유하는 facade. 일봉 맥락과 분봉 지표/반등·실제 보유 상태를 분리하고 하위 모듈의 live/timed 구현을 연결 |
| `trading/strategy/price_condition.rs` | 종목별 가격 조건 전략 구현 |
| `trading/strategy/leveraged_trend_hold/live.rs` | `Strategy` 구현과 live tick/포지션 동기화. 일봉 초기화는 별도 bounded 맥락 ring만 교체하고, 분봉 초기화는 분봉·반등·볼린저 snapshot을 교체하며 실제 보유 포지션은 유지 |
| `trading/strategy/leveraged_trend_hold/preview.rs` | 입력 timestamp 기반 timed replay와 실제 잔여 수량·평균 체결가 snapshot 피드백. 부분 청산 뒤 잔량과 가격 기준을 보존하며 bool callback 호환 경로도 유지한다. EMA/RSI/ADX·반등·볼린저는 실제 장중 관측만 적재하고 일봉 맥락으로 분봉 준비 상태를 만들지 않음 |
| `trading/strategy/leveraged_trend_hold/indicators.rs` | 장중 버퍼 기반 EMA/RSI/ADX와 진입·단계/급반등 조건 계산 |
| `trading/strategy/leveraged_trend_hold/exits.rs` | 최소 보유 관측·초기 위험·수익 보호·진입 실패 청산 조건과 포지션 정리 |
| `trading/strategy/leveraged_trend_hold/session.rs` | live/timed 세션·blackout·시각 파싱. timed 경로는 입력 시각을 사용하지만 거래소 시간대/DST 검증이 없는 고정 KST 모델 |
| `trading/strategy/leveraged_trend_hold/{tests,buffer_tests}.rs` | 기존 진입·청산 회귀와 일봉/분봉 독립 상한·재초기화·보유/실행 피드백 불변, 부분 수량과 평균 체결가 동기화 fixture |
| `trading/strategy/leveraged_trend_hold/bollinger.rs` | 레버리지 전략의 볼린저 스퀴즈·추세 돌파 진입 확인과 중심선/ATR 청산 조건. 마지막 미확정 봉을 제외하고 종목별 캔들 버퍼를 512개로 제한 |
| `tests/lth_replay_clock.rs` | `cfg(test)` live clock 대역 없이 production library의 LTH timed replay를 검증하는 통합 테스트. 입력 시각 기반 세션·blackout·장마감, 비어 있지 않은 반복 신호/체결과 미래 봉 변경 불변 fixture |
| `api/detect.rs` | KIS 토큰 응답 기반 실전/모의 앱키 자동 감지 |
| `api/rest.rs` | KIS REST client facade. rate-limit group을 거쳐 잔고/주문/체결/시세/차트 요청 수행 |
| `api/rest/chart.rs` | 기존 KIS 국내/해외 일·주·월봉 메서드의 request/response 처리. facade의 공통 인증·rate limit·timeout·응답 크기 제한을 재사용하며 국내/해외 차트별 TR-ID와 date anchor를 설정 |
| `api/rest/chart_tests.rs` | 합성 인증값으로 production 차트 request/response recipe, Unicode 오류와 HTTP body cap을 검증하는 build-only/localhost fixture. 실제 provider 응답 검증과 구분 |
| `api/rest/types.rs` | KIS REST 타입, `OrderSide`/`OrderType`, 국내/해외 잔고·체결·시세 응답, 해외 주문 사전 검증 |
| `api/rest/exchange.rs` | 공개 USD/KRW 환율 fallback fetcher (`fetch_usd_krw_rate`) |
| `broker/` | 다중 증권사 공통 타입(`BrokerScope` 포함), adapter trait, `RateLimitScheduler`. KIS 기존 REST 호출을 점진 래핑하고 Toss token/accounts/holdings/market-data/market-info/order client를 수용 |
| `broker/candles.rs` | provider별 typed cursor와 `CandlePageRequest`/`BrokerCandlePage`, KR/US 시장 날짜의 공통 `candle_market_date` 변환. KIS 날짜 anchor와 Toss inclusive timestamp 커서를 구분하고 커서 무진전을 감지한다. `BrokerAdapter::get_candle_page`로 KIS/Toss를 공통 호출하되 기존 `get_candles`는 유지 |
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
| `trading/order.rs` | OrderManager facade와 공유 상태·provider trace. health·pending·legacy·execution_run·submission·fills의 하위 책임을 연결 |
| `trading/order/execution_run.rs` | 비민감 `ExecutionRunContext`(epoch/revision/scope/profile)와 시작 검증에 사용한 client/profile snapshot. 시작·정지·scope 변경은 실행을 무효화하고 설정 revision 갱신은 이전 예약 후보를 폐기. pending·submitting·예산을 지우지 않음 |
| `trading/order/configuration.rs` | 설정 편집 예약의 profile/client/scope identity와 실제 KR/US tracker 보유·pending/submitting/modifying 충돌 검증. 편집 대상의 주문 예약과 같은 scope 수동 holdings 교체를 보류하며 Weak 예약 종료 시 자동 해제 |
| `trading/order/configuration/tests.rs` | 설정 편집 중 주문·holdings 차단, 예약 해제·identity 변경, 실제 보유 대상 제거·시장 변경의 합성 검증 |
| `trading/order/health.rs` | 매수 정지·계좌/체결 동기화·영속화 장애 상태, health view와 일 초기화 |
| `trading/order/pending.rs` | pending 복원·영속화·종료 상태와 취소/수정 예약, 종료 시 예산 정산 |
| `trading/order/legacy.rs` | 기존 단일 manager의 매수/매도 처리와 KIS 재시도 호환 경로. daemon의 shared 제출 경로와 구분 |
| `trading/order/submission.rs` | 수동·자동 주문 제출 조정. 자동 주문의 필수 실행 context를 준비 후·예약 전에 같은 OrderManager 잠금에서 검증하고 stale은 guard·submitting·예산 변경 전에 폐기. 예약 후 provider 응답·pending 처리는 원래 scope로 계속 추적 |
| `trading/order/submission/positions.rs` | 제출용 실제 포지션 snapshot과 수동 주문 전 holdings 갱신. 조회에 고정한 profile/client/scope가 현재와 같은지 OrderManager 잠금에서 확인한 뒤 tracker에 게시 |
| `trading/order/submission/prepare.rs` | 위험·보유수량·주문 유형과 가격 검증, provider별 준비 요청 생성 |
| `trading/order/submission/provider.rs` | 준비 요청 전송. KIS 국내/해외 재시도는 최초 snapshot의 같은 client를 사용하고 Toss 요청은 고정 profile을 사용 |
| `trading/order/submission/records.rs` | 접수·실패 주문과 pending record 생성, provider trace·원래 broker scope 보존 |
| `trading/order/submission/run_tests.rs`, `trading/order/test_support.rs` | 합성 profile·캐시 토큰·localhost provider로 실행 context, 준비/예약 경합, 예약 후 client 고정과 pending 추적 검증. 실제 계좌/provider 검증과 분리 |
| `trading/order/budget.rs` | broker/account와 KRW/USD별 전용 현금·자동매매 소유수량·주문 예약 원장. 주문 전 예약 저장, 접수/체결/종료 반영, 미확인 주문·저장 실패 시 차단 |
| `trading/order/fills.rs` | `OrderManager::on_fill()`의 체결·수수료·포지션·통계·TradeStore 반영과 체결 확인 하위 모듈 연결 |
| `trading/order/fills/collector.rs` | 고정 KIS client/Toss profile의 주문번호·종목·날짜 조회 결과를 공통 누적 체결/종료 상태로 정규화 |
| `trading/order/fills/provider_mode.rs` | 저장된 KIS 주문 TR-ID 8종으로 실전/모의 출처를 확인하고 현재 client 모드와 비교. 반대 모드·누락/미상 출처의 pending 조회 후보·직접 체결·체결 조회의 종료 상태 및 예산 반영을 보류 |
| `trading/order/fills/reconciliation.rs` | 현재 scope의 pending 후보와 provider snapshot을 캡처하고 OrderManager 잠금 밖에서 조회. 적용 전 scope·profile·인증/모드를 재검증하며 다른 계좌 응답과 zero-fill 종료 처리를 보류. 같은 계좌·인증의 실제 체결은 전략 epoch 변경과 독립적으로 반영 |
| `trading/order/fills/reconciliation/tests.rs` | localhost 체결 조회 중 계좌·profile 변경의 결과 보류, 동일 계좌 실행 변경 후 실제 체결 반영, 저장 TR-ID의 실전/모의·미상 출처 및 직접 체결/종료 보류 회귀 검증 |
| `trading/order/conflicts.rs` | 실행 `BrokerScope` 기준 같은 방향/반대 방향 pending 주문 충돌 판정과 provider trace 기반 pending provider 판정 |
| `trading/risk.rs` | 일일 손실 한도, 비상 정지, `record_pnl` |
| `market_hours.rs` | 시장 개장 여부 (KRX 09:00-15:30 / US 22:00-07:00 KST) |
| `server/mod.rs` | axum 웹 서버 route table, ServeDir fallback |
| `server/budget.rs` | `/api/auto-trading-budget` GET/POST. IPC와 같은 scope 검증·정지 조건·예산 view를 사용 |
| `server/market.rs` | 웹 REST KIS 잔고/해외잔고, broker holdings, 현재가, 주문, 차트, 종목 검색/갱신. 공통 캔들 페이지 GET은 `commands/candles.rs`의 동일 서비스와 camelCase view를 재사용 |
| `server/records.rs` | 웹 REST 포지션, 통계/체결 조회, pending 주문, 로그 설정/최근 로그, 체결 보관 설정/통계, 프론트엔드 로그. REST 보관 설정 변경도 IPC와 같이 즉시 purge를 예약 |
| `server/profiles.rs` | 웹 REST 프로파일 CRUD/활성 전환/실전·모의 감지/Toss accountSeq·diagnostic. 적용 전 실행을 무효화하고 manager → OrderManager에서 scope·client·tracker·전략 설정을 함께 게시. 프로파일 view는 IPC `profile_to_view()`를 재사용 |
| `server/toss.rs` | 웹 REST Toss 시세 snapshot, 종목 유의사항, 주문 전 preflight, 호환용 1주 소액매매 검증 endpoint, 접수 주문 조회·정정, market-calendar, candles |
| `server/trading.rs` | 웹 REST 자동매매 상태/시작/정지, 전략 목록/수정. 웹 start는 broker별 설정 검증·잔고 복원 뒤 profile/client 고정 실행 epoch를 게시. stop은 실행을 무효화하고 전략 수정은 IPC와 같은 `trading/configuration.rs` 서비스를 사용. status는 실제 identity와 camelCase `dailyHistory` 제공 |
| `storage/trade_store.rs` | `data/trades/YYYY/MM/DD/trades.json` (`provider_*` 원본 요청 trace 포함) |
| `storage/order_store.rs` | `data/orders/YYYY/MM/DD/orders.json` (`provider_*` 원본 주문 trace 포함) |
| `storage/stats_store.rs` | `data/stats/YYYY/MM/daily_stats.json` |
| `storage/strategy_store.rs` | `data/strategies/{profile_id}/strategies.json` (`StrategyConfig`에 broker/account scope 저장) |
| `storage/database.rs` | PostgreSQL/MariaDB 문서 호환 계층, schema v2 정규화 projection, transaction import/retention, JSON export, fail-closed backend 전환 |
| `notifications/discord.rs` | Discord Bot 알림 |
| `config/mod.rs` | `secure_config.json` + `.env` 로드 |

---

### 문서 생성과 연구 회귀

| 경로 | 책임 |
|---|---|
| `scripts/render-todo.mjs` | `docs/todo.md`의 상태가 명시된 열린 최상위 항목을 `docs/todo.html`로 생성하고 원본 hash·상태별 색상·검색·필터를 제공한다. `npm run check:todo`는 원본과 생성물의 동기화를 검사 |
| `scripts/render-todo.test.mjs` | 열린 항목 보존·상태 분류·escaping·freshness와 실제 Chromium의 좁은 화면·키보드 필터·JavaScript 없는 읽기를 검증 |
| `scripts/verify-toss-openapi.mjs` | 공식 Toss OpenAPI의 예상 경로 목록을 검사하며 sector 목록·순위·상세·주식·ETF 경로도 포함. 이 목록의 확인을 앱 adapter 구현 또는 장기 분봉 자료 확보와 혼동하지 않음 |
| `tests/e2e/live-daily-history.spec.ts` | 기존 상태 polling의 pending→ready, 부족/실패/legacy/mismatch와 계좌 범위, 512항목 요약·3행 페이지·모바일 표시를 mock 상태로 검증. 주문·자동매매 시작/정지 요청 없이 UI 검사 |
| `tests/e2e/candle-pagination.spec.ts` | 페이지 중복·종료·무진전·상한·계좌 scope 변경·오류, 평가 200봉+이전 252봉 분리와 Web query cursor encoding을 mock provider/page 응답으로 검증 |
| `tests/e2e/strategy-atr-readiness.spec.ts` | ATR 토글 기본값·연구 요청 가정·자료 부족/평가 중 준비·과거 응답 미확인·비활성 상태와 LTH 평가 불가 우선 표시를 mock replay로 검증 |
| `docs/research/autonomous-corrections-20261018/compare.py` | 기존 동결 입력의 수정 전후 생산 replay trace를 비교하고 trace hash·변화 행·기술 통계를 `comparison.json`에 보관. 원래 연구 JSON/CSV는 변경하지 않으며 전체 trace는 `.cache/`에 유지 |

`docs/todo.md`를 수정한 뒤 `npm run todo:render`로 HTML을 갱신한다.
생성 HTML은 원본을 대신 편집하는 위치가 아니며, 앱 화면·자동매매 실행과
독립적인 문서다. 상세 검증 명령은 `docs/HARNESS_MAP.md`를 따른다.

## 4. 백그라운드 데몬 목록 (lib.rs spawn 순서)

| 번호 | 역할 | 제어 방식 |
|------|------|----------|
| 1 | KRX 종목 목록 로드 | 1회성 |
| 2 | 자동매매 폴링 (`commands/trading/daemon.rs::run_trading_daemon`) | `is_trading: Arc<Mutex<bool>>`; `commands/trading.rs` re-export를 통해 spawn |
| 3 | axum 웹 서버 | 영구 실행 |
| 4 | 환율 갱신 (USD/KRW) | `watch::Receiver` — Toss 우선/공개 환율/fallback 캐시 정책 + interval 변경 즉시 반영 |
| 5 | 로그/체결기록 일일 정리 | 24h 주기 |
| 6 | 잔고 갱신 + 이벤트 발행 | `watch::Receiver` — interval 변경 즉시 반영, 활성 broker가 KIS일 때만 KIS 잔고 조회 |

---

## 5. 데이터 흐름

### 공통 캔들 페이지와 일반 preview 자료 수집

```text
strategyPreviewPanel.tsx → candlePageCollector.ts (최대 8요청 / 유지 500봉)
    ↓ getCandlePage / useCandlePage 계약
Tauri get_candle_page / Web GET /api/candles/page
    ↓ commands/candles.rs: 활성 profile/account + scope/asOf 커서 검증
BrokerAdapter::get_candle_page
    ├ KIS: rest/chart.rs 날짜 anchor
    └ Toss: inclusive nextBefore 그대로 다음 요청에 전달
    ↓ OHLCV·통화 검증 / 미완료 제외 / 정렬·중복 제거
CandlePageView: 고정 scope/asOf + nextCursor + 완료 정책·제외 수
    ↓ UI scope 재검증 / 커서 반복·무진전·상한·일부 오류에서 종료
최근 요청 평가봉 + 그 이전 최대 252봉 → preview_strategy
```

일·주·월봉은 서울/뉴욕 현재 시장 기간을 보수적으로 제외하고, 분봉은
제공된 종료 시각이 고정 asOf 이하인 관측만 사용한다. 완료 정책은 provider의
거래소 달력·조기폐장 검증이나 장기 보유 기간 보장을 뜻하지 않는다. 페이지
부족을 합성 봉으로 채우지 않고 실제 확보 범위와 종료 사유를 표시한다.
기존 Vec 차트 API와 전용 LTH 자료 조회는 유지되며 이 수집기는 live 완료
일봉 갱신을 제공하지 않는다.

### Live 52주 완료 일봉 갱신

```text
IPC/Web start 준비 완료 → manager generation + 실행 identity → 실행 플래그
    ↓ daemon: 시장 대기 전 DailyHistoryJobs::maintain
prepare_daily_history_targets: config/profile/scope/종목/시장 날짜 ticket
    ↓ 같은 종목 설정 공유 / 동시 2작업 / 작업당 8회 요청·512봉 유지
공통 완료 일봉 페이지 → 종료 사유·실제 완료 OHLC
    ↓ 완료 handle 수거 / 현재 ticket 재검증
manager/live_history.rs → 52주 initialize_ohlc (마지막 완료봉 포함)
    ↓ dailyHistory 상태 (대상 511 + 초과 진단 1)
TradingStatus IPC/Web → Dashboard/Strategy LiveDailyHistoryPanel (3행씩)
```

새 시장 날짜·설정 변경·정지/재시작은 이전 준비와 비동기 결과를 무효화한다.
quote 중 자정을 넘으면 tick 직전 manager lock에서 다시 준비를 비운다.
부족·실패·상한 초과 대상은 새 매수를 보류하고 보유 손절은 유지하며,
실패/부족은 30·60·120·240·480·900초 간격으로 재시도한다. 최신 완료봉은
현재 시장 날짜보다 1~10일 이전이어야 한다. 긴 휴장도 보류하는 보수적 기준이며
거래소 휴장 달력 확인을 대신하지 않는다.

동일 ID/broker/account의 설정 변경은 52주를 포함한 13개 전략의 보유 보호를
유지하며, 저장 실패는 기존 인스턴스와 revision을 그대로 둔다. 완료 일봉 준비는
성공한 설정 게시의 generation 변경에 따라 다시 수집한다. quote 이후 주문 예약
경합은 아래 실행 context와 설정 편집 예약 경계에서 보호한다.
LTH initializer나 sampled tick은 완료 일봉 자료로 사용하지 않는다.

### 자동 주문 실행과 계좌 게시 경계

```text
IPC/Web start 준비 → manager generation/revision + OrderManager 실행 epoch
    ↓ 검증한 profile·scope·client snapshot 고정 → running=true
daemon/fence::capture_cycle (running → manager → OrderManager)
    ↓ 고정 provider 조회 (잠금 밖) → 현재 context 재검증
전략 tick → 종목명/실제 포지션 조회 → context 포함 submit_signal_shared
    ↓ 준비 후 OrderManager 잠금에서 epoch/revision/scope/profile 재검증
guard/submitting 예약 → 예산 예약 저장 → 고정 provider 전송·재시도
    ↓ 정지/재시작 뒤에도 원래 scope의 접수·pending·예산 추적 유지
주문번호 체결 조회 (고정 snapshot, 잠금 밖)
    ↓ 현재 scope/profile/인증 + 저장 주문 TR-ID의 투자 모드 재검증
실제 체결·종료 반영 (출처 불명·모드 불일치는 보류 + health 진단)
```

성공한 설정 저장은 manager → OrderManager 잠금에서 새 revision을 게시한다.
저장 실패에는 revision을 바꾸거나 기존 인스턴스를 재빌드하지 않는다.
정지·같은 scope 재시작도 epoch를 바꾸므로 이전 quote·신호·전략 상태
복원은 새 실행에 적용하지 않는다. 예약 전 폐기는 guard·예산을 소비하지 않으며,
이미 예약한 요청은 정지로 취소되었다고 간주하지 않는다.

프로파일 적용은 먼저 실행을 무효화하고 자료 로드 후 같은 manager → OrderManager
경계에서 scope/client 교체·tracker 초기화·전략 설정을 게시한다. 수동 holdings는
최초 요청의 profile/client/scope를 재검증한 뒤 tracker를 교체한다. 체결 조회는
전략 epoch보다 실제 계좌·인증의 연속성을 확인하므로 같은 계좌의 실제 체결을
재시작만으로 버리지 않으며, 다른 계좌의 지연 응답은 적용을 보류한다.
KIS pending은 저장 TR-ID로 확인한 실전/모의 출처가 현재 client와 같아야 한다.
출처가 없거나 알려지지 않았거나 모드가 다르면 조회 후보에서 제외하고 직접
체결과 체결 조회의 종료 상태 반영도 보류하여 포지션·예산을 보존하고 체결 동기화
health에 남긴다.

### 전략 설정 저장과 보유 상태 보존

```text
IPC/Web 전략 수정 → trading/configuration.rs의 작업 소유 task
    ↓ strategy_update_lock / 생산 constructor로 유효 대상 정규화
manager → OrderManager: 미완료 신호·보유 대상·예약 주문·계좌 identity 검증
    ↓ Weak 편집 예약: scope의 전략 tick / 대상 주문 예약 / 수동 holdings 교체 보류
잠금 밖 StrategyStore 저장 (기존 전략 인스턴스·revision 유지)
    ↓ manager → OrderManager: profile/client/scope·실제 보유 재검증
같은 ID/scope 전략 설정 갱신 → 기존 보유 보호·유효 관측 보존 → 새 revision 게시
    ↓ 작업 종료·Weak 예약 해제 → daemon 짧은 대기 후 재개
```

저장 전 또는 저장 자체의 실패는 기존 runtime을 유지한다. 첫 저장 오류도 교체 후
오류일 수 있으므로 저장 후 검증 실패와 함께 원래 저장값을 복구한다. 복구까지
실패하면 영속화 장애로 주문을 차단한다.
IPC/HTTP 호출자의 대기 취소가 저장과 게시 사이를 끊지 않도록 독립 task가 끝까지
처리한다. 내부 보유 보호 또는 실제 tracker 보유가 있는 대상 제거·국내외 시장
변경은 저장 전에 거부하고, 다른 전략·신규 대상에 공용 tracker 보유를 복사하지
않는다. 프로파일 게시는 동일 계좌 문자열이어도 새 전략 인스턴스를 구성한다.

### 일반 전략 일봉 replay

```text
strategyPreviewPanel.tsx → preview_strategy → strategy_preview/generic.rs
    ↓ historyCandles 초기화 / 평가 candles 별도 검증
Strategy::on_trading_day_start(symbol, open) — 시가만 공개
    ↓
Strategy::on_daily_close_tick(symbol, completed OHLC, volume) — 종가 평가
    ↓ raw 신호가 있으면 run_backtest_with_history → sync_position (실제 보유 snapshot)
Strategy::on_completed_candle — 다음 평가일의 과거 상태 준비
    ↓
preparation / atrPreparation / raw 신호 / backtest / 재현 메타데이터 → 연구 UI
```

검증된 동일 주기 사전자료는 전략 초기화와 별도로 위험 지표 ATR 준비에
사용한다. 사전자료는 주문·평가 수익률에 포함하지 않으며 ATR 준비 진단은
전략 지표 준비와 구분한다. 선택적 ATR 수량 산정이 꺼져 있으면 준비 부족으로
신규 매수를 추가 차단하지 않는다.

`D`/`1d` cadence는 `dailyCloseWithDayBoundary`다. 강한 종가와 변동성 확장은
이 간격만 지원하며 날짜 전환에 보유 포지션을 reset하지 않는다. 완료일 조건을
다음 평가일에 사용하는 강한 종가도 신호 봉 종가에 체결하는 근사다.
live 현재가 polling과 전용 LTH replay는 이 generic 완료봉 흐름을 사용하지 않는다.
Generic LTH는 기본/접미 ID 모두 factory 실행 전에 명시 거부한다. 입력 시각 기반
전용 LTH 커맨드의 provider/profile 계약을 안내하며 제공봉 입력을 네트워크
조회로 자동 전환하지 않는다. SOXQ 연구 `runner.rs`는 지원 24실행과 LTH의
예상 거부 2건(`rejectedRuns`)을 분리하고 다른 오류에서는 전체 실행을 실패시킨다.

### LTH 분봉 표본과 일봉 진단

```text
leveragedTrendHoldEditorPanel.tsx → preview_leveraged_trend_hold
    ↓ Toss profile/account 검증 및 차트 자료 조회
1d → 가격 차트 진단 (합성 장중 tick·전략 실행 없음)
1m → 이전 완료 일봉 맥락 / 실제 분봉 관측 분리
    ↓ leveraged_trend_hold/preview.rs → 잔여 수량·평균 체결가 snapshot 피드백
replay.assessment → strategyResearchPanel.tsx
    ↓ dailyDiagnostic 또는 notEvaluable / 과거 assessment 누락
성과 표시·A/B 저장 및 비교 차단
```

일봉 맥락은 `dailyContextBars`, 장중 관측은 `intradayBars`로 구분하고
LTH `warmupCount`는 0이다. 유효한 분봉 표본은 `sampleOnly`이며 기간 전체
성과 검증을 뜻하지 않는다. 현재 시각·세션은 `unverified`, `lookAheadSafe`는
false다. SOXQ 2026-07-06~10-02의 실제 분봉과 거래소 시간대/DST·세션 검증은
P1-05의 미완료 항목으로 남아 있다.

### 오프라인 전략 전수 연구

```text
공개 시세 원본 (.cache/research/strategy-review-20261018/)
    ↓ prepare_data.py: 정규화·hash·준비/평가 구간 선언
docs/research/strategy-review-20261018/input.json
    ↓ runner.rs → 생산 preview_strategy_from_candles / run_backtest
전체 실행 trace (.cache/) / 전용 LTH 실제 분봉 표본 진단
    ↓ analyze.py
results.jsonl (실행별 compact 결과) / summary.json
    ↓ stock_analyst의 입력·결과·해석 감사
report.md / validation.json → docs/todo.md의 근거·우선순위·완료 조건
```

연구 실행기는 별도 전략을 재구현하지 않고 생산 Rust 라이브러리를 사용한다.
일봉 지원 전략, generic LTH 거부, 실제 분봉 LTH 진단과 benchmark를 구분한다.
분기별 포지션 초기화 결과와 겹치는 최근 기간은 따로 집계하며, 분봉 표본을
3개월 시장 검증으로 승격하지 않는다. 이 흐름은 앱 IPC·UI·데몬 경로를 변경하지
않고 주문 실행이나 사용자 계정 설정 변경 없이 연구 파일과 TODO를 남긴다.

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

### 자동매매 전용 예산

```text
Settings → api/budgetHooks.ts → commands/budget.rs 또는 server/budget.rs
    ↓ 활성 broker/account 검증 (변경은 자동매매 정지 상태)
OrderManager → trading/order/budget.rs
    ↓ storage::read_json_or_default / write_json
data/orders/auto_trading_budget.json (JSON 또는 DB document)

자동 주문 → submission.rs → 전용 현금/소유수량 확인과 예약 영속화
    ↓ provider 접수 → pending 저장 및 예약 접수 상태 반영
fills.rs → 누적 체결을 원장 현금·소유수량·잔여 예약에 반영
    ↓ 주문 완료 → 잔여 예약 해제
```

원장은 broker/account와 KRW/USD를 분리한다. 자동매도는 원장에 기록된
자동매매 소유수량으로 제한하고, 수동매도는 해당 수량을 제외한다.
접수 여부가 불명확한 예약은 예산 사용을 차단하며, 원장 저장 실패는
OrderManager의 영속화 오류 차단 경계로 전달한다.

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

공통 `.cargo/config.toml`은 상대 `target-dir = "target"` 설정으로 추적한다.
Cargo 빌드 산출물은 프로젝트 루트 `target/`, 일회성 검증·연구 harness는
`.cache/`에 두며 둘 다 Git에서 제외한다. `scripts/setup-local.sh`는 로컬
도구·의존성을 확인하고 공통 Cargo 설정을 외부 절대 경로로 덮어쓰지 않는다.

---

## 10. 에이전트 역할

| 역할 | 프로젝트 파일 | 책임 |
|---|---|---|
| `project_mapper` | `.codex/agents/project_mapper.toml` | 전체 파일 인벤토리와 구조 설명 동기화 |
| `symbol_navigator` | `.codex/agents/symbol_navigator.toml` | Serena 기반 정의·참조·영향 범위 추적 |
| `helper_curator` | `.codex/agents/helper_curator.toml` | Graphify 후보 탐색과 Serena 동등성 검증 후 공용화 |
| `stock_analyst` | `.codex/agents/stock_analyst.toml` | 생산 엔진 기반 주식 전략 연구의 입력·결과·재무 해석 감사와 근거 있는 개선 TODO 제안 |

Serena 설정과 공유 메모리는 `.serena/`, Graphify 코드 그래프는
`graphify-out/`에 둔다. Graphify의 캐시·로컬 경로·분석 중간 산출물은
Git에서 제외하고, 공유 그래프·보고서·manifest·source-hashes를 유지한다. 세 역할의 위임
조건, 민감 파일 제외, 갱신·검증 명령은 `docs/agent-tooling.md`를 따른다.

`stock_analyst`는 읽기 전용 연구를 기본으로 하며 실행기는 부모가 생산
라이브러리에 연결해 실행한다. 연구 산출물 편집은 부모가 할당한 경로에서만
수행하고 주문 실행·자동매매 시작·민감 계정 파일 접근은 맡지 않는다.
