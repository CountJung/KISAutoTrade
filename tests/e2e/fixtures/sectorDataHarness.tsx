import React from 'react'
import ReactDOM from 'react-dom/client'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import CssBaseline from '@mui/material/CssBaseline'
import { createTheme, ThemeProvider } from '@mui/material/styles'
import { SectorPortfolioPanel } from '../../../src/pages/strategy/ui/SectorPortfolioPanel'

// Source harness only: no operational app shell, account provider or desktop process.
const client = new QueryClient({ defaultOptions: { mutations: { retry: false } } })
ReactDOM.createRoot(document.getElementById('root')!).render(<React.StrictMode>
  <QueryClientProvider client={client}><ThemeProvider theme={createTheme()}><CssBaseline /><SectorPortfolioPanel /></ThemeProvider></QueryClientProvider>
</React.StrictMode>)
