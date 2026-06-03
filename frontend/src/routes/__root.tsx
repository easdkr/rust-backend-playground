/// <reference types="vite/client" />
import type { ReactNode } from 'react'
import {
  Link,
  Outlet,
  createRootRoute,
  HeadContent,
  Scripts,
} from '@tanstack/react-router'

import { Button, Card } from '~/design-system'

import '~/design-system/tokens.css'

export const Route = createRootRoute({
  head: () => ({
    meta: [
      {
        charSet: 'utf-8',
      },
      {
        name: 'viewport',
        content: 'width=device-width, initial-scale=1',
      },
      {
        title: 'TanStack Start — rust-backend-playground',
      },
    ],
  }),
  notFoundComponent: RootNotFound,
  component: RootComponent,
})

function RootComponent() {
  return (
    <RootDocument>
      <Outlet />
    </RootDocument>
  )
}

function RootDocument({ children }: Readonly<{ children: ReactNode }>) {
  return (
    <html>
      <head>
        <HeadContent />
      </head>
      <body>
        {children}
        <Scripts />
      </body>
    </html>
  )
}

/**
 * Top-level 404 surface. Rendered when the router cannot match any
 * route. The `RootDocument` chrome stays out of here on purpose so
 * unauthenticated 404s do not flash the authed-shell header.
 */
function RootNotFound() {
  return (
    <main
      data-ui="not-found"
      className="bg-bg flex min-h-screen items-center justify-center px-6 py-12"
    >
      <Card className="w-full max-w-sm text-center">
        <p className="text-text-subtle text-xs font-medium uppercase tracking-wider">
          404
        </p>
        <h1 className="text-text mt-1 text-xl font-semibold">Page not found</h1>
        <p className="text-text-muted mt-2 text-sm">
          The URL you followed doesn’t match any known route.
        </p>
        <div className="mt-4 flex justify-center">
          <Link to="/posts" data-ui="not-found-home">
            <Button variant="primary" size="sm">
              Go to posts
            </Button>
          </Link>
        </div>
      </Card>
    </main>
  )
}
