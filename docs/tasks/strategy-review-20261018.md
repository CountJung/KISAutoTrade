# 작업 기록: 주식 전략 분석 에이전트·전수 시뮬레이션

## 1. 메타데이터

- 작업: 2026-10-18 사용자 요청 — 전문가 에이전트 설정, 현재 자동 전략 시뮬레이션, 개선 TODO 정리
- 담당: Codex / stock_analysis_expert / research_runner / research_audit / research_map
- 상태: Done — 실행·분석·최종 검증 완료
- 브랜치/시작: `fix/mac-security-storage-2026-10-06` / `2cd04db`
- 위험: 보통 — 금융 연구 해석, 생산 주문/리스크 코드 변경 없음
- 근거: [보고서](../research/strategy-review-20261018/report.md), [검증](../research/strategy-review-20261018/validation.json), [TODO](../todo.md)

## 2. 목표·완료 조건

- [x] 재사용할 `stock_analyst` 프로젝트 역할을 설정한다.
- [x] 코드 등록 13종을 확인하고 적합한 데이터·생산 엔진으로 실행한다.
- [x] 일봉 12종 1,224회, LTH 실제 분봉 표본 4회와 generic 거부 102회를 구분한다.
- [x] 비용·기간·종목별 평가, 손익·차단·미청산·benchmark를 독립 분석한다.
- [x] 기존 TODO 13개 전략 및 공통 과제를 갱신하고 연속 운용 P2-18을 추가한다.
- [x] 최종 리뷰·문서/map/graph 게이트 완료. 커밋·푸시 기록은 현재 브랜치 Git 이력 참조.

비목표: 생산 전략/계정 설정 수정, 실전/모의 주문, 자동매매 시작, 파라미터 최적화, 유료 데이터 가입. 기본값을 연구하며 민감한 사용자 저장 파라미터는 읽지 않는다.

## 3. 시작 상태

`git status --short --branch`는 현재 브랜치가 origin 동명 브랜치와 일치하고 변경 파일이 없는 상태였다. 기존 보안/저장 수정과 SOXQ 과거 연구는 보존했다.

- [x] 사용자 변경과 범위를 구분했다.
- [x] `.env`, `secure_config.json`, `profiles.json`을 읽지 않았다.

## 4. 조사·영향

| 파일/경계 | 확인/변경 |
|---|---|
| strategy factory / generic preview | 등록13종, generic12종·LTH 명시 거부, 기본 파라미터 재사용 |
| LTH timed preview / simulation | 실제 분봉 전용 진단과 생산 체결/차단 피드백, 시장 시각 미검증 유지 |
| `.codex/agents/stock_analyst.toml` | 공식 custom-agent 형식 name/description/developer_instructions, 모델 부모 상속 |
| 새 research runner/prepare/analyze | 자료 정규화·창 선언, 실제 Rust 실행, 집계만 분리. 전략을 Python으로 재구현하지 않음 |
| `docs/todo.md` / tooling / Rust skill | 수치·근거·완료 조건과 재사용 분석 역할 기록 |

