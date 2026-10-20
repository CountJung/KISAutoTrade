# P2-04 이격도 소유·진입·청산 연구 — 2026-10-20

상태: 완료. P2-04의 실제 소유/미체결 경계를 교정하고 사전 고정 후보를 각각 비교했다.

## 범위와 완료 조건

- 기존 현재 포함 MA20/매수−5%이하·매도+5%이상과 guard/risk/exact owner 경계를 보존한다.
- 관측과 실제 소유를 분리하고 무보유 매도·보유 중 매수 후보를 제거한다. 미체결/차단·부분체결·설정/준비자료 변경·재시작 보호를 검증한다.
- 선택 진입 하향 교차, 직전200관측 장기 추세,63관측 보유,평균체결가 대비5%관측 손절을 각각 단독 비교한다. 기본은 level/다른옵션0이며 연구 목적의 운용 설정 변경은 없다.
- 실시간 약10초 관측과 선택 봉 replay를 구분한다. 신규 longtrend의 live 버퍼에는 일봉/1분봉 warmup을 섞지 않는다. maxHold은 체결 뒤 관측 수이며 실제체결을 보장하는 날짜 상한이 아니다.
- 신규 params의 UI/저장·단기/연속 preview·A/B 표시와 오류 복구를 함께 검증한다.
- 테스트·독립 stock 감사·문서/지도/그래프 게이트 후 커밋하고 현재 브랜치를 origin에 푸시한다.

## 사전 고정 연구

정본: [design.json](../research/deviation-policy-20261020/design.json). frozen strategy-review20261018 입력 SHA는 사전 선언에 고정한다. archived bca6601 생산 라이브러리와 새 v15를 분리하고 비이격도1,122회 전체 report/signals exact·이격도102회 차이를 측정한다. 별도36연속/36대체분할/432분기 초기화 계좌를 실행한다. SOXQ/SPY/QQQ 각 strict252준비+2023~2025의752봉, 최초 수량51/2/3·현금1천만원·고정FX1450·nextOpen·수수료/슬리피지각10/20bp·overlay0을 유지한다.

사후 진단이며 이미 관측한 상관ETF3개·작은 청산 표본이다.2026봉은 사용하지 않지만 독립 OOS로 부르지 않는다. 분기마다 새1천만원 계좌이며 수익률 합산/복리하지 않는다. raw감소와 실제체결·비용·손익을 구분하고 낮은 노출·미청산 이익을 우수성으로 표현하지 않는다.

## 위임과 안전

symbol_navigator가 Serena로 실제 definitions/callers·factory/default·save/two preview·exact owner 피드백/수량 clamp를 추적했다. backend와 UI를 분리 위임하고 stock_analyst가 사전 계약과 비용/손실·표본 해석을 감사한다. 파일/책임 구조는 project_mapper가 마무리한다. 새 helper 후보는 도메인 의미를 검증하며 기존 공유 숫자 초안 parser를 사용한다.

실제·모의 주문, 자동매매 시작, 계좌·자격증명·운영 DB는 사용하지 않는다. 고정 공개 가격과 합성/mock fixture만 사용한다. runtime옵션 오류로 기존 보유 청산을 차단하지 않고 신규 매수는 fail closed한다.

## 검증과 전달

구현 후 focused/전체 Rust·Clippy·tsc/FSD/lockfiles/web build·focused/전체 브라우저·TODO renderer를 실행한다. 독립 장부 대사와 입력/코드 hash를 보관하고 실제 provider/E2E/장중손실한도로 확대하지 않는다. 결과와 미실행 범위는 검증 후 기록한다. rollback은 해당 커밋 revert다.

## 검증 결과와 발견 수정

| 검증 | 결과 |
|---|---|
| Rust fmt/check | workspace/all-targets/locked 통과, 경고0 |
| Rust 전체 | 첫 전체 실행은 lib582 중581성공/1실패. 옛 stateless 3전략 테스트가 Deviation의 None을 요구한 fixture를 RSI/MA와 실제 소유 Deviation manager 회귀로 분리한 뒤 최종 lib583+통합4+doc3=590/590 통과 |
| focused Rust | Deviation32개, 최종 manager live feedback19개 통과. focused 개수는 전체와 합산하지 않음 |
| Clippy | 최종 workspace/all-targets/all-features/locked -D warnings 통과 |
| TypeScript/FSD/lockfiles | 모두 통과 |
| web build | 경고0, main341.63kB/charts175.25kB. 기존 Vite chunk 경계/설정 유지 |
| Chromium | 저장 응답 수정 뒤 전체155/155(2.7분). 테스트만 추가한 queued A→B→A 경계1개 통과, 해당 spec 최종18/18(21.3초) 통과. 이를 단일156/156 실행으로 합쳐 기록하지 않음 |
| 생산 연구 | archived bca6601 flat1224·연속6/분할6/분기72, 최종 v15 비이격도1122 exact·이격도102 delta·연속30/분할30/분기360. 합36/36/432 |
| 부모 기록 정산 | 468원장68,421개 산술 검사 통과 |
| 독립 stock 감사 | 468원장199,065+flat102의1,430=200,495검사 오류0. 최대 약0.01원/0.000050022%p 출력 반올림 차이 |

