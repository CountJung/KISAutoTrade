# 작업 기록: P2-11 추세 필터 실제 소유·청산 비교

## 1. 메타데이터

- 작업 ID: SOXQ-P2-11
- 상태: Done — 구현·연구·독립 정산·전체 검증 완료
- 작성2026-10-23, 구현/갱신2026-10-25
- 브랜치: fix/mac-security-storage-2026-10-06
- 시작 HEAD70df3b3. 중단 중 b3c441c까지 추가된 문서3커밋은 보존했다. 연구 기준은 별도81fa00d이며 브랜치를 되돌리지 않았다.
- 관련 [TODO](../todo.md), [고정 설계](../research/trend-filter-policy-20261023/design.json), [계약](../research/trend-filter-policy-20261023/protocol.md), [보고서](../research/trend-filter-policy-20261023/report.md), [독립 감사](../research/trend-filter-policy-20261023/audit-findings.md)
- 위험: 높음. 실제 소유/보호·저장/복원·일반 preview와UI를 함께 바꾸며 fixture/mock·공개 동결자료만 실행했다.

## 2. 목표와 비목표

원래200/5/20·현재 포함 산술 MA·strict조건·세MA준비후장기이탈을 유지하면서 실제 소유/native평균·잔량청산·pending 기원·보유 나이/peak/무장을 구현한다. 고정5후보의 실제 청산·비용/노출·실현/잔여를 생산 Rust API로 비교한다. 사전 문서 준비로 구현/실행을 대체하지 않는다.

선택값 자동 승격·사후 창/임계값/수량 조정·guard 완화·별도 신호 모델·실전/모의 주문·자동매매 시작·계좌/운영 DB·민감 파일 접근은 하지 않는다. 자료·미발동·작은 표본을 효능0/운용안전성으로 바꾸어 표현하지 않는다.

## 3. 완료 조건

- [x] 실제 소유/native 평균·잔량·거절/부분/동량/pending·unknown 나이와7필드원자적checkpoint
- [x] 이탈 기본1/단독2·수익5/되밀림3·보유63과보호우선·준비부족평가
- [x] 원래 MA/strict/비정렬기간·warmup/reset가격보존·핵심/선택변경·재시작/청크수명
- [x] 엄격 저장/두preview·raw초안/current-A-B/legacy미확인·잠금·모바일·실패/지연응답
- [x] 기준 archive생산API와현재생산API로30연속/30분할/360별도분기·비대상1122호환·대상102비교
- [x] 390원장독립정산·gross/net·실현/잔여/기말·MDD·실체결수량/노출/회전/차단과비교보유
- [x] 전체UI·최종지도/문서/그래프·생성물
- Git 커밋·원격 확인: 최종 응답과 현재 브랜치 이력 참조.

## 4. 조사와 근거

Serena 지침·저장소 지침·관련Rust/React/FSD/UI스킬을 먼저 읽었다. symbol_navigator가실제OrderManager caller·trait·저장·두preview·UI경로를추적했다. `apply_strategy_ownership`은sync전에had_metadata를읽고sync→pending→조건부restore하므로positive unknown도metadataSome이어야반복feedback이확인streak를지우지않는다. 실제caller fixture로cold restore·동량·부분을검증했다.

helper_curator는Graphify후보/Serena정의·실참조로observation의정수/advance/sync age3개만재사용한범위가타당함을확인했다. StrongClose전체상태와Trend를공용화하지않고kind/이탈streak재관측을독립유지했다. UI의공용numberDraft만재사용하며전략별입력범위/오류/기본은local이다.

공식KIS/Toss동작변경·외부사실조회는없다. 소스표현bool/설정Sell수량만으로실제초과매도사고를단정하지않고기존공통clamp/guard·소유feedback을유지했다.

## 5. 영향 범위와 구현

- `strategy/mean_trend.rs`: 실수량/native평균·관측나이/peak/무장·연속확인과잔량Sell. 합은u128, 합→f64→나눗셈보존. 원래기본신호reason도보존한다.
- `mean_trend/trend_filter_params.rs`: 필드별parse와strict raw검증. 누락기본/비정렬MA허용, 손상runtime진입차단·유효보호/설정불명잔량후보를분리한다.
- `mean_trend/trend_filter_protection.rs`: 엄격7필드version1/kindtrendFilter. 전항목성공뒤원자게시·복원streak0. 양수unknown도Some.
- 저장configuration·두preview공통driver검증과두pending연결·enginev22. 기존generic분봉지원·DTO/API/guard경계보존.
- 전략/재설정·실제orderfeedback·설정저장·preview fixture. 실보유2/설정7기존fixture는Sell2로교정했다.
- `trendFilterControls.tsx`:7필드raw타입/단위/오류·현재/A-B·v22이전미확인. `strategyPolicyControls.tsx`:기존9전략+Trend의조립만추출. Page982행·새파일1000행미만.
- 연구실행기·압축390원장·실행/소스hash·부모정산63559개·별도감사171604개와해석을보관한다.

신규IPC/DTO/query/polling·broker/account scope·운영설정·보관소schema변경은없다.7필드metadata만기존exactowner원장경계에추가된다. 저장응답의removeDraftIfUnchanged/scope epoch를유지해저장중새편집을보존한다.

