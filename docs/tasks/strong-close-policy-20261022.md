# 작업 기록: P2-08 강한 종가의 실제 소유·완료 일봉·선택 청산

## 1. 메타데이터

- 작업 ID: SOXQ-P2-08
- 담당자: Codex와 위임된 심볼·helper·Rust·UI·연구 감사 역할
- 상태: 완료(구현·오프라인 검증 범위)
- 작성일: 2026-10-22
- 브랜치: `fix/mac-security-storage-2026-10-06`
- 관련 문서: [TODO](../todo.md), [고정 설계](../research/strong-close-policy-20261022/design.json), [연구 보고서](../research/strong-close-policy-20261022/report.md)
- 위험 등급: 높음(전략·체결 피드백). 이번 실행 범위는 동결 가격·합성 fixture·mock 검증이다.

## 2. 목표와 비목표

실제 수량·원가·잔량 청산과 기존 강한 종가/손절식을 보존하고, 이전 완료 일봉 추세·수익 보호·보유 관측 한도를 기본 비활성 선택값으로 제공한다. 기존 live 일봉 갱신 누락은 별도 정확성 교정으로 다룬다. 현재 자료에서 선택값을 자동 승격하거나 실제 주문·운용 검증을 수행하는 작업은 아니다.

사용자 완료 조건:

- [x] 소유·기본 식·완료봉/날짜·거절/부분체결·재설정/복원을 검증한다.
- [x] UI·저장·두 preview·A/B·과거/손상 응답·모바일을 동기화한다.
- [x] 고정 설계의30연속/30분할/360분기와 원가·실현/잔여·기말 비용·노출을 비교하고 독립 감사한다.
- [x] 전체 검사·리뷰·문서 게이트를 완료했다. 이 기록을 포함한 변경을 현재 브랜치에 커밋·푸시하며 최종 식별자와 원격 일치는 완료 안내에서 확인한다.

## 3. 시작 상태

```text
git status --short: clean
HEAD: ac9d7d0e432b2e992aba58fce68187de18777fa5
branch: fix/mac-security-storage-2026-10-06
upstream과 일치, 이전 P2-07 변경은 커밋·푸시됨
```

- [x] 시작 시 사용자 변경이 없음을 확인했고 다른 worker 변경을 보존한다.
- [x] `.env`, `secure_config.json`, `profiles.json`을 읽지 않았다.

## 4. 조사 근거

| 파일/심볼 | 확인한 사실 | 영향 |
|---|---|---|
| `breakout.rs::StrongCloseStrategy` | 기본3/3·(H-C)/H의 inclusive 조건, 손절0은 원가 이하 청산 | 기본 의미와 사용자 안내 보존 |
| `on_live_tick/sync_position` | bool로 수량을 축약하고 실제 체결 전 신호 상태를 복원 | 실제 수량/원가와 제안을 분리 |
| `manager/live_history.rs` | opt-in이 없어 StrongClose가 live 갱신 대상에서 제외 | 검증된 날짜 전달과 새날 조건 갱신 필요 |
| `driver.rs` | D-1 조건→D종가 신호→D+1 관측 시가 | 다음날 시가 매수라는 오해 방지 |
| `Page/getEdit` | 일부 전략 외 raw 값을 숫자로 강제 변환 | 새 정책 필드 타입·손상 gate 보존 |

symbol_navigator는 Serena 정의/참조로 공용 훅·유일한 manager caller와 기존 scope/date/generation/legacy invalidation을 확인했다. stock_analyst의 사전 감사는 5단독 조건과 stop0·연속/분기·시점/실현 해석을 검토했다. provider endpoint·공식 응답 동작 변경은 N/A이며 실제 API 호출은 없다. 공급 범위·조정·달력·기업행위는 미인증 상태다.

## 5. 영향 범위

