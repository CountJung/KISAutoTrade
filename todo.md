# KISAutoTrade — Todo

> 자동매매 손실을 줄이고, Windows/macOS 양쪽에서 안정적으로 운영하기 위한 개선 백로그입니다.  
> 완료 이력은 `git log --oneline`과 PR/커밋 메시지에 맡기고, 이 문서는 앞으로 할 일을 우선순위 중심으로 유지합니다.

---

## P0 — 반복 매매 손실 방지

- [x] 공통 `TradeGuard` 계층 추가
  - 모든 전략 신호가 주문으로 넘어가기 전에 `symbol + side` 단위 쿨다운을 검사한다.
  - 기본값: 동일 종목 재매수 30분, 매도 후 재진입 60분, 손절 후 재진입 당일 금지.
  - 구현 위치: `src-tauri/src/trading/guard.rs` + `OrderManager::submit_signal()`.
- [x] 최소 기대수익 필터 추가
  - 익절 조건은 수수료·세금·슬리피지 추정치를 뺀 순기대수익이 양수일 때만 주문한다.
  - 국내 수수료 추정은 기존 `calculate_domestic_fee()`와 연결하고, 해외는 USD 수수료/환율 추정값을 별도 설정으로 둔다.
  - 구현: 매도 신호가 손절이 아닌 익절 성격이고 실제 보유 평균가 대비 이익 구간일 때, 국내/해외 추정 비용과 슬리피지를 차감해 순익 0 이하이면 차단한다.
- [x] 횡보장/휩소 필터 추가
  - MA 교차, RSI, 이격도처럼 등락 반복 구간에 취약한 전략은 ATR 또는 밴드폭 기준으로 신호를 차단한다.
  - 최근 N틱 내 반대 신호가 반복되면 해당 종목을 `cooldown_until` 상태로 전환한다.
  - 구현: 공통 `TradeGuard`가 최근 반대 방향 신호 반복을 감지해 종목 단위 쿨다운으로 전환한다.
  - 남은 작업: 전략별 ATR/밴드폭 기반 횡보장 필터는 별도 고도화가 필요하다.
- [x] 전략 상태와 실제 보유 포지션 동기화
  - 현재 일부 전략은 내부 `in_position`만 보고 신호를 낸다.
  - 자동매매 시작 시 `PositionTracker`/KIS 잔고를 읽어 전략별 포지션 상태를 복원한다.
  - 해외 보유종목은 `get_overseas_balance()` 기반으로 별도 복원한다.
  - 구현: `Strategy::sync_position()` 훅과 자동매매 시작 전 국내/해외 잔고 동기화를 추가했다. `PriceConditionStrategy`와 `LeveragedTrendHoldStrategy`가 실제 보유 포지션을 복원한다.
- [x] 주문 체결 확인을 실제 체결/미체결 API 기반으로 전환
  - 현재 폴링 루프는 다음 틱에서 시장가 체결을 가정해 `confirm_fill_by_symbol()`을 호출한다.
  - 국내/해외 모두 주문번호 기반 체결 조회로 확인하고, 미체결/부분체결/거부 상태를 분리한다.
  - 구현: 국내 주문은 `get_today_executed_orders()`에서 주문번호를 찾아 실제 누적 체결수량/평균가로 먼저 확인한다.
  - 구현: 해외 주문은 `get_today_overseas_executed_orders()`에서 주문번호를 찾아 실제 누적 체결수량/평균가(USD cents)로 먼저 확인한다.
  - 구현: `PendingOrder`가 누적 체결량을 보존해 미체결/부분체결/완전체결을 분리하고, Dashboard 미체결 주문 패널에서 부분체결과 잔여 수량을 표시한다.
  - 구현: KIS 주문 접수 거부는 pending과 분리해 `OrderStatus::Failed` 주문 이력으로 저장한다.
  - 비고: `confirm_fill_by_symbol()`은 KIS 조회 실패/반영 지연 시 보완 fallback으로만 유지한다.

