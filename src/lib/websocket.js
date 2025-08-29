// Optimized reconnecting WS client for localhost server
export function connectFeed({ port = 8787 } = {}) {
  let socket;
  let listeners = new Set();
  let timer;
  let reconnectAttempts = 0;
  const maxReconnectAttempts = 5;

  const url = `ws://127.0.0.1:${port}/stream`;

  function connect() {
    socket = new WebSocket(url);
    socket.onopen = () => {
      reconnectAttempts = 0; // Reset on successful connection
      for (const cb of listeners) cb({ type: "__status__", event: "open" });
    };
    socket.onmessage = (ev) => {
      try {
        const msg = JSON.parse(ev.data);
        // Performance optimization: Skip heartbeat logging
        if (msg.type !== "heartbeat") {
          for (const cb of listeners) cb(msg);
        }
      } catch (e) {
        console.warn("WS parse error", e);
      }
    };
    socket.onclose = () => {
      for (const cb of listeners) cb({ type: "__status__", event: "close" });
      // Exponential backoff for reconnection
      if (reconnectAttempts < maxReconnectAttempts) {
        const delay = Math.min(1000 * Math.pow(2, reconnectAttempts), 10000);
        timer = setTimeout(connect, delay);
        reconnectAttempts++;
      }
    };
    socket.onerror = () => {
      for (const cb of listeners) cb({ type: "__status__", event: "error" });
      try { socket.close(); } catch {}
    };
  }
  connect();

  return {
    subscribe(cb) {
      listeners.add(cb);
      return () => listeners.delete(cb);
    },
    close() {
      try { clearTimeout(timer); socket?.close(); } catch {}
    }
  };
}
