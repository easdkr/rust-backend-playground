/**
 * Login route — `/login`.
 *
 * Centered Linear-like form. Submits via the `loginServerFn` from
 * `src/server/auth.ts`. On success, navigates to `/posts`. On failure,
 * surfaces the backend's normalized `error` message via `ErrorMessage`.
 *
 * This file intentionally does NOT pre-check the session — if a logged-in
 * user navigates to `/login` we still show the form (they can re-auth or
 * just click "Cancel" / navigate away). The auth-guard layout at
 * `/_authed.tsx` is the one that redirects *unauthenticated* users *away*
 * from protected routes.
 */
import { useState, type FormEvent } from 'react'
import { createFileRoute, redirect, useNavigate } from '@tanstack/react-router'
import { Button, Card, ErrorMessage, Field, Input } from '~/design-system'
import { getCurrentSessionServerFn, loginServerFn } from '~/server/auth'

export const Route = createFileRoute('/login')({
  validateSearch: (search: Record<string, unknown>) => ({
    redirect: typeof search.redirect === 'string' ? search.redirect : '/posts',
  }),
  beforeLoad: async ({ search }) => {
    // If a session is already present, send the user straight to the
    // intended destination (default `/posts`). This avoids the awkward
    // "log in again" loop when an authenticated user opens /login in a
    // new tab.
    const session = await getCurrentSessionServerFn()
    if (session !== null) {
      throw redirect({ to: search.redirect })
    }
  },
  component: LoginPage,
})

function LoginPage() {
  const navigate = useNavigate()
  const { redirect: redirectTo } = Route.useSearch()

  const [username, setUsername] = useState('')
  const [password, setPassword] = useState('')
  const [error, setError] = useState<string | null>(null)
  const [submitting, setSubmitting] = useState(false)

  async function onSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault()
    setError(null)
    setSubmitting(true)
    try {
      await loginServerFn({ data: { username, password } })
      await navigate({ to: redirectTo })
    } catch (err) {
      // `apiFetch` throws UnauthorizedError / ApiError / ApiConfigError.
      // We surface the message; the error class itself isn't relevant to
      // the user.
      const message = err instanceof Error ? err.message : 'Sign-in failed'
      setError(message)
    } finally {
      setSubmitting(false)
    }
  }

  return (
    <main className="bg-bg flex min-h-screen items-center justify-center px-6 py-12">
      <div className="w-full max-w-sm">
        <header className="mb-8 text-center">
          <p className="text-text-subtle text-xs font-medium uppercase tracking-wider">
            rust-backend-playground
          </p>
          <h1 className="text-text mt-1 text-2xl font-semibold leading-tight">
            Sign in
          </h1>
          <p className="text-text-muted mt-2 text-sm">
            Use your backend credentials to access the post workspace.
          </p>
        </header>

        <Card>
          <form
            onSubmit={onSubmit}
            className="flex flex-col gap-4"
            noValidate
            data-ui="login-form"
          >
            {error !== null ? (
              <ErrorMessage data-ui="login-error" role="alert">
                {error}
              </ErrorMessage>
            ) : null}

            <Field
              label="Username"
              htmlFor="login-username"
              hint="The username you registered on the Rust backend."
            >
              <Input
                id="login-username"
                name="username"
                type="text"
                autoComplete="username"
                required
                autoFocus
                value={username}
                onChange={(e) => setUsername(e.target.value)}
                disabled={submitting}
                invalid={error !== null}
                data-ui="login-username"
              />
            </Field>

            <Field
              label="Password"
              htmlFor="login-password"
              hint="Your password is never stored in the client."
            >
              <Input
                id="login-password"
                name="password"
                type="password"
                autoComplete="current-password"
                required
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                disabled={submitting}
                invalid={error !== null}
                data-ui="login-password"
              />
            </Field>

            <Button
              type="submit"
              variant="primary"
              loading={submitting}
              disabled={submitting || username.length === 0 || password.length === 0}
              data-ui="login-submit"
            >
              {submitting ? 'Signing in…' : 'Sign in'}
            </Button>
          </form>
        </Card>

        <p className="text-text-subtle mt-6 text-center text-xs">
          Sessions are held in server memory only — refresh keeps you
          signed in until the server restarts or the token expires.
        </p>
      </div>
    </main>
  )
}
