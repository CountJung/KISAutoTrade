> 후속 최신 결과: [P1-03(v5) 날짜 전환·완료봉 검증](p1-03-validation.md). 이 문서는 v4 시점의 관측을 보존한다.

# SOXQ-P1-01 검증과 재평가 (2026-10-03)

엔진 `strategy-replay-v4`. 같은 동결 SOXQ 입력의 평가 64봉(2026-07-06~10-02)을 13개 전략 모두에 유지했다. 사전자료 모드는 이전 252봉을 `historyCandles`로 전달한다. `ui_three_month_defaults`는 추가 사전자료가 전혀 없는 비교 진단이며, 현재 UI의 provider 응답 수에 따라 확보하는 사전자료 조회를 재현한 이름은 아니다.

[전체 기본 결과](p1-01-results.json), [각 20bp 비용 스트레스](p1-01-cost20-results.json), [이전 v3 검증](p1-02-validation.md). 기존 v2/v3 보고서·원시 결과는 당시 관측으로 보존한다.

## 수정과 경계

- 생략한 warmupCount는 모든 전략에서 0. 별도 history와 명시 legacy prefix는 같은 신호·hash를 만들며 결과 candles에는 평가 봉만 남는다.
- 6개 전략의 과거 가격 버퍼 및 MA/RSI 이전 지표를 직접 복원한다. 과거 on_tick 실행·가상 포지션 생성 없이 보유 상태를 유지한다. 신고가 기준은 완료된 과거 252봉 전체 고가와 마지막 종가를 사용한다.
- actual history readiness와 조건 평가 가능 여부를 분리한다. firstReadyTime/unreadyEvaluationBars는 처리 후 지표 상태, evaluatedBars는 처리 전 can_evaluate_next_tick의 실제 조건 평가 가능 여부다. MA/RSI/돌파 실패 seed-only 마지막 봉은 readyAtEnd=true여도 notEvaluable이다. 미지원 값은 null이며 미준비 false로 세지 않는다.
- Live manager의 신고가 경로는 provider 완료 여부를 알 수 없는 최신 1봉을 제외하는 기존 정책을 유지한다. Generic history는 엄격한 완료 prefix이므로 마지막 봉까지 포함한다. Live 252입력→251자료 부족, 253입력→252준비, 1봉 재초기화의 stale 기준 제거를 회귀 검증한다. 고가 준비가 부족해져도 보유 손절이 작동하도록 손절을 먼저 평가한다.
- 합계 500봉, 일/주/월봉 날짜 8자리·분봉 시각 14자리, 유효 OHLC, 중복/겹침/모호한 prefix를 검사한다. 자료 오류를 걸러 평가 봉을 사전자료로 대신 쓰지 않는다.
- UI는 동일 요청 평가 봉 수를 우선 보존한다. KIS 한 응답과 Toss 현재 커맨드 최대 200봉 안의 이전 자료만 사용한다. 부족 상태를 표시하고 추세 200·신고가 252 기준을 축소하지 않는다. 페이지 조회는 SOXQ-P2-17로 남겼다.

## 사전자료 252봉 모드

| 전략 | 시작 준비 | 조건 평가 봉 | 결과 | 수익률(각 10bp) | 각 20bp | 완료 거래 |
|---|---|---:|---|---:|---:|---:|
| ma_cross | 완료 | 64 | traded | -1.1197% | -1.2549% | 2 |
| rsi | 완료 | 64 | traded | +2.0224% | +1.9147% | 2 |
| momentum | 완료 | 64 | traded | -1.2637% | -1.4574% | 3 |
| deviation | 완료 | 64 | traded | +1.3756% | +1.2649% | 2 |
| fifty_two_week_high | 완료 | 64 | conditionsNotMet | +0.0000% | +0.0000% | 0 |
| consecutive_move | 완료 | 64 | traded | -0.6445% | -0.7816% | 2 |
| failed_breakout | 완료 | 64 | traded | -0.0005% | -0.0851% | 1 |
| strong_close | 완료 | 64 | conditionsNotMet | +0.0000% | +0.0000% | 0 |
| volatility_expansion | 완료 | 64 | traded | -0.0300% | -0.0600% | 0 |
| mean_reversion | 완료 | 64 | traded | +2.2083% | +2.1561% | 1 |
| trend_filter | 완료 | 64 | traded | +0.8722% | +0.8440% | 0 |
| price_condition | 완료 | 64 | traded | +2.2408% | +0.9777% | 12 |
| leveraged_trend_hold | 미지원 | 미지원 | notEvaluable | +0.0000% | +0.0000% | 0 |

