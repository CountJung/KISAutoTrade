# SOXQ-P1-02 — 고가·종가 warmup 분리 검증

- 상태: Done, 2026-10-03, main
- 위험: 보통. live/preview 공통 전략 초기화의 계산 의미 변경. 외부 계좌/API 호출과 주문 실행 없음.
- 근거: [최초 분석](report.md), [수정 전 결과](results.json), [입력](replay-input.json), [수정 후 결과·전체 비교](p1-02-results.json)
- 선택 이유: 평균회귀·추세 필터의 기준선에 고가가 들어가는 오류부터 해결해야 이후 전략 A/B 실험이 의미를 갖는다.

## 구현과 완료 조건

- 공통 `initialize_strategy_warmup()`은 OHLC 훅을 호출하고 historical 훅을 따로 호출하지 않는다.
- `Strategy::initialize_ohlc()` 기본 구현은 **종가** 배열을 historical initializer에 전달한다. 평균회귀 밴드·추세 필터 MA가 이를 사용한다.
- 52주 신고가는 OHLC 훅을 재정의해 **고가** 배열을 기존 initializer에 전달한다. 기존 직접 historical 호출 의미와 마지막 봉 제외 동작은 유지한다.
- LTH의 전용 OHLC 초기화, 강한 종가의 `(high, close)` 입력, 변동 범위 입력, 일봉/분봉 경계는 유지한다.
- 추세 필터 200봉, 신고가 252봉 기본값, 종목 독립성, 기존 보유 수량/평균가 동기화를 유지한다.
- replay engine version을 `strategy-replay-v3`로 올려 계산 의미가 다른 과거 결과와 해시를 구분한다. IPC 이름/serde/TS 타입/저장 형식 변경은 없다.
- 실제 UI의 일반 preview는 동일 공통 경로를 호출하므로 수정된 신호·성과와 v3 메타데이터를 반환한다. 새 설정은 없다.

## 회귀 검증

1. high=20000, close=10000 fixture에서 평균회귀 20봉 밴드 mean/upper/lower 모두 10000. 첫 10000 가격에는 Hold, 9000에는 Buy.
2. 252봉 상승 종가 fixture에서 추세 필터는 마지막 200종가로 초기화. long/mid/short MA = 10151.5/10241.5/10249, 첫 평가 가격 10252에 Buy.
3. 52주 신고가는 close=10000보다 높은 15000에도 Hold, 실제 high=20000 돌파 20001에 Buy. manager 공통 warmup과 기존 high 직접 초기화가 일치.
4. historical 호출은 일봉에 정확히 1회, intraday-only에는 0회. 고가와 분봉 종가를 일봉 종가 버퍼로 보내지 않음.
5. warmup은 기존 보유 상태/진입가를 보존하고 무관한 종목을 생성하지 않음. 분봉만 전달해도 일봉 버퍼가 변하지 않음.
6. 해외 USD→센트 preview에서 고가만 바꿔도 Mean/Trend 신호는 동일. 미래 평가 봉을 바꿔도 앞선 신호가 동일하며 252봉 사전자료는 평가 구간 밖에 유지.

## 동일 SOXQ 입력 재실행

평가 2026-07-06~10-02 64거래일, warmup 252봉, 기본 파라미터·예산·환율·수수료/슬리피지 가정은 최초 연구와 같다. runner 전체를 기본 비용, 비용 스트레스, 기본 비용 반복으로 실행했다.

| 전략/비용(수수료·슬리피지 각각) | 수정 전 계좌 수익률 | 수정 후 계좌 수익률 | 수정 전→후 MDD | 완료 거래 전→후 |
|---|---:|---:|---:|---:|
| 평균회귀 / 10bp | +1.4402% | +2.2083% | 1.6683%→1.6558% | 2→1 |
| 평균회귀 / 20bp | +1.3333% | +2.1561% | 1.6696%→1.6562% | 2→1 |
| 추세 필터 / 10bp | +0.8722% | +0.8722% | 1.6917%→1.6917% | 0→0 |
| 추세 필터 / 20bp | +0.8440% | +0.8440% | 1.6921%→1.6921% | 0→0 |

평균회귀의 7/7 매수와 7/17 손절은 사라지고, 7/29 매수→9/21 청산만 남았다. 고가 혼용 때문에 생성되었던 초기 거래가 제거된 결과다. 추세 필터의 8/7 첫 매수는 그대로지만 신호에 표시되는 장기MA는 7456→7371센트로 수정됐다.

