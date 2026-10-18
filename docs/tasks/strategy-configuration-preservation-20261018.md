# 작업 기록: 설정 저장 중 전략 보유 보호 보존

## 1. 범위와 시작 상태

- 작업: P1-CONFIG-01, 2026-10-18 KST
- 상태: Done — 구현·독립 코드/문서 리뷰·전체 검증 완료, 현재 브랜치 커밋/푸시로 전달
- 브랜치: `fix/mac-security-storage-2026-10-06`, 시작 HEAD `3040e8c`
- 시작 상태: 이전 회차의 설정 transaction·전략 보존 초안과 리뷰 지적을 이어서 처리했다. 별도 문서 배치는 기존 `694fa14`에 반영되어 있다.
- 위험: 높음 — 전략 상태·저장·신호/주문 예약 동시성
- 민감 파일은 읽지 않는다. 합성 credential과 localhost fixture만 사용하며 실제·모의 주문과 자동매매 시작은 실행하지 않는다.

## 2. 문제와 완료 조건

기존 IPC/Web 설정 수정은 인스턴스를 재생성한 뒤 저장하고, 저장 실패 시 다시 재생성했다. 일반 전략의 보유 기준과 LTH 최고가·보유 관측 수가 사라질 수 있었다. 시작 시 position cache는 정상 체결 후 계속 갱신되는 자료가 아니므로 복원 원본으로 사용할 수 없다.

- [x] 13개 전략에서 같은 ID/broker/account의 잔존 대상 보유 보호를 유지하고 새 params/수량을 적용한다.
- [x] durable save 전 인스턴스/revision을 변경하지 않고 저장 실패 후 원래 손절을 유지한다.
- [x] 저장 await는 manager/order lock 밖에서 수행하고, 저장 중 신호·주문·전체 수동 잔고 게시 경합을 차단한다.
- [x] 보유 대상 제거·국내/해외 변경을 저장 전에 거부한다. PC의 빈 params.symbols도 실효 대상에 반영한다.
- [x] 새 대상에 다른 전략/수동 보유를 자동 배정하지 않고, 다른 프로파일의 private 상태를 이전하지 않는다.
- [x] waiter 취소와 저장 후 identity 변경·복구 실패를 검증한다.
- [x] 독립 코드/문서 리뷰와 전체 검증을 완료했다. 변경은 현재 브랜치에 커밋하고 origin에 푸시하며, 최종 commit/upstream 참조를 확인한다.

## 3. 구현과 안전 경계

`trading/configuration.rs`의 공통 coordinator를 Tauri와 Web이 사용한다. production constructor로 실효 대상을 정규화한 뒤 manager의 private 보유 보호와 order의 현재 tracker·인증·주문 충돌을 검사한다. 기존/새 대상의 합집합에 대한 Weak reservation을 등록하고 manager/order lock을 해제한 뒤 저장한다. 설정/시작/프로파일 변경의 `strategy_update_lock`은 owned task가 유지한다.

Weak lease가 살아 있는 동안 해당 scope의 새 tick 신호와 영향 종목 예약, 같은 scope의 전체 수동 holdings 게시를 보류한다. raw 신호가 아직 처리 중이면 저장을 시작하지 않는다. 보류를 시장 폐장으로 해석하지 않고 짧게 재시도한다. 보유 대상의 제출·pending·정정이 있으면 저장을 거부한다. 이미 제출 중인 주문을 저장으로 취소하거나 숨기지 않는다.

성공 시 manager→order에서 identity를 다시 검증하고 인스턴스를 유지한 채 설정/revision을 공개한다. 실패 시 기존 runtime/revision을 유지한다. 첫 저장 오류도 디스크 교체 후 fsync/commit 오류일 수 있으므로, 저장 후 검증 실패와 함께 이전 설정 복구를 시도한다. 복구 실패는 persistence block으로 보고한다. HTTP/IPC 대기자가 사라져도 owned task는 저장과 공개/복구를 완료한다.

