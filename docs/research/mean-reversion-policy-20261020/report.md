# P2-10 평균회귀: 실제 소유·추세/보유/중심선 단독 후보

상태: 생산 비교·기록 정산·최종 독립 감사 완료. 운용 기본값은 기존 upperBand와 선택 옵션0을 유지한다.

## 사전 선언과 재현

[design](design.json), [기준 실행기](baseline_runner.rs), [현재 생산 실행기](runner.rs), [기록 정산](validate.py)을 보관한다. 고정 입력은 [전수 연구 원본](../strategy-review-20261018/input.json), SHA-256 `d48ce5b59b7876c0a1457121e877fdc1117a77f990b4ec07ce649c92fe153317`다. broker/계좌 설정을 읽거나 변경하지 않았다.

archived474aa06(v15)와 currentv16 Rust production preview를 실제로 실행한다. Python은 기록 체결·현금·노출 산술을 검증하며 전략 신호를 다시 만들지 않는다. flat1,224개에서 비평균회귀1,122개 전체 report/signals 호환과 평균회귀102개 차이를 별도 측정한다. 후보 성과 비교는2023-01-03~2025-12-31의752봉, strict 이전252봉(2021-12-31~2022-12-30)을 사용한다. SOXQ/SPY/QQQ 최초 수량51/2/3,1천만원·고정FX1450·nextOpen·fee/slippage각10/20bp·tax0·배당/FX시나리오없음·overlay stop/takeprofit0과 기존guard/risk/maxPositionRatio0.3을 고정한다.

3년 분기12청크를500+252분할과 대조한다. 독립12분기 초기화 계좌는 각각 새1천만원이며 수익률을 합치거나 복리하지 않는다. 기존45분기·최근겹침창 통계와 합산하지 않는다.2026가격은 새 후보 성과에서 제외하지만 fixed compatibility와 원래 frozen QQQ 회귀에는 사용한다. 어느 쪽도 unseen OOS로 부르지 않는다.

| 후보 | 단독 변경 |
|---|---|
| legacy | archived 원본 |
| confirmedUpper | 실제 owned quantity/native avg만 피드백으로 갱신, 기존 upperBand/stop 유지 |
| middleBand | confirmedUpper+exit_mode middleBand, 같은 current-inclusive mean을 strict 상회하면 청산 |
| trend200 | confirmedUpper+현재가가 현재 제외 이전200관측 평균보다 높은 신규 진입만 허용 |
| maxHold63 | confirmedUpper+실제 체결 뒤 기록 보유63관측 청산 후보 |
| maxHold21 (사후 탐색) | Stage A 결과 확인 뒤 추가한 단일21관측 청산 후보 |

밴드는 현재 포함 최근20관측 모집단 분산(ddof0)·표준편차배수2, strict price<lower/price>upper다. 기존 손절은 ((nativeAvg−price)/nativeAvg*100)>=stop_loss_pct 공식이며 기본5를 유지한다.0도 원가 이하 손절이며 비활성이 아니다. stop→maxHold→선택band 순서다. stop/band는 기존 설정수량, maxHold은 소유 잔량 전량 후보이며 기존 guard/clamp/risk와 거절/부분체결/재시도를 유지한다. nextOpen과 비용·갭이 있어5% 장중 손실 한도가 아니다.

새 추세 context는 live 최초 관측에서 비우고 실제 약10초 관측만 모아 이전N을 평가한다. 일봉 warmup과 실시간 관측을 혼합하지 않는다. replay는 strict 이전 일봉을 seed할 수 있다. MeanReversion의 기존 분봉 history warmup no-op은 그대로여서1분 preview는 평가 관측을 모아야 밴드를 준비한다. 이는 검증된1분 사전자료 처리나 live 일봉/분봉 일치라는 주장이 아니다. reset은 기존 가격 버퍼를 유지하고 보유 보호를 정리하며 exact owner metadata로 기록 나이를 복원한다.

보유 나이는 off 중에도 실제 양수 소유 뒤 관측을 세고 sameqty/partial/warmup/reconfigure로 줄이지 않는다. 진입 origin이나 유효 exact owner metadata 없이 복원된 보유는 ageunknown, 활성 maxHold에서 다음 관측 보수적 후보를 낸다.63관측은 이번 일봉 연구에서63봉이며 live63일이 아니다.

