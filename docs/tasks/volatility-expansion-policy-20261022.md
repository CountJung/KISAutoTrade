# 작업 기록: P2-09 변동성 확장의 실제 소유·범위와 청산 비교

## 1. 메타데이터

- 작업 ID: SOXQ-P2-09
- 담당자: Codex와 심볼·helper·Rust·UI·독립 연구·지도 역할
- 상태: Done — 테스트 가능한 구현·일봉 검증 범위. 실제 장중 근거는 미완료로 유지한다.
- 최종 검증일: 2026-10-23
- 작성일: 2026-10-22
- 브랜치: `fix/mac-security-storage-2026-10-06`
- 관련 문서: [TODO](../todo.md), [설계](../research/volatility-expansion-policy-20261022/design.json), [보고서](../research/volatility-expansion-policy-20261022/report.md)
- 위험 등급: 높음(전략·소유 피드백); 실행은 동결 가격·합성 fixture·mock만 사용한다.

## 2. 목표와 비목표

실수량/native 원가·잔량 청산을 확인하고 이전N완료 일봉 범위와 당일 공개 OHLC·관측 quote를 분리한다. 기본10/2/손절3·엄격 > 두 조건·0범위 평균 포함을 보존하고 단독 기간20·배수1.5·수익5/되밀림3·보유63을 고정 비교한다.

- [x] 소유·거절/부분·미체결 기원·복원/재설정·관측 보호를 검증한다.
- [x] 유효 시장일의 새날/같은날·실패/지연·늦은 legacy 자료 경계를 검증한다.
- [x] UI·엄격 저장·두 preview·현재/A-B·진단·모바일을 동기화한다.
- [x] 고정36연속/36분할/432별도분기와 부모·독립 정산을 완료한다.
- [x] 전체 검증·리뷰·문서 게이트를 마쳤다. 이 기록을 포함한 변경을 현재 브랜치에 커밋·푸시하며 정확한 식별자와 원격 일치는 최종 안내·git log에서 확인한다.
- [ ] 실제 장중 최초 발동·체결 가능성은 일봉만으로 평가 불가다. 자료 시각·세션·누락/공급 범위가 확인된 장중 근거 확보까지 기존 P2-09를 열린 상태로 유지한다.

비목표는 운영 설정 자동 적용·파라미터 최적화·실제/모의 주문·자동매매 시작이다.

## 3. 시작 상태

```text
git status --short: clean
HEAD: 47fc10ba744223ec069edc71d77a4d05f425ff19
branch: fix/mac-security-storage-2026-10-06
upstream과 일치, 앞선 P2-08은 커밋·푸시됨
```

- [x] 기존 사용자 변경이 없음을 확인했고 다른 작업자 변경을 보존한다.
- [x] `.env`, `secure_config.json`, `profiles.json`을 읽지 않는다.

## 4. 조사 근거

| 심볼/파일 | 사실 | 영향 |
|---|---|---|
| `breakout.rs::VolatilityExpansionState` | 실제 수량을 bool로 축약·신호 시 보유/원가 변경·설정 수량 Sell | 실제 소유·잔량·보호를 분리한다 |
| `on_daily_close_tick/on_completed_candle` | 당일 완성 OHLC 판단 뒤 범위를 이전 평균에 적재 | P1-03 공개 순서를 유지한다 |
| `required_completed_daily_bars` | 기본 None으로 live coordinator 미참여 | 유효 시장일로 관측 범위를 나눈다 |
| `on_tick` | 범위 > 평균×배수 AND 현재가 > 시가 | UI의 이상 문구를 엄격 초과로 고친다 |
| `StrategyPreviewView/StrategyExperimentSnapshot` | 기존 응답/A-B 연결 경계 | 전용 조건 빈도 DTO를 함께 보존한다 |

