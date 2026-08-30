import type { ReactElement } from 'react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { render } from '@testing-library/react'

// Fresh QueryClient per render so cached data/mutation state can't leak
// between tests. Retries are disabled so failed mutations reject/settle
// immediately instead of waiting out the default retry/backoff.
function renderWithProviders(ui: ReactElement) {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
      mutations: { retry: false },
    },
  })

  return {
    queryClient,
    ...render(<QueryClientProvider client={queryClient}>{ui}</QueryClientProvider>),
  }
}

export { renderWithProviders }
