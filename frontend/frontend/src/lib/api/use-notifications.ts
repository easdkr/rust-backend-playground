/**
 * Client-side hook for real-time notifications via WebSocket.
 *
 * The WebSocket connection is established directly from the browser
 * (not through server functions) because:
 *   - Server functions are request/response RPC, not persistent streams
 *   - The browser needs to receive push events without polling
 *   - The JWT token is sent in the first message after connection
 *
 * Connection flow:
 *   1. Open ws://localhost:8080/ws/notifications
 *   2. Send { type: "auth", token: "<jwt>" }
 *   3. Receive { type: "notification", ... } push events
 *   4. Heartbeat ping/pong every 30s
 */

import { useCallback, useEffect, useRef, useState } from 'react'
import type { NotificationDto } from './types.ts'

export type WebSocketStatus = 'connecting' | 'open' | 'closed' | 'error'

export interface UseNotificationsOptions {
  /** JWT access token. If null/undefined, connection is deferred. */
  token: string | null | undefined
  /** Called when a new notification arrives. */
  onNotification?: (notification: NotificationDto) => void
  /** Called when the connection status changes. */
  onStatusChange?: (status: WebSocketStatus) => void
  /** Reconnect delay in ms (default: 3000). */
  reconnectDelay?: number
}

export interface UseNotificationsReturn {
  /** Current connection status. */
  status: WebSocketStatus
  /** Send a message to the server (e.g., mark as read). */
  send: (message: unknown) => boolean
  /** Manually reconnect. */
  reconnect: () => void
  /** Last error message, if any. */
  lastError: string | null
}

const WS_URL = 'ws://localhost:8080/ws/notifications'
const HEARTBEAT_INTERVAL = 30000

export function useNotifications(options: UseNotificationsOptions): UseNotificationsReturn {
  const { token, onNotification, onStatusChange, reconnectDelay = 3000 } = options

  const [status, setStatus] = useState<WebSocketStatus>('closed')
  const [lastError, setLastError] = useState<string | null>(null)

  const wsRef = useRef<WebSocket | null>(null)
  const reconnectTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null)
  const heartbeatRef = useRef<ReturnType<typeof setInterval> | null>(null)
  const onNotificationRef = useRef(onNotification)
  const onStatusChangeRef = useRef(onStatusChange)

  // Keep refs in sync with latest callbacks
  useEffect(() => {
    onNotificationRef.current = onNotification
    onStatusChangeRef.current = onStatusChange
  }, [onNotification, onStatusChange])

  const clearHeartbeat = useCallback(() => {
    if (heartbeatRef.current) {
      clearInterval(heartbeatRef.current)
      heartbeatRef.current = null
    }
  }, [])

  const clearReconnect = useCallback(() => {
    if (reconnectTimerRef.current) {
      clearTimeout(reconnectTimerRef.current)
      reconnectTimerRef.current = null
    }
  }, [])

  const connect = useCallback(() => {
    if (!token) return
    if (wsRef.current?.readyState === WebSocket.OPEN) return

    clearReconnect()
    setStatus('connecting')
    onStatusChangeRef.current?.('connecting')

    const ws = new WebSocket(WS_URL)
    wsRef.current = ws

    ws.onopen = () => {
      setStatus('open')
      setLastError(null)
      onStatusChangeRef.current?.('open')

      // Authenticate
      ws.send(JSON.stringify({ type: 'auth', token }))

      // Start heartbeat
      clearHeartbeat()
      heartbeatRef.current = setInterval(() => {
        if (ws.readyState === WebSocket.OPEN) {
          ws.send(JSON.stringify({ type: 'ping' }))
        }
      }, HEARTBEAT_INTERVAL)
    }

    ws.onmessage = (event) => {
      try {
        const payload = JSON.parse(event.data)

        if (payload.type === 'notification' && payload.notification) {
          onNotificationRef.current?.(payload.notification as NotificationDto)
        } else if (payload.type === 'pong') {
          // Heartbeat response — no-op
        } else if (payload.type === 'error') {
          setLastError(payload.message ?? 'Unknown server error')
        }
      } catch {
        // Ignore non-JSON messages
      }
    }

    ws.onerror = () => {
      setStatus('error')
      setLastError('WebSocket error')
      onStatusChangeRef.current?.('error')
    }

    ws.onclose = () => {
      clearHeartbeat()
      setStatus('closed')
      onStatusChangeRef.current?.('closed')

      // Auto-reconnect
      reconnectTimerRef.current = setTimeout(() => {
        connect()
      }, reconnectDelay)
    }
  }, [token, reconnectDelay, clearHeartbeat, clearReconnect])

  // Connect when token becomes available
  useEffect(() => {
    if (token) {
      connect()
    } else {
      // Close existing connection if token is cleared
      if (wsRef.current) {
        wsRef.current.close()
        wsRef.current = null
      }
      clearHeartbeat()
      clearReconnect()
      setStatus('closed')
    }

    return () => {
      clearHeartbeat()
      clearReconnect()
      if (wsRef.current) {
        wsRef.current.close()
        wsRef.current = null
      }
    }
  }, [token, connect, clearHeartbeat, clearReconnect])

  const send = useCallback((message: unknown): boolean => {
    const ws = wsRef.current
    if (ws?.readyState === WebSocket.OPEN) {
      ws.send(JSON.stringify(message))
      return true
    }
    return false
  }, [])

  const reconnect = useCallback(() => {
    if (wsRef.current) {
      wsRef.current.close()
    }
    connect()
  }, [connect])

  return { status, send, reconnect, lastError }
}
