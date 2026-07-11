# KISAutoTrade — Codex 프로젝트 지침

개인용 자동 주식 매매 시스템. **Rust (Tauri v2) + React 18 + TypeScript** 풀스택.  
작업 전 반드시 `AGENTS.md`를 읽어 현재 구조를 파악한다.

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

## 기능 구현 UI 동기화 원칙

> **백그라운드 로직만 구현하고 끝내지 않는다.** 새 기능·상태·설정 값을 Rust 백엔드에 추가했다면, **반드시 사용자가 UI에서 확인하거나 조작할 수 있는지 검증**한다.

### 체크리스트 (기능 구현 완료 조건)

| 구현 항목 | UI 확인 항목 |
|----------|------------|
| 새 AppState 필드 추가 | Settings 또는 Dashboard에서 읽기/쓰기 가능한가? |
| 새 IPC 커맨드 추가 | 해당 커맨드를 호출하는 React 컴포넌트·버튼·훅이 있는가? |
| 새 설정 값 추가 | Settings 페이지에서 수정 가능한가? 저장 후 재기동 없이 즉시 반영되는가? |
| 새 상태 플래그 추가 | Dashboard 또는 관련 페이지에서 시각적으로 표시되는가? |
| 에러/경고 상태 추가 | 사용자에게 알림(토스트·배지·색상)이 표시되는가? |
| 백그라운드 작업 결과 | 완료·실패 여부를 사용자가 어디선가 확인 가능한가? |

### UI 도구 검증 게이트

다음 조건 중 하나라도 해당하는 UI 작업은 타입 체크만으로 완료하지 않는다. 가능하면 Playwright 또는 동등한 브라우저 자동화 도구로 실제 화면을 열어 검증한다.

- 새 화면/패널/다이얼로그/차트/주문·전략 제어 UI를 추가하거나 주요 배치를 바꾼 경우
- 스크롤, 리사이저, 드래그, Select/Menu, tooltip, chart marker처럼 포인터 상호작용이 중요한 경우
- 모바일/좁은 폭에서 버튼·입력·표·차트가 겹치거나 잘릴 위험이 있는 경우
- 이전에 사용자가 “보이지 않음”, “스크롤 안 됨”, “클릭 안 됨”, “엉뚱한 영역 표시” 같은 UI 회귀를 보고한 영역을 다시 수정한 경우

검증 기본 흐름:

1. dev server가 필요하면 실행하고 URL을 확보한다.
2. Playwright로 대상 페이지에 진입해 핵심 컨트롤의 존재, 선택/클릭/스크롤/드래그 결과, 에러·빈 상태를 확인한다.
3. 차트/캔버스는 nonblank 여부와 마커·축·컨테이너 크기를 확인한다.
4. 시크릿·실계좌 호출이 필요한 기능은 네트워크를 mock/stub하거나 read-only 상태까지만 검증하고, 실행하지 못한 부분을 최종 보고에 명시한다.
5. 도구·환경 문제로 Playwright를 실행하지 못하면 `npx tsc --noEmit` 등 가능한 검증을 수행하고, 미실행 사유와 다음 턴용 검증 명령을 남긴다.

### 적용 예시

| 상황 | ❌ 불완전 구현 | ✅ 완전 구현 |
|------|-------------|------------|
| `buy_suspended` 플래그 추가 | Rust에만 존재, UI 반응 없음 | Dashboard에 "매수 정지" 배지 + 해제 버튼 표시 |
| 체결 기록 보관 기간 설정 추가 | `TradeArchiveConfig` 구조체만 추가 | Settings에서 retention_days 입력 + 즉시 정리 실행 표시 |
| 로그 정리 데몬 추가 | 백그라운드에서만 동작 | Settings → 로그 탭에서 정리 결과·보관 현황 확인 가능 |

### 예외

순수 인프라(토큰 갱신, 파일 정리, WebSocket 재연결 등)는 사용자에게 노출할 필요 없는 내부 동작이므로 UI 동기화 불필요.  
다만 **실패 시** 사용자에게 알림이 가야 한다면 Discord 알림 또는 토스트 추가를 검토한다.

---

## 버그 발견 시 개선 원칙

> **버그를 문서화하고 끝내지 않는다.** 버그라고 판정되면 **항상 개선 가능성을 검토하고 구현 방향을 탐색**한 뒤, 개선할 수 있으면 즉시 구현한다.

### 절차

1. **버그 원인 분석** — 왜 발생하는가? (설계 결함 / 미구현 기능 / 엣지 케이스 누락)
2. **개선 가능성 탐색** — 코드 구조 변경, 외부 API/라이브러리 활용, 폴백 전략 등 검토
3. **구현 우선** — 단순히 "알려진 버그" 문서화로 끝내지 말고 개선 코드를 작성한다
4. **문서 갱신** — 개선 내용을 user-guide.md/SKILL.md에 반영한다 (버그가 아닌 "기능 지원"으로 업데이트)