보유 상태를 가진 전략은 정확한 기존 entry/buy 기준을 보존한다. LTH는 high-water와 held-observations도 보존한다. 관측 버퍼는 새 기간에 맞게 제한·재계산하며 평균회귀는 긴 새 기간의 밴드가 준비되기 전에도 보유 손절을 평가한다. 강한 종가의 이전 진입 대기는 새 기준으로 재생성하고 보유 포지션만 유지한다. 프로파일 publication은 같은 계좌 문자열이라도 private 인스턴스를 새로 만든다.

공유 tracker의 0수량은 private 보유를 삭제할 근거로 쓰지 않는다. 정상 체결의 전략 소유권 피드백과 일반 잔고 조회의 늦은 응답 경합은 후속 P1으로 분리한다. 현재 변경을 전체 주문 lifecycle 안전성이나 실제 provider 검증으로 확대하지 않는다.

## 4. 리뷰 지적과 반영

- 설정 대기 중 active_symbols/시장 필터가 false가 되어 5분 폐장 pause로 넘어갈 수 있었다. 설정 보류와 폐장을 구분한다.
- 가격 조건의 빈 params.symbols가 이전 target_symbols를 남겨 제거 검사를 우회했다. 빈 배열까지 constructor가 정규화하고 저장 전/후 회귀를 추가한다.
- 같은 종목의 여러 raw 신호 중 앞 주문의 폐장 오류로 조기 종료하면 나머지 unsettled가 남았다. 처리하지 않은 exact 전략만 현재 run gate에서 실제 수량으로 정리한다.
- 공유 tracker를 근거로 보유 보호를 지우는 초안을 제거했다. private held도 제거/시장 변경 검사에 사용한다.
- 첫 저장이 실제 디스크 교체 후 오류를 반환할 수 있다. 최초 오류에도 원본 복구를 시도하고 실패 시 신규 주문을 차단한다. 실제 저장 뒤 합성 오류를 반환하는 fixture로 검증했다.
- PC/LTH 실제 엔진은 첫 중복 entry를 사용한다. 실효 시장도 같은 우선순위를 사용하고 LTH legacy inverse 필드가 다른 거래 대상을 덮지 않게 했다.
- 빈 저장 프로파일에 이전 entry params가 남아 PC/LTH 대상을 재생성할 수 있었다. 프로파일 publication에서 기본 인스턴스의 params·targets·enabled·수량을 초기화한 뒤 새 저장본을 적용한다.
- 일반 차단/오류 피드백도 현재 bool→manager→order gate에서 최신 tracker를 읽어 해당 전략만 정리한다. provider await 이전 snapshot을 다시 쓰지 않는다.

## 5. 도구·계약·문서

- Serena: 이전 symbol_navigator의 manager→order, 전략 상태 및 체결 피드백 추적을 바탕으로 실제 정의/참조 확인.
- Graphify/helper_curator: CLI BFS 28개 후보와 Serena 대표 정의, 13곳의 정확한 첫 조건 비교로 ID/broker/account predicate 동등성을 확인했다. 이 검사만 StrategyConfig::same_identity로 모았으며 전략별 상태·관측 로직은 분리했다. Navigator가 새 정의와 정확히13개 호출자를 별도 감사했고 외부 API/serde 영향이 없음을 확인했다. Graphify MCP의 무응답은 중단하고 정상 동작하는 CLI를 사용했다.
- project_mapper: 신규 coordinator·manager/order 경계·daemon 파일의 지도 pass.
- IPC/Web 공개 입력·camelCase DTO·TS wrapper는 유지한다. 오류는 기존 UI mutation 오류 표시 경로로 전달한다.
- 문서: TODO·IPC·아키텍처·사용 가이드·Rust 원본 스킬·지도 갱신.

## 6. 검증