Serena navigator가 실제 정의·caller와 manager scope/date/generation·미체결 억제 순서를 확인했다. stock_analyst가 사전 설계를 읽기 전용 감사했다. KIS/Toss endpoint 변경은 N/A이며 실제 API를 호출하지 않는다. 달력·기업행위·공급원 전체 범위·실제 장중 순서는 미확인이다.

## 5. 영향 범위

Rust 전략 분리와 params/context/protection·core optional 진단·저장/driver/엔진v21·fixture를 갱신한다. TS 미러·전용 controls/preparation·기존 preview/current/A-B 전달과 focused spec을 동기화한다.

- [x] Frontend/FSD 공개 경계
- [x] Rust serde와 TS 선택 응답 미러
- [ ] 신규 IPC/등록 N/A — 기존 두 preview·저장 transport 사용
- [x] Web REST 기존 경로와 optional 응답
- [ ] 새 query/cache/event N/A — 기존 요청·무효화 사용
- [x] strict kind/version 보호 metadata와 unknown 기원
- [x] IPC 문서·지도·스킬 동기화

## 6. 금융·자동매매 안전 검토

실제 주문 가능성: 이번 검증 없음.

- [x] 실전·모의 주문과 자동매매 시작은 실행하지 않는다.
- [x] broker/profile/account·owner/market 경계 보존
- [x] 기존 수량/매수가능액/실잔량·동의/guard/risk 보존
- [x] 미체결·반대·동시 제출의 기존 억제 보존
- [x] timeout/자료 실패·지연은 신규 Buy를 보류하고 보유 보호 유지
- [x] pending/order/risk 영속화 실패의 기존 차단 보존
- [x] 비상정지·일일/연속 손실·주문 제한 유지
- [x] 계좌·credential·provider 원문을 문서/trace에 남기지 않는다.
- [x] replay/live·관측 단위·실현/잔여/비용/노출·체결 미검증 한계 표시

새 검증 시장일 첫 quote는 거래소 시가가 아니며 이후 관측 고저도 전체 장중 고저가 아니다. 미관측 자료를 보간하지 않는다.

## 7. 구현 계획

1. 후보 실행 전 설계를 고정하고47fc10b 원문을 프로젝트 `.cache/`에 격리한다.
2. 실제 소유·날짜 자료·엄격 검증·전용 빈도/UI를 구현한다.
3. 생산 비교·부모/독립 감사·전체 회귀·지도/문서 후 커밋·푸시한다.

롤백은 완료 커밋 revert이며 운영 설정은 바꾸지 않는다.

## 8. 도구 선택 기록

- Serena: navigator의 정의/참조·호출 경계 분석 완료.
- Graphify/Serena: helper 감사는 로컬 그래프6316노드/13855간선과 실제 정의로 integer/advance/sync age만 동등함을 확인했다. 평균 H−L은 ATR/종가평균과 다르며 날짜/peak/metadata는 공용화하지 않았다. 해당 helper의 MCP query_graph/initial_instructions 병렬 호출은688.7초 뒤 중단되어 성공 결과/재개 핸들을 받지 못했다. 별도 navigator의 Serena 정의/참조 성공 증거와 실제 소스를 결합했고 부모의 Serena 지침/활성화 호출은 마지막 점검에서 성공했다. 최종 그래프는 CLI로 갱신했다.
- 지도: mapper가 두 지도와 최종 gzip·집계·독립 감사 목록을 갱신하고 체크를 통과했다. StrongClose와 변동성의 개별 dated 구현도 구분했다.
- 리뷰: UI 역할이 Rust의 소유/날짜/보호/driver를 읽기 전용 검토했다. Rust 역할의 UI·문서 검토에서 항상 nextOpen으로 설명하던 문구와 StrongClose 독점 날짜 설명을 발견해 수정했다. 부모는 진단의512봉 상한을 넘는 응답도 준비 완료로 표시할 수 있음을 확인해 미확인 처리·회귀를 보강했다. 마지막 금융 해석 검토는 표80행·기계적 빈도·한계와 열린 장중 조건을 대조해 통과했다.
- 한국어 안내·위임·검토 전송 전 언어 혼입을 점검한다.

