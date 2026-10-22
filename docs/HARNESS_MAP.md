# KISAutoTrade 검증 Harness 지도

> 원칙: 가장 작은 빠른 검증부터 실행하고, 위험 반경에 따라 확대한다. 어떤 명령도 실거래 안전성이나 전략 수익성을 증명하지 않는다. 외부 계좌/API를 사용하는 수동 검증은 명시적 승인 없이는 실행하지 않는다.

## 1. 환경 기준

- Node.js `>=20`, npm `>=10` (`package.json`, `.nvmrc`)
- Rust `>=1.90`, workspace member `src-tauri` (root `Cargo.toml`); Tauri 2.12 의존성의 최소 버전
- 재현 가능한 의존성 검증은 npm `ci`와 Cargo `--locked`를 우선한다.

```bash
npm ci --ignore-scripts
cargo metadata --locked --format-version 1 --no-deps
```

기존 작업 환경에서 문서/소규모 변경만 검증할 때 불필요한 reinstall은 생략할 수 있다.

프로젝트별 캐시는 저장소 안에 둔다: Cargo `target/`, Vite `node_modules/.vite/`, Playwright `test-results/`·`playwright-report/`, Serena `.serena/cache/`, Graphify `graphify-out/cache/`. 일회성 오프라인 연구 harness는 `.cache/`에 두며 모두 Git에서 제외한다. `.cargo/config.toml`은 상대 `target-dir = "target"`인 공통 설정으로 추적한다. `scripts/setup-local.sh`는 이를 외부 절대 경로로 덮어쓰지 않는다. Rust toolchain·Cargo registry와 npm 전역 다운로드 캐시는 도구의 사용자 공통 저장소를 유지한다.

## 2. 기본 정적·단위 게이트

| 목적 | 실제 명령 | 근거 |
|---|---|---|
| TypeScript 타입 | `npx tsc --noEmit` | release-gate frontend |
| Frontend FSD import | `npm run check:fsd` | `package.json` |
| Rust 전체 target check | `cargo check --workspace --all-targets --locked` | release-gate rust |
| Rust lib tests | `cargo test --workspace --lib --locked` | release-gate rust |
| lockfile 일관성 | `npm run check:lockfiles` | release-gate frontend |
| 할일 HTML 원본 동기화 | `npm run check:todo` | release-gate frontend |
| Web production build | `npm run build:web` | `package.json` |

`cargo check`/`test`는 저장소 루트 workspace에서 실행한다. `src-tauri`에서 빠르게 확인할 때도 최종 보고에는 실행 위치와 명령을 정확히 적는다.

## 3. 변경 유형별 필수 조합

| 변경 범위 | 최소 harness | 추가 harness/확인 |
|---|---|---|
| 문서만 | 링크·경로 수동 확인 | 파일 추가/이동이면 project-map drift 확인 |
| React 컴포넌트/페이지 | `npx tsc --noEmit`, `npm run check:fsd` | focused Playwright, `npm run build:web` |
| shared TS 타입/command/hook | 위 frontend 2개 | Rust serde/IPC counterpart, 관련 UI query invalidation |
| Rust 일반 로직 | `cargo check ...`, focused `cargo test <filter> --locked` | 최종 `cargo test --workspace --lib --locked` |
| IPC 추가/변경 | frontend 2개 + Rust 2개 | `lib.rs` 등록, Web route 여부, `docs/ipc-commands.md` |
| broker/provider | Rust 2개 | `npm run verify:toss-openapi`(Toss), 공식 API 근거, mock/fixture |
| 주문/preflight/fills | Rust focused + 전체 lib test | scope, 중복 주문, provider 실패, 영속화 실패, no-live-order 확인 |
| 리스크/복원 | risk/restore focused + 전체 lib test | 일자 경계, scope 격리, restart restore, emergency stop |
| 저장/DB | Rust 2개 | database contract test 요구 환경 확인, export/import fail-closed |
| UI 상호작용/레이아웃 | frontend 2개 | `npm run test:e2e` 또는 focused spec |
| 파일/모듈 구조 | 해당 코드 게이트 | `npm run project-map:update`, `npm run check:project-map` |
| 구조 리팩터링 | 해당 코드 게이트 | Graphify refresh/check, Serena 참조 확인(필요 시) |
| 릴리스 | CI 전체와 `docs/release-security.md` | artifact 검증; 별도 승인 필요 |

