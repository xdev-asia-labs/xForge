import { FitAddon } from '@xterm/addon-fit';
import { Terminal } from '@xterm/xterm';
import '@xterm/xterm/css/xterm.css';
import { useEffect, useRef, useState } from 'react';
import { useNavigate, useParams } from 'react-router-dom';
import { getServer, getTerminalWsUrl, type Server } from '../lib/api';

export default function ServerTerminal() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const termRef = useRef<HTMLDivElement>(null);
  const terminalRef = useRef<Terminal | null>(null);
  const wsRef = useRef<WebSocket | null>(null);
  const [server, setServer] = useState<Server | null>(null);
  const [status, setStatus] = useState<'connecting' | 'connected' | 'disconnected' | 'error'>('connecting');
  const [error, setError] = useState('');

  useEffect(() => {
    if (!id) return;
    getServer(id)
      .then(setServer)
      .catch((err) => setError(err.message));
  }, [id]);

  useEffect(() => {
    if (!id || !termRef.current || !server) return;

    const term = new Terminal({
      cursorBlink: true,
      fontSize: 14,
      fontFamily: '"JetBrains Mono", "Fira Code", "Cascadia Code", monospace',
      theme: {
        background: '#0a0a0a',
        foreground: '#e5e5e5',
        cursor: '#e5e5e5',
        selectionBackground: '#3b82f680',
      },
    });

    const fitAddon = new FitAddon();
    term.loadAddon(fitAddon);
    term.open(termRef.current);
    fitAddon.fit();
    terminalRef.current = term;

    term.writeln(`\x1b[1;36mConnecting to ${server.name} (${server.host})...\x1b[0m`);

    const wsUrl = getTerminalWsUrl(id);
    const ws = new WebSocket(wsUrl);
    wsRef.current = ws;

    ws.onopen = () => {
      setStatus('connected');
      term.writeln('\x1b[1;32mConnected!\x1b[0m\r\n');
    };

    ws.onmessage = (event) => {
      try {
        const msg = JSON.parse(event.data);
        if (msg.type === 'output' || msg.type === 'data') {
          term.write(msg.data);
        } else if (msg.type === 'error') {
          term.writeln(`\r\n\x1b[1;31mError: ${msg.data}\x1b[0m`);
          setStatus('error');
          setError(msg.data);
        } else if (msg.type === 'status' && msg.data === 'connected') {
          // SSH session established
        }
      } catch {
        // Raw text message
        term.write(event.data);
      }
    };

    ws.onclose = () => {
      setStatus('disconnected');
      term.writeln('\r\n\x1b[1;33mConnection closed.\x1b[0m');
    };

    ws.onerror = () => {
      setStatus('error');
      setError('WebSocket connection failed');
    };

    term.onData((data) => {
      if (ws.readyState === WebSocket.OPEN) {
        ws.send(JSON.stringify({ type: 'input', data }));
      }
    });

    term.onResize(({ cols, rows }) => {
      if (ws.readyState === WebSocket.OPEN) {
        ws.send(JSON.stringify({ type: 'resize', cols, rows }));
      }
    });

    const handleResize = () => fitAddon.fit();
    window.addEventListener('resize', handleResize);

    return () => {
      window.removeEventListener('resize', handleResize);
      ws.close();
      term.dispose();
    };
  }, [id, server]);

  const statusColors: Record<string, string> = {
    connecting: 'bg-yellow-600/20 text-yellow-400',
    connected: 'bg-green-600/20 text-green-400',
    disconnected: 'bg-gray-600/20 text-gray-400',
    error: 'bg-red-600/20 text-red-400',
  };

  return (
    <div className="flex flex-col h-[calc(100vh-4rem)]">
      {/* Header */}
      <div className="flex items-center justify-between mb-4">
        <div className="flex items-center gap-4">
          <button
            onClick={() => navigate('/servers')}
            className="px-3 py-1.5 bg-gray-800 text-gray-300 text-sm rounded-lg hover:bg-gray-700 transition-colors"
          >
            ← Back
          </button>
          <div>
            <h2 className="text-lg font-bold text-white">
              Terminal: {server?.name || '...'}
            </h2>
            {server && (
              <p className="text-xs text-gray-500 font-mono">
                {server.ssh_user}@{server.host}:{server.port}
              </p>
            )}
          </div>
        </div>
        <span className={`px-2.5 py-1 text-xs font-medium rounded-md ${statusColors[status]}`}>
          {status}
        </span>
      </div>

      {error && (
        <div className="bg-red-600/20 border border-red-600/30 text-red-400 text-sm p-3 rounded-lg mb-4">
          {error}
        </div>
      )}

      {/* Terminal */}
      <div
        ref={termRef}
        className="flex-1 bg-[#0a0a0a] rounded-xl border border-gray-800 overflow-hidden p-1"
      />
    </div>
  );
}