공식 근거: [OpenAI subagents](https://learn.chatgpt.com/docs/agent-configuration/subagents), [Nasdaq 통상 정규장](https://www.nasdaq.com/market-activity/stock-market-holiday-schedule), 입력 metadata의 Yahoo 공개 원본 URL/수집 시각/hash. 미확인: 기업행동 조정·전 기간 캘린더/분봉 completeness·시장 체결 경로.

## 5. 계약 체크

- [x] Frontend/FSD·Rust serde/TypeScript·Tauri 등록·웹 transport·query/cache/event: N/A — 생산 코드·IPC·UI 변경 없음.
- [x] 영속 schema: N/A — 계정/DB schema 변경 없음, 연구 JSON은 별도 경로.
- [x] IPC 문서: N/A — command 계약 유지.
- [x] 프로젝트 지도: mapper가 새 파일과 실행 흐름을 반영했고 freshness 검사 통과.

## 6. 금융·자동매매 안전

- 실제/모의 주문·자동매매 시작: NOT RUN.
- [x] 기존 broker/account scope, 주문 수량/통화·동의·risk, pending/reconciliation·영속화·재시작/비상정지 경계: 변경 없음.
- [x] 차단 신호를 실제 provider 주문으로 보내지 않는다.
- [x] 소스 원문·hash는 공개 시세이며 계정/credential은 포함하지 않는다.
- [x] same-close·고정FX·무배당·정수 수량·낮은 노출·미청산 손익·상관된 표본 한계를 명시한다.
- [x] LTH 일봉/최근 분봉의 0신호를 성과0%로 집계하지 않는다. P1-05 실제3개월 검증은 잔여로 유지한다.

## 7. 실행·롤백

SOXQ/SPY/QQQ를 결과 확인 전에 지정하고 각 5년 일봉을 공개 다운로드했다. 첫 sandbox DNS 실패 후 필요한 공개 다운로드만 권한 확장으로 재시도했다. 분기252봉 준비 자료+평가 구간, 최근248봉, 두 비용과 같은 수량을 사용한다. 일봉 1,224·benchmark102·LTH거부102·분봉진단4, 총1,432행이다. 원본과 full trade/equity trace는 ignored 프로젝트 `.cache/`, 빌드는 `target/`, 재현 입력/요약은 연구 폴더에 보관한다.

되돌릴 때 연구/역할/문서 커밋을 revert한다. 생산 전략·사용자 설정은 이번 작업에서 바꾸지 않았다.

## 8. 도구·위임

- Serena: 활성화/매뉴얼 확인, 생산 strategy/simulation symbol 경계 확인. 공용 심볼 변경이 없어 별도 변경 영향 위임은 N/A.
- Graphify: 구조 리팩터링/공용 helper 이동 없음, freshness 검사만 수행한다.
- stock_analysis_expert: 새 역할 지침으로 기본값/실험 설계와 재무 해석 독립 감사, 기존 TODO ID 대조.
- research_runner: 새 runner만 소유, cached lock의 생산 의존성 동일성 확인.
- research_audit: 코드·원본/입력 hash·1,432행 compact/full hash·반복 결과 전수 확인, 수정 필요 finding 없음.
- project_mapper: 새 파일 인벤토리와 연구/생산 경계 문서 pass.

현재 세션의 사용자 정의 역할 목록은 시작 시점 목록이므로 새 역할 지침을 일반 서브 에이전트에 전달해 실제 분석했다. 역할 파일 저장과 세션 역할 목록의 즉시 reload를 혼동하지 않는다.

## 9. 검증

| 상태 | 검증 | 결과 |
|---|---|---|
| PASS | workspace check all targets locked | 코드 경고0 |
| PASS | workspace test locked | lib197 + production 통합3 + doc3 =203 |
| PASS | workspace Clippy all targets/features -D warnings | 코드 경고0 |
| PASS | 연구 harness check/run/Clippy all targets/features -D warnings | 경고0, 1,432행 |
| PASS | `npx tsc --noEmit` | 타입 검사 |
| PASS | 반복 실행·동결 회귀 | 전체1,432행 generatedAt 제외 일치; 기존 SOXQ24/거부2/진단/benchmark 일치 |
| PASS | TOML 파싱 | Python3.14 tomllib, 필수필드/모델상속 확인 |
| PASS | fmt·FSD/lockfile·graph/map·diff·문서 감사 | 수정 필요 finding 없음 |
| NOT RUN | Playwright | UI/생산 코드 변경 없음 |

Python3.9 기본 런타임에는 tomllib가 없어 TOML 검증은 설치된 Python3.14로 수행했다. 새 런타임/패키지를 설치하지 않았다. 원본 시세는 재다운로드하면 수정될 수 있으므로 재평가는 보관한 input.json을 기준으로 한다.

## 10. 전달

전담 역할, 전수 결과, 보고서와 기존 TODO의 우선순위/완료 조건을 전달한다. 아직 개선 코드를 구현한 것으로 표시하지 않는다. P1-05의 실제3개월 분봉·시장 시각 검증도 그대로 남는다.

- [ ] 배포 전 자동매매 정지·모의/소액 검증: N/A — 배포·주문 실행 없는 연구 작업.
- [x] 현행 운영 비상정지·rollback·trace 경계 유지.
