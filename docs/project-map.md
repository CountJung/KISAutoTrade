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
│   │   ├── consecutive-filters-20261018/
│   │   │   ├── aggregate.json
│   │   │   ├── baseline-hashes.json
│   │   │   ├── baseline_runner.rs
│   │   │   ├── continuous-results.jsonl.gz
│   │   │   ├── design.json
│   │   │   ├── long-baseline-hashes.json
│   │   │   ├── report.md
│   │   │   ├── reset-results.jsonl.gz
│   │   │   ├── run-summary.json
│   │   │   ├── runner.rs
│   │   │   ├── summary.csv
│   │   │   ├── validate.py
│   │   │   └── validation.json
│   │   ├── continuous-replay-20261018/
│   │   │   ├── aggregate.json
│   │   │   ├── baseline-hashes.json
│   │   │   ├── baseline_runner.rs
│   │   │   ├── continuous-results.jsonl.gz
│   │   │   ├── design.json
│   │   │   ├── report.md
│   │   │   ├── reset-results.jsonl.gz
│   │   │   ├── run-summary.json
│   │   │   ├── runner.rs
│   │   │   ├── summary.csv
│   │   │   ├── validate.py
│   │   │   └── validation.json
│   │   ├── deviation-policy-20261020/
│   │   │   ├── aggregate.json
│   │   │   ├── baseline-hashes.json
│   │   │   ├── baseline-run-summary.json
│   │   │   ├── baseline_runner.rs
│   │   │   ├── continuous-results.jsonl.gz
│   │   │   ├── design.json
│   │   │   ├── flat-deviation-deltas.json
│   │   │   ├── report.md
│   │   │   ├── reset-results.jsonl.gz
│   │   │   ├── run-summary.json
│   │   │   ├── runner.rs
│   │   │   ├── summary.csv
│   │   │   ├── validate.py
│   │   │   └── validation.json
│   │   ├── ma-cross-filters-20261020/
│   │   │   ├── aggregate.json
│   │   │   ├── baseline-hashes.json
│   │   │   ├── baseline-run-summary.json
│   │   │   ├── baseline_runner.rs
│   │   │   ├── continuous-results.jsonl.gz
│   │   │   ├── design.json
│   │   │   ├── report.md
│   │   │   ├── reset-results.jsonl.gz
│   │   │   ├── run-summary.json
│   │   │   ├── runner.rs
│   │   │   ├── summary.csv
│   │   │   ├── validate.py
│   │   │   └── validation.json
│   │   ├── mean-reversion-policy-20261020/
│   │   │   ├── aggregate.json
│   │   │   ├── baseline-hashes.json
│   │   │   ├── baseline-run-summary.json
│   │   │   ├── baseline_runner.rs
│   │   │   ├── continuous-results.jsonl.gz
│   │   │   ├── design.json
│   │   │   ├── flat-mean-reversion-deltas.json
│   │   │   ├── report.md
│   │   │   ├── reset-results.jsonl.gz
│   │   │   ├── run-summary.json
│   │   │   ├── runner.rs
│   │   │   ├── summary.csv
│   │   │   ├── validate.py
│   │   │   └── validation.json
│   │   ├── momentum-feedback-20261020/
│   │   │   ├── aggregate.json
│   │   │   ├── baseline-hashes.json
│   │   │   ├── baseline-run-summary.json
│   │   │   ├── baseline_runner.rs
│   │   │   ├── continuous-results.jsonl.gz
│   │   │   ├── design.json
│   │   │   ├── flat-momentum-deltas.json
│   │   │   ├── report.md
│   │   │   ├── reset-results.jsonl.gz
│   │   │   ├── run-summary.json
│   │   │   ├── runner.rs
│   │   │   ├── summary.csv
│   │   │   ├── validate.py
│   │   │   └── validation.json
│   │   ├── replay-accounting-20261018/
│   │   │   ├── baseline-hashes.json
│   │   │   ├── baseline_runner.rs
│   │   │   ├── report.md
│   │   │   ├── results.json
│   │   │   ├── runner.rs
│   │   │   ├── validate.py
│   │   │   └── validation.json
│   │   ├── replay-execution-20261018/
│   │   │   ├── baseline-hashes.json
│   │   │   ├── baseline_runner.rs
│   │   │   ├── report.md
│   │   │   ├── results.json
│   │   │   ├── runner.rs
│   │   │   └── validation.json
│   │   ├── rsi-policy-20261022/
│   │   │   ├── aggregate.json
│   │   │   ├── analyze.py
│   │   │   ├── baseline-hashes.json
│   │   │   ├── baseline-run-summary.json
│   │   │   ├── baseline-source-validation.json
│   │   │   ├── baseline_runner.rs
│   │   │   ├── continuous-results.jsonl.gz
│   │   │   ├── design.json
│   │   │   ├── flat-rsi-deltas.json
│   │   │   ├── independent-audit.json
│   │   │   ├── policy-observations.json
│   │   │   ├── report.md
│   │   │   ├── reset-results.jsonl.gz
│   │   │   ├── run-summary.json
│   │   │   ├── runner.rs
│   │   │   ├── summary.csv
│   │   │   ├── validate.py
│   │   │   └── validation.json
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
│   │   ├── consecutive-entry-filters-20261018.md
│   │   ├── continuous-replay-20261018.md
│   │   ├── database-tls-reconnect-20261018.md
│   │   ├── deviation-policy-20261020.md
│   │   ├── documentation-layout-20261018.md
│   │   ├── execution-run-reservation-20261018.md
│   │   ├── frontend-hooks-split-20261018.md
│   │   ├── holdings-publication-fence-20261018.md
│   │   ├── live-completed-daily-history-20261018.md
│   │   ├── lth-minute-timing-20261022.md
│   │   ├── ma-cross-filters-20261020.md
│   │   ├── mean-reversion-policy-20261020.md
│   │   ├── momentum-feedback-20261020.md
│   │   ├── project-health-and-soxq-p1-01.md
│   │   ├── provider-candle-contract-20261018.md
│   │   ├── replay-accounting-20261018.md
│   │   ├── replay-execution-model-20261018.md
│   │   ├── root-documents-20261020.md
│   │   ├── rsi-policy-20261022.md
│   │   ├── soxq-p1-04.md
│   │   ├── soxq-p1-05.md
│   │   ├── strategy-configuration-preservation-20261018.md
│   │   ├── strategy-fill-ownership-20261018.md
│   │   ├── strategy-review-20261018.md
│   │   └── trading-module-split-20261018.md
│   ├── ARCHITECTURE.md
│   ├── HARNESS_MAP.md
│   ├── LICENSE
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
│   ├── database-tls-contract.mjs
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
│   │   │   │   ├── consecutiveEntryControls.tsx
│   │   │   │   ├── consecutiveEntryPreparation.tsx
│   │   │   │   ├── continuousReplayPanel.tsx
│   │   │   │   ├── deviationControls.tsx
│   │   │   │   ├── leveragedTrendHoldEditorPanel.tsx
│   │   │   │   ├── leveragedTrendHoldPreviewChart.tsx
│   │   │   │   ├── lthMinuteTimingPanel.tsx
│   │   │   │   ├── maCrossEntryControls.tsx
│   │   │   │   ├── maCrossEntryPreparation.tsx
│   │   │   │   ├── meanReversionControls.tsx
│   │   │   │   ├── momentumEntryControls.tsx
│   │   │   │   ├── priceConditionEditorPanel.tsx
│   │   │   │   ├── rsiControls.tsx
│   │   │   │   ├── strategyAccountingPanel.tsx
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
│   │   │   ├── draftMap.ts
│   │   │   ├── formatters.ts
│   │   │   ├── index.ts
│   │   │   ├── numberDraft.ts
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
│   │   │   │   ├── continuous.rs
│   │   │   │   ├── continuous_tests.rs
│   │   │   │   ├── daily_event_tests.rs
│   │   │   │   ├── deviation_entry_tests.rs
│   │   │   │   ├── driver.rs
│   │   │   │   ├── entry_filter_tests.rs
│   │   │   │   ├── generic.rs
│   │   │   │   ├── generic_tests.rs
│   │   │   │   ├── ma_cross_entry_tests.rs
│   │   │   │   ├── mean_reversion_entry_tests.rs
│   │   │   │   ├── minute_timing_tests.rs
│   │   │   │   ├── momentum_entry_tests.rs
│   │   │   │   ├── preparation_tests.rs
│   │   │   │   ├── rsi_entry_tests.rs
│   │   │   │   └── tests.rs
│   │   │   ├── toss/
│   │   │   │   ├── small_order/
│   │   │   │   │   └── tests.rs
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
│   │   │   │   ├── risk_restore/
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
│   │   │   ├── database/
│   │   │   │   ├── configuration.rs
│   │   │   │   ├── connection.rs
│   │   │   │   └── tests.rs
│   │   │   ├── database_contract_tests/
│   │   │   │   └── tls_restart.rs
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
│   │   │   │   ├── tests/
│   │   │   │   │   └── rsi.rs
│   │   │   │   └── tests.rs
│   │   │   ├── order/
│   │   │   │   ├── configuration/
│   │   │   │   │   └── tests.rs
│   │   │   │   ├── feedback/
│   │   │   │   │   ├── tests/
│   │   │   │   │   │   ├── mean_reversion.rs
│   │   │   │   │   │   └── rsi.rs
│   │   │   │   │   └── tests.rs
│   │   │   │   ├── fills/
│   │   │   │   │   ├── collector/
│   │   │   │   │   │   ├── notional/
│   │   │   │   │   │   │   └── tests.rs
│   │   │   │   │   │   └── notional.rs
│   │   │   │   │   ├── ownership/
│   │   │   │   │   │   └── tests.rs
│   │   │   │   │   ├── reconciliation/
│   │   │   │   │   │   └── tests.rs
│   │   │   │   │   ├── collector.rs
│   │   │   │   │   ├── ownership.rs
│   │   │   │   │   ├── provider_mode.rs
│   │   │   │   │   └── reconciliation.rs
│   │   │   │   ├── holdings/
│   │   │   │   │   ├── parse/
│   │   │   │   │   │   └── tests.rs
│   │   │   │   │   ├── tests/
│   │   │   │   │   │   └── http.rs
│   │   │   │   │   ├── fetch.rs
│   │   │   │   │   ├── parse.rs
│   │   │   │   │   └── tests.rs
│   │   │   │   ├── ownership/
│   │   │   │   │   ├── runtime/
│   │   │   │   │   │   └── tests.rs
│   │   │   │   │   ├── runtime.rs
│   │   │   │   │   └── tests.rs
│   │   │   │   ├── submission/
│   │   │   │   │   ├── owned/
│   │   │   │   │   │   └── tests.rs
│   │   │   │   │   ├── owned.rs
│   │   │   │   │   ├── positions.rs
│   │   │   │   │   ├── prepare.rs
│   │   │   │   │   ├── provider.rs
│   │   │   │   │   ├── records.rs
│   │   │   │   │   └── run_tests.rs
│   │   │   │   ├── budget.rs
│   │   │   │   ├── configuration.rs
│   │   │   │   ├── conflicts.rs
│   │   │   │   ├── execution_run.rs
│   │   │   │   ├── feedback.rs
│   │   │   │   ├── fills.rs
│   │   │   │   ├── health.rs
│   │   │   │   ├── holdings.rs
│   │   │   │   ├── legacy.rs
│   │   │   │   ├── ownership.rs
│   │   │   │   ├── pending.rs
│   │   │   │   ├── submission.rs
│   │   │   │   └── test_support.rs
│   │   │   ├── risk/
│   │   │   │   ├── fill_accounting/
│   │   │   │   │   └── tests.rs
│   │   │   │   └── fill_accounting.rs
│   │   │   ├── simulation/
│   │   │   │   ├── session/
│   │   │   │   │   └── accounting.rs
│   │   │   │   ├── accounting.rs
│   │   │   │   ├── accounting_tests.rs
│   │   │   │   ├── assessment.rs
│   │   │   │   ├── atr.rs
│   │   │   │   ├── comparison.rs
│   │   │   │   ├── execution_tests.rs
│   │   │   │   ├── minute_time.rs
│   │   │   │   ├── minute_timing.rs
│   │   │   │   ├── minute_timing_tests.rs
│   │   │   │   ├── session.rs
│   │   │   │   └── tests.rs
│   │   │   ├── strategy/
│   │   │   │   ├── breakout/
│   │   │   │   │   ├── day_event_tests.rs
│   │   │   │   │   ├── high_tests.rs
│   │   │   │   │   ├── live_feedback_tests.rs
│   │   │   │   │   └── reconfigure_tests.rs
│   │   │   │   ├── classic/
│   │   │   │   │   ├── deviation/
│   │   │   │   │   │   ├── params.rs
│   │   │   │   │   │   ├── state.rs
│   │   │   │   │   │   └── tests.rs
│   │   │   │   │   ├── momentum/
│   │   │   │   │   │   └── tests.rs
│   │   │   │   │   ├── rsi/
│   │   │   │   │   │   ├── params.rs
│   │   │   │   │   │   └── tests.rs
│   │   │   │   │   ├── deviation.rs
│   │   │   │   │   ├── live_feedback_tests.rs
│   │   │   │   │   ├── momentum.rs
│   │   │   │   │   ├── reconfigure_tests.rs
│   │   │   │   │   └── rsi.rs
│   │   │   │   ├── leveraged_trend_hold/
│   │   │   │   │   ├── bollinger.rs
│   │   │   │   │   ├── buffer_tests.rs
│   │   │   │   │   ├── exits.rs
│   │   │   │   │   ├── indicators.rs
│   │   │   │   │   ├── live.rs
│   │   │   │   │   ├── live_feedback_tests.rs
│   │   │   │   │   ├── preview.rs
│   │   │   │   │   ├── reconfigure_tests.rs
│   │   │   │   │   ├── session.rs
│   │   │   │   │   └── tests.rs
│   │   │   │   ├── ma_cross/
│   │   │   │   │   ├── context/
│   │   │   │   │   │   └── tests.rs
│   │   │   │   │   ├── context.rs
│   │   │   │   │   ├── entry_tests.rs
│   │   │   │   │   ├── params.rs
│   │   │   │   │   └── reconfigure_tests.rs
│   │   │   │   ├── manager/
│   │   │   │   │   ├── configuration.rs
│   │   │   │   │   ├── live_feedback_tests.rs
│   │   │   │   │   ├── live_history.rs
│   │   │   │   │   └── live_history_tests.rs
│   │   │   │   ├── mean_trend/
│   │   │   │   │   ├── mean_reversion/
│   │   │   │   │   │   ├── params.rs
│   │   │   │   │   │   ├── state.rs
│   │   │   │   │   │   └── tests.rs
│   │   │   │   │   ├── live_feedback_tests.rs
│   │   │   │   │   ├── mean_reversion.rs
│   │   │   │   │   └── reconfigure_tests.rs
│   │   │   │   ├── price_condition/
│   │   │   │   │   ├── live_feedback_tests.rs
│   │   │   │   │   └── reconfigure_tests.rs
│   │   │   │   ├── sequence/
│   │   │   │   │   ├── consecutive/
│   │   │   │   │   │   ├── context.rs
│   │   │   │   │   │   └── tests.rs
│   │   │   │   │   ├── consecutive.rs
│   │   │   │   │   ├── live_feedback_tests.rs
│   │   │   │   │   └── reconfigure_tests.rs
│   │   │   │   ├── breakout.rs
│   │   │   │   ├── classic.rs
│   │   │   │   ├── completed_daily.rs
│   │   │   │   ├── core.rs
│   │   │   │   ├── leveraged_trend_hold.rs
│   │   │   │   ├── ma_cross.rs
│   │   │   │   ├── manager.rs
│   │   │   │   ├── mean_trend.rs
│   │   │   │   ├── observation.rs
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
│       ├── balance-events.spec.ts
│       ├── candle-pagination.spec.ts
│       ├── live-daily-history.spec.ts
│       ├── lth-minute-timing.spec.ts
│       ├── settings-database.spec.ts
│       ├── strategy-accounting.spec.ts
│       ├── strategy-atr-readiness.spec.ts
│       ├── strategy-configuration-errors.spec.ts
│       ├── strategy-consecutive-entry-filters.spec.ts
│       ├── strategy-continuous-replay.spec.ts
│       ├── strategy-deviation-policy.spec.ts
│       ├── strategy-execution-model.spec.ts
│       ├── strategy-ma-cross-filters.spec.ts
│       ├── strategy-mean-reversion-policy.spec.ts
│       ├── strategy-momentum-entry.spec.ts
│       ├── strategy-replay-assessment.spec.ts
│       ├── strategy-rsi-policy.spec.ts
│       ├── strategy-scrollbar.spec.ts
│       └── toss-commission-ratio.spec.ts
├── .env.example
├── .gitignore
├── .graphifyignore
├── .nvmrc
├── AGENTS.md
├── CLAUDE.md
├── Cargo.lock
├── Cargo.toml
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

루트에는 `README.md`, `AGENTS.md`, `CLAUDE.md`를 문서 진입점으로 유지한다. 라이선스 정본은 [`docs/LICENSE`](LICENSE)에 두고 README 배지/본문에서 연결한다.
일반 운영·설계·할일 문서는 `docs/`, 작업 기록은 `docs/tasks/`에 모은다.
도구가 요구하는 `.github/`, `.codex/`, `.claude/`, `.serena/` 설정·스킬과
빌드 설정·앱 진입 파일·Graphify 생성 산출물은 각 기존 위치를 유지한다.

