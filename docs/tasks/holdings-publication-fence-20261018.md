# 작업 기록: P1-BAL-01 잔고 응답 게시 보호

## 범위와 위험

- 작성일: 2026-10-18 KST, 시작 커밋 `323727c`, 현재 브랜치 `fix/mac-security-storage-2026-10-06`.
- 상태: Done — 구현·회귀 검증·최종 독립 리뷰 완료.
- 위험: 높음 — 계좌별 국내/해외 포지션, 시작 복원, 수동 주문 전 잔고, 브라우저 캐시 경계.
- 목표: 조회 중 계좌·투자 모드·실행·설정·포지션이 바뀌면 이전 잔고로 현재 상태를 덮지 않는다. 실제 빈 잔고와 삭제 종목은 정상 반영한다.
- 실제 주문·자동매매 시작·provider 인증은 실행하지 않았다. 민감 계정 파일은 읽지 않았다. HTTP 회귀는 synthetic token과 localhost 서버만 사용했다.

## 구현과 계약

- 기존 tracker 게시 14곳을 추적했다. 국내/해외 `replace`의 생산 호출은 이제 `OrderManager::publish_holdings` 각 한 곳이다. IPC·daemon·IPC/Web 시작·수동 주문 갱신이 같은 gate를 거친다. Web 시작은 해외 tracker도 함께 반영한다.
- 조회 ticket은 profile fingerprint·client identity·broker/account·투자 모드·실행 epoch·설정 Weak lease identity와 시장별 version을 고정한다. 잘못된 source로 시작한 요청은 정상 요청을 무효화하지 않는다. 네트워크 await 동안 전역 mutex를 유지하지 않는다.
- tracker는 최신 조회와 수량 변경 UUID를 따로 보관한다. 새로운 조회·buy/sell/clear/replace·재생성이 이전 응답을 무효화한다. price update는 수량 version을 변경하지 않는다. 두 시장 응답은 모두 검증한 뒤 함께 게시한다. 최신 요청이 실패해도 오래된 요청으로 되돌리지 않는다.
- 실제 empty/0수량은 삭제로 처리하고 잘못된 수량·음수/NaN 가격은 전체 응답 오류로 처리한다. 해외 가격은 cents 단위, 거래소는 기존 trim/대문자 매핑을 유지한다.
- 일반 읽기 전용 Web/broker 조회도 반환 전 identity를 확인한다. ReadOnly 조회는 tracker나 다른 수량 조회 version을 바꾸지 않는다. no-profile 빈 응답도 gate를 거친다.
- KIS 공통 fetch는 종목명 저장을 완료한 뒤 최종 gate를 실행한다. daemon 이벤트는 성공한 gate 안에서 동기 발생한다. 프런트엔드는 잔고 이벤트 payload를 직접 저장하지 않고 현재 계좌 query를 다시 조회한다.
- 국내·해외·broker holdings key는 profile/broker/account/모드를 포함한다. query 시작/종료에서 신뢰 가능한 설정과 scope를 확인하고 전환 시 취소·이전 cache 제거를 수행한다. 설정 불확정/오류 때는 이전 잔고를 숨긴다.
- profile 변경 5종을 직렬화했다. 새 설정 공개 전에 이전 설정 query를 취소한다. 추가/삭제 뒤 설정 조회 실패는 오류로 표시하며 복구 전까지 이전 계좌 data를 노출하지 않는다. 늦게 등록된 listener는 해제하고 unmount 뒤 callback을 무시한다.
- IPC/REST 입력·DTO·이벤트 payload 형식은 유지한다. 새로운 IPC는 없다.

## 조사와 독립 검토

- symbol_navigator: 기존 게시 14곳·읽기 전용 경로 조사와 최종 `replace` 참조 확인. 생산 누락 없음.
- helper_curator: Graphify CLI 후보 조사와 Serena 참조 검증으로 profile fingerprint·해외 거래소 정규화의 동등성 확인. broker 검사는 별도로 유지하며 다른 의미의 주문 거래소 helper는 합치지 않았다.
- backend 독립 리뷰: 계좌 identity, lock 순서, 설정 lease, 실제 fill, 최종 callback 경계를 확인. 새 P1/P2 없음.
- frontend 독립 리뷰에서 발견한 늦은 AppConfig 응답·설정 재조회 실패·동시 mutation 순서 문제와 backend 적용 후 mutation 오류를 수정하고 각각 브라우저 회귀를 추가했다. 최종 재검토에서 추가 P1/P2·교착·폴링 재개 결함 없음.
- project_mapper: 신규 모듈·흐름·테스트 파일을 두 지도에 반영. 문서 pass에서 지적한 과거 이벤트 예시와 다중 탭 quota 보장 문구를 현재 공통 gate·query 무효화와 추가 요청 가능 경계로 수정했다.

## 검증

| 검사 | 결과 |
|---|---|
| Rust workspace locked 전체 | 359 lib + 3 integration + 3 doc = 365 통과 |
| Rust check / Clippy all-targets all-features / fmt | 경고 0, 통과 |
| TypeScript / FSD / web production build | 통과, 경고 0 |
| 계좌 전환·잔고 이벤트 Playwright | 13 통과 |
| 기존 strategy scrollbar / errors Playwright | 26 통과 |
| TODO 렌더러 회귀 | 3 통과, 사용자 판단/검증 대기 강조·필터 보존 |
| 지도·그래프·TODO·diff 공백 | 모두 통과, 열린 TODO 27개 |
| 문서 로컬 링크 | 159개 확인, 깨진 링크 0 |
| 최종 독립 코드·문서 리뷰 | 필수 지적 수정 후 추가 P1/P2 없음 |

Graphify의 노드 없는 JSON 설정 4개 진단은 기존 도구 분석 한계이며 Rust/TypeScript 코드 경고와 구분한다. 모든 변경 소스는 1,000행 미만이다.

Rust fixture는 A→B→A, 정지/재시작, profile/account/key/secret/모드/동의/client 교체, 설정 lease 시작·종료, 후속 query, 실제 partial fill, buy/sell·clear·재생성, 실제 empty/0수량·두 시장 원자성·가격 갱신을 포함한다. HTTP barrier는 실제 공통 fetch coordinator를 사용해 늦은 응답과 이벤트 미발행을 확인한다. 실제 `on_fill` 경로로 수량이 바뀐 뒤 이전 전체 잔고를 거부하는 회귀도 포함한다.

## 남은 경계와 롤백

provider가 제공한 잔고 자체의 지연이나 조회 시작 전 이미 반영된 fill과 이후 reconciliation의 이중 반영까지 해결한 것은 아니다. 정상 체결의 소유 전략 피드백과 함께 P1-FILL-01의 lifecycle 검증으로 남긴다. 공유 계좌 총수량을 각 전략의 소유 수량으로 추정하지 않는다. 실제 provider mapping·운용 검증 대기는 별도 TODO에 유지한다.

롤백은 이 변경 커밋을 revert하고 지도·그래프·TODO를 재생성한다. 기존 연구 JSON/CSV·실운영 설정·주문 기록은 변경하지 않았다.
