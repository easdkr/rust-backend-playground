/**
 * Notification bell icon with unread badge.
 *
 * Shows a bell icon with a red dot/badge when there are unread
 * notifications. Clicking opens a dropdown panel with the notification
 * list.
 */

import { useState } from 'react'
import type { NotificationDto } from '~/lib/api/types.ts'
import { Spinner } from './spinner.tsx'

export interface NotificationBellProps {
  /** Unread notification count. */
  unreadCount: number
  /** Recent notifications to display in the dropdown. */
  notifications: NotificationDto[]
  /** Loading state for the notification list. */
  loading?: boolean
  /** Called when a notification is clicked. */
  onNotificationClick?: (notification: NotificationDto) => void
  /** Called when "Mark all as read" is clicked. */
  onMarkAllRead?: () => void
  /** Called when the dropdown is opened (to fetch notifications). */
  onOpen?: () => void
}

export function NotificationBell({
  unreadCount,
  notifications,
  loading = false,
  onNotificationClick,
  onMarkAllRead,
  onOpen,
}: NotificationBellProps) {
  const [open, setOpen] = useState(false)

  const handleToggle = () => {
    if (!open && onOpen) {
      onOpen()
    }
    setOpen(!open)
  }

  return (
    <div className="relative">
      <button
        onClick={handleToggle}
        className="relative rounded-full p-2 text-gray-600 hover:bg-gray-100 hover:text-gray-900 focus:outline-none focus:ring-2 focus:ring-blue-500"
        aria-label="Notifications"
      >
        <svg
          className="h-6 w-6"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={2}
            d="M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9"
          />
        </svg>
        {unreadCount > 0 && (
          <span className="absolute -right-0.5 -top-0.5 flex h-5 w-5 items-center justify-center rounded-full bg-red-500 text-xs font-bold text-white">
            {unreadCount > 99 ? '99+' : unreadCount}
          </span>
        )}
      </button>

      {open && (
        <>
          <div
            className="fixed inset-0 z-10"
            onClick={() => setOpen(false)}
          />
          <div className="absolute right-0 z-20 mt-2 w-80 rounded-lg border border-gray-200 bg-white shadow-lg">
            <div className="flex items-center justify-between border-b border-gray-100 px-4 py-3">
              <h3 className="text-sm font-semibold text-gray-900">Notifications</h3>
              {unreadCount > 0 && onMarkAllRead && (
                <button
                  onClick={onMarkAllRead}
                  className="text-xs text-blue-600 hover:text-blue-800"
                >
                  Mark all as read
                </button>
              )}
            </div>

            <div className="max-h-96 overflow-y-auto">
              {loading ? (
                <div className="flex justify-center py-8">
                  <Spinner size="sm" />
                </div>
              ) : notifications.length === 0 ? (
                <p className="py-8 text-center text-sm text-gray-500">
                  No notifications
                </p>
              ) : (
                notifications.map((n) => (
                  <button
                    key={n.id}
                    onClick={() => {
                      onNotificationClick?.(n)
                      setOpen(false)
                    }}
                    className={`w-full px-4 py-3 text-left transition-colors hover:bg-gray-50 ${
                      !n.is_read ? 'bg-blue-50' : ''
                    }`}
                  >
                    <p className="text-sm font-medium text-gray-900">{n.title}</p>
                    <p className="mt-0.5 text-xs text-gray-600 line-clamp-2">{n.body}</p>
                    <p className="mt-1 text-xs text-gray-400">
                      {formatTime(n.created_at)}
                    </p>
                  </button>
                ))
              )}
            </div>
          </div>
        </>
      )}
    </div>
  )
}

function formatTime(iso: string): string {
  const date = new Date(iso)
  const now = new Date()
  const diffMs = now.getTime() - date.getTime()
  const diffMins = Math.floor(diffMs / 60000)
  const diffHours = Math.floor(diffMs / 3600000)
  const diffDays = Math.floor(diffMs / 86400000)

  if (diffMins < 1) return 'just now'
  if (diffMins < 60) return `${diffMins}m ago`
  if (diffHours < 24) return `${diffHours}h ago`
  if (diffDays < 7) return `${diffDays}d ago`
  return date.toLocaleDateString()
}
