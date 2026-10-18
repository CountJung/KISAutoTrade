# 작업 기록: P2-06 연속 가격 관측의 진입 필터

## 1. 메타데이터

- 작업: SOXQ-P2-06, 2026-10-18 KST
- 상태: Done
- 담당: Codex, core/UI 구현 담당, stock_analyst 독립 감사
- 브랜치: `fix/mac-security-storage-2026-10-06`, 기준 `6f9ca35`
- 위험: 보통 — 자동 전략의 선택적 신규 진입 조건과 완료 일봉 준비 경계
- 관련: [할일](../todo.md), [연구 설계](../research/consecutive-filters-20261018/design.json)

## 2. 목표와 완료 조건

- [x] 누적 변화·ATR·장기 MA 필터를 기본 0(비활성)으로 제공한다.
- [x] 기존 buy_days/sell_days와 실제 체결 피드백·청산·공통 guard/risk를 보존한다.
- [x] live 가격 관측열과 직전 완료 일봉 맥락을 분리한다.
- [x] 자료 부족·0 ATR·잘못된 설정은 신규 매수만 보류하고 보유 청산을 유지한다.
- [x] 저장·short/continuous preview·준비 진단·A/B·모바일 입력을 검증한다.
- [x] 1,224회 기본 호환, 6회 장기 기본 호환, 24회 후보 연속·24회 분할 대조·288회 별도 분기 계좌를 검증한다.
- [x] 비용별 순수익·MDD·회전·최악 완료 손실·실현/잔여·노출과 기각 근거를 기록한다.
- [x] 독립 감사·문서·지도·그래프·경고 0 검증 후 커밋/푸시한다.

## 3. 시작 상태와 보존

문서 정리 요청 확인으로 잠시 대기했던 P2-06 UI 6개 파일과 연구 설계를 이어받았다. 루트 문서 정리는 이미 `694fa14`에 반영되어 별도 변경하지 않는다. 기준 생산 라이브러리는 Git archive로 `.cache/consecutive-head-baseline/`에 고정했다. 연구 harness/빌드 산출물은 `.cache/`와 `target/`에 둔다. 민감 설정 파일을 읽지 않고 기존 연구 원본을 보존한다.

## 4. 조사 근거와 계약

기존 `sequence.rs`의 연속 조건은 실시간 약 10초 가격 관측을 세며 일수로 간주할 수 없다. 기존 필드 이름은 저장 호환을 위해 유지하고 화면과 두 신호 reason의 단위를 관측 횟수로 바로잡는다. 기본 호환 대조는 Consecutive의 reason 키에서 정확한 숫자+상승/매수·하락/매도 템플릿만 정규화하며 다른 필드를 바꾸지 않는다.

`manager/live_history.rs`의 완료 일봉 coordinator는 기존 52주 고가 전용 경계를 선택적 전략 훅으로 확장한다. Consecutive의 새 context 적재/무효화는 가격 관측열과 확정 보유를 초기화하지 않아야 한다. 일반 replay의 후보 OHLC는 해당 평가와 실제 피드백이 끝난 후에만 완료 자료로 게시한다.

| 필드 | 범위와 조건 |
|---|---|
| min_cumulative_move_pct | 0..100, 현재/매수 관측 수 이전 가격의 누적 변화율 이상 |
| min_move_atr_multiplier | 0..20, 같은 가격 상승폭이 이전 완료 일봉 15개의 TR14 평균×배수 이상 |
| long_trend_period | 정수 0..512, 현재 가격이 이전 완료 종가 MA보다 엄격하게 큼 |

각 0은 비활성이고 복수 활성은 AND다. ATR/MA는 generic 일봉에서만 지원하고 미지원 간격은 명시적으로 거부한다. live에서는 완료 일봉과 현재 quote 관측의 서로 다른 간격을 화면·문서에서 구분한다. 준비 진단은 포지션 크기 조절 ATR 준비와 분리한다. 진입후보·준비 부족·각 필터 거절·통과를 후속 주문 차단과 구분한다.

## 5. 영향과 금융 경계

- Rust: Consecutive 모듈 분리, 선택적 strategy 훅, 완료 일봉 공급, 저장/preview 검증, optional DTO.
- UI: page-local 입력/준비 표시, TS 미러, 기존 params 전달·A/B 보존·이전 응답 무효화.
- 새 IPC/REST endpoint: N/A — 기존 저장/short/continuous 경로를 사용한다.
- DB/계좌 schema·broker 인증·rate limit 변경: N/A.
- 실제·모의 주문, 자동매매 시작: 실행하지 않는다.
- 기존 scope·owner·pending·guard·risk·지속 저장 실패 차단을 유지하며 수익 gate를 완화하지 않는다.
- 연구는 동결 공개 가격과 합성 fixture만 사용한다. live 설정에 후보를 자동 채택하지 않는다.

## 6. 도구·독립 감사

symbol_navigator 역할의 targeted source 참조 조사로 factory·trait·warmup·coordinator·preview·저장 경로를 확인했다. Serena MCP는 앞선 실행에서 응답 지연이 확인되어 이번에는 좁은 소스/컴파일 참조 확인으로 대체한다. 구조 분리 후 Graphify CLI refresh/check를 수행하고 project_mapper pass를 위임한다. 기존 simulation ATR은 정수 평균/별도 공개 계약이 달라 새 완료 일봉 context와 자동 공용화하지 않는다. stock_analyst가 사전 설계·정규화 범위를 감사했으며 결과의 독립 회계·해석 감사는 실행 후 수행한다.