| 문서 | 책임 |
|---|---|
| `README.md` | 프로젝트 소개·실행과 운영 문서로의 루트 진입점 |
| `docs/LICENSE` | 원문을 유지한 MIT 라이선스 정본. README 배지·본문에서 연결 |
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
| `database_config.json` | 앱 데이터 폴더. DB password는 OS keychain에 저장하며 keychain 사용 불가 시 권한 제한 파일에 유지. IPC view에는 미반환 |
| `database_config.json.recovery` | 앱 데이터 폴더. 설정 파일·keychain 저장/게시 미완료를 나타내는 비밀 없는 내구성 표시 |
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
| `shared/lib/` | 공통 유틸 (`persistentLayout` localStorage 숫자 상태 저장/복원, `formatters` 숫자/decimal/money 표시, `numberDraft` decimal 초안 파싱·엄격 유한수 범위 판정, `draftMap` 제출 초안 identity에 따른 정리) |
| `shared/config/theme/` | 앱 테마 생성과 theme mode 타입 |
| `shared/config/scheduler/` | TanStack Query 공통 폴링 주기 |
| `entities/*/model/` | Zustand 전역 상태 (계좌, 매매, 설정) |
| `api/hooks.ts` | 기존 import를 유지하는 TanStack Query 훅 facade. 도메인별 hook 모듈과 기존 budget/database hook, `KEYS`, `useBackendEvents`를 re-export하며 훅 구현을 직접 소유하지 않음 |
| `api/accountHooks.ts` | 앱 계정 설정·설정 확인·국내/해외/broker 잔고·프로파일 CRUD/전환·거래 환경 감지·Toss 연결/계좌 조회 hook. 잔고 query key를 profile/broker/account/투자 모드로 분리하고 응답 전후 현재 scope를 검사. 계좌 전환 시 이전 query를 취소·무효화하며 이전 계좌의 placeholder/initial data를 공유하지 않음 |
| `shared/api/liveDailyHistoryTypes.ts` | `TradingStatus.dailyHistory`의 읽기 전용 준비 진단 타입. 기존 응답의 필드 누락은 준비 완료로 추정하지 않도록 optional 호환 계약 유지 |
| `shared/ui/LiveDailyHistoryPanel.tsx` | Dashboard/Strategy가 기존 상태 query로 공유하는 완료 일봉 준비 패널. 실행/활성 profile·broker/account 범위를 확인하고 상태·확보량·시장 날짜·출처·실패/재시도 사유 표시. 최대 512상태를 요약하고 실제 행은 3개씩 표시하며 프로파일 없는 legacy KIS 실행은 일치 계좌의 실패 진단만 허용 |
| `api/marketHooks.ts` | 국내/해외/Toss 시세·차트·검색·종목 목록 갱신·시장 일정/유의사항·환율 hook과 해외 차트 preset. 갱신 설정은 `settingsHooks.ts::useRefreshConfig`를 직접 참조. `useCandlePage`는 프로파일·broker/account·종목·간격·거래소·커서별 공통 페이지 조회 계약을 사용 |
| `api/orderHooks.ts` | 국내/해외 수동 주문·Toss 주문 전 검증/정정·소액 검증·미체결 조회 hook과 주문 후 관련 query 무효화 |
| `api/strategyHooks.ts` | scope별 전략 목록·수정, 기존 단기 replay·연속 replay·LTH preview mutation hook |
| `api/settingsHooks.ts` | 업데이트 확인·웹 설정·갱신 주기/상세 설정·broker rate-limit 상태·Discord 테스트 hook |
| `api/historyHooks.ts` | 당일/기간별 provider 체결·로컬 거래·통계와 거래 보관 설정/현황 hook |
| `api/tradingHooks.ts` | 자동매매 상태·시작/정지·매수 정지 해제·포지션·리스크 설정·비상정지 hook |
| `api/logHooks.ts` | 로그 설정·최근 로그 조회·프런트 로그 기록 hook과 `frontendLogger` |
| `api/queryKeys.ts` | TanStack Query `KEYS` 단일 원천 (`hooks.ts`에서 하위 호환 re-export) |
| `api/backendEvents.ts` | Tauri backend event 구독. 환율은 전역 캐시를 갱신하고 잔고 이벤트는 payload를 대입하지 않고 현재 계좌 query를 무효화하여 재조회. listener 등록/해제 지연과 실패 및 unmount 뒤 callback을 처리 |
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
| `pages/strategy/ui/Page.tsx` | 활성 broker scope, 전략별 저장 broker/account scope 표시, 전략 활성화/대상 종목/전체 너비 카드/카드별 미리보기 route 조립. 일반·가격조건·LTH 저장 성공은 제출한 immutable 초안과 현재 초안의 identity가 같을 때만 정리하고 scope epoch가 달라진 응답은 현재 편집 상태에 적용하지 않음 |
| `pages/strategy/ui/priceConditionEditorPanel.tsx` | 가격조건 전략의 종목별 수량·매수가·익절가·익절률·손절률 편집 테이블 |
| `pages/strategy/ui/strategyMetadata.ts` | 일반 전략 파라미터 입력 메타, 전략 설명, strategy id 기반 타입 판별. Consecutive의 관측 횟수와 선택 진입3필드, Momentum 기본 level/선택 crossing·재무장 %p와 Deviation 진입 모드·장기/보유 관측 수·native 손절률, MeanReversion 청산 모드·장기/보유 관측 수와 기존 손절0, MA 교차의 기존 관측 기간과 완료 일봉 필터4개, RSI의 기존14/30/70 단순평균·진입 추세/손절/보유 관측3개의 단위·상한 설명을 소유 |
| `pages/strategy/ui/deviationControls.tsx` | 기본 level/선택 하향 crossing·직전 N관측 추세·보유 관측 한도·native 손절률 편집과 현재/A-B 요약. stored 타입·raw 소수 초안을 보존하며 엄격 유효성으로 저장/두 preview를 차단. live 약10초와 선택 replay 봉, live 추세 별도 준비·나이 불명 복원·보호 잔량 전량·nextOpen 갭을 안내 |
| `pages/strategy/ui/rsiControls.tsx` | RSI 선택 추세 관측·native 손절률·최대 보유 관측의 raw/stored 초안·엄격 유효성 및 현재/A-B 요약. 기존 단순평균14/30/70·횡보 zero-loss100, 실제 소유와 Sell잔량, 원래 교차 소비/선택 보호 재시도·live 최초 추세 seed 분리·ageunknown·관측 단위/손실 상한 한계를 안내. shared decimal/유한수 helper를 사용하며 과거 엔진 요약을 보호 적용/체결 증거로 표시하지 않음 |
| `pages/strategy/ui/meanReversionControls.tsx` | 기본 upperBand/선택 middleBand, 직전 N관측 추세·최대 보유 관측의 편집·엄격 검증과 현재/A-B 요약. stored enum/null/string·숫자 raw 초안을 보존해 저장/두 preview를 차단하고 shared decimal/유한수 helper를 재사용. strict 현재 포함 밴드, 기존 손절0, 실제 소유·부분 체결, live/일봉/1분 warmup 차이·관측 단위·청산 수량과 갭 한계를 안내 |
| `pages/strategy/ui/momentumEntryControls.tsx` | 기본 level/선택 crossing과 재무장 %p 편집·현재/A-B 요약. stored enum/null/string을 원형 보존해 엄격 검증하고 crossing의 간격≤임계값을 확인. level에서는 유효 간격 초안을 보존하되 미사용을 표시하며 소수 raw state·필드 범위·crossing 관계는 local, decimal 파싱과 비강제 유한수/범위 판정은 shared helper를 사용 |
| `pages/strategy/ui/maCrossEntryControls.tsx` | 기존 관측5/20과 별도인 완료 일봉 추세·장기 MA 기울기·ATR 비율·밴드폭4개 진입 조건 편집/엄격 검증·현재/A-B 요약. raw/stored 초안과 shared decimal/유한수 helper를 사용하며 기울기 활성 long_period1..511 관계를 확인. 현재봉 제외·ATR/밴드폭0 준비·거절한 교차 비재생·일봉 전용·후보/체결 차이를 안내 |
| `pages/strategy/ui/maCrossEntryPreparation.tsx` | 기본 누락/0은 중립 비활성, 활성 필터의 legacy 진단 누락은 준비 미확인 경고로 표시. 완료 일봉/TR 수·골든크로스 후보/미준비·추세/기울기/ATR/폭 거부·통과를 수량 산정 ATR과 구분하고 선택 slope/bandwidth 누락을 정보 없음으로 유지. 현재 결과와 A/B에서 재사용 |
| `pages/strategy/ui/consecutiveEntryControls.tsx` | Consecutive 진입 필터 편집·가시 안내와 params 요약. 잘못된 stored 타입·빈칸·유한수/범위/정수 위반을 보존해 저장/preview를 차단하고, 로컬 raw state로 소수점 키보드 입력을 유지하며 유효값만 숫자로 전달 |
| `pages/strategy/ui/consecutiveEntryPreparation.tsx` | 선택 진입 준비·완료 OHLC/TR 수·후보/거부/통과 진단을 수량 산정 ATR과 별도로 표시. current/saved params가 모두0이면 중립 비활성, 활성 필터의 진단 누락은 미확인 경고로 구분하며 통과를 체결로 세지 않음 |
| `pages/strategy/ui/leveragedTrendHoldEditorPanel.tsx` | 레버리지 추세 보유 전략의 ETF 검색/편집, 진입·반등·청산 파라미터, Toss 1분/일봉·50/100/200봉 preview와 계좌 scope/stale 응답 차단 |
| `pages/strategy/ui/strategyPreviewPanel.tsx` | 일반/가격조건 전략 카드에서 `candlePageCollector`로 KIS 일/주/월봉 또는 Toss 1분/일봉을 순차 조회하고 요청 평가봉과 그 이전 사전자료를 나눠 편집값·비용 가정과 함께 `preview_strategy`에 전달. 실제 페이지 수·종료 사유·미완료/무효/중복 제외 수와 확보한 사전자료를 표시한다. 강한 종가·변동성 확장은 일봉 선택만 제공하고 날짜 이벤트와 선택한 연구 체결 모델의 가정을 안내한다. 실제 평가 기간·사전자료 수·준비 상태·평가 불가/조건 불충족/무체결을 표시하고 티커/봉/구간/파라미터/수량/broker/account/가정 변경 시 결과 무효화. 연속 일봉은 별도 수집/preview_strategy_continuous를 사용하고 응답 청크·평가/사전자료 수를 검증하며 가정·날짜·계좌 변경 또는 취소 뒤 오래된 결과를 게시하지 않는다. Consecutive ATR/MA 또는 MA 교차4필터 활성 비일봉과 잘못된 진입 필터 draft는 계산을 차단하며 entryFilterPreparation을 공통 결과에 전달 |
| `pages/strategy/ui/continuousReplayPanel.tsx` | 단기/연속 날짜 범위 선택·기본 최근 3년 요청과 자료 수집 진행/보류·실제 범위·누락 한계 표시. 연속 API가 반환한 청크별 시작/종료 수량·대기 신호를 공급 경계로 표시하며 분기 리셋·독립 실험으로 해석하지 않음 |
| `pages/strategy/ui/strategyResearchPanel.tsx` | 초기자본·수수료·세금·슬리피지·환율·리스크·학습구간 입력, 수익률/MDD/승률/손익비/turnover/exposure, 원시 신호와 주문 가능/체결 구분, equity curve, 거래 목록, in/out-of-sample, A/B 비교 UI. LTH assessment에 따라 일봉·관측 부족·과거 평가 정보 없는 결과의 성과 표시와 저장/비교를 차단하고, 분봉은 기간 전체 성과와 구분한 표본으로 안내. 연구 전용 ATR 수량 산정 토글·위험 예산·손절 배수 입력과 전략 지표 준비와 별도인 ATR 준비/미준비·과거 응답의 미확인 상태를 표시. sameClose/nextOpen, 봉 내부 미검증/손절 우선, 연구 손절·익절 가정과 신호 공개/체결 단계·미체결·동시 도달 진단을 표시한다. 이전 응답의 누락 정보는 미확인으로 유지하고 A/B에 엔진·가정 차이를 안내하며 운용 설정을 변경하지 않음. Consecutive 선택 진입 진단과 params를 A/B에 보존하고 진입 조건 차이·활성 legacy 진단 누락을 표시. Momentum은 두 진입 params와 모드/간격 요약·조건 차이를 현재 결과/A-B에 보존하고 표시. Deviation은 진입·추세·보유 관측·손절 params와 요약·조건 차이를 현재 결과/A-B에 보존하고 표시. MeanReversion은 청산 모드·추세·보유 관측·기존 손절 params 요약과 A/B 조건 차이를 표시. MA 교차는 params·선택 진입 준비/거부 진단·조건 차이를 현재 결과/A-B에 보존하고 표시. RSI는 기존 core·선택3params를 현재/A-B에 요약하고 조건 차이를 경고한다. 전용 LTH1분의 선택 minuteTiming을 현재/A-B에 표시하고 버전·봉 표기·시간대·검증 가정 차이를 경고하며 기존 assessment 성과 gate는 유지 |
| `pages/strategy/ui/lthMinuteTimingPanel.tsx` | LTH1분 라벨의 공급/선택/실행/가격 제외·bounded 진단 수, UTC/현지 시각·DST offset·정규 시간창/주말/미완료·같은 날짜 내부 미관측 슬롯을 표시. 날짜8개·예시6개·A/B compact2일로 제한하고 legacy 누락/미지원 버전을 미확인으로 유지. 진단 비교 key는 별도 버전·봉 표기·시간대·legacy 시계/검증 가정을 사용하며 성과·달력/coverage 인증으로 승격하지 않음 |
| `pages/strategy/ui/strategyAccountingPanel.tsx` | 일반·LTH 공통 연구 비용/현금흐름 입력과 결과 표시. 방향별·최소 수수료와 수동 출처, bounded 환율·배당 JSON·배당 가격 조정 선언을 검증하고 오류 초안은 실행을 차단하며 이전 성과를 무효화. 실현/잔여·가격/환율·배당·종료 가상 청산·관측 봉말 투자 비중·동일 요청 수량 보유/기록 비용 환급 비교와 한계를 표시. 과거 응답의 누락 회계를 0으로 추정하지 않음 |
| `shared/api/strategyResearchTypes.ts` | 일반/LTH replay의 가정·회계/비교 DTO와 typed 연속 chunk/input/continuity 계약. A/B의 calculationMode/continuity와 과거 저장 결과의 새 회계 필드는 optional로 유지하며 누락을 미확인으로 표시. 단기/연속 공통 EntryFilterPreparation과 A/B 보관 필드는 선택적이며 비활성 기본값과 활성 필터의 진단 누락을 구분. MA 전용 slopeRejections/bandwidthRejections는 optional이며 Consecutive 응답의 생략과 과거 누락을 보존. ReplayAssessment.minuteTiming은 선택 LthMinuteTiming DTO로 시각/세션/관측 슬롯·출처 수·상한·미검증 플래그를 미러하며 과거 응답 누락은 정보 없음 |
| `shared/lib/numberDraft.ts` | `parseDecimalDraft(raw)`는 명시 decimal 문법과 finite 숫자 변환을 소유하고 빈칸·문법 불일치·비유한 입력은 원문 문자열로 반환. `isFiniteNumberInRange`는 타입을 변환하지 않고 유한수·상하한·선택 정수 조건만 판정. public export를 통해 Consecutive/Momentum/Deviation/MeanReversion/MA 교차/RSI가 재사용하며 필드별 범위·기본값·오류 문구·crossing 관계와 raw state는 local에 유지 |
| `shared/lib/draftMap.ts` | `removeDraftIfUnchanged`는 저장 당시 초안과 현재 항목이 Object.is로 같을 때만 immutable map에서 제거. 이후 편집을 보존하는 순수 helper이며 broker scope epoch와 저장 성공 정책은 Page가 소유 |
| `shared/lib/formatters.ts` | 공통 숫자·금액 표시. 연구 화면의 `fmtRoundedKrw(number)`는 기존 Math.round→ko-KR→원 순서를 보존하며 일반/회계 패널이 alias로 재사용한다. BrokerMoneyView의 통화·누락·decimal 문자열 처리를 하는 `fmtBrokerMoney`와 음수 절반 반올림 의미가 달라 별도 유지 |
| `pages/strategy/model/candlePageCollector.ts` | 단기 모드는 최대 8회 요청·500봉을 유지하고 평가 50/100/200봉과 이전 최대 252봉을 나눈다. 별도 연속 일봉 수집은 명시 날짜 범위·최대 48회 요청·4608봉 보관·평가 4096봉·엄격한 이전 252봉을 요구하고 준비 실패 시 계산을 보류한다. scope/asOf/source 검증·중복/커서 무진전·provider 오류와 첫 성공 이전 KIS 거래소 fallback을 공유한다. 연속 모드의 시작/종료 차이·관측 간격 7달력일 제한은 보수적 앱 경계이며 coverageVerified=false를 유지한다 |
| `shared/api/candlePageTypes.ts` | desktop IPC와 Web GET에 공통인 `CandlePageInput`/`CandlePageView` 계약과 scope·커서·완료 정책·제외 수 타입. 기존 Vec 차트 계약과 구분 |
| `pages/strategy/model/experimentStore.ts` | credential을 저장하지 않고 broker/account/strategy/symbol scope별 A/B 결과·전략 버전·파라미터·데이터 범위/source·비용 가정·생성 시각을 localStorage에 최대 2개 저장; 연속 결과의 실제 calculationMode/continuity를 함께 보존하고 과거 누락 모드는 추정하지 않음 |
| `pages/strategy/ui/bollingerControls.tsx` | 레버리지 추세 보유 전략의 선택적 볼린저 보강 활성화와 기간·배수·스퀴즈·ATR 설정 입력 |
| `pages/strategy/ui/leveragedTrendHoldPreviewChart.tsx` | 레버리지/일반 전략 미리보기의 캔들·종가선·signal marker, 모바일 가로 패닝·핀치 확대/축소와 버튼식 줌 표시. offset 포함 시각은 실제 instant로 파싱하고 compact 분봉은 기존 KST 규칙으로 해석 |
| `pages/history/ui/Page.tsx` | 활성 broker scope, 자동매매 체결 기록과 기간별 통계 조회, provider 원본 trace 표시 |
| `pages/log/ui/Page.tsx` | 로그 레벨/검색 필터, provider trace 토큰 chip 표시 |

