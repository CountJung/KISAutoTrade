# 에이전트 코드 탐색 도구 운영

이 프로젝트는 프로젝트 구조, 심볼 참조, 중복 헬퍼 후보를 서로 다른
도구와 서브 에이전트로 관리한다.

## 역할 라우팅

| 상황 | Codex 역할 | Copilot 역할 | 주 도구 | 완료 조건 |
|---|---|---|---|---|
| 파일 추가·이동·삭제, 모듈/데몬/데이터 흐름 변경 | `project_mapper` | `project-map-maintainer` | `scripts/project-map.mjs` | `npm run check:project-map` |
| 메서드·타입·컴포넌트 정의와 호출자 추적 | `symbol_navigator` | `symbol-navigator` | Serena MCP | 정의·참조·영향 범위 보고 |
| 두 곳 이상 유사한 helper 발견 또는 공용화 검토 | `helper_curator` | `helper-curator` | Graphify + Serena | 동등성 확인, 공용 위치 이동, 빌드·그래프 재검증 |
| 주식 자동 전략 시뮬레이션·비용/손실 분석·개선 백로그 | `stock_analyst` | — | 실제 Rust replay + Serena | 동일 기간·사전자료·비용 비교, 평가 불가 분리, 근거와 완료 조건이 있는 TODO |
| UI/UX 화면·사용 흐름 검토와 개선 백로그 | `ui_ux_expert` | — | 소스·mock/offline 화면·키보드/DOM 관측 | 근거 등급·영향·완료 기준과 TODO 중복 병합 |

프로젝트 전용 Codex 역할은 `.codex/agents/*.toml`에 있으며, Copilot
호환 역할 설명은 `.github/agents/*.agent.md`에 있다. Codex는 신뢰한
저장소에서 `.codex/config.toml`을 읽으며 동시에 최대 세 개의 서브
에이전트를 실행하도록 설정되어 있다.

`stock_analyst`는 `.codex/agents/stock_analyst.toml`에 정의한 프로젝트 연구
역할이다. 모델은 부모 설정을 상속하며 주문 실행·자동매매 시작·민감 계정 파일
접근을 맡기지 않는다. 분석은 읽기 전용을 기본으로 하고 실행기는 부모가
생산 Rust 라이브러리에 연결해 실행한다. 결과·문서 작성이 필요하면 부모가
연구 출력 경로만 명시적으로 할당한다. 파일의 지시는 역할 범위이며 별도
보안 sandbox 권한을 부여하거나 변경하지 않는다.

연구 기본 흐름은 `prepare_data.py`의 자료·시각 정규화/구간 선언 → 실제
`preview_strategy_from_candles`/전용 LTH 실행 → `analyze.py`의 결과 집계 →
전담 에이전트의 해석 감사 → 기존 `docs/todo.md` 항목 갱신이다.
최근 사례는 [`strategy-review-20261018`](research/strategy-review-20261018/report.md)다.
일봉 LTH·준비 자료 부족·분봉 표본은 정상 성과 순위와 분리하고, 실제 자료와
코드 fixture를 섞지 않는다. 생산 전략/계정 설정을 연구 목적으로 바꾸지 않는다.

