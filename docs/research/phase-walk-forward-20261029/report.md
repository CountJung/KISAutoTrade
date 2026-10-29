# P2-16 회고 walk-forward 절차와 구간 진단 검증

2026-10-29. 결과를 보기 전에 [설계](design.json)를 고정하고 현재 생산 Rust v23 API로 12개 고정 전략의 context/evaluation을 별도 초기화해 실행했다. **이미 여러 연구에서 관측한 가격으로 수행한 회고 절차 검증이며 새 OOS·독립 표본·실거래 효능 검증이 아니다.** 학습 모델·최적 파라미터·우승 전략을 선택하지 않았고 전략·주문·리스크 정책은 이 연구에서 수정하지 않았다. 실제 API·계좌·키·주문은 사용하지 않았다.

## 결과 전 선언과 정보 경계

- 선언 시각: 2026-10-29T06:44:43.969239+00:00. 설계 SHA-256은 `b28db1f2d9579c5155fde426778ef720b8e25e0ccded343e7a62bbc212224994`이며 실행 요약에 선언문 전체를 보존해 이후 변경 여부를 검사한다.
- 원본: [동결 입력](../strategy-review-20261018/input.json), SHA-256 `d48ce5b59b7876c0a1457121e877fdc1117a77f990b4ec07ce649c92fe153317`. 출처·수집 시각·미국 timezone·원본/정규화 hash·OHLC 센트 반올림·제외 이력은 [원래 연구](../strategy-review-20261018/report.md)를 따른다. 새로운 거래소 달력·누락·기업행위·가격조정 완전성 인증은 하지 않았다.
- SOXQ/SPY/QQQ, 2024Q1~2026Q3의 완전11개 평가분기. 각 평가분기 직전4개 완전분기가 context다. 원본의 실제 관측수는 context249~252봉, 평가60~64봉이며 매 실행에 엄격히 이전252개 완료 일봉을 별도 warmup으로 전달했다. 관측 간격을 바꾸거나 누락봉을 합성하지 않는다.
- 기존 generic12전략의 명시된 기본 설정을 모두 고정했다. context는 과거 성과를 기술하는 별도 replay이며 후보 순위·파라미터 선택에 사용하지 않는다. LTH는 generic 일봉이 평가불가이므로 포함하지 않았고 실제 장중 자료 gate를 유지한다.
- 자본1천만원, 고정FX1450, nextOpen, 포지션상한30%, ATR 수량산정 비활성, 연구 overlay/세금/배당0. fee와 slippage는 각각10 또는20bp이며 매수/매도 양쪽에 적용한다. 개략 왕복40/80bp이고 왕복10/20bp라는 뜻이 아니다. 생산 replay의 기존 guard/risk를 유지하며 live 계좌의 주문횟수·체결 지연을 검증한 결과로 해석하지 않는다.
- 수량은 context 첫 평가일 직전 완료 종가로 `floor(0.15×10000000/(USD종가×1450×1.002))`를 정했다. 해당 fold의 context/evaluation과 두 비용에서 같은 수량을 고정한다. 평가분기 시작 직전 종가로 다시 산정하지 않는다. PC 기준가·내부수량과 native익절5%/손절3%도 같은 사전 정보로 고정한다. fold 사이 기준가는 계획대로 달라지며, 이를 이전 PC 연구의 3년 단일 고정 기준가 실험과 혼합하지 않는다.

context/evaluation은 각각 새 전략·현금·보유·pending·guard·risk 계좌다. context의 실제 보유나 보호 metadata를 evaluation으로 이전하지 않고, evaluation은 그 시점까지의 실제252봉으로 지표만 준비한다. 이전 evaluation 가격이 이후 context/warmup에 포함되는 것은 해당 시점의 과거 정보이며, 결과를 보고 선택 규칙을 바꾸지 않았다. 계좌 내부500봉 공급 분할은 상태를 보존하지만 context/evaluation 경계는 의도적으로 reset한다.

마지막 nextOpen 후보는 다음 관측이 없으면 미체결이다. 기말 잔량은 평가손익으로 남기고 종료청산은 보고 전용 가정이며 실제 매도·승률·위험 상태를 만들지 않는다. 개별 분기 계좌의 수익률을 더하거나 복리화해 연속 운용 성과로 표시하지 않는다. 계좌 내부70/30은 같은 장부의 앞/뒤 관측 구간이며 이 연구의 별도 context/evaluation과 다른 경계다.