### 예시

| 버그 상황 | ❌ 소극적 대응 | ✅ 적극적 대응 |
|----------|-------------|-------------|
| KRX WAF 차단으로 종목 검색 실패 | "KRX가 차단 중이어서 검색 안 됨" 문서화 | k-skill-proxy 폴백 추가 → 검색 항상 동작 |
| 전략 다중 종목 등록 시 신호 충돌 | "버그: 1종목만 등록하세요" 경고 추가 | HashMap per-symbol 리팩토링 → 다중 종목 완전 지원 |
| 잔고 부족 시 매수 루프 반복 | "잔고 부족 오류 로그 확인하세요" 안내 | `buy_suspended` 플래그 + 자동 복구 로직 구현 |

### 예외

실제로 외부 의존성 제약(KIS API 정책, 거래소 규정 등)으로 코드 개선이 불가능한 경우만 "알려진 제한"으로 문서화한다.

---

## 자동매매 손실 방지 원칙

> 반복 등락장에서 잦은 매수/매도 루틴이 관찰되면, 단일 전략 파라미터 조정으로 끝내지 말고 공통 주문 방어 계층을 먼저 검토한다.

### 우선순위

1. `Strategy::on_tick()`에서 나온 신호를 바로 주문하지 말고 `OrderManager` 앞단의 공통 guard에서 쿨다운, 최소 기대수익, 손절 후 재진입 금지를 검사한다.
2. 전략 내부 `in_position`은 실제 보유 수량과 다를 수 있으므로, 자동매매 시작/재시작 시 KIS 잔고와 로컬 포지션을 기준으로 복원한다.
3. 횡보장 필터(ATR, 밴드폭, 최근 반대 신호 횟수)를 전략별 옵션이 아닌 공통 리스크/guard 후보로 우선 설계한다.
4. 체결은 가격 틱 기반 가정이 아니라 주문번호 기반 체결 조회로 확인한다. 로컬 자동 체결은 모의/개발 편의 기능으로 격리한다.
5. 반복 매매 방지 변경은 `todo.md`, `rust-skills/SKILL.md`, 필요 시 `kis-api/SKILL.md`에 함께 기록한다.

---

## 코드 리뷰·위임 트리거

> 장기 운영 앱이므로 대형 파일, 중복 헬퍼, polling/cache/log reader 변경은 기능 완료와 별개로 적극 정리한다.

- 소스 파일이 1000라인을 초과하면 신규 기능을 더 얹기 전에 페이지/route/helper/domain 단위로 분리한다. 즉시 분리가 어렵다면 `todo.md`에 구체 파일과 분리 축을 기록한다.
- 숫자/금액/decimal 표시, provider trace, broker scope, 프로파일 view처럼 두 곳 이상 반복되는 helper는 `shared/lib`, `shared/ui`, backend view builder 등 공용 위치로 승격한다.
- 로그 조회, 이벤트 listener, background daemon, TanStack Query polling, cache, per-symbol map을 추가하거나 수정할 때는 OOM·jank·quota 낭비 가능성을 점검한다.
- **위임 게이트는 작업 시작 전과 최종 보고 직전에 두 번 실행한다.** 모든 구현/리뷰 턴에서 먼저 "위임 판단 체크"를 수행한다. 아래 조건 중 하나라도 맞으면 subagent 도구가 노출된 런타임에서는 서브 에이전트 위임을 시도한다. 이 저장소에서 코드 리뷰/문서 drift 점검은 사용자가 기본 허용한 read-only 감사로 본다. 이미 구현을 끝낸 뒤 조건을 발견했어도 최종 보고 전에 review/documentation pass를 별도 위임하거나 직접 수행해야 한다.
- 다음 조건에서 서브 에이전트 위임을 시도한다.
  - 코드 리뷰 감사: 코드 리뷰 전용 요청이 있거나, 1000라인 초과 파일/중복 helper/OOM·jank 위험/polling·cache·listener·daemon 변경/문서·스킬 drift 가능성이 있으면 별도 explorer/worker에 read-only 감사를 맡긴다. 사용자가 read-only를 명시하면 메인 에이전트도 구현/문서 수정을 하지 않고 감사 결과만 보고한다.
  - Rust backend: `commands.rs`, `server/mod.rs`, `lib.rs`, broker adapter, trading daemon/risk/order 로직이 2개 파일 이상에 걸칠 때
  - React/FSD: 1000라인 초과 페이지 분리, `pages`→`widgets/features/shared` 이동, shared hook/component 추출이 필요할 때
  - Performance: polling, log reader, event listener, cache, unbounded Vec/HashMap, long-running task를 새로 만들거나 변경할 때
  - Broker/API: KIS/Toss 인증, account scope, rate limit, request cadence, official response mapping을 바꿀 때