### Backend (Rust)

| 모듈 | 책임 |
|------|------|
| `lib.rs` | Tauri Builder + window-state 플러그인 + 6개 백그라운드 데몬 spawn + `on_window_event` (종료 안전 처리) |
| `commands.rs` | AppState + IPC command facade, 공통 helper |
| `commands/accounts.rs` | 계좌 프로파일 CRUD/활성화, KIS 국내·해외 잔고, broker holdings view. KIS 잔고는 공통 holdings fetch/publish 서비스를 사용하고 broker holdings 읽기 응답은 ReadOnly identity gate로 검증. 프로파일 적용 전 실행을 무효화하고 자료 로드 뒤 manager → OrderManager 잠금에서 scope·client 교체·tracker 초기화·전략 복원을 함께 게시 |
| `commands/archive.rs` | 거래 파일 보관 설정, 보관 통계, 오래된 trade/log 파일 purge IPC |
| `commands/budget.rs` | 활성 broker/account 검증, 자동매매 전용 예산 조회·변경 IPC. 변경은 자동매매 정지와 저장소/프로파일 변경 잠금 경계에서 수행 |
| `commands/database.rs` | 인증 없는 REST에는 노출하지 않는 Tauri-only DB 관리/이관 IPC |
| `commands/market.rs` | KIS 국내/해외 시세, 차트, 종목 검색, 해외 주문 사전 검증 IPC |
| `commands/candles.rs` | 공통 읽기 전용 페이지 IPC와 Web 공유 서비스. 활성 profile/broker/account와 KIS client 일치, 커서의 scope/asOf/provider를 검증한다. OHLCV·통화를 검증하고 서울/뉴욕 현재 시장 기간보다 이전인 일/주/월봉 또는 asOf 이하 종료 시각의 분봉만 정렬·중복 제거해 반환한다. 완료 정책과 미완료/무효 제외 수를 노출하며 기존 Vec 차트 IPC는 유지 |
| `commands/candles/tests.rs` | 시장 날짜·주/월 경계, DST 날짜와 분봉 offset instant, 미완료/무효/중복 제거, 고정 asOf 커서 및 profile/account/broker 불일치의 fail-closed fixture |
| `commands/orders.rs` | 수동 주문 제출 IPC |
| `commands/records.rs` | 체결/거래/통계 조회, Discord config 저장, frontend log 저장 IPC |
| `commands/settings.rs` | app config/check_config, refresh interval, log/web 설정, USD/KRW 환율 IPC |
| `commands/strategy_preview.rs` | 기존 단기 generic과 새 continuous 공개 surface/IPC를 re-export하고 공통 변환·현금흐름 가정 검증 및 Toss LTH preview를 유지한다. LTH는 활성 profile/account·분봉 시작 이전 완료 일봉 맥락을 검증하고 일봉 합성 장중 관측을 실행하지 않는다. 분봉 Open/Signal/Close를 ReplaySession에 연결하며 assessment의 진단/표본/시각·세션 미검증 경계를 유지한다. provider I/O 전 referenceAsOf를 고정하고 실행용 lexical tail/가격 필터를 지난 원본 timed 라벨만 별도 minuteTiming으로 진단해 시그널·정산·입력 hash와 분리 |
| `commands/strategy_preview/generic.rs` | 기존 단기 입력 DTO·신호/준비 view와 공통 OHLC/시각 정규화. history+평가 합계 500봉, legacy warmupCount와 strictly-prior history 경계를 보존한 뒤 공통 driver에 한 청크로 위임한다. 기존 flat 입력 정렬은 유지하고 연속 입력에는 별도 엄격 시간순 정규화를 제공한다. generic LTH 및 강한 종가/변동성 확장의 비일봉 거부 유지. 분봉 RFC3339/compact KST 구문 해석만 minute_time::parse_minute_instant에 위임하고 기존 고정 정밀도 UTC 정규화 출력은 유지 |
| `commands/strategy_preview/driver.rs` | 단기/연속이 공유하는 생산 전략 driver. 과거 warmup·ReplaySession을 한 번 만들고 모든 청크를 시간순으로 공급해 전략 지표·실제 포지션 피드백·guard/risk·pending·ATR을 보존하며 끝에서 한 번만 finish/비교한다. 정규화 현금흐름 검증·전역 준비/분할 지표·결과 조립과 mode를 포함하고 청크 분할은 제외한 입력 hash를 소유. factory 전에 Consecutive/Momentum/Deviation/MeanReversion/MA 교차/RSI 새 params와 Consecutive ATR/MA·MA 교차4필터의 일봉 간격을 검증하고 평가 후 선택 진입 준비/후보 진단을 최상위 응답에 추가. Momentum/Deviation/MeanReversion 옵션은 기존 params로 전송하며 새 응답 DTO를 만들지 않음 |
| `commands/strategy_preview/continuous.rs` | 공급 자료만 사용하는 단일 요청 연속 API. 1~16청크·각 1~500봉·평가 총4096봉·history 최대512봉을 제한하고 청크 안/사이의 시간 역전·중복과 평가/history 겹침을 실행 전에 거부한다. 서버 세션 저장 없이 공통 driver 한 인스턴스를 사용하며 기존 preview JSON에 continuity ledger를 함께 반환 |
| `commands/strategy_preview/minute_timing_tests.rs` | 합성 BrokerCandle로 실제 준비 lexical tail/count200·가격 제외와 실행 라벨 일치, daily 진단 생략/빈1분 평가불가, 공용 parser wrapper byte 호환 및 진단 유무/asOf 차이가 생산 신호·정산·입력 hash를 바꾸지 않는 경계를 검증 |
| `commands/strategy_preview/rsi_entry_tests.rs` | 실제 단기/연속 preview의 optional strict 거부, nextOpen 실제 체결 원가/손절·체결 뒤 보유 나이/청크 경계, 분할 일치·미래 suffix 인과성, 거절 교차 비재생·flatSell/heldBuy 제거와 최소수익 guard 거부 뒤 cap 재후보를 합성 입력으로 검증 |
| `commands/strategy_preview/ma_cross_entry_tests.rs` | 생산 단기/연속4옵션 strict 검증·활성 비일봉 거부/기본 분봉 호환, integer-float 활성·자료 부족 중 Sell 유지, 현재 OHLC 제외·분할 불변/미래 prefix 인과성·guard 거절 뒤 교차 비재생을 합성 입력으로 확인 |
| `commands/strategy_preview/mean_reversion_entry_tests.rs` | 실제 생산 단기/연속 driver의 새 params 엄격 거부, sameClose/nextOpen 청크 불변·미래 suffix 인과성, guard 차단/동일0 pending 재시도·실제 체결 뒤 보유 나이·손절 우선순위/native 원가와 다음 시가 체결을 합성 가격으로 검증 |
| `commands/strategy_preview/deviation_entry_tests.rs` | 합성 가격으로 단기/연속 strict 검증, 실제 생산 전략의 sameClose/nextOpen·분할 불변/미래 prefix, guard 거절/동일0 피드백 재시도와 실제 체결 뒤 나이·손절 우선순위를 검증 |
| `commands/strategy_preview/momentum_entry_tests.rs` | 생산 short/continuous 양쪽의 새 params strict 거부, 기본 level/crossing/간격의 같은 장부 분할 일치·미래 prefix 불변, guard 차단과 nextOpen 동일0 피드백 뒤 미체결 재시도 회귀 |
| `commands/strategy_preview/entry_filter_tests.rs` | 생산 preview의 신규 params·간격 거부와 정수값 float 표현 유지, 기본 진단 생략·조건/준비/후보 구분, 현재 후보 OHLC 배제와 동결 SPY 비용별 매도 경로 회귀 |
| `commands/strategy_preview/continuous_tests.rs` | 실제 생산 전략의 500봉 초과 단일/분할 공급 일치, 매수·매도 pending 경계, cooldown·일일 손실/시장 날짜·연속 손실 차단 보존, 미래 prefix 불변·strict 순서/상한·FX/최소수수료/ATR·모드 hash·중립 구간 경고 회귀 |
| `docs/research/continuous-replay-20261018/{design.json,baseline_runner.rs,runner.rs}` | 사전 고정한 3년 연속 창·이전 warmup과 분기 리셋을 같은 생산 12전략/수량/비용으로 별도 실행. 직전 flat 기준과 전체 report/typed 신호 호환을 확인하고 연속 공급 partition 비교·리셋 결과를 분리 보관하며 분기 수익을 연속 수익으로 합성하지 않음 |
| `docs/research/continuous-replay-20261018/validate.py` | 기록 fill·원본 OHLC로 현금·부분 원가·매봉 equity·관측 비중·terminal·실제 수량/보유구간 시장손익·chunk ledger를 독립 대사한다. 연속/초기화 JSONL을 손실 없는 .gz로 보관하고 평문은 Git 제외 프로젝트 .cache로 이동하며 summary.csv·aggregate.json·validation.json에 집계/압축 전후 SHA·소스 hash를 기록 |
| `commands/strategy_preview/generic_tests.rs` | generic 분봉의 UTC/offset 시각 정렬·동일 instant 중복·사전자료 겹침 거부와 실제 관측 간격 보존 회귀 fixture |
| `commands/strategy_preview/tests.rs` | 일봉 정보 공개 시점, session 경계, 미래 봉 변경이 이전 신호에 영향을 주지 않는 deterministic fixture |
| `commands/strategy_preview/preparation_tests.rs` | 동일 평가 기간, 사전자료 겹침·중복 입력 거부, 지표 준비 상태·실제 조건 평가봉 수와 사전자료 부족 fixture. generic LTH의 기본/접미 ID·간격·history 조합과 malformed 입력의 factory 이전 명시 거부 검증 |
| `commands/strategy_preview/daily_event_tests.rs` | generic 일봉의 전일 조건·이전 범위, 실행 차단 피드백, 평가 중 준비, 미래 봉 변경 불변과 비일봉 거부 회귀 검증 |
| `commands/toss.rs` | Toss accountSeq 조회, 연결 진단, 주문 전 preflight view facade, 접수 주문 목록 조회와 정정 command |
| `commands/toss/small_order.rs` | 호환용 Toss 1주 소액매매 검증 endpoint. 실거래 동의/최종 확인/최대 허용금액/preflight/open-order scan 후 시장가 매수 제출. 즉시 체결은 원 주문 ID·종목·BUY·통화·정수 1주·명시 체결대금과 체결일자(미제공 시 주문일)를 확인해 일반 fill 경계에 전달하고 미확정 자료는 대조 대상으로 유지. 현재 UI는 일반 수동주문 경로를 사용 |
| `commands/toss/small_order/tests.rs` | 원 주문과 다른 응답의 ID·종목·방향·통화·수량 거부, 금액 누락/실제 0수량 보류 및 주문가·평균가로 체결대금을 만들지 않는 회귀 fixture |
| `commands/toss_market.rs` | Toss 시세 snapshot, 종목 유의사항, market-calendar override, candles chart command |
| `commands/trading.rs` | 자동매매 상태/시작/정지·매수 정지 해제의 4개 IPC와 시작 순서·scope 검증을 조립하는 facade. daemon·잔고·위험 복원·히스토리 initializer의 기존 공개/내부 re-export 경로를 유지. 시작 준비 뒤 검증한 profile/client와 revision으로 실행 epoch를 게시하고 실행 플래그를 켠다. 정지는 실행과 일봉 generation을 무효화하며 예약·pending 추적은 보존. 상태에 완료 일봉 진단 포함 |
| `commands/trading/positions.rs` | 활성 broker의 KIS/Toss 전체 잔고를 고정 holdings query로 조회·파싱하고 manager → OrderManager 아래 KR/US tracker에 게시. 전략 복원은 같은 gate에서 소유 원장의 exact 전략별 수량·metadata로 수행하며 계좌 총보유를 전체 전략에 broadcast하지 않음 |
| `commands/trading/balance.rs` | KIS 국내/해외 및 Toss 계좌의 현금·평가잔고를 조회해 리스크 기준 총잔고를 KRW로 환산 |
| `commands/trading/risk_restore.rs` | 실행 broker/account·투자 모드를 검증한 당일 체결·주문 ledger로 리스크 후보를 만들고 저장 성공 후 공개. 당일 체결 이벤트 영수증도 같은 durable trade 원본에서 재구성해 pending 재시도의 손익 중복 반영을 방지. KIS 모드 출처 불명·조회·저장 실패를 호출자에게 전달 |
| `commands/trading/risk_restore/tests.rs` | 리스크 복원의 계좌·실전/모의 출처 격리, 실패 전후 기존 상태 보존과 이벤트 영수증 경계의 합성 회귀 fixture |
| `commands/trading/history.rs` | 활성 전략의 broker별 과거 OHLC 조회·통화 단위 변환과 일봉/분봉 warmup dispatch. live 초기화용 `calculate_atr`를 소유하며 replay 전용 ATR 준비 모듈과 구분. 52주 전용 종목은 legacy seed를 생략하고 공유 종목의 다른 전략은 기존 seed를 받는다. Consecutive의 legacy 종가 seed는 방향 관측에만 사용하고 검증된 완료 일봉 context로 승격하지 않는다. Toss 실행 scope의 프로파일이 없으면 KIS로 대체하지 않고 초기화를 중단 |
| `commands/trading/daemon.rs` | 영구 `run_trading_daemon`과 시세 polling 사이클, KIS/Toss 가격 소스·세션 판단·장 종료 대기·해외 거래소 조회 helper를 소유. 실행에 고정한 provider로 조회하고 fence로 오래된 잔고·quote·종목명·포지션 결과와 전략 복원을 폐기한다. 시장 대기 전 일봉 작업을 관리하고 tick 직전 시장 날짜 변경 재확인. facade의 기존 export를 통해 `lib.rs`에서 실행 |
| `commands/trading/daemon/fence.rs` | running → StrategyManager → OrderManager 잠금 순서의 실행 snapshot 캡처·현재 context 검사. provider await는 잠금 밖에서 수행하고 전략 tick/복원은 현재 epoch·revision일 때만 적용. 잔고 동기화 상태·일 초기화도 실행 범위 검증 후 반영 |
| `commands/trading/daemon/fence/tests.rs` | quote·종목명·포지션 대기, 설정 변경·정지/재시작과 stale 전략 상태 복원 경계의 합성 회귀 검증 |
| `commands/trading/daemon/flow.rs` | 설정 편집 대기와 실제 시장 종료를 구분하고, 장 종료로 제출하지 않은 신호 묶음의 실제 포지션 복원·현재 revision 정산을 연결. 설정 저장 대기는 짧게 재확인하여 장 종료의 긴 대기로 바꾸지 않음 |
| `commands/trading/daily_history.rs` | 공통 완료 페이지로 live 52주 및 opt-in Consecutive/MA 교차 확인용 일봉 사전자료 수집. 작업당 최대 8회 요청·512봉 유지, 이전 시장 날짜·OHLC·scope/asOf/source 검증과 거래소 fallback·중복·커서 무진전·부분 오류 종료. 마지막 완료봉을 다시 제외하지 않으며 preview의 500봉 상한과 분리 |
| `commands/trading/daily_history/jobs.rs` | 실행 프로파일에 고정한 background scheduler. 같은 profile/종목의 여러 설정을 최대 요구량 1회 조회로 공유하고 동시 2작업만 실행. 완료 handle만 수거하며 제거/새 generation/config/scope/시장 날짜의 과거 작업 취소. 실패/부족은 30~900초 backoff, 정지/drop 시 작업 중단 |
| `commands/trading/daily_history/{tests.rs,jobs/tests.rs}` | 합성 DTO/fetch로 실제 252/512봉·중복/미완료/오류·scope drift·수집 상한, 공유 조회·동시 2작업·backoff·대기 중 손절·정지/재시작 결과 폐기 검증 |
| `commands/strategy.rs` | 포지션 조회, 전략 목록/수정 IPC. 공통 `trading/configuration.rs`에 검증·저장·보유 상태 보존과 revision 게시를 위임하며 전략 view는 `trading/views.rs::build_strategy_view()` 재사용 |
| `commands/risk.rs` | 리스크 설정/비상정지 IPC와 pending 주문 view |
| `trading/strategy.rs` | 전략 facade. 공개 import 경로를 유지하면서 하위 전략 모듈을 re-export |
| `trading/strategy/classic.rs` | 분리된 RSI·Momentum·Deviation 타입/검증의 공개 re-export facade. 기존 import 경로와 historical/reconfigure/live feedback 회귀 진입을 유지 |
| `trading/strategy/classic/rsi.rs` | 기존 period 변화의 단순평균 RSI·zero-loss100·매 관측 prevRSI 교차 갱신과 실제 owned 수량/native 평균가·보유 나이/진입 origin을 분리. flat Buy/held Sell만 허용하며 손절→보유 한도→원래70하향 교차 순서로 실제 잔량 전량을 요청. 선택 추세는 current 제외 이전 입력가격이며 live 첫 seed만 분리. exact kind=rsi metadata·warmup/재구성 보유 보존·reset 전체 clear를 연결 |
| `trading/strategy/classic/rsi/params.rs` | custom 기존 period/oversold/overbought를 유지하는 기본0 선택 추세0..512정수·손절0..99.9유한수·보유0..1000000정수 파싱/검증. optional 손상으로 legacy 설정이나 유효 보유 보호를 덮지 않고 runtime 신규 Buy만 차단. 공통 정수 helper를 재사용 |
| `trading/strategy/classic/rsi/tests.rs` | 단순평균/횡보100·교차 경계/소비, 실제소유/잔량·동일0/부분/거절·live추세 seed와 bounded 버퍼, stop0off·우선순위/준비 부족·known/unknown age·strict metadata·warmup/설정/범위/reset 보호를 합성 fixture로 검증 |
| `trading/strategy/classic/momentum.rs` | bounded 가격 관측·이전 모멘텀·무장/미체결 후보와 실제 owned_quantity를 분리. 기본 inclusive level과 선택 entry crossing/%p 재무장, 같은0 피드백 재시도·부분 청산 뒤 보유 매도를 유지. warmup은 가상 후보를 만들지 않고 동일 params 재구성은 상태 보존, params 변경은 소유/잔존 가격을 남겨 진입 관측을 재seed |
| `trading/strategy/classic/momentum/tests.rs` | inclusive 경계·동일0/실제 양수 피드백·후보 취소와1%p 재무장 분리, warmup/부분 청산·거절 뒤 보유 매도·잘못된 optional 설정/legacy값, 동일/변경 설정 재구성과 live pending 관측·bounded/미래 prefix 불변 회귀 |
| `trading/strategy/classic/deviation.rs` | 실제 owner 수량·native 평균가·보유 관측 나이를 가격/진입 후보와 분리. flat Buy/held Sell, 선택 하향 교차 재시도와 손절→관측 한도→고이격 청산을 평가하고 보호 후보는 잔량 전량 요청. exact metadata로 나이/진입 후보를 복원하며 같은 수량·부분 매도·warmup·설정 변경으로 나이를 초기화하지 않음 |
| `trading/strategy/classic/deviation/params.rs` | 기본 legacy MA/임계값을 보존하는 optional 진입 모드·장기 관측·최대 보유·손절률 파싱/검증. 손상된 optional 값으로 유효 legacy 청산을 덮지 않고 runtime 신규 진입은 차단 |
| `trading/strategy/classic/deviation/state.rs` | bounded 현재 포함 MA 가격열과 이전 이격도/하향 교차 후보를 관리. 장기 추세는 현재 제외 직전 N관측 평균을 사용하고 live 전용 provenance·512개 추세 버퍼를 별도 유지 |
| `trading/strategy/classic/deviation/tests.rs` | 실제 수량/평균가 피드백·inclusive 경계·교차 재시도/취소, live 추세 seed 격리·bounded 관측, 체결 후 나이와 부분 매도·재구성, 청산 우선순위·unknown age·metadata 검증·잘못된 optional 값/legacy 청산 회귀 |
| `trading/strategy/mean_trend.rs` | 기존 TrendFilter 구현과 MeanReversion 공개 타입·검증 re-export facade. TrendFilter 계산·판정은 유지하며 평균회귀 구현 책임을 하위 모듈로 분리 |
| `trading/strategy/mean_trend/mean_reversion.rs` | 현재 포함 모집단 분산 Bollinger 밴드의 strict 하단 진입·기본 상단/선택 중심선 청산. 실제 수량/native 평균가만 피드백으로 갱신하며 손절→최대 보유→밴드 순으로 평가. 손절/밴드는 설정 수량, 보유 한도는 실제 잔량을 요청. metadata·same/changed params 재구성·legacy reset 가격 보존·분봉 initializer no-op을 연결 |
| `trading/strategy/mean_trend/mean_reversion/params.rs` | 기존 period/std_dev/stop_loss_pct를 보존하는 청산 모드·장기 관측0..512·보유 관측0..1000000 파싱/엄격 검증. optional 손상은 runtime 신규 진입만 차단하고 유효 legacy 밴드/손절을 덮지 않음 |
| `trading/strategy/mean_trend/mean_reversion/state.rs` | bounded 가격열과 실제 owner 수량/native 평균가, 보유 나이/진입 origin 후보, live 전용 이전 관측 추세를 분리. 공통 observation helper에 추세 산술/버퍼 동작만 위임 |
| `trading/strategy/mean_trend/mean_reversion/tests.rs` | strict 밴드·기존 손절0·청산 우선순위와 수량, actual/부분/동일0 피드백·나이 불명/metadata·재시작/재구성/reset 가격 보존, live seed 격리·512추세 상한과 513관측 준비·손상 optional/legacy 청산 회귀 |
| `trading/strategy/observation.rs` | Deviation/MeanReversion/RSI가 공유하는 pub(super) helper8개: optional 정수 범위 판정, 보유 나이 증가/동기화, kind를 검증하는 v1 metadata encode/decode, 직전 N추세 판정·512개 버퍼 갱신·최초 live seed 정리. 전략별 밴드/손절·후보 취소·reset·1분 warmup 정책은 소유하지 않으며 kind가 다른 checkpoint를 거부 |
| `trading/strategy/core.rs` | `Signal`, `StrategySignal`, `BrokerPositionSnapshot`, `StrategyConfig`, `HistoryReadiness`, `EntryFilterPreparation`, `Strategy` trait와 live/preview 공통 `initialize_strategy_warmup()`. live의 관측 평가·확인된 소유 수량·pending·미제출 신호 폐기·보호 metadata 훅을 replay와 구분한다. 일봉 OHLC 초기화 기본 훅은 종가를 historical initializer에 전달한다. replay 날짜 시작·완료봉 훅은 기본 no-op, 종가 평가 훅은 기존 `on_tick()` 위임이다. 준비·평가 가능 상태와 내부 보유 여부의 미지원 값은 `None`이다. 기본 false인 `reconfigure_preserving_position` opt-in으로 같은 ID/broker/account 설정 변경의 보유 상태 보존 계약 제공. opt-in required/apply/invalidate/readiness 완료 일봉 훅으로 검증된 live context를 legacy initializer와 구분 |
| `trading/strategy/manager.rs` | `StrategyManager`, `build_strategy()`, config 적용과 live warmup dispatch. live 관측·pending 신호 억제와 exact 전략/scope/시장 소유 피드백·metadata 훅을 연결한다. legacy warmup의 최신 1봉 제외는 유지하되 새 52주 live 실행은 완료 페이지 경계로 준비. config 변경·복원·reset은 일봉 generation 무효화. Consecutive legacy warmup은 방향 관측만 준비하며 늦게 도착해도 완료 일봉 검증 상태를 무효화해 새 scoped 수집을 요구한다. 13개 전략은 같은 ID/scope 설정 변경 시 opt-in 훅으로 보유 상태와 유효한 관측을 보존한다. 프로파일 게시 경로는 같은 계좌 문자열이어도 새 인스턴스를 구성 |
| `trading/strategy/manager/configuration.rs` | 설정 편집 Weak 예약의 활성 여부 관리, 현재 revision의 미완료 신호 확인·정산과 미제출 신호 복원. 전략 내부 보유 보호 중인 대상 제거·시장 변경을 저장 전에 거부 |
| `trading/configuration.rs` | IPC/Web 공통 설정 변경 서비스. 작업 소유 task와 `strategy_update_lock`으로 호출자 취소와 동시 저장을 제어한다. 생산 constructor로 대상 정규화 후 manager → OrderManager에서 편집 예약을 만들고 잠금 밖에서 저장. 성공 후 identity 재검증·보유 상태 보존·revision 게시, 사후 검증 실패 시 원본 저장값 복구와 복구 실패의 영속화 차단. Consecutive/Momentum/Deviation/MeanReversion/MA 교차/RSI 신규 params 오류를 저장 전에 명시 거부 |
| `trading/configuration/tests/rsi.rs` | 실제 manager/store와 합성 소유 fixture로 RSI optional 검증의 저장 전 거부/정수float roundtrip, 교체 후 저장 실패 rollback의 실제 원가·나이 보존, 보유 대상 제거 거부와 성공 편집의 나이 보존을 검증 |
| `trading/configuration/tests.rs` | 실제 manager/store와 합성 주문 fixture로 저장 실패·지연·호출자 취소·보유 대상 제거·프로파일 변경·사후 복구 경계를 검증 |
| `trading/strategy/{classic,breakout,mean_trend,sequence,ma_cross,price_condition,leveraged_trend_hold}/reconfigure_tests.rs` | 전략별 설정 변경의 보유·손절·관측 보존, 파라미터 재해석과 다른 ID/scope 상태 격리를 검증 |
| `trading/strategy/{classic,breakout,mean_trend,sequence,price_condition,leveraged_trend_hold,manager}/live_feedback_tests.rs` | live 평가가 확인된 보유를 미리 변경하지 않는 경계, exact 전략 피드백·미제출/부분 체결·pending 보호와 LTH 상태 metadata 회귀 fixture |
| `trading/strategy/manager/live_history.rs` | run generation·전체 config fingerprint·profile/broker/account·종목·서울/뉴욕 시장 날짜 ticket 검증과 opt-in 전략별 완료 OHLC 적용(52주 고가·Consecutive/MA 교차 별도 진입 context). 실제 대상 511개+초과 진단 1행 상한, 준비 무효화·실패/부족/재시도 상태 및 최신 완료봉의 1~10일 freshness 관리. 보유 청산을 유지하고 Consecutive/MA 교차의 가격 관측열을 지우지 않으며 LTH 분봉 initializer 미호출 |
| `trading/strategy/manager/live_history_tests.rs` | 마지막 완료 high/close·다음날 고가 만료·252/512봉·대상 cap·재시작/config ABA·scope/profile 변경·quote 중 자정·DST·실패 중 손절·동일 scope 설정/rollback 보유 보존·무관 전략 불변 fixture. Consecutive context 갱신/무효화·15/512봉 요구량·이전 tick 관측과 기존 청산 보존도 검증 |
| `trading/simulation.rs` | deterministic simulated portfolio, 공통 TradeGuard/RiskManager 기반 주문 가능 판정, 비용·환율·equity·MDD·turnover·exposure·고정 chronological in/out-of-sample 지표. `run_backtest`/`run_backtest_with_history`는 ReplaySession을 구동하는 batch facade다. 동일 주기 과거 관측으로 위험 지표만 준비하고 주문·equity는 평가 구간에서만 만든다. ATR 미준비·변동폭 0 또는 거래당 위험 예산 0이면 선택적 ATR 수량 산정의 신규 매수를 차단하고 보유분 매도는 유지한다. preview command가 engine/strategy/source/scope/params/가정/warmup 경계와 실제 replay OHLCV fingerprint를 hash한다. v9 replay 시각은 RFC3339 offset의 실제 instant를 우선 사용한다. |
| `trading/simulation/session.rs` | 공통 ReplaySession 회계·리스크·포지션·보류 신호 상태. begin_bar에서 해당 봉 FX와 직전 보유 수량의 배당을 적용한 후 전일 손실 상태 갱신·이전 신호 시가 체결·연구 보호 청산을 처리하고 process_close에서 ATR·종가 신호·mark/equity·관측 투자 비중을 반영. native 평균 체결가와 진입 KRW 원가를 분리하고 sameClose/nextOpen, 원시 신호·보호 주문·마지막 미체결을 구분 |
| `trading/simulation/accounting.rs` | 비용 출처/방향별·최소 수수료, 최대 64개 FX·배당 시나리오 DTO와 평가 봉 라벨·중복·수치·배당 조정 가격 중복 계상 검증. 공통 fee 계산과 회계 결과 DTO를 소유 |
| `trading/simulation/session/accounting.rs` | 실현·잔여·가격/환율·배당 귀속, 실제 수량의 매도 projection과 최소수수료 기대순익 gate, 종료 가상 청산·투자 비중·표본/출처 한계 생성. 종료 청산은 실제 거래·리스크를 변경하지 않고, 최소수수료 모드의 일반 이익 매도만 실제 비용 순익을 검사하며 손절·native 손실 청산을 보존 |
| `trading/simulation/accounting_tests.rs` | 부분 매도·잔여 매수 비용·현금흐름 대사, 미래 FX 차단·native 진입가 불변, 배당 권리/원천징수·가격 조정 중복 거부, 가상 청산 비변이·노출/부호 경고, 최소수수료·실제 수량/환율 순익과 손절 예외 회귀 |
| `trading/simulation/comparison.rs` | 같은 입력 수량의 첫 평가 종가 보유 지시를 같은 ReplaySession으로 계산해 실제 수량·봉말 투자 비중 차이를 공개. 실제 체결 경로를 고정한 발생 시점 KRW 비용 환급 산술 참조를 제공하고 독립 무비용 전략 재실행과 구분 |
| `trading/simulation/execution_tests.rs` | 다음 관측 봉 시가·마지막 미체결·갭 수량/리스크·미래 ATR 차단·실제 체결 원가 보호/부분 매도·동시 도달 정책·chunk/batch 회계 일치·기본 sameClose 수치 호환 합성 회귀 |
| `trading/simulation/assessment.rs` | optional `ReplayAssessmentView`로 LTH의 dailyDiagnostic/notEvaluable과 intradaySample/sampleOnly를 구분한다. 분봉 관측0개도 notEvaluable이며 일봉 맥락·장중 봉 수와 미검증 시각/세션·모델 한계를 제공. 선택 minuteTiming은1분 전용 진단이며 timestampStatus/sessionStatus·lookAheadSafe·성과 적합성은 바꾸지 않음 |
| `trading/simulation/minute_time.rs` | pub(crate) 순수 분봉 구문 parser. RFC3339 offset을 UTC 실제 순간으로 변환하고 compact12/14는 기존 KST 가정으로 해석. naive local ISO/date-only를 추측하지 않으며 generic wrapper와 시각 진단이 재사용. 세션/봉 완료·호스트 시간대·전략 시계를 소유하지 않음 |
| `trading/simulation/minute_timing.rs` | 별도 lth-minute-timing-v1 진단과 camelCase DTO. barClose/barStart/unknown·US NY/KR Seoul·명시 asOf로 라벨의 시작/종가 공개 가능 시각, 중복/역순/비분정렬·일반 평일 정규창·주말/미완료·동일 관측일 내부 슬롯 공백을 계산. 입력512·라벨64byte·날짜8·예시6 상한과 산술 gap으로 bounded 유지. 실제 휴장/조기마감·provider/요청 coverage·실제 발행 검증은 false이며 실행 라벨을 정렬/추가/제거하지 않음 |
| `trading/simulation/minute_timing_tests.rs` | 합성 시각의 DST 전후/중복 현지시간·offset 동일 순간/역순·KST 자정 날짜, 봉 시작/종료/unknown·시장 일반창 경계·초/소수초·asOf·내부gap/앞뒤창·야간/주말/휴장 미인증·입력/날짜/예시 상한과 parser byte 호환을 검증 |
| `trading/simulation/atr.rs` | 이전 종가를 포함한 True Range 14개를 bounded ring에 보관하고 단순 평균 ATR을 계산한다. 같은 주기 관측 15봉을 요구하며 갭을 보존하고 누락 봉을 보간하지 않는다. `AtrPreparationView`로 사전 관측 수·시작/종료 준비·최초 준비 시각·미준비 평가봉 수를 제공 |
| `trading/simulation/tests.rs` | 비용·리스크·실행 차단, 날짜별 손실 상태, ATR 준비·위험 예산·실제 포지션과 동일 replay/report/hash 회귀 fixture |
| `trading/strategy/state.rs` | per-symbol 전략 버퍼 상한 helper. user-param 기반 `VecDeque` OOM 방지 |
| `trading/strategy/completed_daily.rs` | 검증된 완료 일봉의 OHLC 유효성·u128 종가 평균·이전 종가를 포함한 양수 TR14 산술평균의 순수 함수3개만 pub(super)로 제공. Consecutive/MA 교차가 공유하고 verified/lifecycle/counter·ATR0 준비 정책은 각 context가 소유 |
| `trading/strategy/{classic,breakout,mean_trend}.rs` | 분리된 RSI/모멘텀/이격도 공개 경로, 돌파 계열, 평균회귀/추세필터 전략 구현. `breakout.rs`의 52주 신고가는 완료 일봉 고가의 bounded ring에서 이전 기간 최고가를 구하며 날짜 경계 뒤 완료봉 1개만 적재해 오래된 고가를 만료한다. 반복 tick·분봉 초기화로 일봉을 전진시키지 않고, 강한 종가는 완료일 조건을 다음 평가일에 사용하며 변동성 확장은 보유 상태를 유지한 채 당일 OHLC와 이전 N개 완료 범위(0 포함)의 bounded ring을 구분한다. `mean_trend.rs`는 추세필터와 분리된 평균회귀 공개 경로를 유지하며 각 일봉 historical initializer는 기본 훅이 전달한 종가로 초기화한다. 평균회귀의 legacy 분봉 initializer no-op은 별도 유지한다. |
| `trading/strategy/breakout/high_tests.rs` | 52주 고가의 전체 준비 창·오래된 최고가 만료·신규 완료 고가·미래 참조 방지·반복 tick/분봉 초기화 불변·종목별 경계·보유 손절 보존 회귀 검증 |
| `trading/strategy/breakout/day_event_tests.rs` | 강한 종가·변동성 확장 event 훅의 날짜별 독립 상태, 차단 피드백·실제 보유 손절, 범위 ring 상한과 flat-day 회귀 검증 |
| `trading/strategy/ma_cross.rs` | 기존 관측 MA 교차·매 관측 prevMA 갱신/데드크로스 Sell을 유지하고 골든크로스 Buy에만 완료 일봉4필터 AND를 적용. 거절 교차는 소비하며 나중에 되살리지 않음. historical/native intraday seed와 검증된 일봉 apply/invalidate/readiness를 분리하고 context 재조회로 교차열을 초기화하지 않음 |
| `trading/strategy/ma_cross/params.rs` | legacy short5/long20 custom 기간을 보존하는 기본0 optional4필터 파싱/엄격 검증과 준비봉 요구량. 추세0..512 정수·기울기/ATR0..100·폭0..1000, 활성 기울기의 long_period1..511 관계를 검사하며 optional 손상은 신규 Buy만 차단 |
| `trading/strategy/ma_cross/context.rs` | 현재 제외 완료 일봉 OHLC 최대512개와 verified/seen/awaiting·후보 집계. 추세 엄격 현재>이전N평균·장기MA 1일변화율·ATR14/이전종가·20종가2σ밴드폭을 AND 판정. 충분자료 ATR/폭0은 ready 후 양수 임계값에서 거부하며 현재봉은 판단 뒤 게시 |
| `trading/strategy/ma_cross/context/tests.rs` | 완료 일봉의 범위/갭·분수 계산·strict 추세/기울기·현재봉 제외·ATR/폭0 준비 정책·준비/후보 집계와 잘못된 OHLC 거부 회귀 |
| `trading/strategy/ma_cross/entry_tests.rs` | 기본0/legacy 기간·잘못된 optional Buy 차단/Sell 보존, 거절 교차 소비·warmup/설정 재구성과 검증 일봉 context·native 관측 분리를 합성 가격으로 검증 |
| `trading/strategy/sequence.rs` | 분리된 Consecutive 모듈의 공개 타입·검증 facade와 기존 돌파 실패 전략. 사전 초기화에서 가상 매매를 만들지 않는 기존 계약을 유지 |
| `trading/strategy/sequence/consecutive.rs` | 연속 가격관측 방향·확정 보유와 기존 buy_days/sell_days를 유지하며 선택3필터를 신규 진입에만 AND 적용. 설정 재해석·보유 피드백·별도 완료 일봉 훅과 관측 후 완료봉 게시를 연결하고 손상된 새 params는 진입만 차단 |
| `trading/strategy/sequence/consecutive/context.rs` | 유한수/범위/정수 params 검증, bounded 이전 완료 OHLC의 ATR14·종가 MA와 준비·후보/거부/통과 카운터. 가격 tick을 일봉으로 합성하지 않고 context 무효화와 가격 방향/확정 보유를 분리 |
| `trading/strategy/sequence/consecutive/tests.rs` | 기본0·legacy 사용자 횟수, 누적/ATR/MA 경계, 가격-only·현재 OHLC 배제와 준비 부족, 손상 params·보유 청산·재설정/재구성·live 완료 context 보존 회귀 |
| `trading/strategy/leveraged_trend_hold.rs` | 레버리지 전략 공개 타입·config·상태와 bounded buffer 상한을 소유하는 facade. 일봉 맥락과 분봉 지표/반등·실제 보유 상태를 분리하고 하위 모듈의 live/timed 구현을 연결 |
| `trading/strategy/price_condition.rs` | 종목별 가격 조건 전략 구현 |
| `trading/strategy/leveraged_trend_hold/live.rs` | `Strategy` 구현과 live tick/소유 포지션 동기화. 확인된 보유와 신호 후보·pending을 구분하고 high-water/보유 관측 수를 metadata로 checkpoint/복원한다. 일봉 초기화는 별도 bounded 맥락 ring만 교체하고, 분봉 초기화는 분봉·반등·볼린저 snapshot을 교체하며 실제 보유 포지션은 유지 |
| `trading/strategy/leveraged_trend_hold/preview.rs` | 입력 timestamp 기반 timed replay와 Open/Signal/Close driver callback으로 공통 session의 체결 포지션을 피드백. 실제 잔여 수량·평균 체결가 snapshot을 사용한다. 부분 청산 뒤 잔량과 가격 기준을 보존하며 bool callback 호환 경로도 유지한다. EMA/RSI/ADX·반등·볼린저는 실제 장중 관측만 적재하고 일봉 맥락으로 분봉 준비 상태를 만들지 않음 |
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
| `trading/order.rs` | OrderManager facade와 공유 상태·provider trace. health·pending·legacy·execution_run·configuration·holdings·ownership·feedback·submission·fills의 하위 책임을 연결 |
| `trading/order/ownership.rs` | profile/투자 모드/broker/account/소유 전략/시장/종목 key의 확인된 수량·원가와 주문별 누적 체결 영수증. 고정 세대·상한·엄격한 역직렬화·동일 intent 검증, 부분 매도 원가·손익 계산과 보호 metadata 보관 |
| `trading/order/ownership/runtime.rs` | managed storage의 `position_ownership.json`과 generation manifest를 읽고 세대 연속성 검증·저장. manifest가 남은 원장 유실, 오염·세대 교체·저장 실패를 persistence block으로 전달 |
| `trading/order/feedback.rs` | 체결 대조 후 manager → OrderManager 순서로 현재 identity의 exact 소유 전략에 수량·평균가·pending과 metadata 전달. live 관측 뒤 보호 checkpoint를 저장하고 체결로 무효화한 계좌 전체 잔고는 provider에서 재조회 |
| `trading/order/feedback/tests/rsi.rs` | 실제 OrderManager exact owner·durable checkpoint 게시 경로의 pending origin/동일0/warmup·실제 원장 양수 체결·부분 매도·재시작 복원 나이를 합성 fixture로 검증. 계좌 총보유를 전략 소유로 복사하지 않는 경계를 유지 |
| `trading/order/feedback/tests/mean_reversion.rs` | 실제 OrderManager exact owner 게시·durable checkpoint 경로를 합성 fixture로 검증. 동일0 pending·warmup 뒤 진입 origin, 원장 양수 체결 뒤 나이·부분 매도·재시작 복원과 계정 총보유를 전략 소유로 대입하지 않는 경계를 확인 |
| `trading/order/feedback/tests.rs` | 다른 전략·수동 보유·계좌/프로파일/모드 격리, 재시작 metadata·부분 수량 변경·stale checkpoint와 이전 계좌 실패 응답의 health 오염 방지 fixture |
| `trading/order/execution_run.rs` | 비민감 `ExecutionRunContext`(epoch/revision/scope/profile)와 시작 검증에 사용한 client/profile snapshot. 시작·정지·scope 변경은 실행을 무효화하고 설정 revision 갱신은 이전 예약 후보를 폐기. pending·submitting·예산을 지우지 않음. 설정 편집과 holdings가 공유하는 비공개 profile 인증·모드 fingerprint 소유 |
| `trading/order/holdings.rs` | `HoldingsQuery`의 scope/epoch/client/profile/설정 편집 예약과 시장별 조회 버전을 캡처하고 게시 직전 검증하는 공통 gate. KR/US 버전을 모두 검사한 뒤 해당 시장만 교체하며 ReadOnly 응답은 tracker를 바꾸지 않음. 해외 주문 거래소 코드 정규화 helper 소유 |
| `trading/order/holdings/fetch.rs` | 공통 잔고 refresh 서비스와 체결 후 KIS/Toss 계좌 전체 재조회. 응답 변환은 `holdings/parse.rs`에 위임. provider await는 잠금 밖에서 수행하고 성공한 게시와 callback 이벤트는 같은 OrderManager gate에서 순서대로 처리 |
| `trading/order/holdings/parse.rs` | 일반 갱신·시작 복원·수동 주문이 공유하는 KIS/Toss→tracker 입력 변환과 엄격한 수량/가격 범위·종목 검증. Toss broker/account·시장/통화·시장별 중복을 확인하고 소수 잔량을 정수 주식으로 올리지 않음 |
| `trading/order/holdings/parse/tests.rs` | Toss 소수 잔량·가격 단위·부동소수 경계·오염 응답 격리, dirty KIS 시장만 갱신과 잘못된 수동 잔고의 tracker 게시·주문 전송 금지 fixture |
| `trading/order/holdings/{tests.rs,tests/http.rs}` | 조회 역순 완료·실제 0수량/빈 목록·양 시장 원자 게시·체결/포지션 변경·재시작·계좌/인증/모드·설정 lease 경합, localhost 지연 HTTP와 게시 전 성공 이벤트 금지 검증 |
| `trading/position.rs` | 국내 원화·해외 USD cents 포지션 tracker와 시세 평가. 고정 크기 UUID로 최신 조회와 체결/교체/초기화 변경 버전을 관리하여 이전 잔고 응답의 덮어쓰기를 차단하며 시세 갱신은 보유 변경 버전을 바꾸지 않음 |
| `trading/order/configuration.rs` | 설정 편집 예약의 profile/client/scope identity와 실제 KR/US tracker 보유·pending/submitting/modifying 충돌 검증. 편집 대상의 주문 예약과 같은 scope 수동 holdings 교체를 보류하며 Weak 예약 종료 시 자동 해제 |
| `trading/order/configuration/tests.rs` | 설정 편집 중 주문·holdings 차단, 예약 해제·identity 변경, 실제 보유 대상 제거·시장 변경의 합성 검증 |
| `trading/order/health.rs` | 매수 정지·계좌/체결 동기화·영속화 장애 상태, health view와 일 초기화 |
| `trading/order/pending.rs` | pending 복원·영속화·종료 상태와 취소/수정 예약, 종료 시 예산 정산 |
| `trading/order/legacy.rs` | 기존 단일 manager의 매수/매도 처리와 KIS 재시도 호환 경로. daemon의 shared 제출 경로와 구분 |
| `trading/order/submission.rs` | 수동·자동 주문 제출 조정. 자동 주문의 필수 실행 context를 준비 후·예약 전에 같은 OrderManager 잠금에서 검증하고 stale은 guard·submitting·예산 변경 전에 폐기. 예약 후 provider 응답·pending 처리는 원래 scope로 계속 추적 |
| `trading/order/submission/positions.rs` | 제출용 실제 포지션 snapshot과 수동 주문 전 holdings 갱신. 요청에 고정한 profile/client/scope로 공통 holdings query를 예약하고 provider await 후 같은 게시 gate로 tracker를 갱신 |
| `trading/order/submission/owned.rs` | 주문 전 소유 원장 세대와 exact 전략 key 고정. 자동 매도는 실제 관측 수량과 해당 전략 소유 수량으로 제한하고 수동 매도는 다른 전략·예산 소유분을 제외. 매도 원가 snapshot을 저장 주문에 연결 |
| `trading/order/submission/owned/tests.rs` | 실제 shared 제출의 owner별 자동/수동 매도 수량 제한, 준비 후 소유 수량 변경의 예약 재검증과 전체 잔고 선반영 fixture |
| `trading/order/submission/prepare.rs` | 위험·보유수량·주문 유형과 가격 검증, provider별 준비 요청 생성 |
| `trading/order/submission/provider.rs` | 준비 요청 전송. KIS 국내/해외 재시도는 최초 snapshot의 같은 client를 사용하고 Toss 요청은 고정 profile을 사용 |
| `trading/order/submission/records.rs` | 접수·실패 주문과 pending record 생성, provider trace·원래 broker scope 보존 |
| `trading/order/submission/run_tests.rs`, `trading/order/test_support.rs` | 합성 profile·캐시 토큰·localhost provider로 실행 context, 준비/예약 경합, 예약 후 client 고정과 pending 추적 검증. 실제 계좌/provider 검증과 분리 |
| `trading/order/budget.rs` | broker/account와 KRW/USD별 전용 현금·자동매매 소유수량·주문 예약 원장. 주문 전 예약 저장, 접수/체결/종료 반영, 미확인 주문·저장 실패 시 차단 |
| `trading/order/fills.rs` | 누적 체결 intent와 소유 원장·예산·리스크·통계·TradeStore의 반영 순서 조정. 실제 계좌 tracker에는 delta를 더하지 않고 조회 버전 무효화와 재조회 필요 상태를 남김 |
| `trading/order/fills/ownership.rs` | 저장 주문의 소유 key·세대·누적 영수증 검증과 확인 체결의 원가/손익 준비. 재시도 시 기존 손익 영수증을 재사용하고 초기 보유 metadata를 보존하며 종료된 소유 영수증 정리를 담당 |
| `trading/order/{ownership/tests.rs,ownership/runtime/tests.rs,fills/ownership/tests.rs}` | 원장 입력·상한·세대·저장 오류, 누적/부분 체결·재시도·소유 격리·확인된 원가와 보호 metadata 경계의 회귀 fixture |
| `trading/order/fills/collector.rs` | 고정 KIS client/Toss profile의 주문번호·종목·날짜 조회 결과를 공통 누적 체결/종료 상태로 정규화 |
| `trading/order/fills/collector/notional.rs` | 체결 수량·일자와 decimal 누적 금액 파싱. native 총액을 KRW 원/USD cents로 한 번 반올림하고 Toss 명시 금액 우선·누락 시 실제 평균가×수량 대체·명시 0/통화 오류를 구분 |
| `trading/order/fills/collector/notional/tests.rs` | decimal 반올림·범위, KIS 3주/301원 같은 원본 총액 보존, Toss 명시 총액 우선·명시 0과 누락 대체 계산의 회귀 fixture |
| `trading/order/fills/provider_mode.rs` | 저장된 KIS 주문 TR-ID 8종으로 실전/모의 출처를 확인하고 현재 client 모드와 비교. 반대 모드·누락/미상 출처의 pending 조회 후보·직접 체결·체결 조회의 종료 상태 및 예산 반영을 보류 |
| `trading/order/fills/reconciliation.rs` | 현재 scope의 pending 후보와 provider snapshot을 캡처하고 OrderManager 잠금 밖에서 조회. 적용 전 scope·profile·인증/모드를 재검증하며 다른 계좌 응답과 zero-fill 종료 처리를 보류. 같은 계좌·인증의 실제 체결은 전략 epoch 변경과 독립적으로 반영 |
| `trading/order/fills/reconciliation/tests.rs` | localhost 체결 조회 중 계좌·profile 변경의 결과 보류, 동일 계좌 실행 변경 후 실제 체결 반영, 저장 TR-ID의 실전/모의·미상 출처 및 직접 체결/종료 보류 회귀 검증 |
| `trading/order/conflicts.rs` | 실행 `BrokerScope` 기준 같은 방향/반대 방향 pending 주문 충돌 판정과 provider trace 기반 pending provider 판정 |
| `trading/risk.rs` | 일일 손실 한도, 비상 정지, `record_pnl` |
| `trading/risk/fill_accounting.rs` | 당일 체결 이벤트별 손익·전략 손실 보호를 한 번만 반영하는 bounded 영수증과 runtime 저장/복원. 다른 날짜 체결과 변경된 이벤트 payload를 구분 |
| `trading/risk/fill_accounting/tests.rs` | 체결 손익 중복·이벤트 변경·날짜·산술 상한과 runtime 복원 경계의 회귀 fixture |
| `tests/e2e/toss-commission-ratio.spec.ts` | 실제 Toss preflight 패널의 소수 비율→퍼센트 표시와 수수료 포함 필요 현금·잔고 부족 표시를 합성 자료로 검증 |
| `market_hours.rs` | 시장 개장 여부 (KRX 09:00-15:30 / US 22:00-07:00 KST) |
| `server/mod.rs` | axum 웹 서버 route table, ServeDir fallback |
| `server/budget.rs` | `/api/auto-trading-budget` GET/POST. IPC와 같은 scope 검증·정지 조건·예산 view를 사용 |
| `server/market.rs` | 웹 REST KIS 잔고/해외잔고, broker holdings, 현재가, 주문, 차트, 종목 검색/갱신. 잔고 읽기 응답은 공통 holdings ReadOnly identity gate를 거쳐 반환하며 tracker 게시를 만들지 않음. 공통 캔들 페이지 GET은 `commands/candles.rs`의 동일 서비스와 camelCase view를 재사용 |
| `server/records.rs` | 웹 REST 포지션, 통계/체결 조회, pending 주문, 로그 설정/최근 로그, 체결 보관 설정/통계, 프론트엔드 로그. REST 보관 설정 변경도 IPC와 같이 즉시 purge를 예약 |
| `server/profiles.rs` | 웹 REST 프로파일 CRUD/활성 전환/실전·모의 감지/Toss accountSeq·diagnostic. 적용 전 실행을 무효화하고 manager → OrderManager에서 scope·client·tracker·전략 설정을 함께 게시. 프로파일 view는 IPC `profile_to_view()`를 재사용 |
| `server/toss.rs` | 웹 REST Toss 시세 snapshot, 종목 유의사항, 주문 전 preflight, 호환용 1주 소액매매 검증 endpoint, 접수 주문 조회·정정, market-calendar, candles |
| `server/trading.rs` | 웹 REST 자동매매 상태/시작/정지, 전략 목록/수정. 웹 start는 broker별 설정 검증과 공통 holdings gate를 통한 KR/US 게시·exact 전략 소유 원장 복원 뒤 profile/client 고정 실행 epoch를 게시. stop은 실행을 무효화하고 전략 수정은 IPC와 같은 `trading/configuration.rs` 서비스를 사용. status는 실제 identity와 camelCase `dailyHistory` 제공. 제공 자료 기반 단기/연속 preview REST도 같은 생산 command 함수를 호출하고 연속 경로는 POST /api/strategy/preview/continuous로 연결 |
| `storage/trade_store.rs` | `data/trades/YYYY/MM/DD/trades.json` (`provider_*` 원본 요청 trace 포함) |
| `storage/order_store.rs` | `data/orders/YYYY/MM/DD/orders.json` (`provider_*` 원본 주문 trace 포함) |
| `storage/stats_store.rs` | `data/stats/YYYY/MM/daily_stats.json` |
| `storage/strategy_store.rs` | `data/strategies/{profile_id}/strategies.json` (`StrategyConfig`에 broker/account scope 저장) |
| `storage/database.rs` | DatabaseManager facade, PostgreSQL/MariaDB 문서 계층, schema v2 projection, transaction import/retention, JSON export와 fail-closed backend 전환. 상태는 같은 설정/pool snapshot으로 생성 |
| `storage/database/configuration.rs` | 설정 로드·검증·파일/OS keychain 저장·원복과 메모리 게시. 내구성 recovery 표시와 manager의 runtime latch로 취소·불확실 복구 시 다음 저장/연결을 차단 |
| `storage/database/connection.rs` | provider별 TLS 옵션과 관리자 DB 생성, pool 생성 직렬화와 fingerprint 재검증. 늦게 완료된 이전 설정의 pool을 닫고 현재 설정/pool snapshot을 반환 |
| `storage/database/tests.rs` | 문서/export 경로·inventory·비밀 없는 view와 합성 keychain 저장·이전 회귀. 설정 복구/경합 회귀는 해당 configuration/connection 모듈에도 포함 |
| `storage/database_contract_tests/tls_restart.rs` | 독립 테스트 프로세스에서 합성 PostgreSQL 인증서/hostname 검증, 실제 pool 경합·socket 종료·서버 강제 종료/재시작, managed store 오류와 commit 자료 보존 계약 |
| `notifications/discord.rs` | Discord Bot 알림 |
| `config/mod.rs` | `secure_config.json` + `.env` 로드 |