## 4. Focused Rust 테스트

Cargo의 test filter를 사용하되, 최종적으로 lib 전체를 실행한다.

```bash
cargo test --workspace preflight --locked
cargo test --workspace risk --locked
cargo test --workspace risk_restore --locked
cargo test --workspace strategy_preview --locked
cargo test --workspace database_contract --locked
cargo test --workspace --lib --locked
```

필터가 실제 테스트명과 일치해 실행 건수가 0이면 성공으로 간주하지 않는다. 출력의 `running N tests`를 확인하고 다른 필터 또는 전체 lib test로 보완한다. DB contract test는 외부 DB 환경 요구 여부와 skip 상태를 결과에 명시한다.

### PostgreSQL TLS·단절·복구

```bash
npm run test:database-tls-contract
```

로컬 Docker 엔진(Unix socket 또는 Windows named pipe), OpenSSL, Rust가 필요하다. `postgres:16` 이미지가 없으면 내려받는다. 일반 contract 3개와 독립 프로세스의 TLS/재접속 2개를 실제 실행한다. PostgreSQL `Require`의 신뢰 CA 성공, hostname·CA 불일치/비TLS/잘못된 암호 거부, backend socket 종료와 서버 SIGKILL/재시작 후 같은 pool 복구를 확인한다. 단절 중 pending·소유 원장·risk 읽기와 쓰기·JSON 복귀가 오류를 반환하고 로컬 정상본/백업이 바뀌지 않는지도 검증한다.

실행마다 `.cache/database-tls-contract/<UUID>/`에 임시 CA와 `localhost` 인증서를 생성하고, loopback에만 포트를 공개하는 고유 이름·소유 라벨의 컨테이너 2개를 사용한다. `PG*`/`KISAT_PG_*` 환경을 정리한 자식 프로세스에 합성 자격증명·빈 pgpass·임시 CA만 전달한다. 실제 앱 설정·OS keychain·계좌를 사용하지 않는다. 종료 시 소유 라벨을 검증해 컨테이너/볼륨과 개인키를 정리하고 `result.json`을 남긴다. 프로세스 강제 종료로 정리가 중단되면 이 UUID의 이름·라벨을 먼저 대조한 뒤 해당 fixture만 정리한다.

환경 변수를 지정하지 않은 일반 `cargo test`에서는 서버 contract가 skip되므로 TLS 검증으로 집계하지 않는다. 릴리스 `database-contract` job은 위 명령을 실행하며 성공 marker와 실제 실행 건수를 확인한다. MariaDB 버전 matrix와 실제 운영 서버 검증은 이 fixture의 범위 밖이다. 설정 복구 집중 테스트는 marker 삭제/동기화 실패 후 재생성과 실행 중 recovery latch 유지, 새 파일의 post-rename 원복, 취소·재저장 차단을 합성 실패 주입으로 확인한다.

## 5. Playwright

설정은 `playwright.config.ts`, spec은 `tests/e2e/`에 있다. Vite를 `127.0.0.1:1430`에서 띄우며 현재 spec은 mock transport를 이용한 설정 DB와 전략 스크롤바 회귀를 다룬다.

```bash
npm run test:e2e
npx playwright test tests/e2e/settings-database.spec.ts
npx playwright test tests/e2e/strategy-scrollbar.spec.ts
```

UI 변경과 직접 관련된 spec을 먼저 실행한 뒤 범위가 넓으면 전체 E2E를 실행한다. 실제 Tauri IPC·provider 주문 E2E로 오인하지 않는다.

## 6. 구조·생성물 검사

할일 상태는 `docs/todo.md`가 원본이며 `npm run todo:render`로 `docs/todo.html`을 생성한다. `npm run check:todo`는 최신 여부, `node --test scripts/render-todo.test.mjs`는 열린 항목 보존·필터·검색·모바일·JavaScript 없는 읽기와 마크업 escaping을 실제 Chromium에서 검증한다. 후자는 CI e2e job에서 Chromium 설치 후 실행한다.

