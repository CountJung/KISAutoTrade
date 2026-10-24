# 릴리스·의존성 보안 게이트

이 문서는 비밀정보나 실계좌 연결 없이 실행되는 릴리스 전 검증을 설명한다.

## 자동화 범위

- `Release Gate`: TypeScript/FSD/프로젝트 맵, Rust `check`와 전체 lib test, Playwright, Toss 공식 OpenAPI drift를 병렬 검증한다.
- `Dependency Security`: main의 lockfile/보안 설정 push, lockfile 변경 PR 및 매주 월요일에 `cargo audit`와 npm audit를 실행한다.
  - 운영 npm 의존성은 `high` 이상을 차단한다.
  - 개발 의존성을 포함한 전체 트리는 `critical`을 차단한다. 그보다 낮은 개발 도구 취약점은 Dependabot PR에서 호환성을 검토한다.
- npm 설치는 `npm ci --ignore-scripts`, Rust 명령은 `--locked`를 사용한다. workspace 빌드·감사·버전 동기화의 유일한 Rust 기준은 루트 `Cargo.lock`이다. Cargo가 무시하던 과거 독립 crate 시절의 `src-tauri/Cargo.lock`은 제거한다.
- Dependabot은 npm, Cargo, GitHub Actions 업데이트를 매주 제안한다.

## 임시 RustSec 예외

`Dependency Security`는 다음 세 advisory만 명시적으로 예외 처리하고 그 외 advisory는 계속 차단한다.

| Advisory | 경로와 완화 근거 | 해제 조건 |
| --- | --- | --- |
| `RUSTSEC-2026-0194`, `RUSTSEC-2026-0195` | `quick-xml 0.38.x ← plist 1.8.0 ← Tauri`. 현재 앱은 이 경로로 외부 사용자 XML을 파싱하지 않으며, `plist`의 `quick-xml ^0.38` 제약 때문에 수정 버전 `0.41+`를 선택할 수 없다. | `plist`/Tauri가 `quick-xml >=0.41`을 허용하면 즉시 업데이트하고 예외 제거 |
| `RUSTSEC-2023-0071` | `rsa 0.9.10 ← sqlx-mysql`. RustSec에 수정 릴리스가 없고, 앱은 MySQL 클라이언트의 서버 공개키 암호화만 사용하며 취약한 RSA 개인키 연산을 수행하지 않는다. | `sqlx-mysql`이 수정된 RSA 구현으로 전환하면 예외 제거 |

최초 검토일은 2026-07-26이며 담당자는 저장소 maintainer다. Dependabot 주간 검토와 매 릴리스 전에 예외의 upstream 상태와 실제 호출 경로를 다시 확인한다.

## 릴리스 artifact 무결성

태그 릴리스는 모든 플랫폼 빌드가 끝난 뒤 설치 파일을 다시 내려받아 다음을 확인한다.

1. 지원 확장자의 artifact가 하나 이상이고 빈 파일이 아닌지 확인한다.
2. 각 파일의 SHA-256을 `SHA256SUMS`에 기록한다.
3. 기계 판독용 크기·해시·서명 sidecar 정보를 `release-artifacts.json`에 기록한다.
4. GitHub artifact attestation을 발급하고 두 무결성 파일을 draft release에 업로드한다.

`verify-release-artifacts.mjs`는 `<artifact>.sig`가 있으면 대상 존재 여부와 빈 서명이 아님을 검증한다. 저장소 변수 `REQUIRE_RELEASE_SIGNATURES=true`를 설정하면 릴리스 workflow가 `--require-signatures`를 추가해 모든 artifact의 detached signature를 강제한다. 서명 생성 단계와 배포 키를 먼저 GitHub Environment에 준비한 뒤 이 변수를 활성화해야 한다. 현재 checksum/attestation은 활성화되지만 플랫폼 코드 서명이나 updater 서명을 대신하지 않는다.

