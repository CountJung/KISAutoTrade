# P2-18: 연속 장부와 분기 초기화 비교

2026-10-18. 생산 replay v12의 전략·회계 상태를 청크 경계를 넘어 유지하도록 구현하고 [사전 고정 설계](design.json)로 검증했다. 실제 계좌·주문·브로커 인증 호출은 없다. 이 연구는 이미 관측한 과거 자료의 재분석이며 투자 권고·운용 승인·미관측 표본의 우수성 검증이 아니다.

## 구현과 검증 범위

기존 flat API의 평가+history 합계500봉 상한은 유지한다. 새 `preview_strategy_continuous`/`preview_strategy_continuous_from_candles`와 REST `/api/strategy/preview/continuous`는 1~16청크·청크당1~500봉·평가합계4096봉·history512봉 상한을 갖는다. history와 각 청크 내부/경계는 정규화 후 엄격한 시간순이어야 한다. 전역 정렬로 역전·겹침을 숨기지 않는다.

공통 driver는 전략 생성·최초 warmup·ReplaySession 생성·최종finish를 각각1회만 실행한다. 청크 경계에는 수량/pending 관측만 남기고 포지션·native평균/원화원가·ATR·쿨다운·시장 날짜별/연속 손실 상태를 재생성하지 않는다. 다음 관측 시가 체결은 경계 다음 봉으로 이어지며 전체 마지막 pending만 미체결로 남는다. 서버에 장기 세션이나 계좌 상태를 저장하지 않는 단일 요청이다. peak 계산은 기존의 이전 반올림 equity 최댓값을 증분 보관해 O(N²) 재검색을 제거했다.

직전 커밋 `cc6f5d0`의 생산 라이브러리1,224회와 신규 기본 flat의 **전체 backtest report·신호가 일치**했다. 새13개 회귀는 실제 PC/모멘텀 전략을 사용하며, 단일 driver 공급/분할 공급·720봉·pending양방향·cooldown·같은 날짜 손실차단/다음날 복구·연속손실3회·ATR/FX/최소fee·엄격한 상한과 시간순을 검증한다. 미래 청크 변경의 과거 신호/체결/equity/실제 수량/native평균/pending 불변을 sameClose/nextOpen 양쪽에서 확인했다.13개는 테스트 수이며13전략 전수 fixture라는 뜻이 아니다. 4096평가+512history의 단일 디버그 실행은 이 환경에서 약54ms였다.

UI는 연속 일봉 모드의 시작/종료 범위와 엄격한 이전252봉을 수집한다. 최대48페이지·보관4608봉, 평가4096봉을 넘기지 않는다. 요청 범위 양끝이나 관측 사이에7달력일을 넘는 공백, warmup부족·불완전 페이지·scope변경 등은 계산을 보류한다.7일은 앱의 보수적 자료 경계이며 거래소 휴장 검증이 아니다. 긴 휴장도 보류될 수 있고, 통과해도 달력 coverage는 미검증으로 표시한다. 누락봉은 합성하지 않는다. 취소·편집 뒤 stale 응답 폐기, A/B의 모드/continuity 보존·legacy미확인, 연속 장부의70/30을 독립학습/검증으로 해석하지 않는 표시를 연결했다. LTH generic 거부·일봉 평가불가 경계는 확장하지 않았다.

## 사전 고정 연구 조건

- 자료: [동결 원본](../strategy-review-20261018/input.json), SOXQ/SPY/QQQ. 입력 SHA-256 `d48ce5b59b7876c0a1457121e877fdc1117a77f990b4ec07ce649c92fe153317`을 강제 확인한다.
- 평가:2023-01-03~2025-12-31의 관측752봉(250/252/250), 엄격히 이전252봉(2021-12-31~2022-12-30).2026 자료는 평가에 사용하지 않았다. 거래소 달력·기업행위·가격 조정 완전성은 독립 검증하지 않았다.
- 일반12전략×3ETF×비용2종=연속72회. 각 결과를 달력 분기12청크와500+252의2청크로 다시 공급한72대조에서 **전체 report·signals·preparation과 경제 inputHash가 일치**했다. 청크 분할은 hash에 넣지 않으며 계산모드는 구분한다.
- 자본1천만원/FX1450/배당없음/세금0/연구보호overlay0/nextOpen. 추정 수수료와 슬리피지를 각각10→20bp로 비교한다. 실제 요율·USD현금 장부·배당 포함 총수익을 재현하지 않는다.
- 수량은 원래2023Q1의10bp 비용 buffer를 포함한15%목표 산정값을 한 번 고정했다: SOXQ51주/SPY2주/QQQ3주. 정수 수량과 비용 때문에 실제15% 노출이 아니다. 파라미터도 원래Q1 설정을 고정했고 PC의 최초 기준가는19.91/382.43/266.28, 내부 수량도 동일하게 유지한다. 이후 분기의 가격을 보고 재설정하지 않았다.
- 별도 분기 초기화는 같은 수량/파라미터/비용과 엄격히 이전252봉을 사용해12분기를 각각 초기현금1천만원으로 실행한864회다. **분기 계좌 수익률을 더하거나 복리로 연결하지 않는다.** 연속 장부의 분기 equity/pending checkpoint는 같은 비초기화 계좌의 상태다.

