# 작업 기록: 자동 주문 실행 context와 예약 보호

## 1. 범위와 시작 상태

- 작업: P1-RUN-01, 2026-10-18 KST
- 상태: Done — 구현·독립 안전/문서 리뷰와 최종 검증 완료
- 브랜치: `fix/mac-security-storage-2026-10-06`, 시작 HEAD `2c593de`
- 시작 변경: 이전 회차에서 중단한 order 모듈 분리와 실행 context 초안을 보존하고 이어서 구현했다. 사용자 변경을 되돌리지 않는다.
- 위험: 높음 — 자동 주문 예약·실행 세대·provider client 소유권 경계
- 실제·모의 증권사 주문과 자동매매 시작은 실행하지 않는다. 테스트는 합성 자료와 localhost만 사용하고 민감 파일을 읽지 않는다.

## 2. 문제와 완료 조건

quote·종목명·포지션·주문 준비를 기다리는 동안 정지/재시작이나 설정 변경이 일어나면 이전 신호가 현재 계좌로 재해석되거나, 이전 snapshot이 새 전략 상태를 덮을 수 있었다. KIS 재시도마다 공유 client를 다시 읽는 경로도 계좌 전환에 영향을 받았다.

- [x] 시작 시 profile/broker/account/credential/투자 모드를 검증하고 run epoch·configuration revision과 provider 의존성을 고정한다.
- [x] 신호 생성과 rollback은 현재 실행에서만 전략을 변경한다.
- [x] 자동 주문은 expected context를 반드시 전달하며 OrderManager 예약 mutex에서 guard/submitting/예산 변경 전에 검증한다.
- [x] 같은 계좌의 정지→재시작과 설정 변경→복원의 ABA를 검출한다.
- [x] 이미 예약한 주문은 최초 client·scope로 전송·재시도·기록하고 정지로 pending/submitting/예산을 지우지 않는다.
- [x] 수동 주문은 자동매매 run 없이 기존 요청 scope로 처리한다.
- [x] barrier/future·localhost 회귀, 전체 Rust 검증, 문서·지도·그래프 게이트와 독립 리뷰를 통과한다.

## 3. 구현 경계

OrderManager는 비민감 `ExecutionRunContext`와 직렬화하지 않는 pinned client/profile을 가진다. start/stop과 scope 교체는 epoch를 변경하고, 설정 변경·저장 실패 복원은 manager의 새 revision을 같은 manager→order 임계구역에서 공개한다. daemon은 bool→strategy→order 순서로 검증한다. provider/network await에 bool/strategy guard를 유지하지 않는다.

예약이 먼저 완료되었다면 정지 후에도 그 주문의 응답과 pending을 원래 scope에서 관리한다. 정지는 증권사 주문 취소가 아니다. stale 거절은 새 전략에 예전 보유 상태를 복원하지 않는다. 일반 전략의 설정 재생성 중 보유 상태 보존은 별도 P1-CONFIG-01로 남긴다.

기존 1000줄 초과 `order.rs`와 `order/submission.rs`는 상태·health·pending·legacy와 positions/prepare/provider/records로 분리했다. 기능 변경 전 71개 함수 본문을 공백 정규화 비교해 동일함을 확인한 감사 자료를 사용했다. 1050줄 fills도 적용·collector·reconciliation·provider mode로 분리했고 원래 collector 본문과 체결 산술·수수료·기존 테스트를 보존했다. 최종 order439/submission856/fills597줄이며 신규 소스도 모두1000줄 미만이다. 자동 submit은 기존 `StrategySignal`로 ID와 신호를 함께 받아 Clippy 인수 제한을 억제 없이 통과한다. 반복 retry helper를 의미 검증 없이 합치지 않는다.

## 4. 영향과 검토