- 문서 업데이트 조건을 충족하는 작업(아래 Living Documentation 업데이트 트리거 참조)은 별도 documentation pass 대상으로 취급한다. subagent 도구와 정책이 허용되면 explorer에게 "변경 파일과 관련 문서/스킬 drift만 감사"하도록 위임한다. 도구가 없거나 즉시 대기할 수 없으면 메인 에이전트가 같은 pass를 수행한다.
- subagent 도구가 없거나 현재 정책상 위임할 수 없으면 최종 보고에 `위임 대체 수행` 항목을 포함한다. 거기에 (1) 어떤 위임 조건이 충족됐는지, (2) 직접 수행한 체크리스트, (3) 다음 턴에 바로 실행할 수 있는 서브 에이전트 프롬프트를 남긴다.
- read-only 요청, 도구 제한, 사용자 중단 등으로 리뷰나 문서 갱신을 완료하지 못하면 최종 응답에 `Next-turn handoff:`를 포함하고 남은 작업·관련 파일·필요 검증 명령·문서 갱신 후보를 구체적으로 적는다.

### 위임 프롬프트 템플릿

코드 리뷰 pass:

```text
Use the repository at <repo>. Review the current diff only. Focus on regressions, missing tests, 1000-line files, duplicated helpers, OOM/jank/polling/cache risks, broker/account scope drift, and documentation/skill drift. Do not edit files. Return findings with file/line references and severity.
```

문서 업데이트 pass:

```text
Use the repository at <repo>. Compare the current diff against AGENTS.md, docs/ipc-commands.md, docs/project-map.md, docs/toss-openapi.md, and relevant .github/skills/** files. Do not edit files. Return missing or stale documentation updates, with exact target files and suggested wording.
```

---

## 핵심 규칙

### Rust

- `cargo check` 성공 없이 코드 완료로 보고하지 않는다
- `cargo check` 및 빌드에서 발생하는 **경고(warning)도 반드시 해소**한다. `#[allow(...)]` 어트리뷰트는 최후 수단으로만 사용하며, 사용 시 이유를 주석으로 명시한다
- IPC 커맨드 반환 타입: `CmdResult<T>` = `Result<T, CmdError { code, message }>`
- 공유 상태: `Arc<RwLock<T>>` (async read-heavy), `Arc<Mutex<T>>` (write)
- JSON 직렬화는 `#[serde(rename_all = "camelCase")]` ← TypeScript 인터페이스와 1:1 매핑
- **axum 웹 핸들러에서 내부 struct(StrategyConfig 등)를 `serde_json::to_value()`로 직접 직렬화 금지** — snake_case가 그대로 노출됨. `serde_json::json!{}` 매크로로 camelCase 키를 명시하거나, `#[serde(rename_all = "camelCase")]` 달린 별도 View struct를 사용. Tauri IPC `commands.rs`의 동일 커맨드 응답 필드와 키 이름이 반드시 일치해야 함 (→ `rust-skills/SKILL.md` 참조)
- 에러 처리: `thiserror` + `?` 연산자. `unwrap()` 최소화
- 새 IPC 커맨드 추가 시 반드시 `lib.rs`의 `generate_handler!`에 등록

### TypeScript / React

- TypeScript 컴파일 경고(unused variable, implicit any 등)도 **반드시 해소**한다. `// @ts-ignore`, `as any` 캐스트는 최후 수단으로만 사용하며, 사용 시 이유를 주석으로 명시한다
- MUI 아이콘: **직접 경로** import 필수 (`@mui/icons-material/PlayArrow`)
- UI 색상은 MUI theme palette를 source of truth로 삼는다. 컴포넌트는 `background.default`, `background.paper`, `text.primary`, `text.secondary`, `divider`, `primary.main`, `success.main`, `warning.main`, `error.main` 같은 semantic token을 우선 사용하고, 다크 모드에서 순수 검정/임의 hex로 스크롤바·배경·텍스트를 직접 지정하지 않는다. 스크롤바 색상은 전역 `MuiCssBaseline`에서 palette 기반으로 관리한다.
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