## 관측 결과

아래는3년 연속 자산 수익률(%)이며, 칸 내부 순서는SOXQ/SPY/QQQ다. 각 비용에서 세 ETF의 완료 청산 합계도 같이 표시했다. 표는 전략 ID순이며 순위가 아니다. 상세 실현/잔여·MDD·노출·회전·차단·단순보유/동일경로 참조는 [summary.csv](summary.csv), 초기화 분포는 [aggregate.json](aggregate.json)을 확인한다.

| 전략 |10bp 수익률 SOXQ/SPY/QQQ|20bp 수익률 SOXQ/SPY/QQQ|완료 청산 합계10/20|
|---|---|---|---:|
|consecutive_move|6.4536 /2.9175 /9.1525|5.7071 /−0.3474 /8.2701|73/71|
|deviation|5.2714 /1.1944 /3.2066|4.5454 /1.1317 /3.0785|9/9|
|failed_breakout|24.6736 /8.1850 /14.1995|24.6405 /8.1618 /14.1745|0/0|
|fifty_two_week_high|−3.6678 /5.9840 /12.0123|−3.8304 /5.9076 /11.9829|3/3|
|ma_cross|9.8147 /4.8227 /8.3260|7.4794 /3.7986 /6.6560|61/61|
|mean_reversion|7.9972 /4.2519 /−1.3113|6.9134 /0.0835 /−1.8039|28/28|
|momentum|20.1403 /4.5582 /11.4065|19.2936 /4.2505 /11.0740|17/17|
|price_condition|0.7526 /0.5483 /0.5670|0.6917 /0.5026 /0.5194|3/3|
|rsi|8.8303 /2.0024 /2.3211|7.9658 /1.5844 /1.7912|23/23|
|strong_close|26.4001 /8.6412 /15.1004|26.3704 /8.6190 /15.0772|0/0|
|trend_filter|10.9148 /6.4121 /11.8010|9.5760 /6.1817 /11.5863|19/19|
|volatility_expansion|25.5628 /5.6269 /10.7332|25.5314 /5.5507 /10.6501|2/2|

각 비용36실행 중 완료 청산20건 미만이31개, 청산0회가8개다. 강한종가·돌파실패는 양비용/세ETF 모두 청산0회로 큰 수익이 잔여 평가 성격이다. SOXQ 강한종가는 같은 요청수량 단순보유와 정확히 같다. PC는 최초 고정기준가에서 각ETF 청산1회, 평균 초기자본 투자비중 중앙0.1259%로 희소 노출이다. 작은 절대수익을 다른 전략과 동일노출 성과로 비교하지 않는다.

비용 변경은 단순 비용 차감 이상이다. SPY 연속상승/하락은+2.9175→−0.3474%, 완료 청산29→28/차단1→5로 바뀌며2023-08-04 매도가08-07로,2025-02-25 매도가03-31로 이동한다. SPY 평균회귀도+4.2519→+0.0835%이고 동일경로 비용복원은+4.9703→+1.5065%라 청산 경로 자체가 달라졌다. 모멘텀20bp 차단195건 중187건이 무보유 매도, 이격도286건 중 무보유 매도200/보유중 매수82다. 기존 중복 신호·체결 피드백 TODO의 추가 근거로 사용하며 guard를 완화하지 않는다.

연속72개 중 실현/자산 증감의 역부호가9개다. 예를 들어 SOXQ 추세필터20bp는 총+9.5760%이지만 실현−240,553.37원/잔여+1,198,152.86원·MDD16.2171%다. 기말 가상청산 비용으로 수익 부호가 바뀐 연속 사례는0개였다. 이전 P2-15의 QQQ2026Q3 부호 전환과 구분한다.

분기 초기화432개/비용의 수익률 중앙값은0%, 완료 청산0회270/271개다. 이는 별도 초기화 계좌의 분포이며 연속3년 수익률과 같은 통계가 아니다. SOXQ2024Q3/2025Q3/Q4의 단순보유 비교72개(3분기×12전략×2비용)는 요청51주가 첫시가에서30% 비중 한도를 넘어 실제0주다.0%를 유효한 동일노출 보유성과로 해석하지 않는다. 연속/초기화 사이 자본·보유·리스크 상태 차이가 실제 체결/차단을 바꾸는 효과를 그대로 남겼다.

## 동일 수량·보유구간 참조

