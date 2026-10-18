# 작업 기록: P2-18 연속 replay 상태 경계

## 1. 메타데이터

- 작업: SOXQ-P2-18, 2026-10-18, Codex와 심볼·코어/UI·독립 주식 분석·지도/helper·문서 역할
- 상태: Done — 구현·생산 연구·전체 회귀·독립 감사·문서/지도/그래프 게이트 완료; 게시 커밋은 Git 이력 참조
- 브랜치: fix/mac-security-storage-2026-10-06, 시작cc6f5d0 clean
- 위험: 보통 — 연구 replay·자료 수집·공통 UI. 실제 주문·자동매매 시작 없음

## 2. 목표와 비목표

- [x] 엄격히 이전 warmup을1회 적용하고 실제 전략/회계를 같은 상태로 순차 공급한다.
- [x] 청크를 넘는 포지션·native/원화 원가·pending·ATR·쿨다운·시장 날짜/연속 손실을 보존한다.
- [x] 입력/전체/history 상한·중복/역전/겹침 거부·단일/분할 및 미래 prefix 불변을 검증한다.
- [x] 같은 최초 수량/파라미터/비용의 실제3년 연속과 별도 분기 초기화, 실제 체결 수량/보유구간 시장가격 참조를 비교한다.
- [x] IPC/REST/TS/UI·bounded 일봉 수집·모드/A-B/legacy·준비/취소/오류/한계를 함께 제공한다.
- 비목표: 실전 운용·주문/계좌/DB 접근, LTH 시간대/세션·실제 분봉 coverage, 수익 보장·파라미터 최적화·walk-forward 완료, 장기 서버 세션 저장.

## 3. 시작 상태·조사

사용자 변경 없는cc6f5d0에서 시작했다. `.env`·secure_config.json·profiles.json을 읽지 않았다. 기존 generic의 factory/warmup/session 생성과 loop·finish 경계를 조사했고, 기존P2-14 scripted Signal의 분할 검증만으로 실제 전략 상태를 보장할 수 없음을 확인했다. 기존UI collector는8페이지/500전체/평가248/사전252로3년 수집이 불가능해 별도 bounded 경로를 추가했다.

Serena MCP의 이전475초 미응답에 회복 근거가 없어 새 장시간 요청을 시작하지 않았다. symbol_navigator 역할에 read-only 영향 분석을 위임하고 정확한 정의/참조 소스와 TypeScript 심볼·Graphify CLI 호출 관계로 보완했다. KIS/Toss provider 동작을 추측하거나 변경하지 않았다.

## 4. 구현·계약

- `generic.rs`의 flat500/legacywarmup/기존 정렬 계약은 유지하고 `driver.rs`로 실제 loop를 추출했다. `continuous.rs`는16청크·각500·평가4096/history512·strict 순서를 검증한 뒤 같은 driver를 호출한다. factory/warmup/session/finish는1회이며 chunk ledger는 관측만 한다.
- native 원가/현금·보호·pending·리스크를 경계에서 재생성하지 않는다. market 날짜 reset은 기존 begin_bar 정책대로 계속 수행한다. peak는 이전 반올림 equity의 최댓값을 증분 보존해 O(N²)를 제거했다. 기존1,224전체report/신호와 일치했다.
- 새 `preview_strategy_continuous`를 Tauri/lib에 등록하고 `/api/strategy/preview/continuous`를 동일 순수 함수로 연결했다. 공통 POST 인증/동일출처/body한도를 유지한다. Web/IPC 모두 live 상태·credential·주문 manager를 사용하지 않는다.
- Rust/TS 입력/응답·wrapper/hooks/transport를 동기화했다. 모드는 경제 hash에 포함하며 청크 분할은 제외한다. continuity는 총량/경계시각/수량/pending을 반환하고 A/B는 모드/continuity를 보존한다. legacy는 계산범위 미확인으로 남긴다.
- UI는 일봉 범위·strict이전252봉, 최대48페이지/보관4608·평가4096·500단위청크를 제공한다. 범위 양끝/관측 사이7달력일 초과, 자료 부족/중간오류/상한/scope변경은 계산 보류한다. 긴 휴장도 보류되는 보수적 앱 경계이며 달력/기업행위 미검증을 계속 표시한다. 누락봉은 합성하지 않는다.
- 기존 generation/input identity가 취소·범위/계좌 변경·늦은 page/result 게시를 막는다. 새 전역 수집 cache는 없다. A/B는 기존12scope·각2슬롯·거래100/곡선약201 축약을 유지하고 원본OHLC/history를 저장하지 않는다. 연속70/30을 같은 장부의 앞/뒤 구간으로 표시한다.
- 운영 JSON/DB/credential·신규 event/query key·LTH 전략 확장은 N/A다. LTH generic 거부와 전용 일봉 평가불가는 유지한다.

## 5. 금융 안전·위임/리뷰

실전/모의 주문·자동매매 시작·실계좌/운영DB 접근은 하지 않았다. 기존 주문 scope·예약·pending·영속화/비상정지 계약은 변경하지 않았다(N/A). 연구 pending/장부를 운영 복구 원장으로 사용하지 않는다.

