# 작업 기록: P2-03 모멘텀 소유 피드백과 진입 교차

## 1. 메타데이터

- 작업: SOXQ-P2-03, 2026-10-20 KST
- 상태: Done
- 기준: fix/mac-security-storage-2026-10-06, 86a9829, clean
- 담당: Codex, Rust/UI 담당, symbol_navigator, stock_analyst, project_mapper
- 위험: 보통 — 자동 전략 raw 후보/실제 보유 피드백과 선택 진입 조건
- 관련: [할일](../todo.md), [사전 설계](../research/momentum-feedback-20261020/design.json)

## 2. 목표와 완료 조건

- [x] 실제 소유 수량을 기반으로 무보유 Sell/보유 중 Buy를 제거한다.
- [x] level 소유 교정과 entry-only crossing/hysteresis 후보 효과를 분리한다.
- [x] 보유 negative level 청산은 차단·실패·부분체결 후 계속 평가한다.
- [x] 같은0수량 피드백과 nextOpen pending을 실제 체결로 추정하지 않는다.
- [x] 이전 momentum/무장을 strict warmup으로 seed하고 가상 후보·체결을 생성하지 않는다.
- [x] 기존 custom lookback/threshold 및 같은identity 재설정의 관측/확정 소유를 보존한다.
- [x] UI 입력/저장/두 preview/AB/이전 응답 폐기/모바일을 검증한다.
- [x] 비모멘텀1122회 기본호환·모멘텀102창 변화, 연속24·분할대조24·별도분기288을 검증한다.
- [x] 비용별 raw억제·guard차단·실제체결·손익·MDD·회전·최악손실·실현/잔여·노출과 기각 근거를 보관한다.
- [x] 독립 코드/연구/문서 감사와 지도·그래프·경고0 게이트 후 커밋·푸시한다.

## 3. 시작 상태와 보존

현재 HEAD/origin은86a9829이며 사용자 미커밋 변경은 없다. .env/secure_config.json/profiles.json을 읽지 않는다. 소스/가격/연구 원본을 보존하고 Git archive로 기준 생산 라이브러리를 .cache/momentum-head-baseline에 고정했다. 빌드/일회성 harness는 target/.cache에 두며 버전 관리하지 않는다. 신규 일반 문서는 docs 아래에 둔다.

## 4. 조사와 설계

Serena manual을 읽고 프로젝트를 활성화했다. symbol_navigator가 classic.rs의 MomentumStrategy, factory/trait, manager live pending, replay driver, exact owner 피드백을 정의·참조로 추적했다. 기존 negative 분기는 실제 보유 검사가 없고 sync_position은 매번 last_sell_price=None으로 돌린다. live도 후보 뒤 두 가격 anchor를 복원해 무보유 매도 억제가 이어지지 않는다. 기존 재매수/매도 가격 anchor는 임계값 crossing이 아니며 실제 매도 후 재진입 anchor 계약도 없다.

stock_analyst가 원본 inclusive 경계를 검증했다. 초기 strict 경계 가정은 후보 실행 전에 폐기하고 기존 매수>=5/매도<=−5를 유지했다. 보유 교정 level은 무보유 매도/보유 중 매수 제거 효과만 분리하며, 실제 체결이 달라지면 별도로 설명하고 구수치를 강제로 맞추지 않는다.

새 entry_mode는 level(기본)/crossing, entry_hysteresis_pct는0(기본)..100 유한수 %p다. crossing은 이전m<5→현재m>=5, 무장은m<=5−간격이며 활성 후보는m<5에서 취소한다. 간격1일 때4<m<5는 후보만 취소하고 다시 무장하지 않는다. level에서는 간격을 보관하되 사용하지 않는다고 명시한다. 전모드 간격값 형/범위는 검증하고 crossing은 threshold이하 관계를 추가 검증한다. 잘못된 저장 optional 설정은 신규 매수만 보류하며 legacy 기간/보유 청산을 보존한다.

관측 상태/후보와 실제 owned_quantity를 분리한다. signal이나 같은0수량 게시로 소유를 만들지 않는다. 실제 양수 수량 피드백으로 진입 후보를 소모하며 잔여 양수 수량의 negative level 청산은 반복 평가 가능하다. 주문 충돌/pending/guard/risk를 우회하거나 완화하지 않는다.

## 5. 영향과 금융 안전

- Rust: Momentum을 classic/momentum.rs로 분리, 선택 params의 default/strict 검증, 기존 save/short/continuous 경계, v14.
- UI: page-local mode/간격 controls, legacy raw타입·소수draft, AB params표시, 기존오류/이전응답무효화.
- 새 IPC/REST/TS DTO: N/A — 기존 params 계약과 저장/preview 경로를 사용한다.
- DB schema/provider인증/계좌scope·pending원장: N/A — 변경하지 않는다.
- 실전·모의 주문/자동매매 시작: 실행하지 않는다.
- 실제 계좌 총량을 전략 소유량으로 공유하지 않는다. exact owner 피드백·지속 저장 실패 차단·공통 guard/risk를 유지한다.

## 6. 검증 계획

