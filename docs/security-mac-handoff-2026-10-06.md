# Mac 보안 수정·동기화 인계 (2026-10-06)

## 원본과 통합 기준

- 실제 기존 Mac 원본: `/Users/jsjmac/Workspace/MacWorking/KISAutoTrade`.
- 시작 상태: clean `main`, `8db7bf8f2631827ce007a8a4821541df15e1ff32`, `origin/main`과 동일, worktree 하나. 미커밋·미푸시 변경은 없었다.
- 검증 작업본: `/Users/jsjmac/Documents/Codex/2026-10-06/task/KISAutoTrade`. 원본에서 Git 객체를 독립 복제했으며 사용자 데이터·자격증명·캐시는 복사하지 않았다.
- 전용 브랜치: `fix/mac-security-storage-2026-10-06`. Windows 원격 `015a28a8b38d9cc9665511ccdae96afe06100c32`에서 시작했고 Mac 원본은 그 정확한 조상이므로 충돌 없이 보안·저장 커밋을 모두 보존했다.
- main 직접 push, 강제 push, reset/clean, PR 병합, 배포와 거래 API 호출은 수행하지 않는다. 최종 SHA와 원본 동기화 상태는 실행 완료 보고와 Git 로그가 기준이다.

## 보안 예외·경고의 근본 수정

| 항목 | 실제 경로·이전 상태 | 수정 결과 |
| --- | --- | --- |
| XML 예외 2개 | Tauri/codegen/plugin → plist 1.8.0 → quick-xml 0.38.4 | plist 1.10.1 → quick-xml 0.42.0. RUSTSEC-2026-0194/0195 해소, CI ignore 두 개 삭제 |
| anyhow unsound | 직접 및 Tauri 경로의 1.0.102 | 1.0.104, RUSTSEC-2026-0190 해소 |
| event-listener unsound | SQLx → sqlx-core → 5.4.1 | 5.4.2, RUSTSEC-2026-0221 해소 |
| rand unsound 2개 | SQLx/WS/RSA/phf/quinn 경로의 0.8.5/0.9.2 | 0.8.8/0.9.5, RUSTSEC-2026-0097 해소 |
| rand 0.7.3 unsound | tauri-utils → kuchikiki → selectors 0.24 → phf_generator 0.8 | Tauri 2.12.1의 dom_query 전환으로 경로 제거 |
| fxhash unmaintained | 같은 selectors 0.24 경로 | Tauri 2.12.1 갱신으로 제거 |
| unic unmaintained 5개 | tauri-utils → urlpattern 0.3 → unic-ucd-ident → char-property/range/common/ucd-version | tauri-utils 2.10.1 → urlpattern 0.6의 ICU 전환으로 5개 모두 제거 |
| spin yanked | SQLx MySQL → rsa → num-bigint-dig → lazy_static[spin_no_std] → spin 0.9.8 | 기존 ^0.9.8 제약을 만족하는 0.9.9. Once unsoundness의 실제 수정 릴리스 |
| Vite/esbuild 개발 취약점 | Vite 5.4.21 → esbuild 0.21.5 | Vite 6.4.4 → esbuild 0.25.12. 기존 React 플러그인 4.7.0과 Node 요구 조건을 유지하며 npm 전체 취약점 0개 |

최종 결과는 기존 예외 **3 → 1개**, RustSec 경고 **14 → 2개**다. 예외를 늘리거나 경고를 숨기지 않았다. React 18 및 Tauri 2 프레임워크, IPC·주문·DB schema·권한 설정을 유지했다. Tauri 안정판 업데이트에 맞춰 Rust 최소 버전을 1.90으로 바로잡았다. Windows 이전 검증 Rust 1.92는 이 기준을 충족하지만 이번 업데이트의 Windows 실행 검증은 하지 않았다.

