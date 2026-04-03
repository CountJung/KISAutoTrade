# 주식 자동 매매 프로젝트 (개인용)

## ⚙️ 에이전트 필수 규칙 (Harness Convention)

> **이 프로젝트에서 AI 에이전트(GitHub Copilot 등)는 반드시 아래 규칙을 따른다.**

1. **모든 작업 시작 전** `agent.md` 파일을 읽어 전체 프로젝트 구조를 파악한다.
2. **코드 변경, 파일 추가/삭제, 구조 변경 후** `agent.md`를 즉시 업데이트한다.
3. `agent.md`는 프로젝트의 "살아있는 구조 맵"이며, 항상 최신 상태를 유지해야 한다.
4. `agent.md`가 없거나 오래된 경우 작업 전에 재생성 또는 갱신한다.
5. 에이전트는 `agent.md`를 참조하지 않고 추측으로 작업해서는 안 된다.

---

### ⚠️ 본 프로젝트는 개인 투자 용도로만 사용하며, 관련 법규를 준수해야 합니다.

- 자동매매는 국가 및 증권사 정책에 따라 제한될 수 있음
- 본 프로젝트는 개인 학습 및 개인 투자 자동화 목적
- 투자 손실에 대한 책임은 사용자에게 있음

---

# 1. 프로젝트 개요

본 프로젝트는 한국투자증권 Open API를 활용하여  
**개인용 자동 주식 매매 시스템**을 구축하는 것을 목표로 한다.

## 구성 목표

- 자동 매매 로직 실행 및 실시간 시세 수신
- 주문 / 체결 / 잔고 관리
- 거래 기록 및 통계를 **JSON 파일 시스템**(연/월/일 분리 폴더)으로 로컬 저장 및 조회
- **Discord 봇**을 통한 심각한 오류 및 중요 이벤트 알림
- UI 기반 설정 및 제어
- 웹 + 데스크탑 환경 동시 지원

---

# 2. 기술 스택

## Frontend

- React 18+
- Tauri v2
- TanStack Query v5
- TanStack Router v1
- MUI (Material-UI v6)
- Zustand (전역 상태 관리)
- Vite 5+

## Backend (Rust)

- Tauri (Core)
- Reqwest (REST API Client)
- Tokio (Asynchronous Runtime)
- tokio-tungstenite (WebSocket 연동)
- Serde / serde_json (직렬화 및 JSON 파일 제어)
- Tracing / tracing-appender (로그 관리)
- chrono (날짜/시간 처리)
- serenity 또는 twilight (Discord 봇 라이브러리)

## API

- 한국투자증권 Open API (REST & WebSocket)
- Discord Bot API (알림 전송)

---

# 3. 아키텍처

```text
React UI (Web / Desktop)
        │
        │ Tauri IPC (Commands / Events)
        ▼
Rust Backend ──────────────────────────┐
  │      │        │                    │
  │      ▼        ▼                    ▼
  │  JSON Storage  Token Manager   Discord Bot
  │  (data/       (Auto Refresh)   (알림 전송)
  │  YYYY/MM/DD/)      │
  ▼                    ▼
한국투자증권 API (REST & WebSocket)

JSON Storage 구조:
  data/
  └── trades/          ← 체결/거래 기록
  │   └── 2025/
  │       └── 04/
  │           └── 02/
  │               └── trades.json
  └── stats/           ← 일별 통계
  │   └── 2025/
  │       └── 04/
  │           └── daily_stats.json
  └── orders/          ← 주문 이력
      └── 2025/
          └── 04/
              └── 02/
                  └── orders.json
```

---

# 4. JSON 데이터 저장 설계

> **DB를 사용하지 않으며, 모든 거래 기록 및 통계는 JSON 파일로 관리한다.**

## 4.1 저장 경로 규칙

```
{app_data_dir}/data/{category}/{YYYY}/{MM}/{DD}/{filename}.json
```

