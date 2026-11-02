// A new SpecDeck that is downloaded and ready: a slim notice on the start screen and a button in
// the session toolbar, so it never covers the terminal (auto-update design decision 8).

import { useEffect, useState } from 'react';
import { Download, X } from 'lucide-react';
import { useBridge } from '../api/bridge';
import type { UpdateState } from '../api/types';

/**
 * The version that is ready to install, or null. Dismissing hides it until the app starts again
 * or a newer version is ready.
 */
export function useReadyUpdate(): { version: string | null; dismiss: () => void } {
  const bridge = useBridge();
  const [state, setState] = useState<UpdateState>({ state: 'idle' });
  const [dismissed, setDismissed] = useState<string | null>(null);
  useEffect(() => {
    let current = true;
    void bridge.updateStatus().then((status) => current && setState(status));
    const stop = bridge.onUpdate(setState);
    return () => {
      current = false;
      stop();
    };
  }, [bridge]);
  const version = state.state === 'ready' && state.version !== dismissed ? state.version : null;
  return { version, dismiss: () => setDismissed(version) };
}

export function UpdateNotice({ version, onRestart, onDismiss }: { version: string; onRestart: () => void; onDismiss: () => void }) {
  return (
    <div className="update-notice" role="status">
      <Download aria-hidden />
      <span>SpecDeck {version} is ready.</span>
      <button className="btn small primary" onClick={onRestart}>
        Restart to update
      </button>
      <button className="btn ghost icon" onClick={onDismiss} aria-label="Not now" title="Not now">
        <X aria-hidden />
      </button>
    </div>
  );
}

export function UpdateButton({ version, onRestart }: { version: string; onRestart: () => void }) {
  return (
    <button className="btn" onClick={onRestart} title={`SpecDeck ${version} is ready. Restart to update.`}>
      <Download aria-hidden /> Update ready
    </button>
  );
}