기존 frozen QQQ 회귀는1주·2026Q3 252+64봉이다. sameClose10bp +0.6274%(9/21밴드),20bp−0.5659%(7/29손절), nextOpen10/20bp−0.4372/−0.4772%(7/29종가신호→7/30시가청산) 네 경로를 유지한다. 새연속 QQQ3주 성과·후보 표본과 합치지 않는다.

재현은 저장소 `.cache/`에474aa06 git archive와 Cargo harness를 만들고 baseline_runner는 archivedsrc-tauri, runner는 현재src-tauri path dependency에 연결한다. 실행기 argv는 저장소 절대 경로다. 먼저 baseline flat1224/연속6/분할6/분기72, 다음 current1122호환/102delta/연속24/분할24/분기288을 실행한다. `python3 docs/research/mean-reversion-policy-20261020/validate.py`는 원장을 gzip보관하고 평문을 ignoredcache로 옮긴다. 생성시각 때문에 재실행 gzipSHA는 달라질 수 있으며 입력/코드/기록 hash를 validation에 구분한다.

## 결과

Stage A의 원래5후보는2026-10-20 22:14:08 KST에 고정했다. 연속30·분할대조30·별도분기360을 실행한 결과63관측 제한이 발동하지 않았다. 이후22:30:35 KST에 maxHold21 하나를 고정해 추가했다.21은 대략 월간 관측이라는 휴리스틱이며 수익률 탐색으로 최적화한 값이 아니다. 이는 **결과를 본 뒤 추가한 탐색**이며 전체 설계를 사전등록으로 부르지 않는다. design의 declaredBeforeContinuousRuns/declaredBeforeCandidateRuns는 false, followup의 declaredBeforeItsCandidateRuns만 true다. 원래 설계 SHA `0988cf4a1b278aa51e76608d4a9fc63bf4914c1f443aa9f5af95925657ff3efd`와 원본 설계·Stage A 실행 요약을 design에 보존했다.

최종6후보는 **연속36·분할대조36·별도분기432, 원장468개**다. 기존30개 연속 대조군의 입력/경제 경로 hash는 모두 그대로였다. 분할 대조는 같은 경로를 다시 나눈 검증이며 독립 성과 표본으로 더하지 않는다. Stage A390과 최종468도 중복 원장을 포함하므로 합산하지 않는다.



기존 flat1,224개는 비평균회귀1,122개의 전체 report/signals가 동일하고, 평균회귀102개의 backtest/signals도 동일하다(engine 버전 등 metadata는 별도). 실제 소유 피드백으로 바꾼 confirmedUpper는 연속6+분기72의78개 backtest가 legacy와 동일했다. 기존 원장도 flatSell/heldBuy가0이므로 이번 자료에서 중복 신호 감소나 수익 개선이 있었다고 주장하지 않는다.

수수료·슬리피지 **각20bp**의 연속3년 결과다. 수익률·MDD·회전율·평균투자비중은%, 완료는 청산 수다. 회전율은 전체 매수·매도 금액/초기자본, 투자비중은 봉말 관측 평균이다. maxHold63은 upper와 전부 같아서 표에서 생략한다.

| 종목 | 후보 | 순평가수익 | MDD | 완료 | 회전율 | 평균투자비중 | 최저 완료손익(원) |
|---|---|---:|---:|---:|---:|---:|---:|
| SOXQ | confirmedUpper | 6.9134 | 5.3840 | 11 | 541.9022 | 5.9824 | -480,262.71 |
| SOXQ | middleBand | 0.1310 | 5.9287 | 13 | 630.9583 | 3.0531 | -480,262.71 |
| SOXQ | trend200 | -3.7217 | 5.7762 | 5 | 237.1558 | 2.7306 | -480,262.71 |
| SOXQ | maxHold21 | 4.8197 | 5.4453 | 12 | 579.1104 | 5.3679 | -480,262.71 |
| SPY | confirmedUpper | 0.0835 | 4.3719 | 11 | 355.7775 | 5.1258 | -223,838.62 |
| SPY | middleBand | 0.0609 | 2.3112 | 13 | 400.8582 | 1.9516 | -119,339.25 |
| SPY | trend200 | 1.6300 | 1.3946 | 7 | 247.8483 | 4.2735 | -119,339.25 |
| SPY | maxHold21 | -0.8185 | 4.4188 | 13 | 399.8716 | 4.0654 | -223,838.62 |
| QQQ | confirmedUpper | -1.8039 | 4.5823 | 6 | 246.3342 | 3.4840 | -137,764.64 |
| QQQ | middleBand | -2.5532 | 4.6151 | 8 | 304.0076 | 1.6061 | -137,764.64 |
| QQQ | trend200 | -0.6736 | 3.4698 | 5 | 209.1959 | 3.4110 | -137,764.64 |
| QQQ | maxHold21 | -1.5671 | 4.5623 | 8 | 304.9957 | 2.9362 | -137,764.64 |