## P0 — 신규 전략: LeveragedTrendHoldStrategy

- [x] 기초/레버리지 종목을 고정하지 않고 선택하는 전략 추가
  - 예: 기초 `SOXX`/`SMH`, 정방향 레버리지 `SOXL`, 역방향 레버리지 `SOXS`.
  - Strategy UI에서 선택한 종목을 정방향 레버리지 항목으로 추가하고, 선택적으로 역방향 레버리지 항목도 설정한다.
  - 각 레버리지 row에 기초 항목을 연결한다.
  - 구현: UI에서 롱 레버리지 ETF, 선택 숏 ETF, 기초/유사 지수 ETF를 하나의 세트로 편집한다. TECL처럼 직접 기초지수가 애매한 경우 `VGT` 같은 유사 기초 ETF를 `proxy` 역할로 저장한다.
  - 구현: 레버리지 전략 섹션 안에 전용 ETF 검색기와 새 세트 구성 슬롯을 두어 기초지수, 롱 ETF, 선택 숏 ETF를 먼저 채운 뒤 한 번에 세트로 등록한다.
  - 구현: 기초/유사 기초 ETF가 없는 세트는 저장을 막고, 숏 ETF가 비어 있으면 하락 진입 없이 롱 진입/청산만 동작한다고 표시한다.
  - 기초 항목 중 하나라도 조건을 통과하면 추세 방향에 맞는 레버리지 종목을 매수한다.
- [x] 진입 조건 구현
  - 기초 항목 현재가 > EMA20, EMA20 > EMA60, RSI 55 이상, ADX 20 이상, 최근 3개 캔들 중 2개 이상 양봉.
  - 역방향은 기초 항목 현재가 < EMA20, EMA20 < EMA60, RSI 45 이하, ADX 20 이상, 최근 3개 캔들 중 2개 이상 음봉.
  - 장 시작 후 15~30분 진입 구간만 허용.
- [x] 청산/거래 금지 조건 구현
  - 레버리지 종목이 진입 후 고점 대비 -1.5% 하락하면 청산.
  - 기초 항목 EMA20 이탈, RSI 50 미만, 장 마감 20분 전 청산.
  - 역방향은 기초 항목 EMA20 상향 회복 또는 RSI 50 초과 시 청산.
  - ADX 18 미만, RSI 45~55, 장 시작 직후, 지정 블랙아웃 시간대, 전일 대비 갭 과대 시 진입 금지.
  - 주요 지표 발표 전후는 외부 캘린더 연동 전까지 `blackout_windows` 수동 설정으로 차단한다.

## P1 — 해외주식 모의투자 안정화

- [x] 해외주식 주문체결내역 IPC 추가
  - KIS 공식 샘플 기준 endpoint: `/uapi/overseas-stock/v1/trading/inquire-ccnl`.
  - TR-ID: 실전 `TTTS3035R`, 모의 `VTTS3035R`.
  - 모의투자 조회 제한: `PDNO=""`, `SLL_BUY_DVSN="00"`, `CCLD_NCCS_DVSN="00"`, `OVRS_EXCG_CD=""`, `SORT_SQN` 기본값 사용.
  - 구현: `get_today_overseas_executed`, `get_overseas_executed_by_range` IPC와 `KisRestClient::get_overseas_executed_orders_range()`를 추가했다.
- [x] 해외 모의투자 주문 사전 검증 추가
  - 미국 주문 TR-ID는 실전/모의 모두 존재한다: 매수 `TTTT1002U/VTTT1002U`, 매도 `TTTT1006U/VTTT1006U`.
  - 단, 모의투자 주문구분은 매수/매도 모두 `ORD_DVSN="00"` 지정가만 안전하게 지원한다.
  - AMEX 또는 일부 ETF에서 `"해당업무가 제공되지 않습니다"`가 발생할 수 있으므로 UI와 자동매매 로그에 제한 사유를 명확히 남긴다.
  - 구현: `KisRestClient::place_overseas_order()` 앞단에서 모의 AMEX/확인된 ETF 매도 제한을 사전 차단하고, Trading UI에서도 모의투자 제한 경고와 버튼 차단을 표시한다.
