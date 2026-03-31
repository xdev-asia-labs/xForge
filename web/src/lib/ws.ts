export interface LogMessage {
  job_id: string;
  type: 'stdout' | 'stderr' | 'complete';
  line?: string;
  exit_code?: number;
  timestamp: string;
}

export function createJobWebSocket(
  jobId: string,
  onMessage: (msg: LogMessage) => void,
  onClose?: () => void
): WebSocket {
  const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
  const host = window.location.host;
  const ws = new WebSocket(`${protocol}//${host}/api/ws?job_id=${encodeURIComponent(jobId)}`);

  ws.onmessage = (event) => {
    try {
      const msg: LogMessage = JSON.parse(event.data);
      onMessage(msg);
    } catch {
      // Ignore non-JSON messages
    }
  };

  ws.onclose = () => {
    onClose?.();
  };

  return ws;
}
