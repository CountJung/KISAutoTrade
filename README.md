# AutoConditionTrade

> **개인용 자동 주식 매매 시스템** — 한국투자증권 KIS Open API 기반  
> Tauri v2 (Rust) + React 18 + TypeScript 풀스택 데스크탑 앱

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](./LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.77+-orange.svg)](https://www.rust-lang.org/)
[![Tauri](https://img.shields.io/badge/Tauri-v2-blue.svg)](https://tauri.app/)
[![React](https://img.shields.io/badge/React-18-blue.svg)](https://react.dev/)
[![Platform](https://img.shields.io/badge/Platform-Windows%20%7C%20macOS-lightgrey.svg)]()
[![Release](https://img.shields.io/github/v/release/CountJung/AutoConditionTrade?include_prereleases)](https://github.com/CountJung/AutoConditionTrade/releases)

---

## 목차

1. [개요](#개요)
2. [주요 기능](#주요-기능)
3. [아키텍처](#아키텍처)
4. [플랫폼별 필수 요구사항](#플랫폼별-필수-요구사항)
5. [설치](#설치)
6. [설정](#설정)
7. [계좌 프로필 관리](#계좌-프로필-관리)
8. [실행 및 개발](#실행-및-개발)
9. [VSCode 디버깅](#vscode-디버깅)
10. [KIS API 발급 가이드](#kis-api-발급-가이드)
11. [Discord 알림 설정](#discord-알림-설정)
12. [화면 구성](#화면-구성)
13. [자동 매매 전략](#자동-매매-전략)
14. [데이터 저장 구조](#데이터-저장-구조)
15. [프로덕션 빌드](#프로덕션-빌드)
16. [문제 해결](#문제-해결)
17. [라이선스 및 면책 조항](#라이선스-및-면책-조항)

---

## 개요

AutoConditionTrade는 **한국투자증권(KIS) Open API**를 사용하는 개인용 자동 주식 매매 시스템입니다.
Rust 기반 Tauri v2 백엔드와 React 18 프론트엔드로 구성된 크로스 플랫폼 데스크탑 애플리케이션으로,
**Windows / macOS** 모두 지원합니다.

- **멀티 계좌(앱 키) 프로필** 등록/전환 지원 (실전·모의투자 분리 관리)
- **이동평균 전략** 등 자동 매매 전략 ON/OFF
- **Discord 봇** 연동 알림 (체결, 에러, 일별 요약)
- **JSON 파일** 기반 데이터 저장 (DB 불필요)

---

## 주요 기능

| 기능 | 설명 |
|------|------|
| 멀티 계좌 프로필 | 여러 KIS 앱 키를 등록하고 활성 프로필을 전환 |
| 잔고 조회 | 실시간 보유 종목 및 평가손익 조회 |
| 수동 주문 | 지정가/시장가 매수·매도 |
| 실시간 시세 | KIS WebSocket (H0STCNT0) 체결 스트림 수신 |
| 자동 전략 | 이동평균 골든/데스 크로스 자동 매매 |
| 리스크 관리 | 일일 손실 한도, 비상 정지, 포지션 비율 관리 |
| 거래 기록 | 체결 내역 JSON 저장 및 날짜 범위 조회 |
| 통계 | 일별/월별 손익 통계 집계 |
| 로그 뷰어 | 레벨 필터, 키워드 검색, 색상 구분 로그 |
| Discord 알림 | CRITICAL / ERROR / TRADE 레벨 알림 |
| 설정 진단 | API 키 설정 상태 실시간 진단 |

---

## 아키텍처

```
React UI  ──(Tauri IPC)──►  Rust Backend  ──►  KIS Open API (REST + WebSocket)
                                           ──►  Discord Bot API
                                           ──►  JSON 파일 Storage (연/월/일 폴더)
```

```
AutoConditionTrade/
├── src/                    # React 프론트엔드 (TypeScript)
│   ├── pages/              # 대시보드, 거래, 전략, 기록, 로그, 설정
│   ├── api/                # Tauri IPC 래퍼, TanStack Query 훅, 타입
│   └── store/              # Zustand 전역 상태
├── src-tauri/              # Rust 백엔드
│   └── src/
│       ├── api/            # KIS REST 클라이언트, 토큰 관리, WebSocket
│       ├── trading/        # 전략 엔진, 포지션 트래커, 리스크 관리
│       ├── storage/        # JSON 파일 I/O (거래, 통계, 잔고)
│       ├── notifications/  # Discord 봇 알림
│       ├── config/         # AccountProfile, ProfilesConfig, DiscordConfig
│       └── commands.rs     # Tauri IPC 핸들러 (24종)
├── .vscode/                # VSCode 디버그/태스크 설정 (git 포함)
├── profiles.json           # 계좌 프로필 (git 제외 — API 키 포함)
├── secure_config.json      # Discord 설정 (git 제외)
└── secure_config.example.json  # 설정 템플릿
```

---

## 플랫폼별 필수 요구사항

### 공통

| 도구 | 최소 버전 | 설치 확인 |
|------|-----------|-----------|
| Node.js | 18.x 이상 | `node --version` |
| npm | 9.x 이상 | `npm --version` |
| Rust | 1.77 이상 | `rustc --version` |

Rust는 [rustup](https://rustup.rs/)으로 설치합니다:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh  # macOS/Linux
# Windows: https://rustup.rs/ 에서 설치 프로그램 다운로드
```

---

### Windows

#### 1. Microsoft C++ Build Tools

Rust 링커에 필요합니다. 다음 중 하나를 설치합니다:
- [Visual Studio](https://visualstudio.microsoft.com/) (C++ 워크로드 포함)
- [Microsoft C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) (최소 설치)

설치 시 **"C++를 사용한 데스크톱 개발"** 워크로드를 반드시 선택합니다.

#### 2. WebView2

Windows 10 21H2 이상 / Windows 11에서는 기본 포함됩니다.  
구버전 Windows는 [Microsoft WebView2](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) 설치 필요.

#### 3. VSCode 디버거 확장

- [C/C++ (ms-vscode.cpptools)](https://marketplace.visualstudio.com/items?itemName=ms-vscode.cpptools) — Rust 디버깅 (Windows MSVC)
- [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

---

### macOS

#### 1. Xcode Command Line Tools

```bash
xcode-select --install
```

설치 확인:
```bash
xcode-select -p
# /Library/Developer/CommandLineTools
```

#### 2. macOS SDK 요구사항

- macOS 12 (Monterey) 이상 권장
- Tauri v2는 macOS 10.15+ 지원이지만 이 앱은 최소 12.0으로 설정됩니다

#### 3. VSCode 디버거 확장

- [CodeLLDB (vadimcn.vscode-lldb)](https://marketplace.visualstudio.com/items?itemName=vadimcn.vscode-lldb) — Rust 디버깅
- [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

CodeLLDB 설치:
```bash
# VSCode에서
code --install-extension vadimcn.vscode-lldb
```

#### 4. (선택) Homebrew 패키지

```bash
brew install pkg-config
```

일부 Rust 크레이트(특히 TLS 관련)가 `pkg-config`를 필요로 할 수 있습니다.

---

## 설치

### 1. 저장소 클론

```bash
git clone https://github.com/your-username/AutoConditionTrade.git
cd AutoConditionTrade
```

### 2. Node 의존성 설치

```bash
npm install
```

### 3. Rust 의존성 확인

```bash
# src-tauri 디렉토리에서
cargo check --manifest-path src-tauri/Cargo.toml
```

처음 실행 시 크레이트 다운로드에 몇 분이 소요될 수 있습니다.

---

## 설정

앱 설정에는 **두 개의 파일**이 사용됩니다. 두 파일 모두 `.gitignore`에 포함되어 Git에 추적되지 않습니다.

| 파일 | 용도 | 저장 방식 |
|------|------|-----------|
| `profiles.json` | KIS 계좌/앱 키 (멀티 프로필) | **앱 내 Settings UI에서 관리** |
| `secure_config.json` | Discord 봇 설정 | 수동으로 파일 작성 |

### secure_config.json (Discord 설정)

```bash
cp secure_config.example.json secure_config.json
```

`secure_config.json`을 열고 Discord 설정만 입력합니다. KIS API 키는 앱 UI에서 관리합니다.

```json
{
  "discord_bot_token": "여기에_Discord_Bot_Token_입력",
  "discord_channel_id": "여기에_채널_ID_입력",
  "notification_levels": ["CRITICAL", "ERROR", "TRADE"]
}
```

Discord 알림이 필요 없으면 이 파일 생성을 생략해도 됩니다.

---

## 계좌 프로필 관리

KIS API 키는 앱의 **Settings > 한국투자증권 계좌 프로필** 섹션에서 관리합니다.
여러 계좌(앱 키 세트)를 등록해두고 클릭 한 번으로 전환할 수 있습니다.

### 계좌 추가

1. 앱 실행 후 좌측 사이드바에서 **Settings** 클릭
2. **계좌 프로필** 섹션에서 **계좌 추가** 버튼 클릭
3. 다음 정보를 입력합니다:

| 필드 | 설명 | 예시 |
|------|------|------|
| 프로필 이름 | 식별용 이름 (자유) | `모의투자 주계좌` |
| 투자 모드 | 모의투자 / 실전투자 토글 | 모의투자 ON |
| APP KEY | KIS Developers에서 발급 | `PSK5Be...` |
| APP SECRET | KIS Developers에서 발급 | (비밀) |
| 계좌번호 | 10자리 (하이픈 유무 모두 가능) | `12345678-01` 또는 `1234567801` |

> **APP SECRET**은 저장 후 화면에 표시되지 않습니다. 분실 시 KIS Developers에서 재확인합니다.

### 계좌 전환

프로필 카드의 **활성화** 버튼을 클릭하면 앱 재시작 없이 즉시 전환됩니다.

### 프로필 저장 위치

등록된 계좌 정보는 프로젝트 루트의 `profiles.json`에 저장됩니다.  
이 파일은 `.gitignore`에 포함되어 있어 **절대 Git에 커밋되지 않습니다**.

```
⚠️ profiles.json에는 APP SECRET이 평문으로 저장됩니다.
   파일 공유, 클라우드 동기화 경로에 주의하세요.
```

### 계좌번호 형식

KIS 계좌번호는 10자리를 그대로 입력하거나 하이픈을 포함한 형식 모두 지원합니다.

| 원본 (10자리) | 입력 예시 |
|---------------|-----------|
| `1234567890` | `1234567890` 또는 `12345678-90` |

앱이 입력값에서 하이픈을 자동 처리(CANO 8자리 / ACNT_PRDT_CD 2자리 분리)하므로 어느 형식으로 입력해도 정상 동작합니다.

---

## 실행 및 개발

### 개발 모드 (권장)

```bash
npm run tauri dev
```

Vite 개발 서버와 Tauri 앱이 함께 시작됩니다. 프론트엔드 변경 시 Hot Reload 적용됩니다.

### 개별 검증

```bash
# TypeScript 타입 체크
npx tsc --noEmit

# Rust 빠른 검증 (빌드 없이)
cargo check --manifest-path src-tauri/Cargo.toml

# 둘 다 한 번에
npm run tauri dev
```

---

## VSCode 디버깅

`.vscode/` 폴더에 디버그 구성이 포함되어 있습니다 (Git 추적).

### 디버그 구성 목록

| 구성 이름 | 설명 | OS |
|-----------|------|-----|
| `Rust: Debug App (Windows MSVC)` | Rust 브레이크포인트 디버깅 | Windows |
| `Rust: Debug App (CodeLLDB)` | Rust 브레이크포인트 디버깅 | Windows/macOS |
| `Frontend: Chrome (Vite Dev)` | React 브라우저 디버깅 | 모든 OS |
| `Frontend: Edge (Vite Dev)` | React 브라우저 디버깅 | Windows |
| `Tauri: Full Debug (MSVC + Chrome)` | Rust + 프론트 동시 | Windows |
| `Tauri: Full Debug (CodeLLDB + Chrome)` | Rust + 프론트 동시 | Windows/macOS |

### Windows에서 디버깅

1. `F5` 키 → **Rust: Debug App (Windows MSVC)** 선택
2. Vite 서버 자동 시작 → Cargo 빌드 → 앱 실행
3. Rust 소스에 브레이크포인트 설정 후 `F5`

**필수 확장**: `ms-vscode.cpptools` (C/C++ 확장)

### macOS에서 디버깅

1. **CodeLLDB** 확장 설치: `code --install-extension vadimcn.vscode-lldb`
2. `F5` 키 → **Rust: Debug App (CodeLLDB)** 선택
3. Rust 소스에 브레이크포인트 설정 후 `F5`

### 태스크 목록

`Ctrl+Shift+P` → `Tasks: Run Task` 에서 실행:

| 태스크 | 설명 |
|--------|------|
| `tauri: dev` | 전체 개발 모드 (디버거 없이) |
| `cargo: build debug` | Rust 디버그 빌드만 |
| `cargo: check` | 빠른 Rust 타입 검증 |
| `tsc: type check` | TypeScript 타입 검증 |
| `verify: all` | cargo check + tsc 동시 실행 |

---

## KIS API 발급 가이드

### 1. 개발자 포털 가입

1. [KIS Developers](https://apiportal.koreainvestment.com/) 접속
2. 한국투자증권 계좌 보유자만 가입 가능
3. 로그인 → **앱 등록** → APP KEY / APP SECRET 확인

### 2. 모의투자 vs 실전투자

| 구분 | API 엔드포인트 | 특징 |
|------|----------------|------|
| 모의투자 | `openapivts.koreainvestment.com:29443` | 가상 주문, 실제 돈 없음 |
| 실전투자 | `openapi.koreainvestment.com:9443` | 실제 계좌, 실제 주문 |

> **⚠️ 처음 사용 시 반드시 모의투자로 먼저 검증하세요!**

### 3. API 호출 제한

| 환경 | 초당 한도 |
|------|-----------|
| 실전투자 | 20건/초 |
| 모의투자 | 2건/초 |

### 4. 계좌 권한 등록

KIS Developers 포털 → 발급받은 앱 → **계좌 권한** 탭에서 사용할 계좌를 등록해야 API 호출이 가능합니다.

---

## Discord 알림 설정

자세한 내용은 [`docs/discord-setup-guide.md`](./docs/discord-setup-guide.md)를 참고하세요.

### 빠른 설정 요약

1. [Discord Developer Portal](https://discord.com/developers/applications) → **New Application** 생성
2. **Bot** 탭 → `Token` 복사
3. **OAuth2 → URL Generator** → `bot` scope + `Send Messages` 권한으로 초대 URL 생성
4. 봇을 서버에 초대 후 알림 받을 채널 ID를 우클릭으로 복사
5. `secure_config.json`에 입력

### 알림 레벨

| 레벨 | 발송 시점 |
|------|-----------|
| `CRITICAL` | 시스템 장애, 비상 정지 |
| `ERROR` | API 오류, 주문 실패 |
| `TRADE` | 매수/매도 체결 |
| `INFO` | 자동 매매 시작/정지 |

---

## 화면 구성

| 메뉴 | 주요 기능 |
|------|-----------|
| Dashboard | 실시간 잔고 카드, 평가손익, 당일 체결 내역 |
| Trading | 종목 현재가 조회, 매수/매도 주문 폼, 체결 내역 |
| Strategy | 이동평균 전략 파라미터 설정, 전략 ON/OFF |
| History | 기간별 거래 기록 및 일별 통계 테이블 |
| Log | 레벨 필터 + 키워드 검색 로그 뷰어 |
| Settings | **계좌 프로필 관리**, 테마, Discord 테스트, 진단 |

---

## 자동 매매 전략

### 이동평균 골든/데스 크로스 (MovingAverageCross)

| 파라미터 | 기본값 | 설명 |
|----------|--------|------|
| `short_period` | 5 | 단기 이동평균 기간 (봉 수) |
| `long_period` | 20 | 장기 이동평균 기간 (봉 수) |
| `order_quantity` | 1 | 1회 주문 수량 (주) |

- **매수 신호**: 단기선이 장기선을 상향 돌파 (골든 크로스)
- **매도 신호**: 단기선이 장기선을 하향 돌파 (데스 크로스)

### 리스크 관리

| 항목 | 설명 |
|------|------|
| 일일 손실 한도 | 설정 수치 초과 시 당일 자동 매매 정지 |
| 단일 종목 비율 | 총 자산 대비 단일 종목 최대 비중 제한 |
| 비상 정지 | Trading 화면에서 즉시 정지 가능 |

---

## 데이터 저장 구조

DB 없이 JSON 파일만 사용합니다.

```
{AppData}/AutoConditionTrade/
└── data/
    ├── trades/{YYYY}/{MM}/{DD}/trades.json    ← 체결 내역
    ├── stats/{YYYY}/{MM}/daily_stats.json     ← 일별 통계
    ├── orders/{YYYY}/{MM}/{DD}/orders.json    ← 주문 기록
    └── balance/{YYYY}/{MM}/{DD}/balance.json  ← 잔고 스냅샷
```

### 데이터 폴더 위치

| OS | 경로 |
|----|------|
| Windows | `%APPDATA%\AutoConditionTrade\` |
| macOS | `~/Library/Application Support/AutoConditionTrade/` |

---

## 프로덕션 빌드

### 로컴 빌드

```bash
npm run tauri build
```

빌드된 설치 파일 위치: `src-tauri/target/release/bundle/`

| OS | 파일 형식 | 설명 |
|----|-----------|------|
| Windows | `.msi` | Windows Installer |
| Windows | `.exe` (NSIS) | 단독 설치 파일 |
| macOS | `.dmg` | 디스크 이미지 |
| macOS | `.app` | 앱 번들 |

---

### GitHub Actions 자동 빌드 & 릴리스

태그를 푸시하면 GitHub Actions가 Windows + macOS 빌드를 자동으로 실행하고 **GitHub Releases**에 드래프트 릴리스를 생성합니다.

#### 사용 방법

```bash
# 1. 태그 및 빌드 버전을 tauri.conf.json 명세와 동기화 (tauri.conf.json이 우선)
#    tauri.conf.json의 version 필드 수정 말고 태그 버전을 커에 맞추세요.

# 2. 태그 생성 & 푸시
git tag v1.0.0
git push origin v1.0.0
```

#### 빌드 결과

| 플랫폼 | 생성 파일 |
|--------|------|
| Windows | `.msi` + NSIS `.exe` |
| macOS | Universal `.dmg` (Apple Silicon + Intel) |

GitHub Actions가 완료되면 **GitHub 리포지토리 → Releases**에 드래프트 릴리스가 만들어집니다.
릴리스 노트 수정 후 **Publish release** 버튼으로 공개하면 파일 다운로드가 가능해집니다.

> ⚠️ **macOS 코드서명 미적용 시**: `.dmg`를 열 때 Gatekeeper 경고가 남을 수 있습니다.  
> `sudo xattr -rd com.apple.quarantine AutoConditionTrade.app` 또는 시스템 환경설정에서 허용하세요.

### macOS 빌드 시 주의사항

#### Apple Silicon (M1/M2/M3)

기본적으로 현재 아키텍처만 빌드됩니다. 유니버설 바이너리(Intel + Apple Silicon)를 빌드하려면:

```bash
rustup target add x86_64-apple-darwin
rustup target add aarch64-apple-darwin
npm run tauri build -- --target universal-apple-darwin
```

#### 코드 서명 (배포 시)

개인 사용 목적이라면 서명 없이도 실행 가능합니다. 다른 Mac에 배포할 경우:

```bash
# 서명 없이 실행하는 방법 (수신인 안내)
sudo xattr -rd com.apple.quarantine /Applications/AutoConditionTrade.app
```

또는 `tauri.conf.json`의 `signingIdentity`에 개발자 인증서를 설정합니다.

#### Gatekeeper 우회 (개인 빌드)

서명되지 않은 앱을 처음 실행할 때:
1. Finder에서 앱을 `Control + 클릭` → **열기** 선택
2. 또는: **시스템 설정 → 개인 정보 보호 및 보안 → 확인 없이 열기**

---

## 문제 해결

### invoke 관련 오류: "Cannot read properties of undefined (reading 'invoke')"

**원인**: Vite가 `@tauri-apps/api`를 pre-bundle할 때 `window.__TAURI_INTERNALS__` 초기화 전에 모듈이 평가됩니다.

**해결됨**: `vite.config.ts`의 `optimizeDeps.exclude: ['@tauri-apps/api']`로 수정됩니다.

만약 여전히 발생하면:
1. `node_modules/.vite` 캐시 삭제 후 재시작
   ```bash
   rm -rf node_modules/.vite
   npm run tauri dev
   ```
2. `npm install`로 패키지 재설치

---

### "계좌 프로필 없음" 안내

앱 첫 실행 시 Settings에서 계좌를 추가해야 합니다. `secure_config.json`의 KIS 키는 더 이상 자동으로 로드되지 않습니다.

1. **Settings** → **계좌 추가** 클릭
2. APP KEY, APP SECRET, 계좌번호 입력

---

### Windows: 빌드 오류

```
error: linker `link.exe` not found
```
→ [Microsoft C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) 설치 필요  
→ "C++를 사용한 데스크톱 개발" 워크로드 선택

---

### macOS: OpenSSL 관련 빌드 오류

```
error: failed to run custom build command for `openssl-sys`
```

해결:
```bash
brew install openssl@3
export OPENSSL_DIR=$(brew --prefix openssl@3)
cargo check --manifest-path src-tauri/Cargo.toml
```

이 프로젝트는 `reqwest`에서 `rustls-tls` 피처를 사용하므로 일반적으로 OpenSSL이 필요 없습니다. 만약 발생한다면 Cargo.lock의 버전 충돌일 수 있습니다:
```bash
cargo update --manifest-path src-tauri/Cargo.toml
```

---

### macOS: 앱이 실행되지 않음 (Gatekeeper)

```
"AutoConditionTrade"은(는) Apple이 악성 소프트웨어가 있는지 확인할 수 없기 때문에 열 수 없습니다.
```

서명되지 않은 개인 빌드 실행 방법:
```bash
# 방법 1: 터미널에서 격리 속성 제거
sudo xattr -rd com.apple.quarantine AutoConditionTrade.app

# 방법 2: 시스템 설정 → 개인 정보 보호 및 보안 → "확인 없이 열기"
```

---

### WebSocket 연결 안 됨

- 실전투자/모의투자 API 키와 엔드포인트가 일치하는지 확인
- KIS Developers 포털 → 앱 → **계좌 권한** 탭에서 해당 계좌 등록 여부 확인
- 방화벽에서 `openapi.koreainvestment.com:9443` 허용 여부 확인

---

### 토큰 만료 오류

KIS Access Token은 24시간 유효하며 만료 5분 전 자동 갱신됩니다.  
오류 발생 시 앱을 재시작하거나 Settings에서 프로필을 재활성화합니다.

---

### 로그 파일 위치

| OS | 경로 |
|----|------|
| Windows | `%APPDATA%\AutoConditionTrade\log\` |
| macOS | `~/Library/Application Support/AutoConditionTrade/log/` |

---

## 라이선스 및 면책 조항

이 프로젝트는 [MIT 라이선스](./LICENSE) 하에 배포됩니다.

> ⚠️ **면책 조항**  
> 이 소프트웨어는 개인 학습 목적으로 제작되었습니다.  
> 실제 주식 거래에 사용 시 발생하는 모든 손익에 대한 책임은 사용자 본인에게 있습니다.  
> **실전 계좌 사용 전 반드시 모의투자로 충분히 테스트하세요.**
