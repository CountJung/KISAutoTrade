# 릴리스·의존성 보안 게이트

이 문서는 비밀정보나 실계좌 연결 없이 실행되는 릴리스 전 검증을 설명한다.

## 자동화 범위

- `Release Gate`: TypeScript/FSD/프로젝트 맵, Rust `check`와 전체 lib test, Playwright, Toss 공식 OpenAPI drift를 병렬 검증한다.
- `Dependency Security`: main의 lockfile/보안 설정 push, lockfile 변경 PR 및 매주 월요일에 `cargo audit`와 npm audit를 실행한다.
  - 운영 npm 의존성은 `high` 이상을 차단한다.
  - 개발 의존성을 포함한 전체 트리는 `critical`을 차단한다. 그보다 낮은 개발 도구 취약점은 Dependabot PR에서 호환성을 검토한다.
- npm 설치는 `npm ci --ignore-scripts`, Rust 명령은 `--locked`를 사용한다. workspace 빌드·감사·버전 동기화의 유일한 Rust 기준은 루트 `Cargo.lock`이다. Cargo가 무시하던 과거 독립 crate 시절의 `src-tauri/Cargo.lock`은 제거한다.
- Dependabot은 npm, Cargo, GitHub Actions 업데이트를 매주 제안한다.

## 남은 RustSec 예외와 경고

**2026-10-24 점검**에서는 Mac 원본의 기존 semver 범위 안에서 lockfile을 갱신해 Cargo 취약점 12개 중 quinn-proto·rustls·webpki 계열 11개를 해소했다. Tokio unsound 경고도 해소했으며 아래 RSA 예외 1개와 Linux 경고 2개는 남는다. CI에는 기존 RSA 예외 하나만 유지하며 추가 ignore나 경고 필터를 두지 않는다. 2026-10-06의 quick-xml 예외 2개 해제는 기존 완료 이력이다.

| 항목 | 실제 경로와 잔여 이유 | 해제 조건 |
| --- | --- | --- |
| `RUSTSEC-2023-0071` (예외) | `sqlx-mysql 0.8.6 → rsa 0.9.10`. 이번 감사에도 수정된 안정 릴리스가 없어 잔여다. 앱은 서버 공개키 암호화를 사용하며 취약한 개인키 연산을 수행하지 않는다. SQLx 0.9 전환은 MSRV 1.94 및 API·인증 정책 호환 검토가 필요해 보류했다. RSA 기능 제거는 현재 지원하는 비 TLS MariaDB 인증의 기능 손실이므로 수행하지 않았다. | 수정된 RSA 구현으로 소비자가 전환하거나 사용자 승인하에 DB 인증 정책을 변경한 뒤 호환성 검증 |
| `RUSTSEC-2024-0429` (unsound 경고) | Linux 전용 `Tauri/tao/wry → GTK3 0.18 → glib 0.18.5`. 수정 버전은 ≥0.20이나 갱신한 Tauri 2.12.5의 안정 GTK 경로도 0.18이다. Mac target에는 포함되지 않는다. | Tauri 안정 GTK 의존성에서 수정된 GLib 사용 |
| `RUSTSEC-2024-0370` (unmaintained 경고) | Linux 전용 `glib-macros/gtk3-macros → proc-macro-error 1.0.4`. 소비자 매크로가 대체 crate를 채택해야 한다. | GTK/GLib 안정 매크로 의존성 업데이트 |

[RSA 공식 공지](https://rustsec.org/advisories/RUSTSEC-2023-0071.html), [GLib 공식 공지](https://rustsec.org/advisories/RUSTSEC-2024-0429.html), [매크로 공식 공지](https://rustsec.org/advisories/RUSTSEC-2024-0370.html). 담당자는 저장소 maintainer이며 Dependabot 주간 검토와 매 릴리스 전에 upstream 상태를 재확인한다. 예외 없는 `cargo audit`는 RSA 때문에 실패하고 기존 예외를 적용한 감사도 경고 두 개를 표시한다. 보안 경고 0개 달성으로 보고하지 않는다.

2026-10-06의 경고 12개 해소 이력과 2026-10-24의 취약점 12→1·버전·Mac 검증 근거는 [Mac 보안·동기화 보고서](security-mac-handoff-2026-10-06.md)에 날짜를 나누어 기록한다. 2026-10-24 npm 운영·개발 전체 감사는 변경 전후 모두 취약점 0개였으며 기존 차단 기준을 유지한다.

2026-10-29 reqwest0.13.5 분리 전환 후에도 raw 감사는 위 RSA1개로 실패했고 기존 예외만 적용한 감사는 통과했다. Linux 경고2개는 그대로이며 추가 ignore는 없다. RustSec DB revision과 실행 범위는 [즉시 작업5A 기록](tasks/immediate-backlog-20261029.md)을 따른다. 기존 HTTP의 Mozilla roots/ring을 보존했으며 시스템 신뢰 범위나 인증서 검증 정책을 변경하지 않았다.

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
  --ignore RUSTSEC-2023-0071
npm run verify:toss-openapi
npm run verify:release-artifacts -- --dir ./release-assets
```

OpenAPI 검증만 공식 Toss 문서를 읽기 위한 외부 HTTP GET을 수행한다. 테스트와 감사 작업은 API 키를 요구하지 않으며 주문 API를 호출하지 않는다.
