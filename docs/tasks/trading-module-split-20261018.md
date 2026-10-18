# P3: 자동매매 커맨드 책임 분리

- 날짜: 2026-10-18 KST
- 기준: `191fa93`, `fix/mac-security-storage-2026-10-06`
- 범위: `commands/trading.rs`의 lifecycle/포지션 복원/평가액/리스크 복원/daemon 분리
- 상태: 구현·독립 리뷰·Rust/TypeScript·지도/그래프 동기화 검증 완료
- 목적: 1,000행을 넘는 파일을 분리해 완료 일봉 공급 등의 후속 변경 경계를 명확히 한다.

| 파일 | 행 | 책임 |
|---|---:|---|
| `commands/trading.rs` | 469 | 상태 타입·IPC4개·scope resolver·기존 공개 facade |
| `commands/trading/history.rs` | 281 | 시작 사전자료·기존 live ATR 계산 |
| `commands/trading/positions.rs` | 329 | broker별 보유 복원·수량/가격 변환 |
| `commands/trading/balance.rs` | 96 | KIS/Toss 계좌 평가액 조회 |
| `commands/trading/risk_restore.rs` | 96 | scope별 당일 ledger의 리스크 복원 |
| `commands/trading/daemon.rs` | 527 | 세션·시세 source·tick polling·daemon 실행 |

Serena navigator가 심볼·호출자·Tauri 생성 매크로 경계를 추적했다. IPC4개의 본문·속성·`commands::*` 재수출 경로는 유지했다. `risk_restore_tests.rs`와 `commands.rs`, `lib.rs`는 수정하지 않았다. private helper는 하위 모듈에 한정하고 실제 외부 호출이 있는 재수출만 기존 visibility를 유지한다.

독립 리뷰에서 기존 함수27개와 상태타입3개의 존재·본문·visibility를 확인했다. reconcile → risk 복원 → warmup → 시작 lock 해제 순서, scope 격리, 실패/skip 시 실제 포지션 되먹임을 보존했다. 수정 요구 finding은 없다. broker별 평가액·수량 변환을 통합하거나 live ATR의 부분-window/반올림 정책을 바꾸지 않았다.

검증: `cargo fmt --all -- --check`, `cargo check --workspace --all-targets --locked`, `cargo test --workspace --lib --locked` 212개 통과. Clippy `-D warnings`·TypeScript·FSD·project-map·Graphify refresh/check·할일 HTML 동기화도 통과했다. 코드 경고0이며 Graphify의 기존 심볼 없는 JSON4개 진단과 구분한다. 순수 이동이므로 실행 동작을 그대로 베끼는 새 테스트는 추가하지 않았다. 실계좌/API·주문·자동매매 시작은 실행하지 않았고 민감 파일은 읽지 않았다.

P3의 다른 대형 파일과 P2-05 live 완료 일봉 공급은 계속 열린다. 이 분리만으로 live 고가 갱신을 구현했다고 보고하지 않는다.
