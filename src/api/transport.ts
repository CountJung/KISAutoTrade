/**
 * Tauri IPC / Web REST 듀얼 모드 invoke 래퍼
 *
 * - 데스크탑 앱(Tauri): window.__TAURI_INTERNALS__ 존재 → Tauri invoke() 사용
 * - 웹 브라우저(모바일 등): /api/* REST 엔드포인트로 자동 fallback
 *
 * commands.ts 에서는 이 파일의 invoke() 만 사용하면
 * Tauri/웹 어디서든 동일한 코드로 작동합니다.
 */
type Args = Record<string, unknown>

/** Tauri 컨텍스트 여부 감지 */
function isTauri(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
}

interface RestRequest {
  method: 'GET' | 'POST'
  url: string
  body?: unknown
}

/**
 * IPC 커맨드 이름 → REST 요청 변환
 * 웹 모드에서 지원할 커맨드만 여기에 정의합니다.
 * 지원하지 않는 커맨드(Settings, Strategy 등)는 에러를 throw합니다.
 */
function resolveRest(command: string, args: Args = {}): RestRequest {
  switch (command) {
    // ─── 잔고 ────────────────────────────────────────────────
    case 'get_balance':
      return { method: 'GET', url: '/api/balance' }

    case 'get_overseas_balance':
      return { method: 'GET', url: '/api/overseas-balance' }

    // ─── 현재가 ──────────────────────────────────────────────
    case 'get_price':
      return { method: 'GET', url: `/api/price/${args.symbol}` }

    // ─── 체결 내역 ────────────────────────────────────────────
    case 'get_today_executed':
      return { method: 'GET', url: '/api/executed' }

    // ─── 종목 검색 ────────────────────────────────────────────
    case 'search_stock':
      return {
        method: 'GET',
        url: `/api/search/${encodeURIComponent(String(args.query ?? ''))}`,
      }

    // ─── 국내 주문 ────────────────────────────────────────────
    case 'place_order':
      return { method: 'POST', url: '/api/order', body: args.input }

    // ─── 차트 데이터 ──────────────────────────────────────────
    case 'get_chart_data': {
      const inp = args.input as {
        symbol: string
        period_code?: string
        start_date?: string
        end_date?: string
      }
      const period = inp.period_code ?? 'D'
      // 서버 측에서 start/end를 period+count로 계산합니다
      const countMap: Record<string, number> = { D: 100, W: 78, M: 60 }
      const count = countMap[period] ?? 100
      return {
        method: 'GET',
        url: `/api/chart/${inp.symbol}?period=${period}&count=${count}`,
      }
    }

    // ─── 해외 현재가 ──────────────────────────────────────────
    case 'get_overseas_price':
      return {
        method: 'GET',
        url: `/api/overseas-price/${args.exchange}/${args.symbol}`,
      }

    // ─── 해외 차트 ────────────────────────────────────────────
    case 'get_overseas_chart_data': {
      const period = (args.periodCode as string) ?? 'D'
      const countMap: Record<string, number> = { D: 100, W: 78, M: 60 }
      const count = countMap[period] ?? 100
      return {
        method: 'GET',
        url: `/api/overseas-chart/${args.exchange}/${args.symbol}?period=${period}&count=${count}`,
      }
    }

    // ─── 해외 주문 ────────────────────────────────────────────
    case 'place_overseas_order':
      return { method: 'POST', url: '/api/overseas-order', body: args.input }
    // ─── 자동매매 제어 ────────────────────────────────────────────────
    case 'get_trading_status':
      return { method: 'GET', url: '/api/trading/status' }

    case 'start_trading':
      return { method: 'POST', url: '/api/trading/start' }

    case 'stop_trading':
      return { method: 'POST', url: '/api/trading/stop' }

    case 'get_strategies':
      return { method: 'GET', url: '/api/strategies' }

    case 'update_strategy':
      return { method: 'POST', url: `/api/strategies/${(args.input as { id: string }).id}`, body: args.input }

    // ─── 앱 설정 / 프로파일 ──────────────────────────────────────────
    case 'get_app_config':
      return { method: 'GET', url: '/api/app-config' }

    case 'list_profiles':
      return { method: 'GET', url: '/api/profiles' }

    // ─── 포지션 ──────────────────────────────────────────────────────
    case 'get_positions':
      return { method: 'GET', url: '/api/positions' }

    // ─── 통계 / 체결 기록 ────────────────────────────────────────────
    case 'get_today_stats':
      return { method: 'GET', url: '/api/today-stats' }

    case 'get_stats_by_range': {
      const inp = args.input as { from: string; to: string }
      return { method: 'GET', url: `/api/stats?from=${inp.from}&to=${inp.to}` }
    }

    case 'get_trades_by_range': {
      const inp = args.input as { from: string; to: string }
      return { method: 'GET', url: `/api/trades?from=${inp.from}&to=${inp.to}` }
    }

    case 'get_kis_executed_by_range':
      return { method: 'GET', url: `/api/kis-executed?from=${args.from}&to=${args.to}` }

    case 'get_pending_orders':
      return { method: 'GET', url: '/api/pending-orders' }

    // ─── 로그 설정 ────────────────────────────────────────────────────
    case 'get_log_config':
      return { method: 'GET', url: '/api/log-config' }

    case 'set_log_config':
      return { method: 'POST', url: '/api/log-config', body: args.input }

    case 'get_recent_logs':
      return { method: 'GET', url: `/api/recent-logs?count=${args.count ?? 100}` }

    // ─── 체결 기록 보관 설정 ─────────────────────────────────────────
    case 'get_trade_archive_config':
      return { method: 'GET', url: '/api/archive-config' }

    case 'set_trade_archive_config':
      return { method: 'POST', url: '/api/archive-config', body: args.input }

    case 'get_trade_archive_stats':
      return { method: 'GET', url: '/api/archive-stats' }

    // ─── 리스크 관리 ─────────────────────────────────────────────────
    case 'get_risk_config':
      return { method: 'GET', url: '/api/risk-config' }

    case 'update_risk_config':
      return { method: 'POST', url: '/api/risk-config', body: args.input }

    case 'clear_emergency_stop':
      return { method: 'POST', url: '/api/risk-config/clear-emergency' }

    // ─── 웹 설정 ─────────────────────────────────────────────────────
    case 'get_web_config':
      return { method: 'GET', url: '/api/web-config' }

    // ─── 설정 진단 ───────────────────────────────────────────────────
    case 'check_config':
      return { method: 'GET', url: '/api/check-config' }

    // ─── 프로파일 관리 ───────────────────────────────────────────────
    case 'add_profile':
      return { method: 'POST', url: '/api/profiles/add', body: args.input }

    case 'update_profile':
      return { method: 'POST', url: '/api/profiles/update', body: args.input }

    case 'delete_profile':
      return { method: 'POST', url: '/api/profiles/delete', body: { id: args.id } }

    case 'set_active_profile':
      return { method: 'POST', url: `/api/profiles/${args.id}/set-active` }

    case 'detect_trading_type':
      return { method: 'POST', url: '/api/detect-trading-type', body: { appKey: args.appKey, appSecret: args.appSecret } }

    case 'detect_profile_trading_type':
      return { method: 'POST', url: `/api/profiles/${args.profileId}/detect` }

    // ─── 종목 목록 ───────────────────────────────────────────────────
    case 'get_stock_list_stats':
      return { method: 'GET', url: '/api/stock-list-stats' }

    case 'set_stock_update_interval':
      return { method: 'POST', url: '/api/stock-update-interval', body: { hours: args.hours } }

    case 'refresh_stock_list':
      return { method: 'POST', url: '/api/refresh-stock-list' }

    // ─── Discord 테스트 ──────────────────────────────────────────────
    case 'send_test_discord':
      return { method: 'POST', url: '/api/test-discord' }

    // ─── 당일 체결 기록 ──────────────────────────────────────────────
    case 'get_today_trades':
      return { method: 'GET', url: '/api/today-trades' }

    // ─── 업데이트 확인 ───────────────────────────────────────────────
    case 'check_for_update':
      return { method: 'GET', url: '/api/check-update' }

    // ─── 웹 설정 저장 ────────────────────────────────────────────────
    case 'save_web_config':
      return { method: 'POST', url: '/api/web-config/save', body: { newPort: args.newPort } }

    // ─── 환율 / 갱신 주기 ────────────────────────────────────────────
    case 'get_exchange_rate':
      return { method: 'GET', url: '/api/exchange-rate' }

    case 'get_refresh_interval':
      return { method: 'GET', url: '/api/refresh-interval' }

    // ─── 매수 정지 / 비상 정지 ───────────────────────────────────────
    case 'clear_buy_suspension':
      return { method: 'POST', url: '/api/buy-suspension/clear' }

    case 'activate_emergency_stop':
      return { method: 'POST', url: '/api/activate-emergency' }

    // ─── 체결 기록 / 통계 저장 ───────────────────────────────────────
    case 'save_trade':
      return { method: 'POST', url: '/api/save-trade', body: args.input }

    case 'upsert_daily_stats':
      return { method: 'POST', url: '/api/upsert-stats', body: args.stats }

    // ─── 프론트엔드 로그 ─────────────────────────────────────────────
    case 'write_frontend_log':
      return { method: 'POST', url: '/api/frontend-log', body: args.input }

    default:
      throw new Error(
        `웹 모드에서 지원되지 않는 커맨드: ${command} (데스크탑 앱에서만 사용 가능)`
      )
  }
}

async function restInvoke<T>(command: string, args: Args = {}): Promise<T> {
  const req = resolveRest(command, args)
  const res = await fetch(req.url, {
    method: req.method,
    headers: req.body ? { 'Content-Type': 'application/json' } : {},
    body: req.body ? JSON.stringify(req.body) : undefined,
  })

  if (!res.ok) {
    const err = await res.json().catch(() => ({ error: res.statusText }))
    throw new Error((err as { error?: string }).error ?? res.statusText)
  }

  const data = (await res.json()) as Record<string, unknown>
  if (data && typeof data === 'object' && 'error' in data && data.error) {
    throw new Error(String(data.error))
  }
  return data as T
}

/** 메인 invoke — Tauri/웹 양쪽에서 동일하게 사용 */
export async function invoke<T>(command: string, args?: Args): Promise<T> {
  if (isTauri()) {
    const { invoke: tauriInvoke } = await import('@tauri-apps/api/core')
    return tauriInvoke<T>(command, args)
  }
  return restInvoke<T>(command, args ?? {})
}