---

### 문서 생성과 연구 회귀

| 경로 | 책임 |
|---|---|
| `scripts/database-tls-contract.mjs` | 로컬 Docker socket의 실행별 PostgreSQL16 TLS/비TLS fixture·임시 CA를 생성하고 일반 계약 3개와 독립 TLS/복구 계약 2개를 실행. 생성물은 Git 제외 `.cache/database-tls-contract/<UUID>/`에만 두며 소유 라벨 확인 후 컨테이너/볼륨·개인키를 정리 |
| `scripts/render-todo.mjs` | `docs/todo.md`의 상태가 명시된 열린 최상위 항목을 `docs/todo.html`로 생성하고 원본 hash·상태별 색상·검색·필터를 제공한다. `npm run check:todo`는 원본과 생성물의 동기화를 검사 |
| `scripts/render-todo.test.mjs` | 열린 항목 보존·상태 분류·escaping·freshness와 실제 Chromium의 좁은 화면·키보드 필터·JavaScript 없는 읽기를 검증 |
| `scripts/verify-toss-openapi.mjs` | 공식 Toss OpenAPI의 예상 경로 목록을 검사하며 sector 목록·순위·상세·주식·ETF 경로도 포함. 이 목록의 확인을 앱 adapter 구현 또는 장기 분봉 자료 확보와 혼동하지 않음 |
| `tests/e2e/live-daily-history.spec.ts` | 기존 상태 polling의 pending→ready, 부족/실패/legacy/mismatch와 계좌 범위, 512항목 요약·3행 페이지·모바일 표시를 mock 상태로 검증. 주문·자동매매 시작/정지 요청 없이 UI 검사 |
| `tests/e2e/candle-pagination.spec.ts` | 페이지 중복·종료·무진전·상한·계좌 scope 변경·오류, 평가 200봉+이전 252봉 분리와 Web query cursor encoding을 mock provider/page 응답으로 검증 |
| `tests/e2e/lth-minute-timing.spec.ts` | mock LTH1분의 봉 종료/DST offset·공급/가격 제외/미관측 진단과 A/B 저장/재조회·봉 표기 차이, legacy 누락/미지원 버전·빈1분 평가불가/일봉 미노출·모바일 상한 표시를 검증. 실제 provider 시각/coverage 또는 주문 실행 검증과 구분 |
| `tests/e2e/strategy-rsi-policy.spec.ts` | 기존 custom14/30/70·누락/0, optional strict stored/raw·숫자 키보드/상한·scope 저장/재조회/실패·활성 잠금·지연 저장 초안/응답·모바일, 단기/연속 params와 A/B 요약/차이·과거 설정의 적용 미확인을 mock transport로 검증 |
| `docs/research/rsi-policy-20261022/{design.json,baseline_runner.rs,runner.rs}` | 고정978640c/v17 baseline 원본·주요 소스 byte 대사와 v18 실제 소유/단독 추세200·손절3·보유63 생산 비교를 실행. 비RSI1122 전체 backtest/signals 호환·RSI102 finalEquity/accounting 동일과100개 raw/report 변화를 구분하고30연속/30분할/360별도분기(390원장)를 보관. owned78개의 실제 경제 경로는 legacy와 같으며 부적합 신호 감소를 수익 개선으로 표현하지 않음. 추세/손절 단독 후보는 기본 채택 기각, hold63은 미발동·효능 미평가로 구분하고14/30/70·옵션0을 유지 |
| `docs/research/rsi-policy-20261022/independent-audit.json` | 별도 stock_analyst의 기록 원장·flat delta·정책 관측·원본/실행 hash 검증 산출물. 직접 대조한 기본78개 경제 경로와 생산 runner의 비RSI1122 해시 호환 증거를 구분하며 검사 수·오류/소스 고정·허용 오차는 이 파일과 작업 기록에서 확인. 독립 감사가 생산 분할/기본 전체를 새로 실행했다는 주장으로 확대하지 않음 |
| `docs/research/rsi-policy-20261022/analyze.py` | 압축 생산 결과와 summary.csv의 기록만으로 준비/무신호·미완료/잔여·stop/cap 원시 후보/부적합 후보, 체결 뒤 최장 보유 관측·hold63 전체 일치·비용별 부호 변화를 policy-observations.json에 집계. RSI 판단·누락 보간/달력을 재구현하지 않고 연속/독립 분기의 기계적 건수를 수익률이나 독립 표본으로 합성하지 않음 |
| `docs/research/rsi-policy-20261022/validate.py` | 생산 기록 fill·원본 OHLC로 cash/원가/비용·equity/MDD/실현잔여·노출/기말/실제 보유 경로를 대사하는 연구 도구. flat RSI 신호/회계 delta와 단독 옵션 비교·원본/실행 소스 hash를 보관하며 신호를 재구현하거나 raw 감소를 비용·수익 개선으로 추정하지 않음 |
| `tests/e2e/strategy-ma-cross-filters.spec.ts` | MA 선택4조건의 stored/raw 입력·scope 저장/재조회/실패·legacy custom·활성잠금/모바일, 일봉 전용 두 preview payload·이전 응답/준비 진단·A/B 보존을 mock transport로 확인하는 회귀 |
| `docs/research/ma-cross-filters-20261020/{design.json,baseline_runner.rs,runner.rs}` | 고정327739a/v16 baseline과 v17 기본0·단독 추세200/기울기0.1/ATR0.5/폭2의 Stage A36연속/36분할/432분기를 실행. ATR0.5/폭2 미선택성을 확인한 뒤 23:03:28 KST에 사후 고정한 ATR2/폭10 스트레스를 추가해 최종48연속/48분할/576별도분기(624원장)를 보관. 기본1224개 backtest/signals·unfiltered78원장 호환과 원래36대조군 hash를 대조하며 사후 탐색·완료 일봉/관측 교차·독립 분기 계좌를 구분 |
| `docs/research/ma-cross-filters-20261020/validate.py` | 생산 기록 fill·원본 OHLC의 cash/원가/비용·equity/MDD/노출·실현/잔여·기말청산·실제 보유 경로와 archived 골든크로스의 이전 완료 일봉 predicate를 대사. 최종624원장의 무손실 gzip2개·summary/aggregate/validation·source/산출물 hash를 보관하고 평문은 ignored .cache로 이동. 신호 재구현·provider 주문이나 독립 감사의 재실행 범위를 대신 주장하지 않음 |
| `tests/e2e/strategy-mean-reversion-policy.spec.ts` | legacy custom/기본 upperBand·strict stored/raw 입력·scoped 저장/재조회/실패 재시도·저장 중 새 초안, 두 preview params/A-B·stale 응답·활성 잠금·모바일을 mock transport로 검증. current-inclusive strict 밴드·기존 손절0·실제 소유·분봉 warmup/관측 단위 안내도 확인 |
| `docs/research/mean-reversion-policy-20261020/{design.json,baseline_runner.rs,runner.rs}` | 고정 474aa06 생산 v15 baseline과 현재 v16 실제 소유 기반 upperBand/중심선/추세200/보유63 단독 후보를 실행. Stage A30연속/30분할/360분기 뒤 사후 maxHold21을 추가한 최종36연속/36분할/432분기(468원장)를 구분한다. 비평균회귀1122회 전체 report/signals 호환·평균회귀102회 delta와 독립 정산을 보관. 일봉 연구를 live 기간·unseen OOS로 해석하지 않음 |
| `docs/research/mean-reversion-policy-20261020/validate.py` | 기록 fill·원본 OHLC 기반 cash/원가/비용·각봉 equity/MDD/노출·실현/잔여·기말 가상 청산·보유 gap/장중 손익의 독립 산술 대사. summary/aggregate/validation과 무손실 gzip2개·source/산출물 hash를 유지하고 평문 JSONL은 ignored .cache로 이동. 신호 재생성이나 후보 성과의 운용 채택을 맡지 않음 |
| `tests/e2e/strategy-deviation-policy.spec.ts` | legacy custom/기본 비활성·scoped 저장 재조회/실패 재시도·strict stored/raw 입력·두 preview/A-B·지연 응답/활성 잠금/모바일을 mock transport로 검증. 일반·가격조건·LTH 지연 저장 뒤 새 초안을 보존하는 회귀도 포함 |
| `docs/research/deviation-policy-20261020/{design.json,baseline_runner.rs,runner.rs}` | 고정 bca6601 생산 baseline과 현재 exact 소유 기반 level/하향 crossing·추세200/보유63/손절5 단독 후보를 실행. 비이격도 전체 report/signals 호환과 이격도 raw/accepted/회계 차이를 분리하고 연속 장부·대체 청크·독립 분기 계좌를 보관. 약10초 live와 일봉 연구를 같은 기간으로 해석하지 않음 |
| `docs/research/deviation-policy-20261020/validate.py` | 기록 fill·원본 OHLC로 현금/원가/수수료/슬리피지·각봉 equity/MDD/노출·실현/잔여·기말 가상 청산·보유 gap/장중 손익을 독립 대사. summary/aggregate/validation과 무손실 압축 JSONL·source/산출물 hash를 유지하며 신호를 재생성하거나 선택 후보의 수익 개선을 보장하지 않음 |
| `tests/e2e/strategy-momentum-entry.spec.ts` | legacy 비교 기간/임계값·기본 level, scoped 저장/재조회·실패 재시도, short/continuous params·A/B 모드/%p 차이, level 미사용 초안과 crossing 상한, stored/raw 오류·소수 키 입력·stale 응답·모바일/활성 잠금을 mock API로 검증 |
| `docs/research/momentum-feedback-20261020/{design.json,baseline_runner.rs,runner.rs}` | 고정 생산 baseline과 현재 exact 소유 기반 level/crossing/1%p를 실행. 비모멘텀 전체 report/signals 호환과 Momentum raw/차단 변화·실제 경제 경로 대조를 분리하고 연속 장부/대체 청크/별도 분기 계좌를 보관. 새 운용 기본 채택·provider 주문 부하/수익 개선을 raw 감소에서 추정하지 않음 |
| `docs/research/momentum-feedback-20261020/validate.py` | 기록 fill·원본 OHLC로 cash/원가/비용·각봉 equity/실현·잔여/노출·기말 가상 청산과 실제 보유 gap/장중 손익을 독립 대사. legacy/confirmedLevel의 기록 경제 경로와 동일 accepted 경로 비용 환급을 대조하고 summary/aggregate/validation·무손실 gzip과 source/산출물 hash를 유지하며 신호를 재생성하지 않음 |
| `tests/e2e/strategy-consecutive-entry-filters.spec.ts` | 기본 비활성·legacy 횟수·strict/raw 소수 입력과 저장 재조회/실패 재시도, short/continuous params·stale 응답·A/B 진단 보존, 별도 entry/sizing ATR·활성 legacy unknown·모바일/활성 입력 잠금을 mock transport로 검증 |
| `docs/research/consecutive-filters-20261018/{design.json,baseline_runner.rs,runner.rs}` | 사전 고정 단독 누적/ATR/MA 후보를 생산 replay로 실행하고 기본0의 직전 단기/장기 전체 report·signals 호환과 분할 무관 결과를 대조. 관측 단위 reason 두 템플릿만 정확히 치환하며 종료 suffix는 보존. 실제 연속 장부와 분기별 독립 초기화 계좌를 분리 보관하고 운영 계정·주문을 호출하지 않음 |
| `docs/research/consecutive-filters-20261018/validate.py` | 기록 fill·OHLC 기반 현금/원가/실현·잔여·기말 가상 청산/노출·후보 진단과 같은 실제 보유 경로의 비용 환급을 독립 대사. summary/aggregate/validation·무손실 압축 JSONL과 파일/source hash를 남기며 후보 신호를 재구현하거나 독립 OOS 성과를 주장하지 않음 |
| `tests/e2e/strategy-execution-model.spec.ts` | 연구 전용 체결/보호 가정 전달, 신호 공개·체결 단계·미체결/동시 도달 표시, A/B 엔진·가정 차이와 legacy 미확인, 좁은 폭·LTH preview 경계를 mock transport로 검증 |
| `tests/e2e/strategy-accounting.spec.ts` | 일반/LTH 공통 비용·FX/배당 입력 전달·잘못된 초안 실행 차단·이전 결과 무효화, 실현/잔여·종료 청산·투자 비중·비교·legacy 미확인과 좁은 화면을 mock transport로 검증 |
| `tests/e2e/strategy-continuous-replay.spec.ts` | 기존 8페이지를 넘는 수집과 bounded 청크 단일 전송, 엄격한 이전 자료 부족·부분 오류·scope 변경·취소 뒤 늦은 응답 차단, A/B의 실제 계산 모드/연속 경계·legacy 누락과 좁은 화면을 mock transport로 검증 |
| `docs/research/replay-accounting-20261018/{baseline_runner,runner}.rs` | `baseline_runner.rs`는 직전 1192673 생산 라이브러리의 동결 입력 report·신호 hash를 생성하고 `runner.rs`는 현재 생산 preview로 새 회계 metadata를 제외한 전체 호환을 대조한다. 실제 관측 가격의 회계·동일 요청 보유와 합성 FX/배당/수수료 가정을 분리한 결과를 저장. 별도 `.cache/` harness manifest의 재현 경로는 같은 디렉터리 report.md를 따르며 사용자 계정·주문 API와 분리 |
| `docs/research/replay-accounting-20261018/validate.py` | 원본 봉과 기록된 생산 fill로 현금·부분 원가·비용·FX/배당 권리·매봉 자산·관측 투자 비중·종료 청산·비용 환급을 독립 대사하고 소스/산출물 SHA-256을 validation.json에 기록. 전략 신호를 재구현하지 않음 |
| `docs/research/replay-execution-20261018/baseline_runner.rs` | 작업 시작 HEAD의 생산 preview로 동결 자료를 실행하여 신호·보고 hash와 summary를 baseline-hashes.json에 보관하는 오프라인 기준 생성기 |
| `docs/research/replay-execution-20261018/runner.rs` | 기존 동결 input/results를 실제 preview/회계 엔진에 전달해 작업 시작 HEAD baseline과 sameClose 신호·보고 hash를 비교하고 과거 연구 summary 차이는 별도 집계. QQQ 일부 전략·비용별 sameClose/nextOpen 및 같은 모델 단순보유 비교를 results.json에 저장하며 운영 계정·provider·주문 API를 사용하지 않음 |
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
| 6 | 잔고 갱신 + 이벤트 발행 | `watch::Receiver` — interval 변경 즉시 반영, 활성 broker가 KIS일 때 공통 holdings refresh로 조회·게시 검증 후 이벤트 발행 |

