import React, { useRef, useState } from 'react'
import ReactDOM from 'react-dom/client'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import CssBaseline from '@mui/material/CssBaseline'
import { createTheme, ThemeProvider } from '@mui/material/styles'
import { SectorPortfolioPanel } from '../../../src/widgets/sector-portfolio'

// Synthetic source harness: no operational shell, settings, account hooks or native process.
function Harness() {
  const [epoch, setEpoch] = useState('A-1')
  const current = useRef(epoch); current.current = epoch
  return <>
    <button onClick={() => { current.current = 'B-2'; setEpoch('B-2') }}>합성 프로필 B</button>
    <button onClick={() => { current.current = 'A-3'; setEpoch('A-3') }}>합성 프로필 A 재게시</button>
    <SectorPortfolioPanel mode="workspace" quoteScopeKey={epoch} quoteScopeReady quoteScopeCurrent={key => key === current.current} />
  </>
}
const client = new QueryClient({ defaultOptions: { mutations: { retry: false } } })
ReactDOM.createRoot(document.getElementById('root')!).render(<React.StrictMode>
  <QueryClientProvider client={client}><ThemeProvider theme={createTheme()}><CssBaseline /><Harness /></ThemeProvider></QueryClientProvider>
</React.StrictMode>)
