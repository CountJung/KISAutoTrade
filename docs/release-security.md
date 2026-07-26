# 릴리스·의존성 보안 게이트

이 문서는 비밀정보나 실계좌 연결 없이 실행되는 릴리스 전 검증을 설명한다.

## 자동화 범위

- `Release Gate`: TypeScript/FSD/프로젝트 맵, Rust `check`와 전체 lib test, Playwright, Toss 공식 OpenAPI drift를 병렬 검증한다.
- `Dependency Security`: lockfile 변경 PR 및 매주 월요일에 `cargo audit`와 npm audit를 실행한다.
  - 운영 npm 의존성은 `high` 이상을 차단한다.
  - 개발 의존성을 포함한 전체 트리는 `critical`을 차단한다. 그보다 낮은 개발 도구 취약점은 Dependabot PR에서 호환성을 검토한다.
- npm 설치는 `npm ci --ignore-scripts`, Rust 명령은 `--locked`를 사용한다. workspace 빌드와 감사의 기준은 루트 `Cargo.lock`이다. 과거 독립 crate 시절부터 추적된 `src-tauri/Cargo.lock`은 의도치 않은 삭제를 막기 위해 존재 여부와 기본 형식만 검증하며, 현재 workspace dependency 기준으로 사용하지 않는다.
- Dependabot은 npm, Cargo, GitHub Actions 업데이트를 매주 제안한다.

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
cargo audit --file Cargo.lock
npm run verify:toss-openapi
npm run verify:release-artifacts -- --dir ./release-assets
```

OpenAPI 검증만 공식 Toss 문서를 읽기 위한 외부 HTTP GET을 수행한다. 테스트와 감사 작업은 API 키를 요구하지 않으며 주문 API를 호출하지 않는다.