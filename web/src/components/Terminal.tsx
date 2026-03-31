import { useEffect, useRef, useState } from 'react';

interface TerminalProps {
  lines: string[];
}

export default function Terminal({ lines }: TerminalProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const [autoScroll, setAutoScroll] = useState(true);

  useEffect(() => {
    if (autoScroll && containerRef.current) {
      containerRef.current.scrollTop = containerRef.current.scrollHeight;
    }
  }, [lines, autoScroll]);

  const handleScroll = () => {
    if (!containerRef.current) return;
    const { scrollTop, scrollHeight, clientHeight } = containerRef.current;
    setAutoScroll(scrollHeight - scrollTop - clientHeight < 50);
  };

  return (
    <div className="bg-gray-950 border border-gray-800 rounded-xl overflow-hidden">
      <div className="flex items-center px-4 py-2 bg-gray-900 border-b border-gray-800 gap-2">
        <span className="w-3 h-3 rounded-full bg-red-500" />
        <span className="w-3 h-3 rounded-full bg-yellow-500" />
        <span className="w-3 h-3 rounded-full bg-green-500" />
        <span className="ml-4 text-xs text-gray-500 font-mono">terminal</span>
      </div>
      <div
        ref={containerRef}
        onScroll={handleScroll}
        className="terminal-output p-4 max-h-[500px] overflow-auto"
      >
        {lines.map((line, i) => (
          <div key={i} className="text-gray-300">
            {line}
          </div>
        ))}
        {lines.length === 0 && (
          <span className="text-gray-600">No output</span>
        )}
      </div>
    </div>
  );
}
