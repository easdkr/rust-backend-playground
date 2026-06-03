/**
 * Server-only API configuration.
 *
 * IMPORTANT: This module is intended to be imported ONLY from inside
 * `createServerFn` server functions, never from React components or route
 * loaders/components. The loader is isomorphic — it runs on both the server
 * and the client — so reading `process.env` from a loader would either
 * silently no-op on the client (Vite doesn't inject unprefixed vars) or
 * cause a build error. The base URL must stay out of the browser bundle.
 *
 * The env var is intentionally NOT prefixed with `VITE_`. Vite only injects
 * `VITE_*` env vars into the client bundle, and the contract is that the
 * browser never sees the backend URL.
 */

import { ApiConfigError } from './errors.ts'

let cachedBaseUrl: string | null = null

/**
 * Resolve the backend base URL from the server-side environment.
 *
 * Throws `ApiConfigError` with an actionable message if the variable is unset
 * (e.g. the developer forgot to copy `frontend/.env.example` to
 * `frontend/.env`, or the deployment didn't set it).
 */
export function getApiBaseUrl(): string {
  if (cachedBaseUrl !== null) {
    return cachedBaseUrl
  }
  const baseUrl = process.env.RUST_API_BASE_URL
  if (!baseUrl || baseUrl.trim() === '') {
    throw new ApiConfigError(
      'RUST_API_BASE_URL is not set; copy frontend/.env.example to frontend/.env and set it to the Rust backend URL (default http://127.0.0.1:3000).',
    )
  }
  cachedBaseUrl = baseUrl.replace(/\/+$/, '')
  return cachedBaseUrl
}

/**
 * Reset the cached base URL. Intended for tests / dev hot-reload that
 * changes the env mid-process. Do not call from application code.
 */
export function __resetApiConfigForTesting(): void {
  cachedBaseUrl = null
}
