# 작업 기록: 전략별 체결 소유권과 계좌 잔고 분리

## 1. 메타데이터

- 작업 ID: P1-FILL-01
- 상태: Done — 구현·전체 회귀·독립 최종 리뷰 완료
- 작성일: 2026-10-18 KST
- 대상 브랜치: `fix/mac-security-storage-2026-10-06`
- 시작 커밋: `886dd5b`
- 담당: Codex, 전략·체결 worker, 리스크·잔고 worker, 독립 리뷰 및 지도/심볼 감사
- 위험: 높음 — 주문 수량·원가·리스크 복원·영속화 경계

## 2. 목표와 시작 근거

정상·부분 체결의 실제 잔여 수량을 주문 소유 전략에만 전달한다. 같은 종목의 다른 전략/수동 보유나 계좌 전체 수량을 전략 보유로 배정하지 않는다. 브로커 잔고에 이미 포함된 체결을 뒤늦은 로컬 대조가 다시 더하거나 빼지 않는다. 거부·취소 후 확정 보유를 유지하고 신규 신호의 임시 상태를 정리한다.

기존 `on_fill_on`은 계좌 tracker를 delta 변경했고 시작 경로는 전체 계좌 수량을 여러 전략에 복사했다. LTH의 일반 `sync_position`은 최고가·보유 관측 수를 초기화했다. P1-BAL-01의 조회 version 검증은 이미 체결을 포함한 broker snapshot 뒤의 중복 가산을 막는 증거가 아니므로 별도 lifecycle 검증이 필요했다.

## 3. 구현 계약

- 계좌 tracker는 검증된 broker snapshot으로 교체한다. 체결은 이전 조회 version을 무효화하고 해당 시장을 재조회 대상으로 표시한다. 재조회 전 신규 예약은 보류한다.
- `OwnershipLedger`는 broker/account/profile/투자모드·전략·시장·종목별 확정 수량과 정수 원가를 보관한다. 자동매도는 소유 수량과 계좌 관측 수량의 교집합으로 제한하고 예약 직전에 다시 확인한다. 수동매도 가능 수량에서 자동 소유·예산 장부의 보유를 제외한다.
- live 신호 관측은 지표·LTH 최고가/관측 수를 갱신하지만 매수/매도 신호만으로 확정 보유를 바꾸지 않는다. 실제 owner 피드백을 전략 manager에 전달한다. replay의 기존 체결 가정은 별도 경로를 유지한다.
- manager → order lock 순서를 유지한다. 주문 mutex 아래에서 전략 mutex를 기다리지 않는다. 영속 소유 원장과 LTH/신고가 실패 보호 metadata를 이용해 재시작을 복원한다.
- 소유 원장과 generation manifest를 함께 검증한다. 누락·세대 충돌·손상은 신규 주문을 차단한다. pending과 소유 원장을 오래된 백업으로 자동 되살리지 않는다.
- 체결 intent의 일자·환율과 idempotent 리스크 영수증을 보관한다. 당일 재복원은 동일 TradeRecord 이벤트를 사용하고 계좌와 투자모드를 검증한 뒤 후보 저장 성공 후 공개한다. 자정 첫 체결이 데몬 초기화보다 먼저 도착해도 당일 손실을 반영한다.