모멘텀은 v3 -0.4868% → v4 -1.2637%, 이격도는 +2.4987% → +1.3756%로 달라졌다. 이는 사전자료 초기화 누락 수정에 따른 진단 결과이며 파라미터 최적화나 우수 전략 판정이 아니다. MA는 수익률이 같지만 시작 전 기준선 복원으로 raw 무보유 매도 차단이 0→1건이다. 평균회귀 +2.2083%와 추세 필터 +0.8722%는 v3 종가 수정 결과와 같다.

사전자료 없는 모드에서 신고가·강한 종가·변동성·추세 필터는 notEvaluable, LTH는 unsupported/notEvaluable로 표시한다. 0% 결과를 조건 불충족 성과로 오판하지 않는다. 평가 중 준비되는 가격 관측 지표는 부족한 시작 상태와 실제 조건 평가 봉 수를 함께 기록한다.

강한 종가/변동성의 날짜 전환(P1-03), LTH 입력 시각/실제 장중 모델(P1-04/05), next-open 체결(P2-14), true-range ATR(P2-16)은 해결하지 않았다. 특히 이들 표 수치는 엔진 진단이며 투자 성과로 판정하지 않는다. 각 10/20bp는 수수료와 슬리피지 각각 적용, 환율 1450·1천만원·10주·배당/환율 변화/기말 강제청산 제외라는 기존 가정을 유지한다.

## 재현과 검사

- 오프라인 동일 입력 재실행: generatedAt 제외 지원 전략 24개 실행 결과(12전략×2모드), 전체 신호·메타데이터·준비 상태·backtest 일치. generic LTH는 비결정적 진단이므로 이 주장에서 제외한다.
- 전 전략/모드의 평가 시작·종료와 64봉 길이 일치. 사전자료 모드의 지원 12전략 모두 시작 준비 완료, cold 모드 warmupCount는 모두 0.
- Rust 전체 lib 167 tests PASS, 전체 target check/Clippy all features와 rustfmt, TypeScript/FSD/lockfile 및 web build PASS. UI·문서/그래프 최종 게이트는 task 기록에 모아 둔다.
- 독립 코드 리뷰의 seed-only 조건 판정·mixed 날짜 granularity·비정렬 추세 기간 버퍼 문제를 모두 수정하고 회귀 테스트로 검증했다.

일회성 Cargo manifest/lock와 repeat JSON은 프로젝트 `.cache/research/soxq-replay/`, 빌드는 `target/`을 사용한다. 연구의 동결 입력·원본 runner·보존 결과는 이 문서와 같은 폴더에 남긴다.

```bash
cargo run --offline --locked --manifest-path .cache/research/soxq-replay/Cargo.toml -- docs/research/soxq-3m-20261003/replay-input.json docs/research/soxq-3m-20261003/p1-01-results.json
```

검증한 기본 결과 SHA-256: `8a49774fcfe58b201e037c2cec0ed2576546b36c14199fccb474da2c222946bf`

임시 harness manifest 내용(저장소 루트에서 `.cache/research/soxq-replay/Cargo.toml`에 저장):

```toml
[workspace]
[package]
name = "soxq-research-runner"
version = "0.1.0"
edition = "2021"
[dependencies]
kis-auto-trade = { path = "../../../src-tauri" }
serde_json = "=1.0.149"
[[bin]]
name = "soxq-research-runner"
path = "../../../docs/research/soxq-3m-20261003/runner.rs"
```

이 작업 환경의 harness lockfile은 `.cache/`에 보존했다. 새 환경은 위 manifest로 lockfile을 생성한 뒤 `--locked`로 실행한다.
