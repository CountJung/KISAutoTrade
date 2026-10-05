# SOXQ-P1-05 일봉 진단·분봉 지표 분리 검증 (2026-10-05)

**구현 검증 완료, 실제 3개월 분봉 재평가는 미완료다.** 엔진 `strategy-replay-v7`, 전용 LTH `leveraged-trend-hold-v3`. [작업 기록](../../../tasks/soxq-p1-05.md), [전체 오프라인 결과](p1-05-results.json), [이전 v6 검증](p1-04-validation.md).

## 수정 결과

- 전용 일봉 경로에서 시가를 15분 뒤 진입 가격으로 간주하는 합성 관측을 제거했다. 일자당 완성 OHLC 한 개만 표시하고 장중 전략/체결 callback은 호출하지 않는다.
- `assessment.model=dailyDiagnostic`, `performanceStatus=notEvaluable`을 반환한다. 일봉 및 평가 metadata 없는 과거 LTH 결과의 수익률·승률·성과 표와 신규 A/B 저장을 숨긴다.
- 일봉 context와 분봉 EMA/RSI/ADX·반동 지표를 별도 bounded buffer에 보관한다. 일봉 값만으로 분봉 지표를 준비하지 않는다. 분봉 snapshot 재초기화는 교체하며 일봉 재초기화는 분봉·볼린저·보유 포지션·high-water·보유 관측 수를 보존한다.
- 분봉은 `intradaySample/sampleOnly`, 유효 관측이 0개이면 `notEvaluable`이다. 최대 200봉 조회를 전체 3개월 성과로 표시하지 않는다. A/B에는 assessment를 보존한다.
- `warmupCount=0`과 별도 `dailyContextBars`를 사용한다. 시간대/세션은 `unverified`, `lookAheadSafe=false`로 표시한다. 현재 고정 KST 세션 모델의 시장 정확성을 검증했다고 주장하지 않는다.
- generic LTH는 계속 `UNSUPPORTED_GENERIC_REPLAY`로 거부한다. LTH 버전은 hash와 응답에 같은 상수를 사용한다.

## 동결 SOXQ 일봉 재실행

252개 사전 일봉과 2026-07-06~10-02의 64개 평가 일봉, 10bp 수수료/슬리피지와 기존 주문·리스크 가정을 그대로 사용했다.

- 지원 12전략 × 2모드 = **24결과**의 candles·raw signals·preparation·backtest가 v6과 일치한다. 버전/hash만 v7 계약으로 변경됐다.
- generic LTH 2모드의 구조화 거부와 benchmark는 v6과 일치한다.
- 전용 일봉 진단은 실제 64개 종가 시각의 equity trace를 보관한다. 장중 관측 0, context 252, 합성 세션 없음, 전략/체결 실행 없음이다. 무거래 cash trace의 0% 수치를 LTH 투자 성과로 해석하지 않는다.
- 생성 시각만 재귀 제외한 전체 JSON은 서로 다른 분·시스템 시간대의 반복 실행에서 완전히 일치한다.

| 실행 | 첫 generatedAt | 시스템 시간대 |
|---|---|---|
| 기본 | 2026-10-05T23:48:35.344877+09:00 | Asia/Seoul |
| 반복 | 2026-10-05T10:50:07.064736-04:00 | America/New_York |

결과 SHA-256: `8ede4d705810f32906827bbcabc93960a4e4bbfe7132624f9bd2b64ede60fd03`. 동결 CSV의 기존 CRLF 바이트와 SHA-256 `10ab23ff4af9bec44d4fe19da33f7603e01857610f6ce8ba60c8d87c33b02dae`를 그대로 보존했다.

```bash
cargo run --offline --locked --manifest-path .cache/research/soxq-replay/Cargo.toml -- docs/research/soxq-3m-20261003/replay-input.json docs/research/soxq-3m-20261003/p1-05-results.json
TZ=America/New_York target/debug/soxq-research-runner docs/research/soxq-3m-20261003/replay-input.json .cache/research/soxq-replay/p1-05-repeat.json
```

## 회귀 검증과 운영 영향

Rust 196개(lib 190·production integration 3·doc 3) PASS. 주요 추가 회귀는 일봉만으로 minute 준비 불가, 일봉 값 변경에 minute signal/지표 불변, 일봉 재초기화의 기존 상태 보존, 반복 분봉 snapshot 교체/빈 snapshot 초기화, 버퍼 상한, 실패 주문 피드백, 유효 분봉 0개 평가 불가다.

Production integration의 분봉 warmup과 09:04 매수→15:25 청산 fixture는 코드 회귀 자료다. 실제 SOXQ 시장 자료나 3개월 성과가 아니다. `gap_pct`의 기존 관측봉 간격 의미, 선택형 반동의 준비 전 진입, 보유 중 청산/주문 피드백을 유지했다. KIS 일봉 warmup만 있는 live LTH는 이후 실제 장중 관측으로 지표를 준비한다. 실전·모의 주문 및 자동매매 시작은 실행하지 않았다.

타입·웹 빌드·FSD·lockfile·Rust check/Clippy는 PASS, 코드 경고 0. 전략 UI 26개(기존 22·assessment 4) PASS이며 유효 분봉 0개 회귀를 포함한다. fmt·graph/map freshness·최종 diff와 독립 리뷰·문서 pass도 통과했다.

## 남은 완료 조건

현재 보관 자료는 일봉이다. **2026-07-06~2026-10-02 실제 SOXQ 1분봉 원본 확보, 원본 시간대·DST·거래소 세션 검증, 전체 기간 replay**는 아직 수행하지 못했다. 공급원 또는 이미 보관한 파일 경로를 사용자에게 확인 중이다. 실제 자료가 확보되면 원본/metadata·누락 구간을 보관하고 검증된 거래소 시각 변환과 세션 모델을 적용한 뒤 재평가한다. 일봉 보간이나 테스트 fixture로 이 조건을 충족시키지 않는다.

### 공급원 검토 (2026-10-05)

무료·무인증으로 해당 전체 기간을 가져오는 경로는 이번 조사에서 확인하지 못했다. 공급원 전체의 불가능성을 뜻하지 않으며 SOXQ의 실제 응답 coverage를 아직 검증하지 않았다.

- Alpaca는 기간 지정 historical bars를 제공한다. 주식 조회 예시는 API key/secret 인증을 사용하며, historical SIP는 종료 시각이 15분 이상 과거이면 구독 없이 조회할 수 있다고 설명한다. 따라서 유료 가입이 반드시 필요한 것으로 단정하지 않지만, 이번 작업에 제공된 인증/원본 자료가 없어 실행하지 않았다. [공식 historical bars](https://docs.alpaca.markets/us/reference/stockbarsingle-1), [공식 Market Data FAQ](https://docs.alpaca.markets/us/docs/market-data-faq).
- Alpha Vantage의 historical intraday는 공식 문서상 premium endpoint다. 기존 권한/원본 제공 없이 가입·결제를 진행하지 않았다. [공식 API 문서](https://www.alphavantage.co/documentation/).

공급원을 선택하더라도 봉의 시작 시각/종료 시각, feed 범위, 가격 조정 여부, 누락·중복, 세션 캘린더와 America/New_York 변환 검증이 선행돼야 한다.