기준 실행 첫 시도는 복사한 실행기의 출력 폴더 이름이 이전 suffix라 실행 전 파일 열기에서 실패했다. 경로를 이번 연구 경로로 바로잡고 기준 전량 재실행했다. 후보 첫 실행 뒤 readiness 진단·손절 부동소수 경계를 보강해 **21:56:51 KST 최종 실행 소스로 전체 후보를 재실행**한 결과만 gzip/SHA에 보관한다. 이전 결과는 ignored cache에 남으며 최종 연구로 집계하지 않는다. 독립 감사는 최종 SHA의468원장을 확인했다.

기본 correctedLevel은78/78개 회계·자산곡선·filled 원기록이 legacy와 같다. flat102개 회계/경제도 같고60개에서 신호/차단 수만 바뀌었다. 비용별 raw305→23/차단286→4는 수수료 절감이 아니다. crossing은 연속 동일/분기1개 개선, trend200은 낮은 노출/분기3개 개선4악화, maxHold63은 분기 동일/guard에 따른 비용별 실제 경로 차이, stop5는 SOXQ20bp 수익·MDD 악화다. 기본 level/나머지0을 유지하며 선택 후보를 결합/최적화하지 않았다.

독립 분기 계좌는 합산/복리하지 않았다. same-path는 filled reason만 제외한 실제 기록/회계/자산 경로, 원기록 전체 동일성은 별도로 보존한다. 독립 감사는 partition36개 hash 대응을 확인했으며 대체 payload를 독립 재실행한 것으로 집계하지 않는다. 실제 provider/Tauri 주문·운영 DB·장중 체결은 실행하지 않았다.

## UI 저장 경합과 helper 게이트

새 숫자 입력의 저장 완료 전 재편집 검사에서 기존 onSuccess의 무조건 map삭제가 새 초안을 지우는 문제가 드러났다. 일반/PC/LTH의 전송 payload에 해당하는 immutable 초안만 정리하고 scope epoch로 A→B→A 과거 callback을 차단했다. 세 실제 지연 저장 회귀에서 새 초안과 두 번째 전송 payload를 확인했다. 새로운 snapshot merge/세션 기본값은 각 handler에 유지한다.

helper_curator가 Graphify CLI와 Serena 정의/모든 참조로 두 순수 동작의 동등성을 확인했다. 세 controls의 타입·유한수·양끝 포함 범위·선택 정수 판정은 isFiniteNumberInRange로, 세 저장7회 map정리는 removeDraftIfUnchanged로 src/shared/lib 공개 진입점에 승격했다. field별 기본값/범위/mode 관계/문구/raw입력과 payload/scope epoch는 local 유지했다. Rust observation/crossing/trend/age/stop과 Consecutive completed일봉 context는 의미가 달라 합치지 않았다. curator가 최종 구현도 읽기 전용으로 승인했다.

## 구조·문서 게이트

project_mapper가 새23파일/디렉터리2개 인벤토리와 책임·데이터 흐름을 갱신했다. 연구14파일 중 gzip2개만 포함하고 평문 JSONL은 캐시로 이동했다. map checker와 맵 링크16개를 확인했으며 제안한 UI 스킬의 공통 판정/초안 저장 설명 drift도 수정했다. Graphify canonical/HTML/report의 노드5667·간선12412·커뮤니티522 대응과 freshness를 확인했다. TODO19개와 renderer3/3, map checker·전체변경문서 상대 링크161개·연구입력/코드/기록 해시87개·최근AGENTS5건·변경 source1000행미만·민감/캐시제외도 최종 확인했다.

## 잔여 범위와 다음 작업

독립 감사의 core/연구 해석 P1/P2 잔여는 없다. 실제 운영 원장·provider coverage·LTH 분봉·배포 키와 사용자 판단 항목은 기존 주황 미완료 표시를 유지했다. 다음 자동 항목은 P2-10 평균회귀 청산 후보와 P2-01/02다. 현재 브랜치에 검증 후 커밋·origin 푸시하며 force push하지 않는다. 롤백은 변경 커밋 revert다.

추가 A→B→A UI 검증은 원래 mutation observer reset으로 옛 per-call callback이 폐기되는 우연한 성공을 피했다. test-only 모듈 응답 wrapper가 실제 Page onSuccess closure를 캡처하고 지연 저장200 성공 뒤 명시 재전달해 새A의 mode/stop9/primitive session pre와 재저장payload 보존을 확인했다. 생산 소스는 바꾸지 않았다.
