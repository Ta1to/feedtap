// Minimal reconnecting WS client for localhost server
export function connectFeed({ port = 8787 } = {}) {
  let socket;
  let listeners = new Set();
  let timer;

  const url = `ws://127.0.0.1:${port}/stream`;

  function connect() {
    socket = new WebSocket(url);
    socket.onopen = () => {
  for (const cb of listeners) cb({ type: "__status__", event: "open" });
    };
    socket.onmessage = (ev) => {
      try {
        const msg = JSON.parse(ev.data);
        for (const cb of listeners) cb(msg);
      } catch (e) {
        console.warn("WS parse error", e);
      }
    };
    socket.onclose = () => {
  for (const cb of listeners) cb({ type: "__status__", event: "close" });
      timer = setTimeout(connect, 1000);
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
