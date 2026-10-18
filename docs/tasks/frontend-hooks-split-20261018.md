# P3: 도메인별 Query 훅 분리

- 날짜: 2026-10-18 KST
- 기준: `eb8b63d`, `fix/mac-security-storage-2026-10-06`
- 범위: `src/api/hooks.ts`를 기존 공개 API facade와 책임별 모듈로 분리
- 상태: 구현·타입/FSD/build·E2E36개·독립 리뷰·생성물 동기화 완료

| 파일 | 행 | 책임 |
|---|---:|---|
| hooks.ts | 19 | 기존 hook·KEYS·event·database·budget 호환 재수출 |
| accountHooks.ts | 221 | 앱·계좌 설정, 프로파일, 잔고·연결 진단 |
| marketHooks.ts | 255 | 차트·시세·종목·세션·환율 |
| orderHooks.ts | 124 | 주문·사전검증·접수/미체결 조회 |
| strategyHooks.ts | 49 | 전략 설정·일반/LTH 미리보기 |
| settingsHooks.ts | 103 | 공통 갱신·웹·업데이트·rate-limit 설정 |
| historyHooks.ts | 165 | 체결/거래·통계·보관 설정 |
| tradingHooks.ts | 109 | 자동매매 상태·포지션·리스크 제어 |
| logHooks.ts | 74 | 로그 설정·조회·frontend logger |

Serena navigator가 72개 함수와 호출자·내부 의존을 추적했다. 기존 databaseHooks/budgetHooks와 같은 점진 구조이며 전면 FSD 이전을 섞지 않는다. market의 환율 훅 두 개만 settings의 `useRefreshConfig`를 직접 import한다. 새 모듈은 facade를 역참조하지 않는다.

TypeScript AST 원문 대조에서 함수72개와 공개 logger·비공개 해외 차트 preset까지 선언74개가 완전히 일치했다. TypeScript checker의 공개 export 집합87개도 전후 같다. KEYS identity, broker/account scope, polling·retry·options spread, 주문 후 지연/무효화와 이벤트·database·budget API를 보존했다. 캐시나 query key를 새로 복제하지 않았다.

검증: `npx tsc --noEmit`, `npm run check:fsd`, `npm run build:web` 통과, 코드 경고0. 전체 Playwright36개 통과, 독립 AST/API/순환 감사 finding 없음. project_mapper의 책임/인벤토리 갱신과 check 통과, Graphify 강제 재생성/check 및 할일 원본 동기화·Chromium3개 검증도 통과했다. 동작 본문을 베끼는 새 단위 테스트는 추가하지 않았다. 주문·계좌 API·자동매매 시작을 실행하지 않았고 민감 파일을 읽지 않았다.

별도 공개 문서 확인 중 Toss inventory 검증이 기존33경로에서 최신38경로 때문에 실패했다. [공식 v1.2.24 스펙](https://openapi.tossinvest.com/openapi-docs/latest/openapi.json)의 섹터 GET5개를 정확한 예상 집합에 추가하여 재검증을 통과했다. 기존 인증/오류/rate-limit/header 검사와 경로 누락 검사는 유지했다. 새 섹터 기능은 구현하지 않았으며 runtime API 동작 변경도 없다. Toss 원본 스킬의 Dashboard 소액주문 UI 문구도 현재 안내 패널/일반 Trading 주문 경계와 맞췄다.
