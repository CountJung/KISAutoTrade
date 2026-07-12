# KISAutoTrade — Todo

> 완료 이력은 `git log`와 릴리스 노트에서 관리한다. 이 문서에는 아직 끝나지 않은, 검증 가능한 작업만 둔다.
> 우선순위는 `P1 정확성/신뢰성 → P2 전략 연구 UX → P3 유지보수` 순이다.

## *마지막 비판적 점검: 2026-07-12*

## P1 — 진행 가능한 잔여

- 없음. 현재 코드와 fixture로 진행할 수 있는 P1 항목은 모두 반영했다.

## P1 — 보류 사항

- [ ] MariaDB contract fixture와 지원 버전 matrix 검증.
  - adapter·DDL은 유지하지만 사용자 결정에 따라 실제 서버 fixture 검증은 보류한다.
- [ ] 여러 broker/account 포지션과 pending을 동시에 상주·대조하는 multi-scope runtime.
  - 현재 제품은 단일 활성 scope 불변식을 사용한다. 동시 다계좌 운용을 제품 범위로 채택할 때 `(broker, account, market, symbol)` tracker와 scope별 credential reconciliation으로 전환한다.

## P1 — 외부 검증 환경 필요

- [ ] TLS verify-full 인증서, 서버 강제 단절·재접속 contract.
  - CA/hostname 인증서 fixture와 서버 kill/restart 제어가 가능한 별도 통합 테스트 환경에서 검증한다.
- [ ] KIS/Toss 실제 정정·취소 응답의 최종 provider mapping 확정.
  - 공통 상태와 정정/reconciliation 직렬화는 구현했으나 old/new order ID 및 거부·만료 필드 의미는 provider fixture/소액 검증 후 고정한다.

## P2 — 전략 시뮬레이션과 연구 워크플로

- [ ] 현재의 “신호 미리보기”와 실제 daemon 실행 모델의 차이를 제거한다.
  - 일봉 1개를 `on_tick()` 1회로 replay하는 전략과 실제 10초 tick/1분봉 전략의 시간축을 명시적으로 분리한다.
  - live와 preview가 같은 입력 정규화, warmup, session/장마감, 주문 수량·리스크 규칙을 재사용하게 한다.
  - look-ahead bias 없는 deterministic replay fixture로 live/preview signal parity를 검증한다.

- [ ] 카드 전체 너비를 활용하는 백테스트 결과 요약을 추가한다.
  - 기간·봉 주기·초기자본·수수료·세금·슬리피지·환율을 입력받는다.
  - 누적 수익률, MDD, 승률, 손익비, turnover, exposure, 거래 목록과 equity curve를 표시한다.
  - 단순 신호 개수와 실제 주문 가능/체결 가정 결과를 구분해 보여준다.

- [ ] 전략 세팅 비교와 재현 가능한 실험 저장을 지원한다.
  - 현재 편집값을 A/B preset으로 복제해 같은 데이터 구간에서 비교한다.
  - 전략 버전, 파라미터, 데이터 범위/source, 비용 가정, 생성 시각을 결과와 함께 저장한다.
  - in-sample/out-of-sample 또는 walk-forward 구간을 제공해 과최적화를 경고한다.

- [ ] 시뮬레이션 UX의 반응형·접근성 회귀를 계속 검증한다.
  - 모든 전략 카드가 데스크톱에서도 1열 전체 너비를 유지하는 Playwright 테스트를 유지한다.
  - 좁은 화면에서 파라미터 입력, 티커 Select, 미리보기 버튼, 차트가 겹치거나 잘리지 않는지 검증한다.
  - 편집값·티커·broker가 바뀌면 이전 미리보기 결과를 즉시 무효화한다.

## P3 — 유지보수·품질 게이트

- [ ] 1,000라인 초과 파일을 책임 단위로 분리한다.
  - `src-tauri/src/trading/strategy/leveraged_trend_hold.rs`: 계산/상태/preview/tests 분리.
  - `src-tauri/src/commands/trading.rs`: lifecycle/reconciliation/price-source/daemon cycle 분리.
  - `src/pages/trading/ui/Page.tsx`: broker별 orchestration과 공통 order form 분리.
  - `src/api/hooks.ts`: account/market/order/strategy/settings query 모듈 분리 후 public API 유지.
  - `src-tauri/src/commands/toss.rs`: diagnostic/preflight/orders surface 분리.
  - `src-tauri/src/trading/order.rs`: facade를 낮추고 남은 helper/state 책임 분리.
  - 검증: 변경 파일과 신규 파일은 1,000라인 아래, FSD/API public surface와 IPC 이름은 유지한다.

- [ ] 핵심 거래 흐름의 자동화 테스트를 release gate로 승격한다.
  - broker mock 기반 제출→부분체결→완전체결→취소/거부→재시작 복구 테스트를 추가한다.
  - balance fail-closed, scope 전환, midnight rollover, manual/auto parity, 인증 없는 REST 거부를 포함한다.
  - `cargo check`, `cargo test`, `npx tsc --noEmit`, `npm run check:fsd`, `npm run test:e2e`, OpenAPI 검증을 CI에서 실행한다.

- [ ] 의존성·릴리스 보안 점검을 자동화한다.
  - `cargo audit`와 npm audit 정책, Dependabot/Renovate, lockfile 검증을 CI에 추가한다.
  - release artifact 서명/해시와 updater 경로를 검증하고, 실패 시 사용자가 확인할 수 있게 한다.

- [ ] 사용자 가이드를 broker별 실제 지원 범위와 일치시킨다.
  - KIS 전용 소개를 KIS/Toss 공통 기능, broker별 주문·시세·자동매매 제한 표로 교체한다.
  - 실제 UI의 13개 전략 카드, 프로파일 설정, 세션/캔들 source, 소액 실거래 gate를 기준으로 오래된 절차를 정리한다.

## 완료 기준

- 각 항목은 코드, 실패/복구 테스트, 사용자-visible 상태, 관련 문서가 함께 반영되어야 닫는다.
- 실계좌 검증은 모의/fixture → read-only 계좌 조회 → 명시 승인된 소액 주문 순서를 지킨다.
- 경고를 포함해 `cargo check`와 TypeScript 검증이 깨끗해야 하며, 주요 UI 배치 변경은 Playwright로 확인한다.