cargo fmt/check, 전체 Rust test, Clippy all-features -Dwarnings; tsc/FSD/lockfiles/web build; focused/전체 Playwright; renderer/map/graph freshness·상대 링크. source파일1000행미만·최근AGENTS5건·민감/캐시제외를 확인한다. rollback은 변경 커밋 revert이며 실제운용 검증/배포와 구분한다.

연구는 기준 생산 lib의 단기1224/연속6/분할6/분기72와 현재 생산 lib의 비모멘텀1122 exact·모멘텀102변화/연속18/분할18/분기216을 구분한다. 고정 자료·수량·비용과 선언은 design에 보관한다. 분기계좌를 합산/복리하지 않고 상관ETF3개의사후진단을 OOS/우수성으로 해석하지 않는다. 최종 결과는 아래 검증 표에 기록했다.


## 7. 최종 검증 결과와 독립 감사

| 검증 | 결과 |
|---|---|
| fmt/check | cargo fmt --all -- --check; cargo check workspace/all-targets/locked, 경고0 |
| 전체 Rust | lib553 + production integration4 + doc3 =560, 실패0 |
| focused Rust | momentum20/classic21/strategy150, 실제 durable save·두 preview strict·미래 prefix/두체결모델/분할·partial/pending 포함 |
| Clippy | workspace/all-targets/all-features/locked -Dwarnings 통과 |
| TypeScript/FSD/lockfiles | 모두 통과 |
| Chromium | 공유 lexical parser 승격 뒤 전체138/138. 승격 전 신규/인접60/60과 별도로 기록 |
| web build | 경고0, main335.60kB/charts175.25kB; 기존 chunk 경계·limit 유지 |
| 생산 기준 대조 | archived flat1224, 현재 비모멘텀1122 report/signals exact 및 Momentum102창 경제·회계 동일 |
| 고정 후보 | archived6/current18 연속 및 대체분할각24, archived72/current216 별도분기288 |
| parent 기록 대사 | 312개 계좌46,772개 정산 통과 |
| 독립 stock 감사 | 원장312개135,530 + flat102개1,430 =136,960검사 오류0, 최대 약0.01원/0.00005003%p 반올림 차이 |
| 코드·문서 감사 | core 인과/실제소유·예외/P1/P2잔여0; Rust 필드표와 설정변경 재seed 설명 drift2건 수정 |

모멘텀 기본 level은78개 회계/자산곡선/실제filledtrade가 legacy와 정확히 같다. raw와 차단 수를 줄였으며 수수료 절감이나 손익 개선으로 표현하지 않는다. 독립 partition24개 검사는 원장/inputHash 대응 확인이며 대체 payload를 독립 재실행한 것으로 집계하지 않는다. 별도 분기 수익률은 합성하지 않았다. 실제 DB환경 없는 전체 test를 DB서버 검증으로 세지 않고 실전·모의 주문/자동매매 시작은 실행하지 않았다.

## 8. Helper·구조·문서 게이트

helper_curator에게 두 controls parsedDraft를 읽기 전용 감사로 위임했다. MCP query_graph/get_symbols batch는558초 결과 없이 대기했고, 부모가 Graphify CLI로 기존 parsedDraft/ConsecutiveEntryControls의16개 후보·caller 범위를 확인한 뒤 Serena로 두 정의와 모든4caller를 추적했다. 도메인 범위/정수/mode관계는 다르며 lexical regex/Number/finite/raw보존만 동등해 src/shared/lib/numberDraft.ts의 공개 parseDecimalDraft로 승격했다. 각 raw-state/useEffect·도메인 검증·A/B 요약은 유지했다. helper_curator도 최신 파일을 확인해 동등성과 좁은 승격을 승인했으며 추가 공용화는 없었다. 전체 브라우저138개로 승격 후 두 전략 입력을 검증했다.

project_mapper가 신규21개 인벤토리와 모듈 책임/흐름을 별도 갱신했다. Graphify CLI refresh 후 canonical/HTML 노드5566·간선12158·커뮤니티527개 대응, freshness와 TODO renderer3/3 통과를 확인했다. 상대링크·민감/캐시 제외·최근AGENTS5건·변경 source1000행미만도 최종 확인했다. 구조 결과와 전달 hash는 Git 이력으로 확인한다. 루트 문서 배치는 README/AGENTS/CLAUDE를 유지한다.

## 9. 잔여 한계·롤백·전달

기본값은 보유 교정 level/간격0이다. 선택 crossing/1%p는 연속장부에서 추가효과가 없고 분기 손익이 일부 악화해 기본 채택하지 않았다. QQQ2024Q3 개선은 청산1→0/낮은 노출/잔여 평가를 동반한다. 기존 RSI/Deviation 실행 코드는 바꾸지 않았으며 다음 자동 항목은 P2-04/P2-10이다. 실제 coverage/체결/운용검증·사용자 판단 항목은 미완료 강조를 유지한다.

변경 rollback은 해당 커밋 revert이며 배포·실운용 전환은 별도 절차다. 기존 사용자 변경과 민감 원본은 읽거나 수정하지 않았다. 최종 검증/리뷰·문서 게이트 후 캐시를 제외해 현재 브랜치에 커밋·origin 푸시하고 강제 푸시하지 않는다.