| 카테고리 | 경로 예시 | 설명 |
|---------|----------|------|
| trades | `data/trades/2025/04/02/trades.json` | 체결 기록 |
| orders | `data/orders/2025/04/02/orders.json` | 주문 이력 |
| stats | `data/stats/2025/04/daily_stats.json` | 월별 일간 통계 |
| balance | `data/balance/2025/04/02/snapshot.json` | 잔고 스냅샷 |

## 4.2 거래 기록 스키마 (trades.json)

```json
[
  {
    "id": "uuid-v4",
    "timestamp": "2025-04-02T09:31:05+09:00",
    "symbol": "005930",
    "symbol_name": "삼성전자",
    "side": "buy",
    "quantity": 10,
    "price": 72000,
    "total_amount": 720000,
    "fee": 360,
    "strategy_id": "rsi_cross_v1",
    "order_id": "KIS-ORDER-12345",
    "status": "filled"
  }
]
```

## 4.3 일별 통계 스키마 (daily_stats.json)

```json
{
  "date": "2025-04-02",
  "total_trades": 5,
  "winning_trades": 3,
  "losing_trades": 2,
  "gross_profit": 150000,
  "gross_loss": -40000,
  "net_profit": 108600,
  "fees_paid": 1400,
  "win_rate": 0.6,
  "profit_factor": 3.75,
  "starting_balance": 5000000,
  "ending_balance": 5108600
}
```

## 4.4 조회 방식

- **날짜 범위 조회**: 해당 연/월/일 폴더를 순회하여 JSON 파일을 읽어 집계
- **월간 조회**: `data/trades/{YYYY}/{MM}/` 하위 모든 일별 파일 병합
- **연간 조회**: `data/trades/{YYYY}/` 하위 모든 월 폴더 병합
- Rust에서 `tokio::fs`로 비동기 파일 I/O 처리
- 대용량 조회 시 스트리밍 방식으로 메모리 효율 유지

---

# 5. Discord 봇 알림 설계

## 5.1 알림 개요

> **Discord 웹훅이 아닌 Discord Bot API를 사용하여 채널에 알림을 전송한다.**  
> Discord 연계 설정 방법은 `docs/discord-setup-guide.md`에 상세히 기술한다.

## 5.2 알림 트리거 조건

| 레벨 | 트리거 | 예시 |
|------|--------|------|
| `CRITICAL` | 앱 패닉 / 복구 불가 오류 | Rust panic, 토큰 갱신 완전 실패 |
| `ERROR` | API 오류 반복 / 주문 실패 | 3회 연속 주문 실패, WebSocket 재연결 실패 |
| `WARNING` | 비정상 시장 감지 / 한도 근접 | 일일 손실 한도 80% 도달 |
| `TRADE` | 매수/매도 체결 완료 | 체결 시 즉시 알림 |
| `INFO` | 자동매매 시작/종료 | 장 시작·종료, 전략 ON/OFF |

## 5.3 Discord 메시지 포맷

```
[🔴 CRITICAL] AutoConditionTrade
시각: 2025-04-02 09:31:05 KST
내용: Rust 패닉 발생 - thread 'tokio-runtime-worker'
원인: 토큰 갱신 재시도 초과 (5/5)
조치: 앱을 재시작하거나 API 키를 확인하세요.
```

```
[🟢 TRADE] 체결 완료
종목: 삼성전자 (005930)
방향: 매수
수량: 10주 @ 72,000원
총액: 720,000원
전략: RSI Cross v1
```

## 5.4 Rust 구현 구조

```
src/
└── notifications/
    ├── mod.rs           ← NotificationService trait 정의
    ├── discord.rs       ← Discord Bot 클라이언트 구현
    └── types.rs         ← NotificationLevel, NotificationEvent 타입
```

알림 서비스는 `Arc<dyn NotificationService>` 형태로 주입하여 테스트 시 Mock 으로 교체 가능

## 5.5 알림 설정 (Settings UI)

- Discord Bot Token 입력 (암호화 저장)
- 알림 채널 ID 입력
- 알림 레벨별 ON/OFF 토글
- 테스트 메시지 전송 버튼

---

# 6. 프론트엔드 설계

## 6.1 Theme System

지원 모드

```
light
dark
system
```

### Hydration 방지

