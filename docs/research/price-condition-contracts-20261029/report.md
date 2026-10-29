# P2-13 가격조건 생산 계약·고정 일봉 재현

2026-10-29. 가격조건의 live 확정 소유 피드백과 생산 replay를 합성 fixture로 검증하고, 기존 고정 일봉 연구를 현재 v22 생산 API로 재현했다. **생산 주문·전략 정책은 변경하지 않았다.** 실제 provider·계좌·키·주문을 호출하지 않았으며 실제 장중 손절·재진입 효능은 미완료다.

## 생산 경로의 합성 검증

- [live 6개](../../../src-tauri/src/trading/order/feedback/tests/price_condition.rs): 실제 StrategyManager→공유 제출→loopback 응답→누적 체결→exact owner 피드백을 사용한다. 즉시 거절, 접수 후 미체결·취소·거절, 부분매수·부분매도, 설정수량/실제수량 차이, 동일수량 평균가 변경, 체결 재적용과 계좌·종목별 guard 분리를 확인했다.
- [replay 9개](../../../src-tauri/src/commands/strategy_preview/pc_replay_tests.rs): 합성 preview34회로 nextOpen 실제 native 평균가·FX 변경, 정확한 손절 임계값·마지막 pending, 미국 시장일·UTC 자정·다음날59/60분·DST29/30분, actual/config 수량 제한, native/overlay, 두 비용과 단일/분할12쌍의 전체 report/signals/preparation/hash 불변을 확인했다. 실제 거래소 session/coverage는 검증하지 않았다.
- live 평균가는 exact owner native 원가/수량의 올림, replay는 실제 평균 체결가의 반올림이라는 기존 계약을 보존했다. 현재 FX로 원화 원가를 역산해 native 손절 기준을 이동시키지 않는다. FX가 실제 매도 체결과 최종 자산에 적용돼도 native 신호가 같다는 단정도 보강했다.

PC Sell의 raw 요청은 설정수량이며 실제 제출/실행 경로가 소유수량을 제한한다. 실제 보유가 설정수량보다 크면 잔량이 남는다. live 기본 일일 매도1회 제한이 후속 매도를 먼저 막고 동일방향 guard도 별도로 막는 것을 확인했다. 이는 잔량·평균가를 보존하는 증거이며 잔량 청산 성공이나 손실 제한 효능의 증거가 아니다. native 손절은 일반 guard를 적용받는다. 연구 overlay는 별도 보호 신호로 guard를 우회하지만 후속 execution/risk 검사는 유지한다. 연구 overlay의 잔량 청산을 live 기본 정책의 결과로 해석하지 않는다.

provider 접수 성공은 체결과 다르다. 접수 후 미체결 취소·거절에도 기존 접수 guard 기록은 유지되고, 접수 전 차단·즉시 거절은 새 손절 당일 금지를 만들지 않는다. Buy preflight의 계좌 전체 보유·평균값은 PC exact owner 소유량이 아니다.

## 고정 일봉 설계 재현

[설계](design.json)는 이미 관측한 [원래 연구 설계](../continuous-replay-20261018/design.json)의 PC3종목만 그대로 사용한다. 2023-01-03~2025-12-31의752봉과 엄격히 이전252봉, 자본1천만원·FX1450·nextOpen·배당/세금/연구 overlay0, native 익절5%/손절3%, 수수료와 슬리피지 각각10/20bp를 유지했다. 기준가 SOXQ19.91/SPY382.43/QQQ266.28과 수량51/2/3은2022-12-30 기준으로 고정하며 후속 분기에서 재설정하지 않았다.

