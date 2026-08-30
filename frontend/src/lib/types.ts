type UserRecord = {
  username: string
  display_name: string | null
  bio: string | null
  country_code: string | null
  role: string
  is_active: boolean
  created_at: string
  last_seen_at: string | null
}

export type { UserRecord }
