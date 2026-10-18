# 작업 기록: P2-14 replay 체결 모델

## 1. 메타데이터

- 작업: SOXQ-P2-14, 2026-10-18, Codex+전담 리뷰·주식 분석·지도/심볼/helper 역할
- 상태: Done — 구현·연구·전체 회귀·독립 리뷰 완료; 게시 커밋은 Git 이력 참조
- 브랜치: fix/mac-security-storage-2026-10-06, 시작 ba74e03, clean
- 위험: 보통 — 연구 replay와 공유 UI. 실제 주문/자동매매 시작은 없음

## 2. 목표와 비목표

- [x] 같은 종가 기본값과 다음 관측 시가 체결을 구분한다.
- [x] 신호 확인/체결의 봉과 공개 단계를 분리하고 마지막 대기 신호를 미체결로 남긴다.
- [x] 별도 장중 보호 시나리오·갭·동봉 순서 불명/손절 우선과 실제 원가 피드백을 검증한다.
- [x] QQQ 비용·체결 시점에 따른 실제 전략 청산 경로를 고정한다.
- [x] 일반/LTH 분봉·공통 UI·A/B 저장·구형 응답을 연결한다.
- 비목표: 실제 거래소 체결/유동성·LTH 시장시각 인증, 실제 주문·운영 설정 변경, 전략 파라미터 최적화, 연속3년 운용.

## 3. 시작 상태와 조사

사용자 변경 없이 clean에서 시작했다. 민감 `.env`·secure_config.json·profiles.json을 읽지 않았다. Serena 조사에서 generic prefix 재계산과 LTH의 신호 발생 시에만 호출되는 callback이 다음 시가 pending settle을 놓칠 수 있음을 확인했다. Trade/Rust↔TS·공통 패널·A/B·두 transport의 영향 범위를 추적했다. KIS/Toss API 동작 변경은 N/A다.

## 4. 구현과 계약

- simulation/session.rs의 단일 ReplaySession이 공통 회계·guard·risk·ATR·수수료/세금/부분 원가·equity를 소유한다. batch wrappers와 generic/LTH가 같은 엔진을 쓴다. 시작에는 이전 ATR·open, 종가에는 완료 OHLC를 사용한다.
- generic은 대기 체결/보호 결과가 바뀐 경우 평가 전 동기화하고, 신호 뒤에는 실제 체결 결과로 speculative 상태를 해소한다. 완료 일봉 적재 순서는 유지한다.
- LTH Open/Signal/Close driver는 신호 없는 봉도 먼저 정산하며 Open에 close high-water를 미리 넣지 않는다. 구형 bool/position API와 일봉 진단은 유지한다.
- 연구 보호0이 기본이다. 갭은 관측 open, 장중은 unverified 또는 stop-first 가정이다. 같은 종가 신규 진입에 과거 고저가를 적용하지 않으며 양방향 접촉을 순서 불명으로 표시한다.
- serde enum/선택 응답과 TS mirror를 함께 갱신했다. 새 IPC·등록·query key는 N/A(기존 두 Tauri/Web 경로). 요청 가정 변경은 기존 generation/key로 stale 응답을 폐기한다. live JSON/DB·credential schema는 N/A다. 구형 A/B 필드는 확인 불가로 표시한다.
- raw 전략 신호와 보호 주문을 구분하며 filled/blocked/unfilled, signalAvailableAt/fillAt 단계·경고를 UI에 표시한다. 봉label은 정확한 거래소 timestamp가 아니다. sameClose lookAheadSafe=false, nextOpen generic의 기술적 불변 검증과 실제 체결 검증을 구분하고 LTH false는 유지한다.

## 5. 금융 안전·도구·리뷰

실제/모의 주문·자동매매 시작·계좌 접근은 하지 않았다. scope·동의·submission/pending/risk 영속화 정책은 변경하지 않았다(N/A). 연구용 snapshot/비용을 운영 원장으로 사용하지 않는다. 소스 파일은 모두1000행 미만이다.