Rust StrongClose 분리 모듈·params/context/protection/tests, 공용 기본 dated hook·manager 호출, 저장/두 preview와 엔진v20을 갱신한다. UI는 전용 컨트롤·준비 상태·current/A/B·metadata와 focused spec을 추가한다. 연구 산출물·운영 문서·지도·스킬은 부모와 문서 역할이 관리한다.

- [x] Frontend/FSD 경계 및 기존 API 재사용 검증
- [x] Rust/TS 기존 EntryFilterPreparation DTO의 일치 검증(새 필드 추가는 N/A)
- [ ] 신규 IPC/등록은 N/A — 기존 저장·두 preview transport를 사용
- [x] Web REST 기존 경로 동일 검증
- [ ] 새 query/poll/event는 N/A — 기존 준비 상태 위젯 사용
- [x] 새 StrongClose 보호 JSON의 strict kind/version·unknown 기원 검증
- [x] IPC 문서·지도·관련 스킬 갱신

## 6. 금융·자동매매 안전 검토

실제 주문 가능성: 이번 검증 없음. 실전·모의 주문과 자동매매 시작은 NOT RUN이다.

- [x] 이 작업의 실행은 동결 자료·fixture/mock만 사용한다.
- [x] 기존 broker/profile/account·owner/market 경계의 회귀를 검증한다.
- [x] 기존 수량/원가·매수 가능액·실잔량·동의·guard/risk 검증을 유지한다.
- [x] 미체결/거절/반대 방향/동시 제출의 원래 억제를 보존한다.
- [x] timeout/조회 실패·날짜 지연은 신규 Buy를 보류하고 보유 보호는 유지한다.
- [x] pending/order/risk 영속화 실패의 기존 fail-closed 정책을 보존한다.
- [x] 비상정지·일일 손실·주문·연속 손실 제한을 완화하지 않는다.
- [x] 계좌·credential·provider 원문을 문서/trace에 추가하지 않는다.
- [x] replay/live 시점과 표본·원가·잔여·비용 한계를 UI/보고서에 표시한다.

실제 운용 안전·데이터 완전성은 이번 테스트만으로 입증하지 않는다. flat 재시작의 하루 전체 거래 quota는 새로 발명하거나 보장하지 않는다.

## 7. 구현 계획

1. 기준 커밋을 프로젝트 `.cache/`에 격리하고 기본1224회·원본6연속/6분할/72분기를 기록한다.
2. 실제 소유와 일봉 날짜 교정·기본 비활성 선택 조건·strict 저장/두 preview/UI를 구현한다.
3. 생산 비교·부모 및 독립 정산·전체 회귀·리뷰·문서를 완료하고 커밋·푸시한다.

롤백은 완료 커밋 revert이며 운영 설정을 임의 변경하지 않는다.

## 8. 도구 선택 기록

- Serena 사용: StrongClose, core hooks, manager/coordinator, driver/저장, UI caller 정의·참조를 navigator에게 위임했다.
- Graphify: helper 역할은 로컬 그래프와 실제 observation/completed_daily/전략 정의를 대조했다. MCP Graphify 호출은 장기 무응답으로 중단했으며 해당 helper의 Serena 초기화도 복구되지 않았다. 이를 성공한 조회로 기록하지 않는다. 별도 navigator의 Serena 정의/참조 검증과 소스 대조를 사용했다. 기존 bounded_optional_integer/advance_owned_age/sync_owned_age/mean_close를 재사용하며 상태·peak·metadata를 무리하게 공용화하지 않았다. 최종 그래프는 CLI로 재생성해6316노드/13855간선/560커뮤니티와 JSON/HTML/legend 일치를 검증했다.
- 프로젝트 맵: 파일 추가·이동 후 mapper pass를 수행했고 설정 fixture의 검증 범위 과장도 바로잡았다. 최종 압축 기록·독립 감사 생성 후 inventory 재생성과 check:project-map을 통과했다.
- 독립 리뷰: UI 역할이 Rust 소유/dated/driver·저장 경계를, Rust 역할이 UI/current/A-B·운영 문서/스킬을 읽기 전용으로 대조했다. 미체결 상태에서 새 진입 증거를 만들던 P1은 수정·회귀 검증했다. 남은 확정 코드 결함은 없다.
- 언어 검토: 사용자 안내와 위임·검토 결과의 한국어 원칙을 AGENTS.md에 명확히 했고 전송 전 혼입 점검을 적용했다. 이미 출력한 안내는 한국어로 다시 설명했다.

