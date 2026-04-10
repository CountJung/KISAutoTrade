# AutoConditionTrade — 프로젝트 지침

개인용 자동 주식 매매 시스템. **Rust (Tauri v2) + React 18 + TypeScript** 풀스택.  
작업 전 반드시 `agent.md`를 읽어 현재 구조를 파악한다.

---

## 아키텍처

```
React UI  →  Tauri IPC  →  Rust Backend  →  KIS Open API (REST + WebSocket)
                                        →  Discord Bot API
                                        →  JSON 파일 Storage (연/월/일 폴더)
```

| 역할 | 경로 |
|------|------|
| Tauri IPC 커맨드 | `src-tauri/src/commands.rs` |
| KIS REST Client | `src-tauri/src/api/rest.rs` |
| Token Manager | `src-tauri/src/api/token.rs` |
| WebSocket | `src-tauri/src/api/websocket.rs` |
| 전략 엔진 | `src-tauri/src/trading/strategy.rs` |
| KRX 종목 목록 | `src-tauri/src/market/mod.rs` |
| 웹 서버 (axum) | `src-tauri/src/server/mod.rs` |
| 업데이트 확인 | `src-tauri/src/updater/mod.rs` |
| React 훅 | `src/api/hooks.ts` |
| TS 타입 미러 | `src/api/types.ts` |

---

## 빌드 / 검증

```powershell
# Rust 빠른 검증 (build 대신 우선 사용)
cd src-tauri && cargo check

# TypeScript 타입 체크
npx tsc --noEmit

# Vite 프로덕션 빌드
npx vite build
```

모든 변경 후 `cargo check` → `npx tsc --noEmit` 순서로 검증한다.  
**경고(warning) 0개**를 목표로 한다. 경고가 남아있으면 완료로 보고하지 않는다.

---

## 사실 검증 우선 원칙

> **추측으로 사용자 요청을 기각하지 않는다.** 사용자가 어떤 사실을 주장할 때, 내부 지식이나 이전 분석과 다르더라도 즉시 기각하지 말고 **외부 소스·도구·코드베이스를 검색하여 먼저 사실을 검증**한 뒤 작업을 진행한다.

### 적용 예시

| 상황 | 잘못된 대응 | 올바른 대응 |
|------|-----------|-----------|
| "KRX 종목코드에 알파벳이 들어갈 수 있다" | "KRX는 6자리 숫자만 사용합니다" 기각 | 실제 종목(예: `0005A0`, `0089C0`) 검색 후 사실 확인 → 코드 수정 |
| "이 API는 이런 필드를 반환한다" | 내부 지식으로 부정 | KIS 포털/샘플코드/웹페이지 fetch로 확인 후 판단 |

### 검증 절차

1. `fetch_webpage` 또는 웹 검색으로 외부 사실 확인
2. 코드베이스 내 관련 파일 검색으로 현재 구현 상태 파악
3. 사실이 확인되면 즉시 관련 코드·문서·스킬 파일 업데이트
4. 잘못된 기존 지식은 스킬 파일(`kis-api/SKILL.md` 등)에서 수정하고 `❌ 잘못된 패턴` 섹션을 추가한다
5. **KIS API 에러코드·응답 필드·동작 특이사항을 새로 발견하거나 구현했을 때는 반드시 `kis-api/SKILL.md`의 해당 섹션을 즉시 업데이트하거나 신규 섹션을 추가한다** — 다음 에이전트 세션의 재발견 비용 제거

---

## 핵심 규칙

### Rust

- `cargo check` 성공 없이 코드 완료로 보고하지 않는다
- `cargo check` 및 빌드에서 발생하는 **경고(warning)도 반드시 해소**한다. `#[allow(...)]` 어트리뷰트는 최후 수단으로만 사용하며, 사용 시 이유를 주석으로 명시한다
- IPC 커맨드 반환 타입: `CmdResult<T>` = `Result<T, CmdError { code, message }>`
- 공유 상태: `Arc<RwLock<T>>` (async read-heavy), `Arc<Mutex<T>>` (write)
- JSON 직렬화는 `#[serde(rename_all = "camelCase")]` ← TypeScript 인터페이스와 1:1 매핑
- 에러 처리: `thiserror` + `?` 연산자. `unwrap()` 최소화
- 새 IPC 커맨드 추가 시 반드시 `lib.rs`의 `generate_handler!`에 등록

### TypeScript / React

- TypeScript 컴파일 경고(unused variable, implicit any 등)도 **반드시 해소**한다. `// @ts-ignore`, `as any` 캐스트는 최후 수단으로만 사용하며, 사용 시 이유를 주석으로 명시한다
- MUI 아이콘: **직접 경로** import 필수 (`@mui/icons-material/PlayArrow`)
- TanStack Query 훅은 `src/api/hooks.ts`에 집중 관리, `KEYS` 상수 사용
- 새 Tauri invoke 래퍼는 `src/api/commands.ts`에 추가
- Zustand store는 `src/store/` 하위 파일별 분리 유지
- 파생 상태는 `useState` + `useEffect` 대신 렌더 중 직접 계산

### 보안

- `appkey`, `appsecret`, Discord 토큰을 코드에 하드코딩 금지
- 민감 정보는 `secure_config.json` (gitignore) 또는 `.env` (gitignore) 에만 저장
- 실계좌 테스트 전 반드시 `KIS_IS_PAPER_TRADING=true` 로 모의투자 검증
- **`.env`, `secure_config.json`, `profiles.json` 파일은 어떠한 경우에도 AI 컨텍스트(파일 읽기·분석·출력)로 사용하지 않는다** — API 키 노출 사고 방지

