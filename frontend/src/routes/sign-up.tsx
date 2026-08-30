import { createFileRoute } from '@tanstack/react-router'
import { SignUpForm } from '../components/auth/SignUpForm/SignUpForm.tsx'

export const Route = createFileRoute('/sign-up')({
  component: SignUpForm,
})
