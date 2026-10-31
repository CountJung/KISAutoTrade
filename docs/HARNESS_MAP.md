# KISAutoTrade 검증 Harness 지도

> 원칙: 가장 작은 빠른 검증부터 실행하고, 위험 반경에 따라 확대한다. 어떤 명령도 실거래 안전성이나 전략 수익성을 증명하지 않는다. 외부 계좌/API를 사용하는 수동 검증은 명시적 승인 없이는 실행하지 않는다.

## 1. 환경 기준

- Node.js `^20.19.0 || >=22.12.0`, npm `>=10` (`package.json`); 권장 검증 버전은 `.nvmrc`의 Node `24.22.1` LTS다. 세 CI workflow도 `node-version-file: .nvmrc`로 같은 버전을 사용한다.
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
| Vite config 타입 | `npx tsc --noEmit --project tsconfig.node.json --tsBuildInfoFile node_modules/.cache/kis-vite-config.tsbuildinfo` | 앱 src 검사와 별도 |
| 격리 WebView bootstrap | `node --test scripts/webview-smoke/bootstrap.test.mjs` | Node VM 합성 계약; GUI 실행 없음 |
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

설정은 `playwright.config.ts`, spec은 `tests/e2e/`에 있다. 로컬 기본 실행은 Vite dev `127.0.0.1:1430`을 사용한다. 직전 2026-10-29 기준은349개였으며 이후 UI/UX 추가 spec도 같은 production 프로젝트에 포함된다. 추가 집중 검증·최종 전체 실행 수는 [UI/UX 회차 기록](tasks/uiux-improvements-20261030.md)과 [키보드 회차 기록](tasks/keyboard-improvements-20261031.md)에 각 실제 결과로 남긴다. CI는 먼저 `npm run build:web`를 수행하고 두 서버·프로젝트를 사용한다. 기존 UI317개와 새 UI/UX spec은 `chromium`의 production preview `127.0.0.1:1431`, 소스 모듈 가로채기를 요구하는 `balance-events.spec.ts`, `toss-commission-ratio.spec.ts`, `strategy-deviation-policy.spec.ts`의 기존32개와 `keyboard-search-scope.spec.ts`는 `chromium-source-harness`의 Vite dev `127.0.0.1:1430`에서 검사한다. 기존27파일·349개의 검사 범위와 새 회귀를 중복·누락 없이 유지한다.

spec은 mock transport를 사용하며 실제 Tauri IPC·provider 주문 검증과 구분한다. 테스트 30초·assertion 5초 제한은 유지한다. CI는 list와 HTML reporter를 함께 생성하고 실패 trace·screenshot을 보존한다.

```bash
npm run test:e2e
npx playwright test tests/e2e/settings-database.spec.ts
npx playwright test tests/e2e/strategy-scrollbar.spec.ts
```

UI 변경과 직접 관련된 spec을 먼저 실행한 뒤 범위가 넓으면 전체 E2E를 실행한다. 실제 Tauri IPC·provider 주문 E2E로 오인하지 않는다.

### 키보드 조회 회귀

`tests/e2e/keyboard-search-contract.spec.ts`는 Trading/Strategy/LTH×KR/US×KIS/Toss의 일반 Enter·조합/229·지역 유효성·1자 US·기존6자리 경로·검색 버튼 이름을 합성 응답으로 대조한다. 국내 Enter는 정지된 clock에서 즉시 조회를 요구해 기존 자동 debounce와 구분한다. 인접 회귀는 `uiux-keyboard-selection.spec.ts`와 `trading-form-contract.spec.ts`다. fixture 파일은 별도 검사 건수가 아니다.

로컬 실행은 프로젝트 .env를 읽지 않는 envDir:false 설정·비민감 환경 allowlist·fresh mock Chromium을 사용한다. 외부/WS·unknown·금융/설정 쓰기를 차단하며 Toss open-orders의 정확한 합성 POST 읽기 예외는 금융 변이로 해석하지 않는다. 실제 실행 명령·red/green·수치·증거·미검증은 [5개 순차 구현 기록](tasks/keyboard-improvements-20261031.md)이 정본이다. 실제 OS IME/WebView/스크린리더·실계좌 검증을 대체하지 않는다. 인접 주문폼5개는 별도 브라우저 합성 주문 제출 회귀이며 실제 주문0이다. 신규 검색14개의 쓰기0과 이 명시적 mock 제출을 구분한다.