Serena 영향 분석, 초기/최종 독립 코드 감사, helper 감사(공통 회계 재사용 확인), project mapper와 별도 주식 분석 감사를 수행했다. 리뷰에서 찾은 기존 연구 runner의 open_units 누락3곳을 보강하고 두 runner도 offline compile-check했다. Graphify는 회계 모듈 추출로 refresh하며 canonical graph를 보존한 HTML/report/labels의 일치 검증을 수행했다.

## 6. 연구 근거와 한계

[독립 감사와 재현](../research/replay-execution-20261018/report.md). 직전 ba74e03 생산코드1,224회와 신규 기본 sameClose의 전체 회계 report/signals가 일치한다. 초기v7 보관 결과와의12개 차이는 이미 반영된52주고가 수정이며 새 회귀가 아니다.

QQQ2026Q3 두전략×두비용×두모델8회와 같은 체결 규칙 benchmark8회를 실제 생산API로 실행했다. 평균회귀 nextOpen은 -0.4372%/-0.4772%, 강한종가는 +0.1049%/+0.0039%지만 완료 손실과 열린 평가이익이 섞인다. 고정FX·일봉·소표본이며 배당/기말 청산/실제 경로를 검증하지 않는다. 합성 장중 보호 fixture와 실제 일봉 비교를 섞지 않는다.

## 7. 검증

| 결과 | 명령/검증 | 결과와 범위 |
|---|---|---|
| PASS | cargo test --workspace --locked(PG fixture 환경 제거) | 단위480·통합4·doc3. 외부 DB5개는 일반 실행 early-return이며 이번 replay 검증의 실서버 근거가 아님 |
| PASS | cargo check --workspace --all-targets --locked | 경고0 |
| PASS | cargo clippy --workspace --all-targets --all-features --locked -- -D warnings | 경고0 (최종 재확인 통과) |
| PASS | TypeScript/FSD/lockfiles/web build | npx tsc --noEmit·FSD·lockfiles·build:web 통과 |
| PASS | focused browser | 신규6개+인접32개, legacy·A/B·모바일·LTH 진단 포함 |
| PASS | 전체 Playwright |95개 통과; mock transport, 실계좌/실제 주문 아님 |
| PASS | simulation 집중 |25개(신규13), generic clock·피드백과 고정QQQ, production LTH 다음 시가/미체결 |
| PASS | offline 연구 |1,224회 기본 전체 회계/신호 일치, 실자료8+benchmark8; 기존 연구runner2개 compile-check |
| PASS | 지도·그래프·TODO·링크·문서 | project-map·graph freshness, 상대 링크·hash·diff·TODO renderer3개 통과. 열린24개, 사용자 대기 강조 유지 |

기존 미래봉 불변 테스트는 sameClose의 lookAheadSafe=true를 기대해1회 실패했다. 불변 신호 검증은 유지하고, 종가 체결 가능성과 분리한 false/경고 계약을 명시 검증하도록 고친 뒤 전체를 통과했다. 연구 harness 최초 생성은 workspace lock을 복사하기 전 registry 최신 후보를 선택해 중단했고, 프로젝트 lock을 복사한 offline 실행으로 재현했다. 원본 input/results/validation은 보존했다. TODO renderer의 검색 회귀는 완료된P2-14 제목을 하드코딩해1회 실패했다. 실제 열린 제목을 검색하도록 고치고 카드/필터/색상/모바일/노스크립트 검증3개를 다시 통과했다.

## 8. 완료·롤백·운영자 확인

기존 live 제어와 실제 운영 설정은 변경하지 않았다. 취소는 이 작업 커밋을 revert하고 Rust/공통 UI 회귀를 재실행한다. 연구 자료는 개인키·계좌·주문 trace를 포함하지 않는다. 배포/실전·모의·소액 테스트 승인은 N/A(이번 범위 외). 열린 P2-15·P2-18·전략별 복수창 비교와 사용자 자료/운용 판단 대기는 별도 완료 조건으로 유지한다.