초기 렌더링 시 theme mismatch 방지

```
localStorage → theme 먼저 로드
↓
document.documentElement.dataset.theme 적용
↓
React hydration
```

---

## 6.2 Layout System (완전 자유형 UI)

요구사항

- 패널 크기 조절
- 위치 이동
- 내부 grid 조절
- 상태 저장

라이브러리

```
react-grid-layout
```

저장 위치

```
localStorage
```

---

## 6.3 Navigation

TanStack Router 기반

기능

- Breadcrumb 표시
- 이전 페이지 이동
- 상위 경로 이동

예

```
Dashboard > Trading > Strategy > Detail
```

---

## 6.4 주요 화면

### Dashboard

- 계좌 요약
- 현재 수익률 (당일 / 누적)
- 실행중 전략 목록
- 최근 체결 내역

---

### Trading

- 수동 매매
- 주문 실행
- 주문 상태 확인
- 실시간 호가창

---

### Strategy

- 자동 매매 전략 설정
- 전략 ON/OFF
- 전략별 수익률 분석

---

### History (거래 기록)

- 날짜 범위 조회 (캘린더 피커)
- 체결 내역 테이블
- 일/주/월 통계 차트
- JSON 내보내기 (Export)

---

### Log

- 전체 로그 조회
- 필터 / 검색
- 로그 레벨별 색상 구분

---

### Settings

- API 설정 (한국투자증권)
- Discord 봇 설정
- 로그 설정
- 테마 설정
- 알림 레벨 설정

---

# 7. 백엔드 설계

## 7.1 API Client

한국투자증권 REST API 호출

```
GET  /uapi/domestic-stock/v1/trading/inquire-balance   ← 잔고 조회
POST /uapi/domestic-stock/v1/trading/order             ← 주문
GET  /uapi/domestic-stock/v1/trading/inquire-daily-ccld ← 체결 내역 조회
```

Rate Limit 준수: 초당 20건 이하 유지

---

## 7.2 Naming Convention 주의

| 영역 | 규칙 | 예시 |
|------|------|------|
| Frontend (TypeScript) | camelCase | `accountBalance` |
| Backend (Rust) | snake_case | `account_balance` |
| JSON 파일 키 | snake_case | `total_amount` |
| IPC Command | snake_case | `get_balance` |

→ Tauri IPC 경계에서 자동 변환 레이어 필수

---

## 7.3 Trading Engine

```
src/trading/
├── mod.rs          ← TradingEngine 메인 루프
├── strategy.rs     ← Strategy trait 및 구현체
├── order.rs        ← 주문 생성/관리
├── position.rs     ← 포지션 추적
└── risk.rs         ← 리스크 관리 (손실 한도 등)
```

실행 흐름:

```
장 시작 감지
↓
전략 조건 체크 (틱마다)
↓
매매 신호 발생
↓
리스크 체크 (일일 손실 한도 등)
↓
주문 요청 → KIS API
↓
체결 확인 (WebSocket 수신)
↓
JSON 기록 (trades/orders)
↓
Discord 알림 전송
↓
통계 집계 갱신
```

---

## 7.4 Logging System

기본 경로

```
{app_data_dir}/log/
```

### 기본 설정

```
보관 기간: 10일
최대 용량: 100MB
```

### 사용자 설정 가능

- UI: Slider
- 설정 키: `log_retention_days`, `log_max_size_mb`

### 로그 파일 종류

| 파일 | 내용 |
|------|------|
| `app.log` | 일반 앱 이벤트 |
| `trade.log` | 매매 전용 상세 로그 |
| `error.log` | ERROR 이상 오류 |
| `debug.log` | 디버그 상세 (진단 모드 시) |

### 진단 모드 활성화 시

- 모든 API 요청/응답 기록
- 전략 실행 단계별 로그
- UI 이벤트 기록
- Discord 알림 발송 이력

---

# 8. Web + Desktop 분리 설계

## 요구사항

- 동일 코드 기반 (feature flag로 분기)
- 독립 실행 가능

---

## Web 환경

- API 직접 호출 (CORS 설정 필요)
- 파일 시스템 접근 불가 → 조회 기능 제한
- 로그/기록 조회는 읽기 전용

