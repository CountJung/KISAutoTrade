# Toss 시뮬레이션 응답 본문 파싱 — 2026-10-31

## 1. 범위·상태

- ID: TOSS-PARSE-01. 브랜치: `fix/mac-security-storage-2026-10-06`.
- 제품 기준: `74c303a901040d2922b2c41ea58cbd8c798d2c3d`; 키보드 문서 전용 커밋 `7703f38636f78e25f015a6f7841d7e599a60bb6c` 위의 별도 수정이다.
- 상태: 재현된 로컬 경로 수정·검증·코드 CI/게시 완료(실사용 응답 원인 미확정). 실제 사용자 응답의 원인은 미확정이다.
- 사용자 단서: 시뮬레이션에서 기억상 “utf-8 본문 … 실패”였다. 근사 표현이며 실제 오류 문자열/본문/헤더를 확보한 것으로 취급하지 않는다. 현재 소스의 `토스증권 응답 본문 UTF-8 파싱 실패` 경로를 우선 조사했다. 화면·본문을 재요청하거나 운영 로그를 읽지 않았다.
- 승인 범위: 파싱 실패의 합성 재현·최소 수정·회귀 검증·일반 커밋/푸시. 키보드 UX는 [별도 검토](../completed-history.md#kbd-audit-20261031)와 TODO 반영만이며 기능 구현0이다.

## 2. 조사와 재현

### 정상 계약과 실패 단계

2026-10-31 인증 없는 공식 공개 [OpenAPI JSON](https://openapi.tossinvest.com/openapi-docs/latest/openapi.json)은 v1.2.28/39paths이며 `npm run verify:toss-openapi`가 통과했다. Candle의 timestamp/OHLC/volume/currency 문자열과 현재 DTO가 일치한다. 공식 daily/minute/lastPage 예제3개는 실제 loopback HTTP client 경로로 파싱됐다. JSON 스키마 변경이 실사용 원인이라는 근거는 없다.

기존 reader는4MiB를 제한하며 chunk를 수집한 뒤 strict UTF-8로 변환한다. 정상 다국어 문자열을1byte HTTP chunk로 나눠도 성공했으므로 정상 chunk 경계 분할 자체를 원인으로 단정하지 않는다. 잘린 multibyte·손상 bytes는 계속 실패해야 한다. 빈/HTML/malformed JSON·필수필드 누락/null/number는 UTF-8 실패와 다른 JSON 단계다. 기존 `to_string()` 오류 표면은 세부 단계를 충분히 구분하지 못했다.

### 압축 협상 비교

현재 reqwest0.13.5에는 gzip/br/deflate/zstd 자동 해제 feature가 없고 기존 Toss client의 wire에는 `Accept-Encoding`이 없었다. [RFC9110 §12.5.3](https://www.rfc-editor.org/rfc/rfc9110.html#section-12.5.3)은 헤더 생략을 모든 content-coding 허용으로 볼 수 있게 한다. [reqwest의 자동 gzip 해제](https://docs.rs/reqwest/0.13.5/reqwest/struct.ClientBuilder.html#method.gzip)는 optional feature가 필요하다.

합성 서버를 “헤더 생략이면gzip, identity면동일 평문JSON”으로 구성해 비교했다. 기존 client의 rawgzip는 위 UTF-8 오류로 실패했고 명시identity는 공개lastPage1봉을 정상 파싱했다. 이 재현된 협상 불일치를 수정했다. 작은 gzip fixture161bytes는 공개lastPage JSON200bytes와 동일한 내용을 담는다.

루트의 별도 공개 HTTP/1.1 관측에서는 토스 문서 URL의 헤더 생략/identity가 각각 평문484973bytes였고, gzip 명시는 Content-Encoding:gzip·59208bytes·prefix1f8b0800·strictUTF8 실패였다. 공개host의 gzip 제공은 확인했지만 실제 사용자의 인증된 캔들 응답이 압축됐다는 증거는 아니다. 계좌·캔들·주문 API는 실제 호출하지 않았다.

## 3. 최소 수정과 보존 경계

| 변경 | 결과·보존한 계약 |
|---|---|
| `broker/toss/http.rs`의 Toss 전용 client | 지원 가능한 `Accept-Encoding: identity`를 명시한다. 다른 HTTP client·전역 decoder·Cargo dependency/lock·TLS/root는 그대로다. |
| 캔들 전용 body wrapper | Content-Encoding 전체를 확인해 없음/단일identity만 읽는다. gzip/br/deflate/zstd는 고정label, unknown·복수/쉼표/stacked coding은other로 명시 실패한다. header 원문은 남기지 않는다. |
| 캔들 읽기/UTF-8 실패 | 고정 endpoint·HTTP status·body_limit/body_read/body_utf8와 길이·offset 등 숫자만 outer 오류에 남긴다. raw bytes/본문/URL query·동적ID·토큰과 source의 응답값을 보존하지 않는다. |
| `client.rs`의 캔들 JSON 실패 | 고정 `/api/v1/candles`이고 account_seq가 없는 GET에만 JSON category/line/column을 남긴다. BrokerAdapterError/to_string에도 안전 metadata가 유지된다. DTO 허용 범위는 넓히지 않는다. |
| 나머지 경로 | token·POST·account/publicSpec의 legacy reader/JSON 오류 정책을 유지한다. Toss identity 기본 헤더만 공통 적용되며 401 body-first 순서/최대1회 갱신·GET send실패 재시도1회·POST body실패 비재전송·typed 거절/불명 예산 경계를 유지한다. |
| 자원·자료 의미 |15초 timeout·4MiB 사전/누적/exact 경계를 유지한다. decoder·lossy 변환·charset 추측·HTML 추출·null/number 강제허용·0캔들 성공·수치/시각/전략 변경은 없다. |

소스 추가는 `response_parsing_tests.rs`와 비식별 공개 JSON/gzip fixture2개다. client의 test module 등록 외 제품 변경은 위2파일에 한정한다. 기존 client method33개와 DTO/adapter/error·공통HTTP/TLS·Cargo manifest/lock·프런트엔드·워크플로 불변을 대조했다. 본문의 공통 경로 접두사는 `src-tauri/src/broker/toss/`다.

## 4. 검증 기록

| 단계 | 실제 결과 | 해석 |
|---|---|---|
| 기존 동작 baseline |9 passed |공개예제/UTF8·read/JSON·크기/token/401/typed상태/재시도 경계의 최초 관측 |
| encoding baseline |11 passed |위9개+헤더 생략/identity wire 및 rawgzip/잘린본문 대조 |
| 신규 안전 진단 red |5 passed/6 failed |구현 전 안전 metadata·identity 기대가 실패함을 확인 |
| 첫 수정 |10 passed/1 failed |gzip 비교에 옛 UTF8 기대가 남은 검사 수정 누락; gzip 허용 요구가 아님 |
| 기대 교정/guard 보강 |13 passed |새 content_encoding 실패와 optional cursor/비시장 legacy 유지 |
| 최종 focused |Toss32+인접HTTP/커서5=고유37 passed |신규14개+기존18개, 관련5개 각필터1개. 중간 재실행과 합산하지 않음 |
| 루트 Rust lib |901 passed/0 failed/0 ignored |기존887+신규14 실제 전체 실행.19.05초 테스트 본문, session2384 |
| replay clock 통합 |4 passed/0 failed/0 ignored |생산 replay clock의 기존 독립 integration target |
| check/types/FSD/fmt/clippy/build |전부 exit0·compiler warning0 |offline `cargo check --workspace --all-targets --locked`→app/config tsc→FSD/fmt→lib/replay tests→strict clippy→lib build 순서 |
| 시뮬레이션 UI |30 passed/0 failed(28.2초,session47910 exit0) |기존 격리 production mock의 candle-pagination/replay-assessment/configuration-errors; 실제 backend/provider E2E와구분 |
| CI·코드 게시 |Release Gate53/Security60 success·코드0e69fb7 원격일치 |Rust901/UI408의 실제로그와 후속문서SHA는7절에서구분 |

새14개는 정상 공개예제3개, syntax/data/eof·잘린UTF8/읽기·다국어chunk·4MiB header/stream/exact·token캐시·401·typed거절·GET/POST·identity·비시장legacy·stacked header를 검증한다. 직접 client/DTO fixture를 생산 adapter의 시장/scope 검증으로 확대하지 않는다. 정상KRW 예제에 사용한 test symbol도 실제미국시장 성공의 증거가 아니다. 기존 별도 Toss cursor HTTP와 전체 lib의 계약을 구분한다.

실패 본문·토큰·계좌·cursor sentinel이 새 일반/alternate 오류 chain에 없고, 잘못된 payload가 정상캔들/빈자료로 바뀌지 않는 것을 단언했다. 최종 gzip 비교는 `stage=content_encoding` 조기 거절이며 이전 raw UTF-8 실패 재현과 구분한다. 전체 Rust의 환경 미구성 DB contract5개 조기 반환을 실제DB 성공으로 세지 않는다.

원본·중간/최종 로그·해시·승인 이력은 `/tmp/kis-toss-parse-20261031/`, 구현 보고는 `/tmp/kis-toss-simulation-parse-fix-20261031.md`, 루트 검증은 `.cache/toss-parse-20261031/`다. 비민감 env allowlist로 DB/broker credential을 전달하지 않았고 Cargo는offline으로 실행했다.

## 5. 독립 검토와 문서 게이트

`/tmp/kis-toss-parse-impact-20261031.md`의사전참조분석과 `/tmp/kis-toss-parse-independent-review-20261031.md`의읽기전용독립리뷰에필수P0/P1/P2는없다. Serena전용도구미지원으로rg/소스대조를사용했다. 지도 책임 누락 P2는 보완 후 해소됐고 최종 내용 리뷰에 잔여 필수 지적은 없다. docs-canon119개 Markdown의 링크·표·앵커·도달성 경고0, TODO16·프로젝트지도·Graphify·FSD·lock·diff 검사를 통과했다.

초기조사전용지시를근거로한자동승인검토의1회거절은CreateProcess전미실행이었다. 부모가사용자의실제수정승인원문을새관련근거로전달한뒤같은미실행동작을한번만재요청해승인받았다. 원인비교·수정·검증은그뒤수행했고우회·중복제출하지않았다. 현재승인차단은없다.

Graphify는 code-only/LLM 없이 갱신했다. 기존5개와 새 공개 response fixture JSON1개를 합한6개 JSON의 zero-node 추출 안내는 그래프 도구 한계이며 compiler warning과 구분한다. 코드 source snapshot과 freshness·문서 링크의 최종 게이트를 통과했다.

## 6. 실제 사용자 환경의 한계

이번에수정한것은합성HTTP에서재현된압축협상불일치와안전진단누락이다. 실제사용자응답/정확문구를확보하지않아실사용원인을gzip로확정하거나사용자장애완전해결을주장하지않는다. identity를무시한압축·손상/잘린본문은계속명시실패하며0자료성공으로덮지않는다.

운영앱교체/재실행·실제broker/계좌조회·주문/취소·실DB·키체인·자격증명/운영로그접근·보안설정변경·배포·강제푸시는수행하지않았다. 사용자가같은조건에서확인할때도원문본문/계좌/토큰대신고정stage/status정도의안전정보로구분할수있다. 추가화면요청없이주어진근사단서로독립조사를진행했다.

## 7. 코드·CI·게시 영수증

검증 코드 커밋은 [`0e69fb7a26eb30d3fe0c73f7cc910a8d2a60d53d`](https://github.com/CountJung/KISAutoTrade/commit/0e69fb7a26eb30d3fe0c73f7cc910a8d2a60d53d)이며 작업 브랜치에 일반 푸시한 뒤 원격 SHA 일치·clean을 확인했다.

- [Release Gate #53](https://github.com/CountJung/KISAutoTrade/actions/runs/41182432956): 위 SHA, workflow_dispatch, **success**. frontend/toss-openapi/rust/e2e4개 job이 성공했고 실제 로그의 **901 Rust passed/0 failed/0 ignored**, **408 UI passed/0 failed(7.7m)**를 확인했다. 로컬30UI·7개키보드관측·반복필터를408개에합산하지않는다. DB미구성5개조기반환은실제DB성공이아니다.
- [Dependency Security #60](https://github.com/CountJung/KISAutoTrade/actions/runs/41182558334): 같은 SHA, **success**. npm 운영/전체 감사0 vulnerabilities, cargo는 기존 `RUSTSEC-2023-0071` 예외와 허용 경고2개(`RUSTSEC-2024-0370`,`RUSTSEC-2024-0429`)를 유지한 통과다. 기존 Node20→24/Node deprecation·runner 전환 안내와 compiler warning을 구분한다.
- 두 workflow는 각각1회만 실행했고 중복 dispatch/재실행·실제 릴리스/서명/운영배포를 하지 않았다. source/tests/deps/workflows는 이 코드 SHA에 고정한다.
- 최종 정본 영수증은 별도 문서 커밋으로 남긴다. 실제 CI대상은 위 코드 SHA이며 후속 문서 SHA에서 CI를 새로 실행했다고 주장하지 않는다. 코드/테스트/의존성/워크플로 차이0과 문서·TODO16·지도/Graphify·독립 리뷰를 확인한 뒤 일반 푸시·원격 SHA·게시 파일을 대조한다.

키보드 검토 커밋은 `7703f38636f78e25f015a6f7841d7e599a60bb6c`이며 P2-KBD-01~05는 여전히 미착수다. 이번 파싱 교정은 재현된 로컬 경로의 수정·검증·CI 범위에서 완료됐으며, 실제 사용자의 응답 원인 확정·운영앱 재시도·실계좌 확인은 미실행이다.
