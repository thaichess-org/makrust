import React, { useState } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { useNavigate } from '@tanstack/react-router'

import type { UserRecord } from '../../../lib/types'
import { API_URL } from '../../../lib/api'

class SignUpRequestError extends Error {
  status: number
  constructor(status: number, message: string) {
    super(`Sign up request failed with status ${status}`)
    this.status = status
    this.message = message
  }
}

const signUp = async (credentials: {
  username: string
  email: string
  password: string
}): Promise<UserRecord> => {
  const response = await fetch(`${API_URL}/users`, {
    method: 'POST',
    body: new URLSearchParams(credentials),
    credentials: 'include',
  })

  if (!response.ok) {
    const { message } = (await response.json()) as { message: string }
    throw new SignUpRequestError(response.status, message)
  }

  return response.json()
}

const SignUpForm = () => {
  const [username, setUsername] = useState('')
  const [email, setEmail] = useState('')
  const [password, setPassword] = useState('')
  const [error, setError] = useState('')

  const navigate = useNavigate()
  const queryClient = useQueryClient()
  const signUpMutation = useMutation({
    mutationFn: signUp,
    onSuccess: async (user) => {
      queryClient.setQueryData(['me'], user)
      await navigate({ to: '/' })
    },
    onError: (error) => {
      if (error instanceof SignUpRequestError) {
        setError(
          error.status === 400 || error.status === 409
            ? error.message
            : 'Something went wrong. Please try again.',
        )
      } else {
        setError('Could not reach the server. Please try again.')
      }
    },
  })

  const handleSubmit = (e: React.SubmitEvent<HTMLFormElement>) => {
    e.preventDefault()
    if (!username || !password || !email) {
      setError('Please fill in all fields.')
      return
    }
    setError('')
    signUpMutation.mutate({ username, email, password })
  }

  return (
    <div>
      <form onSubmit={handleSubmit}>
        <h2>Sign Up</h2>

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
          <label htmlFor="email">Email Address</label>
          <input
            type="email"
            id="email"
            value={email}
            onChange={(e) => setEmail(e.target.value)}
            placeholder="Enter your email"
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

        <button type="submit">Sign Up</button>
      </form>
    </div>
  )
}

export { SignUpForm }
