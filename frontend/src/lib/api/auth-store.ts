/**
 * In-memory access-token store for the API proxy.
 *
 * IMPORTANT: This module is intended to be used ONLY from inside
 * `createServerFn` server functions. The Node-side process is the runtime
 * context, so a module-scoped variable is sufficient to hold the token
 * across requests in dev. There is intentionally no `localStorage`,
 * `sessionStorage`, or `document.cookie` access here — those are browser
 * APIs and would either throw at server-side evaluation time or leak the
 * token into the client bundle.
 *
 * Cookie-based sessions can be added later if a real user request needs
 * them; for MVP this in-memory store is the source of truth, and it is
 * populated by the `POST /auth/login` server function (Task 7).
 */

let accessToken: string | null = null

export function getAccessToken(): string | null {
  return accessToken
}

export function setAccessToken(token: string): void {
  accessToken = token
}

export function clearAccessToken(): void {
  accessToken = null
}
