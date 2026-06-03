/**
 * Typed error classes for the API proxy layer.
 *
 * The Rust backend returns a uniform error envelope
 *   { "success": false, "error": "<message>" }
 * with an appropriate HTTP status. The client (`apiFetch`) normalizes that
 * into one of these three classes so callers can branch on `instanceof`
 * without having to parse the message.
 *
 * - `ApiError`         — generic non-2xx response (400, 403, 404, 5xx, ...)
 * - `UnauthorizedError`— 401 specifically: token is missing / invalid /
 *                        expired. Callers should drop the token and
 *                        route to login.
 * - `ApiConfigError`   — server-side misconfiguration (e.g. the
 *                        `RUST_API_BASE_URL` env var is unset). Thrown at
 *                        module-load time, not from a network response.
 */

export class ApiError extends Error {
  readonly status: number
  readonly backendMessage: string

  constructor(status: number, backendMessage: string) {
    super(`API error ${status}: ${backendMessage}`)
    this.name = 'ApiError'
    this.status = status
    this.backendMessage = backendMessage
    // Preserve the prototype chain when targeting older runtimes.
    Object.setPrototypeOf(this, new.target.prototype)
  }
}

export class UnauthorizedError extends ApiError {
  constructor(backendMessage: string) {
    super(401, backendMessage)
    this.name = 'UnauthorizedError'
    Object.setPrototypeOf(this, new.target.prototype)
  }
}

export class ApiConfigError extends Error {
  constructor(message: string) {
    super(message)
    this.name = 'ApiConfigError'
    Object.setPrototypeOf(this, new.target.prototype)
  }
}
