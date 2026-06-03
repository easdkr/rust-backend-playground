/**
 * Server functions for the MVP auth flow.
 *
 * Each exported function is a `createServerFn` wrapper that runs
 * **server-side only**. Loaders and React components call these
 * functions the same way they call any other async function; the
 * build process replaces the implementation with an RPC stub in the
 * client bundle, so the body of these handlers (and the secrets they
 * touch) never reaches the browser.
 *
 * Why not a route loader reading the auth state directly? The loader
 * is isomorphic — it runs both on the server (for SSR) and on the
 * client (for navigation). Anything it reads on the server that the
 * client should not see has to be a server function. So:
 *
 *   - `loginServerFn` — the only thing that ever receives a raw access
 *     token. Stores it in the in-memory `auth-store` and returns a
 *     truncated preview.
 *   - `getCurrentSessionServerFn` — returns a non-secret preview of
 *     the in-memory token, or `null`. (For MVP we don't have a `/me`
 *     endpoint, so this is the best we can do.)
 *   - `clearSessionServerFn` — drops the in-memory token.
 *
 * None of these read `process.env` outside the handler body — every
 * call to `getApiBaseUrl()` happens inside the `handler` so the
 * `ApiConfigError` path is exercised on every invocation, not at
 * module-load time.
 */

import { createServerFn } from '@tanstack/react-start'

import { apiFetch, getApiBaseUrl } from '~/lib/api'
import {
  clearAccessToken,
  getAccessToken,
  setAccessToken,
} from '~/lib/api/auth-store.ts'
import type {
  CurrentSessionResult,
  LoginCmd,
  LoginResult,
  TokenResponse,
} from '~/lib/api/types.ts'

const ACCESS_TOKEN_PREVIEW_LEN = 8

function truncateAccessToken(token: string): string {
  if (token.length <= ACCESS_TOKEN_PREVIEW_LEN) {
    return token + '…'
  }
  return token.slice(0, ACCESS_TOKEN_PREVIEW_LEN) + '…'
}

// ---------------------------------------------------------------------------
// `POST /auth/login`
// ---------------------------------------------------------------------------

/**
 * Authenticate with username + password. On success, the raw access
 * token is stored in the in-memory auth store on the server process
 * and **never returned to the client** — the function returns a
 * `LoginResult` containing only `token_type`, `expires_in`, and an
 * 8-character preview of the access token.
 *
 * `apiFetch` does not send an `Authorization` header (there is no
 * token yet) and does not set `Content-Type: application/json` until
 * we hand it a non-undefined body, both of which match the backend's
 * public `POST /auth/login` contract.
 */
export const loginServerFn = createServerFn({ method: 'POST' })
  .inputValidator((data: LoginCmd) => data)
  .handler(async ({ data }): Promise<LoginResult> => {
    // Touched inside the handler so the env-missing error path is
    // exercised on every call (not only on the first module import).
    getApiBaseUrl()

    const response = await apiFetch<TokenResponse>('/auth/login', {
      method: 'POST',
      body: data,
    })

    // Store the raw token in the server-side in-memory store. Subsequent
    // calls to `apiFetch` from any server function on this process will
    // pick it up automatically via `getAccessToken()`.
    setAccessToken(response.access_token)

    return {
      token_type: response.token_type,
      expires_in: response.expires_in,
      access_token_preview: truncateAccessToken(response.access_token),
    }
  })

// ---------------------------------------------------------------------------
// `getCurrentSessionServerFn` — session introspection
// ---------------------------------------------------------------------------

/**
 * Return a non-secret snapshot of the current session, or `null` if
 * no token is held in the in-memory store.
 *
 * The MVP has no `/me` endpoint (the backend's `user` DTOs expose user
 * info only at login time), so we cannot fetch the username here.
 * This function is intentionally minimal — it just confirms "yes,
 * there is a token in memory" and shows a preview. The full token is
 * never returned; a route layer that needs the real token can call
 * `getAccessToken()` from another server function, but should not
 * ship it to the client.
 */
export const getCurrentSessionServerFn = createServerFn({ method: 'GET' })
  .handler(async (): Promise<CurrentSessionResult> => {
    getApiBaseUrl() // surface `ApiConfigError` consistently with the other fns

    const token = getAccessToken()
    if (token === null) {
      return null
    }
    return {
      has_session: true,
      access_token_preview: truncateAccessToken(token),
      // We don't track expiry in the in-memory store for the MVP —
      // `expires_in` is only known at login time. A later task can
      // augment the store with an issued-at timestamp.
      expires_in: null,
    }
  })

// ---------------------------------------------------------------------------
// `clearSessionServerFn` — drop the in-memory token
// ---------------------------------------------------------------------------

/**
 * Drop the in-memory access token. The MVP has no `/auth/logout`
 * endpoint on the backend, so the "logout" story is server-side state
 * reset only — the JWT remains technically valid until `exp`, but
 * with the in-memory store cleared the server function layer can no
 * longer reach any protected endpoint.
 */
export const clearSessionServerFn = createServerFn({ method: 'POST' })
  .handler(async (): Promise<{ cleared: true }> => {
    getApiBaseUrl() // surface `ApiConfigError` consistently with the other fns
    clearAccessToken()
    return { cleared: true }
  })
