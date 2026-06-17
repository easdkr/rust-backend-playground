/**
 * Server functions for comment operations.
 *
 * Endpoints:
 *   - `GET    /posts/:post_id/comments`                         — listCommentsServerFn
 *   - `POST   /posts/:post_id/comments`                         — createCommentServerFn
 *   - `POST   /posts/:post_id/comments/:parent_comment_id/replies` — createReplyServerFn
 *   - `PUT    /posts/:post_id/comments/:comment_id`             — updateCommentServerFn
 *   - `DELETE /posts/:post_id/comments/:comment_id`             — deleteCommentServerFn
 */

import { createServerFn } from '@tanstack/react-start'

import { apiFetch, getApiBaseUrl } from '~/lib/api'
import type {
  CommentDto,
  CommentThreadDto,
  CreateCommentCmd,
  CursorPage,
  ListCommentsQuery,
  UpdateCommentCmd,
} from '~/lib/api/types.ts'

// ---------------------------------------------------------------------------
// `GET /posts/:post_id/comments` — list comment threads with cursor pagination
// ---------------------------------------------------------------------------

export const listCommentsServerFn = createServerFn({ method: 'GET' })
  .inputValidator((data: { post_id: number; query?: ListCommentsQuery | undefined }) => data)
  .handler(async ({ data }): Promise<CursorPage<CommentThreadDto>> => {
    getApiBaseUrl()

    const q = data.query ?? {}
    const params = new URLSearchParams()
    if (q.cursor !== undefined && q.cursor !== null) {
      params.set('cursor', String(q.cursor))
    }
    if (q.limit !== undefined && q.limit !== null) {
      params.set('limit', String(q.limit))
    }

    const queryString = params.toString()
    const path =
      queryString.length > 0
        ? `/api/posts/${data.post_id}/comments?${queryString}`
        : `/api/posts/${data.post_id}/comments`

    return apiFetch<CursorPage<CommentThreadDto>>(path, { method: 'GET' })
  })

// ---------------------------------------------------------------------------
// `POST /posts/:post_id/comments` — create a root comment
// ---------------------------------------------------------------------------

export const createCommentServerFn = createServerFn({ method: 'POST' })
  .inputValidator((data: { post_id: number; cmd: CreateCommentCmd }) => data)
  .handler(async ({ data }): Promise<CommentDto> => {
    getApiBaseUrl()
    return apiFetch<CommentDto>(`/api/posts/${data.post_id}/comments`, {
      method: 'POST',
      body: data.cmd,
    })
  })

// ---------------------------------------------------------------------------
// `POST /posts/:post_id/comments/:parent_comment_id/replies` — create a reply
// ---------------------------------------------------------------------------

export const createReplyServerFn = createServerFn({ method: 'POST' })
  .inputValidator(
    (data: { post_id: number; parent_comment_id: number; cmd: CreateCommentCmd }) => data,
  )
  .handler(async ({ data }): Promise<CommentDto> => {
    getApiBaseUrl()
    return apiFetch<CommentDto>(
      `/api/posts/${data.post_id}/comments/${data.parent_comment_id}/replies`,
      {
        method: 'POST',
        body: data.cmd,
      },
    )
  })

// ---------------------------------------------------------------------------
// `PUT /posts/:post_id/comments/:comment_id` — update a comment
// ---------------------------------------------------------------------------

export const updateCommentServerFn = createServerFn({ method: 'POST' })
  .inputValidator(
    (data: { post_id: number; comment_id: number; cmd: UpdateCommentCmd }) => data,
  )
  .handler(async ({ data }): Promise<CommentDto> => {
    getApiBaseUrl()
    return apiFetch<CommentDto>(`/api/posts/${data.post_id}/comments/${data.comment_id}`, {
      method: 'PUT',
      body: data.cmd,
    })
  })

// ---------------------------------------------------------------------------
// `DELETE /posts/:post_id/comments/:comment_id` — delete a comment
// ---------------------------------------------------------------------------

export const deleteCommentServerFn = createServerFn({ method: 'POST' })
  .inputValidator((data: { post_id: number; comment_id: number }) => data)
  .handler(async ({ data }): Promise<{ success: boolean; message: string }> => {
    getApiBaseUrl()
    return apiFetch<{ success: boolean; message: string }>(
      `/api/posts/${data.post_id}/comments/${data.comment_id}`,
      {
        method: 'DELETE',
      },
    )
  })