---

## Desktop (Tauri)

- 로컬 파일 시스템 접근 (JSON 저장/로드)
- 백그라운드 자동 매매 실행
- 시스템 시작 시 자동 실행 옵션
- Discord 알림 발송

---

# 9. 보안 설계

## 9.1 .env 관리

| 항목 | 허용 여부 |
|------|----------|
| `VITE_API_URL` | ✅ 프론트 사용 가능 |
| `KIS_APP_KEY` | ❌ Rust Backend 전용 |
| `KIS_APP_SECRET` | ❌ Rust Backend 전용 |
| `DISCORD_BOT_TOKEN` | ❌ Rust Backend 전용 |

---

## 9.2 민감 정보 처리

- Rust Backend에서만 관리
- Frontend로 토큰/시크릿 전달 금지
- Discord Bot Token은 OS 키체인 또는 암호화된 설정 파일에 저장

---

## 9.3 AI Agent 보호

- `.env` 파일 직접 접근 금지
- 민감 정보는 `secure_config.json`에 분리 저장 (git ignore)
- 에이전트는 `agent.md`를 통해서만 구조를 파악

---

## 9.4 .gitignore 필수 항목

```
.env
.env.local
/log
/data
node_modules
dist
target
secure_config.json
discord_config.json
```

---

# 10. 사용자 준비사항

## 10.1 한국투자증권 API 발급