## 9. 검증 계획과 결과

| 상태 | 검증 | 결과 |
|---|---|---|
| PASS | 설계·원본 입력 | 고정 SHA, 252이전/752평가·51/2/3주 |
| PASS | 기준 Git 소스 격리 | 주요6파일 원문 SHA 일치 |
| PASS | archived baseline | 기본1224·연속6·분할6·분기72; 원본 연속은 SOXQ완료0/SPY·QQQ각1, 전부기말보유 |
| PASS | Rust fmt/check/Clippy·집중/전체 | fmt --all -- --check, check --offline --all-targets, Clippy --all-targets --all-features -- -D warnings; 집중41·전체 단위775+통합4+doc3=782, 컴파일 경고0 |
| PASS | TypeScript/Vite/FSD/lockfiles | tsc --noEmit·Vite1202모듈·FSD·lockfiles 통과 |
| PASS | focused/전체 Playwright·375px | 전체306/306 이후 체결 안내만 수정한 최종 집중46/46, mock 사용·375px 화면 직접 확인. 최종 수집308개 전체를 다시 실행했다고 주장하지 않는다 |
| PASS |36연속/36분할/432분기·부모/독립 정산 | 최종 생산v21·비대상1122/기존102/owned78 호환, 부모70828·현재390진단·독립249412 검사와468원장 통과 |
| PASS | graph/map/TODO·링크·크기·diff | 최종 CLI그래프/지도·TODO 동기화, TODO 렌더러3/3, 변경 소스1000행 이하(Page998), 링크·diff·해시·언어 혼입 점검 |

실전 주문: NOT RUN. 모의 주문: NOT RUN. 실제 장중 발동/체결과 공급 범위: 평가 불가/미검증.

## 10. 완료 보고

실제 소유·검증된 관측 시장일·범위 진단·선택 보호와 UI를 구현했고 기본 경제/신호 호환을 보존했다. 고정 단독 비교·독립 정산을 완료했으나 기본 승격 근거가 없어 기존10/2/손절3·선택값0을 유지한다. 실제 장중 최초 발동·체결 근거는 기존 P2-09에 미완료로 남겼으며 상위 열린 항목은13개다. 다음 자동 과제는 P2-11이다. 이 기록과 검증된 변경만 현재 브랜치에 커밋·origin 푸시하고 캐시/민감 파일·강제 옵션을 제외한다.

검증 중 재시도: 처음 전체299개는 기존 LTH 클릭 시간초과1건(298통과)이 있었고, 병렬 집중 실행은 공유 dev server 종료로 연결 실패가 발생했다. 원인을 제품 결함으로 확정하지 않았으며 소스 동결 뒤 단독 전체306개를 통과했다. 마지막 sameClose/nextOpen2개 회귀의 정확 일치 locator·비동기 요청 대기 오류는 테스트에서 수정해 최종46개를 통과했다. 제품 코드의 추가 변경은 조건부 체결 안내 문구뿐이다. 서로 다른 실행 결과를 단일 전체308개 통과로 합산하지 않는다.

도구 한계: Graphify 추출기의 기존 설정 JSON4개0노드 안내는 컴파일 경고0과 구분한다. 대체500+252 원장은 미저장으로 별도 독립 재정산하지 않았고 생산 경제/입력 hash 동등성36회에 한정한다. 실제 거래소 시가·전체 장중 범위·최초 발동/체결·공급 coverage·운영 안전성은 이번 fixture/일봉 연구만으로 입증하지 않는다.

운영자 확인 필요:

- [ ] N/A — 배포·자동매매 시작·운영 설정을 변경하지 않는다.
- [ ] N/A — 실전 전 모의·소액 검증은 이번 작업에 포함하지 않는다.
- [ ] N/A — 기존 비상정지/rollback/provider 대조 운영 게이트를 유지한다.
