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
