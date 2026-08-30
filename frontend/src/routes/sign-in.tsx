import { createFileRoute } from '@tanstack/react-router'
import { SignInForm } from '../components/auth/SignInForm/SignInForm.tsx'

export const Route = createFileRoute('/sign-in')({
  component: SignInForm,
})
