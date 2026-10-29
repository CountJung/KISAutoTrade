# KISAutoTrade — 남은 할일

> 갱신: 2026-10-29 · 현재 구현 근거는 [즉시 작업 회차 기록](tasks/immediate-backlog-20261029.md)과 완료 이력을 따른다.
> 우선순위는 P1 정확성/신뢰성 → P2 전략 연구 UX → P3 유지보수다. 로컬 구현 완료와 실제 provider·실자료 검증 완료를 구분한다.

## 대분류별 진행 현황

| 대분류 | 상태·완료 근거 요약 | 남은 최상위 항목 | 핵심 남은 일 | 사용자 결정·자료 |
|---|---|---|---|---|
| 주문·계좌 정합성 | 설정/체결 소유·실행 예약·불명 복구 UI의 로컬 검증 완료; 운영 검증 대기 | 2개: P1-SCOPE-01, P1-PROVIDER-01 | 동시 scope 범위 결정, 실제 원장·복구 대조 | 범위 결정 1·실사례 검증 1 |
| 전략 연구·해석 | 가격조건 계약·구간 진단·별도 회고 walk-forward 완료; 장중 효능 대기 | 2개: SOXQ-P2-13·09 | 가격조건/변동성 실제 장중 근거 | 장중 자료 2 |
| LTH 실제 장중 평가 | 일봉 평가불가·분봉 시각 진단 구현 완료; 실제 기간 평가 대기 | 2개: SOXQ-P1-05, SOXQ-P2-12 | 실제 3개월 원본/coverage·DST 검증 후 옵션 비교 | 같은 원본 자료 공유·검증 2 |
| DB 지원·호환 전환 | PostgreSQL 로컬 TLS 기록 존재; MariaDB/MSRV 정책 미확정 | 3개: P1-DB-01, DEP-P3-01·02 | 지원 matrix·최소 Rust 결정 후 SQLx/DB/RSA 검증 | 정책 결정 2; SQLx 착수 의존 |
| 구조·의존성 유지보수 | 대상 세 파일·HTTP 전환과 CI 완료; WS 로컬 검증·CI 대기 | 6개: DEP-P3-03~08 | HTTP/WS·서버·UI·빌드·도구 검토 | 독립 검토 가능; Xcode 보류 유지 |
| 릴리스 서명 | checksum/attestation·기본 게이트 완료; 실제 서명 미완료 | 1개: P3-SIGN-01 | 플랫폼·키 관리 결정과 서명 검증 | 사용자 결정·관리자 준비 1 |

**미완료 16개 = 자동 구현 7개 + 사용자 판단 4개 + 사용자 검증·자료 5개.** 각 상위 ID는 표에서 한 번만 센다. 하위 체크리스트와 완료 이력은 더하지 않으며 근거 없는 진행률은 산출하지 않는다. 자동 구현 7개 중 DEP-P3-02의 전환은 DEP-P3-01 결정이 선행한다.

[완료·경과 이력](completed-history.md) · [색상·필터 보기](todo.html) · [운영 계획](MASTER_PLAN.html) · [검증 지도](HARNESS_MAP.md)

이 Markdown이 현황표·상세 작업의 정본이다. 기존 HTML 보기는 열린 작업 카드·검색·상태 필터를 제공하며 상단 표는 이 문서에서 확인한다. 수정 후 `npm run todo:render`, 동기화 확인은 `npm run check:todo`를 실행한다.

- **🟢 자동 구현**: 별도 판단 없이 가능한 코드·fixture·연구 검증. 선행조건을 확인하며 어려움만으로 사용자 대기로 바꾸지 않는다.
- **🟠 사용자 판단 대기**: 제품 범위·기존 보류 해제·배포 키 등 명시적인 선택이 필요하다.
- **🟠 사용자 검증·자료 대기**: 검증 가능한 실자료·환경·별도 승인 범위가 필요하다. 독립적인 로컬 구현은 계속할 수 있다.

대기는 미완료다. 실제 연결·주문·전략 기본값 변경은 이 목록 정리로 승인된 것이 아니다. 각 항목의 완료 기준과 검증 범위를 구분한다.

## 1. 바로 진행할 코드·개선 작업

### P3 — 독립 검토 가능한 호환 업데이트

아래 현재→검토 버전은 2026-10-24 직전 읽기 검토값이며 최신 버전의 영구 고정값이 아니다. 각 작업 착수 시 공식 릴리스·레지스트리·보안 공지와 열린 Dependabot PR #13·#11의 현재 상태를 재확인하고 중복 변경을 피한다. [적용·보류 및 기존 검증 정본](security-mac-handoff-2026-10-06.md), [잔여 RSA·보안 정책 정본](release-security.md)을 따른다. 아래 작업은 모두 미완료이며 착수 시 검토값과 실제 지원 조합을 다시 확인한다.