`keyboard-search-race.spec.ts`는 production mock에서 지연 응답·반복 입력/버튼·stale 선택/오류/fallback·입력/ABA·시장/SPA 이탈·재시도를 대조한다. `keyboard-search-scope.spec.ts`는 source 프로젝트에서 실제 router/Page/QueryClient를 유지한 합성 appConfig 재조회로 공개 scope 필드와 ABA를 구분한다. 두 spec은 전용 `keyboardSearchRaceFixture.ts`의 ticket header·response.finished 관측을 사용한다. 정확한 실행 건수·부분/최종 결과는 회차 정본에서 확인하며 실제 provider 요청 취소·계좌 검증으로 확대하지 않는다.

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

- frontend: npm ci, lockfiles, 앱 및 Vite config TypeScript, 합성 WebView bootstrap VM, FSD, project map, todo freshness
- rust: Cargo metadata/check/lib test with lock
- e2e: Chromium 설치, todo renderer 테스트, `npm run build:web`, 기존 UI production317개와 추가 UI/UX spec, source dev32개의 Playwright 검사. 실패하면 `playwright-report/`와 `test-results/`를 artifact로 7일 보관한다.
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

## P2-07 돌파 실패 검증

전략·driver·완료 history/manager·owner/configuration의 focused 테스트와 전체 Rust를 확인한다. 가격 관측/완료 고가·미래 HLCV 불변, TR gap/0/마지막15, volume unknown/실0/평균0/ratio 경계, atomic1회 forwarding, nextOpen0→첫실fill anchor 승격·거절/부분/v1v2·age/peak·scope/저장 실패를 검증한다.

`npx playwright test tests/e2e/strategy-failed-breakout-policy.spec.ts`는 기본·엄격 입력/쌍설정·두 preview/A-B·준비 진단·일봉 필터의 분봉 거부/보호만 분봉 허용·재시도/늦은 응답/새 초안·모바일을 mock으로 검증한다. [설계](research/failed-breakout-policy-20261022/design.json)의42연속/42동일경로분할/504독립분기와 부모·독립 정산은 [연구](research/failed-breakout-policy-20261022/report.md)·[작업](tasks/failed-breakout-policy-20261022.md)에 기록하며 실제 주문/provider 검증으로 확대하지 않는다.

### P2-08 강한 종가

`cargo test --offline strong_close`의 실수량/원가/잔량·기본 stop0·복원/거절/부분/재설정·preview prefix/분할과 manager 새날/같은 날/실패/late legacy·stale scope/generation fixture를 확인한다. dated 기본 hook의 타 전략1회 전달도 검증한다. `tests/e2e/strategy-strong-close-policy.spec.ts`에서 raw/쌍·저장/두 preview/A-B·daily-only·늦은 응답/취소·활성 잠금/새 초안·모바일을 확인하고 전체 Rust/UI·타입/빌드/FSD·graph/map/TODO 게이트를 유지한다. [사전 설계](research/strong-close-policy-20261022/design.json)는30연속/30분할/360분기이며 실운용·미관측 OOS로 승격하지 않는다.

### P2-09 변동성 확장

`cargo test --offline volatility`로 이전N범위/당일OHLC·0범위/평균0/엄격 경계·실수량/원가·부분/거절/기원/pending·재설정/메타데이터·scope/date/freshness fixture를 확인한다. 두 preview의 current 제외/미래 prefix·분할·진단 빈도와 strict 입력을 함께 검증한다. 변동성 전용 focused Playwright는 저장/두 preview·현재/A-B·raw/쌍·과거/손상 진단·활성 잠금·지연 응답/저장 중 초안·모바일을 mock으로 확인한다. 전체 Rust/UI·타입/빌드/FSD/graph/map/TODO 게이트를 유지한다. [설계](research/volatility-expansion-policy-20261022/design.json)의36연속/36분할/432분기 정산은 [작업](tasks/volatility-expansion-policy-20261022.md)에 기록하며 실제 장중 최초 발동/체결은 검증된 장중 자료 확보까지 열린 조건이다.


## P2-11 추세 필터 v22

