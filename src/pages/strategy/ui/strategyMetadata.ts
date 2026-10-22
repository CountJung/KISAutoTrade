export interface StrategyParamMeta {
  key: string
  label: string
  min: number
  max: number
  step?: number
  description: string
}

export const STRATEGY_PARAM_META: Record<string, StrategyParamMeta[]> = {
  ma_cross: [
    { key: 'short_period', label: '단기 MA', min: 2, max: 50, description: '단기 이동평균 입력 관측 수 (기본 5, 실시간 약10초/선택 봉)' },
    { key: 'long_period', label: '장기 MA', min: 5, max: 200, description: '장기 이동평균 입력 관측 수 (기본 20, 완료 일봉 필터와 별개)' },
    { key: 'long_trend_period', label: '진입 완료 일봉 추세 MA 기간', min: 0, max: 512, step: 1, description: '0 비활성 · 현재 후보 가격 > 이전 완료 일봉 N종가 평균. 이전 N봉 필요' },
    { key: 'min_long_ma_slope_pct', label: '최소 장기 일봉 MA 기울기 (%)', min: 0, max: 100, step: 0.1, description: '0 비활성 · 이전 장기 MA / 한 일봉 전 장기 MA의 변화율 이상. long_period+1 이전 일봉 필요' },
    { key: 'min_atr_pct', label: '최소 일봉 ATR 비율 (%)', min: 0, max: 100, step: 0.1, description: '0 비활성 · 이전 완료 OHLC 15봉의 TR14 산술평균 / 이전 종가 ×100 이상' },
    { key: 'min_bandwidth_pct', label: '최소 일봉 볼린저 밴드폭 (%)', min: 0, max: 1000, step: 0.1, description: '0 비활성 · 이전 20종가의 4×모집단 표준편차 / 평균 ×100 이상' },
  ],
  rsi: [
    { key: 'period', label: 'RSI 기간', min: 5, max: 50, description: '입력 가격 변화의 단순평균 RSI 기간 (기본 14, Wilder 아님)' },
    { key: 'oversold', label: '과매도 기준', min: 10, max: 40, step: 1, description: '이전 RSI <= 기준 → 현재 RSI > 기준에서 미보유 매수 후보 (기본 30)' },
    { key: 'overbought', label: '과매수 기준', min: 60, max: 90, step: 1, description: '이전 RSI >= 기준 → 현재 RSI < 기준에서 실제 보유 잔량 매도 후보 (기본 70)' },
    { key: 'long_trend_period', label: 'RSI 진입 추세 MA 관측 수', min: 0, max: 512, step: 1, description: '0 비활성 · 미보유 반등 교차 후보의 현재 가격 > 같은 주기의 직전 N가격 평균, 현재 관측 제외' },
    { key: 'stop_loss_pct', label: 'RSI 보유 손절 기준 (%)', min: 0, max: 99.9, step: 0.1, description: '0 비활성 · 종목 통화 기준 실제 평균 매수가 대비 관측 손실률이 입력 이상이면 잔량 전량 매도 후보' },
    { key: 'max_hold_observations', label: 'RSI 최대 보유 관측 수', min: 0, max: 1000000, step: 1, description: '0 비활성 · 실제 양수 체결 후 관측 횟수. 일수/보장 보유시간과 다름' },
  ],
  momentum: [
    { key: 'lookback_period', label: '비교 기간', min: 5, max: 60, description: 'N기간 전 가격 대비 변화율 계산 기간 (기본 20)' },
    { key: 'threshold_pct', label: '임계값 (%)', min: 1, max: 20, step: 0.1, description: '진입은 양의 임계값 이상, 보유 청산은 음의 임계값 이하 (기본 5.0)' },
    { key: 'entry_hysteresis_pct', label: '재무장 간격 (%p)', min: 0, max: 100, step: 0.1, description: 'crossing 전용 재무장 간격, 기본 0. level에서는 미사용하며 crossing에서는 임계값 이하' },
  ],
  deviation: [
    { key: 'ma_period', label: 'MA 기간', min: 5, max: 120, description: '이격도 기준 이동평균 기간 (기본 20)' },
    { key: 'buy_threshold_pct', label: '매수 이격 (%)', min: -20, max: -1, step: 0.1, description: '현재가가 MA 대비 이 % 이하이면 매수 (기본 -5.0, 음수)' },
    { key: 'sell_threshold_pct', label: '매도 이격 (%)', min: 1, max: 20, step: 0.1, description: '현재가가 MA 대비 이 % 이상이면 매도 (기본 5.0)' },
    { key: 'long_trend_period', label: '진입 추세 MA 관측 수', min: 0, max: 512, step: 1, description: '0 비활성 · 현재 가격 > 같은 주기의 직전 N가격 MA, 현재 관측 제외. 실시간 시작 후 N관측 준비' },
    { key: 'max_hold_observations', label: '최대 보유 관측 수', min: 0, max: 1000000, step: 1, description: '0 비활성 · 실제 보유 확인 후 가격 관측 수. 일봉 63관측과 실시간 63관측의 기간은 다름' },
    { key: 'stop_loss_pct', label: '보유 손절 기준 (%)', min: 0, max: 100, step: 0.1, description: '0 비활성 · 실제 native 평균가 대비 현재 관측가의 손실률 이상에서 매도 후보. 갭/체결 모델 영향 있음' },
  ],
  fifty_two_week_high: [
    { key: 'lookback_days', label: '조회 기간 (거래일)', min: 60, max: 504, step: 1, description: '52주 신고가 계산을 위한 과거 거래일 수 (기본 252 ≈ 1년)' },
    { key: 'stop_loss_pct', label: '손절 기준 (%)', min: 1, max: 15, step: 0.1, description: '매수가 대비 이 % 이상 하락 시 손절 매도 (기본 3.0)' },
  ],
  consecutive_move: [
    { key: 'buy_days', label: '연속 상승 관측 횟수', min: 2, max: 10, step: 1, description: 'N회 연속 가격관측 상승 시 매수 (기본 3, 실시간 약10초 관측/replay 선택 봉 종가)' },
    { key: 'sell_days', label: '연속 하락 관측 횟수', min: 2, max: 10, step: 1, description: 'M회 연속 가격관측 하락 시 매도 (기본 3, 신규 진입 필터와 독립)' },
    { key: 'min_cumulative_move_pct', label: '최소 누적 상승률 (%)', min: 0, max: 100, step: 0.1, description: 'N회 상승 구간의 누적 상승률 하한. 0은 비활성, 신규 진입만 확인' },
    { key: 'min_move_atr_multiplier', label: '최소 상승폭 ATR 배수', min: 0, max: 20, step: 0.1, description: 'N회 가격 상승폭이 직전 완료 OHLC의 ATR14 × 배수 이상. 0은 비활성' },
    { key: 'long_trend_period', label: '진입 장기 MA 기간', min: 0, max: 512, step: 1, description: '현재 후보 가격이 직전 완료 종가 MA보다 높을 때만 진입. 0은 비활성, 정수 관측 수' },
  ],
  failed_breakout: [
    { key: 'lookback_days', label: '전고점 기간', min: 5, max: 60, step: 1, description: '기본 이전 가격관측 수 또는 선택 완료 일봉 수 (기본 20, 현재 제외)' },
    { key: 'buffer_pct', label: '돌파 버퍼 (%)', min: 0.1, max: 5, step: 0.1, description: '이전 기준 고점 대비 추가 비율 (기본 0.5), 실패 기준은 버퍼 전 고점' },
    { key: 'atr_buffer_multiplier', label: '돌파 일봉 ATR 버퍼 배수', min: 0, max: 20, step: 0.1, description: '0 비활성 · 기존 비율 문턱(내림)과 기준 고점 + 이전 완료 일봉 TR14 × 배수 중 큰 가격. ATR 0도 준비 가능' },
    { key: 'min_volume_ratio', label: '최소 완료 일봉 거래량 비율', min: 0, max: 100, step: 0.1, description: '0 비활성 · 직전 완료 일봉량 / 그 이전 20일 평균. 총 21개 이전 봉 필요' },
    { key: 'profit_activation_pct', label: '돌파 수익 보호 시작 (%)', min: 0, max: 1000, step: 0.1, description: '실제 평균 매수가 대비 이익률 이상에서 한 번 무장. 되밀림과 함께 양수 또는 둘 다 0' },
    { key: 'profit_trailing_pct', label: '돌파 수익 보호 되밀림 (%)', min: 0, max: 99.9, step: 0.1, description: '실제 보유 중 관측 최고가 대비 되밀림률 이상이면 잔량 전량 매도 후보' },
    { key: 'max_hold_observations', label: '돌파 최대 보유 관측 수', min: 0, max: 1000000, step: 1, description: '0 비활성 · 실제 양수 체결 뒤 관측 수. 일수나 보장 보유시간이 아님' },
  ],
  strong_close: [
    { key: 'threshold_pct', label: '강한 종가 기준 (%)', min: 0.5, max: 10, step: 0.1, description: '이전 완료 일봉 (고가−종가)/고가×100 ≤ 기준 (기본 3.0). 양봉·상승률 조건 아님' },
    { key: 'stop_loss_pct', label: '손절 기준 (%)', min: 0, max: 10, step: 0.1, description: '실제 종목 통화 평균가 대비 관측 손실률 ≥ 기준 (기본 3.0). 0도 평균가 이하 청산' },
    { key: 'long_trend_period', label: '강한 종가 진입 완료 일봉 추세 MA 기간', min: 0, max: 512, step: 1, description: '0 비활성 · 현재 후보 가격 > 이전 완료 일봉 N종가의 산술평균, 현재 관측 제외' },
    { key: 'profit_activation_pct', label: '강한 종가 수익 보호 시작 (%)', min: 0, max: 1000, step: 0.1, description: '실제 평균 매수가 대비 이익률 이상에서 한 번 무장. 되밀림과 함께 양수 또는 둘 다 0' },
    { key: 'profit_trailing_pct', label: '강한 종가 수익 보호 되밀림 (%)', min: 0, max: 99.9, step: 0.1, description: '실제 보유 중 관측 최고가 대비 되밀림률 이상이면 실제 잔량 전량 매도 후보' },
    { key: 'max_hold_observations', label: '강한 종가 최대 보유 관측 수', min: 0, max: 1000000, step: 1, description: '0 비활성 · 실제 양수 체결 뒤 관측 횟수. 일수·보장 보유시간이 아님' },
  ],
  volatility_expansion: [
    { key: 'lookback_days', label: '평균 기간 (거래일)', min: 3, max: 60, step: 1, description: '평균 변동폭 계산에 사용할 과거 거래일 수 (기본 10)' },
    { key: 'expansion_factor', label: '확장 배율', min: 1.1, max: 5, step: 0.1, description: '당일 변동폭이 평균의 이 배 이상이면 매수 (기본 2.0)' },
    { key: 'stop_loss_pct', label: '손절 기준 (%)', min: 1, max: 10, step: 0.1, description: '매수가 대비 이 % 이상 하락 시 손절 (기본 3.0)' },
  ],
  mean_reversion: [
    { key: 'period', label: '볼린저 밴드 기간', min: 5, max: 120, step: 1, description: '이동평균과 표준편차 계산 기간 (기본 20)' },
    { key: 'std_dev', label: '표준편차 배율', min: 0.5, max: 4, step: 0.1, description: '상/하단 밴드 너비 조정 (기본 2.0 = ±2시그마)' },
    { key: 'stop_loss_pct', label: '손절 기준 (%)', min: 0, max: 15, step: 0.1, description: '실제 native 평균가 대비 관측가 손절 (기본 5.0). 0도 비활성이 아니라 원가 이하 손절' },
    { key: 'long_trend_period', label: '진입 추세 MA 관측 수', min: 0, max: 512, step: 1, description: '0 비활성 · 현재 가격 > 같은 주기의 직전 N가격 MA, 현재 관측 제외. 실시간 시작 후 N관측 준비' },
    { key: 'max_hold_observations', label: '최대 보유 관측 수', min: 0, max: 1000000, step: 1, description: '0 비활성 · 실제 양수 보유 후 가격 관측 수. 일봉 63봉과 실시간 63관측은 기간이 다름' },
  ],
  trend_filter: [
    { key: 'long_period', label: '장기 MA 기간', min: 50, max: 500, step: 1, description: '장기 추세 판단 기준 이동평균 기간 (기본 200일)' },
    { key: 'short_period', label: '단기 MA 기간', min: 2, max: 30, step: 1, description: '단기 모멘텀 판단 이동평균 기간 (기본 5일)' },
    { key: 'mid_period', label: '중기 MA 기간', min: 5, max: 60, step: 1, description: '중기 추세 비교 이동평균 기간 (기본 20일)' },
  ],
}

