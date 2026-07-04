# KISAutoTrade — Codex 에이전트 가이드

> 작업 전 이 파일을 읽고 시작한다.  
> 상세 정보는 아래 링크의 문서를 참조한다.

---

## 빠른 참조

| 항목 | 경로 |
|------|------|
| 디렉토리 맵 + 아키텍처 | `docs/project-map.md` |
| 개선 백로그 | `todo.md` |
| IPC 커맨드 목록 (35개+) | `docs/ipc-commands.md` |
| 코딩 가이드 (AppState·IPC·데몬) | `docs/coding-guide.md` |
| KIS API 스킬 | `.github/skills/kis-api/SKILL.md` |
| Toss API 스킬 | `.github/skills/toss-api/SKILL.md` |
| Rust 코딩 스킬 | `.github/skills/rust-skills/SKILL.md` |
| React/Tauri 성능 스킬 | `.github/skills/react-best-practices/SKILL.md` |
| Frontend FSD 스킬 | `.github/skills/frontend-fsd/SKILL.md` |
| UI 컨벤션 스킬 | `.github/skills/ui-conventions/SKILL.md` |
| Codex 상세 지침 | `.github/codex-instructions.md` |
| Copilot 호환 지침 | `.github/copilot-instructions.md` |
| Codex 프로젝트 브리지 스킬 | `.codex/skills/kisautotrade-*` |

---

## 핵심 파일 경로

| 역할 | 파일 |
|------|------|
| Tauri IPC 커맨드 + AppState | `src-tauri/src/commands.rs` |
| 백그라운드 데몬 + Builder | `src-tauri/src/lib.rs` |
| KIS REST Client | `src-tauri/src/api/rest.rs` |
| 전략 엔진 | `src-tauri/src/trading/strategy.rs` |
| React 훅 (TanStack Query) | `src/api/hooks.ts` |
| TypeScript 타입 미러 | `src/api/types.ts` |
| invoke() 래퍼 | `src/api/commands.ts` |

---

## 빌드 / 검증

```powershell
cd src-tauri; cargo check            # Rust 빠른 검증
cd ..; npx tsc --noEmit              # TypeScript 타입 체크
```

**경고 0개** 달성 후 완료 보고.

---

## Codex 작업 원칙

- `.env`, `secure_config.json`, `profiles.json`은 읽지 않는다.
- 코드 변경 전 현재 구현을 먼저 검색하고, 기존 패턴을 우선한다.
- KIS API 동작·TR-ID·제한사항은 추측하지 말고 공식 포털 또는 `koreainvestment/open-trading-api` 샘플로 확인한다.
- 새 IPC 커맨드는 Rust command, `lib.rs` 등록, TypeScript 타입/래퍼/훅, 문서를 함께 갱신한다.
- 반복 매매·손실 방지 관련 변경은 `todo.md`와 관련 스킬 문서에 남긴다.
- Copilot 호환 지침과 `.github/skills/**`의 원본 스킬은 프로젝트 브리지 스킬(`.codex/skills/kisautotrade-*`)을 통해 재사용한다. Codex 런타임이 계정 스킬만 읽는 경우 `scripts/sync-codex-skills.ps1`로 동기화한다. 원본은 저장소의 `.github/skills/**/SKILL.md`로 유지한다.

---

## 최근 변경 요약

> 전체 이력은 `git log --oneline`. 여기는 최근 5건만 유지.

| 날짜 | 한줄 요약 |
|------|----------|
| 2026-07-04 | 자동매매 주문 제출 경로를 lock-short shared submission으로 분리 |
| 2026-07-04 | broker/toss.rs를 client·adapter·orders·types 등 하위 모듈로 분리 |
| 2026-07-04 | strategy.rs facade 분리와 전략 버퍼 상한/target helper 적용 |
| 2026-07-04 | commands.rs facade 1000라인 이하 분리와 전략 config 갱신 재빌드 적용 |
| 2026-07-04 | commands 자동매매·전략·리스크 IPC 추가 분리와 Toss 응답 body streaming cap 적용 |
