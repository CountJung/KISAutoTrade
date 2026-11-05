# Replay v10 체결 모델 비교

완료: 같은 입력을 사용한 직전 코드 `ba74e03`의 1,224회와 신규 엔진 기본 sameClose의 전체 회계 report·raw signals가 일치했다. 신규 가정·진단 필드만 비교에서 제외했다. 이후 QQQ2026Q3의 평균회귀·강한 종가를 두 비용·두 체결 모델로 8회 실행하고 같은 규칙의 benchmark8회를 비교했다. 원자료를 포함한 독립 주식 분석 감사에서 시점·가격·비용·손익·기말 보유·MDD 산술 검산이 일치했다.

## 입력과 의미

- [동결 입력](../strategy-review-20261018/input.json): SHA256 `d48ce5b59b7876c0a1457121e877fdc1117a77f990b4ec07ce649c92fe153317`.
- QQQ2026Q3: history `[927..1179)` 252봉, evaluation `[1179..1243)` 64봉(2026-07-01~09-30). 같은 사전 결정 수량1주·자본1천만원·환율1450·세금0, 수수료와 슬리피지는 각각10bp 또는20bp. 전략 params는 기존 기본값을 유지한다.
- Yahoo에서 보관한 실제 일봉이다. 캘린더 coverage·조정/기업행동·실제 호가·장중 경로는 인증하지 않았고, 배당·변동FX·기말 강제 청산은 제외했다. 연구 보호 stop/take는 0으로 비활성이다.
- Benchmark도 첫 평가봉 **종가 신호**를 입력한다. sameClose는 그 종가, nextOpen은 다음 관측봉 시가에 같은 수량을 매수하고 기말 보유한다. 첫 시가 사전 매수 결과를 nextOpen에 끼워 비교하지 않는다. 기간과 수량은 같지만 실제 자본 투자비중·보유 기간은 같지 않다.
- sameClose 기본값은 호환을 위한 낙관적 체결 가정이다. nextOpen도 다음 **관측 봉** 시가와 고정 슬리피지를 사용할 뿐 유동성·실거래 가능성을 증명하지 않는다. 봉label+정보 단계와 정확한 거래소 timestamp를 구분한다.

## 관측 결과

순수익은 현금+기말 보유 평가액의 자본 대비 변화다. 잔여 손익은 순자산 변화−실현손익으로 열린 매수 비용을 포함하며 순가격 미실현액과 다르다. 강한 종가는 모든 완료2회가 손실이고 기말1주의 평가이익이 상쇄한다.

| 전략 | 비용 각bp | 모델 | 순수익% | MDD% | 완료 | 기말주 | 실현원 | 잔여원 | benchmark% |
|---|---:|---|---:|---:|---:|---:|---:|---:|---:|
| strong_close | 10 | sameClose | +0.3005 | 0.8311 | 2 | 1 | -81191.51 | +111238.02 | +0.1907 |
| strong_close | 10 | nextOpen | +0.1049 | 0.8182 | 2 | 1 | -81816.58 | +92306.71 | +0.1847 |
| mean_reversion | 10 | sameClose | +0.6274 | 0.6838 | 1 | 0 | +62736.35 | -0.00 | +0.1907 |
| mean_reversion | 10 | nextOpen | -0.4372 | 0.6844 | 1 | 0 | -43723.11 | +0.00 | +0.1847 |
| strong_close | 20 | sameClose | +0.2003 | 0.9313 | 2 | 1 | -89288.68 | +109316.13 | +0.1696 |
| strong_close | 20 | nextOpen | +0.0039 | 0.8996 | 2 | 1 | -89958.82 | +90346.98 | +0.1636 |
| mean_reversion | 20 | sameClose | -0.5659 | 0.7222 | 1 | 0 | -56591.14 | -0.00 | +0.1696 |
| mean_reversion | 20 | nextOpen | -0.4772 | 0.6846 | 1 | 0 | -47716.29 | +0.00 | +0.1636 |

평균회귀 sameClose의 10bp는 7/17 $696.02533에 진입해 9/21 밴드 익절, 20bp는 $696.72066 원가가 되어 7/29 손절한다. nextOpen은 같은 7/17 신호를 7/20 시가 $702.86216/$703.56432에 체결하여 두 비용 모두 7/29 손절 신호→7/30 시가 $674.08524/$673.41048 청산이다. 실제 체결 원가 피드백이 신호 경로를 바꾸는 회귀 fixture로 고정했다. nextOpen이 항상 성과를 낮춘다는 결론은 아니다(20bp의 손실은 sameClose보다 작다).

8개 모두 준비 완료·차단0·마지막 미체결0이다. 마지막 미체결, 시가 갭, 같은 봉 손절/익절 순서 불명은 별도 **합성 fixture**에서 검증했다. 실제 일봉8회로 장중 손절 성과나 LTH 분봉 성과를 검증했다고 해석하지 않는다. Exposure는 봉말 보유 비율이며 실제 평균 자본 투자비중이 아니다. 완료1~2회·단일창으로 최적 파라미터나 수익성을 확정하지 않는다.

## 기본 수치 호환과 과거 연구

[baseline-hashes.json](baseline-hashes.json)은 Git `ba74e03`의 라이브러리를 별도 `.cache/` 소스 snapshot에서 컴파일하고 동일12전략×51창×비용2종을 실행한 회계 report/signals의 hash와 요약이다. [baseline_runner.rs](baseline_runner.rs)가 생산 API만 호출한다. [runner.rs](runner.rs)는 현재 생산 API를 실행해 새 executionWarnings·가정4개·summary진단2개·trade시점/경로3개만 제외하고 전체 report/signals hash를 비교한다. 1,224개 모두 일치한다.

초기 v7 동결 연구(`2cd04db`) 요약과 다른12개는 이전 `191fa93`·`2c593de`의 52주 완료 고가 이동창 수정이 이미 반영된 차이다. 이를 이번 v10의 회귀나 새 수익 개선으로 보고하지 않는다. 과거 입력/결과/validation을 덮어쓰지 않았으며 기존 runner 두 파일은 새 open_units 입력을 채우는 3줄의 컴파일 호환만 수정했다.

## 재현

작업 디렉터리는 저장소 루트다. 로컬 임시 Cargo harness는 `.cache/`에 만들고 저장소 `Cargo.lock`으로 버전을 고정한다. 다음 manifest에서 `kis-auto-trade`는 `../../src-tauri`, bin `research`는 `../../docs/research/replay-execution-20261018/runner.rs`, 의존성은 `serde_json=1`이며 빈 `[workspace]`를 둔다. 첫 실행 전에 루트 lockfile을 harness로 복사한다.

```bash
cargo run --offline --locked --manifest-path .cache/replay-execution-harness/Cargo.toml --target-dir target --bin research -- "$PWD"
```

추적된 baseline hash와 현 코드의 전체 보고서·신호 일치를 검증한 뒤 results.json의 8회 비교를 다시 생성한다. baseline 자체를 재생성하려면 `git archive ba74e03`을 `.cache/replay-head-baseline/`에 추출하고 그 `src-tauri`에 의존하는 별도 Cargo harness에서 baseline_runner.rs를 실행한다. 현재 엔진에 연결해 baseline을 덮어쓰면 호환 검증의 독립 기준이 사라진다. 기본 생성물·입력의 hash는 [validation.json](validation.json)을 확인한다.

후속 작업은 기존 TODO P2-15의 실현/잔여·기말 청산 비용·실제 투자비중 분리, P2-10/P2-08의 고정 모델·비용 복수창 청산 비교, P2-18의 연속 상태/분기 리셋 비교에 통합한다.
