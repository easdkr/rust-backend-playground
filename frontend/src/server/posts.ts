/**
 * Server functions for the MVP post operations.
 *
 * These wrap the four MVP endpoints the frontend may call:
 *   - `GET    /posts`      — `listPostsServerFn`
 *   - `GET    /posts/:id`  — `getPostServerFn`
 *   - `POST   /posts`      — `createPostServerFn`
 *   - `PUT    /posts/:id`  — `updatePostServerFn`
 *
 * All four run server-side only. The bearer token is attached
 * automatically by `apiFetch` from the in-memory auth store — the
 * server function layer never receives or passes the token explicitly.
 *
 * The "extra" call to `getApiBaseUrl()` at the top of every handler is
 * intentional: it forces the `ApiConfigError` path to be exercised
 * inside the handler (not at module load), so every call surface has
 * the same env-missing behavior.
 */

import { createServerFn } from '@tanstack/react-start'

import { apiFetch, getApiBaseUrl } from '~/lib/api'
import type {
  CreatePostCmd,
  CursorPage,
  ListPostsQuery,
  PostDto,
  UpdatePostCmd,
} from '~/lib/api/types.ts'

// ---------------------------------------------------------------------------
// `GET /posts` — list posts with cursor pagination + filter/sort/search
// ---------------------------------------------------------------------------

/**
 * Fetch a page of posts. The `query` argument is optional — passing
 * `{}` (or `undefined`) yields the backend's default page (20 most
 * recent, no filter, no search). To paginate, pass the previous
 * page's `next_cursor` back in as `query.cursor`.
 *
 * @example
 *   const page = await listPostsServerFn({ data: { limit: 10 } })
 *   const next = page.has_more
 *     ? await listPostsServerFn({ data: { cursor: page.next_cursor, limit: 10 } })
 *     : null
 */
export const listPostsServerFn = createServerFn({ method: 'GET' })
  .inputValidator((data: ListPostsQuery | undefined) => data ?? {})
  .handler(
    async ({ data }): Promise<CursorPage<PostDto>> => {
      getApiBaseUrl()

      // Build the query string from the optional input. We only emit
      // keys the caller actually set — the backend applies its own
      // defaults (limit=20, sort=newest) for everything else.
      const params = new URLSearchParams()
      if (data.cursor !== undefined && data.cursor !== null) {
        params.set('cursor', String(data.cursor))
      }
      if (data.limit !== undefined && data.limit !== null) {
        params.set('limit', String(data.limit))
      }
      if (data.status !== undefined && data.status !== null) {
        params.set('status', data.status)
      }
      if (data.author_id !== undefined && data.author_id !== null) {
        params.set('author_id', data.author_id)
      }
      if (data.q !== undefined && data.q !== null) {
        params.set('q', data.q)
      }
      if (data.sort !== undefined && data.sort !== null) {
        params.set('sort', data.sort)
      }
      if (data.include_deleted !== undefined && data.include_deleted !== null) {
        params.set('include_deleted', data.include_deleted ? 'true' : 'false')
      }

      const queryString = params.toString()
      const path = queryString.length > 0 ? `/posts?${queryString}` : '/posts'

      return apiFetch<CursorPage<PostDto>>(path, { method: 'GET' })
    },
  )

// ---------------------------------------------------------------------------
// `GET /posts/:id` — post detail
// ---------------------------------------------------------------------------

/**
 * Fetch a single post by id. The backend spawns a fire-and-forget
 * `view_count` increment on the request (`handlers.rs:104-110`), so
 * the returned `view_count` is **the value at the time of read**, not
 * the value after the increment. The MVP UI should not display the
 * view count prominently — show it as a subtle metadata line at most.
 */
export const getPostServerFn = createServerFn({ method: 'GET' })
  .inputValidator((data: { id: number }) => data)
  .handler(async ({ data }): Promise<PostDto> => {
    getApiBaseUrl()
    return apiFetch<PostDto>(`/posts/${data.id}`, { method: 'GET' })
  })

// ---------------------------------------------------------------------------
// `POST /posts` — create post
// ---------------------------------------------------------------------------

/**
 * Create a new post. The backend always returns the new `PostDto` with
 * `status: "draft"` and `view_count: 0`. The HTTP status on success is
 * `201 Created`, but the wrapper just returns the parsed JSON — the
 * route layer is responsible for any post-create navigation.
 *
 * Throws `UnauthorizedError` if the caller has no token, `ApiError`
 * with status 400 if validation fails, and 403 if the caller lacks the
 * `PostCreate` permission.
 */
export const createPostServerFn = createServerFn({ method: 'POST' })
  .inputValidator((data: CreatePostCmd) => data)
  .handler(async ({ data }): Promise<PostDto> => {
    getApiBaseUrl()
    return apiFetch<PostDto>('/posts', { method: 'POST', body: data })
  })

// ---------------------------------------------------------------------------
// `PUT /posts/:id` — update post
// ---------------------------------------------------------------------------

/**
 * Update an existing post. All fields in `cmd` are optional — omitted
 * fields are left unchanged. The backend validates per-field (non-empty
 * title, slug rules, 500-char excerpt cap) and returns the updated
 * `PostDto`.
 *
 * Throws `UnauthorizedError` (no token), `ApiError` 403 (insufficient
 * role OR not the post owner), 404 (post does not exist), or 400
 * (validation).
 */
export const updatePostServerFn = createServerFn({ method: 'POST' })
  .inputValidator((data: { id: number; cmd: UpdatePostCmd }) => data)
  .handler(async ({ data }): Promise<PostDto> => {
    getApiBaseUrl()
    return apiFetch<PostDto>(`/posts/${data.id}`, {
      method: 'PUT',
      body: data.cmd,
    })
  })
