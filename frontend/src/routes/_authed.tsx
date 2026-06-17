/**
 * Auth-guard layout for protected routes.
 *
 * This is a TanStack Router *pathless* layout (the leading underscore in
 * `_authed` means it does not contribute a URL segment; children appear
 * at their own paths). Any route file placed under `routes/_authed/...`
 * is rendered inside this layout AND is gated by the `beforeLoad`
 * session check below.
 *
 * Unauthenticated users hitting any authed route are redirected to
 * `/login?redirect=<original-path>` so the login page can send them
 * back after a successful sign-in.
 *
 * The component renders the Linear-like app shell: top header with the
 * product mark, a minimal nav (only MVP links), and a sign-out button.
 * The shell is the *only* place navigation chrome lives — feature
 * routes render their own page content via `<Outlet />`.
 */
import { type ReactNode } from 'react'
import { Link, Outlet, createFileRoute, redirect } from '@tanstack/react-router'
import { Button } from '~/design-system'
import { UnauthorizedError } from '~/lib/api'
import { clearSessionServerFn, getCurrentUserServerFn } from '~/server/auth'

export const Route = createFileRoute('/_authed')({
  beforeLoad: async ({ location }) => {
    try {
      const user = await getCurrentUserServerFn()
      return { user }
    } catch (err) {
      if (err instanceof UnauthorizedError) {
        throw redirect({
          to: '/login',
          search: { redirect: location.href },
        })
      }
      throw err
    }
  },
  loader: ({ context }) => context.user,
  component: AuthedShell,
})

function AuthedShell() {
  // `loader` puts the current user on the route context, available via
  // `Route.useLoaderData()` if a child wants it. The shell itself only
  // needs the username for the "Signed in as …" chip.
  const user = Route.useLoaderData()

  return (
    <div
      data-ui="authed-shell"
      className="bg-bg text-text min-h-screen"
    >
      <header className="border-border-subtle bg-bg/80 sticky top-0 z-10 border-b backdrop-blur">
        <div className="mx-auto flex h-12 max-w-5xl flex-wrap items-center gap-x-4 gap-y-1 px-4 sm:gap-x-6 sm:px-6">
          <Link
            to="/posts"
            className="text-text text-sm font-semibold tracking-tight"
            data-ui="shell-brand"
          >
            rust-backend-playground
          </Link>

          <nav
            className="text-text-muted flex items-center gap-4 text-sm"
            data-ui="shell-nav"
          >
            <Link
              to="/posts"
              activeOptions={{ exact: false }}
              activeProps={{
                className: 'text-text',
              }}
              className="hover:text-text transition-colors"
              data-ui="shell-nav-posts"
            >
              Posts
            </Link>
            <Link
              to="/posts/new"
              activeProps={{
                className: 'text-text',
              }}
              className="hover:text-text transition-colors"
              data-ui="shell-nav-new"
            >
              New post
            </Link>
          </nav>

          <div className="ml-auto flex items-center gap-3">
            <span
              className="text-text-subtle hidden text-xs sm:inline"
              data-ui="shell-user"
            >
              Signed in as <span className="text-text-muted font-mono">{user.username}</span>
            </span>
            <SignOutButton />
          </div>
        </div>
      </header>

      <Outlet />
    </div>
  )
}

function SignOutButton() {
  // Sign-out is a server-side clear + client-side navigate. We do NOT
  // need a server function wrapper for the *click*; we call the server
  // function directly. The button is `type="button"` so it never
  // submits an enclosing form.
  return (
    <Button
      type="button"
      variant="ghost"
      size="sm"
      onClick={async () => {
        await clearSessionServerFn()
        // Force a full navigation so the guard re-runs and bounces
        // us to /login. `router.navigate` would also work, but
        // `window.location.assign` is the simplest way to invalidate
        // the loader cache and guarantee the guard re-evaluates.
        window.location.assign('/login')
      }}
      data-ui="shell-signout"
    >
      Sign out
    </Button>
  )
}

// Re-export so children of this layout that need a typed session prop
// can import it.
export type { ReactNode }