- `cargo test --offline --workspace --lib trend_filter --locked`: 소유/나이/peak/strict checkpoint·실제 feedback caller·저장·두 preview·분할/prefix/guard fixture. 당시 전체 lib는789개였다. 현재 전체 실행은 상단 기본 게이트와 최신 회차 기록을 따른다.
- `npx playwright test tests/e2e/strategy-trend-filter-policy.spec.ts tests/e2e/strategy-strong-close-policy.spec.ts tests/e2e/strategy-volatility-expansion-policy.spec.ts --workers=1`: 새14개와이웃/조립회귀70개, 총84개mock검증이다. 실제 주문/계좌/API 검증이아니다. 당시 Page 정책조립 추출에서 전체322개도 실행했다. 현재 전체 UI는 상단349개 구성을 따른다.
- `python3 docs/research/trend-filter-policy-20261023/validate.py`와 `independent-audit.py`: 고정설계/동결입력·30연속/360분기압축원장을재정산한다. Rust실행기의30대체분할검사와기록원장감사를구분한다. 실행기/사전설계/의존성한계는같은폴더보고서를참조한다.
- loopbackmock서버가EPERM이면test실패를코드통과로간주하지않고허용된환경에서전체를다시실행한다. 운영provider/계좌또는실제자동매매로대체하지않는다.


## P2-13 가격조건 생산 계약 (2026-10-29)

`cargo test --locked --lib pc_live_`는 실제 PC→공유제출/loopback→누적체결→exact owner 피드백6개를 검증한다. `cargo test --locked --lib pc_replay_tests`는 native 평균가/FX·손절·시장일/DST·실제수량/설정수량·overlay·두비용/분할9개를 검증한다. 계좌 전체 preflight 값과 소유 원장, 접수와 체결, 잔량 상태 보존과 실제 청산 성공을 구분한다.

[고정 설계와 재현](research/price-condition-contracts-20261029/report.md)의6연속+6대체분할+72별도분기 및 기록체결 정산은 기존 일봉 입력 재분석이며 실제 장중 손절 효능 검증이 아니다. 전체 Rust와 strict clippy, 문서·지도·그래프 게이트 결과는 [회차 기록](tasks/immediate-backlog-20261029.md)에 보존한다.


## P2-16 동일 장부 구간 진단·별도 walk-forward

- `cargo test --offline --lib trading::simulation`와 `cargo test --offline --lib commands::strategy_preview`: 경계잔량/pending·부분매도·FX/배당/원가·0/1봉/100%·양수절반/보합/음수·분할불변·기존TR/ATR/LTHgate를 확인한다. 최종 전체lib831 및 strict clippy 통과 근거는 [회차 기록](tasks/immediate-backlog-20261029.md).
- production `build:web` 후 `CI=1 npx playwright test tests/e2e/strategy-phase-diagnostics.spec.ts tests/e2e/strategy-continuous-replay.spec.ts tests/e2e/strategy-replay-assessment.spec.ts`:20개. 회계·ATR·scrollbar인접3spec:37개. A/B진단 보존·legacy/손상 미확인·375px overflow·LTH성과숨김을 검증한다.
- [연구 재현법](research/phase-walk-forward-20261029/report.md)은 동결일봉/생산API와 독립 기록산술을 사용한다. 별도reset평가·사전고정 설정을 같은장부70/30과 구분하고 이미본자료를 새OOS로 부르지 않는다.


## P3 대형 모듈 책임 분리

Trading/LTH의 state/effect/closure는 각 `use*Model`을 parent에서 한 번 호출해 보존하고 표시는 child panel로 분리한다. Toss는 기존 facade/command macro re-export와 서버caller 계약을 유지한다. [동등성/행수/회차검증](tasks/immediate-backlog-20261029.md).

`CI=1 npx playwright test tests/e2e/trading-form-contract.spec.ts tests/e2e/strategy-scrollbar.spec.ts tests/e2e/lth-minute-timing.spec.ts tests/e2e/strategy-configuration-errors.spec.ts tests/e2e/strategy-replay-assessment.spec.ts tests/e2e/toss-commission-ratio.spec.ts`를 production build 후 실행한다.47개(46production+1sourceharness) mock검증이며 실제주문/계좌연결아니다. Rust 전체lib와 strictclippy/check/fmt,TS/FSD도확인한다.


## HTTP 의존성 전환 계약

`cargo test --offline --locked --lib http::tests`: 합성 HTTP/TLS10개로 query/form/JSON/header·쿠키/redirect·timeout/proxy·오류·응답 유실 POST 재제출 없음·TLS1.2/1.3/SNI/ALPN·CA/hostname 거부를 검사한다. `http/testdata`는 공개 합성 fixture이며 OS trust에 등록하지 않는다. caller는 `crate::http::client_builder()`로 기존 Mozilla roots/ring을 유지한다. 실제 provider 접속·MSRV 실행·다른 플랫폼 검증과 구분한다. [5A 회차 근거와 한계](tasks/immediate-backlog-20261029.md).


