# KISAutoTrade 검증 Harness 지도

> 원칙: 가장 작은 빠른 검증부터 실행하고, 위험 반경에 따라 확대한다. 어떤 명령도 실거래 안전성이나 전략 수익성을 증명하지 않는다. 외부 계좌/API를 사용하는 수동 검증은 명시적 승인 없이는 실행하지 않는다.

## 1. 환경 기준

- Node.js `>=20`, npm `>=10` (`package.json`, `.nvmrc`)
- Rust `>=1.78`, workspace member `src-tauri` (root `Cargo.toml`)
- 재현 가능한 의존성 검증은 npm `ci`와 Cargo `--locked`를 우선한다.

```bash
npm ci --ignore-scripts
cargo metadata --locked --format-version 1 --no-deps
```

기존 작업 환경에서 문서/소규모 변경만 검증할 때 불필요한 reinstall은 생략할 수 있다.

## 2. 기본 정적·단위 게이트

| 목적 | 실제 명령 | 근거 |
|---|---|---|
| TypeScript 타입 | `npx tsc --noEmit` | release-gate frontend |
| Frontend FSD import | `npm run check:fsd` | `package.json` |
| Rust 전체 target check | `cargo check --workspace --all-targets --locked` | release-gate rust |
| Rust lib tests | `cargo test --workspace --lib --locked` | release-gate rust |
| lockfile 일관성 | `npm run check:lockfiles` | release-gate frontend |
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

## 5. Playwright

설정은 `playwright.config.ts`, spec은 `tests/e2e/`에 있다. Vite를 `127.0.0.1:1430`에서 띄우며 현재 spec은 mock transport를 이용한 설정 DB와 전략 스크롤바 회귀를 다룬다.

```bash
npm run test:e2e
npx playwright test tests/e2e/settings-database.spec.ts
npx playwright test tests/e2e/strategy-scrollbar.spec.ts
```

UI 변경과 직접 관련된 spec을 먼저 실행한 뒤 범위가 넓으면 전체 E2E를 실행한다. 실제 Tauri IPC·provider 주문 E2E로 오인하지 않는다.

## 6. 구조·생성물 검사

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
