# 작업 기록: <짧은 제목>

> 이 템플릿을 복사해 작업별 문서를 만든다. 체크되지 않은 항목을 삭제하지 말고 `N/A — 이유`로 남긴다. 계좌번호, API key/secret/token, provider 응답 원문 등 민감 정보는 기록하지 않는다.

## 1. 메타데이터

- 작업 ID/이슈:
- 담당자:
- 상태: `Draft | In Progress | Blocked | Review | Done`
- 작성/갱신일:
- 대상 브랜치:
- 관련 문서/PR:
- 위험 등급: `낮음 | 보통 | 높음(주문/리스크/credential/데이터 파괴)`

## 2. 목표와 비목표

### 목표

-

### 비목표

-

### 사용자 완료 조건

- [ ]

## 3. 시작 상태

```text
git status --short:

branch:

관련 기존 변경(보존 대상):

```

- [ ] 사용자 변경과 작업 범위를 구분했다.
- [ ] `.env`, `secure_config.json`, `profiles.json` 등 민감 파일을 읽지 않았다.

## 4. 조사 근거

| 확인한 파일/심볼 | 확인한 사실 | 변경에 미치는 영향 |
|---|---|---|
|  |  |  |

- 공식 KIS/Toss 근거(해당 시 URL/문서명/확인일):
- 가정 또는 미확인 사항:

## 5. 영향 범위

### 예상 변경 파일

| 경로 | 역할 | 변경 이유 |
|---|---|---|
|  |  |  |

### 계약 체크

- [ ] Frontend public API/FSD import
- [ ] Rust serde ↔ TypeScript 타입 미러
- [ ] Tauri command ↔ `lib.rs::generate_handler!`
- [ ] Web REST/transport 지원 여부
- [ ] query key/cache invalidation/backend event
- [ ] persisted JSON/DB schema 및 하위 호환
- [ ] `docs/ipc-commands.md`/프로젝트 지도

## 6. 금융·자동매매 안전 검토

- 실제 주문 가능성: `없음 | 모의 | 실전`
- [ ] 이 작업에서 실전·모의 주문과 자동매매 시작을 실행하지 않는다. 실행이 필요하면 별도 명시적 승인과 절차를 기록한다.
- [ ] `BrokerScope`(broker/profile/account)가 요청→주문→pending→체결→risk에서 보존된다.
- [ ] provider 호출 전에 수량/가격/통화, 매수가능금액/매도가능수량, 동의와 리스크를 검증한다.
- [ ] 중복·반대 방향·동시 submission 충돌을 다룬다.
- [ ] timeout/rate limit/부분·불명확 응답은 fail-closed다.
- [ ] pending/order/risk 영속화 실패 시 신규 주문 차단 정책을 보존한다.
- [ ] 비상정지와 일일 손실/주문/연속 손실 상태가 재시작으로 우회되지 않는다.
- [ ] 로그·trace에 credential/계좌 원문을 남기지 않는다.
- [ ] preview/backtest 한계를 사용자에게 숨기지 않는다.

해당 없음 또는 잔여 위험:

-

## 7. 구현 계획

1.
2.
3.

롤백/비상 중단 방법:

-

## 8. 도구 선택 기록

- Serena: `사용 | 미사용`
  - 공용 타입/IPC/주문/리스크 변경에서 선택 사용한 이유 또는 미사용 이유:
  - 확인한 정의/참조:
- Graphify: `사용 | 미사용`
  - 구조 리팩터링일 때만 사용. 사용한 구조 변경과 질의/갱신:
  - 단순 기능·문서 변경이라 미사용했다면 그 사실:

## 9. 검증 계획과 결과

[`HARNESS_MAP.md`](../HARNESS_MAP.md)에서 범위에 맞게 선택한다.

| 상태 | 명령 | 실행 결과(테스트 수 포함) | 비고 |
|---|---|---|---|
| `PASS/FAIL/NOT RUN` | `npx tsc --noEmit` |  |  |
| `PASS/FAIL/NOT RUN` | `npm run check:fsd` |  |  |
| `PASS/FAIL/NOT RUN` | `cargo check --workspace --all-targets --locked` |  |  |
| `PASS/FAIL/NOT RUN` | `cargo test --workspace --lib --locked` |  |  |
| `PASS/FAIL/NOT RUN` | focused test |  |  |
| `PASS/FAIL/NOT RUN` | focused/전체 Playwright |  |  |
| `PASS/FAIL/NOT RUN` | `npm run check:project-map` |  |  |
| `PASS/FAIL/NOT RUN` | provider/구조 검사 |  |  |

- 실전 주문: `NOT RUN` (기본값)
- 모의 주문: `NOT RUN` (기본값)
- 검증하지 못한 항목과 이유:

## 10. 완료 보고

### 변경 요약

-

### 변경 파일(절대경로)

-

### 기존 사용자 변경 보존 결과

-

### 알려진 제한/잔여 위험

-

### 운영자 확인 필요

- [ ] 자동매매는 정지 상태에서 배포/설정 변경한다.
- [ ] 실전 전 모의·소액 검증은 별도 승인 절차로 수행한다.
- [ ] 비상정지, rollback, 로그/provider trace 확인 경로를 알고 있다.