## 실행과 독립 검산

[runner](runner.rs)는 생산 `preview_strategy_continuous_from_candles`를 사용했다. 12전략×3ETF×11fold×2비용×context/evaluation2계좌 = **1,584회**(context792/evaluation792)다. 결과는 [전체 원장](walk-forward-results.jsonl.gz), 모든 계좌의 비용·순손익·잔여·MDD·차단사유·회전·노출·비교는 [요약 CSV](summary.csv), 기술 분포는 [집계](aggregate.json)에 보존했다.

[검증기](validate.py)는 전략 의사결정을 재구현하지 않는다. 원본 시가와 기록된 실제 수량으로 nextOpen fill·슬리피지·수수료를 대조하고, 부분매도의 원가·매수수수료를 비례 배분해 현금·실현/잔여·매봉 equity·기말 가상청산·MDD·노출·회전을 독립 산술로 검증한다. 후속 봉에서만 체결되는 시각 경계와 수량의 비음수성도 확인한다.

새 phase diagnostics는 `sameLedgerObservationSplit` v1이다. 관측/보유봉수, 경계 전후 실제수량·pending, 실현손익·시작/종료 잔여손익·순배당·자산증감을 기록체결로 대조했다. 시작 snapshot은 첫 봉의 FX/배당/pending 시가체결 이전, 종료는 마지막 close 이후다. 검산 항등식은 다음과 같다.

`구간 자산증감 = 구간 실현순손익 + 종료 잔여순평가손익 − 시작 잔여순평가손익 + 구간 순배당`

워크포워드와 PC호환 원장을 합친 **3,324개 phase**, 기록산술 **358,749건**이 통과했다. 최대 금액차이는 약0.01원이며 표시 반올림을 허용한0.031원 이내다. 이 검사 건수는 시장 표본 수가 아니다. 이번 실제 자료 실험은 FX고정·배당0이라 해당 phase 항등의 비제로 FX/배당 동작까지 이 자료로 입증하지 않는다. 그 경계는 공통 backend의 별도 합성 fixture 검증 범위다.

### 이전 PC 경제 경로 호환

[2회차 PC 설계·원장](../price-condition-contracts-20261029/report.md)의 동일 기준가·수량·비용·752/252봉으로78원장과 대체분할6회를 추가 재현했다. [호환 원장](pc-compatibility-results.jsonl.gz)은 기존 PC78개와 전체 `signals/preparation` 및 `backtest` 경제값이 일치한다. 제외 필드는 phase `diagnostics`, `overfitWarning`, `executionWarnings`뿐이며 기존 회계·체결·원시 phase 수치는 제외하지 않는다. 같은 v23의12분기/500+252 분할은 새 진단까지 전체 report·signals·preparation·inputHash가 같다.

`baselineCommit=00ea198a...`는 회차 시작 소스 포인터다. 이전 커밋을 새로 빌드한 baseline 비교가 아니며, 비교대상은2회차 보관 PC원장이다. 엔진v23에 따른 inputHash 변경과 진단/경고 교정을 경제 성과 변경으로 해석하지 않는다. [실행 기록](run-summary.json)과 [검증·source hash](validation.json)가 실제 실행 범위의 근거다.

## 평가분기의 기술적 분포

아래는 전략별·비용별33개 별도계좌(11fold×3ETF)의 분포이며 요청한12전략 순서대로 표시한다. 각 셀은10/20bp이고, 노출은 봉말 평가금액의 평균 초기자본 비중이다. 같은 비중·독립 알파·일반화된 순위가 아니다.