설정 형식은 [공식 custom agents 문서](https://learn.chatgpt.com/docs/agent-configuration/subagents)의
프로젝트 `.codex/agents/*.toml` 및 `name`/`description`/`developer_instructions`
계약을 따른다. 새 역할을 로드하지 않은 기존 세션은 같은 지침을 일반
서브 에이전트에 전달해 분석할 수 있으며 역할 파일 저장과 현재 세션의
역할 목록 반영을 구분한다.

## UI/UX 전문가 사용

역할 정의는 [ui_ux_expert.toml](../.codex/agents/ui_ux_expert.toml)이다. 기존 역할과 같은 `name`/`description`/`developer_instructions` 형식을 사용하며 모델을 부모에서 상속한다. 별도 sandbox·인증·네트워크 권한을 추가하지 않는다.

```text
ui_ux_expert에게 현재 HEAD의 Dashboard/Trading/Strategy/Settings/History/Log를
읽기 전용으로 검토하게 해줘. 프로필·실거래 접근 없이 부모가 제공한 격리 mock 증거를 쓰고,
화면 관측/코드 추론/미검증을 구분해 기존 TODO ID와 합쳐 제안해줘.
각 제안에 대상·문제 근거·사용자 영향·우선순위·완료/검증 기준을 적고 실제 UI는 수정하지 마.
```

역할을 아직 로드하지 않은 세션은 일반 서브 에이전트에 같은 TOML 지침과 구체 범위를 전달한다. 2026-10-30 검토는 이 방식으로 수행했다. 파일 저장·현재 세션 위임·새 세션 자동 로딩을 구분하며, 새 세션의 자동 탐색은 이 감사에서 실행하지 않았다.

부모는 화면 프로세스와 산출물을 소유한다. 기존 정적 dist를 loopback에서 서브하거나 [HARNESS](HARNESS_MAP.md)의 envDir=false 격리 구성을 사용하고, 새 브라우저 context에서 페이지 이동 전 mock을 설치한다. 외부 페이지 요청·알 수 없는 API·금융 쓰기를 거부하고 확인한 화면·주입 상태·테마/크기·시각·코드 SHA를 기록한다. 계좌 파일·키·운영 앱/DB·보안 설정을 열거나 바꾸지 않는다. 미확인 상태를 정상 숫자로 보이는 문제와 백엔드 guard 위반을 같은 것으로 판정하지 않는다.

새로운 상시 분석 문서를 늘리지 않고 현재 후보는 [TODO](todo.md), 완료된 검토의 근거·범위는 [완료 이력](completed-history.md)에 놓는다. 보고서·화면 원본은 Git 제외 검증 캐시에 보관한다. 기존 키보드/복구/계좌 표시와 사용자 판단·실자료 작업은 새 근거 없이 다시 미구현으로 만들지 않는다.

## Serena

Serena는 Rust와 TypeScript language server를 사용해 메서드/타입/컴포넌트
심볼과 참조를 추적한다.

- 프로젝트 설정: `.serena/project.yml`
- 공유 온보딩 메모리: `.serena/memories/`
- 로컬 캐시·로그·override: Git 제외
- 우선 도구: `get_symbols_overview` → `find_symbol` →
  `find_referencing_symbols`
- 인덱스 점검: `serena project health-check .`
- 메모리 점검: `serena memories check .`

`.serena/project.yml`은 `.gitignore` 외에 `.env*`,
`secure_config.json`, `profiles.json`을 다시 명시해 민감 파일을
심볼 인덱스에서 제외한다.

## Graphify

Graphify는 로컬 AST로 코드 지식 그래프를 생성한다. 현재 그래프는 Rust,
TypeScript/TSX, JavaScript와 주요 JSON 설정, Cargo 의존성을 포함하며 외부
LLM으로 소스 코드를 전송하지 않는 `--code-only` 방식으로 생성한다.

```bash
graphify extract . --code-only --cargo
graphify cluster-only . --no-label
```

일반 추가·수정 후에는 `npm run graphify:update`, 심볼 삭제·이동을 포함한
리팩터링 후에는 `npm run graphify:refresh`를 사용한다. 두 명령은
`graphify-out/source-hashes.json`도 갱신한다. 완료 게이트
`npm run check:graphify`는 현재 코드 해시가 공유 그래프를 생성한 코드와
일치하는지 검사해 오래된 caller graph 사용을 막는다.

Graphify의 `dedup`은 동일 그래프 엔티티를 합치는 기능이며 복사된 코드나
의미상 같은 helper를 판정하는 clone detector가 아니다. 따라서 공용화는
다음 순서를 지킨다.

1. Graphify로 같은 역할·이름·호출 이웃을 가진 후보와 영향 범위를 좁힌다.
2. Serena로 각 정의와 모든 참조를 확인한다.
3. 입력, 출력, 오류 처리, 직렬화, broker/account/risk 의미가 실제로
   같은지 확인한다.
4. 순수 TypeScript helper는 `src/shared/lib`, 재사용 UI는
   `src/shared/ui`, Rust helper는 기존 view builder 또는 가장 좁은 공용
   backend domain module로 이동한다.
5. 빌드·타입·FSD 검사 후 그래프를 강제 갱신해 삭제된 심볼이 남지 않았는지
   확인한다.

커밋 대상은 `graphify-out/graph.json`, `graph.html`,
`GRAPH_REPORT.md`, `manifest.json`, `source-hashes.json`이다. 이 생성물은
`helper_curator`가 helper 이동 후 갱신하며 `npm run check:graphify`로
freshness를 검증한다. 캐시, 비용 파일, 로컬 절대 경로와 분석 중간 산출물은
`.gitignore`로 제외한다.

## 현재 머신

현재 머신에는 다음 도구와 Codex MCP가 설치되어 있다.

| 항목 | 버전/명령 |
|---|---|
| uv | `0.11.19` |
| Serena | `serena-agent 1.6.1` |
| Graphify | `graphifyy[mcp] 0.9.26` |
| Serena MCP | `serena start-mcp-server --context=codex --project-from-cwd` |
| Graphify MCP | `graphify-mcp <repository>/graphify-out/graph.json` |

설정은 `~/.codex/config.toml`에 등록되어 있다. MCP와 Serena hook을 새로
읽으려면 열린 Codex App/CLI/IDE 세션을 재시작한다.

등록 상태와 로컬 그래프 질의는 다음으로 점검한다.

```bash
codex mcp list
graphify query "shared format helper callers" --budget 700
npm run check:graphify
```

재시작한 Codex 세션에서는 `/mcp` 또는 도구 목록에서 `serena`와
`graphify`가 노출되는지 확인하고, Serena `find_symbol`과 Graphify
`graph_stats`를 각각 한 번 호출해 end-to-end 연결을 확인한다.
