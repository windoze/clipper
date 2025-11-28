import { useEffect, useRef, useCallback, useState } from "react";

export interface ClipNotification {
  type: "new_clip" | "updated_clip" | "deleted_clip";
  id: string;
  content?: string;
  tags?: string[];
}

interface UseWebSocketOptions {
  onNewClip?: (id: string, content: string, tags: string[]) => void;
  onUpdatedClip?: (id: string) => void;
  onDeletedClip?: (id: string) => void;
  onError?: (error: string) => void;
  enabled?: boolean;
}

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
 */
export function useWebSocket({
  onNewClip,
  onUpdatedClip,
  onDeletedClip,
  onError,
  enabled = true,
}: UseWebSocketOptions = {}) {
  const wsRef = useRef<WebSocket | null>(null);
  const reconnectTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const [isConnected, setIsConnected] = useState(false);
  const [isSecure] = useState(isSecureContext);

  // Store callbacks in refs to avoid reconnecting when they change
  const callbacksRef = useRef({ onNewClip, onUpdatedClip, onDeletedClip, onError });
  callbacksRef.current = { onNewClip, onUpdatedClip, onDeletedClip, onError };

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
        setIsConnected(true);
      };

      ws.onmessage = (event) => {
        try {
          const notification: ClipNotification = JSON.parse(event.data);

          switch (notification.type) {
            case "new_clip":
              callbacksRef.current.onNewClip?.(
                notification.id,
                notification.content || "",
                notification.tags || []
              );
              break;
            case "updated_clip":
              callbacksRef.current.onUpdatedClip?.(notification.id);
              break;
            case "deleted_clip":
              callbacksRef.current.onDeletedClip?.(notification.id);
              break;
          }
        } catch (e) {
          console.error("Failed to parse WebSocket message:", e);
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

        // Reconnect after a delay if not intentionally closed
        if (enabled && event.code !== 1000) {
          reconnectTimeoutRef.current = setTimeout(() => {
            console.log("Attempting to reconnect WebSocket...");
            connect();
          }, 5000);
        }
      };

      wsRef.current = ws;
    } catch (e) {
      console.error("Failed to create WebSocket:", e);
      callbacksRef.current.onError?.("Failed to create WebSocket connection");
    }
  }, [enabled, isSecure]);

  const disconnect = useCallback(() => {
    if (reconnectTimeoutRef.current) {
      clearTimeout(reconnectTimeoutRef.current);
      reconnectTimeoutRef.current = null;
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