```bash
npm run check:project-map
npm run project-map:update
npm run check:graphify
npm run graphify:refresh
```

- `project-map:update`: 파일 추가·이동·삭제 또는 모듈 책임 변경 때 `docs/project-map.md` 생성 블록을 갱신한다. 생성 전 사용자 diff를 확인한다.
- `check:graphify`: 구조 리팩터링 결과의 공유 그래프 freshness 확인용이다.
- `graphify:refresh`: 심볼 삭제/이동, 공용 helper 추출, 모듈 재편 등 **구조 리팩터링 때만** 실행한다. 문서·기능 변경마다 생성물을 흔들지 않는다.
- 공용 타입/IPC/주문/리스크 변경은 다중 참조 누락 위험이 있을 때 Serena를 선택 사용한다. Serena 사용 여부가 테스트를 대체하지 않는다.

## 7. Provider·릴리스 검사

`package.json`에 존재하는 명령만 사용한다.

```bash
npm run verify:toss-openapi
npm run release:dry
npm run verify:release-artifacts -- <artifact...>
npm run build:app:debug
npm run build:app
```

- Toss spec 검증은 실제 endpoint/operation drift 확인용이지 계좌 주문 테스트가 아니다.
- `release:dry`를 제외한 version/release 명령은 파일·Git 상태를 변경할 수 있으므로 별도 승인 후 실행한다.
- Tauri build는 플랫폼 의존성과 시간이 크므로 릴리스 또는 통합 변경에서 선택한다.
- 실제 주문, 자동매매 시작, 비상정지 해제는 harness가 아니며 자동 검증에 포함하지 않는다.

## 8. CI와 로컬의 대응

`.github/workflows/release-gate.yml`은 다음 job을 실행한다.

- frontend: npm ci, lockfiles, TypeScript, FSD, project map
- rust: Cargo metadata/check/lib test with lock
- e2e: Chromium Playwright
- toss-openapi: Toss spec 검증

로컬 완료 보고에는 CI가 나중에 수행할 것이라고만 쓰지 말고, 변경 범위의 최소 게이트를 직접 실행한 결과를 남긴다.

## 9. 결과 보고 형식

```text
검증:
- PASS `npx tsc --noEmit`
- PASS `cargo test --workspace preflight --locked` (N tests)
- NOT RUN `npm run test:e2e` — 문서-only 변경
- NOT RUN 실거래/모의주문 — 안전 경계에 따라 의도적으로 미실행

잔여 위험:
- 실제 provider 응답/장중 체결은 검증하지 않음
- DB contract test는 <환경 사유>로 <skip/not run>
```

실패, warning, 0-test, 환경 blocker를 성공으로 축약하지 않는다.

LTH 역사 시각/실시간 경로 분리 변경은 `cd src-tauri && cargo test --locked --test lth_replay_clock`로 production library(단위 테스트 clock 대역 없음)의 입력 시각·blackout·청산·미래 봉 불변을 검증한다. generic LTH 거부는 lib preparation 테스트와 오프라인 연구 runner의 `rejectedRuns`를 함께 확인한다.

P1-05 일봉/분봉 분리는 `cd src-tauri && cargo test --locked --lib buffer_tests`와 lib preview/assessment 테스트에서 일봉의 분봉 지표·반동·볼린저 혼입 방지, snapshot 교체, 포지션 보존, 독립 버퍼 상한, 일봉 전략/체결 미호출 및 유효 분봉 0개의 평가 불가를 확인한다. UI는 `npx playwright test tests/e2e/strategy-scrollbar.spec.ts tests/e2e/strategy-replay-assessment.spec.ts`로 일봉/legacy 성과·A/B 숨김, 분봉 표본 경고와 assessment 보존, 기존 generic 성과 표시를 함께 검증한다. synthetic fixture는 시장 coverage 검증을 대체하지 않는다. 실제 2026-07-06~10-02 SOXQ 분봉을 보관하고 시간대·DST·세션·기간 coverage 검증 후 별도 재평가해야 P1-05 전체 완료다.