---

## 5. 데이터 흐름

### DB 설정 게시·연결·단절 복구

`database/configuration.rs`는 저장 시작 전에 runtime latch와 비밀 없는
`database_config.json.recovery` 표시를 세우고, blocking 작업에서 설정 파일과
OS keychain을 함께 저장한다. 쓰기 오류는 rename 이후에도 발생할 수 있으므로
이전 파일 내용·keychain을 모두 복원하고 검증한다. 성공은 설정과 pool 무효화를
같은 잠금 구간에서 게시한 뒤 표시와 latch를 해제한다. 취소·복원 불확실·표시 제거
실패는 연결/다음 저장을 차단하며, 디스크 표시가 남으면 앱 초기 로드도 실패한다.
`lib.rs`는 초기 로드와 활성 DB 연결 오류를 전파해 JSON으로 우회하지 않는다.

`database/connection.rs`는 별도 생성 mutex로 cold pool 생성을 직렬화한다.
네트워크 연결 뒤 현재 설정 fingerprint를 다시 검사하여 이전 대상의 늦은 pool은
닫고, 현재 설정/pool snapshot만 게시한다. `database.rs`의 상태 조회도 이
snapshot을 사용한다. 서버 단절은 managed store에 오류로 전달되고 로컬 JSON
정상본/백업을 대체하지 않는다. 서버 복구 뒤 기존 SQLx pool의 새 연결 취득으로
회복하며, 완료 여부가 불명확한 쓰기를 애플리케이션이 자동 재실행하지 않는다.

