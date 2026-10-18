# KISAutoTrade

한국투자증권 **KIS Open API**와 **토스증권 Open API**를 연결하는 개인용 주식 매매 앱입니다. Tauri v2/Rust 백엔드와 React 18/TypeScript 화면을 사용하며 Windows와 macOS를 대상으로 합니다.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Tauri](https://img.shields.io/badge/Tauri-v2-blue.svg)](https://tauri.app/)
[![React](https://img.shields.io/badge/React-18-blue.svg)](https://react.dev/)
[![Release](https://img.shields.io/github/v/release/CountJung/KISAutoTrade?include_prereleases)](https://github.com/CountJung/KISAutoTrade/releases)

## 프로젝트 문서

처음 사용하는 경우 [사용 가이드](docs/user-guide.md)를 읽으세요. 운영·설계·할일 문서는 `docs/`, 작업 기록은 `docs/tasks/`에 두고 루트에는 이 문서와 에이전트 진입 문서를 둡니다.

| 문서 | 위치 |
|---|---|
| 사용·계좌 설정·주문·문제 해결 | [user-guide.md](docs/user-guide.md) |
| 단계 계획 | [MASTER_PLAN.html](docs/MASTER_PLAN.html) |
| 작업별 탐색 / 전체 파일 지도 | [PROJECT_MAP.md](docs/PROJECT_MAP.md) / [project-map.md](docs/project-map.md) |
| 아키텍처 / 검증 명령 | [ARCHITECTURE.md](docs/ARCHITECTURE.md) / [HARNESS_MAP.md](docs/HARNESS_MAP.md) |
| 개선 할일 / 작업 기록 템플릿 | [todo.md](docs/todo.md) · [상태·색상 보기](docs/todo.html) / [TASK_TEMPLATE.md](docs/tasks/TASK_TEMPLATE.md) |
| API 계약 / Toss 연동 | [ipc-commands.md](docs/ipc-commands.md) / [toss-openapi.md](docs/toss-openapi.md) |
| Discord 설정 / 릴리스 보안 | [discord-setup-guide.md](docs/discord-setup-guide.md) / [release-security.md](docs/release-security.md) |
| 에이전트 진입 | [AGENTS.md](AGENTS.md) / [CLAUDE.md](CLAUDE.md) |

## 주요 기능과 지원 범위

| 기능 | KIS 프로파일 | Toss 프로파일 |
|---|---|---|
| 계좌 설정 | APP KEY/SECRET, 계좌번호, 실전·모의 구분 | Client ID/Secret, 조회 후 선택한 `accountSeq`, 실거래 동의 |
| 조회 | 국내·미국 현재가, 잔고, 체결 | 국내·미국 현재가, 보유종목, 호가·최근 체결·안전 정보 |
| 수동 주문 화면 | 국내 지정가·시장가, 미국 USD 지정가 | 국내 지정가·시장가, 미국 USD 지정가; 동의·사전검증 필요 |
| 자동 전략 | 활성 계좌의 KIS 시세·과거 자료 사용 | 활성 계좌의 Toss 시세·캔들 사용; 실거래 동의 필요 |
| 시세 처리 | REST 폴링, KIS WebSocket 보조 | REST 폴링 |
| 차트 | 일·주·월봉 기반 프리셋 | 일봉·1분봉 |
| 세션 | 앱의 시장 시간 gate와 provider 응답 | provider 달력과 미국 자동/주간/프리/정규/애프터 선택 |

표는 **현재 앱에 연결된 기능**입니다. 계좌별 권한·종목·세션의 실제 주문 가능 여부는 증권사 응답으로 확인합니다. KIS 모의투자에는 해외 조회·주문 조건 제한이 있으며, Toss는 앱에서 모의투자 모드를 제공하지 않습니다. 실제 provider 응답·운용 검증이 남은 항목은 [할일](docs/todo.md)에 별도로 표시합니다.

공통 기능은 13개 전략 카드, 프로파일별 전략 설정, 주문 전 리스크·중복 방어, 자동매매 전용 예산, 미체결·체결 대조, 거래·통계·로그, Discord 알림입니다. 저장은 기본 JSON이며 데스크톱 Settings에서 PostgreSQL/MariaDB로 명시 전환할 수 있습니다.

## 설치와 개발

저장소의 `package.json` 기준 Node.js 20 이상/npm 10 이상, `src-tauri/Cargo.toml` 기준 Rust 1.90 이상이 필요합니다. Windows는 C++ Build Tools와 WebView2, macOS는 Xcode Command Line Tools가 필요하며 앱의 macOS 최소 버전은 12.0입니다.

```bash
git clone https://github.com/CountJung/KISAutoTrade.git
cd KISAutoTrade
npm ci
cargo check --workspace --locked
npm run tauri dev
```

`npm run tauri dev`는 Vite 개발 서버와 데스크톱 앱을 함께 실행합니다. `npm run dev`만 실행하면 Rust 백엔드는 시작하지 않습니다. VSCode 디버그 구성은 [`.vscode/launch.json`](.vscode/launch.json)에 있습니다.

```bash
# 정적 검사
cargo check --workspace --locked
npx tsc --noEmit
npm run check:fsd
npm run check:lockfiles
npm run check:graphify
npm run check:project-map

# 프론트엔드 산출물 dist/만 생성
npm run build:web

# 데스크톱 패키지 생성
npm run build
```

변경별 테스트·릴리스 검증은 [Harness 지도](docs/HARNESS_MAP.md)를 따릅니다. Cargo 산출물은 루트 `target/`, 임시 연구·검증 캐시는 `.cache/`에 두며 Git에서 제외합니다.

## 처음 실행할 때

1. **Settings → 계좌 프로필 → 계좌 추가**에서 KIS 또는 Toss를 선택합니다.
2. KIS는 발급받은 키·계좌번호·투자 모드를 확인합니다. Toss는 Client ID/Secret으로 계좌 목록을 조회하고 마스킹된 계좌의 `accountSeq`를 선택합니다.
3. 프로파일을 활성화한 뒤 KIS 설정 진단 또는 Toss **연결 진단**으로 조회 상태를 확인합니다. Toss의 실거래 동의 저장은 조회 진단과 별도 단계입니다.
4. **Strategy**에서 대상 종목·수량·파라미터를 저장하고 미리보기의 자료·평가 상태를 확인합니다. **Settings**에서 리스크와 선택 계좌의 자동매매 전용 예산을 설정합니다.
5. 주문 또는 자동매매 시작은 사용자가 해당 화면에서 실행합니다. 미리보기 실행이나 전략 설정 저장만으로 자동매매를 시작하지 않습니다.

KIS 키 발급은 [공식 개발자 포털](https://apiportal.koreainvestment.com/), Toss는 [공식 개발자 문서](https://developers.tossinvest.com/docs)를 이용합니다. 프로파일의 비밀키와 선택 설정은 앱에서 관리하고 저장 파일을 Git에 추가하지 않습니다.

## 전략과 연구

일반 전략 11종에 가격 조건 매매와 레버리지 추세 보유(LTH)를 더한 **13개 카드**가 있습니다. 전체 목록과 각 화면의 사용 절차는 [전략 가이드](docs/user-guide.md#5-전략-설정과-미리보기)를 참고하세요.

미리보기는 완료봉 신호와 선택한 체결 가정(같은 종가/다음 관측 시가)을 사용하는 연구 모델입니다. 비용·리스크 가정과 단순 보유 비교를 확인할 수 있지만 실제 장중 체결·지연을 재현하지 않습니다. LTH 일봉은 **성과 평가 불가 진단**, 1분봉은 **제한된 표본**으로 구분하며 실제 3개월 분봉·시간대 검증은 잔여 과제입니다. [최근 전략 전수 분석](docs/research/strategy-review-20261018/report.md)은 기본 파라미터 연구 결과이며 운용 설정 변경이나 수익 보장을 뜻하지 않습니다. [체결 모델 비교](docs/research/replay-execution-20261018/report.md)는 비용·시가 갭이 청산 경로를 바꾸는 사례와 한계를 정리합니다. [손익·비용·현금흐름 비교](docs/research/replay-accounting-20261018/report.md)는 실현/잔여 손익, 가정 청산, 실제 투자비중과 가상 FX·배당 입력을 구분합니다.

## 저장과 웹 접속

- 프로파일·DB 설정: OS 앱 데이터의 `com.countjung.kisautotrade` 디렉터리. 프로파일 비밀키는 평문 저장이므로 파일 권한과 백업 공유 범위를 관리합니다.
- 거래·전략·리스크·예산: 실제 실행 작업 디렉터리(CWD)의 `data/`. 이전 앱 데이터의 `data/`가 있으면 시작 시 이전을 시도하며 실패하면 이전 위치를 계속 사용합니다.
- 로그: 실행 CWD의 `logs/`. DB 전환은 프로파일·로그·비밀 설정을 이관하지 않습니다.

데스크톱 앱이 실행 중이면 기본 `http://localhost:7474`에서 웹 UI를 제공합니다. `dist/` 상태와 경로는 Settings에서 확인합니다. LAN 접속은 데스크톱 Settings에서 명시적으로 허용하고 재시작해야 합니다. 변경 API는 32자 이상의 서버 API token과 브라우저 Bearer token을 요구하며, 조회 GET/HEAD는 이 token으로 보호되지 않습니다. LAN 공개는 조회 자료도 같은 네트워크에 노출하는 선택입니다.

웹의 주문·전략 변경은 지원된 REST 경로와 인증을 사용합니다. DB 자격증명·이관·파괴 작업은 데스크톱 전용입니다. 자세한 [저장 경계](docs/user-guide.md#8-저장과-db-전환)와 [웹 접속 절차](docs/user-guide.md#9-데스크톱과-웹-접속)를 확인하세요.

## 라이선스

[MIT License](LICENSE). 실제 거래 손익과 운용 판단은 사용자에게 있습니다. 모의·실제 자료·fixture·평가 불가 결과를 구분해서 사용하세요.
