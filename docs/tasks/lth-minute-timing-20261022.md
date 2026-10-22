# LTH 분봉의 거래소 시각·세션·미관측 구간 진단 — 2026-10-22

상태: Done. 시작 ec57c02, 브랜치 fix/mac-security-storage-2026-10-06, 시작 시 작업 트리 clean. 프로젝트 Rust/UI/Toss API 브리지와 원본 규칙을 적용한다.

## 목표와 완료 조건

- 열린 TODO의 LTH 분봉 거래소 시각·DST·봉 시작/종가 공개 모델·정규장/마감·누락 진단을 합성 fixture와 mock UI로 구현한다. 실제 3개월 자료와 P1-05/P2-12 검증은 별도 미완료로 유지한다.
- 기존 Toss 분봉은 봉 종료 표기다. barClose의 시작은 label−60초/종가 공개 가능 시각은 label, 명시적 barStart fixture는 label/+60초로 구분한다. 실제 수신·발행 시각 검증으로 표현하지 않는다.
- RFC3339 offset은 실제 순간으로 NY/Seoul 변환하고 compact12/14는 legacy KST 가정이라고 명시한다. naive local ISO·무효 날짜·초/소수초 정렬·중복 순간·역순/동일 순간 다른 offset을 진단한다.
- 기존 LTH의 fixed KST 세션·live 시계·시그널·체결/정산·입력hash·profile/account 검사를 바꾸지 않는다. 새 진단은 별도 버전이며 time parsing 성공으로 sampleOnly/notEvaluable/lookAheadSafe:false를 승격하지 않는다.
- 일별 일반 평일 정규 시간창과 실제 달력 검증을 구분한다. 휴장/조기마감/거래정지/무거래봉 생략 원인이나 전체 요청 범위 coverage를 추측하지 않는다. 야간·주말을 연속 분봉 누락으로 합산하지 않는다.
- 실제 replay에 준비된 분봉 label을 진단하며 raw provider 수/선택 수/가격 제외/분봉 표본 상한을 구분한다. 어떤 관측도 진단 목적으로 실제 replay에 추가·제거·정렬/재해석하지 않는다.
- max512 입력·64byte label·날짜/예시 목록 상한, 산술 gap 계산으로 장기간 minute 배열을 만들지 않는다. 현재 시각 검사가 있다면 명시 asOf를 응답에 기록해 fixture에서 고정한다.
- assessment 선택 객체와 TS/UI 현재/A-B·legacy 정보없음·일봉 평가불가/빈자료·모바일을 함께 검증한다. 새 IPC/주문·자동매매·provider 설정 변경은 없다.
- Rust focused/전체·fmt/check/Clippy 경고0, tsc/FSD/build·focused/전체 browser·map/graph·TODO/링크·민감/캐시 제외, 심볼/helper/금융/문서 독립 감사 후 커밋·푸시한다. rollback은 해당 커밋 revert다.

## 조사 근거와 역할

symbol_navigator가 Serena로 raw BrokerCandle→timed converter→LTH driver/session 및 assessment/TS/UI/A-B를 추적했다. normalize_preview_chart_time은 분봉 raw label을 보존한다. 기존 LTH minute parser는 wall-clock HH:MM을 fixed KST 창에 적용하는 기존 미검증 모델이므로 새 시간대 진단으로 조용히 바꾸지 않는다. helper_curator는 기존 timestamp parser의 실제 동등성과 재사용 범위를 검토한다. stock_analyst는 세션 시계와 자료 coverage·성과 검증의 의미를 분리하도록 감사한다. 파일 구조/문서는 project_mapper에게 위임한다.

공개 근거: [NYSE](https://www.nyse.com/trade/hours-calendars), [Nasdaq](https://www.nasdaq.com/market-activity/stock-market-holiday-schedule), [NIST DST](https://www.nist.gov/pml/time-and-frequency-division/popular-links/daylight-saving-time-dst), [KRX 규정](https://regulation.krx.co.kr/contents/RGL/03/03010100/RGL03010100T1.jsp). 미국 일반 정규장09:30~16:00 ET와 실제 휴장/조기마감 달력은 다르다. Toss 봉 종료 계약은 [기존 공식 스펙 검증](provider-candle-contract-20261018.md)을 따른다. 계좌·토큰·운영 DB·실제/모의 주문을 사용하지 않는다.

## 검증 기록

- Rust focused minute_timing25개(순수20/command5), strategy_preview79개 통과. fmt/check all-targets/Clippy all-targets all-features `-D warnings` 경고0. 전체 `cargo test --offline -p kis-auto-trade`는 lib660+production integration4+doc3=667개 통과했다. DB 환경변수를 해제하고 로컬 mock/fixture만 사용했다.
- 실제 급반등 LTH 신호·양수 체결이 있는 fixture에서 진단 전후 signals/backtest/input hash의 정확한 일치를 확인했다. 일봉 minuteTiming 생략·유효0개 notEvaluable, 기존 generic timestamp Nanos 출력/회계 시나리오 정렬을 검증했다. 신규 진단은 기존 엔진 v17/LTH v4를 유지한다.
- focused mock UI는 최초12개 중11통과/1실패였다. LTH fixture가 generic lookAheadSafe=true를 상속한 문제를 실제false 계약으로 수정했다. 손상 컬렉션3개 저장/복원과 compact 헤더2개 표시를 추가한 최종15개 전체 통과를 확인했다. 제품의 정상 성과 적합성을 높이는 수정은 없다.
- tsc/FSD 통과. 전체 mock UI202/202개 통과(단일 최종 실행3.1분). web build·lockfile·map·graph·TODO/renderer·링크·제외파일 검증도 통과했다.
- symbol_navigator가 기존 시각→driver/세션·DTO·A/B 참조를 추적했고 helper_curator가 generic 파서 추출의 구문/UTC bytes 동등성과 최종 bounds/UTC/DST/gap/실행 보존을 감사했다. 초기 UI null 컬렉션 예외·compact 날짜수 불일치 지적은 구현·회귀 fixture로 해결했다. 최종 read-only 재감사에서 추가 P1/P2 없음.
- stock_analyst는 시간/봉 공개 모델·평일창/실제달력·미관측/누락확정·성과상태 보존을 감사하고 자동 진단 하위항목만 완료 가능하다고 확인했다. 실제 자료/성과 연구나 계좌/주문 검증으로 확대하지 않았다. project_mapper가 파일7개와 책임/흐름/상한을 지도에 반영했다.

- web build·lockfile·project-map·Graphify freshness·TODO 렌더/검사·renderer3개 통과. 변경 문서 로컬 링크188개·anchor2개·최근 이력5개·변경 소스1000줄 미만을 확인했다. Graph JSON/HTML의5966노드·13073edge·530community legend가 일치한다. Graphify의 기존 설정JSON4개 zero-node 추출 경고는 AST 도구의 제한이며 compiler 경고가 아니다. 실제 민감파일은 읽지 않았고 source/manifest/hash에 로컬 절대 경로가 없다.

## 완료와 남은 조건

자동 구현 하위항목1개만 완료했다. 열린 TODO는17→16개이며 P1-05/P2-12의 실제3개월 자료·시장 달력/시각·coverage/실운용 검증은 대기로 남는다. 다음 자동 전략 연구 우선순위는 P2-02 RSI 소유·진입/청산 비교다. 코드/심볼/helper·금융 의미·문서 독립 감사의 미해결 P1/P2는 없다. 커밋·현재 브랜치 origin push 후 로컬/원격 일치를 확인하며 캐시·민감파일은 제외한다.