## P2-14 체결 모델 회귀

- `cargo test --workspace --lib simulation --locked`: 다음 관측 시가·이전 ATR·현금/비중·부분매도 원가·갭·동봉 순서불명/손절우선·마지막 미체결·기본 수치·분할 공급 계약.
- `cargo test --workspace --lib generic::tests --locked`: 실제 전략 피드백·미래 봉 불변·고정 QQQ2026Q3 두 비용/두 체결 모델의 실제 청산 경로.
- `cargo test --workspace --test lth_replay_clock --locked`: production library의 Open/Signal/Close·다음 시가 원가·마지막 미체결, 기존 입력 clock/session/미래봉 불변.
- `npx playwright test tests/e2e/strategy-execution-model.spec.ts`: 가정 입력→전송·이전 응답 무효화, 시점/미체결/불명경로·legacy A/B·모바일·LTH 일봉 진단 유지. 인접 ATR·assessment·scrollbar 또는 전체 UI 회귀를 함께 확인한다.
- [새 연구의 재현 절차](research/replay-execution-20261018/report.md)는 동결 입력과 직전 커밋 생산 라이브러리의 1,224회 호환 비교, QQQ 두 전략×두 비용×두 모델과 같은 규칙 benchmark를 구분한다. 이 비교는 실제 주문·장중 체결 검증이 아니다.


## P2-15 비용·현금흐름·손익 분해 회귀

계약 정본은 [IPC 문서](ipc-commands.md)의 v11 회계·비교 항목이다. 아래 검증은 합성 자료·보관 연구 입력과 mock API를 사용하며 실제 브로커 수수료·배당·주문 실행을 검증하지 않는다.

- `cargo test --workspace --lib simulation --locked`: 방향별 비용·최소 수수료의 실제 청산 전망, 부분 수량/매수 비용/FX 배분, 손실 청산·손절 예외, 실현+잔여+배당의 장부 일치, 원화 현금과 native 원가 유지, 관측 평균/최대 노출, 보고 전용 종료 청산과 비교 정책을 확인한다.
- `cargo test --workspace --lib generic::tests --locked`: 생산 preview의 분봉 alias 정규화와 실제 평가 봉 일치, 누락 봉·국내 FX 거부를 확인한다. 시나리오 범위·중복·배당 조정 중복은 simulation 테스트에서 확인한다. `cargo test --workspace --lib strategy_preview --locked`로 기존 LTH 일봉 진단·분봉 공통 동작을 확인한다. LTH 일봉의 시나리오 명시 거부는 소스 감사로 확인했으며 실제 provider를 호출한 API 검증은 실행하지 않았다. 필터의 실제 실행 건수를 검증하고 전체 lib test로 보완한다.
- `npx playwright test tests/e2e/strategy-accounting.spec.ts tests/e2e/strategy-execution-model.spec.ts tests/e2e/strategy-atr-readiness.spec.ts tests/e2e/strategy-replay-assessment.spec.ts tests/e2e/strategy-scrollbar.spec.ts`: 고정 소스에서 **47/47 통과**(2026-10-18). 새 회계9개와 기존38개로 입력→payload, 잘못된 JSON의 이전 결과 폐기/계산 차단, 국내 FX·64행/중복·배당 조정 중복 거부, 실현/전체 부호 차이·청산 표본·가상 청산, 비교 정책·A/B 집계 보존·legacy unknown·모바일 overflow·LTH 일봉 진단을 검증했다. 실제 Tauri IPC/브로커 E2E로 집계하지 않는다.
- `npx tsc --noEmit`, `npm run check:fsd`, `npm run build:web`와 구조·그래프 게이트를 현재 변경 범위에 맞게 수행한다. 보관 응답의 `accounting`/`comparison` 누락을 0 또는 잘린 거래 목록으로 복원하면 회귀다.

실제 비용·배당 자료의 출처, 가격 배당 조정 여부, 거래소 시간·세션·coverage는 별도 자료 검증이다. 겹친 기간·부분 분기·20건 미만 청산 표본의 비교를 독립 일반화 근거로 확대하지 않는다.