단순보유는 같은 **요청수량**이며 실제 수량·노출이 같다는 보장은 없다. 별도로 실제 accepted 체결 수량·시점을 고정한 시장가격 경로 참조를 대사했다. 이번 fixedFX/no배당/nextOpen/nooverlay 조건에서:

`원화 시장손익_t = 1450 × [전봉 보유수량 × (현재시가−전봉종가) + 현재시가 체결후 보유수량 × (현재종가−현재시가)]`

전봉 수량은 현재 시가 전에, 현재 수량은 accepted 시가체결 후에 정해진다. 미래 종가의 신호로 이미 지난 갭/장중 노출을 채우지 않는다. 부분매도·차단은 실제체결만 반영한다. 이 합+초기자본은 기존 **final+기록된fee+tax+slippage** 비용환급 참조와 일치했다. 같은 수량·보유구간의 비용 분리이며, 환급으로 자산 분모가 달라져 같은%투자비중은 아니다. 무비용 전략 재실행이나 독립적인 알파 benchmark도 아니다.

## 재현·감사와 보관

[runner.rs](runner.rs)는 생산 API를 호출하며 전략 신호를 재구현하지 않는다. [validate.py](validate.py)는 원본 OHLC와 기록된 실제 fill로 현금·매봉 equity·비용·관측 노출·terminal·위 두구간 시장손익·경계 ledger를 독립 대사했다. **136,717체크**가 통과했다. 별도 주식 분석 역할이 이 검증 코드를 재사용하지 않고936개 전체에서 **143,866체크**를 수행해 일치했으며 최대 돈차이는 약0.01원/비중차이는0.00005%p였다. 별도 core 감사도 입력/상태/수치/IPC 경계에서 P1/P2를 발견하지 못했다.

연속/초기화 전체 원장은 [continuous-results.jsonl.gz](continuous-results.jsonl.gz)와 [reset-results.jsonl.gz](reset-results.jsonl.gz)에 손실 없이 압축했다. 약30MB 평문을 약4.2MB로 보관하며, 평문 생성본은 프로젝트 `.cache/continuous-research-output/`로 옮긴다. SHA-256·압축 전 SHA/바이트·생산 소스 hash는 [validation.json](validation.json)에 기록한다. 검증기는 평문 또는 압축본을 읽고 동일 대사를 수행한다. 소스 안정화/포맷 후 hash를 다시 생성했으며 원본 연구 파일은 수정하지 않는다.

저장소 루트에서 재현한다. 두 harness는 아래 manifest와 루트 Cargo.lock을 각각 사용하고 `target/`을 공유한다.

1. `git archive cc6f5d0`을 `.cache/continuous-head-baseline/`에 풀고 baseline manifest를 `.cache/continuous-head-baseline-harness/Cargo.toml`에 둔다.
2. `cargo run --offline --manifest-path .cache/continuous-head-baseline-harness/Cargo.toml --target-dir target -- "$PWD"`로 직전 생산 라이브러리 기준1,224개를 만든다.
3. current manifest를 `.cache/continuous-research-harness/Cargo.toml`에 둔다.
4. `cargo run --offline --manifest-path .cache/continuous-research-harness/Cargo.toml --target-dir target -- "$PWD"`로 flat 대조와72+72+864실행을 수행한다. [run-summary.json](run-summary.json)의 기대 카운트가 다르면 실패한다.
5. `python3 docs/research/continuous-replay-20261018/validate.py`로 원장 대사·요약 CSV/분포·손실 없는 압축·해시를 생성한다. 원장의 generatedAt/실행시간은 재실행마다 달라질 수 있으며 경제 report/signals·inputHash로 재현성을 판단한다.

baseline manifest:

```toml
[package]
name = "kis-continuous-head-baseline"
version = "0.1.0"
edition = "2021"
[workspace]
[dependencies]
kis-auto-trade = { path = "../continuous-head-baseline/src-tauri" }
serde_json = "1"
[[bin]]
name = "baseline"
path = "../../docs/research/continuous-replay-20261018/baseline_runner.rs"
```

current manifest:

```toml
[package]
name = "kis-continuous-research"
version = "0.1.0"
edition = "2021"
[workspace]
[dependencies]
kis-auto-trade = { path = "../../src-tauri" }
serde_json = "1"
[[bin]]
name = "research"
path = "../../docs/research/continuous-replay-20261018/runner.rs"
```

3ETF의 상관·선정편향, 이미 관측한 과거·같은 자료 반복, 작은 청산 표본, 고정FX/무배당/추정비용, 관측 시가의 체결 가능성·유동성·달력/기업행위 미검증은 남는다.2026을 이번 실행에 쓰지 않았다는 사실만으로 미관측 OOS 검증을 주장하지 않는다. 다음 전략별 가설은 하나씩 사전 고정해 실제 경로·노출·비용별 결과와 기각 결과를 보관한다. P2-16의 별도 walk-forward 조건은 이 작업으로 완료하지 않는다.
