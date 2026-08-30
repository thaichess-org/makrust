import { beforeEach, describe, expect, it, vi } from 'vitest'
import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { HttpResponse, http } from 'msw'

import { API_URL } from '../../../lib/api'
import { testUser } from '../../../test/mocks/handlers'
import { server } from '../../../test/mocks/server'
import { renderWithProviders } from '../../../test/renderWithProviders'
import { SignUpForm } from './SignUpForm'

const mockNavigate = vi.fn()

vi.mock('@tanstack/react-router', () => ({
  useNavigate: () => mockNavigate,
}))

beforeEach(() => {
  mockNavigate.mockClear()
})

const fillForm = async (user: ReturnType<typeof userEvent.setup>) => {
  await user.type(screen.getByLabelText('Username'), 'newuser')
  await user.type(screen.getByLabelText('Email Address'), 'newuser@example.com')
  await user.type(screen.getByLabelText('Password'), 'hunter2')
}

describe('SignUpForm', () => {
  it('renders username, email, and password fields', () => {
    renderWithProviders(<SignUpForm />)

    expect(screen.getByLabelText('Username')).toBeInTheDocument()
    expect(screen.getByLabelText('Email Address')).toBeInTheDocument()
    expect(screen.getByLabelText('Password')).toBeInTheDocument()
  })

  it('shows a validation error and does not submit when fields are empty', async () => {
    const user = userEvent.setup()
    renderWithProviders(<SignUpForm />)

    await user.click(screen.getByRole('button'))

    expect(await screen.findByText('Please fill in all fields.')).toBeInTheDocument()
    expect(mockNavigate).not.toHaveBeenCalled()
  })

  it('signs up successfully and navigates home', async () => {
    const user = userEvent.setup()
    const { queryClient } = renderWithProviders(<SignUpForm />)

    await fillForm(user)
    await user.click(screen.getByRole('button'))

    await waitFor(() => expect(mockNavigate).toHaveBeenCalledWith({ to: '/' }))
    expect(queryClient.getQueryData(['me'])).toEqual(testUser)
  })

  it('shows the server message on a 409 conflict', async () => {
    server.use(
      http.post(`${API_URL}/users`, () =>
        HttpResponse.json({ message: 'Username already taken.' }, { status: 409 }),
      ),
    )
    const user = userEvent.setup()
    renderWithProviders(<SignUpForm />)

    await fillForm(user)
    await user.click(screen.getByRole('button'))

    expect(await screen.findByText('Username already taken.')).toBeInTheDocument()
  })

  it('shows a generic message for unexpected server errors', async () => {
    server.use(
      http.post(`${API_URL}/users`, () => HttpResponse.json({ message: 'boom' }, { status: 500 })),
    )
    const user = userEvent.setup()
    renderWithProviders(<SignUpForm />)

    await fillForm(user)
    await user.click(screen.getByRole('button'))

    expect(await screen.findByText('Something went wrong. Please try again.')).toBeInTheDocument()
  })

  it('shows a network error message when the server is unreachable', async () => {
    server.use(http.post(`${API_URL}/users`, () => HttpResponse.error()))
    const user = userEvent.setup()
    renderWithProviders(<SignUpForm />)

    await fillForm(user)
    await user.click(screen.getByRole('button'))

    expect(
      await screen.findByText('Could not reach the server. Please try again.'),
    ).toBeInTheDocument()
  })
})
