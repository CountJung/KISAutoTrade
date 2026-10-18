# 작업 기록: 루트 문서 정리

## 1. 메타데이터

- 요청: README와 에이전트 진입 문서만 루트에 남기고 일반 문서를 docs로 배치
- 작성일: 2026-10-18 KST
- 담당: Codex / symbol_navigator / project_mapper / 독립 문서 리뷰
- 상태: Done — 문서 이동·참조 갱신·최종 검증 완료
- 시작: `fix/mac-security-storage-2026-10-06`, `cd35389`, 작업 트리 clean
- 위험: 낮음 — 문서/참조 경로 정리, 생산 동작 변경 없음

## 2. 목표와 범위

- [x] 루트 문서는 README.md·AGENTS.md·CLAUDE.md만 유지한다.
- [x] MASTER_PLAN.html·PROJECT_MAP.md·ARCHITECTURE.md·HARNESS_MAP.md·todo.md를 docs로 이동한다.
- [x] 기존 작업 기록과 템플릿을 docs/tasks로 이동한다.
- [x] 상대 링크와 에이전트/스킬/메모리의 문서 참조를 갱신한다.
- [x] 인벤토리·링크·경고0 검사와 독립 문서 리뷰를 통과했다. 게시 기록은 현재 브랜치 Git 이력으로 확인한다.

소스·빌드 manifest·런타임 index.html·LICENSE와 도구 고유 진입 위치는 유지한다. `.env`, `secure_config.json`, `profiles.json`은 읽지 않았다. 기존 연구 입력/결과/CSV·코드·검증 JSON은 바꾸지 않는다.

## 3. 이동과 계약

| 이전 루트 경로 | 현재 위치 |
|---|---|
| MASTER_PLAN.html | [단계 계획](../MASTER_PLAN.html) |
| PROJECT_MAP.md | [작업별 지도](../PROJECT_MAP.md) |
| ARCHITECTURE.md | [아키텍처](../ARCHITECTURE.md) |
| HARNESS_MAP.md | [검증 명령](../HARNESS_MAP.md) |
| todo.md | [할일](../todo.md) |
| tasks/ | docs/tasks/ |

PROJECT_MAP.md는 underscore, 기존 자동 지도 project-map.md는 hyphen을 사용해 파일명이 구별된다. `scripts/project-map.mjs`의 출력 경로는 원래 docs/project-map.md이므로 스크립트 수정은 필요 없었다. README에 문서 탐색 표를 추가했다.

Tauri IPC·웹 transport·serde/TypeScript·query/cache·계정/주문/리스크·저장 schema 변경은 모두 N/A다. Rust contract test의 TODO 주석 경로만 갱신했다. 실전·모의 주문 및 자동매매 시작은 실행하지 않았다.

## 4. 조사·위임·보존

- symbol_navigator가 스크립트/도구의 문서 참조와 이동에 따른 상대 링크 영향을 읽기 전용으로 조사했다.
- project_mapper가 새 문서/작업 기록 인벤토리와 수동 책임 설명을 반영했고 지도 검증을 통과했다.
- 독립 리뷰가 루트 문서 목록·진입 경로·링크·연구 artifact 보존을 확인했다. 수정 요구 finding 없음.
- `.codex/`의 보호된 역할/스킬 경로 수정은 필요한 권한 확장 후 수행했다.
- 연구 JSON/JSONL/CSV/Rust/Python 원본의 SHA-256을 이동 전후 비교해 20개 파일 모두 동일함을 확인했다. 루트 정리 때문에 기존 연구 validation hash를 재작성하지 않는다.
- Graphify는 코드 구조 이동이 없어 전체 refresh 대신 주석 변경에 맞춘 일반 update/freshness 확인을 수행한다.

## 5. 검증

| 상태 | 확인 | 결과 |
|---|---|---|
| PASS | 루트 문서 allowlist·기존 경로 없음 | README/AGENTS/CLAUDE, 앱 index.html 구분 |
| PASS | Markdown/HTML 상대 링크·진입 문서 경로 | 전체 검사110개, 독립 감사107개; 깨진 대상0 |
| PASS | 연구 SHA-256 | 20개 보관 자료/기존 연구 결과 불변 |
| PASS | project-map update/check | mapper가 전체 신규 위치 반영 |
| PASS | graphify update/check | 코드 주석 경로 반영, freshness 일치 |
| PASS | cargo check / TypeScript | workspace all-targets locked / tsc noEmit, 코드 경고0 |
| PASS | diff 공백·독립 리뷰 | 수정 요구 finding 없음 |
| NOT RUN | Playwright·실제 주문 | UI/실행 동작 변경 없음 |

Graphify의 symbol 없는 JSON 설정 4개 진단은 기존 도구 분석 한계이며 Rust/TypeScript 코드 경고와 구분한다. 생산 동작 변경이 없어 이번 작업에서 Rust 단위/E2E 테스트는 재실행하지 않았다.

## 6. 전달·롤백

문서 탐색은 루트 README/AGENTS/CLAUDE에서 시작한다. 변경 취소는 문서 정리 커밋을 revert하고 지도/graph freshness를 확인한다. 기존 연구 artifact와 실운영 설정은 보존했다. 운영 배포/모의·소액 검증은 이번 문서 작업의 대상이 아니다.
