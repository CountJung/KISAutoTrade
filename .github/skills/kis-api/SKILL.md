---
name: kis-api
description: "KIS(한국투자증권) Open API 연동 스킬. 인증/토큰 발급, REST API 호출 패턴, WebSocket 실시간 시세, TR-ID 목록, 에러코드, 계좌번호 분리, 모의/실전 환경 전환. Keywords: KIS, 한국투자증권, Open API, token, REST, WebSocket, TR-ID, 계좌, 잔고, 주문, 체결, 시세, 모의투자, EGW"
---
# KIS Open API 연동 스킬

> 참조: [KIS Developers 포털](https://apiportal.koreainvestment.com/) · [Open Trading API 샘플](https://github.com/koreainvestment/open-trading-api) · [KIS AI Extensions](https://github.com/koreainvestment/kis-ai-extensions)

---

## 1. 환경 설정

### Base URL

| 환경 | REST Base URL | WebSocket URL |
|------|--------------|---------------|
| 실전투자 | `https://openapi.koreainvestment.com:9443` | `wss://openapi.koreainvestment.com:9443/websocket/client` |
| 모의투자 | `https://openapivts.koreainvestment.com:29443` | `wss://openapivts.koreainvestment.com:29443/websocket/client` |

### 계좌번호 형식

계좌번호 10자리를 반드시 분리합니다:

```rust
// ✅ 올바른 분리 (Rust)
fn split_account(account_no: &str) -> (&str, &str) {
    // "12345678-01" 또는 "1234567801" 모두 처리
    let clean = account_no.replace('-', "");
    (&clean[..8], &clean[8..])  // (CANO, ACNT_PRDT_CD)
}

// ACNT_PRDT_CD 값 참조
// "01" → 종합계좌
// "03" → 국내선물옵션
// "08" → 해외선물옵션
// "22" → 개인연금
// "29" → 퇴직연금
```

### API 호출 제한

| 환경 | 초당 제한 | 대응 방법 |
|------|---------|---------|
| 실전투자 | 20건/초 | `tokio::time::sleep(Duration::from_millis(55))` |
| 모의투자 | 2건/초 | `tokio::time::sleep(Duration::from_millis(550))` |

에러코드 `EGW00201` = 초당 거래건수 초과

---

## 2. 인증 (Access Token)

### 토큰 발급

```
POST {base_url}/oauth2/tokenP
Content-Type: application/json

{
  "grant_type": "client_credentials",
  "appkey": "...",
  "appsecret": "..."
}
```

**응답:**
```json
{
  "access_token": "eyJ...",
  "token_type": "Bearer",
  "expires_in": 86400,
  "access_token_token_expired": "2026-04-03 09:00:00"
}
```

**주의:**
- 토큰 유효기간: 24시간
- 1분당 1회 발급 제한 → 만료 5분 전에 갱신
- `is_expired()`: `expires_at - Utc::now() < Duration::minutes(5)`

### Rust 인증 헤더 패턴

```rust
// 모든 REST 요청에 공통 헤더 추가
async fn auth_headers(&self, tr_id: &str) -> Result<HeaderMap> {
    let token = self.token_manager.get_token().await?;
    let mut headers = HeaderMap::new();
    headers.insert("authorization", format!("Bearer {token}").parse()?);
    headers.insert("appkey", self.app_key.parse()?);
    headers.insert("appsecret", self.app_secret.parse()?);
    headers.insert("tr_id", tr_id.parse()?);
    headers.insert("content-type", "application/json".parse()?);
    Ok(headers)
}
```

---

## 3. 주요 REST API

### 잔고 조회

```
GET /uapi/domestic-stock/v1/trading/inquire-balance
```

| 파라미터 | 값 |
|---------|---|
| `CANO` | 계좌번호 앞 8자리 |
| `ACNT_PRDT_CD` | 뒤 2자리 |
| `AFHR_FLPR_YN` | `N` |
| `OFL_YN` | `""` |
| `INQR_DVSN` | `02` |
| `UNPR_DVSN` | `01` |
| `FUND_STTL_ICLD_YN` | `N` |
| `FNCG_AMT_AUTO_RDPT_YN` | `N` |
| `PRCS_DVSN` | `01` |
| `CTX_AREA_FK100` | `""` |
| `CTX_AREA_NK100` | `""` |

| TR-ID | 환경 |
|-------|------|
| `TTTC8434R` | 실전투자 |
| `VTTC8434R` | 모의투자 |

### 주식 현재가 조회

```
GET /uapi/domestic-stock/v1/quotations/inquire-price
```

| 파라미터 | 값 |
|---------|---|
| `FID_COND_MRKT_DIV_CODE` | `J` (주식/ETF) |
| `FID_INPUT_ISCD` | 종목코드 6자리 |

| TR-ID | 환경 |
|-------|------|
| `FHKST01010100` | 실전/모의 공통 |

### 주문 (매수/매도)

```
POST /uapi/domestic-stock/v1/trading/order-cash
Content-Type: application/json

{
  "CANO": "12345678",
  "ACNT_PRDT_CD": "01",
  "PDNO": "005930",
  "ORD_DVSN": "00",
  "ORD_QTY": "10",
  "ORD_UNPR": "75000"
}
```

| 주문유형 `ORD_DVSN` | 설명 |
|--------------------|------|
| `"00"` | 지정가 |
| `"01"` | 시장가 |

| TR-ID | 환경 | 방향 |
|-------|------|------|
| `TTTC0802U` | 실전 | 매수 |
| `TTTC0801U` | 실전 | 매도 |
| `VTTC0802U` | 모의 | 매수 |
| `VTTC0801U` | 모의 | 매도 |

### 당일/기간별 체결 내역

```
GET /uapi/domestic-stock/v1/trading/inquire-daily-ccld
```

| TR-ID | 환경 |
|-------|------|
| `TTTC8001R` | 실전투자 |
| `VTTC8001R` | 모의투자 |

| 주요 파라미터 | 값 |
|------------|---|
| `INQR_STRT_DT` | 시작 날짜 `YYYYMMDD` |
| `INQR_END_DT` | 종료 날짜 `YYYYMMDD` |
| `SLL_BUY_DVSN_CD` | `00` (전체), `01` (매도), `02` (매수) |
| `INQR_DVSN` | `00` (역순) |
| `PDNO` | `""` (전체 종목) |
| `CCLD_DVSN` | `00` (전체), `01` (체결), `02` (미체결) |
| `INQR_DVSN_3` | `00` (전체) |

#### ❌ 잘못된 패턴
`CCLD_DVSN: "01"` (체결만) 사용 시 모의투자 환경에서 주문 자체가 조회 안 될 수 있음.  
수동/자동 체결 모두 포함하려면 `"00"` (전체) 사용.

#### ✅ 올바른 패턴
```rust
("CCLD_DVSN", "00"), // 00=전체(체결+미체결), 01=체결, 02=미체결
```

---

## 4. WebSocket 실시간 시세

### 접속키 발급 (승인 요청)

WebSocket 연결 후 첫 메시지로 승인키를 요청합니다:

```json
{
  "header": {
    "approval_key": "{ws_approval_key}",
    "custtype": "P",
    "tr_type": "1",
    "content-type": "utf-8"
  },
  "body": {
    "input": {
      "tr_id": "H0STCNT0",
      "tr_key": "005930"
    }
  }
}
```

WebSocket 접속키 발급:
```
POST {base_url}/oauth2/Approval
{ "grant_type": "client_credentials", "appkey": "...", "secretkey": "..." }
```

### 실시간 체결가 수신 (H0STCNT0)

수신 메시지 형식: `0|H0STCNT0|001|{필드^구분된 데이터}`

```rust
fn parse_realtime_price(text: &str) -> Option<RealtimePrice> {
    // "0|H0STCNT0|001|005930^75000^500^0.67^..."
    let parts: Vec<&str> = text.splitn(4, '|').collect();
    if parts.len() < 4 || parts[1] != "H0STCNT0" { return None; }
    
    let fields: Vec<&str> = parts[3].split('^').collect();
    Some(RealtimePrice {
        symbol: fields[0].to_string(),          // 종목코드
        price: fields[2].parse().ok()?,          // 현재가
        change: fields[4].parse().ok()?,         // 전일 대비
        change_rate: fields[5].parse().ok()?,    // 등락률
        volume: fields[14].parse().ok()?,        // 누적 거래량
        trade_time: fields[1].to_string(),       // 체결시간 (HHmmss)
    })
}
```

### TR-ID 구독 종류

| TR-ID | 데이터 | 설명 |
|-------|--------|------|
| `H0STCNT0` | 체결가 (실전) | 실시간 현재가 |
| `H0STCNS0` | 체결가 (모의) | 모의투자 현재가 |
| `H0STASP0` | 호가 | 매수/매도 호가 |
| `H0STCNI0` | 체결통보 | 내 주문 체결 알림 |

---

## 5. 에러코드 처리

### 주요 에러코드

| 코드 | 설명 | 대응 |
|------|------|------|
| `EGW00201` | 초당 거래건수 초과 | sleep 후 재시도 |
| `OPSP00002` | 유효하지 않은 토큰 | 토큰 재발급 |
| `OPSQ00002` | 앱키 오류 | `appkey` 확인 |
| `40600000` | 비정상 접근 | API 키 확인 |
| `OPSQ00001` | 계좌번호 오류 | CANO/ACNT_PRDT_CD 확인 |

### Rust 에러 처리 패턴

```rust
// KIS API 에러 응답 구조
#[derive(Deserialize)]
struct KisErrorBody {
    rt_cd: String,    // "1" = 에러, "0" = 정상
    msg_cd: String,   // 에러코드
    msg1: String,     // 에러 메시지
}

// 응답 검증
fn check_response(body: &KisErrorBody) -> Result<(), ApiError> {
    if body.rt_cd != "0" {
        return Err(ApiError::KisApi {
            code: body.msg_cd.clone(),
            message: body.msg1.clone(),
        });
    }
    Ok(())
}
```

---

## 6. 이 프로젝트의 구현 위치

| 기능 | 파일 |
|------|------|
| Token 발급/갱신 | `src-tauri/src/api/token.rs` |
| REST 클라이언트 | `src-tauri/src/api/rest.rs` |
| WebSocket 연결 | `src-tauri/src/api/websocket.rs` |
| IPC 커맨드 | `src-tauri/src/commands.rs` |
| TS 타입 정의 | `src/api/types.ts` |
| IPC 래퍼 | `src/api/commands.ts` |
| React 훅 | `src/api/hooks.ts` |

### 프로젝트 내 TR-ID 사용 현황

```rust
// rest.rs — is_paper 플래그로 자동 선택
let tr_id = match (is_paper, side) {
    (false, OrderSide::Buy)  => "TTTC0802U",
    (false, OrderSide::Sell) => "TTTC0801U",
    (true,  OrderSide::Buy)  => "VTTC0802U",
    (true,  OrderSide::Sell) => "VTTC0801U",
};
```

---

## 7. 보안 주의사항

- `appkey`, `appsecret`은 반드시 `secure_config.json` 또는 `.env`에만 저장
- `access_token`은 로그에 출력 금지
- 실전 주문 전 `is_paper_trading` 플래그 확인 필수
- WebSocket 메시지에서 계좌번호 마스킹 처리

---

## 8. 전략 및 신호 참조 (KIS AI Extensions)

KIS AI Extensions의 10개 프리셋 전략 중 이 프로젝트 구현 현황:

| 전략 | 상태 | 구현 위치 |
|------|------|---------|
| 01 골든크로스 (이동평균 교차) | ✅ 구현 | `src-tauri/src/trading/strategy.rs` |
| 02 모멘텀 | 미구현 | - |
| 05 이격도 | 미구현 | - |
| 09 평균회귀 | 미구현 | - |

추가 전략 구현 시 `Strategy` trait과 `StrategyManager`를 참조합니다:
```rust
// src-tauri/src/trading/strategy.rs
pub trait Strategy: Send + Sync {
    fn on_tick(&mut self, symbol: &str, price: u64) -> Signal;
}
```

---

> 참조 링크:
> - [KIS Developers 포털](https://apiportal.koreainvestment.com/)
> - [Open Trading API 샘플](https://github.com/koreainvestment/open-trading-api)
> - [KIS AI Extensions](https://github.com/koreainvestment/kis-ai-extensions)

---

## 9. 국내 종목 검색 (KRX 차단 → NAVER 폴백)

### ❌ 잘못된 패턴 — KRX data.krx.co.kr AJAX 직접 호출

```rust
// ❌ 브라우저 없이 AJAX 직접 호출 → WAF "LOGOUT" 응답 반환
// data.krx.co.kr은 2024년 이후 세션/JS 없는 봇 접근 차단됨
let resp = client
    .post("https://data.krx.co.kr/comm/bldAttendant/getJsonData.cmd")
    .body("bld=dbms%2FMDC%2FSTAT%2Fstandard%2FMDCSTAT01901&...")
    .send().await?;
// → 응답이 "LOGOUT" 텍스트 → JSON 파싱 실패 → 0개
```

### ✅ 올바른 패턴 — NAVER Finance 자동완성 API 실시간 폴백

```rust
// ✅ KRX 캐시 없을 때 NAVER Finance ac.stock.naver.com으로 폴백
// - 인증 불필요, 브라우저 없이도 접근 가능
// - reqwest .query()로 한글 자동 URL 인코딩

let resp = client
    .get("https://ac.stock.naver.com/ac")
    .query(&[("query", query), ("target", "stock,etf"), ("source", "domestic")])
    .header("Referer", "https://finance.naver.com/")
    .send()
    .await?;

// 응답: { "query": "...", "items": [{"code":"005930","name":"삼성전자",...}] }
```

### 아키텍처 결정

| 상황 | 동작 |
|------|------|
| KRX 캐시 있음 (24h 이내) | 로컬 즉시 검색 |
| KRX 캐시 없음 | NAVER 실시간 검색 자동 폴백 |
| NAVER도 실패 | `STOCK_LIST_EMPTY` 에러 + UI 경고 |
| `refresh_stock_list` 실행 | KRX 시도 → 0개면 `KRX_EMPTY` 에러 (검색은 이미 NAVER로 동작) |

### ❌ 문제: KRX/NAVER 종목 이름 검색 불안정

| 방법 | 상태 | 원인 |
|------|------|------|
| `data.krx.co.kr` | ❌ 차단 | WAF 봇 차단 |
| `ac.finance.naver.com` | ❌ DNS 없음 | 도메인 폐지 |
| `ac.stock.naver.com` | ⚠️ 항상 빈 결과 | API 스펙 변경됨 |

### ✅ 해결: Yahoo Finance 코드→이름 조회 (종목코드 6자리 전용)

```
GET https://query1.finance.yahoo.com/v1/finance/search
    ?q=005930.KS&lang=ko&region=KR&quotesCount=1&newsCount=0&listsCount=0
```

응답 `quotes[0].longname` = `"삼성전자(주)"` (한글 정식명)

- **한글 이름 검색은 지원하지 않음** (Error "Invalid Search Query")
- **6자리 코드 + `.KS` 접미사로만 사용 가능**
- API 키 불필요, 별도 인증 없음

```rust
// 구현 위치: src-tauri/src/market/mod.rs
pub async fn lookup_name_by_code(code: &str) -> Result<String> {
    let symbol = format!("{}.KS", code);
    // GET query1.finance.yahoo.com/v1/finance/search?q={symbol}...
    // quotes[0].longname 반환
}
```

`search_stock` IPC 6자리 코드 처리 순서:
1. StockStore 캐시 확인 (O(1))
2. KIS `get_price` (인증 필요, 이름 포함)
3. Yahoo Finance `lookup_name_by_code` (인증 불필요)
4. 실패 시 빈 배열

---

## KRX 종목코드 패턴 (중요)

### ✅ 올바른 패턴

KRX에 상장된 종목의 코드는 **6자리 영숫자**이며, 알파벳이 포함될 수 있다.

| 종류 | 코드 예시 | 패턴 |
|------|----------|------|
| 일반 주식 (KOSPI/KOSDAQ) | `005930`, `035720` | 6자리 숫자 |
| ETF (커버드콜 등) | `0005A0`, `0089C0` | 6자리, 대문자 알파벳 포함 가능 |
| ETN | `580006` | 6자리 숫자 |

```typescript
// ✅ 올바른 검증 (6자리 영숫자)
/^[A-Z0-9]{6}$/i.test(code)

// ❌ 잘못된 검증 (숫자만)
/^\d{6}$/.test(code)  // 0005A0, 0089C0 등 ETF 코드를 거부함
```

```rust
// ✅ 올바른 Rust 검증
fn is_valid_krx_code(code: &str) -> bool {
    code.len() == 6 && code.chars().all(|c| c.is_ascii_alphanumeric())
}
// ❌ 잘못된 Rust 검증
code.chars().all(|c| c.is_ascii_digit())
```

### ❌ 잘못된 패턴 (실제 발생한 사례)

**이전 세션에서의 오류**: 사용자가 "KRX 종목코드에 알파벳이 들어갈 수 있다"고 주장했을 때,  
"KRX는 6자리 숫자만 사용합니다"로 기각함.

→ 실제 확인 결과: `KODEX 미국S&P500데일리커버드콜OTM`의 코드는 `0005A0`,  
`KODEX 미국S&P500변동성확대시커버드콜`의 코드는 `0089C0` (삼성자산운용 공식 사이트 확인).

**교훈**: 사용자가 사실을 주장할 때 먼저 외부 소스나 실제 데이터로 검증한 후 판단한다.

---

### 구현 위치

- `src-tauri/src/market/mod.rs` — `lookup_name_by_code()`, `search_naver_live()`, `StockList::fetch_from_krx()`
- `src-tauri/src/commands.rs` — `search_stock`: Yahoo 폴백 로직, `refresh_stock_list`: KRX_EMPTY 에러 처리

---

## 10. IPC 에러 표시 (CmdError → JS)

### ❌ 잘못된 패턴 — `String(e)` 그대로 사용

```typescript
// ❌ Tauri v2는 Rust Err를 JSON 객체로 throw → String(e) = "[object Object]"
onError: (e) => setErrorMsg(String(e))
```

### ✅ 올바른 패턴 — CmdError.message 추출

```typescript
// ✅ CmdError { code, message } 에서 message 필드 추출
onError: (e) => {
  const err = e as { message?: string } | Error | null
  setErrorMsg(err instanceof Error ? err.message : (err as { message?: string })?.message ?? String(e))
}
```

### 원인
- Tauri v2에서 `Result<T, CmdError>`의 Err를 반환하면 JS 측에서 `{ code: "ERROR", message: "..." }` 형태의 plain object가 throw됨
- `Error` 인스턴스가 아니므로 `String(e)` → `[object Object]`
- 해외 주문(`place_overseas_order`) 등 KIS API 오류 발생 시 증상 나타남

---

## 11. 자동매매 폴링 루프 설계 (핵심 패턴)

### ❌ 잘못된 패턴 — WebSocket broadcast 수신자 즉시 드롭

```rust
// ❌ receiver(_)를 드롭하면 아무도 메시지를 받지 못함
let (price_tx, _) = broadcast::channel(256);
// price_tx.send(event) → 수신자 없음 → 모든 틱 유실 → 전략 한 번도 실행 안 됨
```

### ✅ 올바른 패턴 — 폴링 루프로 독립 동작

`start_trading()` 에서 WebSocket과 **별도로** 폴링 루프를 spawn한다.

```rust
// ✅ 폴링 루프: 10초마다 가격 조회 → on_tick → submit_signal
let is_trading = Arc::clone(&state.is_trading);
let strategy_mgr = Arc::clone(&state.strategy_manager);
let order_mgr = Arc::clone(&state.order_manager);
let rest_arc = Arc::clone(&state.rest_client);

tauri::async_runtime::spawn(async move {
    loop {
        if !*is_trading.lock().await { break; }

        let symbols = strategy_mgr.lock().await.active_symbols();
        let rest = rest_arc.read().await.clone();

        for symbol in &symbols {
            let tick = if is_domestic_symbol(symbol) {
                rest.get_price(symbol).await
                    .map(|p| (p.stck_prpr.parse::<u64>().unwrap_or(0),
                              p.acml_vol.parse::<u64>().unwrap_or(0)))
                    .map_err(|e| e.to_string())
            } else {
                fetch_overseas_tick(&rest, symbol).await.map_err(|e| e.to_string())
            };

            if let Ok((price, volume)) = tick {
                let signals = strategy_mgr.lock().await.on_tick(symbol, price, volume);
                for signal in signals {
                    let _ = order_mgr.lock().await.submit_signal(signal, symbol, 0).await;
                }
            }
            // rate-limit: 모의 600ms, 실전 120ms
        }

        // 10초 대기 (100ms × 100 슬라이스 — 종료 즉시 반응)
        for _ in 0u32..100 {
            if !*is_trading.lock().await { break; }
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    }
});
```

### 해외 종목 가격 단위 변환

- `get_overseas_price()` 반환값 `last` = USD float 문자열 (예: `"78.72"`)
- `on_tick(price: u64)` 는 정수 필요 → `(price_f * 100.0).round() as u64` (센트 단위)
- `initialize_historical()` 에도 동일 변환 적용 (일관성 유지)

### 국내/해외 종목 자동 판별

```rust
fn is_domestic_symbol(symbol: &str) -> bool {
    // KRX 종목코드: 6자리, 첫 글자가 숫자 (예: 005930, 0005A0)
    symbol.len() == 6 && symbol.chars().next().map_or(false, |c| c.is_ascii_digit())
}
```

### 일별 초기화

폴링 루프 내에서 매 반복 시 날짜 변경 감지:

```rust
let mut last_reset_date = chrono::Local::now().date_naive();
// ...루프 내...
let today = chrono::Local::now().date_naive();
if today != last_reset_date {
    last_reset_date = today;
    risk_mgr.lock().await.reset_if_new_day();
    order_mgr.lock().await.reset_day();
}
```

### 장 마감 / 장외 시간 자동 대기

KIS API 응답 메시지에서 장 마감 키워드를 감지하면 5분 대기 후 재시도.
`WARN` 로그가 누적되는 것을 방지하고 불필요한 API 호출을 차단한다.

```rust
/// KIS API 장외 시간 응답 감지 (실제 수집된 메시지 패턴)
fn is_market_closed_error(msg: &str) -> bool {
    msg.contains("장종료")
        || msg.contains("장마감")
        || msg.contains("장시작전")
        || msg.contains("장운영시간")
        || msg.contains("시간외거래")
        || msg.contains("OPCODE-100")
}
```

폴링 루프에서의 처리 패턴:

```rust
let mut market_pause_until: Option<tokio::time::Instant> = None;

'main_loop: loop {
    // 대기 중이면 30초 슬립 후 재진입
    if let Some(until) = market_pause_until {
        if tokio::time::Instant::now() < until {
            tokio::time::sleep(Duration::from_secs(30)).await;
            continue 'main_loop;
        }
        tracing::info!("장 마감 대기 완료 — 폴링 재개");
        market_pause_until = None;
    }

    'symbol_loop: for symbol in &symbols {
        match fetch_tick(&rest, symbol).await {
            Err(e) if is_market_closed_error(&e) => {
                tracing::info!("장 마감/장외 감지 ({}): {} — 5분 대기", symbol, e);
                market_pause_until = Some(
                    tokio::time::Instant::now() + Duration::from_secs(300)
                );
                break 'symbol_loop; // 나머지 종목 건너뜀
            }
            Err(e) => tracing::warn!("현재가 조회 실패 ({}): {}", symbol, e),
            Ok(_) => { /* 신호 처리 */ }
        }
    }
    // 장 마감 감지 시 즉시 대기 루프로 진입
    if market_pause_until.is_some() { continue 'main_loop; }
    // 정상 10초 대기...
}
```

> **핵심 규칙**: 장 마감 감지 시 `WARN` 대신 `INFO`로 한 번만 기록하고, 이후 대기 기간 동안은 30초 sleep → continue 패턴으로 API 호출을 완전히 차단.

---

## 12. 체결 내역 조회 주의사항

### 국내 vs 해외 체결 조회 API 분리

`TTTC8001R` / `VTTC8001R` (`inquire-daily-ccld`)은 **국내 주식 전용**이다.  
해외 주식(NASDAQ/NYSE/AMEX)은 이 API로 조회되지 않는다.

| 대상 | TR-ID (실전/모의) | Endpoint |
|------|-----------------|----------|
| 국내 체결 | `TTTC8001R` / `VTTC8001R` | `/uapi/domestic-stock/v1/trading/inquire-daily-ccld` |
| 해외 체결 | `TTTS3035R` / `VTTS3035R` | `/uapi/overseas-stock/v1/trading/inquire-ccnl` |

### CCLD_DVSN 파라미터

```
"00" = 전체(체결 + 미체결) ← 미체결 주문도 포함되어 체결 내역 탭에서 혼동 유발
"01" = 체결만              ← History 조회 시 올바른 값
"02" = 미체결만
```

✅ **올바른 설정**: `("CCLD_DVSN", "01")` — 체결 내역 조회 시 반드시 사용

### 모의투자 환경에서 odno(주문번호) 빈 문자열 반환

KIS 모의투자 환경에서 `place_order()` 성공 시 `odno`가 빈 문자열 `""` 로 반환될 수 있다.  
이 경우 pending map의 키가 충돌하여 모든 미체결 주문이 하나로 덮어써진다.

❌ **잘못된 패턴**:
```rust
// ondo가 ""면 모든 주문이 같은 키 → 덮어쓰기
self.pending.insert(response.ondo, PendingOrder { ... });
```

✅ **올바른 패턴**:
```rust
let ondo = if response.ondo.is_empty() {
    format!("LOCAL-{}", uuid::Uuid::new_v4())  // 로컬 UUID로 대체
} else {
    response.ondo
};
self.pending.insert(ondo, PendingOrder { ... });
```

---

## 13. 시장가 주문 체결 확인 패턴

KIS 시장가 주문은 접수 즉시 체결이 일어난다. WebSocket 체결 이벤트가 없는 상황에서  
폴링 루프에서 `on_fill()`을 호출하려면 "다음 틱 자동 확인" 패턴을 사용한다.

```rust
// 폴링 루프 시작 전 선언
let mut fills_pending: Vec<(String, u64)> = Vec::new(); // (symbol, 접수 당시 가격)

'main_loop: loop {
    // ① 이전 틱에서 접수된 주문의 체결 확인 (10초 뒤 → 시장가 주문 체결 충분)
    if !fills_pending.is_empty() {
        let fills = std::mem::take(&mut fills_pending);
        for (sym, fill_price) in fills {
            let _ = order_mgr.lock().await
                .confirm_fill_by_symbol(&sym, fill_price).await;
        }
    }

    'symbol_loop: for symbol in &symbols {
        let (price, volume) = fetch_tick(...);
        let signals = strategy.on_tick(symbol, price, volume);
        for signal in signals {
            match order_mgr.submit_signal(signal, ...).await {
                Ok(()) => {
                    fills_pending.push((symbol.clone(), price)); // ② 다음 틱에 체결 확인 예약
                    tokio::time::sleep(delay_ms).await;           // ③ rate-limit 방지
                }
                Err(e) => { /* 오류 처리 */ }
            }
        }
        tokio::time::sleep(delay_ms).await; // 가격 조회 후 delay
    }
    // ④ 다음 틱까지 10초 대기
}
```

### EGW00201 rate-limit 재시도

KIS 초당 거래건수 제한 오류 (`EGW00201`) 도 `EGW00133`과 마찬가지로 재시도 처리가 필요하다.  
`place_with_retry()`에서 두 코드를 모두 감지해야 한다:

```rust
let is_rate_limit = msg.contains("EGW00133") || msg.contains("EGW00201");
if is_rate_limit && attempt < MAX_RETRIES - 1 {
    tokio::time::sleep(Duration::from_secs(2)).await; // 2초 대기 (모의: 0.5req/s)
}
```

---

## 해외 자동매매 — 주요 패턴

### 거래소 코드 변환 (가격 API vs 주문 API)

```
가격 조회 EXCD:  NAS  →  주문 OVRS_EXCG_CD: NASD
가격 조회 EXCD:  NYS  →  주문 OVRS_EXCG_CD: NYSE
가격 조회 EXCD:  AMS  →  주문 OVRS_EXCG_CD: AMEX
```

❌ **잘못된 패턴**: NAS 코드를 그대로 `place_overseas_order`에 사용  
✅ **올바른 패턴**: 아래 매핑 후 사용

```rust
let order_exch = match exch.as_str() {
    "NAS" => "NASD",
    "NYS" => "NYSE",
    "AMS" => "AMEX",
    other => other,
};
```

### 해외 자동매매는 지정가만 지원

KIS 해외 주문 API (`TTTT1002U`)는 시장가(ord_dvsn="00"만)를 지원하지 않는다.  
`fetch_overseas_tick`에서 반환된 현재가를 그대로 지정가로 사용한다.

```rust
// fetch_overseas_tick → (price_cents, volume, exchange)
// price_cents = USD × 100
let usd_price = tick_price as f64 / 100.0;
```

### submit_signal 시그니처 (현재 버전)

```rust
pub async fn submit_signal(
    &mut self,
    signal: Signal,
    symbol_name: &str,
    total_balance: i64,
    exchange: Option<String>,  // None = 국내, Some("NAS"/"NYS"/"AMS") = 해외
    tick_price: u64,           // 국내 = 원, 해외 = USD × 100
) -> Result<()>
```

### 해외 잔고 조회 (TTTS3012R / VTTS3012R)

```
GET /uapi/overseas-stock/v1/trading/inquire-balance
params: CANO, ACNT_PRDT_CD, OVRS_EXCG_CD(""), TR_CRCY_CD("USD"),
        CTX_AREA_FK200(""), CTX_AREA_NK200("")
```

응답 output1 (per item): `ovrs_pdno`, `ovrs_item_name`, `ovrs_cblc_qty`, `pchs_avg_pric`, `now_pric2`, `ovrs_stck_evlu_amt`, `frcr_evlu_pfls_amt`, `evlu_pfls_rt`, `ovrs_excg_cd`, `tr_mket_name`  
응답 output2 (summary): `frcr_pchs_amt1`, `ovrs_tot_pfls`, `frcr_evlu_tota`, `tot_pftrt`

> 마지막 업데이트: 2026-04-09T13:00:00

---

## 가격 스케일 — 국내/해외 on_tick 단위 불일치 패턴 (재발 방지)

### 핵심 규칙

| 종목 구분 | on_tick `price: u64` 단위 | 트리거가 저장 단위 | 비교 시 변환 |
|----------|-------------------------|-----------------|------------|
| 국내 (is_domestic_symbol) | 원 (KRW) 정수, e.g. 55000 | 원 그대로 | 변환 불필요 |
| 해외 (US 주식)             | USD × 100 (cents), e.g. $619.50 → 61950 | USD face value, e.g. 619.50 | `trigger × 100` 후 비교 |

### ❌ 잘못된 패턴 — 해외 종목에서 영원히 신호가 발동하지 않음

```rust
// ❌ VOO $619.50 → price=61950, buy_trigger_price=620.0
// 61950 <= 620 → false → 매수 신호 발동 안 됨!!
if price <= sym_cfg.buy_trigger_price as u64 { ... }
```

### ✅ 올바른 패턴 — is_overseas 플래그로 스케일 맞춤

```rust
// strategy.rs
let scale: f64  = if sym_cfg.is_overseas { 100.0 } else { 1.0 };
let buy_thresh  = (sym_cfg.buy_trigger_price  * scale).round() as u64;
let sell_thresh = (sym_cfg.sell_trigger_price * scale).round() as u64;
let to_disp     = |p: u64| p as f64 / scale;  // 로그 출력용 역변환

if buy_thresh > 0 && price <= buy_thresh { /* 매수 */ }
if sell_thresh > 0 && price >= sell_thresh { /* 익절 */ }
```

### fetch_overseas_tick 반환 규격

```rust
// commands.rs
async fn fetch_overseas_tick(rest, symbol) -> (price_cents: u64, volume: u64, exchange: String)
// price_cents = USD 현재가 × 100 (정수화, e.g. $619.50 → 61950)
// on_tick(symbol, price_cents, volume) 으로 전달됨
```

### PriceConditionSymbolConfig — is_overseas 필드

```rust
// strategy.rs
pub struct PriceConditionSymbolConfig {
    pub is_overseas: bool,  // true = 해외, 가격 단위 USD (on_tick에서 ×100 처리됨)
    pub buy_trigger_price: f64,   // 국내: 원 | 해외: USD face value (e.g. 620.0)
    pub sell_trigger_price: f64,  // 국내: 원 | 해외: USD face value
    // ...
}
```

UI에서 해외 종목 추가 시 `is_overseas: market === 'US'` 자동 설정 (Strategy.tsx `handleAdd`).

### 중복 주문 방지 — in_position 플래그

`PriceConditionStrategy`의 `positions: HashMap<String, (bool, Option<u64>)>`에서  
`(in_position, entry_price)` 를 추적합니다.

- 매수 신호 발생 → `in_position = true` **즉시** (비동기 주문 제출 전에 설정)
- 다음 틱에서 `in_position = true` → 매수 신호 스킵 → **중복 매수 방지**
- 매도 신호 발생 → `in_position = false`, `entry_price = None` 즉시
- on_tick은 동기 함수 내에서 flag 설정하므로 race condition 없음

