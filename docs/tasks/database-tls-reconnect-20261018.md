# DB TLS·단절·설정 복구 계약 (2026-10-18)

- 상태: 완료 — 실제 로컬 서버 계약·전체 회귀·독립 리뷰 통과; 게시 커밋은 Git 이력 참조
- 대상: TODO P1 TLS verify-full 인증서·서버 강제 단절·재접속
- 시작: `fix/mac-security-storage-2026-10-06`, `f7d3a24`
- 범위: PostgreSQL16 로컬 합성 fixture, pool 게시/설정 복구, 릴리스 gate
- 제외: 실제 계좌/운영 DB/주문, MariaDB 지원 버전 결정, TLS 설정 IPC·UI schema 변경

## 구현과 불변식

`scripts/database-tls-contract.mjs`는 실행별 UUID·소유 라벨의 PostgreSQL16 컨테이너2개(TLS/비TLS), 임시 CA와 `DNS:localhost` 인증서를 구성한다. 로컬 Docker socket만 사용하고 포트는 `127.0.0.1`에 고정 공개하여 재시작 전후 같은 주소를 유지한다. 프로세스의 기존 PG 환경·pgpass를 사용하지 않고 합성 자격증명과 빈 pgpass, 자식 프로세스에만 유효한 CA를 전달한다. 컨테이너 제어·정리는 정확한 소유 라벨을 먼저 검증한다. 개인키는 종료 시 삭제하며 생성물은 프로젝트 `.cache/`에 둔다.

