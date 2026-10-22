# RSI 소유 피드백과 단독 진입·청산 후보 — 2026-10-22

상태: Done. baseline978640c, 시작clean. 목표는 TODO P2-02의 실제소유/조기진입/청산 후보를 현재 Rust replay로 검증하는 것이다.

## 범위와 완료 조건

- RSI period14/30/70 단순평균·zero-loss100·once-cross·매관측기준 갱신을 유지하며 actualowned quantity/entry basis를 사용한다. flatSell/heldBuy 제거와 실제체결량/부분/차단/미체결·checkpoint/restart/reconfigure를 검증한다.
- 선택 long_trend_period0..512/stop_loss_pct0..99.9/max_hold_observations0..1000000을 strict save/두preview·UI/A-B에 연결한다. 기본0비활성. 기존guard/리스크·손절예외·계좌경계 유지. 새설정은운용기본값을바꾸지않는다.
- [사전 고정 설계](../research/rsi-policy-20261022/design.json)의 legacy/owned/trend200/stop3/hold63을 SOXQ/SPY/QQQ 2023~2025 nextOpen·동일최초요청수량·각비용10/20bps에서 비교한다. 연속30/alternate30/분기초기화360회, 이전엔진baseline과1224기본(비RSI1122) 비교를 보관한다. 성과 개선/무신호/준비부족/작은표본/기말보유/비용경로·노출을분리한다. 사후튜닝없음.
- backend/frontend focused/전체·fmt/check/Clippy경고0·tsc/FSD/build·graph/map·TODO/문서/스킬·독립비용/원장감사 후커밋push. 실제계좌/API/주문·자동매매·운영DB/민감파일을사용하지않는다.

## 설계 판단

symbol_navigator가 실제RsiParams/RsiStrategy와 save/driver/live feedback/checkpoint/UI 참조를 추적한다. helper_curator가 기존observation의순수동등성만검증한다. stock_analyst가 단순RSI/원래교차재시도없음·추세입력단위·실소유age/보호재시도·비용/표본해석을독립감사한다. 파일/지도는project_mapper에위임한다.

## 검증 기록

실제 구현·생산 실행·기록 정산·독립 감사와 최종 게이트 결과를 아래에 기록한다.

- baseline는 git978640c 원본과 주요strategy/driver/simulation/Cargo.lock 바이트 동일성을 [검증](../research/rsi-policy-20261022/baseline-source-validation.json)했다. actual Rust baseline flat1224/continuous6/partition6/reset72 완료. sourceHash·252/752 경계·unique정렬일자·최초수량51/2/3을확인했다.
- UI worker는 tsc/FSD·Page992줄·최종focused15/15를통과했다. 최초15중14통과/1실패는테스트가localStorage A/B index배열을slot으로오인한fixture문제였고테스트만수정했다. actual계좌/주문없이mock만사용했다. 전체217개UI와webbuild도아래최종기록대로통과했다.

- backend 신규27개(순수17/command6/save3/exactowner복원1)와classic60/60통과. stop우선순위/실원가·cap의실제nextOpen비용guard거절2회/보유age1·2·3지속후보/마지막잔량전량체결을검증했다. 초기단위fixture RSI기대33→66교정, command거절assert를실제signaldate가아닌nextOpen집행시각으로교정했다. 생산/guard는이fixture수정으로바꾸지않았다.
- fmt/check all-targets/Clippy all-targets all-features `-D warnings` 경고0. 부모전체Rust lib687+productionintegration4+doc3=694개통과. DB환경변수해제·로컬fixture/mock만사용했다. 전체UI217/217(단일최종3.3분), webbuild·lockfiles·tsc/FSD통과.
- 생산연구는비RSI1122exact/RSI102delta, 연속30/동일경로분할30/독립분기360완료. 부모정산61,782항등식통과, owned78개의실accepted경로와legacy가같고hold63은owned와전체report/signals78개같았다. [결과](../research/rsi-policy-20261022/report.md)는부적합신호제거·비용부호반전·노출·무신호/미청산·미발동63효능미평가를구분한다. 실제운용설정변경/사후튜닝은없다.
- 초기독립코드리뷰는P1/P2없고검증gap(capguard거절증거)·README끝빈줄을지적했다. 둘다수정후최종read-only재감사에서해결/추가P1/P2없음을확인했다. stock_analyst의독립기록정산·해석감사도통과했다.

- 최종 [stock_analyst 독립 감사](../research/rsi-policy-20261022/independent-audit.json)는185,202검사·오류0이다. 390개생산원장과RSI102개차이의실제기록·정산/노출/기말·정책카운터·baseline원본및최종소스manifest를검증했다. 비RSI1122·분할30은부모생산실행증거를확인했으며독립재실행으로중복집계하지않았다. 최대금액오차0.010000001639원/지표0.000050000000%p(허용0.011원/0.000061%p)다.
- 독립audit중발견한최근commandfixture소스hash만최종manifest에갱신했다. runtime/가격/경제원장SHA는변경없으며본체원장재실행은불필요했다. validation과독립auditJSON은자기자신/상호변경되는audit산출물hash의순환참조를만들지않는다.
- Graphify freshness·JSON/HTML6049노드/13255edge/548community legend일치,TODO렌더/검사·renderer3개·문서링크232개·source1000줄미만·로컬절대경로/캐시/민감파일제외검사를통과했다. 설정JSON4개zero-node는기존Graphify추출한계이며compiler경고가아니다. project_mapper가두지도와새파일책임·기각/미평가/관측단위를갱신하고최종문서drift없음을확인했다. 최종링크235개/anchor3개·최신5건·연구18파일(gzip2/평문0)·TODO15·project-map검사를통과했다.

## 완료와 남은 조건

P2-02만완료하여열린TODO는16→15개다. 기본RSI14/30/70·선택옵션0과기존공통주문제한을유지하며최적후보를자동선택하지않았다. P1-05/P2-12실제자료·운용검증은그대로남고다음자동연구는P2-07돌파실패의실제청산/보호비교다. 검증·최종문서게이트후현재브랜치에커밋·origin push하고로컬/원격일치·clean을확인한다. rollback은해당커밋revert이며강제push는하지않는다.

현재production pathdependency의연구harness 자체도 `cargo clippy --offline --manifest-path .cache/rsi-research-harness/Cargo.toml --target-dir target --all-targets --all-features -- -D warnings`를경고0으로통과했다. 연구manifest110개현재파일hash·원장SHA·독립audit오류0/185202를최종재확인했다.
