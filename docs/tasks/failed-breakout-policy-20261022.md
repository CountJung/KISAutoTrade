# P2-07 돌파 실패: 완료 자료·실제 보유·선택 청산

상태: 완료. baseline `13c5fd3` / replay v18과 현재 v19를 비교했다. 운영 설정을 자동 변경하지 않았으며 실제 계좌·주문·provider·운영 DB·민감 파일에 접근하지 않았다.

## 구현과 완료 조건

- 기존 lookback20/buffer0.5, 현재 제외 가격 최대값, 정수 내림 문턱의 inclusive 돌파와 strict 실패 기준을 유지했다. 실제 수량·원가 피드백으로 보유를 갱신하고 모든 매도는 실제 잔량을 사용한다.
- 다음 시가 매수 직전 수량0 피드백으로 후보 anchor가 지워지던 문제와 접수 대기 중 후속 tick이 최초 anchor를 덮던 문제를 교정했다. 체결을 먼저 반영한 뒤 pending을 해제하며 거절·부분체결·재설정은 실제 보유 보호를 지우지 않는다.
- 이전 완료 일봉 고가·TR14 버퍼·직전 완료 거래량 비율, 수익 무장/관측 되밀림·보유 관측 한도를 기본 비활성 선택값으로 제공했다. 실제 나이/최고가/무장과 v1의 미확인 정보를 구분하며 v2를 엄격히 복원한다.
- `CompletedDailyBar`에 OHLC·Option 거래량을 원자적으로 전달한다. raw 원문에서 정상0·결측/오류를 구분하고 현재 봉/실시간 누적량을 과거 완료 일봉량으로 쓰지 않는다. 기존 훅은 정확히 한 번 전달하고 scope/date/generation·512 상한을 유지한다.
- 저장·두 preview·current/A-B·이전/손상 응답·일봉 전용 진입 자료와 보호 전용 분봉·활성 잠금·저장 중 새 초안을 동기화했다. 최종 UI 설명은 ATR 가격 문턱과 변동폭을 구분한다.

## 생산 비교와 판단

[사전 고정 설계](../research/failed-breakout-policy-20261022/design.json)의 7조건×3ETF×2비용으로 42연속·42분할 대조·504독립 분기 원장을 만들었다. 비대상1,122회 report/signals exact와 대상102회 sameClose 변화0을 확인했으나 nextOpen의 owned 교정은 수익 경로를 바꾸므로 분리했다. [보고서](../research/failed-breakout-policy-20261022/report.md)에 실현/잔여·기말 가정 청산 비용·노출/MDD·회전·무신호/미발동을 기록했다.

거래량 조건은 연속6개 모두 손실이다. 수익 보호 체결은 연속38/분기16, 63관측 한도는 연속28/분기0이며 한도의 분기72개는 owned와 정확히 같다. 보호 조건은 청산·노출을 바꿨지만 연속 수익률이 감소했다. 선택값의 자동 기본 승격을 모두 기각했고 임계값 재조정·그리드·조합은 실행하지 않았다. SOXQ 분기 비교 보유42개는 비중 제한으로 진입0주임을 명시했다.

부모 기록 정산85,073회와 `stock_analyst`의 별도 정산233,251회(546원장·63,168봉), 오류0이다. [독립 감사](../research/failed-breakout-policy-20261022/independent-audit.json)의 OOS/자료 범위·비교 노출 제한을 보존한다. 후속 새 자료는 관측 전에 구간·universe·수량·비용·후보를 고정하고 공급 범위·조정·달력·누락과 정산을 검증해야 한다. P1-05/P2-12 실제 자료·운용 게이트는 미완료 유지다.

## 검증

- Rust 전체718·통합4·doc3, FailedBreakout 집중30 통과. `cargo fmt --all -- --check`, `cargo check --offline --all-targets`, `cargo clippy --offline --all-targets --all-features -- -D warnings` 통과·컴파일/정적 경고0.
- TypeScript·Vite build·FSD·lockfiles 통과. UI 집중21 및 전체238 통과. MUI의 실제 접근성 이름·계산/저장 응답 완료 대기를 테스트에 반영했고 모바일375px 캡처를 확인했다. 모의 fixture의 요청만 사용했다.
- TODO renderer3 통과. graph/map freshness·JSON/HTML/legend 동일(6,187노드/13,575간선/550커뮤니티), 변경 소스1000줄 미만(Page994), 상대 문서 링크·공백·변경 범위·민감/캐시 제외를 점검했다. Graphify의 기존 설정JSON4개 zero-node 진단은 코드 추출기 한계이며 컴파일 경고와 구분한다.
- 사전 `symbol_navigator`의 Serena 영향 추적, `helper_curator`의 Graphify 후보/Serena 동등성 검증을 반영했다. 최종 별도 read-only 감사에서 핵심 전략·완료 자료·driver·collector/manager의 확정 결함은 없었다. 최종 심볼 도구 묶음은 장시간 응답하지 않아 중단했으며 그 호출의 결과는 근거로 사용하지 않았다. reviewer가 추가 추적하지 못한 jobs·order feedback/daemon/fence는 부모가 실제 호출과 게시 순서를 직접 대조했고 UI·문서는 별도 backend worker가 감사했다.
- mapper가 두 지도의 구조/책임을 갱신했고 최종 생성 인벤토리도 재검증했다. 관련 IPC·가이드·아키텍처·Rust/UI 스킬·README·AGENTS·TODO를 갱신했다.

검증 완료 변경을 현재 브랜치에 커밋·origin 푸시한다. 롤백은 해당 변경 커밋의 revert이며 운영 설정을 임의로 바꾸지 않는다.
