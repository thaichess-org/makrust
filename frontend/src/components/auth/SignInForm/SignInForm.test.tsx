import { beforeEach, describe, expect, it, vi } from 'vitest'
import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { HttpResponse, http } from 'msw'

import { API_URL } from '../../../lib/api'
import { testUser } from '../../../test/mocks/handlers'
import { server } from '../../../test/mocks/server'
import { renderWithProviders } from '../../../test/renderWithProviders'
import { SignInForm } from './SignInForm'

const mockNavigate = vi.fn()

vi.mock('@tanstack/react-router', () => ({
  useNavigate: () => mockNavigate,
}))

beforeEach(() => {
  mockNavigate.mockClear()
})

const fillForm = async (user: ReturnType<typeof userEvent.setup>) => {
  await user.type(screen.getByLabelText('Username'), 'testuser')
  await user.type(screen.getByLabelText('Password'), 'hunter2')
}

describe('SignInForm', () => {
  it('renders username and password fields', () => {
    renderWithProviders(<SignInForm />)

    expect(screen.getByLabelText('Username')).toBeInTheDocument()
    expect(screen.getByLabelText('Password')).toBeInTheDocument()
  })

  it('shows a validation error and does not submit when fields are empty', async () => {
    const user = userEvent.setup()
    renderWithProviders(<SignInForm />)

    await user.click(screen.getByRole('button', { name: 'Sign In' }))

    expect(await screen.findByText('Please fill in all fields.')).toBeInTheDocument()
    expect(mockNavigate).not.toHaveBeenCalled()
  })

  it('disables the submit button while the request is pending', async () => {
    server.use(
      http.post(`${API_URL}/sign-in`, async () => {
        await new Promise((resolve) => setTimeout(resolve, 50))
        return HttpResponse.json(testUser)
      }),
    )
    const user = userEvent.setup()
    renderWithProviders(<SignInForm />)

    await fillForm(user)
    await user.click(screen.getByRole('button', { name: 'Sign In' }))

    expect(await screen.findByRole('button', { name: 'Signing in…' })).toBeDisabled()
    await waitFor(() => expect(mockNavigate).toHaveBeenCalled())
  })

  it('signs in successfully and navigates home', async () => {
    const user = userEvent.setup()
    const { queryClient } = renderWithProviders(<SignInForm />)

    await fillForm(user)
    await user.click(screen.getByRole('button', { name: 'Sign In' }))

    await waitFor(() => expect(mockNavigate).toHaveBeenCalledWith({ to: '/' }))
    expect(queryClient.getQueryData(['me'])).toEqual(testUser)
  })

  it('shows an incorrect-credentials message on a 401', async () => {
    server.use(http.post(`${API_URL}/sign-in`, () => new HttpResponse(null, { status: 401 })))
    const user = userEvent.setup()
    renderWithProviders(<SignInForm />)

    await fillForm(user)
    await user.click(screen.getByRole('button', { name: 'Sign In' }))

    expect(await screen.findByText('Incorrect username or password.')).toBeInTheDocument()
  })

  it('shows a generic message for unexpected server errors', async () => {
    server.use(http.post(`${API_URL}/sign-in`, () => new HttpResponse(null, { status: 500 })))
    const user = userEvent.setup()
    renderWithProviders(<SignInForm />)

    await fillForm(user)
    await user.click(screen.getByRole('button', { name: 'Sign In' }))

    expect(await screen.findByText('Something went wrong. Please try again.')).toBeInTheDocument()
  })

  it('shows a network error message when the server is unreachable', async () => {
    server.use(http.post(`${API_URL}/sign-in`, () => HttpResponse.error()))
    const user = userEvent.setup()
    renderWithProviders(<SignInForm />)

    await fillForm(user)
    await user.click(screen.getByRole('button', { name: 'Sign In' }))

    expect(
      await screen.findByText('Could not reach the server. Please try again.'),
    ).toBeInTheDocument()
  })
})
