import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import Box from '@mui/material/Box'
import Container from '@mui/material/Container'
import CssBaseline from '@mui/material/CssBaseline'
import Typography from '@mui/material/Typography'
import { createTheme, ThemeProvider } from '@mui/material/styles'
import { SectorPortfolioPanel } from '../../strategy/ui/SectorPortfolioPanel'

const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false }, mutations: { retry: false } } })
const theme = createTheme({
  palette: { mode: 'light' },
  typography: { fontFamily: '"Noto Sans KR", "Roboto", "Helvetica", "Arial", sans-serif' },
})

/** Standalone entry: deliberately imports no router, account hooks, settings store, or app shell. */
export default function ResearchPage() {
  return <QueryClientProvider client={queryClient}><ThemeProvider theme={theme}><CssBaseline />
    <Box component="main" sx={{ minHeight: '100dvh', bgcolor: 'background.default', py: { xs: 2, sm: 4 }, overflowWrap: 'anywhere' }}>
      <Container maxWidth="xl" sx={{ minWidth: 0 }}>
        <Typography component="h1" variant="h5" sx={{ fontWeight: 700, mb: 1 }}>섹터 ETF 독립 연구</Typography>
        <Typography sx={{ mb: 3 }}>파일 검증과 과거 자료 계산 전용입니다. 시작할 때 계좌·설정·시세를 조회하지 않습니다. KIS 자료 수집은 일반 앱에서 별도로 요청하세요.</Typography>
        <SectorPortfolioPanel researchOnly />
      </Container>
    </Box>
  </ThemeProvider></QueryClientProvider>
}
