import { useEffect, useState } from 'react';
import { errorMessage, useBridge } from './api/bridge';
import type { Profile } from './api/types';
import { HostScreen } from './screens/HostScreen';
import { StartScreen } from './screens/StartScreen';
import { ViewerScreen } from './screens/ViewerScreen';

type Screen =
  | { kind: 'start'; code?: string; joinError?: string }
  | { kind: 'host' }
  | { kind: 'viewer'; code: string };

export function App() {
  const bridge = useBridge();
  const [profile, setProfile] = useState<Profile | null>(null);
  const [screen, setScreen] = useState<Screen>({ kind: 'start' });
  const [error, setError] = useState<string | null>(null);

  const refresh = () =>
    bridge.getProfile().then((p) => {
      setProfile(p);
      return p;
    });

  useEffect(() => {
    refresh()
      // A window reopened while a session runs goes straight back to it.
      .then((p) => {
        if (p.hosting) setScreen({ kind: 'host' });
        const demo = import.meta.env.DEV ? new URLSearchParams(location.search).get('demo') : null;
        if (demo === 'viewer') setScreen({ kind: 'viewer', code: '7K2M9QXH4TPA' });
      })
      .catch((e) => setError(errorMessage(e)));
    // Loads once.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  if (error) {
    return (
      <div className="center-state">
        <div className="box" role="alert">
          <h2>SpecDeck could not start</h2>
          <p>{error}</p>
        </div>
      </div>
    );
  }
  if (!profile) {
    return (
      <div className="center-state" aria-busy="true">
        <span className="spinner" aria-label="Loading" />
      </div>
    );
  }

  switch (screen.kind) {
    case 'host':
      return <HostScreen onEnded={() => refresh().then(() => setScreen({ kind: 'start' }))} />;
    case 'viewer':
      return (
        <ViewerScreen
          code={screen.code}
          onLeave={(retry) => setScreen(retry ? { kind: 'start', code: retry.code, joinError: retry.message } : { kind: 'start' })}
        />
      );
    case 'start':
      return (
        <StartScreen
          profile={profile}
          onProfile={setProfile}
          onHosting={() => setScreen({ kind: 'host' })}
          onJoin={(code) => setScreen({ kind: 'viewer', code })}
          initialCode={screen.code}
          joinError={screen.joinError}
        />
      );
  }
}
