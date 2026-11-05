> 최신 상태: [P1-05(v7) 일봉 진단·분봉 지표 분리](#validation-p1-05) 구현 검증 완료, 실제 3개월 분봉 재평가는 자료 대기 중이다. [P1-04(v6) 현재시각 경로 차단](#validation-p1-04), [P1-03(v5) 날짜 이벤트](#validation-p1-03), [P1-01(v4) 준비 상태](#validation-p1-01). 아래 본문은 초기 v2 관측이다.

# SOXQ 최근 3개월 자동투자전략 평가 — 2026-10-03

> 이 문서는 수정 전 v2 엔진 분석 기록이다. 이후 **SOXQ-P1-02 고가/종가 warmup 오류를 수정**했고 [수정 후 검증·재평가](#validation-p1-02)에 차이를 기록했다. 아래 수치와 charts.html은 최초 분석 시점의 결과를 보존한다.

현재 프로젝트의 13개 전략을 실제 Rust 시뮬레이션 라이브러리로 실행했다. Python으로 매매 전략을 새로 구현하지 않았고, 실계좌 접속·모의계좌 주문·자동매매 시작·저장된 전략 설정 변경은 하지 않았다.

이격도·가격조건·RSI가 이번 종가 replay에서 양의 순수익을 보였지만, 표본 수와 엔진 제약 때문에 전략 순위를 확정할 수 없다. 우선 개선 대상은 **사전 데이터/준비 상태, 고가·종가 초기화 구분, 일자별 이벤트, LTH 역사 시각/분봉 처리**다. [TODO](../../todo.md)에 P1 5개와 P2 16개(13개 전략별 평가 과제 포함)를 남겼다.

## 입력과 공통 가정

- 요청한 최근 3개월: 2026-07-03~2026-10-02. 해당 데이터의 실제 평가 봉은 **2026-07-06~10-02, 64거래일**이다.
- 사전자료: 2025-07-02~2026-07-02의 **252거래일**. 평가 64봉과 합쳐 316봉이므로 공통 preview의 500봉 제한 안에 있다.
- [Yahoo Finance SOXQ 실제 일봉](https://finance.yahoo.com/quote/SOXQ/history/)의 raw OHLCV를 내려받아 USD 가격을 센트로 정규화했다. API URL·SHA-256·원본 날짜는 [replay-input.json](replay-input.json)에, 실제 사용 316봉은 [candles.csv](candles.csv)에 보관한다.
- 초기자본 10,000,000원, 주문당 10주, 최대 포지션 30%, 고정 환율 1,450원/USD, 매수/매도 각각 수수료 10bp와 슬리피지 10bp. 매도 거래세는 연구 가정 0. 실제 broker 요율이나 납세액 검증으로 해석하지 않는다.
- 일일 손실한도 1,000,000원, 변동성 수량 산정 비활성, 공통 guard 활성. 기존 기본 전략 파라미터를 사용한다.
- 가격조건 전략은 빈 종목 목록으로는 작동하지 않아, **평가 전에 알려진 7/2 종가 $99.29**를 고정 매수가로 입력했다. 익절 5%/손절 3%/10주. 평가 구간을 보고 최적화한 값이 아니다.
- LTH는 SOXQ/해외/10주로 등록하고 나머지는 기본값이다. SOXQ를 넣은 엔진 진단이며 레버리지 상품 결과로 일반화하지 않는다.
- 배당, 환율 변동, 세후 총수익, 실제 호가/유동성, 장중 손절 경로는 포함하지 않는다. 종료일에 강제 매도하지 않으며 열린 포지션은 종가로 평가한다.
- **수익률/MDD는 전체 계좌 기준**이다. 첫날 10주 보유 기준 사용액은 약 147만원(자본 약 14.7%)으로, 전액 투자 SOXQ 수익률과 직접 순위를 비교하지 않는다. exposurePct는 시간/관측 비율이며 자본 투자 비중이 아니다.

## 실행 방식과 해석

1. **사전자료 제공:** 모든 공통 preview에 252봉 prefix와 warmupCount=252를 전달하고 같은 64거래일을 평가했다. MA·RSI·모멘텀·이격도·연속 상승/하락·돌파 실패는 historical initializer가 없으므로 실제 지표는 평가일에 새로 모인다. “모든 전략 warmup 완료”를 뜻하지 않는다.
2. **현재 UI 기본 방식:** 64봉만 입력하고 warmupCount를 생략했다. 신고가·강한 종가·변동성 확장은 앞의 32봉을 warmup으로 소비해 **8/19부터 32거래일**만 평가한다. 표의 두 방식은 엔진 차이를 확인하기 위한 것으로 평가 기간이 다른 행끼리 순위를 비교하지 않는다.
3. LTH는 generic preview 진단과 기존 전용 preview_signals_with_execution 일봉 어댑터를 둘 다 실행했다. 전용 경로의 기존 합성 KST 09:15 시가/16:50 종가 이벤트와 체결 callback을 재현했다. 실제 NASDAQ 장중 시각이나 15분 뒤 가격을 복원한 것이 아니다.
4. 수수료·슬리피지를 각각 20bp로 높여 같은 입력을 재실행했다. 이는 비용 민감도이며 broker 실제 비용이나 장중 체결 검증이 아니다.
5. 기본 파라미터를 고정했으며 최적화 탐색은 수행하지 않았다. 엔진의 시간순 70/30 구간 분할(7/6~9/3, 9/4~10/2)은 표본 진단이며 반복 walk-forward나 독립 검증을 대체하지 않는다.

## 전략별 결과

사전자료 제공 방식의 전체 계좌 성과. 강한 종가·변동성 확장·LTH는 엔진 진단이며 투자 성과 판정에서 제외한다. 평균회귀·추세 필터는 고가 warmup 영향이 있어 초기화 수정 후 재평가해야 한다.


| 전략 | 순수익률 | 계좌 MDD | 완료 거래 | 승률(완료 거래) | 차단 주문 | 기말 보유 | 해석 |
|---|---:|---:|---:|---:|---:|---:|---|
| 이동평균 교차 | -1.1197% | 2.2089% | 2 | 0.0% | 0 | 10주 | 완료 2회 모두 손실; 사전 초기화 미지원 |
| RSI | +2.0224% | 1.0140% | 2 | 100.0% | 0 | 0주 | 완료 2회 승리; 표본 부족·사전 초기화 미지원 |
| 모멘텀 | -0.4868% | 1.4493% | 2 | 0.0% | 9 | 10주 | 완료 2회 손실; 무보유 매도 차단 9건 |
| 이격도 | +2.4987% | 0.8931% | 2 | 100.0% | 10 | 0주 | 완료 2회 승리; 무보유 매도 차단 10건 |
| 52주 신고가 | +0.0000% | 0.0000% | 0 | — | 0 | 0주 | 무거래; 252일 롤링 고가·준비 상태 검증 필요 |
| 연속 상승/하락 | -0.6445% | 2.9268% | 2 | 0.0% | 0 | 10주 | 완료 2회 손실; 보유 관측 비율 81.25% |
| 돌파 실패 | -0.0005% | 0.8096% | 1 | 0.0% | 0 | 10주 | 완료 1회 손실; 미청산 보유 10주 |
| 강한 종가 | +0.0000% | 0.0000% | 0 | — | 0 | 0주 | 전일 조건이 날짜별 갱신되지 않아 평가 신뢰 불충분 |
| 변동성 확장 | -0.0300% | 0.0300% | 0 | — | 0 | 10주 | 3개월을 당일 범위로 누적해 평가 신뢰 불충분 |
| 평균회귀 | +1.4402% | 1.6683% | 2 | 50.0% | 0 | 0주 | 고가를 종가로 warmup; 초기화 수정 후 재평가 |
| 추세 필터 | +0.8722% | 1.6917% | 0 | — | 0 | 10주 | 고가 warmup 영향; 미청산 보유 10주 |
| 가격조건 | +2.2408% | 1.5896% | 12 | 58.3% | 0 | 0주 | 7/2 종가 기준 연구 조건; 완료 12회·높은 회전율 |
| 레버리지 추세 보유(LTH) | +0.0000% | 0.0000% | 0 | — | 0 | 0주 | 전용 일봉도 무신호; 실제 장중 평가 불충분 |

SOXQ 평가 첫 종가 USD 101.53 → 마지막 USD 103.41, 비용 전 가격 수익률 +1.8517%.
- 동일 10주 매수 후 보유: 계좌 순수익률 +0.2431%, MDD 2.8725%.
- 전액에 가까운 매수 후 보유(투자 비중 차이 참고용): 계좌 순수익률 +1.6290%, MDD 19.1942%.

LTH 전용 일봉 실행도 신호·체결 0건/수익률 0.0000%다. 장중 진입 창이 필요한 전략의 우수/열위 판단 근거로 사용하지 않는다.

## 현재 UI 기본 방식과 차이

| 전략 | UI 평가 시작 | UI 기본 순수익 | 사전자료 제공 순수익 | 설명 |
|---|---|---:|---:|---|
| 이동평균 교차 | 20260706 | -1.1197% | -1.1197% | 동일 결과 |
| RSI | 20260706 | +2.0224% | +2.0224% | 동일 결과 |
| 모멘텀 | 20260706 | -0.4868% | -0.4868% | 동일 결과 |
| 이격도 | 20260706 | +2.4987% | +2.4987% | 동일 결과 |
| 52주 신고가 | 20260819 | +0.0000% | +0.0000% | 입력 절반 warmup으로 평가가 32일 단축됨 |
| 연속 상승/하락 | 20260706 | -0.6445% | -0.6445% | 동일 결과 |
| 돌파 실패 | 20260706 | -0.0005% | -0.0005% | 동일 결과 |
| 강한 종가 | 20260819 | -0.5586% | +0.0000% | 입력 절반 warmup으로 평가가 32일 단축됨 |
| 변동성 확장 | 20260819 | +1.3575% | -0.0300% | 입력 절반 warmup으로 평가가 32일 단축됨 |
| 평균회귀 | 20260706 | +0.0000% | +1.4402% | 고가 초기화 영향; 사전자료 차이 |
| 추세 필터 | 20260706 | +0.0000% | +0.8722% | 고가 초기화 영향; 사전자료 차이 |
| 가격조건 | 20260706 | +2.2408% | +2.2408% | 동일 결과 |
| 레버리지 추세 보유(LTH) | 20260706 | +0.0000% | +0.0000% | LTH 시각 모델 검증 불충분 |

## 비용 민감도

수수료·슬리피지 각각 10bp → 각각 20bp. 기존 guard의 비용 판단과 체결 비용을 모두 반영했다.

| 전략 | 각10bp 수익 | 각20bp 수익 | 변화(%p) | 각20bp 완료 거래 |
|---|---:|---:|---:|---:|
| 이동평균 교차 | -1.1197% | -1.2549% | -0.1352 | 2 |
| RSI | +2.0224% | +1.9147% | -0.1077 | 2 |
| 모멘텀 | -0.4868% | -0.6226% | -0.1358 | 2 |
| 이격도 | +2.4987% | +2.3903% | -0.1084 | 2 |
| 52주 신고가 | +0.0000% | +0.0000% | +0.0000 | 0 |
| 연속 상승/하락 | -0.6445% | -0.7816% | -0.1371 | 2 |
| 돌파 실패 | -0.0005% | -0.0851% | -0.0846 | 1 |
| 강한 종가 | +0.0000% | +0.0000% | +0.0000 | 0 |
| 변동성 확장 | -0.0300% | -0.0600% | -0.0300 | 0 |
| 평균회귀 | +1.4402% | +1.3333% | -0.1069 | 2 |
| 추세 필터 | +0.8722% | +0.8440% | -0.0282 | 0 |
| 가격조건 | +2.2408% | +0.9777% | -1.2631 | 11 |
| 레버리지 추세 보유(LTH) | +0.0000% | +0.0000% | +0.0000 | 0 |

## 개선 판단

- 이격도/RSI/가격조건은 후속 검증 후보로 남긴다. 이격도와 RSI의 100% 승률은 완료 2회에서 나온 값이다. 승패가 없는 경우 PF=null은 수익성 무한대를 뜻하지 않는다.
- MA 교차·모멘텀·연속 상승/하락은 이번 구간에서 완료 거래 모두 손실이었다. 초기화 및 체결 모델부터 바로잡은 뒤 장기 추세/변동성 필터를 비교한다. 돌파 실패는 한 번의 손실과 열린 포지션이 있어 결과 판정이 제한된다.
- 가격조건은 24회 체결/12회 완료 거래와 회전율 325.2350%를 기록했다. 익절·손절 직후 재진입과 비용 의존성을 공통 guard 관점에서 점검한다.
- 무보유 매도 차단은 모멘텀 9건/이격도 10건이었다. guard가 주문을 막았으므로 실제 체결 손실이나 무조건 매매 실패로 해석하지 않는다. 전략의 신호·보유 상태 동기화를 개선할 근거다.
- 평균회귀·추세 필터의 초기 고가/종가 혼용, 강한 종가·변동성 확장의 날짜 처리, LTH 시각/지표 간격은 수익률 조정보다 먼저 해결한다. 수정 후 이 데이터에 재실행해 신호와 성과 변화까지 비교한다.

## 재현과 파일

- [대화형 SOXQ 일봉/체결/계좌 차트](charts.html): 13개 전략 선택. 라이브러리와 데이터를 내장해 외부 CDN 없이 연다. LTH 차트는 전용 일봉 근사 결과를 표시한다.
- [전체 기본 실행](results.json), [비용 각20bp 실행](results-cost20.json), [입력/출처/hash](replay-input.json), [사용 OHLCV](candles.csv), [실제 라이브러리 호출 runner](runner.rs), [검증 로그](validation.json).
- 현재 작업 저장소: /Users/jsjmac/Workspace/MacWorking/KISAutoTrade. 이전 외장 드라이브 경로는 이번 실행 시 존재하지 않았다.
- runner는 현재 저장소의 src-tauri 라이브러리를 path dependency로 연결한다. 앱의 run()이나 provider API를 호출하지 않는다. SOXQ_COST_BPS=20이면 비용 스트레스를 재현한다.

재현 예시(저장소 루트에서 실행):

~~~bash
research_dir="$PWD/docs/research/soxq-3m-20261003"
harness_dir="$(mktemp -d /tmp/kis-soxq-replay.XXXXXX)"
cat > "$harness_dir/Cargo.toml" <<EOF
[package]
name = "soxq-research-runner"
version = "0.1.0"
edition = "2021"
[dependencies]
kis-auto-trade = { path = "$PWD/src-tauri" }
serde_json = "=1.0.149"
[[bin]]
name = "soxq-research-runner"
path = "$research_dir/runner.rs"
EOF
cp Cargo.lock "$harness_dir/Cargo.lock"
cargo run --offline --manifest-path "$harness_dir/Cargo.toml" -- "$research_dir/replay-input.json" "$harness_dir/repeat.json"
SOXQ_COST_BPS=20 cargo run --offline --manifest-path "$harness_dir/Cargo.toml" -- "$research_dir/replay-input.json" "$harness_dir/cost20.json"
~~~

## 검증

- PASS: research runner formatting.
- PASS: 26 baseline strategy runs + LTH specialized + 2 benchmarks.
- PASS: cost stress each20bp.
- PASS: repeat baseline.
- PASS: identical repeat results excluding generatedAt.
- PASS: dataset hashes,13 strategy coverage,equal primary dates.
- PASS: 13 strategy chart switches + mobile layout.
- PASS: cargo check workspace.
- PASS: existing strategy_preview tests.
- PASS: existing simulation tests.
- PASS: TypeScript noEmit.

반복 실행은 generatedAt을 제외한 입력·신호·체결·수익 곡선·hash 일치를 확인했다. 동일 시스템 시각 범위 내 재실행이므로 generic LTH의 시각 의존성 해소를 입증하지는 않는다. 별도 읽기 전용 감사로 계산 의미와 문서 drift를 확인했으며, 운영 전략 코드는 수정하지 않았다.

## 후속 검증 부록: v3~v7

아래는 2026-10-03~05의 동일 입력 검증을 버전순으로 이관한 기록이며 문서 통합 시점의 재실행 결과가 아니다. 원래 ID 순서와 엔진 순서는 다르다: **v3(P1-02)→v4(P1-01)→v5(P1-03)→v6(P1-04)→v7(P1-05)**. 본문의 초기v2 결과·차트는 당시 관측으로 유지한다. 현재 남은 일은 [TODO](../../todo.md)를 따른다.

공통 입력은 [동결 replay 입력](replay-input.json)과 같은 폴더의 원본 CSV/metadata, 평가2026-07-06~10-02의64봉·이전252봉 또는 사전자료 없는 진단이다. 초기자본1천만원·FX1450·10주·세금0·수수료와 슬리피지 **각각**10/20bp 및 당시 guard를 유지했다. 배당/변동FX/기말 강제청산·실제 장중 체결 가능성은 검증하지 않았다. `ui_three_month_defaults`는 사전자료 없는 옛 runner 모드명이며 현재 UI provider 조회의 재현이라는 뜻이 아니다.

<a id="validation-p1-02"></a>
### P1-02 — v3 고가·종가 warmup 분리 (2026-10-03)

**당시 완료·main, 위험 보통.** 공통 warmup은 OHLC 훅을1회 호출하고 기본 initializer에는 종가,52주 신고가 override에는 고가를 전달한다. 원래 마지막 봉 제외·200/252 기준·종목 독립성·기존 보유/평균가와 LTH/강한종가/변동범위의 전용 경계를 보존했다. 엔진은 `strategy-replay-v3`; IPC/serde/TS/저장 형식은 무변경이다.

high20000/close10000의 평균회귀,252봉 상승종가의 MA10151.5/10241.5/10249, 신고가15000 Hold/20001 Buy, 일봉historical1회·intraday-only0회·기존 보유 불변과 미래봉 이전신호 불변의 신규6회귀를 확인했다. 동일 입력52개 전후 비교(26실행×두 비용)와 기본비용 반복은 생성시각을 제외하고 일치했으며 Mean/Trend 외 모든 신호/backtest·LTH 일봉진단·두 benchmark는 그대로였다.

| 전략/각 비용 | 수정 전→v3 수익률 | MDD 전→후 | 완료 거래 전→후 |
|---|---|---|---|
|평균회귀10bp|1.4402→2.2083%|1.6683→1.6558%|2→1|
|평균회귀20bp|1.3333→2.1561%|1.6696→1.6562%|2→1|
|추세필터10bp|0.8722→0.8722%|1.6917→1.6917%|0→0|
|추세필터20bp|0.8440→0.8440%|1.6921→1.6921%|0→0|

평균회귀의 잘못된7/7매수·7/17손절이 제거되고7/29→9/21청산이 남았다. 추세필터 첫매수8/7은 같지만 장기MA는7456→7371센트로 바뀌었다. 사전자료 없는 Mean/Trend는 변함없다. 작은 완료 표본을 우수성으로 해석하지 않으며 사전준비·날짜이벤트·next-open·generic LTH 시스템시각 문제는 당시 미해결이었다.

당시 lib142(신규6)·check/Clippy/TS·수정Rust6파일 rustfmt·Chromium mock18·map/graph PASS였다. **전체 cargo fmt는 기존5파일**(`rate_limit.rs`, `commands/toss.rs`, `server/profiles.rs`, `storage/database_projection.rs`, `trade_store.rs`) 차이로 NOT PASS였고 전체 재서식은 하지 않았다. compiler/Clippy0과 기존 Graphify4JSON zero-node·NO_COLOR/FORCE_COLOR 안내는 별개다. DB 환경 없는 contract의 내부skip은 실제 DB 성공이 아니며 외부 계좌/API·실전/모의 주문은 미실행이다.

[수정 전 결과](results.json) · [전후 비교 결과](p1-02-results.json) · [4021e38 고정 상세 원문](https://github.com/CountJung/KISAutoTrade/blob/4021e38bc5caaf4751eacaf96c20dc54dcf0f708/docs/research/soxq-3m-20261003/p1-02-validation.md). 당시 재현은 [runner](runner.rs)를 최초 보고서 명령으로 실행해 새 결과에 쓰고 대상 신호/backtest/inputHash를 대조했다. 주문/guard/risk/provider 변경은 rollback 범위가 아니다.

<a id="validation-p1-01"></a>
### P1-01 — v4 사전자료·준비 상태 (2026-10-03)

`strategy-replay-v4`는6전략의 과거 가격/MA·RSI를 과거tick/가상 포지션 없이 복원하고 warmupCount 생략0·명시legacy prefix/별도history의 신호/hash 동등성을 유지했다. 처리 후 readiness와 처리 전 `can_evaluate_next_tick`을 구분하며 seed-only 마지막봉은 readyAtEnd여도 notEvaluable, 미지원은null이다. live 신고가는 완료 불명 최신1봉 제외(252입력→251부족/253→252준비), generic stricthistory는 마지막봉까지 포함한다. stale기준 정리·보유손절 우선과500봉/OHLC/날짜granularity·중복/겹침 검사를 보존했다.

아래는 history252/64평가봉의 당시 결과다. 지원12전략은 시작준비 완료·평가64이며 LTH는 미지원/notEvaluable이다.

| 전략 | 각10bp / 각20bp 수익률 | 완료 거래 |
|---|---|---:|
|ma_cross|−1.1197 / −1.2549%|2|
|rsi|2.0224 / 1.9147%|2|
|momentum|−1.2637 / −1.4574%|3|
|deviation|1.3756 / 1.2649%|2|
|fifty_two_week_high|0 / 0%·conditionsNotMet|0|
|consecutive_move|−0.6445 / −0.7816%|2|
|failed_breakout|−0.0005 / −0.0851%|1|
|strong_close|0 / 0%·conditionsNotMet|0|
|volatility_expansion|−0.0300 / −0.0600%|0|
|mean_reversion|2.2083 / 2.1561%|1|
|trend_filter|0.8722 / 0.8440%|0|
|price_condition|2.2408 / 0.9777%|12|
|leveraged_trend_hold|unsupported/notEvaluable·성과판정 아님|0|

모멘텀 v3−0.4868→v4−1.2637%, 이격도2.4987→1.3756%는 누락 초기화 교정에 따른 진단이며 최적화가 아니다. MA의 raw 무보유매도 차단0→1, Mean/Trend는v3와 동일했다. cold 모드의 신고가/강한종가/변동성/추세는 notEvaluable이며 무거래0%와 구분한다. UI는 원래 평가봉을 우선 보존하고 당시 KIS1응답/Toss최대200봉의 준비부족을 표시했으며200/252 기준을 줄이지 않았다. provider 페이지와 날짜/LTH시각·next-open/ATR는 당시 잔여였다.

generatedAt 제외 지원24결과(12전략×2모드)의 신호·준비/metadata·backtest 반복 일치, 평가기간/64봉·history252/cold0과 혼합날짜/비정렬기간 회귀를 확인했다. 당시 lib167·전체target check/allfeaturesClippy/rustfmt·TS/FSD/lock/web build PASS이며 **비결정적 generic LTH는 반복 일치 주장에서 제외**했다.

[기본 결과](p1-01-results.json) · [20bp 결과](p1-01-cost20-results.json) · [4021e38 고정 상세 원문](https://github.com/CountJung/KISAutoTrade/blob/4021e38bc5caaf4751eacaf96c20dc54dcf0f708/docs/research/soxq-3m-20261003/p1-01-validation.md). 기본결과 SHA256=`8a49774fcfe58b201e037c2cec0ed2576546b36c14199fccb474da2c222946bf`.

<a id="validation-p1-03"></a>
### P1-03 — v5 날짜 시작·완료봉과 보유 피드백 (2026-10-03)

`strategy-replay-v5`는 날짜 시작에 시가만 공개하고 reset하지 않는다. 종가 시점의 완성OHLC→기존 비용/risk 체결·차단→실수량/평균가 피드백→완료봉 적재 순서다. 강한종가는 이전 완료일 조건, 변동성은 현재범위와 이전N완료일 평균을 비교하며 flat-day0도 일수에 포함한다. 차단매수의 낙관적 내부 보유를 방지하고 새날 당일고저만 정리한다. 두 전략은 D/1d만 지원하고 분/주/월은 `UNSUPPORTED_REPLAY_INTERVAL`; cadence=`dailyCloseWithDayBoundary`다. live polling/전용 LTH 경계는 바꾸지 않았다.

| 전략 | v4→v5 각10bp 수익률 | v5 각20bp / MDD10bp | 완료거래 / 체결주문 / 첫매수 |
|---|---|---|---|
|강한종가|0→1.7179%|1.5871% / 2.0740%|2 / 5 / 2026-07-07|
|변동성확장|−0.0300→0.7807%|0.7523% / 0.2531%|0 / 1 / 2026-09-21|

강한종가의 완료2회는 모두 손실이고 두 전략 모두 기말10주를 보유한다. 양의 평가수익을 완료 승률/우수성으로 판정하지 않는다. cold 강한종가는7/6봉 처리 후 준비·실제평가63/처리후미준비0, 변동성10일은7/17준비·평가54/미준비9다. history252에서는 둘 다64평가이며 cold와 같은 신호/체결이어도 준비 집계는 다르다. 당시 체결은 해당일 종가 근사로 next-open·장중손절/가격경로를 재현하지 않았다.

26generic 실행의 기간/64봉, generatedAt 제외 지원24의 반복 일치, 변경두전략/LTH 외20실행의v4호환과 미래봉·차단피드백·bounded범위/평가중준비·비일봉 거부를 검증했다. 당시 lib180/UI22와target check/Clippy/rustfmt·TS/FSD/lock/build PASS, 독립 리뷰에서 미수정 결함은 없었다. generic LTH는 여전히 반복 일치 주장에서 제외했다.

[기본 결과](p1-03-results.json) SHA256=`87790f4d2c1adf4b183b877cdc5d1237e065421840b73416a542799af68f4e3e` · [20bp 결과](p1-03-cost20-results.json) SHA256=`5d6255db273b33760282311817a39a6104ccf02791e1e1e9456863b221a06c07` · [4021e38 고정 상세 원문](https://github.com/CountJung/KISAutoTrade/blob/4021e38bc5caaf4751eacaf96c20dc54dcf0f708/docs/research/soxq-3m-20261003/p1-03-validation.md).

<a id="validation-p1-04"></a>
### P1-04 — v6 generic LTH 명시 거부 (2026-10-03)

`strategy-replay-v6`는 live의 Local::now() 기반 LTH를 일반 과거 replay로 실행하지 않도록 `starts_with("leveraged_trend_hold")`의 기본/접미ID를 factory·정규화·params/warmup 이전에 `UNSUPPORTED_GENERIC_REPLAY`로 거부한다. 두 대안 중 generic거부를 선택한 것이며 입력봉 오프라인 계약을 Toss/provider 네트워크 조회로 자동 전환하거나 live에 역사시계를 주입하지 않았다. 전용 `preview_signals_with_execution`은 입력 `timed.time`, 기존 profile/account/interval을 유지하고 generatedAt은 hash에서 제외한다.

지원24결과의 candles/signals/preparation/backtest는v5와 같고2generic LTH는 `rejectedRuns` 구조화 오류로만 남긴다. 0수익/무신호 정상 결과로 세지 않는다. 다른 오류는 runner를 실패시킨다. Asia/Seoul의 `2026-10-03T23:34:39.865022+09:00`와 America/New_York의 `2026-10-03T10:37:33.494989-04:00` 실행은 UTC로도 다른 분이며 generatedAt만 재귀 제외한 **전체 JSON**(24정상 hash·2거부·전용LTH·benchmark·입력)이 같았다. v5까지의 generic LTH 재현성 제외 예외는 제거됐다.

단위42요청(3ID×7간격×history2)의 조기거부, production 통합3개의09:04매수→15:25청산·2체결/완료1·세션밖/blackout·미래봉 이전신호/equity 불변을 확인했다. KORU 이름/국내세션·합성가격은 실제 시장 자료가 아니다. 당시 workspace187(lib181+통합3+doc3)/UI22·타입/구조/lock/build/Clippy/fmt/map/graph PASS, 코드경고0이었다. live 주문/전용 replay 로직은 그대로이고 일봉 합성관측/지표혼합·KST/DST·실제분봉은P1-05에 남았다. 실전/모의 주문·자동매매는 미실행이다.

[전체 결과](p1-04-results.json) SHA256=`5358b95ff0710fa725415a42004611b0bc76ffa720614a0c20aa5dd98420a74b` · [4021e38 고정 상세 원문](https://github.com/CountJung/KISAutoTrade/blob/4021e38bc5caaf4751eacaf96c20dc54dcf0f708/docs/research/soxq-3m-20261003/p1-04-validation.md). CSV의CRLF/metadata hash를 보존했고 반복/check 산출물과 harness lock은 ignored cache다.

<a id="validation-p1-05"></a>
### P1-05 — v7 일봉 진단·분봉 지표 분리 (2026-10-05)

**구현 검증 완료, 실제3개월 분봉 재평가는 미완료다.** `strategy-replay-v7`/전용`leveraged-trend-hold-v3`는 일봉시가를15분뒤 진입으로 해석하던 합성 관측을 제거했다. 일봉은 `dailyDiagnostic/notEvaluable`, 분봉은 `intradaySample/sampleOnly`(유효0이면notEvaluable)이며 과거 assessment없는 LTH 성과표·새A/B저장을 숨긴다. 최대200봉 표본을3개월 성과로 표시하지 않는다.

일봉 context와 분봉 EMA/RSI/ADX·반동은 별도 bounded buffer다. 일봉 재초기화는 분봉·볼린저·실보유/high-water/보유관측을 보존하고 분봉 snapshot은 교체한다. 일봉만으로 분봉준비가 되지 않는다. warmupCount0/dailyContextBars·unverified시간대/세션·lookAheadSafe=false와 generic명시거부를 유지하며 응답/hash의 LTH버전은 같다.

동일64평가/252history에서 지원24결과는v6와 candles/signals/preparation/backtest가 같고 버전/hash만 바뀌었다. generic거부2/benchmark도 같다. 전용 일봉은64종가 equity trace·장중0/context252·합성세션/전략·체결실행0이다. 무거래cash0%를 LTH 투자성과로 해석하지 않는다. `2026-10-05T23:48:35.344877+09:00`(Asia/Seoul)와 `2026-10-05T10:50:07.064736-04:00`(America/New_York)의 generatedAt 제외 전체JSON도 일치했다.

당시 Rust196(lib190+production3+doc3), UI26(기존22+assessment4)와타입/build/FSD/lock/check/Clippy/fmt/map/graph·독립리뷰 PASS, 코드경고0이었다. 유효분봉0·상태보존·snapshot교체/초기화·buffer상한·실패주문피드백 회귀를 포함한다. 09:04→15:25 분봉 fixture는 실제 SOXQ나3개월 성과가 아니며 gap_pct/반동준비전진입·보유청산/피드백은 보존했다. live는 KIS일봉 warmup만으로 minute지표를 준비하지 않고 후속 실제 관측을 기다린다.

**남은 조건:**2026-07-06~10-02 실제 SOXQ1분봉 원본/metadata·누락구간, 원본 봉시작/종료시각·feed/조정·거래소시각/세션/DST·coverage 검증과 전체기간 replay가 필요하다. 일봉 보간·합성 fixture로 충족시키지 않는다. 당시 무료·무인증 전체기간 경로를 확인하지 못했을 뿐 공급원 전체가 불가능하다는 판단은 아니다. 10/05 기록의 [Alpaca historical bars](https://docs.alpaca.markets/us/reference/stockbarsingle-1)·[FAQ](https://docs.alpaca.markets/us/docs/market-data-faq)는 인증/15분이전 historical SIP 설명을, [Alpha Vantage](https://www.alphavantage.co/documentation/)는 premium endpoint 설명을 근거로 보존한다. 가입/결제·인증조회는 하지 않았으며 현재 제공조건을 새로 확인한 기록은 아니다. 실제/모의 주문·자동매매도 미실행이다.

[전체 결과](p1-05-results.json) SHA256=`8ede4d705810f32906827bbcabc93960a4e4bbfe7132624f9bd2b64ede60fd03` · CSV SHA256=`10ab23ff4af9bec44d4fe19da33f7603e01857610f6ce8ba60c8d87c33b02dae` · [4021e38 고정 상세 원문](https://github.com/CountJung/KISAutoTrade/blob/4021e38bc5caaf4751eacaf96c20dc54dcf0f708/docs/research/soxq-3m-20261003/p1-05-validation.md).

당시 공통 재현은 [runner.rs](runner.rs)와 ignored `.cache/research/soxq-replay/Cargo.toml`/lock, `target/`을 사용했다. v4 원문의 manifest는 production path dependency와 serde_json1.0.149를 고정하며 새 환경에서 lock생성 후 `--locked`로 실행한다. 기본/20bp·TZ반복의 명령·환경/rollback·전체 수치·위임 범위는 각 고정 원문에 보존한다. 이 통합에서 원본/runner/JSON을 재실행하거나 raw bytes/hash를 수정하지 않았다.
