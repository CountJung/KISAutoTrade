/**
 * 전역 폴링 스케쥴러
 *
 * KIS API 허용 주기:
 *  - 실전: 20 calls/s (초당 20건, TR-ID 별 제한)
 *  - 모의: 2 calls/s (초당 2건, TR-ID 별 제한)
 *
 * TanStack Query는 동일한 queryKey를 여러 컴포넌트가 구독해도
 * 하나의 폴링만 실행하여 자동 중복 제거한다.
 * → 각 훅의 refetchInterval은 이 파일의 상수를 사용하여 중앙에서 관리한다.
 *
 * ## 폴링 카테고리
 * | 카테고리 | 기본 주기 | 해당 데이터                     |
 * |---------|--------|-------------------------------|
 * | FAST    | 10s    | 현재가, 자동매매 상태, 리스크        |
 * | NORMAL  | 30s    | 체결 내역, 포지션, 미체결 주문       |
 * | SLOW    | 60s    | 잔고, 통계                       |
 * | LOG     | 5s     | 앱 로그 스트리밍                   |
 * | PENDING | 5s     | 미체결 주문 (체결 확인 목적)         |
 *
 * ## 모의 투자 모드에서의 주기 조정
 * 모의투자는 2 calls/s 제한이므로 FAST 카테고리도 최소 15s 이상으로 설정한다.
 * `getIntervals(isPaper)` 함수를 사용하면 자동 계산된다.
 *
 * ## 업데이트 규칙 (Living Documentation)
 * - 새 폴링 훅 추가 시 이 파일의 카테고리 상수를 사용할 것
 * - 모의/실전 분기가 필요한 경우 `getIntervals(isPaper)` 사용
 * - API rate-limit 에러(EGW00133) 발생 시 해당 카테고리 주기를 늘릴 것
 */

/** 실전 기본 폴링 주기 (ms) */
export const POLL_INTERVALS = {
  /** 빠른 갱신: 현재가, 자동매매 상태, 리스크 설정 */
  FAST: 10_000,
  /** 보통 갱신: 체결 내역, 포지션 */
  NORMAL: 30_000,
  /** 느린 갱신: 잔고, 일별 통계 */
  SLOW: 60_000,
  /** 로그 스트리밍 */
  LOG: 5_000,
  /** 미체결 주문 체크 */
  PENDING: 5_000,
} as const

/**
 * 실전/모의투자 모드에 따른 폴링 주기 반환
 *
 * 모의투자: KIS API 2 calls/s 제한 → 모든 주기 × 2
 */
export function getIntervals(isPaper: boolean) {
  const mul = isPaper ? 2 : 1
  return {
    /** 현재가 / 자동매매 상태 / 리스크 */
    price:         POLL_INTERVALS.FAST   * mul,
    tradingStatus: POLL_INTERVALS.FAST   * mul,
    riskConfig:    POLL_INTERVALS.FAST   * mul,
    /** 체결 내역 / 포지션 */
    todayExecuted: POLL_INTERVALS.NORMAL * mul,
    positions:     POLL_INTERVALS.NORMAL * mul,
    /** 미체결 주문 */
    pendingOrders: POLL_INTERVALS.PENDING * mul,
    /** 잔고 */
    balance:       POLL_INTERVALS.SLOW   * mul,
    /** 앱 로그 */
    logs:          POLL_INTERVALS.LOG    * mul,
  } as const
}

/**
 * 주문 성공 후 체결 내역 갱신 대기 시간 (ms)
 *
 * KIS API는 주문 접수 후 내부 처리에 약간의 딜레이가 있다.
 * 즉시 re-fetch하면 새 체결이 반영되지 않으므로 일정 시간 대기 후 갱신한다.
 *  - 실전: 1.5s (빠름)
 *  - 모의: 3.0s (처리가 느림)
 */
export const ORDER_REFETCH_DELAY_MS = {
  REAL:  1_500,
  PAPER: 3_000,
} as const
