import React from 'react'
import ReactDOM from 'react-dom/client'
import { RouterProvider } from '@tanstack/react-router'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 1000 * 30,
      retry: 2,
    },
  },
})

// Choose before evaluating the operational router: its AppShell imports account
// stores, update checks and backend event hooks at module evaluation/render time.
const researchOnly = (window as Window & { __KIS_RESEARCH_ONLY__?: boolean }).__KIS_RESEARCH_ONLY__ === true
  || new URLSearchParams(window.location.search).get('researchOnly') === '1'

async function bootstrap() {
  const content = researchOnly
    ? React.createElement((await import('./pages/research')).default)
    : React.createElement(RouterProvider, { router: (await import('./router')).router })

  ReactDOM.createRoot(document.getElementById('root')!).render(
    <React.StrictMode>
      <QueryClientProvider client={queryClient}>
        {content}
      </QueryClientProvider>
    </React.StrictMode>,
  )
}

void bootstrap()
