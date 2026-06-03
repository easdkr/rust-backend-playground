/**
 * Home route — `/`.
 *
 * The MVP product surface is the authenticated post workspace
 * (`/_authed/...`). The home URL simply redirects there; the auth
 * guard in `/_authed.tsx` will bounce unauthenticated users to
 * `/login` (preserving the original path as `?redirect=`).
 */
import { createFileRoute, redirect } from '@tanstack/react-router'

export const Route = createFileRoute('/')({
  beforeLoad: () => {
    throw redirect({ to: '/posts' })
  },
  // `component` is required by createFileRoute's types, but the route
  // never renders — `beforeLoad` always throws. Keep it trivial.
  component: () => null,
})