검증 실행기·테스트 책임은 위 표와 [DB TLS 작업 기록](tasks/database-tls-reconnect-20261018.md),
실행 환경·명령은 [Harness 지도](HARNESS_MAP.md)를 따른다. 임시 인증서·개인키·결과는
프로젝트 `.cache/`에서만 생성하고 Git에서 제외한다.

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

### Live 완료 일봉 갱신(52주·Consecutive/MA 교차 진입 확인)

```text
IPC/Web start 준비 완료 → manager generation + 실행 identity → 실행 플래그
    ↓ daemon: 시장 대기 전 DailyHistoryJobs::maintain
prepare_daily_history_targets: config/profile/scope/종목/시장 날짜 ticket
    ↓ 같은 종목 설정 공유 / 동시 2작업 / 작업당 8회 요청·512봉 유지
공통 완료 일봉 페이지 → 종료 사유·실제 완료 OHLC
    ↓ 완료 handle 수거 / 현재 ticket 재검증
manager/live_history.rs → opt-in apply_completed_daily_history (마지막 완료봉 포함)
    ├ 52주: 완료 고가/준비 버퍼
    ├ Consecutive: 별도 진입 OHLC context (가격 관측열·확정 보유 유지)
    └ MA 교차: 별도 진입 OHLC context (기존 관측 교차열·Sell 유지)
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

### Consecutive 선택 진입 필터와 관측 경계

```text
strategyMetadata.ts → consecutiveEntryControls.tsx: 0 비활성·숫자 범위·raw draft
    ├ scoped update_strategy → trading/configuration.rs: 검증·저장·보유 보존 게시
    └ short/continuous preview → driver.rs: params·일봉 필요 간격 검증
        ↓ 한 전략의 가격관측 방향 + 이전 완료 OHLC context
