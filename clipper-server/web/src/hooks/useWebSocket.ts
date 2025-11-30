import { useEffect, useRef, useCallback, useState } from "react";

export interface ClipNotification {
  type: "new_clip" | "updated_clip" | "deleted_clip" | "clips_cleaned_up";
  id?: string;
  content?: string;
  tags?: string[];
  ids?: string[];
  count?: number;
}

interface UseWebSocketOptions {
  onNewClip?: (id: string, content: string, tags: string[]) => void;
  onUpdatedClip?: (id: string) => void;
  onDeletedClip?: (id: string) => void;
  onClipsCleanedUp?: (ids: string[], count: number) => void;
  onError?: (error: string) => void;
  onAuthError?: (error: string) => void;
  enabled?: boolean;
  /** Auth token to send after connection (if server requires auth) */
  token?: string;
}

// Connection timeout - if no message received within this time, consider connection dead
// Server sends ping every 30s, so we wait 60s (2x interval) before timing out
const CONNECTION_TIMEOUT_MS = 60_000;

// Reconnection delays (exponential backoff)
const INITIAL_RECONNECT_DELAY_MS = 1_000;
const MAX_RECONNECT_DELAY_MS = 30_000;

/**
 * Check if the current page is served over HTTPS
 */
export function isSecureContext(): boolean {
  return window.location.protocol === "https:";
}

/**
 * Get the WebSocket URL based on the current page URL
 */
function getWebSocketUrl(): string {
  const protocol = window.location.protocol === "https:" ? "wss:" : "ws:";
  const host = window.location.host;
  return `${protocol}//${host}/ws`;
}

/**
 * Hook to manage WebSocket connection to the clipper server.
 * Only connects when running on HTTPS for security.
 * Supports message-based authentication when a token is provided.
 */