[P2-15 연구 재현·한계](research/replay-accounting-20261018/report.md)는 직전1192673 생산코드1224회 기본호환, 단일분기40개 합성 현금흐름 시나리오/120개 항등식과 기록fill 독립4540대사를 구분한다.


## P2-18 연속 재생·분할 공급 회귀

계약 정본은 [IPC 문서](ipc-commands.md)의 v12 연속 재생 항목이다. 기존 flat500봉과 전용 LTH를 유지한 별도 bounded 단일 요청이며 실제 provider의3년 coverage 인증은 별도다.

- `cargo test --workspace --lib strategy_preview --locked`: continuous 상한·unknown/flat 혼용·정규화된 history/경계 중복·역전·시나리오 오류, factory 이전 거부, 동일 경제 입력의 분할 무관 결과/hash와 계산 모드별 hash 차이를 확인한다. 단일 세션의 보유/native 원가·pending·손실/risk 보존, 미래 봉 불변과 기존 generic/LTH 회귀를 함께 확인한다. 필터의 실행 건수를 확인하고 전체 lib 결과와 구분한다.
- `npx playwright test tests/e2e/strategy-continuous-replay.spec.ts tests/e2e/strategy-accounting.spec.ts tests/e2e/strategy-execution-model.spec.ts tests/e2e/strategy-atr-readiness.spec.ts tests/e2e/strategy-replay-assessment.spec.ts tests/e2e/strategy-scrollbar.spec.ts`: 새 연속10개와 인접47개의 mock UI 조합이다. 2026-10-18 최초57개 실행에서 **56개 통과·1개 실패**였으며 같은 종가만 안내하던 기존 일봉 도움말 assertion을 선택 체결 모델·장중 미검증 문구에 맞게 고친 뒤 해당 **1개 재실행 통과**를 확인했다. 이를 단일57/57 실행으로 합쳐 기록하지 않는다. 전체 UI 최종 실행 결과는 작업 기록에서 별도로 확인한다.
- 연속 focused 항목은 합성3년 일봉→strict 이전252봉→500/500/96 공급 묶음→한 번의 연속 payload, 이전12봉 부족에도 평가 봉을 history로 옮기지 않음, 과거자료만 존재하는 종료 범위 차이/중간 간격7일 gate, 부분 오류·scope 변경·48페이지/4096봉 상한, 취소 후 늦은 응답 폐기, A/B 모드·원장 보존과 legacy unknown, 잘못된 날짜·모바일 overflow를 확인한다. mock API 통과는 실제 Tauri IPC·브로커 또는 거래소 휴장/누락 검증을 뜻하지 않는다.
- `npx tsc --noEmit`, `npm run check:fsd` 및 변경 범위의 구조/문서 게이트를 수행한다. `coverageVerified=false`를 유지하고7일을 검증된 거래소 달력으로 표시하지 않는지, 연속 앞/뒤70/30을 독립 훈련/검증으로 부르지 않는지도 확인한다.


## P2-06 Consecutive 신규 진입 확인

- `cargo test --workspace --lib sequence --locked`: 새 strict params/default0/custom legacy, 누적·정밀TR14·MA 경계, zero ATR·자료 부족·OHLC 미지원에서 보유 청산, 완료 게시·관측 보존과 재설정 카운터를 확인한다. 실제 실행 건수는 해당 작업 기록에서 확인한다.
- `cargo test --workspace --lib entry_filter --locked`: 생산 preview의 optional 진단, 잘못된 params/미지원 간격, 현재 OHLC 미래 정보 배제와 SPY 비용별 완료 매도 경로를 확인한다. `cargo test --workspace --lib live_history --locked`와 configuration 회귀로 pending/실패/날짜 전환/legacy warmup 경합과 저장 이전 검증을 확인한다.
- `npx playwright test tests/e2e/strategy-consecutive-entry-filters.spec.ts`: strict raw draft·실제 소수 키입력·저장 실패/재시도·활성 잠금·short/continuous payload·이전 응답 폐기·A/B 준비 진단/모두0/활성 legacy unknown·수량 ATR 분리·모바일 overflow를 확인한다.
- `npm run build:web`는 warnings0이어야 한다. 차트 라이브러리 chunk를 분리한 뒤에는 실제 vite preview의 전략/차트 회귀를 실행해 production 초기화를 확인한다.
- 동결 입력·생산 실행·독립 기록 산술은 [연구 재현](research/consecutive-filters-20261018/report.md)과 [작업 기록](tasks/consecutive-entry-filters-20261018.md)에 구분한다. 필터 통과를 체결이나 운용 우수성으로 해석하지 않는다.


