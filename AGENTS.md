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
| Claude Code 지침 (AGENTS.md import) | `CLAUDE.md` |
| Codex 프로젝트 브리지 스킬 | `.codex/skills/kisautotrade-*` |
| Claude Code 프로젝트 브리지 스킬 | `.claude/skills/kisautotrade-*` |

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
# UI 시각/상호작용 위험 변경: npm run test:e2e 또는 focused Playwright spec 실행
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
- 코드 리뷰 또는 문서 업데이트 조건이 충족되면 `.github/codex-instructions.md`의 위임 게이트를 따른다. subagent 도구가 있으면 review/documentation pass를 위임하고, 도구가 없으면 같은 체크리스트를 직접 수행한 뒤 다음 턴용 위임 프롬프트를 최종 보고에 남긴다.

---

## 최근 변경 요약

> 전체 이력은 `git log --oneline`. 여기는 최근 5건만 유지.

| 날짜 | 한줄 요약 |
|------|----------|
| 2026-07-08 | 좌측 사이드바 동작상태 chip 옆에 자동매매 시작/정지 전역 버튼 추가 |
| 2026-07-08 | 전략 소수 파라미터 spinner 조절 단위를 0.1로 통일 |
| 2026-07-08 | 레버리지 반등 전략에 초기 실패 손절과 수익 보호 청산 분리 적용 |
| 2026-07-08 | 레버리지 전략 청산을 수익 활성 후 본전 보호/추적손절 모델로 전환 |
| 2026-07-08 | 레버리지 전략 UI에 추적손절 설정과 파라미터 튜닝 가이드 추가 |