export function useWebSocket({
  onNewClip,
  onUpdatedClip,
  onDeletedClip,
  onClipsCleanedUp,
  onError,
  onAuthError,
  enabled = true,
  token,
}: UseWebSocketOptions = {}) {
  const wsRef = useRef<WebSocket | null>(null);
  const reconnectTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const activityTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const reconnectDelayRef = useRef(INITIAL_RECONNECT_DELAY_MS);
  const [isConnected, setIsConnected] = useState(false);
  const [isSecure] = useState(isSecureContext);
  // Track if we're waiting for auth response
  const isAuthenticatingRef = useRef(false);

  // Store callbacks in refs to avoid reconnecting when they change
  const callbacksRef = useRef({ onNewClip, onUpdatedClip, onDeletedClip, onClipsCleanedUp, onError, onAuthError });
  callbacksRef.current = { onNewClip, onUpdatedClip, onDeletedClip, onClipsCleanedUp, onError, onAuthError };

  // Store token in ref to use in callbacks
  const tokenRef = useRef(token);
  tokenRef.current = token;

  // Reset activity timeout - called whenever we receive any message from server
  const resetActivityTimeout = useCallback(() => {
    if (activityTimeoutRef.current) {
      clearTimeout(activityTimeoutRef.current);
    }
    activityTimeoutRef.current = setTimeout(() => {
      console.log("WebSocket activity timeout - no messages received, closing connection");
      if (wsRef.current) {
        wsRef.current.close(4000, "Activity timeout");
      }
    }, CONNECTION_TIMEOUT_MS);
  }, []);

  const connect = useCallback(() => {
    // Only connect if enabled and on HTTPS
    if (!enabled || !isSecure) {
      return;
    }

    // Clean up existing connection
    if (wsRef.current) {
      wsRef.current.close();
      wsRef.current = null;
    }

    const wsUrl = getWebSocketUrl();
    console.log("Connecting to WebSocket:", wsUrl);

    try {
      const ws = new WebSocket(wsUrl);

      ws.onopen = () => {
        console.log("WebSocket connected");
        // Reset reconnect delay on successful connection
        reconnectDelayRef.current = INITIAL_RECONNECT_DELAY_MS;
        // Start activity timeout
        resetActivityTimeout();

        // If we have a token, send auth message and wait for response
        if (tokenRef.current) {
          console.log("WebSocket: sending auth message");
          isAuthenticatingRef.current = true;
          ws.send(JSON.stringify({ type: "auth", token: tokenRef.current }));
          // Don't set isConnected yet - wait for auth_success
        } else {
          // No auth required, mark as connected immediately
          setIsConnected(true);
        }
      };

      ws.onmessage = (event) => {
        // Reset activity timeout on any message (including ping responses)
        resetActivityTimeout();

        try {
          const data = JSON.parse(event.data);

          // Handle auth responses first
          if (data.type === "auth_success") {
            console.log("WebSocket: auth successful");
            isAuthenticatingRef.current = false;
            setIsConnected(true);
            return;
          }

          if (data.type === "auth_error") {
            console.error("WebSocket: auth failed:", data.message);
            isAuthenticatingRef.current = false;
            callbacksRef.current.onAuthError?.(data.message || "Authentication failed");
            // Close the connection - server will close it anyway
            ws.close(4001, "Auth failed");
            return;
          }

          // Handle clip notifications (only after authenticated)
          const notification = data as ClipNotification;

          switch (notification.type) {
            case "new_clip":
              callbacksRef.current.onNewClip?.(
                notification.id || "",
                notification.content || "",
                notification.tags || []
              );
              break;
            case "updated_clip":
              callbacksRef.current.onUpdatedClip?.(notification.id || "");
              break;
            case "deleted_clip":
              callbacksRef.current.onDeletedClip?.(notification.id || "");
              break;
            case "clips_cleaned_up":
              callbacksRef.current.onClipsCleanedUp?.(
                notification.ids || [],
                notification.count || 0
              );
              break;
          }
        } catch (e) {
          // Not a JSON message (could be ping/pong), ignore parse error
        }
      };

      ws.onerror = (event) => {
        console.error("WebSocket error:", event);
        callbacksRef.current.onError?.("WebSocket connection error");
      };

      ws.onclose = (event) => {
        console.log("WebSocket closed:", event.code, event.reason);
        setIsConnected(false);
        wsRef.current = null;
        isAuthenticatingRef.current = false;

        // Clear activity timeout
        if (activityTimeoutRef.current) {
          clearTimeout(activityTimeoutRef.current);
          activityTimeoutRef.current = null;
        }

        // Reconnect with exponential backoff if not intentionally closed
        // Don't reconnect on:
        // - 1000: Normal closure
        // - 4001: Auth failed (no point retrying with same token)
        if (enabled && event.code !== 1000 && event.code !== 4001) {
          const delay = reconnectDelayRef.current;
          // Add jitter (±20%) to prevent thundering herd
          const jitter = delay * 0.2 * (Math.random() * 2 - 1);
          const actualDelay = Math.round(delay + jitter);

          console.log(`Attempting to reconnect WebSocket in ${actualDelay}ms...`);
          reconnectTimeoutRef.current = setTimeout(() => {
            connect();
          }, actualDelay);

          // Increase delay for next attempt (exponential backoff)
          reconnectDelayRef.current = Math.min(
            reconnectDelayRef.current * 2,
            MAX_RECONNECT_DELAY_MS
          );
        }
      };

      wsRef.current = ws;
    } catch (e) {
      console.error("Failed to create WebSocket:", e);
      callbacksRef.current.onError?.("Failed to create WebSocket connection");
    }
  }, [enabled, isSecure, resetActivityTimeout]);

  const disconnect = useCallback(() => {
    if (reconnectTimeoutRef.current) {
      clearTimeout(reconnectTimeoutRef.current);
      reconnectTimeoutRef.current = null;
    }
    if (activityTimeoutRef.current) {
      clearTimeout(activityTimeoutRef.current);
      activityTimeoutRef.current = null;
    }
    if (wsRef.current) {
      wsRef.current.close(1000, "Client disconnecting");
      wsRef.current = null;
    }
    setIsConnected(false);
  }, []);

  useEffect(() => {
    connect();
    return () => {
      disconnect();
    };
  }, [connect, disconnect]);

  return {
    isConnected,
    isSecure,
    reconnect: connect,
    disconnect,
  };
}