## WebSocket 의존성 전환 계약

`cargo test --offline --locked --lib api::websocket::tests`: 생산 private transport helper의 합성 WS/WSS14개로 승인/구독 wire·수신·자동Pong·Close echo·오류상태·명시재구독·4KiB버퍼/20KiB메시지·TLS1.2/1.3 신뢰/거부를 확인한다. `http/testdata`의 공개 fixture만 재사용하고 실제 KIS나 OS native-root 연결을 실행하지 않는다. 기존 native roots/ring·금융 payload를 유지한다. 자동 재연결 검증으로 해석하지 않는다. [5B 회차 기록](tasks/immediate-backlog-20261029.md).


## Axum router 계약과 격리 fixture (2026-10-29)

`cargo test --offline --locked --lib server::`는 기존6개와 신규18개, 총24개다. production `server/router.rs`의 70 URL/74 명시 method binding, 13개 capture, POST 토큰 거부140건, HEAD/Origin/OPTIONS·405/Allow·SPA fallback, JSON/query/path 거부, IPC와 REST view 값·케이싱, KIS loopback4경로를 검사한다. 0.7 baseline→0.8.9/tower0.5→0.8.9/tower0.7.1에서 각각 통과했다. 최종 trailing JSON은 framework의400 거부를 단정한다.

fixture는 임시 경로와 합성 token/profile만 사용하고 `DatabaseManager::isolated_json_for_tests`로 config·키체인·전역 DB 설치를 피한다. 전체 route 등록/차단 검사가 모든 handler의 성공 경로 검증을 뜻하지 않는다. Toss는 미설정/없는 profile 오류, 웹 설정 저장·종목 목록 갱신·거래 시작은 차단 또는 입력 거부까지만 실행한다. 실제 provider·운영 앱·계좌 접근을 추가하지 않는다. tower-http cors feature는 선언돼 있지만 실제 CorsLayer 사용은 없으며 기존 자체 Origin 정책을 유지한다. [회차 근거·제약](tasks/immediate-backlog-20261029.md).


## React19·MUI9 UI 계약 (2026-10-29)

`CI=1 npx playwright test tests/e2e/ui-dependency-contract.spec.ts`는 production preview의 합성 UI5개를 검사한다. dialog focus·Escape 복귀, Select 키보드·mock 저장, 좁은 Dashboard 표, painted chart의 theme/resize/SPA재진입을 다룬다. external origin·browser warning/error와 숨김 상태의 차트 중복을 검사한다. Drawer transition 때 aria-hidden을 CSS display와 혼동하지 않으며 정상 접근 가능/route/닫힘을 기다린다. 강제 클릭으로 동기화를 대체하지 않는다.

Switch는 MUI7 이상에서 role=switch이며 실제 Checkbox와 구분한다. MUI9 Alert는 variant/severity의 분리된 정확 class 토큰과 내용으로 검증한다. React vendor는 react·react-dom/client·JSX runtime 진입점을 포함하고 최종 production chunk의 초기화/순환과 경고를 함께 확인한다. 단계별·전체 실행과 actual native runtime을 구분한다. [회차 기록](tasks/immediate-backlog-20261029.md).


## Vite8 및 독립 macOS WebView 확인

Vite6→7→8/plugin6 전환에서는 명시한 Safari13/Chrome105 target과 production/debug 네 조합, Rolldown 다섯 vendor 그룹·Oxc, cold/warm source harness와 production UI를 구분해 확인한다. syntax target 빌드는 실제 지원 OS/브라우저 실행을 뜻하지 않는다. 단계별 검증과 현재 결과는 [회차 기록](tasks/immediate-backlog-20261029.md)의 DEP-P3-06에 둔다.

