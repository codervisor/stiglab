import { useEffect, useRef, useState } from 'react';

interface SSEData {
  state: string;
  output: string | null;
}

export function useSessionLogs(sessionId: string | undefined) {
  const [logs, setLogs] = useState<string>('');
  const [state, setState] = useState<string>('');
  const eventSourceRef = useRef<EventSource | null>(null);

  useEffect(() => {
    if (!sessionId) return;

    const es = new EventSource(`/api/sessions/${sessionId}/logs`);
    eventSourceRef.current = es;

    es.onmessage = (event) => {
      try {
        const data: SSEData = JSON.parse(event.data);
        if (data.output) {
          setLogs(data.output);
        }
        if (data.state) {
          setState(data.state);
        }
      } catch {
        // ignore parse errors
      }
    };

    es.onerror = () => {
      es.close();
    };

    return () => {
      es.close();
    };
  }, [sessionId]);

  return { logs, state };
}