시작/종료 symbol/코드/주식 분석/UI·cache/helper·문서·mapper pass를 위임했다. 종료 감사에서 P1/P2 금융/집계/IPC/UI finding은 없었다. 기존 scope/cancel/calendar helper는 두 collector가 재사용하며 새 공용 승격 대상은 없었다. 변경 소스18개 모두1000행 미만이다. Graphify CLI refresh의 source/canonical·HTML/report 일치를 확인했다.

## 6. 연구 근거·감사

[연구 보고서](../research/continuous-replay-20261018/report.md)와 design/runner·압축 전체 원장·summary/validation을 보관한다. 2023-01-03~2025-12-31 각752평가봉/이전252로3ETF×12전략×비용2의72연속·72alternate partition과864별도분기 계좌를 실행했다. 수량SOXQ51/SPY2/QQQ3와PC기준가/파라미터는 최초2023Q1값을 고정하며 분기마다 바꾸지 않았다. 분기 수익률 합산/복리는 없다.

원본 OHLC와 실제 accepted 수량으로 close→open/open→close 시장손익을 나눠 같은 경로 비용환급과 대사했다. root 독립136,717체크 및 별도 stock 역할의143,866체크가 통과했다. SOXQ 일부 분기 단순보유72개는 비중 한도로 실제0주라 동일노출 비교가 아님을 명시했다. 강한종가/돌파실패의3년 높은 평가수익도 완료청산0회다. 비용 변화로 SPY 연속상승/하락+2.9175→−0.3474%, 평균회귀+4.2519→+0.0835%의 실제 청산 경로 변화가 관측됐다. 기존전략별TODO에 근거만 보강하고 최적값/우수성을 확정하지 않았다.

실제가격은 동결 공개 자료지만 calendar/corporate action coverage·유동성·실제 체결은 미검증이다. 고정FX/무배당/추정비용·상관된3ETF·이미 관측한 과거와 작은 완료 표본 한계를 유지한다.2026을 쓰지 않았다는 사실만으로 unseen OOS라고 주장하지 않는다. P2-16 walk-forward는 미완료로 유지한다.

## 7. 검증 결과

| 상태 | 확인 | 결과/범위 |
|---|---|---|
| PASS | 전체 Rust | 단위517·통합4·doc3, PG환경 제거. 일반실행의 외부DB5개 early-return은 실서버 검증으로 계산하지 않음 |
| PASS | check/Clippy alltargets·allfeatures/fmt | 경고0 |
| PASS | focused continuous13+기존preview37/simulation48 | 실제전략 단일/분할·pending·cooldown·손실risk/시장날짜·FX/ATR·future양모델·상한 |
| PASS | 전체 Playwright114 | 신규10+기존104, mock transport; 실제provider/Tauri E2E 아님 |
| PASS | TypeScript/FSD/lockfiles/build:web | 경고0 |
| PASS | 직전 생산기준 flat1224 | 신규metadata 제거 없이 전체 backtest report·typed 신호 JSON 일치 |
| PASS | 연속72/분할대조72/초기화864 | 연속/분할752평가+history252; 초기화분기60~64평가+history252, 분할 report/signals/preparation·inputHash 동일 |
| PASS | root/독립 주식 분석 산술 |136717/143866체크, 돈 최대약0.01원·비중0.00005%p |
| PASS | 최종 지도/그래프/TODO/링크·문서 | 신규18파일,5397nodes/11782edges/527communities의canonical/HTML/report일치, TODO renderer3·열린22개·사용자대기강조·상대링크133개 |

초기 flat 대조 실패는 연구 스크립트가 typed 신호를 JSON Value로 바꾸며 object 키 순서가 달라진 hash 문제였다. 회계 report는 처음부터 일치했다. 기대 baseline은 바꾸지 않고 typed 신호 JSON 순서로 대조해1224회를 통과했다. 초기FX+ATR fixture의1bps 예산은 실제위험보다 작아0주차단이 정상이었고, 사전10bps로 충분한 위험예산을 지정해 보존 경로를 확인했다. Clippy의 테스트 vec2개는 array로 정리했다. UI57첫실행의56통과/1실패는 옛 '종가only' 도움말 assertion이 원인으로 선택가정/실제체결미검증 계약을 함께 확인하게 수정했으며 focused1과 전체114를 통과했다.

원장JSONL 약30MB는 손실 없이 gzip 약4.2MB로 보관하고 생성 평문은 `.cache/continuous-research-output/`로 옮겼다. 압축 전후 bytes를 비교하고 압축전 SHA를 저장했다. 원본 연구 input/results/validation은 덮어쓰지 않았다. 감사 중 Rust 포맷 이후 runner hash 차이는 최종 source 안정화 후 검증기를 재실행해 갱신했고 독립 감사에서 hash drift0을 확인했다. 마지막 문서 감사가 발견한 초기화 분기의 평가봉 표기60~64/연속752 구분도 바로잡았다. Graphify의 symbol 없는JSON4개 진단은 기존 AST 도구 한계이며 코드 경고와 구분한다.

## 8. 완료·롤백·운영자 확인

배포·실전/모의/소액 검증 승인은 이번 범위에서 N/A다. 운영자 판단/자료 대기는 기존 TODO 주황색 강조로 남기고 자동 구현 부분은 계속 진행한다. 롤백은 작업 커밋을 revert하고 preview/simulation/공통 UI·map/graph 검사를 수행한다. P2-06·P2-03/04·P2-10 등 전략별 사전 가설 비교와 P2-16 walk-forward는 별도 완료 조건이다.
