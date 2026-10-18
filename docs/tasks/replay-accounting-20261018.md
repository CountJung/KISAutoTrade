# 작업 기록: P2-15 연구 회계·비용·현금흐름

## 1. 메타데이터

- 작업: SOXQ-P2-15, 2026-10-18, Codex와 심볼·회계·UI·지도·독립 경제/문서 감사
- 상태: Done — 구현·생산 연구·전체 회귀·독립 감사·문서/그래프 게이트 완료; 게시 커밋은 Git 이력 참조
- 브랜치: fix/mac-security-storage-2026-10-06, 시작1192673 clean
- 위험: 보통 — 연구 replay와 공통 UI. 실제 주문·자동매매 시작 없음

## 2. 목표와 비목표

- [x] 실현/잔여 순평가·가격/FX 귀속·배당·비용과 자산 증감을 대사한다.
- [x] 기말 가상 청산의 추가 비용을 실제 거래와 구분한다.
- [x] 방향별/최소 수수료·사용자 수동 출처·엄격히 제한된 FX/배당 입력을 제공한다.
- [x] 같은 요청 수량 단순보유와 실제 체결 경로 고정 비용 환급, 실제 투자비중 차이를 표시한다.
- [x] 일반/LTH 분봉·A/B·legacy 응답에 연결하고 평가 불가 gate를 유지한다.
- 비목표: 실제 브로커 요율 인증, 실제 배당/FX 자료 수집, USD 현금 장부, 실전 주문·운영 설정, 전략 최적화와 장기 연속 운용.

## 3. 시작 상태·조사

1192673의 깨끗한 상태에서 시작한 P2-15 변경을 보존해 이어서 수행했다. 별도 문서 정리694fa14는 이미 origin에 반영되어 있어 다시 이동하지 않았다. 민감 `.env`·secure_config.json·profiles.json은 읽지 않았다. Serena 참조 추적에서 고정 환율 원가 역산과 A/B의 잘린 거래배열을 확인해 native 진입평균 보존과 집계 직접 저장을 선택했다. KIS/Toss provider 동작은 변경하지 않았다(N/A).

## 4. 구현·계약

- 공통 session이 native 평균과 역사적 KRW 원가/매수 fee를 분리한다. 부분매도는 수량 비례로 원가/fee를 배분한다. 가격 변화는 진입 FX, FX 변화는 최종 native 금액에 귀속한다.
- 환율은 지정된 평가봉부터 carry-forward, 배당은 그 봉 직전 보유 수량에 세후 KRW로 바로 지급한다. 미래 FX가 과거 진입/신호에 영향을 주지 않는다. 유휴 현금은 KRW다.
- 방향별 fee는 공통 feeBps를 fallback하고 최소 fee는 주문당 native 단위다. 수동 확인은 사용자 선언이며 자동 인증이 아니다. 배당 조정 가격과 현금배당은 거절한다.
- 시나리오 각각64개·시각64바이트·수치 범위·중복·history 제외 평가봉의 정확한 일치를 검증하고 정렬한 값을 hash에 넣는다. 누락봉/근접 시각으로 옮기지 않는다. invalid 입력은 명시 오류이며 LTH 일봉의 현금흐름 시나리오는 평가 불가다.
- 추가 기말청산 비용은 report-only다. 같은 요청 수량의 단순보유도 실제 체결 수량/노출이 다를 수 있어 함께 보고한다. 실제 경로 비용 환급은 기록된 당시 원화 비용을 보상한 산술 참조이며 무비용 전략 재실행이 아니다.
- 평균/최대 투자비중은 관측봉 종가 평가금액 기준이다. 보유봉 비율·시간 가중/장중 최대와 구분한다. 작은 청산 표본·실현/총자산 역부호·부분/겹침창 경고를 표시한다.
- Rust serde/TS DTO와 두 기존 preview transport를 함께 갱신했다. 신규 IPC/등록·query key·운영 JSON/DB·credential schema는 N/A다. A/B는 전체 집계/가정을 보존하며 legacy 누락을 잘린 trade 배열에서 재구성하지 않는다. invalid 초안은 stale 응답을 무효화하고 계산을 막는다.

## 5. 금융 안전·리뷰·도구

실전/모의 주문·자동매매 시작·실계좌/운영 DB 접근을 수행하지 않았다. 기존 scope·주문 예약/pending·운영 리스크 영속화 계약은 변경하지 않았다(N/A). 연구 원장을 운영 원장으로 사용하지 않는다.

초기/최종 독립 감사에서 최소 수수료가 기대수익 guard에서 빠진 문제를 발견했다. 국내1주10000→10100, 양방향 최소1000원의 일반매도는 순손익−1900원이라 차단해야 한다. 실제 수량·비례 매수 fee·현재 FX 매도 비용의 공통 `project_sale`로 gate와 정산을 일치시켰다. 최소0 기본경로 및 손절·보호·native 손실 예외를 유지했다. 부분매도·해외FX 포함 신규8회귀와 simulation48개가 통과했다.

