# 작업 기록: SOXQ-P1-05 일봉 진단과 장중 평가 분리

## 1. 메타데이터

- 작업 ID: SOXQ-P1-05
- 담당자: Codex / symbol_navigator, 구현·리뷰·문서 위임
- 상태: **In Progress — 구현 검증 완료, 실제 3개월 분봉 자료 대기**
- 작성/갱신일: 2026-10-05 KST
- 브랜치/시작 커밋: main / cbc4553
- 관련 문서: [검증 기록](../docs/research/soxq-3m-20261003/p1-05-validation.md), [TODO](../todo.md)
- 위험 등급: 보통 — 지표 초기화는 live LTH에도 적용, 주문 제출·리스크 경계 변경 없음

## 2. 목표와 완료 조건

- [x] 일봉 시가를 15분 뒤 가격으로 사용하는 합성 관측을 제거하고 성과 평가 불가를 표시한다.
- [x] 일봉 context와 분봉 EMA/RSI/ADX·반동 버퍼를 분리한다.
- [ ] 2026-07-06~2026-10-02 실제 SOXQ 분봉 원본을 확보하고 거래소 시간대·DST·세션을 검증하여 전체 기간을 재실행한다.

실제 자료 없이 일봉을 분봉으로 보간하거나 코드 fixture를 시장 성과로 집계하지 않는다. 공급원/파일 경로를 사용자에게 질문했고 답변 전 독립적인 구현·검증을 진행했다. 실제 주문, 계정 인증, 유료 데이터 가입은 작업 범위 밖이다.

## 3. 시작 상태

`git status --short`는 비어 있었으며 main은 origin/main의 cbc4553과 일치했다. 이전 연구 CSV와 결과를 보존했다.

- [x] 사용자 변경과 작업 범위를 구분했다.
- [x] `.env`, `secure_config.json`, `profiles.json`을 읽지 않았다.

## 4. 조사와 영향 범위

| 확인한 정의/참조 | 사실과 변경 |
|---|---|
| LTH `initialize_ohlc` / `initialize_intraday_ohlc` | 같은 지표 deque에 일봉과 분봉이 쌓였다. bounded daily context를 분리하고 분봉 snapshot은 교체한다. |
| LTH `preview_signals_with_execution` | replay 역시 일봉으로 분봉 지표를 seed했다. 일봉 context만 보관하고 시간순 장중 관측으로 계산한다. |
| 전용 command의 일봉 변환 | 시가를 진입 시각으로 합성했다. 일자당 완성 OHLC 한 개만 반환하고 장중 엔진/체결 callback을 호출하지 않는다. |
| live warmup 호출 | Toss 일봉→분봉 순서와 KIS 일봉→live tick 순서를 보존한다. KIS LTH는 실제 장중 관측이 모여야 분봉 지표가 준비된다. |
| ResearchPanel / 저장된 A/B | optional assessment를 보존하고 daily/0관측/legacy LTH의 성과와 신규 A/B 저장을 숨긴다. |

`gap_pct`의 기존 연속 관측봉 간 의미와 선택형 반동의 지표 준비 전 진입 조건은 유지한다. 일봉 재초기화는 분봉·볼린저·포지션·high-water·보유 관측 수를 보존하며, 빈 분봉 snapshot은 장중 지표만 비운다.

## 5. 계약 체크

- [x] Frontend public API/FSD import: 기존 경계를 유지한다.
- [x] Rust serde ↔ TypeScript: optional `replay.assessment`를 미러한다.
- [x] Tauri 등록: N/A — 새 IPC 없음, 기존 command 유지.
- [x] Web transport: 기존 전용 command의 공유 결과 타입을 사용한다.
- [x] query/cache/event: N/A — 새 호출·query key·event 없음.
- [x] persisted JSON: assessment 없는 과거 LTH A/B 결과도 성과 평가 불가로 취급한다.
- [x] IPC 문서/프로젝트 지도: 새 contract와 모듈 책임을 갱신한다.

엔진은 v7, LTH 전략 버전은 v3이며 hash와 응답은 같은 상수를 사용한다. `warmupCount=0`, `dailyContextBars`와 `intradayBars`를 별도로 표시한다. 시간대/세션은 `unverified`, `lookAheadSafe=false`이며 계산의 결정성과 시장 시각 검증을 구분한다.

