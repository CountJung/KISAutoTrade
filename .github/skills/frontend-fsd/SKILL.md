---
name: frontend-fsd
description: "React frontend Feature-Sliced Design migration rules for KISAutoTrade. Use when changing files under src/, moving frontend modules, adding shared UI/API/state code, splitting pages/components, or reviewing imports between app/pages/widgets/features/entities/shared layers."
---

# Frontend FSD

이 스킬은 React 프론트엔드 `src/`를 Feature-Sliced Design(FSD) 방향으로 점진 정리할 때 사용한다. Rust/Tauri 백엔드(`src-tauri/src/**`)는 이미 도메인별 분리가 되어 있으므로 이 스킬의 구조 이동 대상이 아니다.

## 기본 원칙

- 기능 변경과 구조 이동을 같은 커밋에 크게 섞지 않는다. 동작 수정이 우선이면 현재 위치에서 고치고, 후속 작은 변경으로 이동한다.
- 건드린 프론트엔드 파일은 가능한 범위에서 FSD 경계에 맞춘다. 전체 앱 일괄 이동은 사용자가 명시한 경우에만 한다.
- 이동 전후 public API, 타입, TanStack Query key, Zustand storage key, Tauri IPC command name을 유지한다.
- 경로 변경 후 `npx tsc --noEmit`으로 import와 타입을 확인한다. Rust IPC를 함께 변경한 경우 `cd src-tauri; cargo check`도 실행한다.
- UI 변경은 `ui-conventions`를 함께 읽고, 성능/쿼리/번들 변경은 `react-best-practices`를 함께 읽는다.

## 레이어

허용 의존 방향:

```
app -> pages -> widgets -> features -> entities -> shared
```

하위 레이어는 상위 레이어를 import하지 않는다.

| 레이어 | 책임 | 예시 |
|--------|------|------|
| `app` | 앱 부트스트랩, providers, router wiring | `main.tsx`, router/provider 조립 |
| `pages` | 라우트 단위 조립만 담당 | `dashboard`, `trading`, `strategy`, `history`, `log`, `settings` |
| `widgets` | 여러 feature/entity를 조합한 큰 UI 블록 | `app-shell`, `sidebar`, `stock-chart`, `account-summary`, `strategy-list`, `log-viewer` |
| `features` | 사용자의 행동/유스케이스 | `manual-order`, `symbol-search`, `strategy-toggle`, `strategy-configure`, `trading-start-stop`, `log-filter`, `discord-notification-config` |
| `entities` | 도메인 명사와 상태/타입 | `account`, `stock`, `order`, `trade`, `position`, `strategy`, `settings`, `log` |
| `shared` | 도메인 무관 공통 코드 | `api`, `lib`, `ui`, `config`, `theme` |

## 초기 매핑

기존 파일을 옮길 때는 아래 후보를 우선 검토한다.

| 현재 위치 | 우선 후보 |
|-----------|-----------|
| `src/api/commands.ts`, `src/api/transport.ts` | `src/shared/api/` |
| `src/api/types.ts`의 도메인 타입 | `src/entities/*/model/` 또는 `src/entities/*/types.ts` |
| `src/api/hooks.ts`의 도메인별 Query 훅 | 관련 `entities/*/api/` 또는 행동 중심 `features/*/api/` |
| `src/theme/**` | `src/shared/config/theme` 또는 `src/shared/ui/theme` |
| `src/store/accountStore.ts` | `src/entities/account/model/` |
| `src/store/settingsStore.ts` | `src/entities/settings/model/` |
| `src/components/layout/AppShell.tsx` | `src/widgets/app-shell/` |
| `src/components/layout/Sidebar.tsx` | `src/widgets/sidebar/` |
| `src/components/chart/*StockChart.tsx` | `src/widgets/stock-chart/` |
| `src/components/LayoutResizer.tsx`, `ResizableDialog.tsx` | `src/shared/ui/` |
| `src/pages/*.tsx` | `src/pages/{route}/ui/Page.tsx` 형태로 얇게 유지 |

## Slice 규칙

- slice 내부는 필요할 때 `ui`, `model`, `api`, `lib`, `config` 세그먼트를 사용한다.
- slice 외부에서 내부 파일을 깊게 import하지 않도록 `index.ts`를 public API로 둔다.
- `shared`에는 비즈니스 도메인 이름과 전략별 의사결정을 넣지 않는다.
- `entities`는 도메인 데이터, 타입, store, query를 담되 사용자 액션 orchestration은 `features`로 올린다.
- `features`는 `entities`와 `shared`만 직접 의존한다. 다른 feature를 직접 import해야 한다면 상위 `widgets`에서 조립한다.
- `pages`는 데이터 가공과 상태 전이를 최소화하고 widgets/features를 배치한다.

## 작업 순서

1. 변경하려는 파일의 현재 import 그래프를 확인한다.
2. 동작 변경이 필요한 경우 먼저 작은 단위로 고친다.
3. 같은 작업 범위에서 이동해도 위험이 낮은 공통 코드만 FSD 위치로 옮긴다.
4. 새 위치의 `index.ts`에서 public API를 export한다.
5. 기존 import를 새 public API로 갱신한다.
6. `npx tsc --noEmit`을 실행하고, 실패한 import를 모두 수정한다.
7. 구조가 바뀐 경우 `docs/project-map.md`와 필요 시 `todo.md`를 갱신한다.

## 경계 검증

ESLint나 별도 스크립트를 추가하기 전까지는 수동으로 아래를 확인한다.

- `shared/**`가 `entities`, `features`, `widgets`, `pages`를 import하지 않는다.
- `entities/**`가 `features`, `widgets`, `pages`를 import하지 않는다.
- `features/**`가 `widgets`, `pages` 또는 다른 feature를 직접 import하지 않는다.
- `pages/**`가 너무 두꺼워지면 해당 로직을 `widgets` 또는 `features`로 내린다.
- Tauri IPC wrapper는 command name과 request/response 타입이 Rust와 계속 일치한다.

## 완료 기준

- 변경 범위가 FSD 방향과 충돌하지 않는다.
- TypeScript 타입 체크가 통과한다.
- UI/React 성능 규칙을 건드린 경우 관련 스킬 문서의 규칙을 지켰다.
- 구조 이동이 있었다면 문서와 todo의 현재 상태가 실제 코드와 맞다.

## 현재 점진 구조

- `src/shared/api`가 Tauri IPC/Web REST wrapper와 Rust 타입 미러의 기준 위치다. 기존 `src/api/{commands,transport,types}.ts`는 호환 re-export로 유지한다.
- `src/shared/config/{theme,scheduler}`와 `src/shared/ui`를 공통 설정/UI 기준 위치로 사용한다.
- Zustand store는 `src/entities/{account,settings,trading}/model`로 이동했고 기존 `src/store/*`는 호환 re-export다.
- AppShell, Sidebar, 국내/해외 StockChart는 `src/widgets/*` public API를 통해 사용한다.
- 라우트 페이지는 `src/pages/{route}/ui/Page.tsx`와 `src/pages/{route}/index.ts` 구조를 사용한다.
- FSD 경계 검증은 `npm run check:fsd`로 실행한다. 스크립트는 `scripts/check-fsd-imports.mjs`에 있으며 `shared → entities → features → widgets → pages → app` 역방향 import를 실패 처리한다.

> 마지막 업데이트: 2026-07-01T17:35:00