평가수익에 잔여 보유가 포함된 차이를 별도로 대조한다. 가정청산은 기말 비용을 반영한 연구 값이며 실제 체결이 아니다.

| 종목 | 후보 | 실현손익(원) | 잔여평가손익(원) | 기말 가정청산 수익(%) |
|---|---|---:|---:|---:|
| SOXQ | confirmedUpper | 691,341.48 | 0.00 | 6.9134 |
| SOXQ | middleBand | 13,096.13 | 0.00 | 0.1310 |
| SOXQ | trend200 | -372,173.71 | 0.00 | -3.7217 |
| SOXQ | maxHold21 | 481,970.50 | 0.00 | 4.8197 |
| SPY | confirmedUpper | -45,281.98 | 53,633.29 | 0.0045 |
| SPY | middleBand | 6,087.07 | 0.00 | 0.0609 |
| SPY | trend200 | 109,365.26 | 53,633.29 | 1.5510 |
| SPY | maxHold21 | -81,851.96 | 0.00 | -0.8185 |
| QQQ | confirmedUpper | -244,984.28 | 64,594.59 | -1.9107 |
| QQQ | middleBand | -255,321.52 | 0.00 | -2.5532 |
| QQQ | trend200 | -131,949.83 | 64,594.59 | -0.7803 |
| QQQ | maxHold21 | -156,711.15 | 0.00 | -1.5671 |

각10bp 순평가수익도 함께 남긴다. 비용 변화는 차감액뿐 아니라 원가·손절·guard와 후속 체결 경로를 바꾼다.

| 후보 | SOXQ | SPY | QQQ |
|---|---:|---:|---:|
| confirmedUpper | 7.9972% | 4.2519% | -1.3113% |
| middleBand | 1.3929% | 0.8669% | -2.1449% |
| trend200 | -3.2474% | 2.1257% | -0.2552% |
| maxHold21 | 5.8303% | 2.2720% | -0.9571% |

중심선 청산은 연속6개 모두 upper보다 순평가수익이 낮고 회전이 늘었다. SPY20bp MDD는4.3719→2.3112%로 낮아지지만 순평가수익도0.0835→0.0609%다. upper의 잔여이익53,633.29원이 있어 기말 가정청산으로 비교하면0.0045→0.0609%로 순서가 바뀐다. 순평가수익·실현손익·가정청산을 섞어 승자를 고르지 않는다.

trend200은 SPY/QQQ 일부 손실·노출을 줄이지만 SOXQ20bp는6.9134→−3.7217%다. SOXQ2024Q3 별도계좌는−0.7740→−4.8026%, 완료2→1로 손실은 남고 회복 진입을 막았다. 평균투자비중9.4869→3.1649%의 감소를 알파 개선으로 표현하지 않는다. SOXQ2023Q4 +2.6783→0%,2025Q1 −1.3305→0%처럼 이익과 손실 기회를 함께 제거했다.

maxHold63은 연속6/별도분기72의78개 backtest가 upper와 정확히 같고 cap 신호0이다. 기록 보유의 최장 관측은 SOXQ49/SPY55/QQQ54로63에 못 미쳤다. 이 자료에서 **효능 평가 불가**이며 효과 없음으로 기각하거나 안전한 보유 상한이라고 결론 내리지 않는다.

