/**
 * Server functions for notification operations.
 *
 * Endpoints:
 *   - `GET    /notifications`              — listNotificationsServerFn
 *   - `GET    /notifications/unread/count` — getUnreadCountServerFn
 *   - `PATCH  /notifications/:id/read`     — markAsReadServerFn
 *   - `PATCH  /notifications/read/all`     — markAllAsReadServerFn
 */

import { createServerFn } from '@tanstack/react-start'

import { apiFetch, getApiBaseUrl } from '~/lib/api'
import type {
  NotificationDto,
  NotificationListQuery,
  UnreadCountResponse,
} from '~/lib/api/types.ts'

// ---------------------------------------------------------------------------
// `GET /notifications` — list notifications with pagination + filter
// ---------------------------------------------------------------------------

export const listNotificationsServerFn = createServerFn({ method: 'GET' })
  .inputValidator((data: NotificationListQuery | undefined) => data ?? {})
  .handler(async ({ data }): Promise<NotificationDto[]> => {
    getApiBaseUrl()

    const params = new URLSearchParams()
    if (data.is_read !== undefined && data.is_read !== null) {
      params.set('is_read', data.is_read ? 'true' : 'false')
    }
    if (data.limit !== undefined && data.limit !== null) {
      params.set('limit', String(data.limit))
    }
    if (data.offset !== undefined && data.offset !== null) {
      params.set('offset', String(data.offset))
    }

    const queryString = params.toString()
    const path = queryString.length > 0 ? `/notifications?${queryString}` : '/notifications'

    return apiFetch<NotificationDto[]>(path, { method: 'GET' })
  })

// ---------------------------------------------------------------------------
// `GET /notifications/unread/count` — unread notification count
// ---------------------------------------------------------------------------

export const getUnreadCountServerFn = createServerFn({ method: 'GET' })
  .handler(async (): Promise<number> => {
    getApiBaseUrl()

    const response = await apiFetch<UnreadCountResponse>('/notifications/unread/count', {
      method: 'GET',
    })
    return response.count
  })

// ---------------------------------------------------------------------------
// `PATCH /notifications/:id/read` — mark single notification as read
// ---------------------------------------------------------------------------

export const markAsReadServerFn = createServerFn({ method: 'POST' })
  .inputValidator((data: { id: number }) => data)
  .handler(async ({ data }): Promise<NotificationDto> => {
    getApiBaseUrl()
    return apiFetch<NotificationDto>(`/notifications/${data.id}/read`, {
      method: 'PATCH',
    })
  })

// ---------------------------------------------------------------------------
// `PATCH /notifications/read/all` — mark all notifications as read
// ---------------------------------------------------------------------------

export const markAllAsReadServerFn = createServerFn({ method: 'POST' })
  .handler(async (): Promise<{ updated: number }> => {
    getApiBaseUrl()
    return apiFetch<{ updated: number }>('/notifications/read/all', {
      method: 'PATCH',
    })
  })
