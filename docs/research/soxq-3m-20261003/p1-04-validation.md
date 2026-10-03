# SOXQ-P1-04 LTH 현재시각 경로 차단 검증 (2026-10-03)

엔진 `strategy-replay-v6`. **Generic LTH 실행을 명시 거부하고 기존 입력 시각 기반 전용 경로를 유지한다.** P1-04 완료 조건의 두 대안 중 generic 거부를 적용했다. 제공봉 오프라인 계약을 provider 조회로 바꾸거나 live 전략에 역사 시계를 주입하지 않았다.

[전체 결과](p1-04-results.json), [작업 기록](../../../tasks/soxq-p1-04.md), [이전 v5 날짜 이벤트 검증](p1-03-validation.md). 이전 v2~v5 결과/차트는 당시 관측으로 보존한다.

## 수정 경계

- LTH 실시간 `on_tick`은 `current_live_minute_key`, `session_minutes`, `in_blackout_window`에서 `Local::now()`를 읽는다. 일반 과거 재생에서 호출하면 분별 OHLC 병합, 보유 관측 수, 세션·장마감·blackout 판단이 실행 시각에 의존한다. `deterministic=false` 및 준비 미지원 표시만으로 실제 실행을 막지 못했다.
- Generic 진입부에서 factory와 같은 `starts_with("leveraged_trend_hold")` 기준으로 기본/접미 ID를 `UNSUPPORTED_GENERIC_REPLAY`로 거부한다. 봉 정규화·파라미터 해석·factory/warmup/tick 전에 적용하며 간격·history·현재 세션에 관계없이 같은 오류를 반환한다. 정상 신호/backtest/hash를 만들어 돌려주지 않는다.
- 오류는 입력 시각 기반 `preview_leveraged_trend_hold`를 안내한다. 기존 전용 커맨드의 Toss 활성 profile/account·interval 계약은 유지한다. Generic 제공봉을 해당 네트워크 조회로 자동 전환하지 않는다.
- 전용 `preview_signals_with_execution`은 `timed.time`으로 세션·blackout·장마감을 계산한다. 해당 production 함수를 `cfg(test)`의 live 시계 대역 없이 통합 테스트한다. 응답 생성 시각 `generatedAt`은 신호/체결/hash 입력에서 제외한다.
- UI는 이미 LTH를 전용 editor/preview로 분기한다. 새 화면/IPC/영속화 schema 변경은 없으며 E2E mock 엔진 버전만 v6으로 맞췄다.

## 같은 SOXQ 입력의 결과

평가 2026-07-06~10-02의 64봉, 사전자료 252봉/없는 모드와 기존 10bp 비용 가정을 유지했다.

- 지원 12전략×2모드의 **24개 정상 결과**: candles·raw 신호·준비 상태·backtest 모두 v5와 일치한다. 엔진/기본 전략 버전 및 그 버전을 포함한 input hash는 v6 계약으로 바뀐다.
- LTH generic 2모드: `rejectedRuns`에 구조화 오류만 기록한다. 정상 `preview`·무신호·수익률 0% 결과로 집계하지 않는다. 다른 예기치 않은 오류는 연구 runner 전체를 실패시킨다.
- 기존 전용 LTH 합성 일봉 진단은 v5와 동일한 무신호/미체결이다. 이는 실제 장중 평가나 수익률 판정 근거가 아니다.

재실행의 생성 시각:

| 실행 | 첫 결과 generatedAt | 시스템 시간대 |
|---|---|---|
| 기본 | 2026-10-03T23:34:39.865022+09:00 | Asia/Seoul |
| 반복 | 2026-10-03T10:37:33.494989-04:00 | America/New_York |

UTC로 변환해도 서로 다른 분이다. `generatedAt`만 재귀 제외한 **전체 JSON**(24 정상 결과의 hash 포함, 2 거부 결과, 전용 LTH, benchmark, 입력)이 완전히 일치한다. 이 검사에 비결정적 generic LTH를 제외해야 하는 예외가 더는 없다.

## 회귀와 한계

- 단위 테스트: 기본/접미 3 ID × 기본/일/분/주/월 7간격 × history 유무 2조건, 총 42 요청의 명시 거부. 잘못된 params/빈 봉에서도 clock/factory 준비 전에 거부한다.
- Production-library 통합 테스트 3개: 입력 시각 09:04 매수→15:25 장마감 매도와 backtest 2체결/완료 1거래의 반복 일치, 입력 시각의 세션 밖/blackout 차단, 미래 봉 변경 이전 신호/equity 불변. 테스트용 KORU 이름과 국내 세션·합성 가격은 코드 fixture이며 실제 상품·시장 자료가 아니다.
- Live LTH의 현재가 polling/주문과 전용 replay 로직은 수정하지 않았다. `leveraged_trend_hold.rs`의 기존 대형 파일에 신규 기능을 더하지 않았다.
- 합성 일봉 open/close 관측, 일봉 context/분봉 지표 혼합, 고정 KST 세션·거래소 시간대/DST·실제 3개월 분봉은 **P1-05**에 남아 있다. 이번 결과는 역사 입력 시각 의존성과 시장 시간 모델 정확성을 구분한다.

Rust 전체 workspace 187개(lib 181·production 통합 3·doc 3)와 UI 22개 PASS. 타입/구조/lockfile/web build/Clippy/포맷·map/graph 검증 및 독립 리뷰·문서 pass 완료. 최종 코드 경고는 0이며 도구 환경 진단은 작업 기록에 구분한다. 실제/모의 주문과 자동매매 시작은 실행하지 않았다.

```bash
cargo run --offline --locked --manifest-path .cache/research/soxq-replay/Cargo.toml -- docs/research/soxq-3m-20261003/replay-input.json docs/research/soxq-3m-20261003/p1-04-results.json
TZ=America/New_York target/debug/soxq-research-runner docs/research/soxq-3m-20261003/replay-input.json .cache/research/soxq-replay/p1-04-repeat.json
```

임시 manifest/lock 및 반복/check JSON은 ignored `.cache/research/soxq-replay/`, 빌드는 `target/`을 사용한다. 동결 CSV는 원래 CRLF 바이트와 metadata hash를 보존한다. 최초 커밋의 추가 CSV 검사에는 `git -c core.whitespace=cr-at-eol diff --cached --check`를 사용했다.

기본 결과 SHA-256: `5358b95ff0710fa725415a42004611b0bc76ffa720614a0c20aa5dd98420a74b`.
