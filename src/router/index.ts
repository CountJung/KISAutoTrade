import { createRouter, createRoute, createRootRoute } from '@tanstack/react-router'
import { AppShell } from '../components/layout/AppShell'
import Dashboard from '../pages/Dashboard'
import Trading from '../pages/Trading'
import Strategy from '../pages/Strategy'
import History from '../pages/History'
import Log from '../pages/Log'
import Settings from '../pages/Settings'

const rootRoute = createRootRoute({
  component: AppShell,
})

const dashboardRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/',
  component: Dashboard,
})

const tradingRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/trading',
  component: Trading,
})

const strategyRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/strategy',
  component: Strategy,
})

const historyRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/history',
  component: History,
})

const logRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/log',
  component: Log,
})

const settingsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/settings',
  component: Settings,
})

const routeTree = rootRoute.addChildren([
  dashboardRoute,
  tradingRoute,
  strategyRoute,
  historyRoute,
  logRoute,
  settingsRoute,
])

export const router = createRouter({ routeTree })

declare module '@tanstack/react-router' {
  interface Register {
    router: typeof router
  }
}
