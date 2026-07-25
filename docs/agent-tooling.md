# 에이전트 코드 탐색 도구 운영

이 프로젝트는 프로젝트 구조, 심볼 참조, 중복 헬퍼 후보를 서로 다른
도구와 서브 에이전트로 관리한다.

## 역할 라우팅

| 상황 | Codex 역할 | Copilot 역할 | 주 도구 | 완료 조건 |
|---|---|---|---|---|
| 파일 추가·이동·삭제, 모듈/데몬/데이터 흐름 변경 | `project_mapper` | `project-map-maintainer` | `scripts/project-map.mjs` | `npm run check:project-map` |
| 메서드·타입·컴포넌트 정의와 호출자 추적 | `symbol_navigator` | `symbol-navigator` | Serena MCP | 정의·참조·영향 범위 보고 |
| 두 곳 이상 유사한 helper 발견 또는 공용화 검토 | `helper_curator` | `helper-curator` | Graphify + Serena | 동등성 확인, 공용 위치 이동, 빌드·그래프 재검증 |

프로젝트 전용 Codex 역할은 `.codex/agents/*.toml`에 있으며, Copilot
호환 역할 설명은 `.github/agents/*.agent.md`에 있다. Codex는 신뢰한
저장소에서 `.codex/config.toml`을 읽으며 동시에 최대 세 개의 서브
에이전트를 실행하도록 설정되어 있다.

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
