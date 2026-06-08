/**
 * TypeScript mirrors of the Rust backend's wire DTOs.
 *
 * Source of truth: `crates/application/src/post/dto.rs`,
 * `crates/application/src/user/dto.rs`, and
 * `crates/application/src/pagination.rs`. Field names match the
 * `#[derive(Serialize/Deserialize)]` structs on those files exactly.
 *
 * Conventions:
 * - `i32` → `number`
 * - `Option<T>` → `T | null` (we never use `?:` to stay JSON-faithful —
 *   serde serializes missing as `null`, not as key-omission).
 * - Timestamps are RFC3339 strings (chrono `DateTime<Utc>` →
 *   `to_rfc3339()`), not JS `Date`s, because we want the raw wire value.
 * - `PostStatus` is a const-asserted union so the runtime values stay
 *   the lowercase snake_case that the backend's `#[serde(rename_all =
 *   "lowercase")]` produces.
 *
 * Used by both the server-function wrappers (`src/server/*`) and the
 * route loaders/components that consume them.
 */

// ---------------------------------------------------------------------------
// Auth
// ---------------------------------------------------------------------------

/** Body of `POST /auth/login` — `crates/application/src/user/dto.rs:7-11`. */
export interface LoginCmd {
  username: string
  password: string
}

/**
 * Response of `POST /auth/login` — `crates/application/src/user/dto.rs:13-19`.
 *
 * NOTE: the raw `access_token` / `refresh_token` never leave the server
 * function boundary — the `loginServerFn` returns a `LoginResult` with
 * only a truncated preview of the access token. This interface is the
 * shape of the actual HTTP response, not what the route layer sees.
 */
export interface TokenResponse {
  access_token: string
  refresh_token: string
  token_type: 'Bearer'
  expires_in: number
}

/**
 * The sanitized, client-safe shape that `loginServerFn` returns. The raw
 * tokens are kept in the in-memory `auth-store` on the server side and
 * are never serialized across the RPC boundary.
 */
export interface LoginResult {
  token_type: 'Bearer'
  expires_in: number
  /** First 8 chars of the access token + an ellipsis, e.g. `"eyJ0eXA…"` */
  access_token_preview: string
}

/**
 * Body of `POST /auth/refresh` — refresh token rotation.
 */
export interface RefreshCmd {
  refresh_token: string
}

/**
 * Result of `POST /auth/refresh` — same shape as login, new tokens.
 */
export interface RefreshResult {
  token_type: 'Bearer'
  expires_in: number
  access_token_preview: string
}

/**
 * Result of `POST /auth/logout`.
 */
export interface LogoutResult {
  logged_out: boolean
}

// ---------------------------------------------------------------------------
// Posts
// ---------------------------------------------------------------------------

/**
 * `PostStatus` — `crates/application/src/post/entity.rs:6-12`.
 * `#[serde(rename_all = "lowercase")]` — values are the lowercase enum
 * names, NOT a separate "deleted" variant (soft-delete is a timestamp
 * column, not a status).
 */
export const PostStatus = {
  Draft: 'draft',
  Published: 'published',
  Archived: 'archived',
} as const

export type PostStatus = (typeof PostStatus)[keyof typeof PostStatus]

/**
 * `PostSort` — `crates/application/src/post/dto.rs:79-100`. Default is
 * `newest` (per the `#[default]` attribute on the enum).
 */
export const PostSort = {
  Newest: 'newest',
  Oldest: 'oldest',
  RecentlyUpdated: 'recently_updated',
  TitleAsc: 'title_asc',
  MostViewed: 'most_viewed',
} as const

export type PostSort = (typeof PostSort)[keyof typeof PostSort]

/**
 * Body of `POST /posts` — `crates/application/src/post/dto.rs:47-56`.
 *
 * Server-side rules (validated by the backend, but documented here for
 * the form layer's benefit):
 *   - `title` required, non-empty after trim
 *   - `excerpt` max 500 chars when present
 *   - `slug` 1..=140 chars, `[A-Za-z0-9._-]`, no leading/trailing `-`
 */
export interface CreatePostCmd {
  title: string
  content: string
  excerpt?: string | null
  slug?: string | null
}

/**
 * Body of `PUT /posts/:id` — `crates/application/src/post/dto.rs:67-76`.
 * Every field optional; omitted fields are left unchanged. Same
 * validation rules as create (applied only when a field is present).
 */