- 26개 실행 × 기본/스트레스 = 52개 수정 전후 비교에서 평가 기간과 warmup 수가 동일했다.
- Mean/Trend 이외 모든 실행의 신호·backtest, LTH 전용 일봉 진단, 두 보유 benchmark가 동일했다.
- UI 기본 방식의 Mean/Trend는 warmup=0이라 결과가 동일했다. 사전자료/준비 상태 과제(P1-01)는 여전히 남는다.
- v3 기본 비용 반복은 `generatedAt`을 제외한 전체 JSON이 동일했다. 일반 LTH의 시스템 시각 의존성 해결을 주장하지 않는다.
- CSV SHA-256이 기존 입력 메타데이터와 일치했다. 기존 결과/차트는 v2 분석 기록으로 보존하고 이 파일로 정정 내용을 연결한다.

표본은 평균회귀 완료 1회, 추세 필터 완료 0회다. 이번 변화로 전략 우수성을 확정하지 않는다. next-open 체결, 날짜 이벤트, LTH 역사 시각/분봉 모델은 별도 과제다.

## 검증 기록

| 결과 | 명령/검사 | 세부 |
|---|---|---|
| PASS | `cargo check --workspace --all-targets --locked` | compiler warning 0 |
| PASS | `cargo test --workspace --lib --locked` | 142 passed. 신규 회귀 6개 포함 |
| PASS | `cargo clippy --all-targets --all-features --locked` | warning 0 |
| PASS | `npx tsc --noEmit` | 타입 오류 없음 |
| PASS | `rustfmt --edition 2021 --check <수정 Rust 파일 6개>` | 수정 파일 서식 일치 |
| PASS | `npx playwright test tests/e2e/strategy-scrollbar.spec.ts` | Chromium 18 passed, mock transport |
| PASS | `npm run graphify:update` → `npm run check:graphify` | 로컬 AST 그래프/소스 hash 갱신 |
| PASS | `npm run check:project-map` | 신규 연구 파일과 책임 문구 동기화 |
| PASS | 동일 입력·반복·전체 전략 비교 | 위 52개 비교와 기본 비용 반복 확인 |
| NOT PASS (기존 서식) | `cargo fmt --all -- --check` | 수정 전부터 존재하던 rate_limit.rs, commands/toss.rs, server/profiles.rs, storage/database_projection.rs, trade_store.rs 서식 차이. 이번 기능 변경 범위에서 전체 재서식은 생략 |
| NOT RUN | 실전·모의 주문 | 외부 주문 실행 없음 |

Graphify는 기존 빈 JSON 설정 4개에서 AST node를 만들지 못한다는 경고를 출력했다. Playwright는 기존 NO_COLOR/FORCE_COLOR 충돌 경고를 출력했다. Rust 빌드·Clippy 경고와 구분한다. PostgreSQL 외부 contract 테스트는 테스트 함수 통과에 포함되지만 DB 환경 변수가 없으면 내부 skip하므로 실제 DB 계약 검증으로 해석하지 않는다.

## 위임·보존·다음 작업

- Serena symbol navigator가 live 시작/서버 시작/일반 preview의 warmup 경로와 상수 참조를 확인했다.
- 별도 read-only 코드 리뷰에서 P0–P2 결함 없음. 권장한 호출 횟수 spy와 intraday-only 불변 테스트를 추가하고 mock 들여쓰기를 정리했다.
- project mapper가 기존 연구 설명을 보존하면서 프로젝트 맵과 문서/스킬 drift를 확인했다.
- 최초 git 상태의 AGENTS.md, todo.md, docs/project-map.md, docs/research/ 변경을 보존했다. 민감 파일은 읽지 않았다.
- 다음 권장 작업은 SOXQ-P1-01의 동일 평가 기간·충분한 사전자료·전략별 준비 상태 구현이다.
- 롤백 범위: 이 작업의 typed warmup/신고가 override/version/회귀 테스트 변경. 주문·guard·risk·provider 코드 변경은 없다.

재현은 [최초 보고서의 runner 명령](report.md#재현과-파일)을 사용하되 결과를 새 파일에 쓴다. 기본/비용20bp 재실행에서 `p1-02-results.json`의 대상 신호·backtest·inputHash와 비교한다. 기본 비용 반복의 generatedAt은 비교에서 제외한다.