원본은 [동결 입력](../strategy-review-20261018/input.json), SHA-256 `d48ce5b59b7876c0a1457121e877fdc1117a77f990b4ec07ce649c92fe153317`이다. 출처·정규화 이력은 [기존 연구](../strategy-review-20261018/report.md)를 따른다. 거래소 달력·기업행위·조정 완전성은 새로 인증하지 않았다. 2026을 이번 실행에서 제외했지만 이후 다른 연구에서 이미 관측했으므로 미관측 holdout으로 부르지 않는다. 사전 감사에서 복사된 reserved 문구만 바로잡았으며 경제 입력·창·후보·선택 규칙은 바꾸지 않았다.

현재 생산API로6연속·6대체분할·72별도 분기 계좌를 실행했다. 연속의12개 분기 청크와500+252 대체 청크는 report/signals/preparation/inputHash가 같았다. 기존 보관 PC78원장과 현재78원장의 전체 backtest/signals/preparation도 정확히 같았다. `baselineCommit=ac3eb564...`는 회차 시작 소스 기준이며, 직전 커밋을 별도 빌드해 재실행한 비교라는 뜻이 아니다.

| 종목 | 고정 수량 | 연속 순수익률10/20bp | 비용별 완료 청산 | 평균 초기자본 노출 |
|---|---:|---|---:|---:|
| SOXQ | 51 | 0.7526% / 0.6917% | 각1회 | 0.0806% |
| SPY | 2 | 0.5483% / 0.5026% | 각1회 | 0.2430% |
| QQQ | 3 | 0.5670% / 0.5194% | 각1회 | 0.1259% |

비용별 연속3원장 합계 완료청산3회, raw손절·완료손절·차단0, 기말보유0이다. 별도 분기 초기화는 비용별36원장 중33개가 완료청산0이며 합계청산3, 손절·차단0이다. 분기 계좌 수익을 더하거나 복리로 연결하지 않는다. 비용별 회전율과 실현/잔여·MDD·노출은 [요약](summary.csv)과 [별도 계좌 분포](aggregate.json)에 보존했다.

이 결과는 손절 후 재진입·cooldown·최소 기대순익 gate가 실제 장중 손실을 줄인다는 증거가 아니다. 손절이 발동하지 않은 희소노출 표본이며3개 상관 ETF·비용쌍·겹치는 창을 독립 표본으로 세지 않는다. guard 완화·기본값 승격·사후 파라미터 선택을 하지 않았다.

## 검증·재현

[runner](runner.rs)는 실제 생산 Rust API만 호출한다. [검증기](validate.py)는 기록된 fill을 사용해 현금·원가·비용·손익·노출·기말·가격경로를 대사하며 전략 신호를 다시 생성하지 않는다. [validation](validation.json)의 독립 산술10,536건과 보관78원장 비교가 통과했다. 전체 결과는 [연속 원장](continuous-results.jsonl.gz)·[별도 분기 원장](reset-results.jsonl.gz)에 손실 없이 압축했다. [실행 요약](run-summary.json)에는 분할 일치 hash와 실행수를 남겼다.

별도 `.cache/pc-contracts-harness/Cargo.toml`은 package `kis-pc-contracts`, 독립 `[workspace]`, `kis-auto-trade={path="../../src-tauri"}`와 `serde_json="1"`, bin path `../../docs/research/price-condition-contracts-20261029/runner.rs`를 사용한다. 루트 Cargo.lock을 복사하고 저장소 루트에서 실행한다.

```sh
cargo run --offline --manifest-path .cache/pc-contracts-harness/Cargo.toml --target-dir target -- "$PWD"
python3 docs/research/price-condition-contracts-20261029/validate.py
```

Rust 전체/strict 검사와 문서·지도·그래프 결과는 [2회차 기록](../../tasks/immediate-backlog-20261029.md)에 둔다. 남은 P2-13 조건은 검증된 실제 장중 자료의 시각·시장일·DST·세션·coverage, 실제 접수·부분체결 근거를 확보한 뒤 동일 기준가·수량·두 비용·guard로 손절·재진입·회전·순손익을 비교하는 것이다. [현재 TODO](../../todo.md)에서 전체 ID를 미완료로 유지한다.