TLS 요구 모드는 기존 PostgreSQL `VerifyFull`, MariaDB `VerifyIdentity`를 유지한다. PostgreSQL의 대상 DB가 없을 때 관리자 DB로 생성하는 연결에도 같은 TLS 모드를 적용한다. CA 신뢰와 hostname 확인은 [PostgreSQL 클라이언트 TLS 문서](https://www.postgresql.org/docs/16/libpq-ssl.html), 서버 인증서·키 권한은 [서버 TLS 문서](https://www.postgresql.org/docs/16/ssl-tcp.html)를 참고한다. 검증 대상은 Cargo.lock의 SQLx0.8.6과 PostgreSQL16이다.

시작 감사에서 발견한 pool cold-start 경합은 별도 생성 mutex와 게시 전 fingerprint 재확인으로 해결한다. 늦게 완료된 이전 설정의 pool은 닫고 캐시에 게시하지 않는다. 상태는 같은 설정/pool snapshot에서 생성한다. 이미 취득한 쓰기를 임의로 재시도하지 않는다.

설정 저장은 파일과 OS keychain을 하나의 복구 단위로 취급한다. 비밀 없는 recovery 표시를 blocking 작업 시작 전에 만들고 파일 commit 후 메모리 게시까지 유지한다. pre-write와 post-rename 오류를 구분하지 않고 실제 파일·keychain을 함께 원복/검증한다. 호출자 취소 또는 불명확한 원복은 다음 load/save/pool을 차단한다. 원복 시 신규 primary 삭제와 marker 삭제도 Unix parent fsync/Windows write-through rename으로 반영한다. marker 삭제 후 동기화 실패는 표시를 재생성하고, 재생성 실패에도 runtime latch로 연결/저장을 차단한다. 검증된 원복 또는 게시 완료 뒤에만 latch를 해제한다. 복구 표시를 자동 삭제해 우회하지 않는다. keychain을 사용할 수 없는 환경의 기존 private 파일 fallback은 유지한다.

DB backend가 활성인 상태에서 서버를 강제 종료하면 status, pending/소유 원장/risk production 읽기, 쓰기, JSON backend 복귀가 오류를 반환해야 한다. 로컬 primary/backup과 저장된 DB backend 설정은 변하지 않아야 한다. 같은 컨테이너 재기동 뒤 같은 cached pool의 새 server 시작 시각과 commit 문서·새 쓰기 왕복을 확인한다.

## 조사·회귀

- symbol_navigator: SQLx 연결 옵션·PGSSLROOTCERT/pgpass·관리자 경로·managed store 참조 조사.
- helper_curator: private atomic writer 재사용과 provider 경계 확인. 동등하지 않은 관리자 생성 SQL은 통합하지 않는다.
- 코드 리뷰: single-flight/status snapshot, post-rename rollback, 취소와 startup failclosed 검토.
- 기존 full roundtrip은 고정2026-07-12 거래가 오늘의 90일 retention 밖이 되어 실패했다. retained=오늘, expired=91일 전의 fixture로 고치고 실제 retained payload와 fills row_count1을 모두 확인한다. 생산 retention 규칙은 변경하지 않는다.
- Windows/Unix 저장 내구성 구현은 플랫폼 규칙과 합성 실패 주입을 검증한다. 실제 전원 차단을 증명하지 않는다.

## 검증 기록

| 결과 | 검증 | 근거 |
|---|---|---|
| PASS | `node scripts/database-tls-contract.mjs` | 최종 run `5f656179-c27b-422d-9a1c-f3392a5befc1`: 일반3+TLS/재접속1+불신CA1 실제 통과, 완료 marker·컨테이너/개인키 정리 확인 |
| PASS | `cargo test --workspace --locked` (PG fixture 환경 제거) | 단위464·통합3·doc3 보고 통과. 단위 중 외부 계약5개는 이 실행에서 early-return, 위 Docker 실행으로 별도 실제 확인 |
| PASS | DB 집중 회귀 | worker 실행 실제29개(설정9개 포함), 별도 서버 계약5개 생략. single-flight24 callers·stale pool 폐기·post-rename/새 파일 원복·marker sync 실패·latch·취소·재저장 확인 |
| PASS | `cargo check --workspace --all-targets --locked` | 경고0 |
| PASS | `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` | 경고0 |
| PASS | `cargo fmt --all -- --check` | 최종 소스 포맷 확인 |
| PASS | TypeScript/FSD/lockfile/web build | `npx tsc --noEmit`, `npm run check:fsd`, `npm run check:lockfiles`, `npm run build:web` |
| PASS | `env -u NO_COLOR npx playwright test tests/e2e/settings-database.spec.ts` | 웹 DB 관리 제한·좁은 폭·예산 보호4개, mock transport |
| PASS | timeout/interrupt 프로세스 회귀 | 실제 runner 함수로 Node 부모+상속 pipe 자식 생성: timeout/종료신호 뒤 두 PID 종료·bounded reject·cleanup 허용2건 |
| PASS | 구조·문서·TODO 게이트 | project-map update/check, Graphify5154 nodes/11070 edges/508 communities의 graph/HTML/report 일치·freshness, 상대 링크164개·diff·TODO renderer3개 통과(열린25개) |
| PASS | 독립 리뷰 | 시작/종료 코드 리뷰·Serena 영향 분석·helper 감사·project mapper. 발견한 설정/풀 경합·rollback 삭제 내구성·subprocess tree timeout 수정 후 신규 P1/P2 없음 |

일반 전체 Rust 테스트의 최초 실행은 sandbox의 localhost bind 거부로26개가 실패했다. 같은 범위를 승인된 제한 밖 실행으로 재실행해 모두 통과했으며, 테스트 실패를 skip/기대값 완화로 숨기지 않았다. Playwright의 NO_COLOR/FORCE_COLOR 중복 환경 알림은 NO_COLOR를 제거해 재검증했다. Graphify의 symbol 없는 JSON 설정4개 진단은 도구 AST 한계이며 코드 경고와 구분한다.

Windows 삭제/복구 경로는 소스 리뷰·플랫폼별 구현만 확인했고 Windows에서 실행하지 않았다. 실제 전원 차단·운영 keychain·운영 DB·MariaDB·KIS/Toss 계좌/주문 검증으로 확대하지 않는다. 릴리스 workflow는 연결을 수정했으며 원격 tag build를 실행하지 않았다.

복구 오류 시 표시만 지우거나 실패한 쓰기를 자동 재전송하지 않는다. 설정 파일/backup/keychain 일치를 확인한 운영 복구가 필요하다. 코드 rollback은 이 작업 커밋을 revert하고 full server contract/정적 게이트를 재실행한다.