사후 maxHold21은 QQQ 연속 손실을 조금 줄였지만 SOXQ/SPY 수익은 양 비용에서 모두 낮아졌다.20bp 최저 완료손익은 세 종목 모두 upper와 같아서 큰 손실을 방지했다고 볼 수 없다. cap 신호를 이유 접두어로 세면 SOXQ10/20bp는4/6건(체결4/4·차단0/2), SPY7/9건(체결7/6·차단0/3), QQQ4/4건(모두 체결)이다. SOXQ20bp의21관측 후보 두 건은 비용 guard 뒤22관측에 재시도했고, SPY2023-09-15 종가21관측 후보는3회 차단 뒤09-20 종가24관측 후보→09-21 시가에 체결됐다. 제한은 청산 **후보를 내는 관측 수**이며 nextOpen·거절·부분체결 때문에 실제 보유 상한이나 손실 상한이 아니다. 연속 합계29체결/5차단, 별도분기18체결/4차단이다. 관측 최장55→24의 감소를 보유 상한 보장으로 해석하지 않는다. 기존 공통 guard는 그대로 유지했다.

각 비용별36개 별도 분기계좌의 upper 대비 순평가수익 개선/악화/동일 개수다. 분기 간 수익률은 합산·복리하지 않았다.

| 후보 | 각10bp | 각20bp |
|---|---|---|
| middleBand | 7 / 11 / 18 | 6 / 12 / 18 |
| trend200 | 5 / 5 / 26 | 5 / 5 / 26 |
| maxHold63 | 0 / 0 / 36 | 0 / 0 / 36 |
| maxHold21 (사후) | 3 / 6 / 27 | 2 / 7 / 27 |

어느 후보도 자동 기본값으로 채택하지 않는다. 기존 upperBand, long_trend_period=0, max_hold_observations=0을 유지하며 UI에서 선택·단독 A/B가 가능하다. 이번 완료는 가설별 구현·비교·기각/평가 불가 기록의 완료이며 운용 성능 검증의 완료가 아니다.

## 정산·감사 근거

[validation.json](validation.json)의 부모 기록 산술74,968개가 통과했다.468개 원장의 다음 관측 시가/종가 신호 시점·방향별 슬리피지·현금·원가·수수료·실현/잔여손익·equity/MDD·회전/노출·기말청산,36개 분할 key·입력 동일성과 원래30개 대조군 hash를 확인했다. [summary.csv](summary.csv)와 [aggregate.json](aggregate.json)은 같은 원장 요약이다. Stage A 독립 감사175,490개(원장174,060+flat102의1,430)는 오류0이며 최종468개의 감사 수로 재사용하지 않는다. 별도 stock_analyst는 최종468원장+flat102를 **219,769개 독립 검사, 오류0**으로 대조했다. 금액 허용0.011원/지표0.000061%p에서 최대 차이0.0100000091원/0.0000500161%p다. Stage A390원장의 backtest/signals/replay/candles 전체와30개 partition/input/economic hash 동일도 확인했다. 생산 분할 실행을 독립 재실행한 감사는 아니다. 독립 감사 scratch `.cache/mean-stage-b-independent-audit.py`의 SHA-256은 `a890f0c5041f53999372701201d7b5cc1cba617cd4ea292f1214ea4b37ba8c20`다. 추가 금융 정합성 P1/P2 발견은 없다.

원장 비압축 SHA-256은 연속 `0f03e6e164d61ef42b0f4024a5866dbe2ab6ffe80b47c59c5a6fdc7d65f3bb1b`, 분기 `48066809e709d6b6fcdc79dbf398f0babce1477227b8ba6bad9dac595b363e91`다. gzip은 각각956,625/975,062바이트다. 해시와 자료·소스 snapshot을 validation에 보관한다.

## 한계

선택된 상관ETF3개·이미 본 가격·고정환율/무배당·자료 달력/기업행사 coverage 미인증과 작은 청산 표본이다. 동일 최초수량은 동일 실제 투자 비중이 아니며 평균/최대 투자 비중은 봉말 관측이다. 실제경로 비용환급은 비용0으로 전략을 다시 실행한 수익률이나 독립 알파 benchmark가 아니다. 낮은MDD/무거래/잔여평가이익을 우수성으로 바꾸어 표현하지 않는다. fixture와 production 가격 replay는 실제 broker/Tauri주문·장중체결·실운용 검증을 대신하지 않는다.