| 상태 | 명령/영역 | 결과 |
|---|---|---|
| PASS | `cargo test --workspace --locked -q` | lib342 + integration3 + doc3 = 348건. 기존 lib297 대비45건 추가 |
| PASS | coordinator/order configuration | 실제 manager/store 저장 실패·취소·불명확 저장 복구·held/params/대상·중복 시장·profile/client·pending/submitting 경계 |
| PASS | strategy reconfigure | 13전략 상태/관측 재설정과 PC 빈 목록 회귀 포함 |
| PASS | daemon pause·batch settlement | 설정 대기 후 같은 revision 재개, 폐장 batch exact 정리, 재시작 격리, 최신 수량 피드백 4건 |
| PASS | `cargo check --workspace --all-targets --locked` | warning 0 |
| PASS | `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` | warning 0 |
| PASS | `cargo fmt --all --check`, `git diff --check` | 형식·공백 검사 |
| PASS | `npx tsc --noEmit`, `npm run check:fsd`, `npm run build:web` | 타입·경계·production build |
| PASS | Playwright `strategy-scrollbar.spec.ts` | 저장 scope·전략 편집·미리보기·스크롤 등 기존22건 |
| PASS | Playwright `strategy-configuration-errors.spec.ts` | busy/held 제거/시장 변경/저장 오류 표시·draft 유지·재시도 4건 |
| PASS | `node --test scripts/render-todo.test.mjs` | 열린 항목 coverage·escaping·필터/키보드/모바일 3건. 샌드박스 Chromium 차단 뒤 승인된 재실행 통과 |
| PASS | `npm run check:lockfiles` | npm/Cargo workspace 일관성 |
| PASS | `npm run check:project-map`, `npm run check:graphify`, `npm run check:todo` | 생성 인벤토리·코드 snapshot 정상, 열린28개(기존27에서 CONFIG 완료·후속2 추가) |
| PASS | 로컬 Markdown/HTML 링크 | 156개, 깨진 링크0 |

초기 검증에서 오류 코드 불일치 1건, worker가 신규 모듈을 작성 중일 때의 missing module, root 테스트의 포맷 차이와 Clippy redundant async block을 발견해 수정했다. 위 결과는 수정 후 검증이다. 마지막 전체 테스트의 localhost 포트 바인딩14건은 샌드박스에서 PermissionDenied로 실패하여, 같은 명령을 승인된 환경에서 다시 실행해348건 통과를 확인했다. 첫 Playwright 실행의 NO_COLOR/FORCE_COLOR 환경 충돌 알림은 새 spec 실행 시 NO_COLOR를 제거하여 해소했다. Graphify의 기존 JSON4개 zero-node 도구 진단은 코드 컴파일 경고와 구분한다.

최종 전체 Rust 테스트는 KISAT_PG_HOST/PORT/USER/PASSWORD를 제거한 환경에서 실행한다. DB contract의 외부 서버 환경이 없는 조기 반환을 실제 서버 검증으로 계산하지 않는다. 실제·모의 주문, 계좌 자료, provider token/order endpoint는 사용하지 않는다.

## 7. 잔여 과제와 운영 경계

- P1-FILL-01: 정상/부분 체결·거부/취소의 exact 전략 소유권 피드백. shared 평균가로 LTH 최고가/보유 기간을 초기화하지 않는다.
- P1-BAL-01: 일반 잔고 조회의 응답 게시도 profile/client/scope와 같은 gate로 검증한다. 설정 transaction이 전체 tracker의 출처를 증명한다고 주장하지 않는다.
- 보유 대상 제거/시장 변경은 먼저 실제 보유/미체결 상태를 확인하고 정리해야 한다. 손실·수익성·거래소 세션 검증 완료를 뜻하지 않는다.
- 재배포·실계좌 검증은 별도 운용 범위다. 기존 사용자 판단/자료 대기 항목은 TODO 색상 구분을 유지한다.