## 7. 검증 결과

최종 고정 소스에서 아래 검증을 통과했다.

| 확인 | 결과 |
|---|---|
| Rust fmt/check/Clippy | workspace all-targets locked; Clippy all-features -D warnings, 경고0 |
| 전체 Rust test | lib537 + production integration4 + doc3 =544, 실패0 |
| focused Rust | consecutive24, entry_filter5; 신규 strict integer-float 저장/preview·늦은 warmup 재요청·비용별 SPY 매도 경로 포함 |
| TypeScript/FSD/lockfiles | 모두 PASS |
| 전체 Chromium E2E | 125/125; 실제 provider/Tauri 주문 E2E 아님 |
| 실제 web build/preview | charts chunk 분리 후 경고0; production 전략·차트33/33 |
| TODO renderer | 3/3; 색상/필터/모바일·열린 항목 보존 회귀 |
| 기본 생산 대조 | flat1224 + long6 전체 backtest/signals, 두 단위 문구와 정확한 알려진 종료 suffix 예외만 적용 |
| 후보 생산 실행 | 연속24·분할동일24·별도분기288, parent 기록 산술55,986 PASS |
| 독립 stock 감사 | 312개 원장148,099체크 오류0; 최대 약0.01원/0.00005%p 출력 반올림 차이 |
| 코드·문서 최종 감사 | P1/P2 잔여0; 모두0 중립/활성 legacy unknown 설명 drift 수정 확인 |
| Graphify | CLI refresh; canonical/HTML node ID·edge multiset·community legend·report5502/12008/528 정확히 일치 |

초기 production build는 index506.49kB 경고1개였고 Vite 기존 manualChunks에 안정된 lightweight-charts 경계를 분리해 index331.06kB/charts175.25kB로 해소했다. warning limit는 바꾸지 않았다. 실제 production preview33개 회귀로 분리 번들 로딩/차트 초기화도 확인했다. 그래프의 symbol 없는 JSON 설정4개 진단은 기존 AST 도구 한계이며 코드/빌드 경고와 구분한다. PostgreSQL 환경이 없는 전체 테스트를 외부 DB contract 검증으로 집계하지 않는다. 실전·모의 주문 및 자동매매 시작은 실행하지 않았다.

project_mapper가 신규21개 파일 인벤토리와 모듈 책임·완료 일봉/재생 흐름을 갱신하고 지도 검사를 통과했다. 최종 상대 링크253개가 모두 유효했고 루트 일반 문서는 README/AGENTS/CLAUDE만 유지했다. 그래프/할일 freshness와 Git 게시 여부는 완료 직전에 재확인한다. 초기 UI 집중 실행 7개 중 4개 통과·3개 실패 후, 실제 모바일 Grid overflow와 두 테스트 locator/입력 편집 문제를 수정해 같은 7개가 통과했다. 이를 처음부터 7개 통과한 것으로 기록하지 않는다.

## 8. 한계·롤백

이미 관측한 2023~2025의 상관된 ETF 3개를 사용하는 사후 진단이다. 미사용 2026을 독립 OOS 증거라고 부르지 않는다. 분기 초기화는 매번 별도 1천만원 계좌이며 수익률을 합산/복리하지 않는다. 동일 요청 수량과 실제 체결 수량·보유 경로·노출 차이를 구분한다. 기본 비활성 필터를 유지하고 변경 커밋을 revert하면 롤백할 수 있다. 실제 자료 coverage·장중 체결·운용 안전성은 이 검증이 증명하지 않는다.


## 9. 전달·검증 보완

- 입력 JSON의 정수값 float 200.0/512.0을 검증과 factory 양쪽에서 같은 기간으로 적용하고 소수 기간0.5/2.5는 저장 전에 거부한다.
- 늦은 legacy warmup이 scoped 완료 context를 무효화하면 해당 ready 상태표도 제거해 다음 tick 준비가 재요청되게 했다. 미보유 신규 매수와 확정 보유의 매도 관측은 분리한다.
- 기본 SPY20 장기 대조의 최초 실패는 정확한 마지막 미체결 reason의 알려진 종료 suffix 때문이었다. 독립 old/new 원문 diff에서 해당 문구1개 외 회계/경로 차이가 없음을 확인하고 suffix 원문 보존 정규화로 재검증했다.
- 연구 결과에 따라 전 필터 기본0을 유지한다. 자료/실거래 검증과 사용자 판단 대기 항목의 주황 강조를 보존한다.
- 변경 파일은 Git diff와 자동 프로젝트 지도에서 확인하며 모든 신규 일반 문서·연구·작업 기록을 docs 아래에 둔다. 커밋/원격 게시 이력은 현재 브랜치 Git log에서 확인한다. 캐시·민감 설정은 커밋하지 않는다.

최종 아카이브 재감사는 312행·148,099검사 오류0이며 이전 독립 감사148,073검사와 경제 지표가 정확히 같음을 추가 확인했다. 변경 commit `d39d15f`와 원격 HEAD 일치·작업 트리 clean을 확인한 뒤 이 최종 감사 기록을 별도 문서 커밋으로 덧붙인다.
