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