## P2-03 Momentum 소유 피드백·교차

- `cargo test --workspace --lib momentum --locked`: 실제 소유 상태, inclusive 경계, strict warmup/무장, 후보 취소/동일0/pending 재시도, 부분/복원·재설정, 두 preview strict/분할·미래 prefix와 실제 저장 거부를 확인한다. 실행 건수와 전체 결과는 [작업 기록](tasks/momentum-feedback-20261020.md)에 구분한다.
- `npx playwright test tests/e2e/strategy-momentum-entry.spec.ts tests/e2e/strategy-consecutive-entry-filters.spec.ts`: 두 controls의 공유 십진 초안 해석·실제 키보드 입력, 도메인별 유효성, 잘못된 stored enum/null/string, level 미사용 간격, 저장 실패/재시도·활성 잠금, 두 preview·이전응답 폐기·AB·모바일을 확인한다. tsc/FSD/lockfiles/build와 전체 회귀를 함께 사용한다.
- 연구의 archived86a9829와 current v14 생산 라이브러리는 분리한다. 비모멘텀1122회 report/signals exact와 Momentum102창의 경제 경로/부적합 raw차이를 구분하며 실패한 모형을 강제로 기존 수치에 맞추지 않는다. [재현·사전조건·기각 결과](research/momentum-feedback-20261020/report.md)를 따른다.


## P2-04 Deviation 실제 소유·단독 후보

- `cargo test --workspace --locked deviation`와 `cargo test --workspace --locked live_feedback`: 실제 소유/교차 재시도·동일0/nextOpen/partial·strict params/저장·생산 preview·미래 prefix/두 체결 모델/분할 불변·live 전용 trend 준비·기록 나이/unknown/metadata·실제 exact owner coordinator 복원·manager 신호를 확인한다. 전체 Rust와 Clippy를 함께 실행하고 옛 stateless 분류의 Deviation 포함을 새 소유 계약으로 정정한다.
- `npx playwright test tests/e2e/strategy-deviation-policy.spec.ts`: 새4필드의 raw/stored 타입·경계·실제 키 입력·활성 잠금·저장/두 preview·A/B·모바일, 저장 실패/재시도와 일반/PC/LTH 지연 응답 뒤 새 초안 보존을 검증한다. scope epoch는 A→B→A의 과거 callback과 현재 초안을 구분한다. 인접 Momentum/Consecutive 및 전체 브라우저를 함께 확인한다.
- `python3 docs/research/deviation-policy-20261020/validate.py`: archived bca6601/현재 v15의 비이격도1122 exact·이격도102 delta,36연속/36분할/432별도 분기의 기록 장부를 대사한다. filled reason-only 차이와 실제 경제 경로 차이를 분리한다. [사전 선언·재현·기각과 한계](research/deviation-policy-20261020/report.md), [검증 기록](tasks/deviation-policy-20261020.md)을 따른다.


## P2-10 MeanReversion 단독 청산 후보

- `cargo test --workspace --locked strategy::`와 `cargo test --workspace --locked strategy_preview`로 내부 공통helper와 기존전략·준비·strict선택params·두모델/분할/미래prefix·frozenQQQ1주4경로를 확인한다. 실제coordinator/strictsave는 order::feedback와configuration 회귀 및 전체Rust로 확인한다. stop0/off차이·resetprice/intradayno-op·unknownage와cold/pending/partial/오염metadata를 포함한다.
- `npx playwright test tests/e2e/strategy-mean-reversion-policy.spec.ts`: raw/stored type·실제정수키입력·stop0의미·legacy/default·활성잠금·저장실패재시도·두payload/A-B·late결과/저장초안·모바일을 확인한다.1분history무사용한계문구를포함한최종focused와전체UI를구분기록한다.
- `python3 docs/research/mean-reversion-policy-20261020/validate.py`: 고정입력·유일factorial·partition대응·signalClose→nextObservedOpen·방향별slippage·기록장부/노출/기말/경로비용참조를 검증한다. [사전단계와후속탐색·재현·한계](research/mean-reversion-policy-20261020/report.md), [전체게이트](tasks/mean-reversion-policy-20261020.md)를 따른다. maxHold63미발동을 효과 없음으로 확정하지 않고 후속21관측은 사후탐색 단계로 구분한다.