export interface UpdatePostCmd {
  title?: string | null
  content?: string | null
  excerpt?: string | null
  slug?: string | null
}

/**
 * Query parameters for `GET /posts` — `crates/application/src/post/dto.rs:111-128`.
 *
 * `cursor` is the **`next_cursor` returned from the previous page** (an
 * `i32` post id), not an opaque base64 token. `limit` defaults to 20
 * server-side. `sort` defaults to `newest`. `include_deleted` defaults
 * to `false`.
 */
export interface ListPostsQuery {
  cursor?: number | null
  limit?: number | null
  status?: PostStatus | null
  author_id?: string | null
  q?: string | null
  sort?: PostSort | null
  include_deleted?: boolean | null
}

/**
 * Author sub-object on `PostDto` — `crates/application/src/post/dto.rs:146-149`.
 * `null` when the post's author is not joined (e.g. service-only paths).
 */
export interface PostAuthorDto {
  id: string
  username: string
}

/**
 * Wire shape of a post — `crates/application/src/post/dto.rs:160-174`.
 *
 * `status` is the post's *state machine* status (`draft` / `published`
 * / `archived`). The "deleted" condition is the separate `deleted_at`
 * timestamp; do NOT treat a missing status as "deleted".
 */
export interface PostDto {
  id: number
  title: string
  content: string
  status: PostStatus
  slug: string | null
  excerpt: string | null
  author: PostAuthorDto | null
  /** RFC3339 timestamp string (chrono `DateTime<Utc>::to_rfc3339()`). */
  created_at: string
  /** RFC3339 timestamp string. */
  updated_at: string
  /** RFC3339 timestamp string, or `null` if never published. */
  published_at: string | null
  /** RFC3339 timestamp string, or `null` if never soft-deleted. */
  deleted_at: string | null
  view_count: number
}

// ---------------------------------------------------------------------------
// Pagination
// ---------------------------------------------------------------------------

/**
 * `CursorPage<T>` — `crates/application/src/pagination.rs:17-22`.
 * Generic envelope used by `GET /posts` (and other list endpoints).
 */
export interface CursorPage<T> {
  data: T[]
  /** Last id on this page; pass back as `cursor` for the next request. */
  next_cursor: number | null
  has_more: boolean
}

// ---------------------------------------------------------------------------
// Error envelope (mirror of the Rust `AppError` JSON response)
// ---------------------------------------------------------------------------

/**
 * `AppError` JSON shape — `crates/api/src/error.rs:38-43`. `apiFetch`
 * already normalizes this into `UnauthorizedError` / `ApiError` on the
 * client side, so most call sites never see this type — it's exported
 * only for components that want to introspect the raw response payload.
 */
export interface BackendErrorEnvelope {
  success: false
  error: string
}

// ---------------------------------------------------------------------------
// Notifications
// ---------------------------------------------------------------------------

/**
 * `NotificationType` — `crates/application/src/notification/entity.rs`.
 * Lowercase snake_case values from the backend enum.
 */
export const NotificationType = {
  PostPublished: 'post_published',
  CommentReceived: 'comment_received',
  Mention: 'mention',
  System: 'system',
} as const

export type NotificationType = (typeof NotificationType)[keyof typeof NotificationType]

/**
 * `NotificationDto` — `crates/application/src/notification/dto.rs`.
 */
export interface NotificationDto {
  id: number
  user_id: string
  notification_type: string
  title: string
  body: string
  data: Record<string, string | number | boolean | null> | null
  is_read: boolean
  /** RFC3339 timestamp string. */
  created_at: string
  /** RFC3339 timestamp string, or `null` if unread. */
  read_at: string | null
}

/**
 * Query parameters for `GET /notifications`.
 */
export interface NotificationListQuery {
  is_read?: boolean | null
  limit?: number | null
  offset?: number | null
}

/**
 * Response for unread count — `GET /notifications/unread/count`.
 */
export interface UnreadCountResponse {
  count: number
}

/**
 * The MVP has no `/me` endpoint, so `getCurrentSessionServerFn` returns
 * this minimal shape: `null` when the user is not signed in, or a
 * truncated preview (first 8 chars) when they are. The full token is
 * never returned to the client.
 */
export interface CurrentSession {
  has_session: true
  access_token_preview: string
  expires_in: number | null
}

export type CurrentSessionResult = CurrentSession | null
