import { HttpResponse, http } from 'msw'

import type { UserRecord } from '../../lib/types'
import { API_URL } from '../../lib/api'

const testUser: UserRecord = {
  username: 'testuser',
  display_name: null,
  bio: null,
  country_code: null,
  role: 'user',
  is_active: true,
  created_at: '2026-01-01T00:00:00Z',
  last_seen_at: null,
}

// Default happy-path handlers. Tests that need an error response override
// these per test with server.use(...).
const handlers = [
  http.post(`${API_URL}/users`, () => HttpResponse.json(testUser)),
  http.post(`${API_URL}/sign-in`, () => HttpResponse.json(testUser)),
]

export { handlers, testUser }