## 9. 검증 계획과 결과

| 상태 | 명령/검증 | 결과 |
|---|---|---|
| PASS | archived ac9d7d0 baseline 실행 | 기본1224·연속6·분할6·분기72 |
| PASS | 원본 입력/구간/수량 검증 | SHA일치, 각252이전/752평가, 51/2/3주 |
| PASS | Rust fmt/check/Clippy·집중·전체 테스트 | fmt --all --check, check --offline --all-targets, Clippy --all-targets --all-features -- -D warnings; 집중34·단위745+통합4+doc3 통과, 컴파일 경고0 |
| PASS | TypeScript/Vite/FSD/lockfiles | tsc --noEmit·Vite1200모듈·FSD·lockfiles 통과 |
| PASS | focused/전체 Playwright·모바일 | 집중24/24·전체262/262, 실제 주문 없는 mock; 375px 오류 입력·컨트롤·안내 화면을 직접 확인 |
| PASS | 현재30연속/30분할/360분기 | 최종 생산 v20; 비대상1122·기존대상102/102·owned78/78 backtest/signals 동일 |
| PASS | 부모·독립 정산 | 부모67393개·독립253595개, 독립 봉45120개·390원장 재정산. 원본3캐시/정규화/설계·baseline 소스 해시 대조 |
| PASS | graph/map/TODO/문서/크기/링크 검사 | graph 최신·JSON/HTML 일치, 최종 mapper·TODO 동기화와 렌더러3/3, 변경 소스1000행 이하(Page996), 로컬 링크·diff 통과 |

원본과 owned 연속6개는 모두 청산0·잔량51/2/3이며 높은 평가수익을 청산 완료 성과로 취급하지 않는다. 선택3조건은 연속6개 모두 수익 하락으로 기본 승격하지 않고 선택값0을 유지한다. 분기 hold63은 마지막 신호10개·실제 한도 청산0이라 손익 동일과 신호 차이를 구분한다. Graphify 추출기의 기존 설정 JSON4개0노드 안내는 남아 있으며 컴파일 경고0과 구분한다. 실전 주문: NOT RUN. 모의 주문: NOT RUN.

## 10. 완료 보고

실제 소유·날짜 자료 교정과 선택 정책/UI를 완료했다. 기본102창·owned78원장 경제/신호 호환과 비대상1122회 호환을 보존했고 단독 세 후보의 자동 기본 승격은 기각했다. 최종 UI/Rust 교차 리뷰와 독립 금융 감사·지도/문서 게이트를 통과했다. 커밋은 이 기록과 함께 현재 브랜치에 저장하고 origin에 강제 옵션 없이 푸시한다. 정확한 완료 커밋은 git log와 최종 안내에서 확인한다.

열린 상위 항목은13개이고 다음 자동 작업은 P2-09다. P1-05/P2-12 실제 자료·실제 provider/운영 검증은 미완료로 보존한다. 대체 분할 원장 자체는 미저장으로 독립 재정산하지 않았으며 생산 경제 hash 동등성 assertion으로 한정한다. 보유 평가수익·마지막 미체결 신호·비중한도0주 기준을 청산 성과/동일 노출 우수성으로 표현하지 않는다.

운영자 확인 필요:

- [ ] N/A — 이번 작업은 배포·자동매매 시작·운영 설정 변경을 수행하지 않는다.
- [ ] N/A — 모의·소액 운용 검증은 이번 완료 주장에 포함하지 않는다.
- [ ] N/A — 실제 운영의 비상정지·rollback·provider 대조 절차는 기존 운영 게이트를 유지한다.