권장 순서: **DEP-P3-01 → 02 → 03 → 04 → 05 → 06**, 기타 후보는 **07·08**에서 개별 판단한다. **DEP-P3-01의 MSRV·격리 DB 지원정책 결정은 DEP-P3-02 SQLx 전환의 실제 선행조건**이다. DEP-P3-03~08의 “다음/이후”는 검토 우선순위이며 앞 번호 전체 완료가 기술상 필수라는 뜻은 아니다. 각 항목의 실제 API·peer·toolchain 의존성이 확인되면 그 근거에 따라 순서를 조정한다. 우선순위는 이 P3 목록 내부 순서이며 기존 P1/P2를 대체하지 않는다. 검증은 금융 자격증명·실거래 연결 없이 합성/mock·격리 fixture로 한다. 운영 DB·거래 데이터·실행 중 서비스와 주문/취소/포지션·엔진 재시작·배포를 건드리지 않는다. Xcode 27.0→27.1은 사용자 보류 상태를 유지한다.

DEP-P3-01·02는 아래 선행 결정 구역에서 관리한다. 아래 03~08은 앞 번호 전체 완료를 기다려야 하는 일괄 차단 목록이 아니다. 버전 표기는 당시 검토값이므로 착수 시 공식 자료로 다시 확인한다.