sequence/consecutive.rs → context.rs: 미보유 연속 상승 후보의 활성 필터 AND 확인
        ↓ raw 매수 → ReplaySession guard/risk·체결/차단 → 실제 보유 피드백
현재 일봉 OHLC → on_completed_candle → 다음 후보의 완료 context
        ↓ 선택 entryFilterPreparation (수량 산정 atrPreparation과 별도)
공통 연구 결과 / A-B: params·서버 후보 진단 보존
```

실시간의 약10초 가격 관측과 replay의 선택 봉 종가는 일수와 다르다.
ATR/장기 MA는 검증된 이전 완료 일봉을 별도로 사용하고 활성화된 비일봉
replay는 거부한다. live 일봉 재조회·실패·시장 날짜 변경은 그 context만
교체/무효화하며 기존 가격 관측열·보유 청산을 초기화하지 않는다.
누적 변화·ATR 하한은 이상, MA 비교는 엄격히 큼이며 미준비는 신규 진입만
보류한다. 후보 거부 수는 중복될 수 있고 통과는 실제 체결 수가 아니다.
모든 새 필터가0인 요청/저장값은 진단 생략을 중립 비활성으로 표시하며,
활성 필터의 이전 응답에 진단이 없을 때만 준비 미확인 경고를 표시한다.
상세 계약은 [IPC v13](ipc-commands.md), 사전 고정 후보·비용/노출 해석은
[연구 보고서](research/consecutive-filters-20261018/report.md)를 따른다.

### MA 교차의 가격 관측과 이전 완료 일봉4필터

```text
maCrossEntryControls.tsx → local4필드/raw/기울기-기존 long_period 관계 검증
    ├ scoped 저장 → configuration.rs strict 검증·기존 설정 게시
    └ short/continuous preview → driver.rs strict 검증·활성 필터 비일봉 거부
ma_cross.rs: 기존5/20 native 관측 MA·이전 MA → 골든크로스 후보
    ↓ ma_cross/context.rs: 검증된 이전 완료 일봉4조건 AND
    ↓ 통과 Buy → 기존 공통 guard/실제 소유 검사·risk·pending·체결
현재 일봉 판단/피드백 뒤 on_completed_candle → 다음 후보의 완료 context
    ↓ entryFilterPreparation (+ 선택 slopeRejections/bandwidthRejections)
maCrossEntryPreparation.tsx / 현재 연구·A/B: 단위·준비·후보/거부/통과 표시
```

기본 short5/long20은 입력 가격 관측 수이며 live 약10초 관측과 선택 replay
봉 종가는 기간이 다르다. 기존 교차·Sell·매 관측 prevMA 갱신은 유지하며
거절한 골든크로스는 소비하고 다음 관측에 되살리지 않는다. 후보는 기존
공통 guard의 실제 보유 검사 전이고 준비/필터 통과를 미보유나 체결로 세지
않는다. 잘못된 optional runtime 값은 custom 기존 기간·Sell을 보존하고
신규 Buy를 차단한다. 네 필터 모두 누락/0이면 새 진단을 생략한다.

새 추세·기울기·ATR·밴드폭은 모두 현재를 제외한 완료 일봉 OHLC만 쓴다.
추세는 후보 현재가>이전 N종가 평균이며 기울기는 이전 long_period 평균과
그 한 일봉 전 평균의 변화율 하한이다. ATR은15봉의 TR14 산술평균/직전
종가, 밴드폭은20종가의4σ/평균 비율이다. 준비 요구는 활성 조건의
N/long_period+1/15/20 중 최대이고 기울기 활성 long_period는1..511이다.
완료 context는 최대512개다. 충분자료의 ATR0/폭0은 MA에서 준비 완료 후
양수 임계값 미달이고 Consecutive의 기존 zeroAtr 미준비 정책과 구분한다.

live coordinator는 기존 scope·generation·시장 날짜 gate로 검증 일봉을
apply/invalidate한다. 재조회/실패로 기존 교차 관측열·Sell을 지우지 않는다.
historical 가격-only 초기화는 일봉 인증을 무효화하고 native 분봉 seed는
교차만 준비하며 별도 검증 일봉 context를 보존한다. 재구성은 기존 교차
가격을 재해석한 뒤 기존 context를 넘겨 진단 카운터를 재설정하고 실제
설정 게시의 generation 변경은 별도 일봉 재검증을 요구한다.

공통 completed_daily3함수는 순수 계산만 소유하고 각 전략의 lifecycle과
후보 정책을 합치지 않는다. 선택 slope/bandwidth 거부 필드는 MA에서만
직렬화하고 Consecutive는 생략한다. UI는 과거 누락을0으로 추정하지 않고
비활성/활성 진단 미확인을 구분하며 수량 산정 ATR과 분리한다. 계약은
[IPC v17](ipc-commands.md), 사전 설계와 실제 실행 상태·한계는
[연구 보고서](research/ma-cross-filters-20261020/report.md)를 따른다.

### RSI 실제 소유·선택 추세와 보유 보호

```text
rsiControls.tsx → shared decimal/유한수 helper + local3필드 타입/범위/raw 검증
    ↓ Page.tsx optional 원형 보존·저장/두 preview 차단
    ├ configuration.rs → strict 검증·기존 보유 보존 저장/게시
    └ driver.rs → strict 검증·동일 ReplaySession (engine v18)
classic.rs 공개 facade → classic/rsi.rs + params
    ↓ 기존 simple RSI/prevRSI 매 관측 갱신·이전 입력가격 추세
    ↓ flat30상향 Buy / held stop → cap → 원래70하향 Sell (실제 잔량 전량)
기존 guard/risk·소유 clamp·pending·체결 → exact q/native avg + kind=rsi metadata
    ↓ 기존 params/report → 현재/A-B 설정 요약·옵션/기존 엔진 차이 안내
```

기본14/30/70과 period 변화의 상승/하락 단순평균을 유지하며 Wilder 방식으로
바꾸지 않는다. 하락평균0은 횡보도100이다. Buy는 prev<=oversold/current>
oversold, 원래 Sell은 prev>=overbought/current<overbought이고 기존 Buy
분기의 우선순위를 유지한다. 매 관측 prevRSI를 갱신하므로 소유·필터·guard
거절 뒤 옛 교차를 재시도하지 않는다. 신호는 보유/원가를 만들지 않는다.

실제 소유 수량/native 평균가만 feedback으로 갱신하고 모든 held Sell은
실제 잔량을 요청한다. 설정 수량과 다른 ATR 수량/부분 체결을 고려한 명시적
계약이며 공통 clamp/guard/risk는 유지한다. 손절과 보유 cap은 조건이 지속되고
잔량이 있으면 재후보를 만들지만 원래 RSI 하락 교차에는 같은 latch를 추가하지
않는다. stop→cap→cross 우선이며 RSI/추세 미준비·손상 optional의 신규 진입
차단은 유효 보유 보호를 막지 않는다. RSI stop0은 비활성이다.

선택 추세는 current 제외 이전 N입력가격이다. 일봉/분봉 native RSI warmup은
기존대로 유지하며 live 최초 관측은 추세 seed만 비우고 실제 Nquote를 모은다.
약10초 live N관측과 선택 replay N봉은 기간이 다르고 완료 일봉 검증 context와
구분한다. readiness는 max(period+1,N+1), 추세 ring은512개까지다. 실제 양수
체결에 age0을 만들고 다음 보유 관측부터1로 누적하며 off/sameqty/partial/
warmup/reconfigure로 줄이지 않는다. 근거 없는 복원 보유는 unknown, 활성 cap은
다음 관측 보수적 매도 후보다. v1 kind=rsi origin/known metadata를 검증하고
다른 kind를 거부한다.

같은 scope 설정은 보유·원가·age/origin·live 추세를 보존하며 RSI period가
바뀔 때만 잔존 가격에서 이전 RSI를 재계산한다. reset은 전체 종목 상태를
지운다. 기존 observation8함수의 기계 동작만 재사용하며 RSI 계산/교차·보호
정책은 전략에 남긴다. 새 endpoint/DTO 없이 기존 params·report/A-B를 사용하고
종가 신호/다음 시가 갭·차단·부분 체결 때문에 손실/실보유시간 상한을 보장하지
않는다. 계약은 [IPC v18](ipc-commands.md), 실행 상태와 비용/표본 한계는
[연구 보고서](research/rsi-policy-20261022/report.md)를 따른다.

### Momentum 소유 피드백과 선택 진입 교차

```text
momentumEntryControls.tsx → shared/lib::parseDecimalDraft / isFiniteNumberInRange
    ↓ local enum/범위/crossing 간격≤threshold 검증 + raw 소수 초안 유지
Page.tsx: 두 새 params의 원형 보존
    ├ scoped update_strategy → configuration.rs: strict 검증·보유 보존 게시
    └ short/continuous preview → 공통 driver: strict 검증·기존 ReplaySession
classic/momentum.rs: 가격 관측/이전m/armed/retry_candidate ≠ 실제 owned_quantity
    ↓ flat level/crossing 매수 후보 또는 보유 negative level 매도
공통 guard/risk·pending·체결 → exact 소유 수량 피드백 → 잔여 보유 판단
    ↓ 기존 report + params → 현재 연구 결과/A-B의 mode·%p·조건 차이
```

기존 level은 `m>=T`/`m<=−T`를 유지한다. crossing은 이전 `m<T`에서
현재 `m>=T`가 될 때 후보를 만들며, `m<T`에서 취소하고 `m<=T−간격`에서
재무장한다. T5·간격1%p에서 `4<m<5` 관측은 취소만 하며 새 무장을 만들지 않는다.
미체결 후보는 조건 유지 중 재시도할 수 있지만 동일0 수량/nextOpen 대기를
체결로 세지 않는다. 실제 양수 수량으로 후보를 소모하고 부분 매도 뒤 잔량도
음의 level 청산을 계속 평가한다. provider 매도 수량 제한은 기존 공통 경계다.
동일 params는 후보를 보존하고 params 변경은 보유/유효 가격을 남긴 채
이전 모멘텀·무장을 재seed해 옛 후보를 해제한다. live 약10초 quote와 선택
replay 봉 종가의 관측 단위는 같지 않으며 level의 유효 간격은 미사용이다.
새 IPC/응답 DTO 없이 기존 params 저장·미리보기/A-B 경로를 사용한다.
상세 계약은 [IPC v14](ipc-commands.md), 실제 경로/후보 비교와 한계는
[연구 보고서](research/momentum-feedback-20261020/report.md)를 따른다.

### Deviation 소유 피드백·진입 조건·보유 보호

```text
deviationControls.tsx → shared/lib::parseDecimalDraft / isFiniteNumberInRange
    ↓ local 모드·필드 범위/기본값·raw 초안 검증
Page.tsx: 네 optional params 원형 보존
    ├ scoped 저장 → configuration.rs: strict 검증·보유 보존 게시
    │   └ 성공 → 같은 scope epoch + 제출 초안 identity일 때만 편집 map 정리
    └ short/continuous preview → 공통 driver: strict 검증·ReplaySession
classic/deviation.rs + params/state: 관측/교차 후보 ≠ 실제 owned_quantity/native avg/보유 나이
    ↓ flat level/하향 crossing + 선택 직전 N관측 추세 진입
    ↓ held 손절 → 최대 보유 관측 → 고이격 level 청산
공통 guard/risk·pending·owner clamp → exact 수량/native avg + 보호 metadata 피드백
    ↓ 기존 report/params → 현재 결과·A/B의 진입/추세/보유 관측/손절 요약
```

기존 이격도 MA는 현재 가격을 포함하고 inclusive 매수/매도 임계값을 유지한다.
하향 crossing은 이전d>buyT·현재d<=buyT에서만 새 후보를 만들며,
조건이 유지된 동일0/pending/거절은 실제 체결이 아니므로 재시도할 수 있다.
상회하면 취소하고 실제 양수 체결로 후보를 소모한다. strict 준비 자료는
관측만 seed하며 가상 체결·진입 후보·보유 나이를 만들지 않는다.

선택 장기 추세는 현재 제외 직전 N가격이다. replay는 엄격한 이전 봉으로
준비하지만 live 최초 관측은 별도 추세 버퍼를 비우고 실제 N관측을 모은다.
일봉/1분봉 warmup을 약10초 live cadence로 인증하지 않으며 검증된 완료 일봉
context와도 구분한다. 기존 이격도 MA warmup은 유지한다.

보유 나이는 첫 실제 양수 체결 뒤 평가부터 옵션 비활성 중에도 누적한다.
같은 수량·부분 매도·warmup·설정 변경으로 초기화하지 않으며 exact owner
metadata로 복원한다. 근거 없이 복원된 양수 보유는 나이 불명이고 최대 보유가
활성이면 다음 관측에 보수적 매도 후보를 만든다. 손절/관측 한도는 실제 잔량
전량을 요청하지만 기존 고이격 level은 설정 수량을 유지하고 공통 제한을 따른다.
손절은 native 평균가 대비 현재 관측 조건이며 nextOpen 갭·비용·FX·미체결을
포함하는 손실 상한이나 체결 보장이 아니다. 새 IPC/응답 DTO는 없다.
상세 계약은 [IPC v15](ipc-commands.md), 고정 후보와 비용·표본 한계는
[연구 보고서](research/deviation-policy-20261020/report.md)를 따른다.

### MeanReversion 실제 소유·선택 청산·관측 보호

```text
meanReversionControls.tsx → shared decimal/유한수 helper + local 모드/정수 범위/raw 검증
    ↓ Page.tsx: optional3개 타입 보존·저장/두 preview 실행 차단
    ├ scoped 저장 → configuration.rs: strict 검증·실제 보유 보존 게시
    └ short/continuous preview → driver.rs: strict 검증·단일 ReplaySession
mean_trend.rs 공개 facade → mean_reversion.rs + params/state
    ↓ 미보유: strict price<현재 포함 lower + 선택 price>이전 N평균
    ↓ 실제 보유: native 손절 → 보유 관측 한도 → strict price>상단/중심선
기존 guard/risk·pending·owner clamp → exact 수량/native avg + v1 보호 metadata
    ↓ 기존 report/params → 현재 결과·A/B의 청산/추세/보유 관측/기존 손절 요약
