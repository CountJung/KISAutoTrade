# 정확성 수정 재실행: rolling 52주 고가와 replay ATR

2026-10-18. 기존 [전수 연구](../strategy-review-20261018/report.md)의 입력·수량·두 비용을 고정하여 생산 라이브러리 v8을 실행했다. 기존 입력/결과/검증 hash는 수정하지 않았다. 전체 trace는 저장소 내부 `.cache/research/strategy-review-20261018/`에 두고 [비교 결과](comparison.json)에 입력 trace hash와 차이를 보관한다.

## 범위와 결과

- 일반 전략 1,224회, generic LTH 지원 불가 102회, 같은 수량 benchmark 102회, 실제 분봉 LTH 진단 4회: 총 1,432행이다.
- 생성 시각·엔진/전략 버전·그 버전이 포함된 입력 hash·추가 ATR 준비 진단만 제외하면 **1,402행 일치**, **52주 신고가 30행 차이**다. 거래·equity·신호·차단·기존 준비 상태를 비교에서 제외하지 않았다. 차이는 손익 외 신호/차단 변화도 포함한다.
- 기본 ATR 수량 산정은 비활성 상태다. 따라서 이 전수 재실행으로 ATR 수량 조정의 성과 개선을 주장하지 않는다. ATR 유효성은 별도 합성 회귀에서 검증한다.
- 차이 30행 중 summary가 바뀐 것은 12행(6창×두 비용)이고 18행은 reason 등의 내용 차이다. 두 비용 모두 기말 보유 창이 24→26으로 늘어 중앙값 상승을 완료 거래의 개선으로 볼 수 없다. 전담 주식 분석 에이전트가 입력·수량·비용·비교 수치와 이 해석을 독립 확인했다.

완전한 45개 종목·분기 창의 52주 신고가 결과:

| 수수료·슬리피지 각각 | 엔진 | 순평가수익 중앙값 | 최악 수익률 | 최악 MDD | 완료 거래 |
|---|---|---:|---:|---:|---:|
| 10bps | v7 | 0% | -1.2416% | 3.0127% | 14 |
| 10bps | v8 | +0.0248% | -1.2416% | 3.0127% | 14 |
| 20bps | v7 | 0% | -1.3617% | 3.0136% | 14 |
| 20bps | v8 | +0.0069% | -1.3617% | 3.0136% | 14 |

오래된 최고가 만료와 이후 완료 일봉의 high 적재를 고친 결과다. 상관된 ETF 세 종목·분기 리셋·작은 완료 표본이므로 수익성이나 실운영 안전성의 근거로 확대하지 않는다. 기존 same-close 체결 가정·기말 미청산·고정 환율·배당 제외 한계도 유지된다. 같은 수량 benchmark 대비 초과수익 중앙값은 -0.4323%/ -0.4492%, 초과 창도 9/45·7/45로 유지된다.

## 구현과 남은 경계

52주 고가는 완료 일봉 ring의 최대값을 매번 재계산하며 tick으로 고가나 거래일 수를 갱신하지 않는다. day-start 뒤 완료를 한 번 소비하고 같은 날 반복 tick은 고가를 만료하지 않는다. 준비 부족이어도 실제 보유 손절은 유지한다. generic 일봉 replay에는 연결됐지만 **live 운용 중 완료 일봉 공급 연결은 P2-05 잔여**다.

replay ATR은 이전 close를 포함한 TR14개를 bounded window로 계산한다. 처음 1봉은 이전 종가 기준만 세우므로 같은 interval 관측15봉부터 준비된다. generic 사전자료는 지표만 준비하고 주문·손익에는 포함하지 않는다. 분봉 LTH에 일봉 ATR history를 넣지 않는다. 수량 산정 활성 시 ATR 부족/변동폭0/위험0bps는 신규 매수를 차단하고 매도는 유지한다. 준비 진단과 설정은 연구 UI에서 확인한다.

전용 LTH v4는 실제 체결 수량·평균가 snapshot을 다음 진입/청산 판단에 반영한다. 수량 확대·축소 뒤 실제 보유 전량 매도, 부분매도 뒤 잔여 보호, 슬리피지 평균가에 따른 손절을 회귀 검증한다. 기존 raw 연구용 bool API는 보존했다. 네 실제 분봉 진단은 무신호이므로 이 새 수량 피드백 경로의 시장 검증이 아니다.

표본 부족 표시·별도 walk-forward는 P2-16, provider 페이지·완료 일봉 연결은 P2-17/P2-05, 장중/next-open 체결과 청산 비용은 P2-14/P2-15에 남는다. 기존 3개월 LTH 자료 확보 조건도 유지한다.

## 재현

```bash
cargo run --offline --locked --manifest-path .cache/research/strategy-review-20261018/Cargo.toml --bin strategy-review-runner -- docs/research/strategy-review-20261018/input.json .cache/research/strategy-review-20261018/autonomous-v8-results.jsonl
python3 docs/research/autonomous-corrections-20261018/compare.py .cache/research/strategy-review-20261018/full-results.jsonl .cache/research/strategy-review-20261018/autonomous-v8-results.jsonl
```

실제 provider 조회·주문·자동매매 시작을 실행하지 않는다. `.cache`는 추적하지 않으며 기존 연구의 manifest/runner 준비 절차는 원 보고서를 따른다.

생성시각을 제외한 v8 재실행 1,432행의 전체 결과가 동일함을 확인했다. 반복 trace hash는 `comparison.json`에 기록한다. 재현 시 비교 명령의 세 번째 경로로 반복 결과를 주면 이 일치를 검사한다.
