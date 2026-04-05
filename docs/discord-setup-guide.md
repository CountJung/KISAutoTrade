# Discord 봇 연계 설정 가이드

> 이 문서는 AutoConditionTrade 프로젝트에서 Discord 봇을 통해  
> 알림(체결, 오류, 경고 등)을 전송하는 방법을 처음부터 끝까지 안내합니다.

---

## 목차

1. [Discord Developer Portal에서 봇 생성](#1-discord-developer-portal에서-봇-생성)
2. [봇을 서버에 초대하는 방법](#2-봇을-서버에-초대하는-방법)
3. [알림 전용 채널 생성 및 채널 ID 확인](#3-알림-전용-채널-생성-및-채널-id-확인)
4. [secure_config.json 설정](#4-secure_configjson-설정)
5. [앱 Settings UI에서 Discord 설정](#5-앱-settings-ui에서-discord-설정)
6. [테스트 알림 전송](#6-테스트-알림-전송)
7. [알림 레벨 설명](#7-알림-레벨-설명)
8. [자주 발생하는 문제 해결](#8-자주-발생하는-문제-해결)
9. [보안 주의사항](#9-보안-주의사항)

---

## 1. Discord Developer Portal에서 봇 생성

### 1-1. Developer Portal 접속

1. 웹 브라우저에서 [Discord Developer Portal](https://discord.com/developers/applications) 에 접속합니다.
2. Discord 계정으로 로그인합니다.

### 1-2. 새 Application 생성

1. 우측 상단의 **"New Application"** 버튼을 클릭합니다.
2. 애플리케이션 이름을 입력합니다. (예: `AutoConditionTrade Bot`)
3. 이용약관에 동의한 후 **"Create"** 버튼을 클릭합니다.

### 1-3. Bot 설정

1. 좌측 메뉴에서 **"Bot"** 항목을 선택합니다.
2. **"Add Bot"** 버튼을 클릭하고 확인합니다.
3. 봇 이름을 원하는 이름으로 변경할 수 있습니다. (예: `ACT Alert Bot`)
4. **"Reset Token"** 버튼을 클릭하여 Bot Token을 발급받습니다.
   - ⚠️ **이 토큰은 한 번만 표시됩니다. 반드시 안전한 곳에 보관하세요.**
   - 토큰이 외부에 유출되면 즉시 Reset 해야 합니다.

### 1-4. Bot 권한 설정

**Privileged Gateway Intents** 섹션에서 아래 항목을 **비활성화** 상태로 유지합니다:
- `Presence Intent` — 불필요
- `Server Members Intent` — 불필요
- `Message Content Intent` — 불필요 (메시지 전송만 하면 됨)

**Bot Permissions** 섹션에서는 별도 설정 없이 기본값을 유지합니다.

---

## 2. 봇을 서버에 초대하는 방법

### 2-1. 알림 전용 Discord 서버 생성 (권장)

1. Discord 앱을 열고 좌측 하단의 **"+"** 버튼을 클릭합니다.
2. **"직접 만들기"** → **"나와 내 친구들을 위한 서버"** 를 선택합니다.
3. 서버 이름을 입력합니다. (예: `AutoTrade Alerts`)

> 기존 서버에 봇을 초대해도 되지만, 자동매매 알림 전용 서버를 분리하는 것을 권장합니다.

### 2-2. OAuth2 URL 생성

1. Developer Portal에서 좌측 메뉴 **"OAuth2"** → **"URL Generator"** 를 선택합니다.
2. **Scopes** 섹션에서 `bot` 을 체크합니다.
3. **Bot Permissions** 섹션에서 다음을 체크합니다:
   - ✅ `Send Messages` — 채널에 메시지 전송
   - ✅ `Embed Links` — Embed 형식 메시지 전송
   - ✅ `Read Message History` — (선택사항)
4. 하단에 생성된 URL을 복사합니다.

### 2-3. 봇 초대

1. 복사한 URL을 브라우저 주소창에 붙여넣기 합니다.
2. 봇을 추가할 서버를 선택합니다.
3. **"인증"** 버튼을 클릭하고 보안 문자를 입력합니다.
4. 서버 멤버 목록에 봇이 나타나면 초대 완료입니다.

---

## 3. 알림 전용 채널 생성 및 채널 ID 확인

### 3-1. 알림 채널 생성

1. Discord 서버에서 **"#"** 아이콘 옆 **"+"** 버튼을 클릭합니다.
2. **"텍스트 채널"** 을 선택합니다.
3. 채널 이름을 입력합니다. (예: `#trade-alerts`)
4. 비공개 채널로 설정하고 봇에게만 접근 권한을 부여하는 것을 권장합니다.

### 3-2. 채널 ID 확인

> 채널 ID를 확인하려면 Discord의 **개발자 모드**를 활성화해야 합니다.

**개발자 모드 활성화:**
1. Discord 앱에서 **설정** (단축키: `Ctrl + ,`) 을 엽니다.
2. **"고급"** 섹션으로 이동합니다.
3. **"개발자 모드"** 를 켭니다.

**채널 ID 복사:**
1. 알림 채널 이름을 **우클릭** 합니다.
2. **"ID 복사"** 를 클릭합니다.
3. 복사된 숫자(예: `1234567890123456789`)가 채널 ID입니다.

---

## 4. secure_config.json 설정

프로젝트 루트에 `secure_config.json` 파일을 생성합니다.  
이 파일은 `.gitignore`에 포함되어 있어 Git에 커밋되지 않습니다.

```json
{
  "discord_bot_token": "여기에_봇_토큰을_입력하세요",
  "discord_channel_id": "여기에_채널_ID를_입력하세요",
  "notification_levels": ["CRITICAL", "ERROR", "WARNING", "TRADE", "INFO"]
}
```

### 설정 항목 설명

| 항목 | 타입 | 설명 |
|------|------|------|
| `discord_bot_token` | String | Discord Developer Portal에서 발급한 Bot Token |
| `discord_channel_id` | String | 알림을 전송할 Discord 채널 ID (숫자 문자열) |
| `notification_levels` | Array | 알림을 보낼 레벨 목록 (원하는 레벨만 포함) |

### notification_levels 옵션

| 값 | 설명 | 기본 포함 |
|----|------|----------|
| `"CRITICAL"` | 앱 패닉, 복구 불가 오류 | ✅ 필수 권장 |
| `"ERROR"` | API 오류 반복, 주문 실패 | ✅ 권장 |
| `"WARNING"` | 손실 한도 근접, 비정상 시장 감지 | 선택 |
| `"TRADE"` | 매수/매도 체결 발생 시 | ✅ 권장 |
| `"INFO"` | 자동매매 시작/종료, 장 시작/종료 | 선택 |

---

## 5. 앱 Settings UI에서 Discord 설정

1. AutoConditionTrade 앱을 실행합니다.
2. 좌측 메뉴에서 **Settings** 로 이동합니다.
3. **"Discord 알림"** 섹션을 찾습니다.
4. 다음 항목을 입력합니다:
   - **Bot Token**: Discord Developer Portal에서 복사한 토큰
   - **Channel ID**: 알림 채널 ID
5. **알림 레벨** 토글에서 받고 싶은 알림 레벨을 선택합니다.
6. **"저장"** 버튼을 클릭합니다.

> Settings UI에서 입력한 내용은 내부적으로 암호화되어 `secure_config.json`에 저장됩니다.  
> Bot Token은 화면에 마스킹(`••••••••••••`)되어 표시됩니다.

---

## 6. 테스트 알림 전송

1. Settings 화면의 Discord 섹션에서 **"테스트 메시지 전송"** 버튼을 클릭합니다.
2. 설정된 Discord 채널에 아래와 같은 테스트 메시지가 도착하면 설정 완료입니다.

```
[🔵 INFO] AutoConditionTrade — 테스트 알림
시각: 2025-04-02 12:00:00 KST
내용: Discord 알림 연결이 성공적으로 설정되었습니다.
```

3. 메시지가 도착하지 않으면 [8. 자주 발생하는 문제 해결](#8-자주-발생하는-문제-해결)을 참조하세요.

---

## 7. 알림 레벨 설명

### CRITICAL — 🔴

앱이 즉시 중단되거나 복구가 불가능한 심각한 오류 발생 시 전송됩니다.

```
[🔴 CRITICAL] AutoConditionTrade
시각: 2025-04-02 09:31:05 KST
내용: Rust 패닉 발생 — thread 'tokio-runtime-worker'
원인: KIS API 토큰 갱신 재시도 초과 (5/5)
조치: 앱을 재시작하거나 API 키를 확인하세요.
```

### ERROR — 🟠

복구를 시도했지만 실패한 오류. 자동 매매가 중단될 수 있습니다.

```
[🟠 ERROR] AutoConditionTrade
시각: 2025-04-02 09:45:22 KST
내용: 주문 실패 (3회 연속)
종목: 삼성전자 (005930), 매수, 10주
원인: HTTP 429 — API 요청 한도 초과
```

### WARNING — 🟡

이상 징후이지만 즉각적인 조치가 필요하지 않은 경우입니다.

```
[🟡 WARNING] AutoConditionTrade
시각: 2025-04-02 10:15:00 KST
내용: 일일 손실 한도 80% 도달
현재 손실: -40,000원 / 한도: -50,000원
조치: 추가 매매 신중히 진행하세요.
```

### TRADE — 🟢

매수 또는 매도 체결 완료 시 전송됩니다.

```
[🟢 TRADE] 체결 완료
종목: 삼성전자 (005930)
방향: 매수
수량: 10주 @ 72,000원
총액: 720,000원
전략: RSI Cross v1
체결 시각: 2025-04-02 09:31:05 KST
```

### INFO — 🔵

앱 상태 변경 등 일반 정보성 알림입니다.

```
[🔵 INFO] 자동매매 시작
시각: 2025-04-02 09:00:00 KST
활성 전략: RSI Cross v1, MA Golden Cross
```

---

## 8. 자주 발생하는 문제 해결

### ❌ 테스트 메시지가 채널에 오지 않는 경우

| 원인 | 해결 방법 |
|------|----------|
| Bot Token이 잘못 입력됨 | Developer Portal에서 토큰을 재확인하거나 Reset 후 재입력 |
| 채널 ID가 잘못됨 | Discord 개발자 모드에서 채널 ID 재확인 |
| 봇이 서버에 없음 | [2. 봇을 서버에 초대하는 방법](#2-봇을-서버에-초대하는-방법) 단계 재진행 |
| 봇에게 채널 권한 없음 | 채널 설정 → 권한에서 봇에게 `Send Messages` 권한 부여 |
| `secure_config.json` 파일이 없음 | 프로젝트 루트에 파일 직접 생성 |

### ❌ `Invalid Token` 오류

1. Developer Portal에서 **"Reset Token"** 을 클릭하여 새 토큰을 발급합니다.
2. `secure_config.json` 또는 Settings UI에서 새 토큰으로 업데이트합니다.

### ❌ `Missing Access` 오류 (HTTP 403)

1. Discord 서버에서 알림 채널의 **권한 설정** 을 엽니다.
2. 봇 역할 또는 봇 이름을 검색하여 추가합니다.
3. `텍스트 채널 보기`, `메시지 보내기` 권한을 허용합니다.

### ❌ `Unknown Channel` 오류 (HTTP 404)

채널 ID가 잘못되었거나 채널이 삭제된 경우입니다.
- 채널이 존재하는지 확인하고 채널 ID를 재확인합니다.

---

## 9. 보안 주의사항

> **Discord Bot Token은 민감한 정보입니다. 아래 규칙을 반드시 따르세요.**

1. **절대 코드에 하드코딩 금지**  
   Bot Token을 소스 코드에 직접 입력하지 마세요.

2. **Git에 커밋 금지**  
   `secure_config.json`은 `.gitignore`에 포함되어 있습니다.  
   실수로 커밋했다면 즉시 Developer Portal에서 토큰을 Reset 하세요.

3. **토큰 유출 시 즉시 재발급**  
   토큰이 외부에 노출되었다면 Developer Portal에서 **"Reset Token"** 을 클릭하여 무효화합니다.

4. **알림 전용 서버 사용 권장**  
   거래 정보가 포함된 알림을 기존 일반 서버에 보내지 않도록 별도 서버를 사용합니다.

5. **채널을 비공개로 유지**  
   알림 채널은 본인과 봇만 접근할 수 있도록 설정합니다.

---

*이 문서는 AutoConditionTrade `docs/MasterPlan.md` 및 `agent.md`와 함께 유지됩니다.*  
*Discord API 변경 시 이 문서도 업데이트가 필요합니다.*
