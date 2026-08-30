import React, { useState } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { useNavigate } from '@tanstack/react-router'

import type { UserRecord } from '../../../lib/types'
import { API_URL } from '../../../lib/api'

class SignInRequestError extends Error {
  status: number
  constructor(status: number) {
    super(`Sign in request failed with status ${status}`)
    this.status = status
  }
}

const signIn = async (credentials: { username: string; password: string }): Promise<UserRecord> => {
  const response = await fetch(`${API_URL}/sign-in`, {
    method: 'POST',
    body: new URLSearchParams(credentials),
    credentials: 'include',
  })

  if (!response.ok) {
    throw new SignInRequestError(response.status)
  }

  return response.json()
}

const SignInForm = () => {
  const [username, setUsername] = useState('')
  const [password, setPassword] = useState('')
  const [error, setError] = useState('')

  const navigate = useNavigate()
  const queryClient = useQueryClient()
  const signInMutation = useMutation({
    mutationFn: signIn,
    onSuccess: async (user) => {
      queryClient.setQueryData(['me'], user)
      await navigate({ to: '/' })
    },
    onError: (err) => {
      if (err instanceof SignInRequestError) {
        setError(
          err.status === 401
            ? 'Incorrect username or password.'
            : 'Something went wrong. Please try again.',
        )
      } else {
        setError('Could not reach the server. Please try again.')
      }
    },
  })

  const handleSubmit = (e: React.SubmitEvent<HTMLFormElement>) => {
    e.preventDefault()
    if (!username || !password) {
      setError('Please fill in all fields.')
      return
    }
    setError('')
    signInMutation.mutate({ username, password })
  }

  return (
    <div>
      <form onSubmit={handleSubmit}>
        <h2>Sign In</h2>

        {error && <p>{error}</p>}

        <div>
          <label htmlFor="username">Username</label>
          <input
            type="text"
            id="username"
            value={username}
            onChange={(e) => setUsername(e.target.value)}
            placeholder="Enter your username"
            autoComplete="username"
          />
        </div>
        <div>
          <label htmlFor="password">Password</label>
          <input
            type="password"
            id="password"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            placeholder="Enter your password"
          />
        </div>

        <button type="submit" disabled={signInMutation.isPending}>
          {signInMutation.isPending ? 'Signing in…' : 'Sign In'}
        </button>
      </form>
    </div>
  )
}

export { SignInForm }
