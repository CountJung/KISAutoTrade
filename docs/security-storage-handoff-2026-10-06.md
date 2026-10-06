# 보안 의존성·Windows 저장 수정 인계 (2026-10-06)

기준 main: `8db7bf8f2631827ce007a8a4821541df15e1ff32`. 전용 브랜치: `fix/security-dependencies-2026-10-06`. Lenovo Windows x64/Rust 1.92.0/Node 22.22.0/npm 11.13.0에서 검증했다. main 직접 push·PR 병합·배포·거래 API 호출은 수행하지 않는다.

## 분리된 변경

첫 커밋 `0812c71`은 authoritative lockfile의 보안 의존성만 갱신한다. manifest/feature/주문 로직 변경 없이 부모 semver 범위를 만족한다.

| 패키지 | 이전 | 수정 |
|---|---|---|
| rustls | 0.23.37 | 0.23.45 |
| rustls-webpki | 0.103.13 | 0.103.15 |
| seroval | 1.5.6 | 1.6.8 |
| nanoid (개발 간접 의존성) | 3.3.16 | 3.3.20 |
| source-map-js (개발 간접 의존성) | 1.2.1 | 1.2.2 |

rustls는 reqwest HTTPS, tokio-tungstenite WSS, SQLx DB TLS에서 사용한다. webpki 갱신은 rustls의 `^0.103.14` 제약 때문이다. seroval은 `@tanstack/react-router → @tanstack/router-core`의 운영 의존성이다. 현재 client-rendered production 번들에서 serializer는 제거되지만 lock의 취약점과 audit 실패를 해결하기 위해 갱신했다. [rustls 공지](https://rustsec.org/advisories/RUSTSEC-2026-0285.html), [seroval critical 공지](https://github.com/lxsmnsyc/seroval/security/advisories/GHSA-p6vx-979v-rg4c), [seroval high 공지](https://github.com/lxsmnsyc/seroval/security/advisories/GHSA-jp82-f5mq-hwhp).

후속 저장 커밋은 `src-tauri/src/storage/database_io.rs`의 OS별 교체/flush 처리와 회귀 fixture를 보강한다. Windows-target 전용 `windows-sys 0.61`을 추가하며 이미 lock에 있던 0.61.2를 재사용한다.

- 같은 디렉터리 temp에 쓰고 `sync_all()` 후 handle을 닫는다.
- 기존 정상본 `.bak`은 Windows에서 writable handle로 flush한다. Unix는 read-only handle fsync를 유지해 `0444` 파일 교체도 보존한다.
- Unix는 기존 rename → parent directory fsync, Windows는 `MoveFileExW(REPLACE_EXISTING | WRITE_THROUGH)`를 사용한다. copy/delete fallback은 허용하지 않는다.
- Windows는 parent만 canonicalize해 긴 경로와 한글 경로를 UTF-16/NUL-terminated buffers로 전달한다.
- 실패를 삼키지 않는다. 교체 API 자체 실패 시 정상본을 유지하고 staged temp를 정리한다. Unix rename 이후 parent fsync 실패는 오류를 반환하지만 새 정상본은 이미 반영될 수 있는 기존 의미를 유지한다. backup copy 자체의 임의 부분 실패까지 기존 `.bak`이 항상 유지된다는 보장은 새로 추가하지 않았다.

[Windows API 계약](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-movefileexw)에 따른 write-through 요청이며 실제 전원 차단·disk/controller 오류의 내구성 증명은 아니다. 실제 사용자 데이터·자격증명은 검사하지 않았다.

## 검증 결과

| 검사 | Lenovo 결과 |
|---|---|
| `npm ci --ignore-scripts --no-audit` | PASS |
| `cargo metadata --locked --format-version 1 --no-deps` | PASS |
| `cargo audit` (기존 예외 3개) | PASS, 미예외 취약점 0, 기존 경고 14 |
| `npm audit --omit=dev --audit-level=high` | PASS, 운영 취약점 0 |
| `npm audit --audit-level=critical` | PASS, critical 0; 개발 high 1/moderate 1 잔여 |
| `cargo check --workspace --all-targets --locked --offline` | PASS, 컴파일 경고 0 |
| `cargo test --workspace --lib --locked --offline` | PASS, 193 passed/0 failed |
| `cargo test --workspace --test lth_replay_clock --locked --offline` | PASS, 3 passed |
| `cargo fmt --all -- --check` | PASS |
| `cargo clippy --workspace --all-targets --all-features --locked --offline` | PASS, 경고 0 |
| `npx tsc --noEmit`, `npm run build:web` | PASS |
| lockfiles/FSD/project-map/Graphify checks | PASS |
| `npm run test:e2e` | PASS, Chromium mock 30/30 |
| Graph HTML 공격 문자열 fixture | PASS, 외부 요청 차단·drawing stub, 실제 DOM 위임 이벤트/이스케이프 검증 |

baseline lib 결과는 181 passed/9 failed(190개)였다. 동일한 9개 storage/ledger/keychain migration/budget/risk restore 실패가 모두 해소됐고 Windows fixture 3개를 추가해 총 193개가 됐다. 기존 corrupt JSON backup 복구·동시 쓰기 검사도 통과한다. 추가 fixture는 반복 저장/직전 백업, 긴 한글 경로, backup 실패의 정상본 보존, Windows delete-sharing 잠금 실패와 재시도를 다룬다.

193 통과에는 외부 DB 환경 미설정으로 즉시 반환한 PostgreSQL contract 3개가 포함된다. 실제 검증은 190개이며 DB roundtrip/잘못된 자격증명/파괴 작업 확인문구 3개는 **미검증**이다. 테스트 프로세스에서 `KISAT_PG_HOST/PORT/USER/PASSWORD`를 비워 접속을 차단했다. MariaDB contract, 실제 broker TLS·주문, desktop 설치 artifact는 검사하지 않았다.

Mac/Unix는 코드 리뷰와 cfg별 테스트 추가만 완료했다. 이 Lenovo에는 WSL distribution도 없어 실제 Unix 실행을 하지 않았다. Mac에서도 수정 전 오류가 재현됐다고 주장하지 않는다. 추가 Unix fixture는 read-only 원본 교체 및 private sync/async 저장 `0600`을 검증하며 맥미니에서 실행해야 한다.

Graphify는 로컬 AST/code-only로 갱신했다. 그래프·리포트·HTML data·source snapshot을 갱신했다. HTML은 기존 안전한 template 전체를 유지해 CDN 버전 고정/SRI와 이웃 링크의 escaped data attribute·위임 이벤트 처리를 보존했다. 생성된 Windows 절대 경로 manifest/cache는 커밋하지 않는다.

## 맥미니에서 안전하게 이어받기

기존 작업 폴더에서 먼저 아래 두 명령을 실행한다. status에 변경이 있으면 그대로 둔 채 **다른 새 경로의 worktree**를 사용한다. reset/clean/강제 checkout으로 미커밋 변경을 덮어쓰지 않는다.

```bash
git status --short
git fetch origin
git worktree add --detach ../KISAutoTrade-security-review-20261006 \
  origin/fix/security-dependencies-2026-10-06
cd ../KISAutoTrade-security-review-20261006
git switch -c review/security-storage-20261006
```

위 새 경로와 로컬 브랜치가 이미 있으면 다른 이름을 선택한다. 기존 맥미니 미푸시 커밋과의 통합은 diff를 검토한 뒤 별도로 진행한다.

```bash
npm ci --ignore-scripts
env -u KISAT_PG_HOST -u KISAT_PG_PORT -u KISAT_PG_USER -u KISAT_PG_PASSWORD \
  cargo test --workspace --lib --locked
cargo test --workspace --test lth_replay_clock --locked
cargo check --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked
npx tsc --noEmit
npm run build:web
npm run check:lockfiles
npm run check:project-map
npm run check:graphify
```

private 파일 `0600`/읽기 전용 원본 교체 fixture를 포함한 Unix lib 결과를 먼저 확인한다. 실제 계좌·DB 접속 테스트는 별도 승인과 격리 서버가 있을 때만 진행한다.

## 남은 기존 문제

- RustSec 예외는 늘리지 않았다: rsa RUSTSEC-2023-0071, quick-xml RUSTSEC-2026-0194/0195. ignore 없는 감사는 이 3개로 여전히 실패한다.
- 기존 RustSec 경고 14개(unmaintained 7/unsound 6/yanked 1)는 그대로다. 범위 확장 없이 각 upstream 제약/실제 노출을 별도로 검토해야 한다.
- 개발 도구 Vite 5.4.21 high와 esbuild 0.21.5 moderate가 남는다. npm 권장 수정은 Vite major 업그레이드이므로 이번 최소 패치에서 제외했다. 현재 보안 정책은 운영 high와 전체 critical을 차단하며 정책을 약화하지 않았다.
- GitHub Release Gate/Dependency Security의 push trigger는 main만 허용한다. 전용 fix 브랜치 push에는 자동 CI가 생성되지 않을 수 있다. PR/승인된 workflow_dispatch에서 별도 실행할 수 있으며 release/tag workflow는 사용하지 않는다.