- [ ] **🟢 자동 구현** **DEP-P3-03 — reqwest 0.12.28→0.13.5 및 tokio-tungstenite 0.24.0→0.30.0을 각각 전환한다.** 우선순위 3, DEP-P3-02 다음; 두 전환의 diff·검증은 분리한다.
  - **5A HTTP 로컬 완료:** reqwest 0.13.5의 `query`·`form`·`rustls-no-provider`와 공통 builder로 기존 Mozilla roots/ring/TLS1.2·1.3/HTTP1.1을 보존했다. 합성 HTTP/TLS10개·전체 Rust841 검증과 보안 잔여는 [회차 기록](tasks/immediate-backlog-20261029.md)을 따른다. 전체 ID는 아직 미완료다.
  - **5B 로컬 완료·CI 대기:** WS0.30의 Utf8Bytes·native roots/ring·4KiB read buffer와 합성 WS/WSS14개를 검증했다. 기존0.24에서 재현한 Close 응답·송신 실패 상태 정리도 수정했다. 자동 재연결은 추가하지 않았으며 명시 재구독만 검증했다. HTTP의 Release Gate43·Security50은 성공했고, 별도 WS 커밋의 CI 결과 확인이 남는다.
  - **완료 조건:** 합성 HTTP/TLS 서버에서 차트 query·헤더·JSON·쿠키·오류 처리를 확인하고 mock WebSocket에서 승인·구독·수신·ping/pong·종료·명시 재연결/재구독을 검증한다. 실제 KIS/Toss 인증 없이 Rust format/strict clippy/check/test/build와 해당 CI를 통과한다. [reqwest 변경](https://github.com/seanmonstar/reqwest/releases/tag/v0.13.0), [WebSocket 변경](https://raw.githubusercontent.com/snapview/tungstenite-rs/v0.30.0/CHANGELOG.md).

  - **착수 의존성:** 앞 번호는 검토 권장 순서다. 실제 API·peer·toolchain 의존성을 확인해 범위를 고정하며 DEP-P3-01 결정 대기를 이 항목의 일괄 차단으로 확대하지 않는다.

- [ ] **🟢 자동 구현** **DEP-P3-04 — axum 0.7.9→0.8.9와 tower-http 0.5.2→0.7.1의 서버 호환성을 검증한다.** 우선순위 4, DEP-P3-03 다음.
  - **확인/미검증 구분:** `server/mod.rs`의 `/:symbol`·`/:id`는 새 `/{symbol}`·`/{id}` 문법으로 바꿔야 한다. tower-http는 현재 CORS 사용 경로의 별도 후보이며 프로젝트에서 구체적 API 실패를 확인하지 않았다. 이미 존재하는 간접 tower-http 0.6.11을 직접 의존성 전환 완료로 취급하지 않는다.
  - **완료 조건:** 주문·계정 provider를 mock으로 격리하고 기존 모든 route의 경로/상태·인증 거부·CORS·오류 응답·Tauri IPC와 REST view 계약을 회귀 검증한다. Rust format/strict clippy/check/test/build 및 CI 통과. [axum 변경](https://github.com/tokio-rs/axum/blob/axum-v0.8.0/axum/CHANGELOG.md).

  - **착수 의존성:** 앞 번호는 검토 권장 순서다. 실제 API·peer·toolchain 의존성을 확인해 범위를 고정하며 DEP-P3-01 결정 대기를 이 항목의 일괄 차단으로 확대하지 않는다.

- [ ] **🟢 자동 구현** **DEP-P3-05 — React 18.3.1→19.3.1·MUI 6.5.0→9.5.0을 단계적으로 이전한다.** 우선순위 5, DEP-P3-04 다음.
  - **실제 코드 영향 확인:** 대시보드·전략의 MUI `Grid item xs`는 `size` API로, 전략 입력의 `InputProps`는 `slotProps.input`으로 이전 대상이다. React는 이미 createRoot/새 JSX 변환을 사용한다. React DOM·타입/ref·오류 처리·주변 라이브러리 호환은 미검증 후보이며 이번에 React 교체 실패를 확인한 것은 아니다.
  - **완료 조건:** React/DOM/타입과 MUI 관련 패키지의 지원 조합을 공식 가이드로 정하고 분리된 변경으로 타입·FSD·production build를 통과한다. 금융 mock E2E에서 전략/설정/대시보드·좁은 화면·입력/메뉴/버튼·테마·차트를 검증하고 Mac/Linux 결과를 구분한다. [React 이전](https://react.dev/blog/2024/04/25/react-19-upgrade-guide), [MUI 이전](https://mui.com/material-ui/migration/upgrade-to-v9/).

  - **착수 의존성:** 앞 번호는 검토 권장 순서다. 실제 API·peer·toolchain 의존성을 확인해 범위를 고정하며 DEP-P3-01 결정 대기를 이 항목의 일괄 차단으로 확대하지 않는다.

- [ ] **🟢 자동 구현** **DEP-P3-06 — Vite 6.4.5→8.3.5·React plugin 4.7.0→6.1.2의 빌드 호환성을 검증한다.** 우선순위 6, DEP-P3-05 다음.
  - **확인/미검증 구분:** Rollup/esbuild→Rolldown/Oxc 전환에 맞춰 `vite.config.ts`의 `minify: 'esbuild'`, MUI/Emotion 등의 manualChunks·Tauri pre-bundling 제외 설정을 검토한다. 현재 설정이 모두 실패한다고 입증한 것은 아니다. 명시된 WebView target을 기본 browser target 변화와 혼동하지 않는다.
  - **완료 조건:** 공식 단계별 migration과 Node/plugin 지원 범위를 확인하고 production chunk 초기화·Tauri custom-protocol native build·WebView 호환을 금융 연결 없이 검증한다. 타입/FSD/build·production UI E2E와 source harness를 통과하며 경고 억제나 크기 제한 완화로 대체하지 않는다. 안전하게 앱을 기동할 수 없으면 해당 런타임 검증은 미완료로 남긴다. [Vite 이전](https://vite.dev/guide/migration).

  - **착수 의존성:** 앞 번호는 검토 권장 순서다. 실제 API·peer·toolchain 의존성을 확인해 범위를 고정하며 DEP-P3-01 결정 대기를 이 항목의 일괄 차단으로 확대하지 않는다.

- [ ] **🟢 자동 구현** **DEP-P3-07 — 나머지 Rust·UI 의존성 후보의 실제 필요와 영향부터 확인한다.** DEP-P3-06 이후 개별 우선순위를 정한다.
  - **미검증 후보:** keyring 3.6.3→4.2.0(DB 암호 저장·mock), sha2 0.10.9→0.11.1(체크섬/이벤트 식별값), thiserror 1.0.69→2.0.21(오류 derive), react-grid-layout 1.5.4→2.3.0(새 API/내장 타입 및 legacy 호환 경로). 모두 확정 대규모 수정으로 판단한 것은 아니다. 이미 간접 thiserror 2.0.21이 있어도 직접 전환 완료는 아니다.
  - **완료 조건:** manifest/실제 참조·공식 breaking change·지원 target을 대조해 각 후보를 전환/유지/제거 검토로 결정한다. react-grid-layout은 당시 `src` 직접 import가 없었으므로 사용·CSS·동적 참조를 먼저 확인하며 이번에는 삭제하지 않는다. 전환 시 keyring은 mock만, sha2는 기존 fixture hash 불변, 오류 타입은 변환/메시지 회귀, UI는 사용 경로의 타입/build/E2E를 검증한다.

  - **착수 의존성:** 앞 번호는 검토 권장 순서다. 실제 API·peer·toolchain 의존성을 확인해 범위를 고정하며 DEP-P3-01 결정 대기를 이 항목의 일괄 차단으로 확대하지 않는다.

- [ ] **🟢 자동 구현** **DEP-P3-08 — TypeScript·패키지매니저 major 후보를 개별 평가한다.** DEP-P3-06·07 검토 이후, 필요성이 확인된 항목만 전환한다.
  - **미검증 후보:** TypeScript 5.9.3→7.0.2, Mac 기본 npm 11.20.0→12.2.1, pnpm 10.30.3→12.15.0. 프로젝트 Node 24.22.1 LTS의 npm은 11.19.1이며 기본 Node의 npm과 구분한다. 프로젝트는 npm을 사용하고 pnpm은 이번에 채택하지 않았다. 개별 프로젝트 실패는 미확인이다.
  - **완료 조건:** 공식 compiler/Node engines·peer 지원과 설치/lockfile 변화를 확인한다. TypeScript는 strict 타입·FSD/build, npm은 금융 파일 없는 깨끗한 snapshot의 `npm ci --ignore-scripts`·lockfile 재현·감사·CI를 통과한다. pnpm은 프로젝트 사용 필요부터 확인하고 불필요하면 미채택 결정을 남기며, 전환·lockfile 병행을 자동 도입하지 않는다.

  - **착수 의존성:** 앞 번호는 검토 권장 순서다. 실제 API·peer·toolchain 의존성을 확인해 범위를 고정하며 DEP-P3-01 결정 대기를 이 항목의 일괄 차단으로 확대하지 않는다.


## 2. 선행 결정 후 진행할 코드 작업

- [ ] **🟢 자동 구현** **DEP-P3-02 — SQLx 0.8.6→0.9.0의 SQL·DB 인증·잔여 RSA를 분리 검증한다.** 우선순위 2, DEP-P3-01 완료 후 전환.
  - **실제 코드 영향 확인:** `storage/database_schema.rs`·`database_projection.rs`의 동적 `query(&query)`/`query(&format!(…))`는 새 SqlSafeStr/AssertSqlSafe 경계 검토·수정 대상이다. 0.9의 비TLS MySQL RSA 인증은 `mysql-rsa` 기능을 별도 요구한다. PostgreSQL/MariaDB 저장·타입/문자열·transaction·실패 복구도 재검증한다. 이 경로는 모두 `src-tauri/src/` 아래다.
  - **완료 조건:** 허용된 테이블 식별자와 bind 경계를 보존하고 격리 DB의 schema/import/export·transaction rollback·TLS 및 지원 인증 계약을 통과한다. `cargo fmt --all -- --check` 및 `--locked` check/test/strict clippy/build와 최소 Rust/stable CI를 확인한다. RUSTSEC-2023-0071은 별도 dependency tree/audit로 재평가하고 SQLx 상승만으로 해소됐다고 처리하지 않는다. 기존 비TLS 지원 제거·인증 정책 변경은 별도 사용자 결정 없이 하지 않는다. [공식 0.9 변경](https://raw.githubusercontent.com/transact-rs/sqlx/v0.9.0/CHANGELOG.md).


## 3. 사용자 판단·준비가 필요한 항목

- [ ] **🟠 사용자 판단 대기** **P1-DB-01 — MariaDB contract fixture와 지원 버전 matrix 검증.**
  - **현재 미완료:** MariaDB 실제 서버의 contract fixture와 지원 버전별 schema/import/export·transaction·실패/재접속·인증 matrix 검증이다. adapter·DDL은 유지하며 PostgreSQL16 로컬 TLS/재접속 성공을 MariaDB 성공으로 확대하지 않는다.
  - **진행할 수 없는 사유:** 실제 서버 fixture는 기존 사용자 결정으로 보류되었고 지원할 MariaDB 버전·인증 범위가 확정되지 않았다. 임의로 보류를 해제하거나 비TLS 인증 지원을 제거하지 않는다.
  - **필요한 사용자 판단:** 보류 해제 여부, 지원 버전과 인증/TLS 범위를 결정한다. 운영 계정/DB 자료 대신 격리된 합성 fixture를 사용한다.
  - **재개 조건:** 범위가 정해지면 운영 서비스와 분리한 일회용 서버·합성 접속정보로 테스트를 실제 실행한다. 환경 미설정으로 즉시 반환한 테스트는 통과로 세지 않는다.
  - **객관적 완료 기준:** 지원 matrix의 각 조합에서 계약 실행 건수·성공/거부·rollback·재접속과 비밀정보 비노출을 기록하고 CI 실행/미실행을 구분한다. [기존 PostgreSQL 검증과 제외 범위](tasks/database-tls-reconnect-20261018.md), SQLx 전환 정책은 DEP-P3-01·02에서 별도 관리한다.
  - **의존성:** DEP-P3-01과 같은 DB 지원정책을 공유한다. 보류 해제와 matrix의 실제 실행을 별도 확인한다.

- [ ] **🟠 사용자 판단 대기** **DEP-P3-01 — 격리 DB 검증환경과 프로젝트 MSRV 기준을 결정한다.** 우선순위 1, SQLx 전환의 선행조건.
  - **확인:** 프로젝트 선언 MSRV 1.90.0→SQLx 0.9.0 요구 최소 1.94.0 결정이 필요하다. Mac Rust 1.99.1은 이미 요구를 충족한다. 기존 갱신 검증은 1.99.1 결과이며 1.90 실행 검증은 하지 않았다. DB 계약 5개는 환경 미설정으로 즉시 반환했으므로 실제 DB 통과가 아니다.
  - **현재 미완료:** SQLx 0.9 전환에 맞는 프로젝트 최소 Rust와 격리 PostgreSQL/MariaDB 지원 matrix·인증 정책의 확정, 해당 최소 버전/stable과 실제 DB 계약 실행이다. 기존 PostgreSQL16 로컬 검증과 2026-10-24 환경 미설정 DB 계약 5개의 즉시 반환을 구분한다.
  - **진행할 수 없는 사유:** 선언 MSRV 1.90과 SQLx 0.9 요구 1.94가 다르며 기존 MariaDB fixture 보류·비TLS 인증 지원의 정책을 임의로 변경할 수 없다. 개발 Mac 1.99.1의 성공만으로 최소 지원 버전을 검증한 것은 아니다.
  - **필요한 사용자 판단:** 최소 Rust를 1.94 이상으로 올릴지, 격리 DB 버전/인증 범위와 MariaDB 보류를 어떻게 다룰지 확정한다. 운영 계정/자료 없이 합성 전용 접속정보를 사용한다.
  - **재개 조건:** 결정된 지원 기준을 선언·CI에 반영하고 임시 DB fixture의 실제 실행·skip 감지를 연결한 후 DEP-P3-02 전환을 진행한다. 결정 전에도 로컬 검증 설계·호환 영향 조사·독립적인 다른 업데이트 검토는 가능하다.
  - **객관적 완료 기준:** 선택한 최소 Rust와 stable의 check/test, 지원 DB matrix별 실제 계약 실행 건수와 성공/거부를 기록한다. 즉시 반환·미실행을 통과로 세지 않고 schema/transaction·TLS·인증·복구를 확인한다. RSA 해소 판정은 별도 audit/의존성 근거로만 한다. [2026-10-24 검증·보류](security-mac-handoff-2026-10-06.md), [정책·RSA 잔여](release-security.md).

- [ ] **🟠 사용자 판단 대기** **P1-SCOPE-01 — 여러 broker/account 포지션과 pending을 동시에 상주·대조하는 multi-scope runtime.**
  - **현재 미완료:** 여러 broker/account의 포지션·pending을 동시에 상주시켜 scope별로 대조하는 runtime이다. 현재의 단일 활성 scope·owner 원장·계좌 전환 경합 보호는 구현했으며 동시 다계좌 운용과 다르다.
  - **진행할 수 없는 사유:** 동시 다계좌 운용의 제품 채택 여부와 scope별 실행·정지·리스크 한도가 정해지지 않았다. 현재 불변식을 임의로 바꾸면 주문·잔고·복원 경계가 달라진다.
  - **필요한 사용자 판단:** 동시 운용 채택 여부, 지원 broker/시장 조합, 계좌별·전체 리스크/비상정지 범위를 결정한다. 실제 credential·계좌 파일 제공은 이 결정에 필요하지 않다.
  - **재개 조건:** 제품 범위가 확정되면 `(broker, account, market, symbol)` tracker와 scope별 credential reconciliation 설계를 고정하고 mock부터 구현한다.
  - **객관적 완료 기준:** 동시 scope의 주문/체결/pending·캐시·소유 원가·재시작·부분 실패가 서로 침범하지 않으며 scope별/전체 정지·리스크를 실패 fixture와 UI에서 검증한다. 실제 계좌 운용은 별도 승인 검증으로 남긴다. [현재 소유·scope 경계](ARCHITECTURE.md), [기존 소유 피드백 검증](tasks/strategy-fill-ownership-20261018.md).

- [ ] **🟠 사용자 판단 대기** **P3-SIGN-01 — 플랫폼 코드 서명·updater 서명 강제를 위한 배포 키 준비.**
  - **현재 미완료:** 대상 플랫폼의 실제 코드 서명·updater 서명 생성과 서명 강제 release artifact 검증이다. checksum/attestation·sidecar 존재/비어 있지 않음·강제 옵션 검사는 구현했으며 암호학적 서명 검증이나 실제 서명 릴리스와 다르다.
  - **진행할 수 없는 사유:** 대상 플랫폼·키/인증서 관리 주체와 GitHub Environment의 서명 준비가 확정되지 않았다. 없는 키를 생성/등록한 것으로 가정하거나 서명 강제 변수만 먼저 켤 수 없다.
  - **필요한 사용자 판단:** 배포 플랫폼·서명 주체·키 보관/회전 정책과 승인할 release 환경을 정하고 관리자가 해당 환경에 키를 준비한다. 에이전트에는 비밀값 대신 준비 여부·키 식별용 비밀 아닌 metadata만 제공한다.
  - **재개 조건:** 환경의 서명 생성 단계와 필요한 키가 준비되면 별도 승인된 release 절차에서 `REQUIRE_RELEASE_SIGNATURES=true`를 적용한다. 로컬 verifier·mock의 자동 검증은 계속 가능하다.
  - **객관적 완료 기준:** 대상 artifact의 유효한 플랫폼/updater 서명을 확인하고 서명 누락·변조·불일치가 release를 차단함을 검증한다. checksum/attestation을 코드/updater 서명의 대체물로 취급하지 않는다. [현재 게이트와 제한](release-security.md).

  - **의존성:** 플랫폼/키 관리 결정과 관리자 준비가 먼저다. 구현 완료된 checksum·attestation과 실제 서명 검증을 구분한다.


## 4. 사용자 환경·실자료 검증이 필요한 항목

- [ ] **🟠 사용자 검증·자료 대기** **P1-PROVIDER-01 — KIS/Toss 기존 운영 원장과 실제 정정·취소 응답의 최종 검증.**
  - **현재 미완료:** 실제 KIS/Toss 정정·취소의 old/new order ID·거부/만료 의미와 도입 이전 보유·기존 체결의 전략/프로파일 귀속·원가·투자모드를 실제 원본에 대조하는 검증이다. 공통 상태·직렬화·exact owner·로컬 합성 재시작/체결은 검증했으며 운영 원장 검증과 구분한다.
  - **진행할 수 없는 사유:** 실제 사례의 비식별 원본과 운영자 귀속 확인이 없고 계좌 조회·소액 주문의 별도 승인이 없다. 새 원장에 계좌 전체 보유를 자동 편입하거나 확인 불가 기록의 주문 차단을 해제할 수 없다.
  - **필요한 비식별 자료/판단:** 식별값을 일관되게 치환한 요청·응답·시간순 상태·누적 수량/대금/비용 사례와 도입 이전 보유 귀속 대조 결과를 제공한다. 운영 연결이 필요하면 read-only 범위를 먼저 정하고 소액 주문은 별도로 명시 승인한다. key·계좌 원문·개인 계정 파일은 받지 않는다.
  - **재개 조건:** 비식별 fixture로 mapping·중복/역순·부분체결/정정/취소/거부/만료를 먼저 재현한다. read-only 대조 후에도 불충분한 실제 주문 검증만 별도 승인 범위에서 진행한다.
  - **객관적 완료 기준:** provider별 원래/새 주문 연결·최종 상태·수량/원가/모드·소유 귀속을 원본 근거와 대사하고 불명확 사례의 fail-closed·재시도 멱등성을 입증한다. 운영자 대조/복구 UI의 로컬 구현은 [P1-RECOVERY-01 이력](completed-history.md#p1-recovery-01)에 보존하며 실제 provider 근거를 통한 운영 검증은 이 항목에서 계속한다. [로컬 검증과 운영 제외 범위](tasks/strategy-fill-ownership-20261018.md).

  - **관련 완료 근거:** P1-RECOVERY-01의 UI·fixture 구현은 완료 이력에 보존하며 이 항목의 실제 provider 검증을 대체하지 않는다.

- [ ] **🟠 사용자 검증·자료 대기** **SOXQ-P1-05 — LTH 일봉 근사와 실제 장중 평가를 구분한다.**
  - **현재 미완료:** 2026-07-06~10-02 SOXQ 실제 분봉 원본, 거래소 시간대·DST·실제 거래일/조기마감·세션·기간 전체 coverage 검증과 생산 replay 재실행이 남아 있다. 2026-10-22의 시간/미관측 슬롯 진단은 합성 fixture·mock 검증이며 실제 공급원·발행/수신 시각·전체 달력을 확인한 것은 아니다.
  - **진행할 수 없는 사유:** 보관한 짧은 실제 분봉 표본과 일봉으로는 요청한 3개월의 장중 경로·누락 원인을 복원할 수 없다. 사용 가능한 해당 기간 원본/이용 권한이 확인되지 않았으며 일봉 보간이나 합성 fixture로 대체하지 않는다.
  - **필요한 비식별 자료:** 원본 파일 경로 또는 사용 가능한 공급원과 자료 이용 범위, feed·가격 조정 여부·봉 시작/종료 표기·요청 기간·수집 시각 metadata를 제공한다. API key·계좌번호·개인 계정 파일은 문서나 에이전트에 전달하지 않는다.
  - **재개 조건:** 원본과 이용 범위가 확인되면 원본 hash·누락/중복·거래소 달력·공개 가능 시각을 먼저 고정하고, 기존 fixed KST 실행 모델과 검증된 시장 시각의 차이를 해결한 뒤 재실행한다. 파싱 진단 통과만으로 `lookAheadSafe`나 성과 상태를 승격하지 않는다.
  - [ ] **객관적 완료 기준:** 요청 기간의 원본·출처·hash·세션별 coverage/제외 사유를 보관하고, DST·휴장/조기마감·gap·진입/마감 창 및 생산 신호/체결·독립 정산을 재현 가능하게 검증한다. 미확인 구간은 평가 불가로 남긴다.
  - **중복 미완료 조건 통합:** 기존 제목 **“LTH 보강의 일봉 성과 미리보기”**의 근거와 잔여 조건을 이 항목에 통합했다. 일봉 context/분봉 지표 분리와 일봉 `notEvaluable`는 이미 구현되었고, 일봉에서 장중 성과를 복원하는 새 기능을 약속하지 않는다. 활성 볼린저 성과는 실제 1분봉·시장 시간 검증 후에만 평가하며 옵션별 비교 결과는 별도 P2-12에 남긴다. 통합은 삭제나 완료 처리가 아니다. [시간 진단의 검증 범위](tasks/lth-minute-timing-20261022.md).
  - 대상: `commands/strategy_preview.rs`, `trading/strategy/leveraged_trend_hold/{live,preview,indicators}.rs`, `trading/simulation/assessment.rs`. [작업 기록](tasks/soxq-p1-05.md), [v7 검증](research/soxq-3m-20261003/p1-05-validation.md).

  - **완료된 부분의 근거:** [이관 기록](completed-history.md#soxq-p1-05)을 참고한다. 상위 완료 조건은 유지한다.

- [ ] **🟠 사용자 검증·자료 대기** **SOXQ-P2-12 — 검증된 LTH 분봉의 기본·볼린저 옵션별 비교.**
  - **현재 미완료:** P1-05의 검증된 실제 3개월 분봉을 사용한 기본/볼린저 옵션별 생산 비교 결과다. 짧은 표본의 신호0과 시간 진단 fixture는 옵션 효과의 근거가 아니다.
  - **진행할 수 없는 사유:** 검증된 전체 기간 자료·시장 달력/세션·공개 시각과 실행 시계의 일치가 확보되지 않았다. P1-05의 자료 준비와 이 항목의 옵션별 비교는 서로 다른 결과물이다.
  - **필요한 비식별 자료:** P1-05의 원본 경로·출처/이용 범위·시각/조정/coverage metadata를 공유한다. 같은 자료를 중복 요청하거나 별도 계좌 credential을 요구하지 않는다.
  - **재개 조건:** P1-05의 자료·시간 검증을 마친 뒤 결과를 보기 전 옵션·창·수량·비용을 고정하고 실제 장중 경로에서 비교한다.
  - **객관적 완료 기준:** 기본/볼린저 각 결과의 신호·체결/차단·완료 표본·실현/잔여·MDD·회전·노출을 독립 대조하고 무신호/자료부족/평가불가를 구분한다. generic LTH 거부·일봉 notEvaluable를 유지하고 짧은 신호0을 효과 없음으로 단정하지 않는다. [표본 연구](research/strategy-review-20261018/report.md), [시장 시각 진단의 한계](tasks/lth-minute-timing-20261022.md).
  - **완료된 부분의 근거:** [이관 기록](completed-history.md#soxq-p2-12)을 참고한다. 상위 완료 조건은 유지한다.

- [ ] **🟠 사용자 검증·자료 대기** **SOXQ-P2-13 — 가격조건의 실제 장중 손절·재진입 효능을 검증한다.**
  - **로컬 완료·이관:** 2026-10-29 생산 live6/replay9 fixture와 기존 고정 일봉 PC78원장 재현·독립 정산을 완료했다. [완료 범위·기존 조건](completed-history.md#soxq-p2-13), [설계·원장·검증·한계](research/price-condition-contracts-20261029/report.md). 상위 ID 전체는 미완료다.
  - **현재 미완료/사유:** 시각·시장일·DST·봉 시작/종료·세션·누락 coverage와 체결 가능성이 확인된 실제 장중 자료가 없다. 손절이 발동하지 않은 일봉과 합성 fixture로 native 손절 직후 재진입·cooldown·최소 기대순익·회전·순손익의 실효성을 확정하지 않는다.
  - **필요한 비식별 자료:** 사용 가능한 실제 장중 파일 경로·출처/이용 범위·가격 조정·발행/수신 시각·봉 표기·세션/누락 metadata. 실제 접수·부분체결까지 평가할 경우 일관되게 치환한 주문·누적체결/원가 근거를 함께 확인한다. 키·개인 계정 파일은 받지 않는다.
  - **재개/완료 기준:** 자료 검증 뒤 결과를 보기 전 기준가·수량·창·수수료/슬리피지10/20bp와 공통 guard를 고정하고 native와 연구 overlay를 분리한다. 실제 native 원가·부분보유/설정수량 차이·거절/접수/부분/종결, 같은 날·다음날·DST, 재진입·비용·노출·실현/잔여·회전을 생산 replay와 독립 장부로 대조한다. 사후 기준가 조정·guard 완화·자료 합성은 하지 않는다.
  - **현재 정책의 관측 한계:** PC 매도는 설정수량 요청 뒤 실제 소유수량으로 제한한다. 잔량이 남으면 live 기본 일일 매도 제한과 일반 guard가 후속 청산을 막을 수 있다. 로컬 검증은 잔량·원가 상태 보존이며 잔량 청산 성공이나 손실 제한 증거가 아니다. 연구 overlay 결과를 live 결과로 바꾸어 부르지 않는다.
  - **의존성:** 비식별 자료만으로 가능한 생산 검증은 자료 확보 후 자동 진행한다. 실제 계좌/provider 연결·금융거래는 이번 구현 범위에 포함되지 않는다.

- [ ] **🟠 사용자 검증·자료 대기** **SOXQ-P2-09 — 변동성 확장의 실제 장중 근거 검증.** 테스트 가능한 구현·일봉 비교는 완료했으며 최초 장중 발동·실제 체결 가능성 조건은 열린 상태다. [작업](tasks/volatility-expansion-policy-20261022.md), [사전 설계·비교·한계](research/volatility-expansion-policy-20261022/report.md), [독립 감사](research/volatility-expansion-policy-20261022/independent-audit.json).
  - **현재 미완료:** 최초 장중 조건 발동·주문 가능 시각·체결/차단·청산 창의 실제 자료 검증이다. 위 일봉/fixture·현재 UI 진단의 완료를 실제 장중 체결 가능성 확인으로 확대하지 않는다.
  - **진행할 수 없는 사유:** 관측 시가 대용인 live 첫 quote와 일봉 nextOpen은 실제 거래소 시가·봉 내부 순서·유동성을 입증하지 못한다. 해당 구간의 출처/공개 시각·세션·누락/coverage가 검증된 장중 자료가 없다.
  - **필요한 비식별 자료:** 장중 파일 경로 또는 공급원/이용 범위와 시장 timezone·봉 표기·발행/수신 시각·가격 조정·누락 metadata를 제공한다. 실제 체결 가능성까지 판단하려면 그 판단을 뒷받침하는 호가/체결 근거가 필요하며 OHLC만 있으면 그 한계를 남긴다.
  - **재개 조건:** 결과를 보기 전 동일 장중 창·자본·비용·단독 후보를 고정하고 coverage/공개 시각을 검증한 자료를 생산 엔진에 입력한다. 실제 계좌 연결·주문 없이 재현할 수 있는 검증은 그대로 자동 진행한다.
  - [ ] **객관적 완료 기준:** 최초 발동→주문 가능→체결/차단→청산의 시점·실현/잔여·비용/노출을 독립 대조하고, 미확인 장중 순서·유동성은 평가 불가로 분리한다. live 첫 quote/일봉 nextOpen 가정을 실제 장중 증거로 바꾸어 부르지 않는다.
  - **완료된 부분의 근거:** [이관 기록](completed-history.md#soxq-p2-09)을 참고한다. 상위 완료 조건은 유지한다.


## 완료와 이력 이동 기준

- 각 ID는 남은 코드·실패/복구 검증·사용자에게 보이는 상태·관련 문서가 함께 반영된 범위만 닫는다. 부분 완료는 이력 링크를 남기고 상위 항목은 열린 상태로 둔다.
- 검증 기록은 코드 상태, 날짜, 환경/모드, 실제 실행 건수, 실패·skip·미실행 경계를 보존한다. 과거 통과를 이번 실행 결과로 재사용하지 않는다.
- 완료·기각·통합 시 [완료·경과 이력](completed-history.md)에 원래 ID/제목·날짜·근거·차단 또는 잔여 조건과 후속 ID를 남기고 열린 TODO에서 완료 기록을 걷어 낸다.
- 실계좌 검증은 모의/fixture → 승인된 read-only → 별도 명시 승인된 소액 주문 순서다. 문서 작업에는 [문서 harness](HARNESS_MAP.md)의 링크·경로·지도·TODO 동기화 검사를 적용하고 거래 빌드를 재실행하지 않는다.