### 데이터 저장

- DB 없음 — JSON 파일만 사용
- 경로 패턴: `{app_data_dir}/data/{category}/{YYYY}/{MM}/{DD}/{file}.json`
- `data/`, `log/`, `.env`, `secure_config.json`은 `.gitignore`에 포함되어야 한다
- `.cargo/config.toml`은 gitignore에 포함 — macOS 외장 드라이브(exFAT) 사용 시 `scripts/setup-local.sh` 실행으로 자동 생성

---

## KIS API 엔드포인트 (Rust 구현 기준)

| 환경 | Base URL |
|------|----------|
| 실전 | `https://openapi.koreainvestment.com:9443` |
| 모의 | `https://openapivts.koreainvestment.com:29443` |
| WebSocket | `wss://openapi.koreainvestment.com:9443/websocket/client` |

- 계좌번호: 10자리 → CANO(앞 8자리) + ACNT_PRDT_CD(뒤 2자리) 분리
- 초당 API 호출 제한: 실전 20건, 모의 2건 — 연속 호출 시 sleep 필요
- TR-ID는 실전/모의 별도 값 사용 (VTTC... / TTTC...)

---

## 스킬 참조

| 작업 | 스킬 |
|------|------|
| React 성능 최적화, waterfall, MUI 번들 | `.github/skills/react-best-practices/SKILL.md` |
| Rust 데이터 구조, trait, 에러 처리, serde | `.github/skills/rust-skills/SKILL.md` |
| KIS API 인증, REST/WS 호출, tr_id, 에러코드 | `.github/skills/kis-api/SKILL.md` |
| MUI v6 컴포넌트, 차트, 색상, 금융 UI 컨벤션 | `.github/skills/ui-conventions/SKILL.md` |

---

## 살아있는 문서 원칙 (Living Documentation)

`agent.md`, `copilot-instructions.md`, `.github/skills/**` 는 **항상 현재 코드베이스를 정확히 반영하는 살아있는 문서**여야 한다.  
작업 종료 시점에 아래 기준을 충족하지 못하면 완료로 보고하지 않는다.

### 업데이트 트리거

| 상황 | 업데이트 대상 |
|------|-------------|
| 새 모듈/파일 추가 | `agent.md` 디렉토리 맵 |
| 새 IPC 커맨드 추가 | `agent.md` 모듈 책임 + IPC 커맨드 목록 |
| UI 패턴/컨벤션 발견 또는 수정 | `ui-conventions/SKILL.md` |
| Rust trait·에러 처리 패턴 추가 | `rust-skills/SKILL.md` |
| KIS API 동작 특이사항 확인 | `kis-api/SKILL.md` |
| **KIS API 에러코드 신규 발견** | **`kis-api/SKILL.md` Section 5 에러코드 테이블** |
| **KIS API 연동 패턴 구현** (주문·잔고·체결 등) | **`kis-api/SKILL.md` 해당 섹션 또는 신규 섹션** |
| React 성능·번들 최적화 적용 | `react-best-practices/SKILL.md` |

### 반복 프롬프트 패턴 감지 및 스킬 자가 개선

같은 사용자 요청이 **여러 턴에 걸쳐 반복**되는 경우(예: "한국 주식 이름 검색이 안 됨", "버튼 정렬이 안 맞음"), 이는 스킬 문서에 해당 패턴이 누락되었거나 잘못 기술된 신호다.

**감지 즉시 다음 절차를 따른다:**

1. **근본 원인 분석** — 왜 반복되는가? (스킬 미문서화 / 잘못된 패턴 기술 / 엣지 케이스 누락)
2. **스킬 갱신** — 해당 스킬 파일에 올바른 패턴·주의사항·예시 코드를 추가
3. **재발 방지 기술** — `❌ 잘못된 패턴` / `✅ 올바른 패턴` 형식으로 명시적으로 기록
4. **코드 수정과 동시 진행** — 버그 수정과 스킬 업데이트를 같은 작업에서 처리

> 💡 **원칙**: 같은 문제를 두 번 질문받으면 스킬이 불완전한 것이다. 스킬을 고쳐서 세 번째 질문이 없도록 한다.

### 스킬 작성 규칙

- 각 스킬 파일 하단에 `> 마지막 업데이트: YYYY-MM-DDTHH:MM:SS` 기록 (날짜+시분초, 충돌 방지)
- 코드 예시는 이 프로젝트의 실제 파일 경로·컨벤션 기준으로 작성
- `❌ 잘못된 패턴` 섹션은 실제로 발생한 버그·실수에서만 추가 (추측 금지)
- 스킬 간 중복 내용은 가장 관련성 높은 한 곳에만 기술하고 타 스킬에서 참조

---

## 참고 문서

- `agent.md` — 전체 디렉토리 맵 및 모듈 책임 (항상 최신 유지)
- `docs/MasterPlan.md` — 전체 설계 문서 (아카이브, 읽기 전용)
- `docs/discord-setup-guide.md` — Discord 봇 설정 가이드
- [KIS Developers 포털](https://apiportal.koreainvestment.com/)
- [KIS Open API 샘플 코드](https://github.com/koreainvestment/open-trading-api)
