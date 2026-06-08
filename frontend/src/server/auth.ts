/**
 * Server functions for the auth flow.
 *
 * Auth server is separated from API server:
 *   - API server (port 3000): /posts, /tags, /notifications
 *   - Auth server (port 3002): /auth/login, /auth/refresh, /auth/logout
 *
 * The Ingress routes all through localhost:8080:
 *   /auth/*  → auth-server
 *   /api/*   → api-server
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
  LogoutResult,
  RefreshCmd,
  RefreshResult,
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

export const loginServerFn = createServerFn({ method: 'POST' })
  .inputValidator((data: LoginCmd) => data)
  .handler(async ({ data }): Promise<LoginResult> => {
    getApiBaseUrl()

    const response = await apiFetch<TokenResponse>('/auth/login', {
      method: 'POST',
      body: data,
    })

    setAccessToken(response.access_token)

    return {
      token_type: response.token_type,
      expires_in: response.expires_in,
      access_token_preview: truncateAccessToken(response.access_token),
    }
  })

// ---------------------------------------------------------------------------
// `POST /auth/refresh` — token refresh
// ---------------------------------------------------------------------------

export const refreshServerFn = createServerFn({ method: 'POST' })
  .inputValidator((data: RefreshCmd) => data)
  .handler(async ({ data }): Promise<RefreshResult> => {
    getApiBaseUrl()

    const response = await apiFetch<TokenResponse>('/auth/refresh', {
      method: 'POST',
      body: data,
    })

    setAccessToken(response.access_token)

    return {
      token_type: response.token_type,
      expires_in: response.expires_in,
      access_token_preview: truncateAccessToken(response.access_token),
    }
  })

// ---------------------------------------------------------------------------
// `POST /auth/logout` — server-side logout
// ---------------------------------------------------------------------------

export const logoutServerFn = createServerFn({ method: 'POST' })
  .inputValidator((data: { refresh_token: string }) => data)
  .handler(async ({ data }): Promise<LogoutResult> => {
    getApiBaseUrl()

    try {
      await apiFetch<LogoutResult>('/auth/logout', {
        method: 'POST',
        body: data,
      })
    } catch {
      // Ignore errors — we clear local session regardless
    }

    clearAccessToken()
    return { logged_out: true }
  })

// ---------------------------------------------------------------------------
// `getCurrentSessionServerFn` — session introspection
// ---------------------------------------------------------------------------

export const getCurrentSessionServerFn = createServerFn({ method: 'GET' })
  .handler(async (): Promise<CurrentSessionResult> => {
    getApiBaseUrl()

    const token = getAccessToken()
    if (token === null) {
      return null
    }
    return {
      has_session: true,
      access_token_preview: truncateAccessToken(token),
      expires_in: null,
    }
  })

// ---------------------------------------------------------------------------
// `clearSessionServerFn` — drop the in-memory token (client-only logout)
// ---------------------------------------------------------------------------

export const clearSessionServerFn = createServerFn({ method: 'POST' })
  .handler(async (): Promise<{ cleared: true }> => {
    getApiBaseUrl()
    clearAccessToken()
    return { cleared: true }
  })
