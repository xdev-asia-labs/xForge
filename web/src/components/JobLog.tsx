import { useEffect, useRef, useState } from 'react';
import { createJobWebSocket, type LogMessage } from '../lib/ws';

interface JobLogProps {
  jobId: string;
  initialOutput?: string | null;
  isRunning: boolean;
}

export default function JobLog({ jobId, initialOutput, isRunning }: JobLogProps) {
  const [lines, setLines] = useState<LogMessage[]>([]);
  const [connected, setConnected] = useState(false);
  const containerRef = useRef<HTMLDivElement>(null);
  const wsRef = useRef<WebSocket | null>(null);

  useEffect(() => {
    if (!isRunning) return;

    const ws = createJobWebSocket(
      jobId,
      (msg) => {
        setLines((prev) => [...prev, msg]);
      },
      () => {
        setConnected(false);
      }
    );

    ws.onopen = () => setConnected(true);
    wsRef.current = ws;

    return () => {
      ws.close();
    };
  }, [jobId, isRunning]);

  useEffect(() => {
    if (containerRef.current) {
      containerRef.current.scrollTop = containerRef.current.scrollHeight;
    }
  }, [lines]);

  return (
    <div className="bg-gray-950 border border-gray-800 rounded-xl overflow-hidden">
      <div className="flex items-center justify-between px-4 py-2 bg-gray-900 border-b border-gray-800">
        <span className="text-sm font-medium text-gray-300">Job Output</span>
        {isRunning && (
          <span className="flex items-center gap-2 text-xs">
            <span
              className={`w-2 h-2 rounded-full ${connected ? 'bg-green-500 animate-pulse' : 'bg-gray-500'}`}
            />
            {connected ? 'Live' : 'Connecting...'}
          </span>
        )}
      </div>

      <div
        ref={containerRef}
        className="terminal-output p-4 max-h-96 overflow-auto"
      >
        {initialOutput && !isRunning && (
          <pre className="text-gray-300 whitespace-pre-wrap">{initialOutput}</pre>
        )}

        {lines.map((line, i) => (
          <div
            key={i}
            className={`${
              line.type === 'stderr'
                ? 'text-red-400'
                : line.type === 'complete'
                ? 'text-yellow-400 font-bold mt-2'
                : 'text-gray-300'
            }`}
          >
            {line.type === 'complete'
              ? `--- Process exited with code ${line.exit_code} ---`
              : line.line}
          </div>
        ))}

        {lines.length === 0 && !initialOutput && (
          <span className="text-gray-600">Waiting for output...</span>
        )}
      </div>
    </div>
  );
}
