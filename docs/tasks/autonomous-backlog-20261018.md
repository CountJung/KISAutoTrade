# 열린 할일 자동 구현: 정확성·준비 상태·사용자 가이드

- 날짜: 2026-10-18 KST
- 요청: 사용자 판단 없이 가능한 열린 항목 구현, 판단·검증 대기 강조/색상 보기
- 시작: `fix/mac-security-storage-2026-10-06`, `694fa14`
- 상태: 이번 묶음 구현·통합 검증·독립 리뷰 완료. 전체 열린 할일의 완료를 뜻하지 않는다.
- 범위: P2-05 generic rolling 고가, P2-16 TR/ATR 준비·연구 UI, LTH 실제 체결 피드백, P3 broker 사용자 가이드, 할일 원본/생성 HTML

## 변경과 안전 경계

1. 52주 신고가: bounded 완료 고가·만료·중복 완료 방지, 보유 손절 보존. live 완료 일봉 연결은 별도 잔여다.
2. ATR: 동일 interval 이전 close TR14, 엄격한 이전 사전자료 seed, 부족/0범위/0bps 매수 차단. 기본 비활성·매도 유지. 거래·손익에 warmup을 포함하지 않는다.
3. LTH: 새 snapshot API로 실제 잔여수량/평균 체결가를 반영, 부분 청산·차단 뒤 보호 상태 유지. 기존 bool API와 동결 연구 runner는 보존.
4. UI: 연구 ATR 설정/준비 상태/legacy 미상 표시. 일봉 LTH 평가 불가 경계와 실제 계정 설정은 유지.
5. 문서: README/사용 가이드를 현재 broker/UI/저장/웹 동작과 맞춤. TODO는 원본 Markdown, HTML은 생성하며 사용자 대기 사유를 분리.

새 IPC 명령·주문·계정·저장 schema 변경 없음. `.env`, `secure_config.json`, `profiles.json`을 읽지 않았다. 실제 주문·계좌 조회·자동매매 시작을 실행하지 않는다.

## 조사와 독립 감사

- symbol_navigator: rolling/ATR call path·live 일봉 부족 경계 추적. LTH 추가 변경은 navigator 역할을 맡은 독립 감사자가 Serena 정의/참조를 확인했다. 동결 runner의 bool API 유지.
- helper_curator: 첫 도구 호출 중단으로 감사 미완료를 보고하여 좁은 범위로 재위임했다. root는 Graphify 후보3개와 Serena 정의/참조를 직접 확인했다: live는 partial window/반올림, LTH는 완료 분봉/f64 보호 청산, replay는 strict14/u128 floor/준비 진단으로 의미가 다르다. 공용 helper 승격은 하지 않고 replay 전용 module을 유지한다. 독립 코드/금융 감사는 별도로 완료했다.
- 독립 감사 1차: 0bps fallback·LTH bool feedback 결함 2건 발견. 명시 차단과 실제 snapshot으로 수정, 회귀 추가.
- 주식 분석: 동일 입력 1,432행 비교. [보고서](../research/autonomous-corrections-20261018/report.md), [차이](../research/autonomous-corrections-20261018/comparison.json).
- project_mapper가 신규 파일/책임 설명을 반영하고 check:project-map을 통과했다. 최종 독립 리뷰에서 코드 finding 추가 없음, 현재 TODO/버전 문구 drift를 수정했다.

## 검증

| 상태 | 검증 | 근거 |
|---|---|---|
| PASS | 초기 전체 Rust 회귀 | 208 lib + 3 integration + 3 doc, 214개 |
| PASS | LTH 실제 snapshot 회귀 | buffer_tests 10개, 확대/축소·부분매도·평균가 포함 |
| PASS | TypeScript/FSD/Web build | warning 0 |
| PASS | focused UI | ATR6 + 기존 평가경계4, 10개 |
| PASS | 최종 전체 Rust/Clippy | 212 lib + 3 integration + 3 doc = 218개, all-target check/Clippy -D warnings 경고0 |
| PASS | 전체 E2E·할일 Chromium | 앱36개 + 할일3개, 로컬 sandbox 포트/브라우저 제한 후 승인된 mock 테스트 |
| PASS | mapper·graph·독립 리뷰 | 신규 파일/문서 반영, 강제 graph 재생성 및 freshness 검사 통과 |
| PASS | 연구 재현 | v8 두 실행 1432행 생성시각 외 완전 동일, 1402행 v7비교 동등/52주30행 차이, 원연구 불변 |

## 잔여 범위

전체 할일은 계속 열린다. live 일봉·provider 페이지, 표본/walk-forward, 연속 replay, next-open·청산 비용, 전략별 고정 실험, 대형 파일 분리, 로컬 TLS/재접속 fixture 등은 자동 구현으로 유지한다. MariaDB 보류 해제·운영 범위 선택·배포 키와 실제 자료/계좌 검증은 사용자 대기 사유를 표시한다. 공학적 난이도나 긴 실행 시간만으로 대기 상태로 바꾸지 않는다.

Graphify의 AST 심볼 없는 기존 JSON 설정 4개 진단은 코드 경고와 구분한다. DB contract lib 테스트는 외부 fixture가 없으면 건너뛰는 기존 계약이며 실제 서버 검증 완료로 계산하지 않는다.