export const STRATEGY_DESCRIPTION: Record<string, string> = {
  ma_cross: '입력 관측의 단기/장기 MA 골든크로스 매수, 데드크로스 매도. 기본 5/20은 실시간 약10초 또는 선택 봉 관측 수이며 5/20일이 아니다. 선택 신규 진입 확인은 별도의 이전 완료 일봉 추세·기울기·ATR·밴드폭을 사용한다.',
  rsi: '가격 변화 단순평균 RSI의 과매도 반등 교차는 미보유 매수, 과매수 하락 교차는 실제 보유 잔량 청산 후보. 기존 기간14·30/70을 유지하며 선택 추세·손절·보유 관측 상한은 0 비활성이다. 관측 수를 일수로 해석하지 않는다.',
  momentum: 'N관측 전 대비 변화율의 기본 level 진입과 선택 crossing 상향 돌파. 재무장 간격은 %p 단위이며 level에서는 미사용이다. 실제 보유는 두 모드 모두 음의 임계값 이하 시 청산하고 실시간 약10초 관측과 replay 봉 종가를 구분한다.',
  deviation: '기본 MA20·매수 -5%·매도 +5%의 level 진입과 보유 청산. 선택 하향 crossing·이전 가격 MA 진입 확인·최대 보유 관측 수·실제 평균가 손절을 비교할 수 있다. 관측 수를 일수로 해석하지 않는다.',
  fifty_two_week_high: '최근 252 거래일(1년) 최고가를 재돌파하면 매수. 매수 후 지정 % 하락 시 손절. 자동매매 시작 시 KIS 차트 API로 초기화됨.',
  consecutive_move: 'N회 연속 가격관측 상승 시 매수, M회 연속 하락 시 매도. 실시간은 약10초 관측, replay는 선택 봉 종가이며 N일을 뜻하지 않는다. 선택 진입 필터는 모두 충족해야 하며 기존 보유 청산에는 적용하지 않는다.',
  failed_breakout: '기본은 이전 N가격관측 최대값 돌파 매수, 실제 보유에서 진입 전 기준 고점 미만이면 잔량 매도 후보. 선택 이전 완료 일봉 고가·ATR·거래량과 수익 보호·보유 관측 상한은 기본 비활성이다. 신호는 체결이 아니며 관측 수는 일수가 아니다.',
  strong_close: '이전 완료 일봉의 (고가−종가)/고가×100 ≤ 기준에서 미보유 매수 후보. 양봉·상승률 조건이 아니며 고가=종가도 통과한다. 실제 평균가 손절과 선택 완료 일봉 추세·수익 보호·보유 관측 상한을 확인한다. 기본 기준3·손절3을 유지하며 새 네 옵션만 누락/0 비활성이다.',
  volatility_expansion: '당일 변동폭(고-저)이 최근 N거래일 평균 변동폭의 K배 이상이며 현재가 > 시가인 경우 매수. 장중 변동성 폭발 구간에 상승 방향으로 편승. 매수 후 지정 % 하락 시 손절.',
  mean_reversion: '현재 관측을 포함한 볼린저 하단 미만에서 매수하고 기본 상단 초과 또는 선택 중심선 초과에서 보유 청산. 기본 기간20·표준편차2·손절5를 유지한다. 선택 추세·보유 상한의 0만 비활성이며 기존 손절 0은 원가 이하 손절이다.',
  trend_filter: '장기 MA(기본 200일) 위에서 단기 MA(5일)가 중기 MA(20일)를 상회할 때만 매수(이중 추세 확인). 현재가가 장기 MA 아래로 하락하면 추세 반전으로 판단하여 청산. 자동매매 시작 시 과거 종가로 버퍼 적재.',
  leveraged_trend_hold: '선택한 레버리지 ETF 자체의 상승 추세에서는 매수하고, 고점 대비 하락·EMA 이탈·RSI 약화·장마감 전에는 청산한다. 롱/숏 ETF 모두 같은 방식으로 다룬다.',
  price_condition: '지정가 이하에서 자동 매수. 매수 후 지정가 또는 설정 % 이상 상승 시 익절 매도, 손절 % 이하 하락 시 손절. 가격/비율 조건을 각각 설정하거나 조합해서 사용 가능. 0은 해당 조건 비활성.',
}

export function getStrategyType(id: string): string {
  if (id.startsWith('ma_cross')) return 'ma_cross'
  if (id.startsWith('rsi')) return 'rsi'
  if (id.startsWith('momentum')) return 'momentum'
  if (id.startsWith('deviation')) return 'deviation'
  if (id.startsWith('fifty_two_week_high')) return 'fifty_two_week_high'
  if (id.startsWith('consecutive_move')) return 'consecutive_move'
  if (id.startsWith('failed_breakout')) return 'failed_breakout'
  if (id.startsWith('strong_close')) return 'strong_close'
  if (id.startsWith('volatility_expansion')) return 'volatility_expansion'
  if (id.startsWith('mean_reversion')) return 'mean_reversion'
  if (id.startsWith('trend_filter')) return 'trend_filter'
  if (id.startsWith('leveraged_trend_hold')) return 'leveraged_trend_hold'
  if (id.startsWith('price_condition')) return 'price_condition'
  return 'unknown'
}