Serena 영향 분석, 독립 경제/코드 감사, project mapper와 문서 pass를 위임했다. helper 후보의 Graphify/Serena 읽기 요청은475초 미응답 후 중단했다. 새 프로세스를 중복 시작하지 않고 실행 중 shell/graph 갱신 handle이 없음을 확인했다. helper 역할이 Graphify CLI의 호출 관계와 TypeScript compiler AST로35개 정의/참조를 대조해 두 원화 함수의 동등성을 확인했다. `shared/lib/formatters.ts::fmtRoundedKrw`로 승격하고 양 패널은 기존 로컬 이름의 alias로 재사용한다. 음수 half-tie가 다른 fmtBrokerMoney는 유지했다. 실제 export 함수의8경계값·심볼35참조·FSD/타입을 확인했다. 변경 소스는1000행 미만이다.

## 6. 연구 근거

[연구 보고서](../research/replay-accounting-20261018/report.md)와 재현 runner/validation을 보관한다. 직전1192673 생산 라이브러리1224회와 기본 mode의 신규 회계 metadata를 제외한 전체 report/신호가 일치했다. QQQ2026Q3 두 전략×두 비용×두 체결 모델×다섯 가정40개는 실제 일봉 가격에 합성 FX/배당/요율을 적용한 비교다. 실제 배당 포함 총수익이나40개 독립 시장 표본으로 해석하지 않는다. 원본 연구 input/results/validation은 덮어쓰지 않았다.

## 7. 검증

| 상태 | 검증 | 범위 |
|---|---|---|
| PASS | focused simulation48·generic11 | 비용/원가·FX·배당·기말청산·비교·정렬/거절·기존 경로 |
| PASS | focused Playwright47·TypeScript·FSD | 신규회계9+기존체결/ATR/LTH평가/scrollbar38 |
| PASS | 생산 연구 | 기본1224회 전체report/신호 일치, 시나리오40회/항등식120회 |
| PASS | 독립 기록 fill 대사4540회 | 매매현금/fee/tax/slip/부분원가/매봉 equity/가격FX/배당/노출/terminal/환급 |
| PASS | 독립 경제 감사640항목 | Decimal로 결과관계240·원본64봉의benchmark400항목 재계산, 최대금액차0.01원 |
| PASS | 전체 Rust | 단위504·통합4·doc3; PG환경 제거, 외부DB5개 일반early-return을 실서버 검증으로 세지 않음 |
| PASS | check·Clippy allfeatures·fmt·tsc·FSD·lockfiles·build:web | 경고0, formatter 공유 후 타입/빌드도 재확인 |
| PASS | 전체 Playwright104 | mock transport, 실제provider/Tauri/주문 검증 아님 |
| PASS | 공통 formatter 후 focused31 | 신규회계9+기존scrollbar22 재통과, 실제 export8경계값·35심볼참조 확인 |
| PASS | 지도·그래프 | 신규14파일, 최종5310nodes/11570edges/508communities의canonical/HTML전체ID·edge/report일치 |
| PASS | TODO renderer3·문서 링크110·diff | 열린23개, 사용자대기 강조 유지; 최종동기화/공백검사 |

첫 focused UI 실행은44/47이었다. 공통 수수료 locator가 방향별 필드도 선택하던 회귀는 exact locator로 수정했다. 두 결과 초기화 실패는 소스 저장/HMR과 겹쳐 고정 소스에서3개 집중 및47개 전체를 통과했다. 테스트를 약화하거나 production 우회는 추가하지 않았다. 문서 작성용 Python 초안은 encoding 오류로 실행되지 않았으며 별도 문서 pass로 작성했다.

전체 Rust 최초 실행의26실패는 localhost mock bind의 Operation not permitted였다. DB 환경을 제거한 동일 명령을 권한 확장해 재실행하여504+4+3을 통과했다. Graphify의 symbol 없는JSON 설정4개 진단은 기존 AST 도구 한계이며 코드 경고와 구분한다. 최종 독립 문서 감사에서 체크된P2-15를 다음 자동과제로 소개하던 TODO 상단을 찾아 완료/후속P2-18 안내로 수정했다. 수정된 최소fee guard·연구표·source SHA는 추가finding없었다.

## 8. 완료·롤백·운영자 확인

이번 범위에 배포·실전/모의·소액 검증 승인은 N/A다. 운영자의 실제 요율/자료/운용 검증 대기는 기존 TODO의 주황 강조를 유지한다. 롤백은 작업 커밋을 revert하고 simulation/공통 UI와 graph/map 검사를 재실행한다. P2-16·P2-18 및 전략별 복수창 비교는 별도 완료 조건을 유지한다.
