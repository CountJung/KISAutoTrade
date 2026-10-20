# P2-10 평균회귀: 실제 소유와 단독 진입·청산 후보 — 2026-10-20

상태: 구현·연구·독립 코드/금융 감사와 검증 완료. 열린 목록의 P2-10을 사용자 판단 없이 수행했다.

## 범위와 완료 조건

- 기존 현재 포함 period20 모집단 분산/ddof0·표준편차2·strict price<lower 진입/price>upper 청산과 native 평균가 대비 stop5를 보존한다. stop0은 비활성이 아닌 원가 이하 손절이다.
- raw tick/live가 확정 보유·원가를 선반영하지 않게 하고 exact owner 수량/평균가·미체결 origin·기록 보유 나이를 분리한다.
- upperBand 기본에서 middleBand(현재>같은mean), trend200(현재>현재 제외 이전200), maxHold63을 각각 단독 비교한다. 기본 옵션 자동 변경·가설 조합·격자 최적화는 하지 않는다. Stage A 뒤 단일21관측 후보를 추가한 사후 탐색을 구분한다.
- 기존 MeanReversion reset의 가격 버퍼 유지와 1분 history warmup no-op, TrendFilter 실행 코드를 유지한다. 기존1분 경로가 준비 분봉을 사용하지 않는 한계와 준비 진단을 숨기지 않는다.
- stop/maxHold은 지표 준비 부족 중에도 평가하고 실제 체결 뒤 나이는 같은 수량·부분체결·준비자료·설정 변경에 초기화하지 않는다. 유효 metadata 없이 복원한 보유의 나이 불명 상태는 활성 maxHold에서 보수적 매도 후보다.
- save/단기·연속 preview의 strict params, UI raw/stored 값·A/B·실제 키보드·오류/실패 재시도·활성 잠금·모바일을 함께 검증한다.
- frozen QQQ1주 252+64봉의 기존 비용/체결4경로 회귀를 유지한다. 새 연구 QQQ3주와 합치지 않는다.
- 검증·독립 연구 해석/코드·문서·map/graph 게이트 완료 뒤 현재 브랜치에 커밋·origin 푸시한다.

## 사전 선언과 자료

[design.json](../research/mean-reversion-policy-20261020/design.json)에 실행 전 후보·공식·범위·자료 SHA를 고정했다. archived474aa06/v15의 flat1224·연속6/분할6/분기72와 currentv16의 비평균회귀1122 exact·평균회귀102 delta, 연속24/분할24/분기288을 구분한다. Stage A는 합30연속/30분할/360별도분기다.63관측 미발동을 확인한 뒤22:30:35 KST에21관측 하나를 선언해 최종36연속/36분할/432별도분기(468원장)를 실행했다. 원래30개 대조군의 입력/경제 hash는 동일하다. 전체 사전등록/OOS로 표현하지 않는다.

SOXQ/SPY/QQQ의 strict252준비와2023-01-03~2025-12-31의752평가봉, 최초고정수량51/2/3, 원화1천만원·FX1450·nextOpen·수수료/슬리피지각10/20bp·세금0·배당/FX시나리오없음·보호overlay0·기존guard/risk를 유지한다. 분기는 각각 새1천만원 계좌이며 수익률 합산/복리하지 않는다.

2026봉은 새 후보 성과 비교에서 제외하되 고정 compatibility와 기존 QQQ regression에는 사용한다. 둘 다 관측한 가격이므로 독립 OOS/통계 우월성으로 부르지 않는다. 같은요청수량은 같은 실제 노출이 아니며 비용을 바꾸면 guard/체결/후속 손익이 달라질 수 있다. 실현/잔여·기말 청산비용·최소 완료손익·MDD·회전/노출과 낮은 표본을 분리한다.

## 역할과 안전

symbol_navigator가 Serena로 params/state·공개 재수출/default/factory·실제 소유 metadata·save/two preview·기존 QQQ/준비/보유 tests와 UI 영향 범위를 추적했다. backend와 UI를 분리 위임했다. helper_curator 전용 세션 재개가 thread 한도로 거절되어 기존 심볼 에이전트가 해당 역할의 Graphify CLI→Serena 정의/참조·동등성 체크리스트를 수행했다. 확인한 기계 동작은 해당 역할 지침을 적용한 기존 작업자에게 공용 승격을 맡기고 새 MeanReversion 호출자와 기존 Deviation 호출자를 분리 소유했다.

동등한 관측 나이/metadata codec·이전N 추세 버퍼·bounded optional 정수만 좁은 전략 내부 공통 모듈로 승격한다. kind·symbol/scope 검증과 전략별 stop/band/entry/initialize/reset/reconfigure는 각 전략에 남긴다. default stop0와 strict/inclusive 차이를 합치지 않는다.

실제/모의 주문·자동매매·민감 계정 파일·운영 DB는 사용하지 않는다. 공개 고정 가격과 합성/mock fixture만 사용한다. 주문 runtime/provider/리스크 gate는 변경하지 않는다.

## 검증과 전달

최종 runtime 소스는2026-10-20 22:22:58 KST에 고정했다. [생산 결과·한계](../research/mean-reversion-policy-20261020/report.md)에 각 비용·종목·후보의 순평가/실현/잔여·기말 가정청산, MDD·회전·노출·최저 완료손익과 별도 분기 개선/악화를 기록한다.63관측은 미발동으로 효능 평가 불가다. 사후21은 QQQ 손실 일부 감소와 SOXQ/SPY 악화, 비용 guard로22/24관측까지 지연된 후보를 구분한다. 기존 기본값을 유지한다.

- fmt, cargo check --all-targets, Clippy --all-targets --all-features -- -D warnings 통과, 빌드 경고0.
- focused289개(strategy194/preview68/orderfeedback8/configuration19)와 전체Rust618개(lib611/integration4/doc3) 통과. focused는 전체의 부분집합이다.
- 기존 QQQ4경로 회귀와 generic_tests.rs를 보존했다. TrendFilter 실행 bytes SHA `1cf48fd7a24b09b07a255da245a04ba4c738d004f6c4be537c871c35aeed28be` 동일.
- 전체 UI171/171 통과 뒤 최종1분 준비 한계 설명문 반영 후 focused15/15 재검증. tsc/FSD/lockfiles/build 통과, main347.70kB·charts175.25kB·MUI368.32kB.
- 공유 observation helper8개·계약 test5개에 대해 실제 helper_curator가 호출자별 소유/종류 검증 경계를 확인했고 최종 심볼·회귀 감사에서 P1/P2 발견0.
- 부모 연구 산술74,968개 통과. Stage A 독립175,490개 오류0과 별도로 최종468원장+flat102 독립219,769개 오류0. 금액/지표 최대 차이0.0100000091원/0.0000500161%p이며 추가 금융 P1/P2 발견0.

TODO18개 동기화와 renderer3개(모바일/키보드/주황 대기 색상 포함), project-map/graph freshness를 통과했다. 최종 문서 pass에서 발견한 project-map의 Stage A 수치를 최종 Stage B와 구분하도록 수정했고 재감사에서 finding0이다. 변경 Markdown 상대링크173개·source1000줄 이하·민감/캐시 제외를 확인했다. 그래프와 HTML은 node5771/edge12650/legend537 및 edge 중복 Counter까지 일치한다. 루트 일반 문서는 README/AGENTS/CLAUDE이며 index.html은 Vite 진입 파일이다. rollback은 해당 커밋 revert다.