## 6. 금융 안전 검토

실제주문가능성: 이번실행은없음. 생산전략을수정하지만테스트는합성fixture/mock·공개과거자료이며계좌/API로실행하지않았다.

- [x] 실전/모의주문·자동매매시작·broker/accountcredential·운영DB·민감파일접근없음
- [x] exactowner·계좌/시장분리와실제잔량·원가·pending/거절/부분보존검증
- [x] 공통clamp/guard/risk·비상정지/일일·연속손실·주문제한우회없음
- [x] 원자적metadata검증·저장전raw거부·새설정실패/늦은응답초안보존
- [x] ready/미발동/완료0·실현/잔여·비용/노출차이와실운용/OOS미검증구분

기존소유교정의과거경제효과는78대응원장/102flat에서0차이로관측됐다. 이는효능0주장이나기존사고입증이아니다. 수익/보유후보도실제손실/보유시간을보장하지않는다.

## 7. 실행과 결과

사전설계SHA f2a3f801da05d77544580b3fabc265197bdc26cd7072859df9f2f4428262040f를보존했다. 부모가81fa00d/v21 archive생산라이브러리와현재v22생산API를직접실행했다. pathharness는offline호환의존성을별도해석했으며원래Cargo.lock과동일빌드를주장하지않는다. 소스7파일Git객체bytes대조·각harnesslockSHA를baseline-source-validation.json에남겼다.

확인2는SOXQ·QQQ일부개선/작은완료표본·SPY악화다. 수익보호/보유63은SOXQ20bp−3.6376/−3.5244%, 회전·차단증가와경로변경이다. 기본승격은기각하며확인1/수익·보유0을유지한다. 연속수익보호58·보유63 45건실체결, 분기수익보호26·보유63 0건이다. 보유63분기72개경제경로owned동일/16개마지막pending만추가이므로분기효능은미평가다.

SOXQowned20bp+9.5760%는실현−240553.37/잔여+1198152.86원이다. 비교보유51/2/3의실제노출과기말추가매도비용·비중guard로0주인분기를분리했다. 세부표/차단/실현·잔여/기말원가/표본조건은보고서·독립감사JSON에있다. 분기수익률은합산/복리하지않는다.

## 8. 도구·독립 리뷰·문서 게이트

symbol_navigator의독립코드리뷰는P1/P2결함없음이었다. helper_curator의독립helper/구조감사도필수수정없음이었다. stock_analyst는허용된독립감사3파일만작성하고390원장·171604검사·오류0및JSON재실행동일을확인했다. 부모는생산실행·추가집계와UI를맡았다.

project_mapper는프로젝트생성인벤토리와책임/흐름·PROJECT_MAP/ARCHITECTURE를갱신하고검사했다. 부모는IPC/v22계약·Rust/UI/FSD스킬·HARNESS_MAP·사전계약현재상태·결과/작업/TODO/AGENTS를동기화한다. Graphify는code-only갱신이며노드5000제한으로HTML이생략되자10000상한에서현재그래프HTML을재생성했다. 외부LLMlabeling은하지않는다.

## 9. 검증

| 검증 | 결과 |
|---|---|
| cargo check --offline --workspace --all-targets --locked | PASS·경고0 |
| cargo test --offline --workspace --lib --locked | PASS789·실패0·ignored0; 환경미설정외부DB서버contract는내부skip이므로실서버검증아님 |
| cargo clippy --offline --workspace --all-targets --all-features --locked -- -D warnings | PASS·경고0 |
| npx tsc --noEmit / npm run check:fsd / npm run build:web | PASS |
| 새Trend/StrongClose/Volatility집중Playwright | PASS84개(새Trend14)·mock |
| CI=1 npx playwright test --workers=4 | PASS322개(23파일,production290/source32) |
| 기준/현재Rust연구 | PASS30연속/30대체분할/360별도분기·비대상1122호환·대상102비교 |
| validate.py / independent-audit.py | PASS63559 /171604검사·오류0; 최종hash재생성 |
| project-map/check·graphify/check·todo/render/check·링크·diff | PASS; 최종commit전 재확인 |

처음전체Rust26건·UI서버시작은sandboxloopback EPERM으로실패했고제한해제환경에서전체Rust789·집중UI84가통과했다. 전체UI첫실행은321통과/기존돌파실패경계입력1실패였고해당fixture단독10재실행은통과했다. 첫저장request수만기다리던경계입력시험은response완료·초안정리를기다리도록수정했다. 의도적인저장중편집시험은별도보존했으며제품기능으로경계를우회하지않는다. 보완뒤전체322개가통과했다.

실제운영DB/주문/자동매매/장중체결가능성검증: NOT RUN.

## 10. 완료 보고

구현·고정생산비교·독립정산과전체Rust789/UI322·문서/지도게이트를마쳐기존P2-11을완료로갱신했다. 검증된변경은같은브랜치에서커밋·origin푸시로전달한다. 실전배포·설정변경·실운용효능완료를뜻하지않는다. 같은브랜치의기존수정과중단중추가된문서3커밋을보존했고강제푸시하지않는다.