공식 근거: [plist 변경 기록](https://raw.githubusercontent.com/ebarnard/rust-plist/v1.10.1/CHANGELOG.md), [Tauri build manifest](https://raw.githubusercontent.com/tauri-apps/tauri/tauri-v2.12.1/crates/tauri-build/Cargo.toml), [Tauri utils manifest](https://raw.githubusercontent.com/tauri-apps/tauri/tauri-v2.12.1/crates/tauri-utils/Cargo.toml), [spin 변경 기록](https://docs.rs/crate/spin/0.9.9/source/CHANGELOG.md), [Vite 보안 공지](https://github.com/vitejs/vite/security/advisories/GHSA-fx2h-pf6j-xcff), [Vite 6 이행 안내](https://v6.vite.dev/guide/migration).

## 잔여 항목과 승인 경계

- RSA RUSTSEC-2023-0071: `sqlx-mysql 0.8.6 → rsa 0.9.10`. 수정된 안정판이 없다. SQLx 0.9도 RSA를 유지하면 취약한 0.10 RC를 사용한다. RSA 기능을 제거하면 현재 Disable/Prefer/Require를 지원하는 비 TLS MariaDB 인증 기능이 달라지므로 수행하지 않았다. 사용자 승인하에 TLS 인증만 지원하도록 바꾸거나 upstream 수정이 필요하다. 개인키 연산은 앱 경로에서 사용하지 않지만 advisory 자체는 잔여다.
- GLib RUSTSEC-2024-0429: Linux 전용 `Tauri/tao/wry → GTK3 0.18 → glib 0.18.5`. 수정 ≥0.20과 안정 Tauri GTK 제약이 불일치한다. alpha GTK 강제 패치·Linux 기능 제거·프레임워크 교체를 수행하지 않았다.
- proc-macro-error RUSTSEC-2024-0370: Linux 전용 `glib-macros/gtk3-macros → 1.0.4`. 소비자 매크로의 upstream 전환이 필요하다. Mac target에는 GLib와 이 매크로 경로가 포함되지 않는다.

[현재 보안 정책과 해제 조건](release-security.md)에 상세 근거를 유지한다. ignore 없는 cargo audit는 RSA로 실패하며, 기존 RSA ignore를 적용한 정책 감사는 통과하지만 경고 두 개를 그대로 출력한다. 보안 경고 0개 또는 전체 보안 과제 완료라고 보고하지 않는다.

## Mac 검증

환경: Apple Silicon macOS, Rust 1.95.0, Node 26.3.0, npm 11.16.0. 실사용 .env/secure_config.json/profiles.json은 읽지 않았고 앱을 시작하지 않았다. 테스트 DB 연결 변수를 제거했다.

| 명령/검사 | 결과 |
| --- | --- |
| npm ci --ignore-scripts | PASS, 최종 lock 재설치 |
| cargo check --workspace --all-targets --locked | PASS, 컴파일 경고 0 |
| cargo test --workspace --lib --locked | PASS 195/195. PostgreSQL contract 3개는 DB 변수 미설정으로 즉시 반환하므로 실제 검증은 192개 |
| cargo test --workspace --test lth_replay_clock --locked | PASS 3/3 |
| cargo clippy --workspace --all-targets --all-features --locked -- -D warnings | PASS. Mac Rust/MSRV에서 드러난 5개 lint를 동등한 표준 메서드로 수정 |
| cargo fmt --all -- --check | PASS |
| npx tsc --noEmit / npm run build:web | PASS. 실제 비어 있는 vendor chunk 설정을 제거해 empty chunk 경고 원인을 해소 |
| npm run test:e2e | PASS Chromium mock 30/30 |
| npm audit / npm audit --omit=dev | PASS, 개발 포함 전체 취약점 0개 |
| cargo audit, 기존 RSA ignore만 적용 | PASS, 추가 취약점 0. Linux 경고 2개 잔여 |
| lockfiles/FSD/project-map/graph freshness | PASS |
| Graph HTML 공격 문자열 fixture | PASS. 외부 요청 차단, drawing stub, 실제 DOM의 escaped data attribute·위임 click 검증 |

저장 테스트는 반복 교체·직전 백업, 긴 한글 경로, 백업 실패의 정상본 보존 및 temp 정리, 손상 JSON 백업 복구, 동시 쓰기, Unix read-only 정상본 교체 및 private sync/async 0600을 포함한다. keychain은 테스트 mock을 사용한다. PostgreSQL/MariaDB 실서버, 실제 broker TLS/주문, desktop 설치·실행과 실제 정전 내구성은 미검증이다.

Graphify는 AST/code-only로 갱신했고 기존 HTML 안전 템플릿의 고정 CDN/SRI·escaped data attribute·위임 이벤트 처리를 보존한다. 머신별 절대 경로 manifest는 변경하지 않는다. Graphify의 JSON 설정 4개(extensions/launch/tasks/secure_config.example)의 0-node 안내는 현재 도구의 추출 한계이며 보안 경고가 아니다. 이를 숨기기 위한 ignore는 추가하지 않았다.

## Linux headless·클라우드 준비 확인

기존 `.github/workflows/release-gate.yml`에 Linux 시스템 패키지, cargo check/lib test 및 Chromium headless E2E가 이미 정의되어 있다. 아래 절차는 문서 안내이며 이번 작업에서 새 클라우드나 시스템 패키지를 설치하지 않았다.

Ubuntu/Debian 준비 기준은 [Tauri 공식 Linux prerequisites](https://v2.tauri.app/start/prerequisites/#linux)와 CI다. Rust ≥1.90, Node ≥20, npm ≥10이 필요하다. CLI만으로 test/check와 웹 E2E가 가능하지만 Rust library에 Tauri가 연결되어 있어 Linux WebKit/GTK 개발 라이브러리는 필요하다.

```bash
sudo apt-get update
sudo apt-get install -y build-essential pkg-config curl wget file \
  libwebkit2gtk-4.1-dev libxdo-dev libssl-dev \
  libayatana-appindicator3-dev librsvg2-dev patchelf
npm ci --ignore-scripts
npx playwright install --with-deps chromium
npm run build:web
cargo check --workspace --all-targets --locked
env -u KISAT_PG_HOST -u KISAT_PG_PORT -u KISAT_PG_USER -u KISAT_PG_PASSWORD \
  cargo test --workspace --lib --locked
cargo test --workspace --test lth_replay_clock --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
CI=1 npm run test:e2e
npm run check:lockfiles
npm run check:fsd
npm run check:project-map
npm run check:graphify
npm audit
cargo audit --file Cargo.lock --ignore RUSTSEC-2023-0071
```

이 명령들은 DB·매매 API를 요구하지 않는다. Linux 실행은 이번 Mac 결과로 대체하지 않으며 Linux GTK 잔여 경고를 별도로 확인해야 한다. 실제 Tauri 데스크톱 창 검증은 디스플레이가 필요한 별도 범위다.

실측 용량(작업 중 2026-10-06): 코드·문서 약 25MB, Git 6.4MB, node_modules 271~283MB, 새 target 약 6GB, 기존 Mac target 13GB, 테스트 산출물 4KB, dist 1.1MB, 프로젝트 임시 도구·로그 약 25MB, 공유 Playwright 브라우저 1.1GB. 공유 Cargo registry 1.3GB, Rust toolchain 1.5GB, npm cache 2.2GB는 다른 프로젝트와 공용이다. 그래프 재생성으로 추적 파일 크기는 달라질 수 있다. 신규 Linux 용량은 실측하지 않았다. 이 Mac 실측에 기초한 개발 공간 추정은 캐시를 포함해 약 20GB 이상, 여유 포함 30GB 수준이며 OS·클라우드 서비스 기본 설치 공간은 별도다. 캐시는 원격 동기화 대상이 아니므로 코드 저장소만 clone하고 Linux에서 다시 생성한다.


## 2026-10-24 안정 업데이트 재검증

### 실제 대상과 버전

실제 원본 `/Users/jsjmac/Workspace/MacWorking/KISAutoTrade`에서 시작했다. 시작 상태는 clean, 브랜치 `fix/mac-security-storage-2026-10-06`, HEAD `04f3b047b14a4046ffd9ebbf6be0776d861fc9fe`였다. 과거 검증 복제본의 사용자 미커밋 변경은 그대로 보존했다. 아래 값은 이번 실제 적용·lockfile 비교이며 위 10월 6일 실측을 대체하지 않는다.

| 대상 | 이전 | 적용 후 |
| --- | --- | --- |
| Rust stable / rustup | 1.99.0 / 1.29.1 | 1.99.1 / 1.29.2 |
| Mac 기본 Node / npm | 26.3.0 / 11.16.0 | 26.12.1 / 11.20.0 |
| 프로젝트 Node / npm | .nvmrc 25.9.0 | .nvmrc 24.22.1 LTS / npm 11.19.1; keg-only 추가 설치 |
| Homebrew | 7.0.7 | 7.0.10 |
| macOS | 27.0.1 (26A434) | 유지; softwareupdate 조회에서 업데이트 없음 |
| Xcode / CLT | 27.0 (27A266a) / 27.0 | 유지; Xcode 27.1 설치 미실행 |
| tauri / tauri-build / tauri-utils | 2.12.1 / 2.7.1 / 2.10.1 | 2.12.5 / 2.7.4 / 2.10.2 |
| tokio / serde / serde_json | 1.50.0 / 1.0.228 / 1.0.149 | 1.54.1 / 1.0.229 / 1.0.154 |
| rustls / rustls-webpki | 0.23.45 / 0.103.15 | 0.23.46 / 0.103.16 |
| webpki-roots / quinn-proto | 1.0.6 / 0.11.15 | 1.0.11 / 0.11.19 |
| 직접 reqwest / tokio-tungstenite / axum | 0.12.28 / 0.24.0 / 0.7.9 | 유지 |
| 직접 tower-http / SQLx / keyring | 0.5.2 / 0.8.6 / 3.6.3 | 유지 |
| TanStack Query / Router | 5.96.1 / 1.168.10 | 5.104.5 / 1.170.46 |
| Tauri JS API / CLI | 2.10.1 / 2.10.1 | 2.12.4 / 2.12.4 |
| zustand / lightweight-charts / react-grid-layout | 5.0.12 / 5.1.0 / 1.5.3 | 5.0.16 / 5.2.1 / 1.5.4 |
| Vite / Playwright | 6.4.4 / 1.61.1 | 6.4.5 / 1.64.1 |
| React / MUI / TypeScript / React plugin | 18.3.1 / 6.5.0 / 5.9.3 / 4.7.0 | 유지 |

Cargo는 기존 semver 범위와 MSRV 1.90 fallback을 사용했다. 갱신 후 metadata에서 선언된 `rust_version`이 1.90을 넘는 패키지는 없었다. **Rust 1.90 toolchain 자체의 실행 검증은 하지 않았으며**, 실제 check/test/build는 stable 1.99.1 결과다. 직접 manifest의 major는 유지했다. 이미 있던 간접 reqwest 0.13.2→0.13.5, tower-http 0.6.8→0.6.11, thiserror 2.0.18→2.0.21은 해당 경로 안에서 갱신했다.

공식 확인 기준: [Rust stable 배포 manifest](https://static.rust-lang.org/dist/channel-rust-stable.toml), [Node 10월 보안 릴리스](https://nodejs.org/en/blog/vulnerability/october-2026-security-releases), [Node 배포 목록](https://nodejs.org/dist/index.json), [crates.io 레지스트리](https://crates.io/), [npm 레지스트리](https://registry.npmjs.org/), [Homebrew 공식 formula API](https://formulae.brew.sh/api/formula.json), [Apple 공식 Xcode 릴리스](https://developer.apple.com/news/releases/), [App Store 공식 조회](https://itunes.apple.com/lookup?id=497799835&country=kr), [Apple macOS 업데이트](https://support.apple.com/en-us/100100). npm/Cargo 최신 후보와 보안 공지는 이 날짜에 확인했으며 베타·nightly를 채택하지 않았다.

### 적용과 보류

- Cargo/npm lockfile을 안정 호환 범위 안에서 갱신하고 `npm ci --ignore-scripts`로 재설치했다. 프로젝트·CI Node는 24.22.1 LTS로 맞췄다. Mac 기본 Node 26 계열도 별도로 보안 패치를 적용했다.
- MUI와 Emotion의 production chunk를 분리해 MUI 505.12→478.24kB로 줄이고 Vite 크기 경고를 해소했다. 거래·IPC·계정 정책을 변경하지 않았다.
- SQLx 0.9는 MSRV 1.94와 API·인증 정책 검토가 필요해 보류했다. 직접 reqwest 0.13, tokio-tungstenite 0.30, axum 0.8, tower-http 0.7, keyring 4, sha2 0.11, thiserror 2도 별도 major 호환 작업으로 남겼다. 해당 직접 의존성을 일괄 교체하지 않았다.
- React 19, MUI 9, Vite 8, React plugin 6, TypeScript 7, react-grid-layout 2는 UI·타입·번들 호환 검증 범위가 커서 보류했다. npm 12와 pnpm 12도 채택하지 않았으며 프로젝트의 패키지매니저는 npm을 유지했다.
- Homebrew는 관련 개발 도구와 그 의존성만 갱신하고 자동 cleanup·의존 앱 재검사를 끄고 진행했다. Node 의존성으로 OpenSSL 4.0.3이 추가됐고 `/opt/homebrew/bin/openssl`도 4.0.3을 가리킨다. 기존 `openssl@3` 3.6.4_1은 보존했으며 macOS 키체인·보안 설정은 변경하지 않았다. 무관한 앱 일괄 upgrade는 수행하지 않았다.
- Apple 공식 Xcode 27.1 (27A9275), 약 3.083GB, macOS 26.6 이상 요구를 확인했다. App Store UI 조회는 Mac 잠금 상태로 차단됐고, 이후 사용자가 나머지 작업 진행을 선택해 Xcode 27.1 설치를 명시 보류했다. 기존 27.0을 유지했다. 업데이트 없음은 `softwareupdate --list`의 OS/CLT 조회 결과이며 Xcode 최신판 설치 완료를 뜻하지 않는다.

### 이번 검증과 남은 경계

테스트는 합성·오프라인 모드이며 금융 자격증명을 사용하지 않았다. DB 접속 환경을 제거했다. 웹 build/E2E에는 저장소 `.env`를 읽지 않도록 빈 `envDir`를 쓰는 Git 제외 `.cache/offline-vite.config.mts`와 `.cache/offline-playwright.config.ts`를 사용했다.

| 검사 | 이번 결과 |
| --- | --- |
| Cargo metadata / check --workspace --all-targets --locked | PASS; workspace/MSRV metadata 확인, 컴파일 경고 0 |
| cargo test --workspace --lib --locked | PASS 775; DB contract 5개는 환경 미설정 즉시 반환, 실질 오프라인 770개 |
| cargo test --workspace --test lth_replay_clock --locked | PASS 4/4 |
| cargo clippy --workspace --all-targets --all-features --locked -- -D warnings | PASS |
| cargo fmt --all -- --check | PASS |
| cargo build --workspace --features custom-protocol --locked | PASS; 생성 바이너리 실행 안 함 |
| TypeScript / Web production build | PASS, Node 24.22.1; Vite 빌드 경고 0 |
| npm run test:e2e -- --config .cache/offline-playwright.config.ts --workers=2 | PASS Chromium mock 308/308, 7.9분 |
| production dist + Vite preview 집중 E2E | PASS 설정 DB·전략 assessment 8/8, 3.9초; mock transport |
| Linux / Mac production 집중 E2E, 모바일 수정 후 | 각각 PASS 60/60, 1.4분 / 51초; 같은 관련 spec 조합 |
| Linux 375px 모바일 집중 검사 | PASS 3/3, 5.2초 |
| 공식 Toss OpenAPI | PASS v1.2.24, 38개 path; 계좌·주문 호출 없음 |
| FSD / lockfiles / project-map / todo freshness | PASS |
| node --test scripts/render-todo.test.mjs | PASS 3/3 |
| Graphify update / freshness | PASS; AST topology 변경 없음, manifest·source hash만 갱신 |
| npm audit 변경 전후 | 전체 0→0 |
| cargo audit 변경 전후 | 취약점 12→1, Tokio 경고 해소; RSA 1개와 Linux 경고 2개 잔여 |
| 기존 RSA ignore를 적용한 정책 감사 | PASS; 예외·필터 추가 없음 |

E2E 로그의 `NO_COLOR`/`FORCE_COLOR` 안내와 Graphify JSON 설정 4개의 zero-node 안내는 남아 있으며 Rust/Vite 컴파일 경고와 구분한다. 보안 잔여의 상세 정책은 [현재 보안 게이트](release-security.md)를 따른다. 이번 DB 즉시 반환을 실제 DB 검증으로 집계하지 않는다. 10월 18일의 별도 로컬 TLS·단절·복구 완료 이력도 이번 재실행 결과로 주장하지 않는다.

production 집중 검증은 임시 설정의 작업 경로 오류를 고친 뒤 `.cache/offline-production-playwright.config.ts`로 `settings-database.spec.ts`와 `strategy-replay-assessment.spec.ts`를 실행했다. dev 서버 결과와 별도로 실제 dist의 MUI/Emotion 초기화·상호작용을 확인했으며 앱·브로커 실행 검증은 아니다.

첫 코드 커밋 `64ed2b60edcef54c6306f8266fb86afc973f1642`를 일반 푸시하고 원격 SHA 일치·당시 clean 상태를 확인했다. 동일 SHA의 [Dependency Security](https://github.com/CountJung/KISAutoTrade/actions/runs/40070512158)는 PASS였다. [Release Gate](https://github.com/CountJung/KISAutoTrade/actions/runs/40070510752)는 Rust·frontend·Toss PASS, E2E는 14.8분에 **279 통과·29 실패**였다. 이를 전체 CI 통과로 보고하지 않는다.

실패 중 26개는 dev 환경의 13개 전략 카드 초기화에서 assertion 5초 제한에 걸렸고, 초기화 성능이 원인이라는 판단은 아직 가설이다. mock 누락이 입증된 것은 아니다. 나머지 모바일 3개는 Linux에서 카드 너비 298px·scrollWidth 346px의 실제 가로 넘침을 재현했다. `src/pages/strategy/ui/Page.tsx`의 제목 왼쪽 flex 영역에 `minWidth: 0`·`flex: 1`, 제목에 `overflowWrap: anywhere`를 적용했다. 수정 후 Linux와 Mac의 관련 production spec 60개가 각각 모두 통과했고 Linux 375px 모바일 3개도 통과했다. 기존 Docker daemon에서 공식 Playwright 1.64.1-noble 이미지와 금융 파일 없는 git archive 격리 snapshot만 사용했으며 기존 컨테이너·서비스는 변경하지 않았다.

CI E2E job은 web build 후 production preview를 검사하도록 변경했다. 테스트·assertion 제한은 늘리지 않았고, 기존 기본 list reporter에서 빠졌던 HTML 보고서를 생성하며 실패 trace·screenshot을 `test-results/`와 함께 artifact로 보존한다. 로컬 dev 실행은 유지한다. **이 후속 수정의 최종 커밋·푸시·전체 CI는 이 기록 시점에 대기 중**이다. 최종 상태는 후속 Git 로그·원격 SHA와 해당 SHA의 CI 결과를 기준으로 확인한다.

로컬 증거는 `/tmp/kis-{lib-test,clock-test,clippy,build,web-build,e2e,production-e2e,graphify,npm-ci}.log`, `/tmp/kis-{linux-mobile-before,linux-overflow,linux-after,mac-final-e2e,linux-375,web-build-final,graphify-final,toss-openapi}.log`, `/tmp/kis-{npm,cargo}-audit-{before,after}.json`, `/tmp/kis-metadata-after.json`에 있다. `/tmp` 로그는 임시 근거이며 저장소 추적 산출물이 아니다. `origin/review/dependabot-prerequisites-20261024`의 `638cf064d88344f003f53699bca340683fd0194f` 보안 변경과 겹치는 부분이 있으므로 후속 통합에서 같은 수정을 중복 cherry-pick하지 않는다.

실행 중인 거래 프로세스는 발견되지 않았다. Chrome Remote Desktop과 LM Studio의 기존 프로세스는 유지했다. 실앱 실행, 주문·취소·포지션 변경, 거래 엔진 재시작, 실제 금융 API·운영 DB 접속, 사용자 거래 데이터 수정은 수행하지 않았다. 계정·네트워크·보안 설정도 변경하지 않았다. 로컬 `~/.codex/memories`는 비어 있고 관련 `memory_summary`를 찾지 못했으며 메모리를 수정하지 않았다. macOS/CLT 설치·재부팅·새 약관 동의는 실행하지 않았고 이번에 적용한 도구 업데이트로 재부팅하지 않았다. 실제 desktop·provider·운영 환경 검증은 별도 범위다.
