# P2-15: 실현·잔여 손익과 비용·배당·환율 비교

2026-10-18. 공개된 일봉 가격을 보관한 [원본 연구](../strategy-review-20261018/report.md)의 동결 입력으로 생산 replay v11을 실행했다. 계좌·주문·브로커 인증 호출은 없다. FX·배당·방향별/최소 수수료 변경값은 **합성 what-if 입력**이며 실제 QQQ 배당·환율·브로커 요율을 뜻하지 않는다.

## 결과와 해석

직전1192673 생산 라이브러리의 같은 기본 입력1,224회와 새 엔진의 전체 report·신호가 일치했다. 비교에서는 새 회계/비교·세부 비용 metadata만 제외했다. 이 반복은 호환 검증이며 독립적인 시장 표본이나 전략 성과 증거가 아니다.

QQQ2026Q3은 평가2026-07-01~09-30의64봉, 엄격히 이전252봉을 사용한다. 자본1천만원·요청1주·초기FX1450·세금0·수수료/슬리피지 각각10 또는20bp, 원래 전략 파라미터를 고정했다. 두 전략×두 비용×sameClose/nextOpen×다섯 가정40회와 같은 엔진의 단순보유 비교를 보관했다. 모두 같은 단일분기의 자료를 재사용하므로40개의 독립 시장 관측으로 합산하지 않는다. 이전 연구의 부분분기·겹침창과도 독립이 아니다.

기본 가정의 nextOpen 결과:

| 전략 | 각각 비용bp | 자산 수익률 | 실현순손익KRW | 잔여순평가KRW | 기말 가정청산 수익률 | 평균 초기자본 투자비중 | 단순보유 수익률 / 평균투자비중 |
|---|---:|---:|---:|---:|---:|---:|---:|
| 강한종가 |10|+0.1049%|−81,816.58|+92,306.72|+0.0835%|9.5756%|+0.1847% /10.1969%|
| 강한종가 |20|+0.0039%|−89,958.82|+90,346.98|−0.0390%|9.5756%|+0.1636% /10.1969%|
| 평균회귀 |10|−0.4372%|−43,723.11|0|−0.4372%|1.2474%|+0.1847% /10.1969%|
| 평균회귀 |20|−0.4772%|−47,716.29|0|−0.4772%|1.2474%|+0.1636% /10.1969%|

강한종가는 완료 청산2회 모두 손실이며 마지막1주의 평가이익이 이를 상쇄한다. 비용20bp에서 자산 수익률은 양수여도 잔여분 청산비용을 가정하면 음수다. 평균회귀는7/30에 청산되어8/3의 가상 배당이나 이후 FX가 전략 손익을 바꾸지 않는다. 같은 요청1주의 단순보유와 전략은 보유기간·실제 투자비중이 달라 수익률 차이를 알파로 해석할 수 없다.

비용10bp/nextOpen 강한종가의 시나리오:

| 입력 가정 | 자산 수익률 | 세후 배당KRW | 잔여순평가KRW | 기말 가정청산 수익률 |
|---|---:|---:|---:|---:|
| 고정FX·배당없음 |+0.1049%|0|92,306.72|+0.0835%|
| 가상FX만 |+1.2146%|0|203,272.22|+1.1909%|
| 가상배당만 |+0.1172%|1,232.50|92,306.72|+0.0958%|
| 가상FX+배당 |+1.2256%|1,105.00|203,272.22|+1.2019%|
| 가상수수료표 |+0.0829%|0|91,836.10|+0.0577%|

가상FX는8/3부터1300,9/1부터1600이다. 가상배당은8/3에1주당USD1·원천징수1500bp를 봉 직전 보유분에 즉시 지급한다. 가상수수료표는 매수8bp/매도12bp·주문당 최소USD1이며 실제 요율이 아니다. 같은 날 환율 가정 때문에 배당 원화액이 달라진다. 실제 가격의 배당 조정 여부는 미확인으로 유지하므로 현금배당 중복계상 경고가 붙는다. 이 표를 실제 배당 포함 총수익으로 사용하지 않는다.

## 회계와 비교의 정확한 의미

- 자산 증감=실현 순손익+잔여 순평가손익+세후 배당. 잔여 평가에는 아직 배분되지 않은 매수fee도 차감한다. 가격 변화는 진입FX로, FX 변화는 최종native금액으로 분해한다. 슬리피지는 체결가에 이미 포함된다.
- native 진입평균과 역사적 KRW 원가를 분리한다. FX 변화가 전략의 진입가격/보호기준을 다시 쓰지 않는다. 유휴 현금은 KRW이며 USD 현금 잔고의 재평가·배당락/실제 지급일·환전료를 재현하지 않는다.
- 가정청산은 마지막 관측종가의 슬리피지·매도fee·tax를 추가 차감한 보고값이다. 거래 수·승률·리스크·보유 상태에는 적용하지 않는다.
- 평균/최대 투자비중은 각 관측봉 종가의 평가금액을 초기자본 또는 당시자산으로 나눈 값이다. 보유봉비율·누락기간의 시간가중·장중최대 노출과 다르다.
- 단순보유는 첫 평가종가의 같은 요청수량 매수지시와 같은 비용/FX/배당·실행모델·ATR/리스크 조건이다. 보호 overlay/전략청산은 없다. 실제체결수량·미체결·노출차이를 같이 표시한다.
- 동일 체결경로 비용복원은 실제 수량·시점·노출을 고정하고 발생 당시 기록된 KRW fee+tax+slippage+배당세만 자산에 더한 사후 산술참조다. 원가/FX귀속을 재계산하거나 무비용 전략을 재실행한 결과가 아니다. 비용변경으로 guard·청산·후속 경로가 바뀌는 효과는 별도 생산 replay로 계산한다.
- 완료 청산20건 미만을 경고하며 부분청산도 각1건으로 센다. 이번1~2회의 청산으로 승률·profit factor·최적 파라미터를 일반화하지 않는다. 가격 coverage·기업행위·실제 장중체결·LTH 시간대/세션/3개월 자료는 이번 검증에 포함하지 않는다.