- [x] 해외 자동매매 포지션 추적 분리
  - 국내 `PositionTracker`에 해외 체결을 섞지 않는다.
  - USD 단가, 환율, 거래소 코드, 소수 가격 단위를 보존하는 `OverseasPositionTracker`를 검토한다.
  - 구현: `OverseasPositionTracker`를 추가해 해외 체결/잔고를 USD cents와 거래소 코드로 별도 추적하고, 국내 손익/수수료 통계에는 혼입하지 않도록 분리했다.
- [x] 해외 수수료/환율 반영
  - 해외 체결 손익은 USD 기준과 KRW 환산 기준을 함께 저장한다.
  - 환율은 `get_exchange_rate` 캐시를 사용하고, 체결 시점 환율을 기록한다.
  - 구현: 해외 자동매매 체결은 `TradeRecord::new_overseas()`로 USD 단가/금액/추정 수수료, 체결 시점 USD/KRW 환율, KRW 환산 금액/손익을 함께 저장한다.

## P2 — 리스크 관리 고도화

- [x] 전략별 일일 주문 횟수 제한
  - 기본값: 종목당 매수 1회, 매도 1회. 사용자가 Settings/Strategy UI에서 조정 가능하게 한다.
  - 구현: `StrategyManager::on_tick()`이 전략 ID와 신호를 함께 전달하고, `RiskManager`가 `전략 ID + 종목 + 방향 + 날짜`별 접수 횟수를 기본 매수 1회/매도 1회로 제한한다.
  - 구현: Settings → 리스크 관리에서 전략/종목별 일일 매수·매도 제한을 각각 조정한다. 값 0은 제한 없음으로 처리한다.
- [x] 연속 손실 차단
  - 같은 전략/종목에서 N회 연속 손실이면 해당 조합을 자동 비활성화한다.
  - 구현: 매도 체결 손익이 확정될 때 `RiskManager`가 `전략 ID + 종목`별 연속 손실 횟수를 집계하고, 기본 3회 도달 시 해당 조합의 신규 매수 진입을 차단한다.
  - 구현: 청산 매도까지 막지 않도록 연속 손실 차단은 신규 매수 신호에만 적용한다. Settings에서 기준 횟수를 조정하며 0은 제한 없음으로 처리한다.
- [x] 변동성 기반 주문 수량 산정
  - 고정 수량 대신 계좌 위험 한도와 ATR 기반 손절폭으로 수량을 계산한다.
  - 구현: 자동매매 시작 시 국내/해외 일봉 OHLC에서 ATR14를 계산해 `RiskManager`에 캐시한다. 국내는 KRW, 해외는 USD cents 단위로 보존한다.
  - 구현: `OrderManager::process_buy()`가 매수 주문 직전 총잔고(KRW), 거래당 위험 한도(bps), ATR 손절 배수, 종목당 최대 비중으로 주문 수량을 재계산한다.
  - 구현: 해외 종목은 현재 환율로 USD cents 위험폭과 주문금액을 KRW 환산해 같은 리스크 한도에서 검사한다.
  - 구현: Settings → 리스크 관리에서 변동성 기반 수량 산정 ON/OFF, 거래당 위험 한도, ATR 손절 배수를 조정하고 ATR 준비 종목 수를 확인한다.
- [x] 슬리피지 추정/기록
  - 신호 가격, 주문 가격, 체결 가격을 모두 저장해 전략 성과를 사후 분석할 수 있게 한다.
  - 구현: 자동매매 체결 기록에 `signal_price`, `order_price`, `price`, `slippage`, `slippage_bps`를 저장한다. 국내는 KRW, 해외는 USD cents 단위이며 양수 슬리피지는 불리한 체결로 해석한다.
  - 구현: History 체결 기록 표에서 데스크톱 화면 기준 슬리피지와 bps를 표시한다.

## P3 — Frontend FSD 구조화

