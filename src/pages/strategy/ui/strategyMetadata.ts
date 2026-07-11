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
    { key: 'short_period', label: '단기 MA', min: 2, max: 50, description: '단기 이동평균 기간' },
    { key: 'long_period', label: '장기 MA', min: 5, max: 200, description: '장기 이동평균 기간' },
  ],
  rsi: [
    { key: 'period', label: 'RSI 기간', min: 5, max: 50, description: 'RSI 계산 기간 (기본 14)' },
    { key: 'oversold', label: '과매도 기준', min: 10, max: 40, step: 1, description: 'RSI가 이 이하 → 이 이상 시 매수 신호 (기본 30)' },
    { key: 'overbought', label: '과매수 기준', min: 60, max: 90, step: 1, description: 'RSI가 이 이상 → 이 이하 시 매도 신호 (기본 70)' },
  ],
  momentum: [
    { key: 'lookback_period', label: '비교 기간', min: 5, max: 60, description: 'N기간 전 가격 대비 변화율 계산 기간 (기본 20)' },
    { key: 'threshold_pct', label: '임계값 (%)', min: 1, max: 20, step: 0.1, description: '매매 발동 최소 변화율 % (기본 5.0)' },
  ],
  deviation: [
    { key: 'ma_period', label: 'MA 기간', min: 5, max: 120, description: '이격도 기준 이동평균 기간 (기본 20)' },
    { key: 'buy_threshold_pct', label: '매수 이격 (%)', min: -20, max: -1, step: 0.1, description: '현재가가 MA 대비 이 % 이하이면 매수 (기본 -5.0, 음수)' },
    { key: 'sell_threshold_pct', label: '매도 이격 (%)', min: 1, max: 20, step: 0.1, description: '현재가가 MA 대비 이 % 이상이면 매도 (기본 5.0)' },
  ],
  fifty_two_week_high: [
    { key: 'lookback_days', label: '조회 기간 (거래일)', min: 60, max: 504, step: 1, description: '52주 신고가 계산을 위한 과거 거래일 수 (기본 252 ≈ 1년)' },
    { key: 'stop_loss_pct', label: '손절 기준 (%)', min: 1, max: 15, step: 0.1, description: '매수가 대비 이 % 이상 하락 시 손절 매도 (기본 3.0)' },
  ],
  consecutive_move: [
    { key: 'buy_days', label: '연속 상승 횟수', min: 2, max: 10, step: 1, description: 'N일 연속 종가 상승 시 매수 (기본 3)' },
    { key: 'sell_days', label: '연속 하락 횟수', min: 2, max: 10, step: 1, description: 'M일 연속 종가 하락 시 매도 (기본 3)' },
  ],
  failed_breakout: [
    { key: 'lookback_days', label: '전고점 기간', min: 5, max: 60, step: 1, description: '전고점 계산을 위한 과거 기간 (기본 20)' },
    { key: 'buffer_pct', label: '돌파 버퍼 (%)', min: 0.1, max: 5, step: 0.1, description: '전고점 대비 돌파로 인정하는 추가 % (기본 0.5)' },
  ],
  strong_close: [
    { key: 'threshold_pct', label: '강한 종가 기준 (%)', min: 0.5, max: 10, step: 0.1, description: '종가가 고가 대비 이 % 이내이면 실제로 강한 종가로 판단 (기본 3.0)' },
    { key: 'stop_loss_pct', label: '손절 기준 (%)', min: 1, max: 10, step: 0.1, description: '매수가 대비 이 % 이상 하락 시 손절 (기본 3.0)' },
  ],
  volatility_expansion: [
    { key: 'lookback_days', label: '평균 기간 (거래일)', min: 3, max: 60, step: 1, description: '평균 변동폭 계산에 사용할 과거 거래일 수 (기본 10)' },
    { key: 'expansion_factor', label: '확장 배율', min: 1.1, max: 5, step: 0.1, description: '당일 변동폭이 평균의 이 배 이상이면 매수 (기본 2.0)' },
    { key: 'stop_loss_pct', label: '손절 기준 (%)', min: 1, max: 10, step: 0.1, description: '매수가 대비 이 % 이상 하락 시 손절 (기본 3.0)' },
  ],
  mean_reversion: [
    { key: 'period', label: '볼린저 밴드 기간', min: 5, max: 120, step: 1, description: '이동평균과 표준편차 계산 기간 (기본 20)' },
    { key: 'std_dev', label: '표준편차 배율', min: 0.5, max: 4, step: 0.1, description: '상/하단 밴드 너비 조정 (기본 2.0 = ±2시그마)' },
    { key: 'stop_loss_pct', label: '손절 기준 (%)', min: 1, max: 15, step: 0.1, description: '매수가 대비 이 % 이상 하락 시 손절 (기본 5.0)' },
  ],
  trend_filter: [
    { key: 'long_period', label: '장기 MA 기간', min: 50, max: 500, step: 1, description: '장기 추세 판단 기준 이동평균 기간 (기본 200일)' },
    { key: 'short_period', label: '단기 MA 기간', min: 2, max: 30, step: 1, description: '단기 모멘텀 판단 이동평균 기간 (기본 5일)' },
    { key: 'mid_period', label: '중기 MA 기간', min: 5, max: 60, step: 1, description: '중기 추세 비교 이동평균 기간 (기본 20일)' },
  ],
}

export const STRATEGY_DESCRIPTION: Record<string, string> = {
  ma_cross: '단기 MA가 장기 MA를 상향 돌파(골든크로스) 시 매수, 하향 돌파(데드크로스) 시 매도.',
  rsi: 'RSI가 과매도 기준 이하에서 반등하면 매수, 과매수 기준 이상에서 하락하면 매도.',
  momentum: 'N기간 전 가격 대비 현재가 변화율이 임계값 이상이면 매수, 이하이면 매도 (추세 추종).',
  deviation: '현재가가 이동평균 대비 일정 % 이하이면 매수(저평가), 일정 % 이상이면 매도(고평가).',
  fifty_two_week_high: '최근 252 거래일(1년) 최고가를 재돌파하면 매수. 매수 후 지정 % 하락 시 손절. 자동매매 시작 시 KIS 차트 API로 초기화됨.',
  consecutive_move: 'N일 연속 종가 상승 시 매수, M일 연속 하락 시 매도. 추세 초입에 상승/하락할 때 조기에 편승하는 추세추종 전략.',
  failed_breakout: '최근 N일 전고점을 버퍼% 이상 돌파하면 매수. 이후 가격이 전고점 이하로 내려오면 돌파 실패로 판단하여 즉시 매도.',
  strong_close: '자동매매 시작 시 전일 종가가 고가 대비 N% 이내여서 강하게 마감하면 당일 첫 틱에 매수. 매수 후 지정 % 하락 시 손절.',
  volatility_expansion: '당일 변동폭(고-저)이 최근 N거래일 평균 변동폭의 K배 이상이며 현재가 > 시가인 경우 매수. 장중 변동성 폭발 구간에 상승 방향으로 편승. 매수 후 지정 % 하락 시 손절.',
  mean_reversion: '현재가가 볼린저 밴드 하단(mean - Nσ) 아래로 바운스하면 매수(과매도). 현재가 상단 밴드 돌파 시 익절 매도, 손절 기준 % 이상 하락 시 손절. 자동매매 시작 시 과거 N일 종가로 버퍼 적재.',
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