별도 발견한 Toss 수수료 단위 오류도 함께 수정한다. [공식 OpenAPI v1.2.24](https://openapi.tossinvest.com/openapi-docs/latest/openapi.json)의 `Commission.commissionRate`는 소수 비율이며 `0.00015` = `0.015%`다. 기존 `/100` 계산과 UI의 원문+`%`를 수정하고 raw wire 값은 유지한다. 현금10001원/필요10002원의 부족 경계와 실제 패널 퍼센트·필요금액 표시를 검증한다. 오류/음수 비율은0 수수료로 통과시키지 않는다.

공식 금액 필드 근거: [KIS 국내 샘플](https://github.com/koreainvestment/open-trading-api/blob/main/examples_llm/domestic_stock/inquire_daily_ccld/chk_inquire_daily_ccld.py)의 `tot_ccld_amt`, [KIS 해외 샘플](https://github.com/koreainvestment/open-trading-api/blob/main/examples_llm/overseas_stock/inquire_ccnl/chk_inquire_ccnl.py)의 `ft_ccld_amt3`, Toss `OrderExecution.filledAmount`(native currency). 공개 문서만 조회했으며 계좌 API는 호출하지 않았다.

## 4. 검증·리뷰 기록

시작 작업 트리에는 이 작업의 기존 미커밋 변경이 있었으며 이를 보존해 이어서 구현·검증했다. 최종 확인 결과는 아래와 같다.

- [x] 전체 Rust test/check/clippy/fmt와 TypeScript/FSD/build 코드 경고0
- [x] fill·partial·reject·cancel·재시작·동일 종목 타 owner·모드/프로파일 경계
- [x] 전체 snapshot 선반영 후 체결 대조의 수량 중복 방지
- [x] 원가·손익·저장 실패 재시도와 누락/손상 manifest
- [x] LTH 최고가·보유 관측 보존 및 보호 metadata 재시작
- [x] 독립 최종 리뷰·문서/스킬·project-map·graph freshness·TODO HTML
- 게시 확인: 이 기록을 포함한 Git 커밋과 현재 브랜치 origin 동기화 상태로 확인한다. 강제 푸시는 사용하지 않는다.

## 5. 안전·실행 범위

합성 자격증명과 localhost fixture만 사용한다. 실전/모의 provider 주문이나 자동매매 시작을 실행하지 않는다. `.env`, `secure_config.json`, `profiles.json`과 실제 계좌 자료를 읽지 않는다. 기존 연구 입력·결과와 파라미터를 변경하지 않는다.

도입 이전 보유·원가 불명 체결을 계좌 총잔고만으로 특정 전략에 귀속하지 않는다. 기존 원장 손실/모드 미상 자료는 추정 손익으로 통과시키지 않는다. 실제 provider 지연, 운영 계좌 원장 확인, 연구 과제의 실제 분봉 검증은 합성 fixture의 완료와 구분한다.

generation manifest와 원장을 모두 삭제한 뒤의 cold start는 신규 설치와 구분할 외부 증거가 없다. 한쪽 누락·세대 불일치·실행 중 교체 감지의 검증을 모든 로컬 기록 삭제 탐지로 확대하지 않는다. 저장 원본을 보존해야 하며 계좌 총잔고만으로 잃어버린 전략 귀속을 복원하지 않는다. 신호 교차·재진입/매수 누적 정책은 기존 P2 과제로 유지한다.


## 6. 최종 검증과 감사

| 확인 | 결과 |
|---|---|
| `cargo check --workspace --all-targets --locked` | PASS, warning0 |
| `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` | PASS; Copy clone/expect_err 지적을 수정한 뒤 경고0 |
| `cargo fmt --all -- --check` | PASS |
| PG 환경 변수를 제외한 `cargo test --workspace --locked -q` | 단위449 + integration3 + doc3 =455 PASS |
| `npx tsc --noEmit`, `npm run check:fsd`, `npm run check:lockfiles`, `npm run build:web` | PASS, 코드 경고0 |
| `env -u NO_COLOR npx playwright test` |89 PASS; 신규 수수료 패널1 포함, 실제 provider 호출 없음 |
| `npm run verify:toss-openapi` | 공식 v1.2.24·38 paths PASS; 공개 문서 조회만 수행 |
| TODO render/check 및 renderer tests | 열린26개, tests3 PASS; 판단 대기 주황·키보드/모바일 필터 유지 |
| project-map update/check, Graphify refresh/check | PASS; 신규 모듈 책임과 인벤토리·caller graph 갱신 |
| 상대 링크·diff 공백·연구 보존 | 링크162개 오류0; 연구 폴더 변경 없음 |

실제 KIS/Toss 주문·자동매매, 외부 DB fixture/운영 원장은 실행하지 않았다. Rust의 환경 의존 DB contract는 실제 PostgreSQL/MariaDB 운용 검증을 의미하지 않으며 TLS·강제 단절 과제는 TODO에 유지한다.

독립 리뷰는 legacy sell에 남은 whole-account 원가 fallback, 자정 첫 손실/실전·모의 합산, KIS 시작/수동 경로의 약한 숫자 변환, 0수량 terminal의 저장 intent 유실, 소액 즉시 fill의 응답 식별자/정수 수량 누락을 발견했다. 모두 반례로 수정하고 마지막 두 finding도 재검토해 해소를 확인했다. 별도 strategy reviewer는18개 live feedback 회귀를 통과시키고 LTH/신고가 실패·다른 owner·replay 보존에서 신규 finding 없음을 보고했다.

symbol_navigator는 daemon·IPC 시작·Web 시작의 공통 coordinator와 exact 전략 hook, production direct fill인 소액 검증의 연결을 추적했다. helper_curator는 exact TR-ID 분류의 공통 사용과 collector/소액 검증의 수량·총액·날짜 helper 재사용을 확인했다. holdings의 소수 내림과 사전평가 estimate는 체결의 exact 정수/총액 계약과 비동등하므로 합치지 않는다. mapper가 지도와 원본 스킬 drift를 두 차례 확인했고 지적을 반영했다.

Graphify의 symbol 없는 JSON설정4개 진단은 기존 도구 한계이며 코드 경고와 구분한다. 5065 node HTML은 도구 기본5000 상한을 넘는다. update는 이를 생략했고 cluster-only는 fuzzy merge로1개 node가 줄어 JSON 교체를 거절했다. canonical graph.json 바이트를 보존한 채 설치된 Graphify exporter/report API와 node limit6000으로 HTML·보고서를 다시 생성했다. 최종5065 nodes/10772 edges/522 communities가 일치한다. 전체 그래프 재분류를 금융 정확성의 증거로 사용하지 않는다. `commands/toss.rs`1093행은 기존 P3 분리 대상이며 이번에는 내부 rate 필드 참조1곳만 변경했다. 새 모듈을 이 파일에 추가하지 않았다.