- [x] 프론트엔드에 Feature-Sliced Design(FSD) 점진 도입
  - Rust/Tauri 백엔드(`src-tauri/src/{trading,storage,api,...}`)는 현재 도메인별 분리가 잘 되어 있으므로 우선 변경하지 않는다.
  - 대상은 React 프론트엔드 `src/`이며, 기능 변경과 구조 이동을 섞지 않고 작은 PR/커밋 단위로 진행한다.
  - 공용 스킬화 완료: `.github/skills/frontend-fsd/SKILL.md`를 앞으로 프론트엔드 구조 변경 작업의 기준으로 사용한다.
- [x] `shared` 레이어 신설
  - 공통 Tauri IPC wrapper, 공통 타입, theme/helper, 범용 UI를 `src/shared/{api,lib,ui,config}`로 이동한다.
  - 1차 후보: `src/api/commands.ts`, `src/api/transport.ts`, `src/theme`, 범용 layout/helper.
  - 구현: `src/shared/api`, `src/shared/config/{theme,scheduler}`, `src/shared/ui`를 만들고 기존 경로는 re-export로 유지한다.
- [x] `entities` 레이어 신설
  - 도메인 명사 기준으로 `account`, `stock`, `order`, `trade`, `position`, `strategy`, `settings`, `log` slice를 만든다.
  - 1차 후보: `src/store/accountStore.ts`, `src/store/settingsStore.ts`, `src/api/types.ts`의 도메인 타입, 전략 관련 타입.
  - 구현: 계좌/설정/자동매매 Zustand store를 `src/entities/{account,settings,trading}/model`로 이동하고 기존 `src/store/*`는 호환 re-export로 유지한다.
- [x] `features` 레이어 신설
  - 사용자 행동 기준으로 `manual-order`, `symbol-search`, `strategy-toggle`, `strategy-configure`, `trading-start-stop`, `log-filter`, `discord-notification-config`를 분리한다.
  - `features`는 `entities`와 `shared`만 직접 의존하도록 한다.
  - 구현: 주요 feature slice 디렉터리와 public API placeholder를 신설해 후속 이동 기준을 고정했다.
- [x] `widgets`/`pages` 레이어 정리
  - 큰 UI 블록은 `widgets/app-shell`, `widgets/sidebar`, `widgets/stock-chart`, `widgets/account-summary`, `widgets/strategy-list`, `widgets/log-viewer`로 분리한다.
  - `pages/{dashboard,trading,strategy,history,log,settings}`는 라우트 조립만 담당하게 얇게 만든다.
  - 구현: AppShell, Sidebar, 국내/해외 StockChart를 `src/widgets/*`로 이동하고 라우트 페이지를 `src/pages/{route}/ui/Page.tsx` 구조로 정리했다.
- [x] FSD import 경계 검증 추가
  - 구조 이동 후 ESLint 또는 별도 스크립트로 하위 레이어가 상위 레이어를 import하지 못하게 한다.
  - 허용 방향: `app → pages → widgets → features → entities → shared`.
  - 구현: `scripts/check-fsd-imports.mjs`와 `npm run check:fsd`를 추가했다.

## P4 — Codex 마이그레이션

- [x] `AGENTS.md`를 Codex의 최상위 작업 지침으로 유지
  - 작업 전 읽을 문서, 검증 명령, 금지사항, 변경 이력만 간결히 둔다.
- [x] `.github/codex-instructions.md`를 상세 에이전트 지침으로 사용
  - 기존 Copilot 중심 지침은 호환용 shim으로 남긴다.
- [x] `.github/skills/**/SKILL.md`는 Codex도 읽을 수 있는 도메인 스킬로 유지
  - KIS 특이사항, Rust/Tauri 패턴, React/UI 규칙을 실제 버그 발견 시 즉시 갱신한다.
  - 프로젝트 브리지 스킬 `.codex/skills/kisautotrade-*`가 현재 작업 저장소 루트 기준으로 원본 스킬을 다시 읽도록 구성했다.
  - Codex 런타임이 계정 스킬만 읽는 경우 `scripts/sync-codex-skills.ps1`로 `~/.codex/skills`에 동기화한다.