## 검증·독립 감사

[runner.rs](runner.rs)는 실제 `preview_strategy_from_candles`를 호출하며 전략 신호를 별도로 구현하지 않는다.1224/40/120의 기대 카운트가 다르면 실패한다. 전체40결과에서 자산증감·가격/FX귀속·가정청산의 항등식120개가 통과했다.

[validate.py](validate.py)는 기록된 생산 fill을 원본 봉과 합쳐 현금·부분원가·매수fee·현재FX/배당권리·매봉 equity·투자비중·비용환급·terminal을 독립 재계산했다. **4,540개 대사**가 통과했다. 표시 반올림 오차허용은 돈0.031원/비중0.00011%p다. 이 스크립트는 신호나 전략/guard를 재구현하지 않는다. source와 artifact SHA-256은 [validation.json](validation.json)에 보관했다.

별도 역할의 코드/경제 감사에서 최소수수료가 일반 이익매도 guard에 빠진 문제를 발견했다. 국내1주10000→10100, 양방향최소1000이면 순손익−1900원이므로 일반 익절을 차단한다. 실제 청산 전망을 guard와 정산이 공유하도록 수정하고 부분매도·해외FX·손절/보호/native손실 예외8개를 추가했다. 기본minimum0 경로는1224회 호환으로 확인했다.

최종 독립 주식 분석 감사는 validate.py를 재사용하지 않고 Decimal 산술로40개 결과의 관계240개와 동결 QQQ원본64봉에서 단순보유40개의 항목400개를 재계산했다. 모두 일치했고 최대 금액 반올림 차이는0.01원이었다. 예를 들어 nextOpen10bp 단순보유는7/2open725.58에 따른 원가1,053,143.091원+fee1,053.143091원, 마지막평가1,072,666.5원으로 기말10,018,470.265909원(표시10,018,470.27원)이 된다. 같은 비교의 평균초기자본비중10.19692109375%도 표시10.1969%와 일치한다. 독립 문서 감사도 연구표 숫자·artifact SHA·최소수수료 수정/손절 예외·terminal 비변경을 확인했다. 발견한 TODO 상단의 오래된 다음과제 안내는 수정했다.

## 재현

저장소 루트에서 동일 lockfile·오프라인 Cargo 환경을 사용한다. 생산 코드·원본 입력/기존 연구 결과를 수정하지 않는다. 산출물과 harness는 `.cache/` 및 `target/`에 둔다.

1. `git archive 1192673`을 `.cache/pnl-head-baseline/`에 풀고 `.cache/pnl-head-baseline-harness/Cargo.toml`을 아래 baseline manifest로 만든다. 루트 Cargo.lock을 해당 harness에 복사한다.
2. `cargo run --offline --manifest-path .cache/pnl-head-baseline-harness/Cargo.toml --target-dir target -- "$PWD"`로 직전 생산 라이브러리 기준1224개를 생성한다.
3. `.cache/replay-accounting-harness/Cargo.toml`을 아래 current manifest로 만들고 Cargo.lock을 복사한다.
4. `cargo run --offline --manifest-path .cache/replay-accounting-harness/Cargo.toml --target-dir target -- "$PWD"`로 호환대조와40개 시나리오를 생성한다.
5. `python3 docs/research/replay-accounting-20261018/validate.py`로 독립 기록fill 대사와 hash를 생성한다.

baseline manifest:

```toml
[package]
name = "kis-pnl-head-baseline"
version = "0.1.0"
edition = "2021"
[workspace]
[dependencies]
kis-auto-trade = { path = "../pnl-head-baseline/src-tauri" }
serde_json = "1"
[[bin]]
name = "baseline"
path = "../../docs/research/replay-accounting-20261018/baseline_runner.rs"
```

current manifest:

```toml
[package]
name = "kis-replay-accounting-research"
version = "0.1.0"
edition = "2021"
[workspace]
[dependencies]
kis-auto-trade = { path = "../../src-tauri" }
serde_json = "1"
[[bin]]
name = "research"
path = "../../docs/research/replay-accounting-20261018/runner.rs"
```

기본sameClose의 전체 report 비교에서 `accounting`/`comparison`, 신규가정8개(costSource/costNote/buyFeeBps/sellFeeBps/minimumFeeNative/fxScenario/dividendScenario/dividendPriceBasis), 신규trade5개(executionReferencePrice/exchangeRateKrw/feeKrw/taxKrw/slippageKrw)만 제거한다. 기존summary·trade·equity·execution metadata와 signals는 모두 대조한다. 원본 동결 input SHA-256은 `d48ce5b59b7876c0a1457121e877fdc1117a77f990b4ec07ce649c92fe153317`이며 기존 연구 결과를 덮어쓰지 않는다.
