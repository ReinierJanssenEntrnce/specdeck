import { useCallback, useEffect, useState } from 'react';

/** A short confirmation at the bottom of the window, such as "Code copied". */
export function useToast(): [string | null, (message: string) => void] {
  const [message, setMessage] = useState<string | null>(null);
  useEffect(() => {
    if (!message) return;
    const timer = window.setTimeout(() => setMessage(null), 1800);
    return () => window.clearTimeout(timer);
  }, [message]);
  return [message, useCallback((text: string) => setMessage(text), [])];
}

export function Toast({ message }: { message: string | null }) {
  return message ? (
    <div className="toast" role="status">
      {message}
    </div>
  ) : null;
}