| 전략 | 순평가수익 중앙값 | 청산0인 창/33 | 평균 초기자본 비중 중앙값 | 회전율 중앙값 |
|---|---:|---:|---:|---:|
| ma_cross | -0.0503 / -0.1380% | 8 / 8 | 6.3885 / 6.5077% | 50.1184 / 49.0324% |
| rsi | +0.0000 / +0.0000% | 21 / 21 | 2.2784 / 2.2784% | 13.6947 / 13.7084% |
| momentum | +0.3851 / +0.3543% | 27 / 27 | 9.1899 / 9.1899% | 15.3819 / 15.3973% |
| deviation | +0.0000 / +0.0000% | 25 / 25 | 2.3845 / 2.3845% | 13.4303 / 13.4437% |
| fifty_two_week_high | +0.0000 / +0.0000% | 22 / 21 | 7.4020 / 7.0557% | 16.7140 / 18.0982% |
| consecutive_move | +0.2113 / +0.1608% | 2 / 2 | 9.7331 / 9.7508% | 66.7981 / 64.5611% |
| failed_breakout | -0.2202 / -0.2932% | 14 / 14 | 6.4269 / 6.4269% | 30.8530 / 30.8533% |
| strong_close | +0.6917 / +0.6738% | 14 / 14 | 15.1546 / 15.1546% | 43.9024 / 43.9032% |
| volatility_expansion | +0.0000 / +0.0000% | 27 / 27 | 0.3572 / 0.3572% | 9.6786 / 9.6882% |
| mean_reversion | +0.0000 / +0.0000% | 13 / 13 | 2.2533 / 2.2533% | 24.7582 / 24.7589% |
| trend_filter | +0.7233 / +0.6932% | 25 / 25 | 14.4501 / 14.4501% | 17.5895 / 17.6071% |
| price_condition | +0.0000 / +0.0000% | 30 / 30 | 0.0000 / 0.0000% | 0.0000 / 0.0000% |

비용별396평가계좌에서 청산0은228/227개, 기말 보유가 남은 계좌는202/201개다. 모두 완료청산20건 미만이다.20이라는 경고선은 기존 UI 해석 경계이지 검정력·신뢰구간을 보증하는 통계 기준이 아니다. 부분청산도 각1건으로 센다. 0%·청산0·경고 없음은 성과 우수나 신뢰 충분을 뜻하지 않는다.

같은 요청수량 buyHold가 실제0주인 평가는 각비용12행이며 모든12전략에 반복되는 한 종목·fold 조건이다. 이를12개 독립 실패나 유효한0% benchmark로 세지 않는다. 같은 요청수량과 실제 체결수량·보유시간·비중은 다르다. 가격경로 비용복원도 기록된 경로의 비용환급 산술이지 무비용 전략 재실행이 아니다.

비용10에서 양수였지만20에서0이하가 된 평가조건은2쌍이다. 비용 변경은 청산·guard·후속 노출을 바꿀 수 있어 고정 경로에 수수료만 더 뺀 실험으로 설명하지 않는다. 상세 MDD·실현/잔여·비용·차단·benchmark는 CSV에 남겼으며 중앙값만으로 후보를 채택하지 않았다.

context 창은 서로 겹치며 뒤 fold의 context에 앞 평가기간이 포함된다. 세 ETF·전략·비용쌍도 상관되어 있다. 792개 평가계좌나1,584개 전체 실행을 독립 표본수로 승격하지 않으며 context/evaluation의 길이가 다른 누적수익 비율로 과최적화를 판정하지 않는다. 이번 자료는 이미 관측한2026도 포함하므로 새로운 미관측 OOS가 아니다.

## 재현과 남은 경계

격리 `.cache/phase-walk-forward-harness/Cargo.toml`은 package `kis-phase-walk-forward`, 독립 `[workspace]`, 의존성 `kis-auto-trade={path="../../src-tauri"}`, `serde_json="1"`, bin path `../../docs/research/phase-walk-forward-20261029/runner.rs`를 사용한다. 루트 Cargo.lock을 복사해 offline으로 실행했다.

```sh
cargo run --offline --manifest-path .cache/phase-walk-forward-harness/Cargo.toml --target-dir target -- "$PWD"
python3 docs/research/phase-walk-forward-20261029/validate.py
```

연구 runner의 최초 컴파일·실행과 별도 harness의 `cargo clippy --offline --all-targets --all-features -- -D warnings`는 통과했고 경고0이었다. 검증기 재실행은 압축 원장을 읽으며 전략 실행을 다시 하지 않는다. 생산/프런트 전체 검사와 문서·지도·회차 완료 판정은 부모의 [순차 기록](../../tasks/immediate-backlog-20261029.md)이 정본이다. 연구 harness만으로 UI/실제 Tauri 런타임·provider coverage 검증을 주장하지 않는다.

이 연구는 [P2-16 완료 이력](../../completed-history.md#soxq-p2-16)의 별도 회고 walk-forward 절차와 phase 진단 검산 근거다. 실제 장중 cooldown·손절 효능, LTH 장중 coverage·시계, 미래 미관측 자료의 일반화나 실거래 안전성은 별도 조건이며 완료로 승격하지 않는다. 기본 전략이나 guard 완화·정책 승격을 제안하지 않는다.