```bash
npm ci --ignore-scripts
npm run check:lockfiles
npm audit --omit=dev --audit-level=high
npm audit --audit-level=critical
cargo metadata --locked --format-version 1 --no-deps >/dev/null
cargo audit --file Cargo.lock \
  --ignore RUSTSEC-2023-0071 \
  --ignore RUSTSEC-2026-0194 \
  --ignore RUSTSEC-2026-0195
npm run verify:toss-openapi
npm run verify:release-artifacts -- --dir ./release-assets
```

OpenAPI 검증만 공식 Toss 문서를 읽기 위한 외부 HTTP GET을 수행한다. 테스트와 감사 작업은 API 키를 요구하지 않으며 주문 API를 호출하지 않는다.

## Dependabot #13/#11 검토 (2026-10-24)

기준 main은 `8db7bf8f2631827ce007a8a4821541df15e1ff32`, #13은 `f8666053dd24a1033812a0de50061c621fdcec88`, #11은 `b29c769230dcfb3c4aab708937aad26c47025463`이다. 두 PR은 열린 상태이며 이 검토는 병합 완료 기록이 아니다.

### 실제 변경과 호환성

| 영역 | 확인 사항 | 검증 경계 |
| --- | --- | --- |
| Rust/Tauri #13 | Cargo.lock만 변경. 12개 직접 업데이트 외 112개 crate 이름의 버전 집합이 달라진다(새 이름 7, 제거 16 포함). Tauri 2.10.3→2.12.3, tauri-runtime/tauri-runtime-wry 2.10.1→2.12.1, wry 0.54.4→0.57.0도 포함된다. | 작은 패치 12개로 축소해서 판단하지 않는다. |
| Rust 최소 버전 | 다운로드한 Cargo metadata에서 #13 Tauri·플러그인·build는 1.90, uuid는 1.89를 요구한다. #11의 기존 Rust 트리도 time/cookie_store/serde_with 등이 1.88을 요구한다. | 1.78 선언은 이미 부정확하다. 선언은 1.90으로 맞추되 1.90 실제 컴파일은 미검증이다. |
| Node 최소 버전 | #11의 history/react-router/router-core는 Node >=20.19, Playwright 1.64는 >=20을 요구한다. main의 Router 1.168.10도 이미 >=20.19를 요구하므로 기존 선언 교정이다. | package.json과 lockfile 루트 engines를 >=20.19.0으로 맞춘다. 로컬 Node 24 검증과 최소 환경 검증은 다르다. |
| Tauri JS/플러그인 | #11 API/CLI 2.12.1, JS shell 2.4.1. #13 Rust shell 2.4.1/window-state 2.5.0. 앱은 기존 core invoke/event API를 사용하고 JS shell import는 없다. | 숫자 차이만으로 비호환을 확정하지 않는다. native IPC/event·창 상태 저장/복원·종료는 별도 검증한다. |
| 상태/세션 | Query 5.96.1→5.104.1, Router 1.168.10→1.170.41, Zustand 5.0.12→5.0.15. polling, query key/cache invalidation, 계좌 scope, event listener가 영향 범위다. 웹 세션 토큰은 sessionStorage, KIS 토큰은 백엔드 캐시다. | 상태 로직 자체는 바뀌지 않는다. 재연결·새로고침·토큰 만료·scope 전환은 mock/fixture로 확인해야 한다. |
| 차트 | lightweight-charts 5.1.0→5.2.1. 기존 코드는 v5 addSeries/createSeriesMarkers를 사용한다. | 타입 통과나 preview canvas 존재는 국내·해외 차트 축/정렬/마커/resize 정확성을 전부 증명하지 않는다. |
| 통신/비동기 | #13 Tokio 1.50→1.53.2와 futures-util 0.3.32→0.3.34. Axum TCP listener, WebSocket read loop와 spawn/lock/timeout 작업이 영향 범위다. reqwest와 tokio-tungstenite 버전은 그대로다. | WS 끊김·재구독·backpressure·종료 시 작업 정리와 TLS 인증은 로컬 fixture 대상이다. 실 provider 접속은 하지 않는다. |
| 로그/자료 | tracing-appender·chrono·serde·uuid가 바뀐다. | rolling/flush, JSON round-trip, 날짜·DST·중복 ID 및 replay 자료 처리를 확인해야 한다. 라이브러리 업데이트만으로 거래 정확성을 보장하지 않는다. |