- [x] 신규 리스크/전략 안정화 패턴을 `rust-skills`와 `kis-api`에 반영
  - 같은 문제가 반복되면 코드 수정과 동시에 스킬 문서를 업데이트한다.
  - 구현: 주문번호 기반 부분체결/거부 상태, 해외 수수료·환율 기록, 전략별 일일 주문 제한, 연속 손실 신규 진입 차단 패턴을 관련 스킬 문서에 반영했다.

## P5 — 검증과 운영

- [x] `cargo check` / `npx tsc --noEmit` 경고 0개 유지
- [x] 전략별 단위 테스트 추가
  - 등락 반복 가격열에서 불필요한 매수/매도 반복이 차단되는지 테스트한다.
  - 구현: 공통 `TradeGuard` 단위 테스트로 동일 방향 쿨다운, 손절 후 재진입 금지, 휩소 차단, 비용 차감 후 기대 순익 필터를 검증한다.
  - 구현: `RiskManager` 단위 테스트로 ATR 기반 국내/해외 수량 산정, 일일 주문 제한, 연속 손실 차단/해제를 검증한다.
- [x] 모의투자 E2E 체크리스트 작성
  - 국내 매수/매도, 해외 매수/매도, 체결 조회, 잔고 반영, 매도 제한 에러 처리를 분리 검증한다.
  - 구현: `docs/mock-trading-e2e-checklist.md`에 국내/해외/자동매매 모의투자 검증 절차를 분리해 작성했다.
- [x] Windows/macOS 데이터 경로 회귀 테스트
  - `data/`, `logs/`, `profiles.json`, `.env` 경로가 OS별로 깨지지 않는지 확인한다.
  - 구현: `storage::build_daily_path()`와 `build_monthly_path()` 단위 테스트로 OS 경로 구분자에 의존하지 않는 날짜 경로 생성을 검증한다.

---

## 최근 점검 메모

- `StrategyManager::on_tick()`의 신호는 `OrderManager::submit_signal()`에서 공통 `TradeGuard`를 통과한 뒤 주문된다.
- `PriceConditionStrategy`와 `LeveragedTrendHoldStrategy`는 자동매매 시작 시 국내/해외 잔고 기반으로 내부 포지션 상태를 복원한다.
- `run_trading_daemon()`은 국내/해외 주문 모두 주문번호 기반 체결 조회를 먼저 시도하고, 실패 시 기존 다음 틱 가격 확인으로 보완한다.
- 미체결 주문 UI는 `PendingOrder.filled_quantity` 기반으로 부분체결/잔여 수량을 표시하고, KIS 접수 거부는 실패 주문 이력으로 분리 저장한다.
- 전략 신호는 전략 ID와 함께 주문 관리자에 전달되며, 리스크 관리가 전략/종목/방향별 일일 주문 접수 횟수를 제한한다.
- 리스크 관리는 전략/종목별 연속 손실을 집계해 기준 횟수 도달 시 신규 매수 진입을 차단하고, Dashboard/Settings에 차단 조합 수를 표시한다.
- 변동성 기반 수량 산정은 자동매매 시작 시 ATR14가 준비된 종목에 대해 전략 고정 수량 대신 계좌 위험 한도와 ATR 손절폭으로 매수 수량을 계산한다.
- 자동매매 체결 기록은 신호가, 주문가, 체결가 기반 슬리피지 비용과 bps를 저장하고 History에서 표시한다.
- 해외 자동매매 체결은 국내 `PositionTracker`가 아닌 `OverseasPositionTracker`에 USD cents 단위로 반영한다.
- KIS 공식 샘플 기준 해외 미국 모의 주문 TR-ID는 존재하나, 모의투자에서는 지정가 주문과 전체 조건 조회 위주로 제한된다.

*마지막 업데이트: 2026-07-01*
