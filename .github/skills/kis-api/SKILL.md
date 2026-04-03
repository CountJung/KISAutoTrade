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

### 당일 체결 내역

```
GET /uapi/domestic-stock/v1/trading/inquire-daily-ccld
```

| TR-ID | 환경 |
|-------|------|
| `TTTC8001R` | 실전투자 |
| `VTTC8001R` | 모의투자 |

| 주요 파라미터 | 값 |
|------------|---|
| `INQR_STRT_DT` | 오늘 날짜 `YYYYMMDD` |
| `INQR_END_DT` | 오늘 날짜 `YYYYMMDD` |
| `SLL_BUY_DVSN_CD` | `00` (전체), `01` (매도), `02` (매수) |
| `INQR_DVSN` | `00` (역순) |
| `PDNO` | `""` (전체 종목) |

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
>
> 마지막 업데이트: 2026-04-02
