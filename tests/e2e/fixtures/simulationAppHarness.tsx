import React, { useEffect } from 'react'
import ReactDOM from 'react-dom/client'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { RouterProvider, createMemoryHistory } from '@tanstack/react-router'
import { router } from '../../../src/router'
import { useSetActiveProfile } from '../../../src/api/accountHooks'
import { KEYS } from '../../../src/api/queryKeys'
import type { AppConfigView } from '../../../src/shared/api/types'

// The production router renders the actual AppShell and SimulationPage. This
// browser-only control calls the production profile mutation; it is never shipped.
const client = new QueryClient({ defaultOptions: {
  queries: { retry: false, refetchOnWindowFocus: false }, mutations: { retry: false },
} })
router.update({ history: createMemoryHistory({ initialEntries: ['/simulation'] }) })

function Controls() {
  const mutation = useSetActiveProfile()
  useEffect(() => {
    const state = () => {
      const query = client.getQueryState<AppConfigView>(KEYS.appConfig)
      return { profileId: query?.data?.active_profile_id, dataUpdateCount: query?.dataUpdateCount,
        dataUpdatedAt: query?.dataUpdatedAt, publishing: client.isMutating({ mutationKey: ['account-profile-publication'] }) }
    }
    Reflect.set(window, '__simulationTest', {
      switchProfile: (id: string) => mutation.mutateAsync(id),
      republish: (releaseCommand?: string) => {
        const before = state()
        client.setQueryData(KEYS.appConfig, (old: AppConfigView | undefined) => old)
        const after = state()
        // Release in the same stack, before React's next render/effect. Production
        // quoteScopeCurrent must consult the QueryClient publication immediately.
        if (releaseCommand) {
          const native = Reflect.get(window, '__simulationNative') as { release: (command: string) => void }
          native.release(releaseCommand)
        }
        return { before, after }
      },
      queryState: state,
    })
  }, [mutation])
  return null
}

ReactDOM.createRoot(document.getElementById('root')!).render(
  <QueryClientProvider client={client}><Controls /><RouterProvider router={router} /></QueryClientProvider>,
)