```

밴드는 현재 관측을 포함한 최근 period의 평균·모집단 분산을 사용한다.
기본 period20/std_dev2/stop5와 strict 매매 경계를 유지하며 선택 중심선도
같은 현재 포함 평균을 엄격히 상회해야 한다. 기존 손절0은 비활성이 아니라
native 원가 이하 조건이고 설정 수량을 요청한다. 최대 보유만 실제 잔량
전량을 요청하며 밴드 청산은 설정 수량이다. 관측 조건은 nextOpen 갭·비용·FX나
pending을 포함한 손실 상한 또는 체결 보장이 아니다.

보유는 신호로 바꾸지 않고 exact 실제 수량/평균가 피드백으로 갱신한다.
진입 origin 뒤 실제 양수 체결은 나이0을 만들며 다음 보유 관측부터 증가한다.
옵션 off 중에도 누적하고 같은 수량·부분 매도·warmup·설정 변경으로 지우지
않는다. origin/유효 metadata 없는 복원 보유는 나이 불명이며 활성 상한은
다음 관측에 보수적 매도 후보를 만든다. 동일 params는 후보를 보존하고
params 변경은 보유·나이·live 추세를 남겨 가격열을 새 period로 제한하며
미체결 진입 후보를 해제한다. reset은 기존 가격/추세 버퍼를 유지하며 보유
수량·평균가·나이·진입 후보를 정리한다.

추세는 현재를 제외한 이전 N관측이다. live 첫 관측은 추세 seed만 비우고
실제 N개 quote를 모아 다음 관측부터 확인한다. 일봉 historical seed와
실시간 약10초 cadence를 섞지 않는다. 기존 분봉 initializer는 no-op이므로
1분 preview는 사전 분봉을 밴드·추세 준비로 인증하지 않고 평가 관측부터
준비한다. readiness는 max(bounded period, N+1)를 요구하며 N512일 때
추세 버퍼512개와 누적513관측 준비를 구분한다. period 기본20·N0은
20관측 준비다. 일봉63봉·live63관측·분봉63관측은 같은 기간이 아니다.

Deviation/MeanReversion의 공통 observation helper는 정수·나이·codec·추세
기계 동작만 재사용한다. 전략 kind·손절0 의미·밴드/후보·reset·분봉 warmup은
각 전략이 소유하며 서로 다른 checkpoint는 복원하지 않는다. 새 IPC/응답 DTO
없이 기존 params와 준비 수치/연속 ledger를 사용한다. 상세 계약은
[IPC v16](ipc-commands.md), 사전 후보·원장·표본 한계는
[연구 보고서](research/mean-reversion-policy-20261020/report.md)를 따른다.

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
경계에서 scope/client 교체·tracker 초기화·전략 설정을 게시한다. 일반·수동 holdings는
아래 공통 게시 경계로 최초 요청의 identity와 포지션 버전을 재검증한다. 체결 조회는
전략 epoch보다 실제 계좌·인증의 연속성을 확인하므로 같은 계좌의 실제 체결을
재시작만으로 버리지 않으며, 다른 계좌의 지연 응답은 적용을 보류한다.
KIS pending은 저장 TR-ID로 확인한 실전/모의 출처가 현재 client와 같아야 한다.
출처가 없거나 알려지지 않았거나 모드가 다르면 조회 후보에서 제외하고 직접
체결과 체결 조회의 종료 상태 반영도 보류하여 포지션·예산을 보존하고 체결 동기화
health에 남긴다.

### 잔고 조회와 tracker 게시

```text
IPC 일반 잔고 / lib.rs 갱신 / IPC·Web 시작 복원 / 수동 주문 준비
    ↓ OrderManager::begin_holdings_query[_for_snapshot]
scope·epoch·client/profile·설정 lease + 시장별 최신 조회/변경 UUID 캡처
    ↓ 잠금 밖 provider await → 수량·가격 파싱
OrderManager::publish_holdings: 현재 identity·lease·조회/변경 버전 재검증
    ↓ KR/US 동시 응답은 양쪽 검증 완료 후 함께 교체 (실제 빈 목록/0수량 포함)
tracker 게시 → 시작 경로의 전략 복원 또는 background 성공 이벤트
```

조회 중 계좌 A→B→A, 정지/재시작, 인증·투자 모드 변경, 설정 저장,
더 최신 조회 또는 체결·포지션 교체가 발생하면 이전 응답을 버린다. 시세만
바뀌는 것은 보유 변경으로 보지 않는다. 게시용 요청과 별도인 ReadOnly gate는
IPC broker holdings와 Web 잔고 응답의 identity를 검증하며 tracker나 최신 게시
조회 번호를 바꾸지 않는다. 계좌 전체 tracker는 증권사의 관측값으로 유지하며
확인된 체결 delta를 다시 더하지 않는다. 소유 전략의 수량과 원가는 별도 원장으로
관리하고 체결 뒤에는 전체 잔고를 재조회한다.

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
strategyPreviewPanel.tsx → preview_strategy → strategy_preview/generic.rs → driver.rs
    ↓ historyCandles 초기화 / 평가 candles 별도 검증
ReplaySession::begin_bar — 이전 종가 신호의 시가 체결·연구 보호 청산
    ↓ sync_position — 체결된 수량·평균가 피드백
Strategy::on_trading_day_start(symbol, open) — 시가만 공개
    ↓ Strategy::on_daily_close_tick(symbol, completed OHLC, volume) — 종가 평가
ReplaySession::process_close — 종가 체결 또는 nextOpen 보류·mark/equity
    ↓ sync_position — 미체결/차단 뒤에도 실제 보유 snapshot 피드백
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
다음 평가일에 사용하는 강한 종가도 선택한 sameClose 또는 다음 관측 봉 nextOpen 가정을 따른다.
nextOpen 주문 수량은 실행 시가와 이전 완료봉 ATR로 검증하고, 마지막 신호는 미체결로 남긴다.
연구 보호 청산은 전략 원시 신호와 별도이며, 원가 기준 손절·익절의 시가 갭 또는
선택한 stopFirst 봉 내부 근사를 사용한다. OHLC만으로 실제 체결 시각·순서를 검증하지 않는다.
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
    ↓ leveraged_trend_hold/preview.rs의 Open/Signal/Close driver
ReplaySession begin_bar → 포지션 피드백 → timed 전략 신호 → process_close/mark
    ↓ 실제 잔여 수량·평균 체결가 snapshot 피드백
replay.assessment → strategyResearchPanel.tsx
    ↓ dailyDiagnostic 또는 notEvaluable / 과거 assessment 누락
성과 표시·A/B 저장 및 비교 차단
```

일봉 맥락은 `dailyContextBars`, 장중 관측은 `intradayBars`로 구분하고
LTH `warmupCount`는 0이다. 유효한 분봉 표본은 `sampleOnly`이며 기간 전체
성과 검증을 뜻하지 않는다. 현재 시각·세션은 `unverified`, `lookAheadSafe`는
false다. SOXQ 2026-07-06~10-02의 실제 분봉과 거래소 시간대/DST·세션 검증은
P1-05의 미완료 항목으로 남아 있다.

P2-14의 직전 코드 기본 모드 호환과 QQQ 체결 모델 비교는
[연구 보고서](research/replay-execution-20261018/report.md), 구현·검증 범위는
[작업 기록](tasks/replay-execution-model-20261018.md)을 따른다. 이 비교를
장중 경로·실제 체결 또는 장기 연속 운용 검증으로 확대하지 않는다.

### LTH 분봉의 별도 시각·관측 슬롯 진단

```text
preview_leveraged_trend_hold_for_profile: provider I/O 전 referenceAsOf 고정
    ↓ 기존 profile/account·1m count와 조회/lexical tail/가격 필터 유지
실행용 timed 라벨 + 공급/선택 수 → lth_preview_assessment (1m만)
    ↓ minute_time::parse_minute_instant (generic 정규화와 구문만 공유)
minute_timing::diagnose_minute_timing → assessment.minuteTiming (선택)
    ↓ 기존 ReplaySession 신호·정산·입력 hash와 독립
strategyResearchPanel → LthMinuteTimingPanel 현재/A-B·legacy 누락/버전 차이
```

진단은 실행에 준비된 label만 최대512개 해석하며 실제 replay의 순서나
관측을 바꾸지 않는다. 전용 preview의 기존 선택 상한200과 공급/선택/가격
제외/실행/진단 처리 수를 구분한다. RFC3339는 offset의 실제 순간을 NY 또는
Seoul로 변환하고 compact12/14는 기존 KST 가정이다. generic wrapper의
nanosecond UTC 문자열·정렬/중복 계약은 유지하며 LTH의 fixed KST 전략
시계와 live 동작은 교체하지 않는다.

생산 Toss1분은 barClose로 label−60초 시작·label 종가 공개 가능 시각을
계산한다. 명시 barStart fixture는 label/+60초이며 unknown은 시각 해석만
하고 세션/공개 가능 시각/슬롯 판정을 생략한다. 이 시각은 표기 해석이고
실제 발행/수신 시각 검증이 아니다. asOf보다 뒤인 종가는 미완료 후보 수로
표시하며 진단 때문에 관측을 제거하지 않는다.

일반 평일 정규 시간창의 분 경계 고유 관측만 날짜별 슬롯으로 계산한다.
공백 합계는 관측된 같은 날짜 내부만 포함하며 앞/뒤 시간창·관측 없는 날짜·
야간·주말을 실제 누락 자료로 합산하지 않는다. 휴장/조기마감·거래정지·
무거래봉 생략 원인은 판단하지 않고 calendar/providerCoverage/
requestedPeriodCoverage/actualPublicationVerified는 false다. 입력512·
라벨64byte·날짜8개·예시6개로 제한하고 긴 기간의 minute 배열을 만들지 않는다.

UI는 현재 및 저장된 A/B에서 별도 진단 버전·봉 표기·시간대·legacy 세션
정책/검증 가정 차이를 알린다. 누락된 과거 응답이나 지원하지 않는 버전을
정상 진단으로 추정하지 않는다. 기존 sampleOnly/notEvaluable·미검증
시각/세션·lookAheadSafe=false를 유지하며 실제3개월 분봉 검증과
P1-05/P2-12는 별도 과제다. 범위·검증 기록은
[작업 기록](tasks/lth-minute-timing-20261022.md)을 따른다.

### 연속 replay 공급과 상태 경계

```text
strategyPreviewPanel.tsx → 연속 일봉 날짜 범위 선택
    ↓ collectContinuousCandles: 공통 완료 페이지/동일 scope·asOf, 이전252봉+평가 구간
continuousReplayPanel.tsx: 준비/보류·요청/실제 범위·수집 한계 표시
    ↓ usePreviewStrategyContinuous → previewStrategyContinuous → desktop IPC / Web REST
preview_strategy_continuous → continuous.rs: 모든 청크/이전 history·현금흐름 검증
    ↓ driver.rs: strategy·warmup·ReplaySession 한 번 생성
청크 순차 공급: begin_bar → 전략 포지션 피드백 → 종가 평가/process_close → 피드백
    ↓ 한 번 finish → 기존 preview + continuity 수량/pending ledger
연구 UI/A-B: 실제 모드·구간 경계 보존, 분기 리셋·독립 학습/검증과 구분
```

기존 단기 API의 500봉 제한과 초기화 정책은 유지한다. 새 연속 요청은 최대16청크,
각500봉·평가4096봉·엄격한 이전 history512봉을 지원하며 서버의 장기 세션 저장이나
append IPC를 만들지 않는다. 청크마다 전략·현금·포지션·쿨다운·일일/연속 손실·
대기 신호·ATR을 다시 만들지 않는다. 경제 입력 hash는 계산 모드를 구분하되 청크
분할 형태에는 의존하지 않는다. 시간 역전을 전역 정렬로 감추거나 누락 봉을
합성하지 않는다. ReplaySession의 이전 rounded-equity 최고점은 증분으로 보존한다.

UI의 연속 일봉 수집은 최대48페이지·4608봉을 유지하고 엄격히 이전252봉과 요청
범위 경계를 확보한 후500봉씩 묶는다. 시작/종료 차이 및 관측 간격의 보수적
7달력일 상한을 넘으면 계산을 보류하며 실제 거래소 휴장/자료 coverage 검증과
구분한다. 실제 수집 기간은 요청한3년과 함께 표시하고 coverageVerified=false를
유지한다. 공급 경계는 분기별 리셋이 아니며 두 결과를 더해 연속 성과로 만들지 않는다.

사전 고정한 연속/별도 분기 초기화 실행과 재현 manifest는
[연구 보고서](research/continuous-replay-20261018/report.md)에 기록한다.
연구 폴더의12파일은 설계·baseline/현재 runner와 hash·run summary·두 압축
전체 원장·집계/CSV·검증기/검증 metadata·보고서로 구성한다.
`continuous-results.jsonl.gz`와 `reset-results.jsonl.gz`는 손실 없는 전체 기록이고,
생성 평문은 `.cache/continuous-research-output/`에 두어 인벤토리/커밋에서 제외한다.
검증기는 두 기록 형식을 모두 읽으며 전략 신호를 재구현하지 않는다. 구현·검증의
범위와 잔여 한계는 [작업 기록](tasks/continuous-replay-20261018.md)을 따른다.

### Replay 회계 귀속과 비교

`simulation/accounting.rs`는 비용 출처와 방향별/최소 수수료, 최대 64개 환율·배당
시나리오를 검증한다. generic/LTH facade는 정규화한 평가 봉과 시나리오 시각을
대조한 뒤 실행하며 사전자료에는 현금흐름을 적용하지 않는다. 해당 봉의 환율을
먼저 적용하고 이전 봉부터 보유한 수량에 배당을 귀속한 뒤 대기 시가 주문을
처리한다. 배당 조정 가격과 현금 배당의 명시적 중복 조합은 거부한다.

원가 장부는 native 평균 체결가와 역사적 KRW 원가를 따로 유지한다. 각 거래·배당은
당시 환율로 KRW 현금에 반영하며 유휴 현금은 USD 자산으로 재평가하지 않는다.
`session/accounting.rs`가 실현 순손익 + 잔여 순평가 손익 + 세후 배당을 총자산
증감과 대사하고 가격/환율 귀속·남은 매수 비용을 표시한다. 종료 청산은 마지막
종가에서 추가 매도 비용만 차감한 projection이며 거래 수·리스크 상태를 바꾸지
않는다. 평균/최대 투자 비중은 관측 봉말 평가금액 기준이며 보유 봉 비율과 다르다.
최소수수료가 있는 일반 이익 매도는 동일 매도 projection으로 요청/보유 수량과
비례 매수 비용·현재 환율의 비용을 검사하며 손절·native 손실 매도는 보존한다.

`comparison.rs`는 같은 요청 수량의 단순 보유와 실제 수량·노출 차이를 공개하고,
실제 체결 경로의 발생 시점 KRW 비용 환급을 별도 산술 참조로 제공한다. 이를
독립 무비용 전략 성과로 표시하지 않는다. 공통 `strategyAccountingPanel.tsx`가
이 결과와 가정을 일반/LTH 연구 UI에 표시하며 과거 응답의 누락 정보는 미확인으로
유지한다. 재현 입력·생산 실행기는 `docs/research/replay-accounting-20261018/`에 두며
[연구 보고서](research/replay-accounting-20261018/report.md)에 직전/현재 harness
manifest와 실행 순서를 기록한다. `validate.py`는 생산 fill을 독립 대사하고
소스/산출물 hash를 보관한다. 구현·검증 범위는
[작업 기록](tasks/replay-accounting-20261018.md)을 따른다.

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

### 확인된 체결과 전략 소유권 피드백

```text
submission.rs / submission/owned.rs → exact 소유 key + 원장 세대 + 신호 시점 보호 metadata
    ↓ provider 접수 → PendingOrder 영속화
fills/reconciliation.rs → 고정 provider의 주문번호별 누적 체결·종료 조회
    ↓ fills/ownership.rs: key·세대·누적 수량/금액과 원가·손익 영수증 검증
fills.rs: 적용 intent → position_ownership.json → 예산·risk·stats·TradeStore
    ↓ 계좌 전체 tracker는 조회 무효화·재조회 표시 (체결 delta 재가산 없음)
feedback.rs: manager → OrderManager → 해당 소유 전략의 수량·pending·metadata
    ↓ holdings/fetch.rs: KIS/Toss 전체 잔고 재조회 → 공통 게시 gate
```

소유 원장은 profile·투자 모드·broker/account·strategy·market·symbol로 나누며
주문별 누적 수량·금액과 원가·손익 영수증으로 부분 체결과 재시도를 처리한다.
계좌 총보유나 다른 전략·수동 주문의 수량을 새 전략에 배정하지 않는다. 시작
경로도 전체 잔고 broadcast 대신 현재 identity의 소유 원장을 복원한다.

live 신호는 관측을 진행하되 확인된 소유 수량을 미리 바꾸지 않으며 pending과
미제출 신호 정리를 별도 훅으로 처리한다. LTH의 high-water·보유 관측 수는
평균가로 다시 만들지 않고 보호 metadata로 checkpoint/복원한다. 소유 원장이나
영수증의 유실·세대 불일치·저장 실패는 persistence block으로 전달한다. 계좌
전체 관측과 소유 원장은 역할이 다르므로 증권사의 체결 반영 지연과 실제 운용
검증까지 이 구조만으로 완료했다고 판단하지 않는다.

일반 잔고 갱신·시작·수동 주문은 `holdings/parse.rs`의 동일 검증을 사용한다.
오염된 수량·가격이나 Toss 응답 범위/통화·중복 종목을 정상 0수량으로 바꾸지
않으며, 정수 주식 경로에서 소수 잔량을 1주로 올리지 않는다. 리스크 복원도
KIS 체결/주문 투자 모드를 확인한 후보를 저장한 뒤 공개한다. 상세 구현·검증
범위는 [전략 체결 소유권 작업 기록](tasks/strategy-fill-ownership-20261018.md)에 남긴다.

provider 누적 체결금액은 `fills/collector/notional.rs`에서 정수 원·cents로
변환한다. Toss 명시 `filledAmount`를 우선하고 누락에만 실제 평균가×수량을
총액으로 계산한 뒤 한 번 반올림한다. 명시 0이나 잘못된 통화를 누락과 섞지
않는다. 소액 검증 즉시 체결은 원 주문·종목·방향·통화·1주와 명시 체결대금을
추가 확인하며 미확정 자료는 정상 체결로 처리하지 않는다.

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
환율 → setQueryData() / 잔고 → invalidateQueries() (이벤트 payload는 사용하지 않음)
    ↓ 잔고는 현재 profile/broker/account/모드 query key로 다시 조회
accountHooks.ts — 응답 전후 현재 scope 검사, 이전 계좌 data 이어받기 금지
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