- Rust 내부 context·자동 submit signature 변경. 새 IPC·TS DTO·저장 schema는 없다.
- IPC/웹 lifecycle·설정 publication·수동 응답 exhaustive match를 함께 갱신한다.
- 프로파일 적용은 manager→order gate 아래 scope/client/config/국내·해외 tracker/전략을 함께 게시한다. 비활성 프로파일 삭제는 활성 runtime을 재생성하지 않는다. 첫 프로파일 자동 활성화와 활성 프로파일 투자 모드 감지는 실행 중 거절하고 설정 publication과 직렬화한다.
- 수동 holdings는 원 profile/client로 조회하고 tracker 게시·예약 전에 scope/profile/client를 확인한다. 동일 계좌의 비활성 profile을 선택하지 않으며 no-profile KIS 설정 계좌도 보존한다. 이전 계좌·모드의 잔고 부족은 새 실행의 매수 정지를 설정하지 않는다.
- 체결 조회는 scope/provider snapshot을 고정하고 응답 적용·health 갱신 전에 다시 검증한다. 다른 scope의 direct fill·0체결 종결도 보류한다. 동일 계좌·같은 provider snapshot의 재시작은 실제 체결을 멱등 반영한다.
- KIS pending의 기존 provider TR-ID로 모의·실전 출처를 확인한다. 반대 모드·출처 미상은 조회/체결/종결을 보류하고 기존 reconciliation health 오류로 진단한다. 실제 endpoint나 저장 schema를 바꾸지 않는다.
- code-navigation: symbol_navigator의 Serena 정의/참조 및 lock 순서 조사. 주문·daemon 구현 위임과 별도 독립 안전 리뷰를 수행한다.
- 새 모듈 지도는 project_mapper에게 위임하고 구조 분리에 맞춰 Graphify를 refresh한다.
- provider API wire 계약·TR-ID·재시도 횟수/간격은 유지한다. 테스트 TokenManager cache 주입은 `cfg(test)`이며 실제 token endpoint를 호출하지 않는다.

## 5. 검증 결과

| 상태 | 확인 | 결과 |
|---|---|---|
| PASS | 실행 경합 신규 회귀22개 | run9·daemon5·reconciliation6·일봉/jobs2, 전체 lib에 포함 |
| PASS | cargo check --workspace --all-targets --locked | 경고0 |
| PASS | cargo test --workspace --locked | lib297 + integration3 + doc3 = 303, 실패/무시0 |
| PASS | cargo clippy --workspace --all-targets --all-features --locked -- -D warnings | 경고0, 기존 StrategySignal 재사용으로 인수 경고 억제 없음 |
| PASS | TypeScript / FSD / Vite | tsc noEmit·import 경계·build:web 경고0 |
| PASS | project-map / graph / TODO / fmt / diff | 신규17파일 지도, graph freshness, 열린27항목, TODO renderer3테스트 |
| PASS | 링크·독립 안전/문서 감사 | 상대 링크152개 broken0, 발견 결함 수정·재감사. close 예시도 현재 무효화 순서로 보정 |
| NOT RUN | Playwright | UI 구조·상호작용 변경 없음, TODO HTML은 renderer/keyboard 회귀3개로 검증 |
| Not run | 실제 provider·모의/실전 운용 | 이번 작업 범위 밖 |

전체 테스트 1차에서 프로파일 없는 진단 ticket이 자료 적용을 허용하는 실패 1건을 발견했다. 단순 기대 변경 대신 apply 경계에서 빈 profile ticket을 명시 거절하여 실패 진단을 유지하도록 수정했고 focused 회귀가 통과했다.

Graphify의 symbol 없는 JSON4개 도구 진단은 기존 분석 한계이며 Rust/TypeScript 코드 경고와 구분한다. 최종 소스는 분리 후 전체 추출과 후속 update를 완료했다. 실제 자료·계좌·운용 성과를 검증했다고 주장하지 않는다.

## 6. 전달과 잔여

UI의 시작/정지·설정 저장 흐름은 유지한다. 늦게 도착한 이전 실행 작업은 폐기하며 이미 예약한 주문은 주문/미체결 화면에서 계속 확인한다. 앱 close의 async stop callback은 종료 전에 완료를 보장하는 구조가 아니므로 이번 변경으로 프로세스 종료 내구성을 새로 보장하지 않는다. 운영 검증과 나머지 열린 TODO는 별도로 남긴다.

롤백은 이 변경 커밋을 revert한 뒤 같은 검증 게이트를 통과한다. pending·예산 저장 자료는 보존한다. 다음 자동 구현 우선순위는 일반 전략 설정 변경/복원 중 보유 상태 보존(P1-CONFIG-01)이다.
