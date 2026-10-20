# P2-01 이동평균 교차 진입 필터 — 2026-10-20

상태: Done. 기준327739a, 브랜치fix/mac-security-storage-2026-10-06, 시작 작업 트리 clean. 사용자 변경 없음. 위험 보통(전략 후보 변경), 주문/리스크/provider 계약 변경 없음.

## 목표·계약·완료 조건

- TODO P2-01의 장기추세·MA기울기·ATR/밴드폭을 단독 구현하고 동일 기간/수량/비용의 순평가·실현/잔여·최저 완료손실·MDD·회전/노출·기각 근거를 보관한다.
- 기존5/20 입력 관측 교차와Sell/prevMA 갱신을 유지한다. 선택옵션0 기본, 필터는Buy후보에만 적용한다. 필터에 거절된 교차를 나중에 되살리지 않는다. 실제 보유는 기존 공통 주문 guard가 처리하며 소유 상태를 신호로 선반영하지 않는다.
- 새4필터는 strict 이전완료 DAILY OHLC만 사용한다. 기존live약10초/선택봉 관측교차와 일봉 필터를 분리하고 현재봉 OHLC는 판단 뒤 반영한다. 준비 재조회는 기존교차/청산을 초기화하지 않는다. ATR0/폭0은 유효자료가 충분하면 준비완료·양수임계값미달이다.
- strict save/두 preview, runtime잘못된옵션 Buy failclosed·기존Sell/legacycustom보존, 재설정/부족/오류/분봉혼입·분할 불변을 검증한다. UI 입력·저장·A/B·오류재시도·활성잠금·모바일·준비진단을 함께 검증한다.
- 운영 설정/민감 파일/실제·모의 주문/자동매매/운영DB는 접근하지 않는다. 신규IPC/저장schema/provider변경 N/A. 공개 frozen가격과합성/mock만 사용한다.
- 코드·금융·문서 위임 감사, Rust/TypeScript/브라우저·map/graph·TODO·링크·소스길이·캐시제외 통과 뒤 커밋하고 origin에 푸시한다. rollback은 해당 커밋 revert.

## 조사·역할

프로젝트 Rust/React/UI/FSD 브리지와 원본 스킬을 따른다. symbol_navigator가 Serena로교차 정의/참조·save/preview/UI 영향 범위를 추적한다. helper_curator가Graphify→Serena로일봉순수helper만동등성확인하며 lifecycle/counter/entry/guard를합치지않는다. stock_analyst가생산연구전정의와사후원장/비용/표본해석을독립감사한다. 파일추가 뒤project_mapper에맵/문서pass를위임한다.

## 사전 선언·검증

[design.json](../research/ma-cross-filters-20261020/design.json)을 후보실행전에고정했다.327739a/v16의flat1224·기준연속6/분할6/분기72와currentv17의기본1224호환·선택연속30/분할30/분기360,Stage A36/36/432를구분하며 아래 사후추가와최종48/48/576은별도로기록한다. SOXQ/SPY/QQQ strict252준비/752평가(2023-01-03~2025-12-31),q51/2/3·초기1천만원·FX1450·nextOpen·fee/slippage각10/20bp·tax/배당/FXscenario/overlay0·기존guard/risk를고정한다. 각분기는별도새계좌이며합산/복리하지않는다. 2026은신규성과에서제외하지만기본호환에는포함된다. 관측된가격이므로unseenOOS/운용승인으로해석하지않는다.

최종 실제 소스 기준 생산 결과·정산·독립감사·검증 결과를 아래에 기록했다.


## 구현·검증 진행 근거

MA 신규27개 focused와 strategy211개 통과(부분집합), fmt/checkalltargets/Clippyalltargets·allfeatures -Dwarnings 경고0. 최종 runtime 전체Rust는635단위+4통합+3doc=642개 통과했다. 기존MA관측·Sell·소유guard를 유지했고 slope활성 시 손상된 legacy기간이기본값으로대체되어검증을우회하던구현중문제도차단했다. symbol_navigator와helper_curator최종read-only감사P1/P2없음.

Stage A는36연속/36분할/432분기,기본1224전체backtest/signals exact,부모81,143산술+2,212이전일봉필터조건대조가통과했다. ATR0.5/폭2는모든후보통과라선택성/활성효능미평가였다. stock_analyst설계감사후23:03:28KST에각각고정ATR2/폭10스트레스후보하나씩만추가한사후탐색을선언했다. 최종48연속/48분할/576분기(624원장)이며원래36controls입력/경제hash를대조한다. 원래설계SHA f97577b60c7a1f5e444c53ed7248d354b598577c60c0ff7eaef08cff831afebd와Stage A요약·원장SHA를design에보존했다. 전체사전등록/OOS/사후임계값변경없음으로표현하지않으며추가재조정·우승선정없이종료한다.


최종 생산은48연속/48분할/576별도분기,원래36controls input/economic hash exact. 부모104,884기록산술+3,240prior-daily predicate검사가통과했다. 별도 stock_analyst 독립313,205개 오류0,금액/지표최대차이0.0100000072원/0.0000500070%p다. Stage A468원장6파트와36partition·unfiltered78fullbacktest/signals를직접대조했다. 기본1224는생산runner fail-on-mismatch 통과/archivedhash를감사한것이며독립current1224재실행주장은하지않는다. 추가금융P1/P2없음. 사후SOXQ일부개선을일반기본으로채택하지않으며기본5/20/선택옵션0을유지한다.


UI 최종20/20 focused와 tsc/FSD/lockfiles/build 통과, main356.44kB·charts175.25kB·MUI368.32kB. 정상numeric legacy기본값을보존하며 잘못된storedlegacy기간의원형을유지해slope활성시명시거부한다. 과거UI Number강제변환의손상타입까지호환한다는주장은하지않는다. rawstring을같은문자로fill해onChange가없던test복구절차와Toss봉단위selector는test에서보정했다. runtime source변경없이최종20개가통과했다.


## 최종 게이트

전체 브라우저191/191 통과(최종 UI/테스트,2.8분), focused20은 부분집합이다. 전체Rust642와 check/Clippy/fmt, tsc/FSD/lockfiles/build를 완료했다. compiler경고0. helper·심볼·금융·최종문서 독립감사에서 미해결P1/P2없음. map 생성inventory는 신규파일23/디렉터리2, 연구13파일·gzip2·평문0이다. graph5890nodes/12916edges/536legend와 HTML node/중복edgeCounter/legend가 일치하고 source freshness를 통과했다. 첫 graph 갱신 중 test 수정으로 freshness가 실패한 뒤 동결 소스로 다시 갱신했다. 루트 일반 문서는README/AGENTS/CLAUDE이며 index.html은 Vite 진입 파일이다.

P2-01 체크 후 열린 항목17개, 주황 사용자판단/검증대기 표시를 유지한다. renderer3개·최종링크/map/graph/diff·source1000줄상한·민감/캐시 제외를 확인하고 현재 브랜치에 커밋/푸시한다. 실제 provider·주문·자동매매·운영DB는 검증 범위가 아니다.