1. 한국투자증권 Open API 신청 (https://apiportal.koreainvestment.com)
2. App Key / App Secret 발급
3. 허용 IP 등록
4. 모의투자 계좌 활성화 확인

---

## 10.2 Discord 봇 설정

> 상세 내용은 `docs/discord-setup-guide.md` 참조

1. Discord Developer Portal에서 봇 생성
2. Bot Token 발급
3. 알림 전용 서버 및 채널 생성
4. 봇을 서버에 초대 (관리자 권한)
5. 채널 ID 확인 후 앱 Settings에 입력

---

## 10.3 환경 설정

`.env` 파일 생성:

```env
VITE_API_URL=http://localhost:1420
KIS_APP_KEY=your_app_key_here
KIS_APP_SECRET=your_app_secret_here
KIS_ACCOUNT_NO=00000000-01
KIS_IS_PAPER_TRADING=true
```

`secure_config.json` 생성 (git ignore 대상):

```json
{
  "discord_bot_token": "your_discord_bot_token",
  "discord_channel_id": "000000000000000000",
  "notification_levels": ["CRITICAL", "ERROR", "TRADE"]
}
```

---

## 10.4 실행

```bash
# 의존성 설치
npm install

# 개발 모드 실행 (Desktop)
npm run tauri dev

# 빌드
npm run tauri build
```

---

# 11. 프로젝트 구조

```
AutoConditionTrade/
├── agent.md                    ← ⚠️ 하네스 맵 (항상 최신 유지)
├── MasterPlan.md               ← 마스터 플랜 (이 문서)
├── docs/
│   ├── discord-setup-guide.md  ← Discord 봇 연계 상세 가이드
│   └── api-reference.md        ← KIS API 참조
├── src/                        ← React Frontend
│   ├── components/
│   │   ├── layout/
│   │   ├── trading/
│   │   ├── charts/
│   │   └── notifications/
│   ├── pages/
│   │   ├── Dashboard.tsx
│   │   ├── Trading.tsx
│   │   ├── Strategy.tsx
│   │   ├── History.tsx
│   │   ├── Log.tsx
│   │   └── Settings.tsx
│   ├── router/
│   ├── store/                  ← Zustand stores
│   └── theme/
├── src-tauri/                  ← Rust Backend
│   ├── src/
│   │   ├── main.rs
│   │   ├── lib.rs
│   │   ├── api/                ← KIS API 클라이언트
│   │   │   ├── mod.rs
│   │   │   ├── rest.rs
│   │   │   ├── websocket.rs
│   │   │   └── token.rs
│   │   ├── trading/            ← 매매 엔진
│   │   │   ├── mod.rs
│   │   │   ├── strategy.rs
│   │   │   ├── order.rs
│   │   │   ├── position.rs
│   │   │   └── risk.rs
│   │   ├── storage/            ← JSON 파일 저장/조회
│   │   │   ├── mod.rs
│   │   │   ├── trade_store.rs
│   │   │   ├── order_store.rs
│   │   │   ├── stats_store.rs
│   │   │   └── balance_store.rs
│   │   ├── notifications/      ← 알림 서비스
│   │   │   ├── mod.rs
│   │   │   ├── discord.rs
│   │   │   └── types.rs
│   │   ├── logging/            ← 로그 관리
│   │   │   └── mod.rs
│   │   └── config/             ← 설정 관리
│   │       └── mod.rs
│   ├── Cargo.toml
│   └── tauri.conf.json
├── data/                       ← JSON 데이터 (git ignore)
│   ├── trades/
│   ├── orders/
│   ├── stats/
│   └── balance/
├── log/                        ← 로그 파일 (git ignore)
├── secure_config.json          ← 민감 설정 (git ignore)
├── .env                        ← 환경변수 (git ignore)
├── package.json
└── vite.config.ts
```

---

# 12. 개발 단계 (Phase)

## Phase 1 — 기반 구성

- [x] Tauri v2 + React 프로젝트 초기화
- [x] Theme 시스템 구현 (light/dark/system)
- [x] TanStack Router 설정 및 기본 라우팅
- [x] MUI 레이아웃 기본 틀
- [x] `agent.md` 최초 생성

---

## Phase 2 — API 연동

- [x] KIS REST API 클라이언트 (Rust)
- [x] 토큰 자동 갱신 로직
- [x] 계좌 조회 (잔고, 보유종목)
- [x] IPC Command 연결 (Tauri ↔ React)

---

## Phase 3 — 매매 기능

- [x] 수동 주문 (매수/매도)
- [x] WebSocket 실시간 시세 수신
- [x] 체결 내역 JSON 저장 (연/월/일 폴더)
- [x] 잔고 스냅샷 저장

---

## Phase 4 — 자동 매매 전략

- [x] Strategy trait 설계
- [x] 기본 전략 구현 (이동평균 골든/데스 크로스)
- [x] 리스크 관리 (일일 손실 한도, 긴급 정지)
- [x] 전략 ON/OFF UI (Strategy.tsx)

---

## Phase 5 — 알림 시스템

- [x] Discord 봇 클라이언트 구현
- [x] 알림 레벨별 메시지 포맷
- [x] Settings UI에서 Discord 설정
- [x] `docs/discord-setup-guide.md` 작성

---

## Phase 6 — 로그 및 통계

- [x] Tracing 로그 시스템 구현
- [x] 일별/월별 통계 집계 및 JSON 저장
- [x] History 화면 (날짜 범위 조회, 통계 요약)
- [x] Log 화면 (레벨 필터, 검색, 색상 구분)

---

## Phase 7 — 최적화 및 배포

- [x] Vite 청크 분리 (vendor/mui/tanstack)
- [ ] 진단 모드 구현
- [ ] 에러 핸들링 강화
- [ ] Tauri 빌드 및 배포
- [ ] `agent.md` 최종 정리

---

# 13. 주의사항

- 자동매매는 증권사 정책에 따라 제한 가능 → 이용약관 반드시 확인
- KIS API 호출 횟수 제한 준수 (초당 20건)
- **실계좌 테스트 전 반드시 모의투자(`KIS_IS_PAPER_TRADING=true`) 충분히 검증**
- JSON 데이터 폴더(`/data`)는 `.gitignore`에 반드시 포함
- Discord Bot Token은 절대 코드에 하드코딩 금지
- `agent.md`는 코드 변경 시마다 즉시 업데이트

---

# 14. 목표

- 개인용 안정적 자동 매매 시스템 구축
- DB 없이 JSON 파일만으로 거래 이력 완전 관리
- Discord 봇을 통한 실시간 거래·오류 알림
- Harness 기법으로 AI 에이전트와 협업 가능한 코드베이스 유지