- 기본은 JSON 파일이며 Settings에서 PostgreSQL 또는 MariaDB document backend로 명시 전환할 수 있다.
- store는 `storage::read_json_or_default()` / `write_json()` 경계를 사용하고 DB 오류를 JSON fallback으로 숨기지 않는다.
- DB 관리·자격증명 IPC는 Tauri 데스크톱 전용으로 유지하며 인증되지 않은 LAN REST에 노출하지 않는다. 임의 SQL도 받지 않는다.
- JSON↔DB 전환은 자동매매 정지, schema 검증, transaction import 또는 JSON 복구를 먼저 완료한다.
- `profiles.json`, `secure_config.json`, `.env`, 로그와 DB password는 DB document/import/export 대상에서 제외한다.
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
| 프론트엔드 FSD 구조화, 레이어 경계, 모듈 이동 | `.github/skills/frontend-fsd/SKILL.md` |
| Rust 데이터 구조, trait, 에러 처리, serde | `.github/skills/rust-skills/SKILL.md` |
| KIS API 인증, REST/WS 호출, tr_id, 에러코드 | `.github/skills/kis-api/SKILL.md` |
| 토스증권 OpenAPI, OAuth2, 계좌 헤더, rate limit, REST adapter | `.github/skills/toss-api/SKILL.md` |
| MUI v6 컴포넌트, 차트, 색상, 금융 UI 컨벤션 | `.github/skills/ui-conventions/SKILL.md` |

Codex 환경에서는 프로젝트 루트의 `.codex/skills/kisautotrade-*` 브리지 스킬이 현재 작업 저장소 루트 기준으로 위 원본 스킬을 다시 읽도록 구성되어 있다. Codex 런타임이 계정 스킬만 읽는 경우 `scripts/sync-codex-skills.ps1`로 `~/.codex/skills`에 동기화한다. 브리지 스킬은 자동 트리거용 얇은 연결 파일이므로, 규칙 변경 시 반드시 저장소의 `.github/skills/**/SKILL.md`를 수정한다.

---

## 살아있는 문서 원칙 (Living Documentation)

`AGENTS.md`, `todo.md`, `docs/project-map.md`, `docs/ipc-commands.md`, `codex-instructions.md`, `.github/skills/**` 는 **항상 현재 코드베이스를 정확히 반영하는 살아있는 문서**여야 한다.  
작업 종료 시점에 아래 기준을 충족하지 못하면 완료로 보고하지 않는다.

문서 업데이트는 구현 완료 조건이다. documentation-update subagent가 있으면 drift 감사를 위임할 수 있지만, 도구가 없으면 메인 에이전트가 직접 갱신한다. read-only 턴에서는 파일을 수정하지 않고 갱신 대상과 권장 문구를 보고한다.

### 업데이트 트리거

| 상황 | 업데이트 대상 |
|------|-------------|
| 새 모듈/파일 추가 | `docs/project-map.md` 디렉토리 맵 |
| 새 IPC 커맨드 추가 | `docs/ipc-commands.md` IPC 커맨드 목록 |
| UI 패턴/컨벤션 발견 또는 수정 | `ui-conventions/SKILL.md` |
| 프론트엔드 모듈 이동 또는 FSD 레이어 경계 변경 | `frontend-fsd/SKILL.md`, `docs/project-map.md` |
| Rust trait·에러 처리 패턴 추가 | `rust-skills/SKILL.md` |
| KIS API 동작 특이사항 확인 | `kis-api/SKILL.md` |
| Codex/Copilot 지침 연결 방식 변경 | `AGENTS.md`, `.codex/skills/**`, `.github/copilot-instructions.md`, `.github/codex-instructions.md` |
| **KIS API 에러코드 신규 발견** | **`kis-api/SKILL.md` Section 5 에러코드 테이블** |
| **토스증권 API 동작·에러·rate limit 신규 확인** | **`toss-api/SKILL.md`, `docs/toss-openapi.md`** |
| **KIS API 연동 패턴 구현** (주문·잔고·체결 등) | **`kis-api/SKILL.md` 해당 섹션 또는 신규 섹션** |
| React 성능·번들 최적화 적용 | `react-best-practices/SKILL.md` |

> **`AGENTS.md` 변경 이력**: 작업마다 한 줄 요약을 추가하되 **최근 5건만 유지**한다. 오래된 항목은 삭제하고 세부 이력은 git commit 메시지에 위임한다.

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

- `AGENTS.md` — Codex 에이전트 가이드 (핵심 경로·빌드·변경 이력)
- `todo.md` — 개선 백로그 및 다음 작업 목록
- `docs/project-map.md` — 전체 디렉토리 맵 및 모듈 책임 (항상 최신 유지)
- `docs/ipc-commands.md` — IPC 커맨드 전체 목록 (항상 최신 유지)
- `docs/coding-guide.md` — 설정 추가·AppState·IPC·데몬·제어흐름 실전 가이드
- `docs/MasterPlan.md` — 전체 설계 문서 (아카이브, 읽기 전용)
- `docs/discord-setup-guide.md` — Discord 봇 설정 가이드
- [KIS Developers 포털](https://apiportal.koreainvestment.com/)
- [KIS Open API 샘플 코드](https://github.com/koreainvestment/open-trading-api)