### 기존 CI와 로컬 검증의 구분

GitHub 로그를 직접 확인했다. #13 [Release Gate 38108882225](https://github.com/CountJung/KISAutoTrade/actions/runs/38108882225)는 main과의 merge SHA `b85adc4ee66d1bbd897ce8e3a62cbbc9506886d0`, #11 [Release Gate 38108845479](https://github.com/CountJung/KISAutoTrade/actions/runs/38108845479)는 merge SHA `da8af000244f248d7dafbc6f516ad438b9dc36f2`를 검사했다. 두 실행 로그 날짜는 10월 11일이며 PR metadata 갱신일 10월 18일과 구별한다.

| 검사 | #13 기존 CI | #11 기존 CI | 이번 격리 환경 |
| --- | --- | --- | --- |
| npm ci / TypeScript / FSD / project-map | 통과 | 통과 | 두 PR 모두 통과 |
| 웹 production build | 해당 job에 없음 | 해당 job에 없음 | 두 PR 통과. #11 MUI chunk 501.99 kB 경고 잔여 |
| cargo metadata --locked | 통과 | 통과 | 두 PR 전체 metadata 통과, 원본 lockfile 변경 없음 |
| cargo check/test | Rust 1.99.0에서 통과, lib 190 passed | Rust 1.99.0에서 통과, lib 190 passed | Rust 1.99.1에서 시도했으나 pkg-config/GTK/WebKit 개발 패키지 부재로 컴파일 단계 차단 |
| strict clippy | workflow에 없음 | workflow에 없음 | 동일 시스템 의존성 문제로 미검증 |
| Playwright Chromium | Node 20.20.2, mock 30/30 | Node 20.20.2, mock 30/30 | 두 버전 Chromium 다운로드가 ZIP 해석 오류로 실패. 로컬 회귀 통과 주장 불가 |
| npm 보안 정책 | seroval critical로 실패 | 통과 | 원본 audit 확인. 개발 취약점은 정책 통과와 별도 기록 |
| cargo audit | rustls 0.23.37로 실패 | rustls 0.23.37로 실패 | 기존 로그 원인 확인, 새 로컬 audit는 미검증 |
| Toss 공식 명세 | 33개 예상/38개 실제로 실패 | 같은 실패 | 원본 실패 재현. 공급자 v1.2.24의 sector 관련 5개 추가 경로를 기존 개발 브랜치에서 재사용 |

CI의 190 passed에는 DB 환경이 없으면 즉시 반환하는 PostgreSQL contract 3개가 포함될 수 있다. 실제 외부 DB 통과로 해석하지 않는다. 두 PR은 앱 코드를 바꾸지 않았지만 기존 mock E2E는 실제 Tauri IPC·WS·TLS·계좌 검증을 대체하지 않는다.

로컬 환경은 Ubuntu 24.04, Node 24.22.1/npm 11.9.0, Rust 1.99.1이다. 시스템 패키지 설치는 setgroups/setuid 권한 오류로 실패했으며 권한을 확대하지 않았다. .env/secure_config.json/profiles.json을 읽거나 실제 앱·실계좌 주문을 실행하지 않았다.

### 선행 수정과 병합 순서

1. 기존 `0812c715e66860ca837273b1a7c14d7cbfe278ba`의 최소 lockfile 보안 수정을 재사용하고 최신 advisory에 맞춰 rustls 0.23.46, rustls-webpki 0.103.16, webpki-roots 1.0.10, quinn-proto 0.11.18까지 좁게 갱신한다. npm은 seroval 1.6.8, nanoid 3.3.20, source-map-js 1.2.2를 유지한다. 뒤따르는 Windows 저장·전략 기능 변경은 섞지 않는다. 최소 환경 선언과 Toss sector 경로 목록도 교정한다. 기존 보안 예외와 검사는 유지한다.
2. 선행 수정 통과 후 #13을 최신 main에 통합해 Rust 1.90/stable check·test·strict clippy, 보안 정책, 기존 프런트엔드 조합과 native smoke를 확인한다. 그 뒤 #13만 병합한다.
3. #11을 그 main에 갱신해 Node 20.19와 개발 환경의 타입·production build·보안 정책·30개 이상 mock E2E를 확인한다. 최종 새 backend/새 frontend 조합의 IPC·세션·차트·WS fixture 통과 후 #11을 병합한다.