`node --test scripts/webview-smoke/bootstrap.test.mjs`는 네 개의 순수 Node VM 계약이다. `node scripts/webview-smoke/driver.mjs --build`는 macOS에서 동결 dist를 복사하고 독립 Wry/Tao workspace를 offline으로 빌드한다. Cargo registry cache가 필요하며 GUI 실행 명령은 없다. 격리 목적상 일회성 결과는 일반 `.cache/` 대신 고유 `/tmp/kis-webview-smoke-8-*/`에 둔다. `snapshot.json`, `isolation.json`, `build-result.json`의 소스·자산·실행 파일 해시를 확인한 뒤 별도로 승인된 CUA 실행만 수행한다. `launched:false`는 빌드 시점 기록이다. 같은 실행 파일을 재실행하면 고정 output 이름을 재사용하므로 이전 PNG가 남을 수 있다. 각 실행을 별도 보관하고 현재 report의 snapshotCount·파일 시각·sidecar와 대조한 유효 산출물 manifest를 둔다. 이전 회차의 더 큰 번호 파일을 새 캡처로 집계하지 않는다.

이 창은 IPC와 생산 앱 crate 없이 비영속 WK store·고정 합성 transport를 사용한다. 알려진 GET/순수 preview만 허용하고 저장·주문·외부 전송은 차단한다. 읽기 전용 화면 상태 변화 시 자기 WebView만 최대12회 캡처하며 `snapshot-NN.json`의 성공과 PNG를 함께 확인한다. report의 rootChildren·canvas paint·오류·denied·overflow·정책 ledger가 실제 관측돼야 하며 null/누락을 통과로 해석하지 않는다. 실제 종료 후 프로세스 부재, 명시 재실행 뒤 marker의 initiallyAbsent/readbackMatches를 확인한다. bare Wry 성공은 원래 Tauri startup/IPC·운영 DB·계정·Windows·macOS 최소버전 검증이 아니다. CSP 설치와 실제 negative control 검증도 구분한다.


## SHA·오류 의존성 계약

`cargo test --offline --locked --lib dependency_contract_tests`는 생산 scope/profile/receipt/config 지문과 실제 DB scanner, 다섯 오류 타입의 고정 계약14개다. 기존 버전에서 입력 bytes·기대값을 먼저 고정하고 전환 후 같은 literal을 쓴다. sha2.11 Output은 LowerHex가 없으므로 기존 hex0.4.3의 소문자·leading-zero 출력을 사용한다. DB checksum fixture는 scanner와 입력/알고리즘 계약이며 production import/export roundtrip은 아니다.

`cargo test --workspace --lib --locked -- --skip storage::database_contract_tests`는 실제 DB 계약 module을 명시 제외한 lib 회귀다. 접속 환경 값은 조회하지 않고 자식에서 제거하며, filtered5를 성공한 DB 테스트로 세지 않는다. 실제 PostgreSQL/TLS fixture는 앞의 별도 명령과 사용자 결정 범위를 따른다. focused·통합 결과, 원본/호환 baseline·동등성·유지 판단은 [9회차 기록](tasks/immediate-backlog-20261029.md)에 보존한다. thiserror 전환에서는 Display/From/source/downcast와 확정거절/불명 결과를 함께 확인하고 HTTP trust·보안 예외를 변경하지 않는다.


## TypeScript native CLI와 오프라인 검증 설정

현재 프로젝트 compiler는 package.json/lock의 TypeScript7 native CLI다. 기존 `npx tsc --noEmit`과 별도 node config 검사 명령을 그대로 쓰며 구형 JS compiler API의 존재를 가정하지 않는다. API 소비가 필요해지면 실제 호출부부터 확인한다. 원문 source/config의 입력 집합과 defaults·OS별 native 준비를 함께 대조하고 compiler host와 배포 앱 최소 OS를 구분한다. [단계 비교·벤치·지원 범위](tasks/immediate-backlog-20261029.md)는10회차 기록을 따른다.

개인 환경 파일을 읽지 않는 임시 Vite 설정은 repo Git 제외 캐시에 두고 bare package와 명시된 source 확장자를 사용한다. 다음은 `.cache/toolchain-migration-20261029/offline-vite.config.mts`의 패턴이다.

```ts
import { defineConfig, mergeConfig } from 'vite'
import base from '../../vite.config.ts'
export default defineConfig(async (env) => mergeConfig(await base(env), { envDir: false }))
```

Vite의 절대 내부 index.js를 직접 묶으면 선택 의존성 경고를 만들 수 있고 source 확장자를 생략하면 향후 native config loader 안내가 남는다. 제품 target·chunk 기준·엄격성을 바꾸거나 경고 억제 환경 변수로 대체하지 않는다. 교정 이전 UI 통과와 후속 짧은 loader/build 검증의 시점·안내를 구분한다. 같은 resolved module·source·dist byte가 유지되면 안내 수정만으로 전체 UI를 반복하지 않는다.