## P2-01 이동평균 교차 진입 필터

- `cargo test -p kis-auto-trade ma_cross` 및 관련 configuration/preview focused: 이전 완료 일봉·현재봉 미포함, 교차 소비·Sell 비차단·재조회/실패/재설정·strict 입력과 분봉 거부를 검증한다.
- `cargo test -p kis-auto-trade consecutive`와 공유 completed_daily 계약: pure helper 이동 뒤 갭/분수/zeroAtr·이전종가평균 의미를 보존한다.
- `npx playwright test tests/e2e/strategy-ma-cross-filters.spec.ts`: 네 raw/stored 입력·저장/재조회/실패 재시도·preview/A-B·진단 없음·활성 잠금·모바일을 mock으로 검증한다.
- [사전 설계](research/ma-cross-filters-20261020/design.json)와 production runners: archived327739a/default1224호환, Stage A36연속/36분할/432분기와 사후 ATR2/폭10 추가 최종48연속/48분할/576별도분기를 구분해 대조한다. Python은 기록 fill/cash/노출을 정산하고 신호를 재생성하지 않는다. [작업 기록](tasks/ma-cross-filters-20261020.md).

## LTH 분봉 시각 진단 (2026-10-22)

- `cargo test --offline -p kis-auto-trade minute_timing`: pure20개와 command5개의 합성 fixture. RFC3339/compact·DST·동일순간 alias/역순·barClose/barStart/unknown·초 경계·휴일 미인증·내부 gap/야간/주말·512행/64바이트/8일/6예시·asOf를 확인한다. command fixture는 기존 signals/backtest/hash와 실제 비어있지 않은 신호·체결을 비교하며 daily skipNone/empty notEvaluable을 확인한다.
- `npx playwright test tests/e2e/lth-minute-timing.spec.ts tests/e2e/strategy-replay-assessment.spec.ts`: 현재/저장/복원·다른봉표기 비교·legacy/미지원/손상 컬렉션·빈자료/일봉 성과숨김·모바일과 compact 날짜수를 mock으로 검증한다. fixture는 LTH lookAheadSafe=false를 유지한다.
- 공용 timestamp 구문 추출은 기존 generic 정규화·회계 시나리오 정렬 회귀와 전체 Rust 검증으로 확인한다. 입력 시계·신호·정산은 기존 `lth_replay_clock`을 함께 통과한다. 실제 provider/3개월 coverage 검증으로 확대하지 않는다. [결과와 한계](tasks/lth-minute-timing-20261022.md).

## RSI 소유·선택 진입/청산 (2026-10-22)

`cargo test --offline -p kis-auto-trade rsi`는 단순RSI·기존교차·strict옵션·실제수량/원가·부분/미체결·나이/metadata·범위/재설정·분할을검증한다. `npx playwright test tests/e2e/strategy-rsi-policy.spec.ts`로원형입력·저장/재조회·두preview/A-B·손상값·late response·저장중새초안·모바일을mock으로확인한다. 전체Rust/UI와지도/그래프를함께검증한다.

연구 [설계](research/rsi-policy-20261022/design.json)·[runner](research/rsi-policy-20261022/runner.rs)·[정산](research/rsi-policy-20261022/validate.py)은기존동결가격과실제Rust preview만사용한다. baseline978640c flat1224/continuous6/reset72, current비RSI1122exact와RSI102변화·연속30/분할30/별도분기360을구분한다. 결과와한계는[작업](tasks/rsi-policy-20261022.md)·[연구](research/rsi-policy-20261022/report.md)에서확인한다. 실제계좌/API/주문검증으로확대하지않는다.
