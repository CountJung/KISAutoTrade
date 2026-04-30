# AutoConditionTrade — 에이전트 가이드

> 작업 전 이 파일을 읽고 시작한다.  
> 상세 정보는 아래 링크의 문서를 참조한다.

---

## 빠른 참조

| 항목 | 경로 |
|------|------|
| 디렉토리 맵 + 아키텍처 | `docs/project-map.md` |
| IPC 커맨드 목록 (35개+) | `docs/ipc-commands.md` |
| 코딩 가이드 (AppState·IPC·데몬) | `docs/coding-guide.md` |
| KIS API 스킬 | `.github/skills/kis-api/SKILL.md` |
| Rust 코딩 스킬 | `.github/skills/rust-skills/SKILL.md` |
| React/Tauri 성능 스킬 | `.github/skills/react-best-practices/SKILL.md` |
| UI 컨벤션 스킬 | `.github/skills/ui-conventions/SKILL.md` |
| 에이전트 지침 | `.github/copilot-instructions.md` |

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
cd src-tauri && cargo check          # Rust 빠른 검증
npx tsc --noEmit                     # TypeScript 타입 체크
```

**경고 0개** 달성 후 완료 보고.

---

## 최근 변경 요약

> 전체 이력은 `git log --oneline`. 여기는 최근 5건만 유지.

| 날짜 | 한줄 요약 |
|------|----------|
| 2026-05-01 | 앱 종료 안전 처리(on_window_event), REFRESH_INTERVAL_SEC .env 통일, agents.md 문서 구조 개편 |
| 2026-04-14 | 해외주식 모의투자 매도 에러 처리: is_paper_unsupported_error() 추가 |
| 2026-04-12 | Dashboard 체결내역 실시간갱신(30s)+페이지네이션, 예수금 D+2 우선 표시 |
| 2026-04-12 | 모바일 BottomNavigation 추가, Sidebar 드로어 자동 닫힘 |
| 2026-04-11 | run_trading_daemon 레이블 루프 제거(poll_symbols_tick 분리), coding-guide.md 신규 |