단일 PR의 기존 CI 성공을 결합 결과로 간주하지 않는다. 최신 개발 브랜치 `fix/mac-security-storage-2026-10-06`에는 더 많은 보안·Clippy 수정과 전략 작업이 있으므로 통째로 main에 가져오지 않는다. 필수 검증이 막힌 단계는 열린 작업 DEP-20261024로 유지한다.

### 선행 수정 로컬 재검증

`review/dependabot-prerequisites-20261024`에서 npm ci, TypeScript, FSD, project-map, 웹 production build, Cargo metadata --locked --no-deps, Toss 38개 경로 검사가 통과했다. npm audit --omit=dev --audit-level=high는 운영 취약점 0으로, 전체 --audit-level=critical도 통과했다. 전체 감사에는 개발용 Vite high 1개/esbuild moderate 1개가 남는다. Vite major 업데이트는 별도 검토 대상이며 이번 수정에서 보안 임계값을 낮추지 않았다.

docs-canon 형식 검사와 별도 read-only diff/문서 검토를 수행했다. 깨진 상대 링크·표 열 오류는 없고 기존 TODO 완료 이력 경고는 유지했다. HARNESS_MAP에서 이 정본으로 연결해 도달성을 보완했다. Rust 빌드/테스트/clippy·최소 환경 컴파일·native 앱·실제 DB·실계좌 검증은 이 수정의 로컬 통과 항목에 포함하지 않는다.

### 최신 감사와 결합 점검

선행 초안 PR #14의 최초 SHA `00ce3c981ad13a7ed1c0ddff94799ba8500ea778`에 대한 [Dependency Security 40065354394](https://github.com/CountJung/KISAutoTrade/actions/runs/40065354394)에서 npm 정책은 통과했지만, 10월 11일 로그 이후 발행된 11개 Rust advisory로 감사가 실패했다. 기존 rustls 0.23.45 패치만으로 현재 보안 검사를 통과한다고 해석하면 안 된다.

- quinn-proto 0.11.15→0.11.18: RUSTSEC-2026-0338/0339/0340/0341/0342. quinn의 기존 semver 제약을 유지하지만 해당 패치가 요구하는 rand 0.10.3/rand_core 0.10.1/rand_pcg 0.10.2 및 chacha20/cpufeatures 전이 변경도 포함된다.
- rustls 0.23.45→0.23.46: RUSTSEC-2026-0346/0347/0348/0349.
- rustls-webpki 0.103.15→0.103.16 및 webpki-roots 1.0.6→1.0.10: RUSTSEC-2026-0382/0381 인증서 신뢰 교정.

공식 근거: [Rustls](https://rustsec.org/advisories/RUSTSEC-2026-0346.html), [Quinn](https://rustsec.org/advisories/RUSTSEC-2026-0340.html), [WebPKI](https://rustsec.org/advisories/RUSTSEC-2026-0382.html), [Roots](https://rustsec.org/advisories/RUSTSEC-2026-0381.html). 새 예외는 추가하지 않는다. Tokio 1.50의 RUSTSEC-2026-0386을 포함한 기존 경고도 임의로 숨기지 않으며 #13 및 후속 검토에서 다시 확인한다.

최초 선행 수정 SHA에 #13→#11의 원본 diff를 3-way 적용한 별도 작업본은 충돌 없이 통합됐고 npm ci/TypeScript/웹 빌드/lockfiles/FSD/project-map 및 전체 Cargo metadata를 통과했다. #11과 결합 빌드에는 MUI chunk 크기 경고가 남았다. 이 결과는 위 최신 TLS·QUIC 후속 갱신 이전의 결합 검사이며 최종 통합 SHA의 검증을 대체하지 않는다.