## 6. 금융·자동매매 안전 검토

- 실제/모의 주문 및 자동매매 시작: **NOT RUN**.
- [x] BrokerScope, 수량/가격/통화·매수가능금액·동의·리스크 검증 경계를 보존한다.
- [x] 중복/반대방향 submission, timeout/rate limit, 불명확 응답 정책: 기존 처리 유지.
- [x] pending/order/risk 영속화와 재시작·비상정지 정책: 변경 없음.
- [x] 민감 계정/provider 응답 원문을 기록하지 않는다.
- [x] 일봉 평가 불가와 분봉 표본의 기간·시각 한계를 화면에 표시한다.

분봉 지표 초기화 수정은 자동매매 전략에도 적용된다. 운영 배포는 자동매매 정지 상태에서 수행하고 모의·소액 검증은 별도 승인 범위로 남긴다.

## 7. 구현과 롤백

LTH facade와 live/preview/indicators/exits/session/tests를 분리하고 simulation의 assessment/tests도 분리했다. 기존 public API와 주문 피드백을 유지한다. 문제가 생기면 해당 변경 커밋을 revert하고 이전 버전으로 복원한다. 이전 합성 일봉 성과를 재검증된 결과로 사용해서는 안 된다.

## 8. 도구·위임 기록

- Serena: 공용 초기화/preview 호출과 live 영향 범위를 symbol_navigator가 추적했다.
- Graphify: 파일 분리에 맞춰 graph를 갱신하고 freshness를 검사한다. helper 승격은 없었다.
- 독립 리뷰: hash/metadata 버전 불일치 및 유효 분봉 0개 상태를 발견해 공용 상수/`notEvaluable`로 수정했다. 최종 재검토 PASS, 남은 코드 finding 없음.
- project_mapper: 프로젝트 지도와 신규 파일 인벤토리·문서 coherence pass.
- 구현 worker: LTH 버퍼/회귀, UI/E2E와 원본 스킬·운영 문서 갱신.

## 9. 검증

| 상태 | 검증 | 결과 |
|---|---|---|
| PASS | `cargo test --workspace --locked` | lib 190 + production 통합 3 + doc 3 = **196** |
| PASS | `cargo check --workspace --all-targets --locked` | 코드 경고 0 |
| PASS | `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` | 코드 경고 0 |
| PASS | `npx tsc --noEmit`, `npm run build:web` | 타입/production 웹 빌드 |
| PASS | `npm run check:fsd`, `npm run check:lockfiles` | 경계/lockfile |
| PASS | 기존 전략+assessment Playwright | 기존 22개 + assessment 4개 = **26** PASS |
| PASS | SOXQ 오프라인 반복 실행 | 24개 지원 결과와 2개 거부 결과 v6 일치; 생성 시각 제외 전체 JSON 일치 |
| PASS | fmt / graph / project-map / 최종 diff | freshness/구조/포맷·diff 검사 |
| NOT RUN | 실제 3개월 분봉 재평가 | 원본/접근 가능한 공급원 미확보 |

Node의 `NO_COLOR`/`FORCE_COLOR` 충돌 메시지와 Graphify의 symbol 없는 JSON 설정 4개 진단은 도구 환경/분석 진단이며 코드 빌드/Clippy 경고와 구분한다. 캐시는 ignored 프로젝트 내부 `.cache/`와 `target/`에 유지한다.

## 10. 전달 상태와 잔여 작업

검증된 구현과 문서는 현재 브랜치에 커밋하고 origin에 푸시한다. **P1-05 전체 완료는 아니다.** 남은 자료에는 원본 시각의 시간대, 봉 시작/종료 의미, 정규/연장 세션, 거래소 캘린더, 조정 여부와 누락 구간을 확인할 수 있는 metadata가 필요하다. 실제 자료를 확보한 뒤 세션 모델을 검증하고 전체 3개월 replay를 수행한다.

- [ ] 운영 배포 전 자동매매 정지 확인: N/A — 이번 작업은 배포/자동매매 실행 없음.
- [ ] 모의·소액 운영 검증: 별도 승인 작업.
- [x] 롤백·비상정지와 provider trace 경계는 기존 운영 문서를 유지한다.
