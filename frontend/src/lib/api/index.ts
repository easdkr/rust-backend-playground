/**
 * Public surface of the API proxy layer.
 *
 * Server-only — import from `createServerFn` server functions, not from
 * React components or loaders. See `frontend/README.md` for the rationale.
 */

export { getApiBaseUrl, __resetApiConfigForTesting } from './config.ts'
export {
  getAccessToken,
  setAccessToken,
  clearAccessToken,
} from './auth-store.ts'
export { apiFetch, type ApiFetchInit } from './client.ts'
export { ApiError, ApiConfigError, UnauthorizedError } from './errors.ts'
