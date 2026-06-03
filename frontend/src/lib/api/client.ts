/**
 * Minimal typed wrapper around `fetch` for the Rust backend.
 *
 * Used exclusively from inside `createServerFn` server functions (Task 7+).
 * Loaders and React components must NOT import this directly — they should
 * go through a `createServerFn` so the call runs server-side and the
 * backend URL / bearer token never reach the browser bundle.
 *
 * Contract:
 *   - Base URL comes from `getApiBaseUrl()` (server-only `process.env`).
 *   - `Authorization: Bearer <accessToken>` is injected when the in-memory
 *     auth store has a token.
 *   - `Content-Type: application/json` is set automatically when a body
 *     is supplied and the caller did not override the header.
 *   - 2xx responses return parsed JSON (`undefined` for `204 No Content`).
 *   - `401` throws `UnauthorizedError`; any other non-2xx throws `ApiError`
 *     with the backend's `error` field as the message.
 */

import { getApiBaseUrl } from './config.ts'
import { getAccessToken } from './auth-store.ts'
import { ApiError, UnauthorizedError } from './errors.ts'

export interface ApiFetchInit extends Omit<RequestInit, 'body' | 'headers'> {
  /**
   * Request body. Will be JSON-serialized. If `undefined` / `null` no body
   * is sent (and no `Content-Type` is set automatically).
   */
  body?: unknown
  /**
   * Additional headers. `Content-Type` and `Authorization` set here
   * take precedence over the defaults.
   */
  headers?: HeadersInit
}

interface BackendErrorEnvelope {
  success?: false
  error?: string
}

function extractErrorMessage(payload: unknown, response: Response): string {
  if (
    payload !== null &&
    typeof payload === 'object' &&
    'error' in payload &&
    typeof (payload as BackendErrorEnvelope).error === 'string' &&
    (payload as BackendErrorEnvelope).error!.length > 0
  ) {
    return (payload as BackendErrorEnvelope).error as string
  }
  return response.statusText || `HTTP ${response.status}`
}

async function parseJsonSafe(response: Response): Promise<unknown> {
  const contentType = response.headers.get('Content-Type') ?? ''
  if (!contentType.includes('application/json')) {
    return null
  }
  try {
    return await response.json()
  } catch {
    return null
  }
}

/**
 * Send a request to the Rust backend and return the typed JSON response.
 *
 * @typeParam T — expected shape of the successful response body.
 * @param path  — path beginning with `/` (e.g. `/auth/login`, `/posts`).
 * @param init  — fetch options; `body` is JSON-serialized, headers are
 *                merged with the auth + content-type defaults.
 */
export async function apiFetch<T>(path: string, init: ApiFetchInit = {}): Promise<T> {
  const baseUrl = getApiBaseUrl()
  const normalizedPath = path.startsWith('/') ? path : `/${path}`
  const url = `${baseUrl}${normalizedPath}`

  const headers = new Headers(init.headers)

  const hasBody = init.body !== undefined && init.body !== null
  if (hasBody && !headers.has('Content-Type')) {
    headers.set('Content-Type', 'application/json')
  }

  const token = getAccessToken()
  if (token !== null && !headers.has('Authorization')) {
    headers.set('Authorization', `Bearer ${token}`)
  }

  // Build the RequestInit with an explicit, correctly-typed body. We
  // can't spread `init` (whose `body` is `unknown`) and then overwrite
  // because TS still sees the spread's narrower type.
  const {
    body: _ignored,
    headers: _ignoredHeaders,
    ...rest
  } = init
  const requestInit: RequestInit = {
    ...rest,
    headers,
    ...(hasBody ? { body: JSON.stringify(init.body) } : {}),
  }

  const response = await fetch(url, requestInit)

  // 204 No Content — no body to parse.
  if (response.status === 204) {
    return undefined as T
  }

  const payload = await parseJsonSafe(response)

  if (response.ok) {
    return payload as T
  }

  const message = extractErrorMessage(payload, response)
  if (response.status === 401) {
    throw new UnauthorizedError(message)
  }
  throw new ApiError(response.status, message)
}
