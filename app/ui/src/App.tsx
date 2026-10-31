import { useEffect, useState } from 'react';
import { errorMessage, useBridge } from './api/bridge';
import type { Profile } from './api/types';
import { ConfirmDialog } from './components/ConfirmDialog';
import { ShortcutsDialog } from './components/ShortcutsDialog';
import { dispatchMenu, useMenu, type MenuCommand } from './menu/menu';
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
  // A host or join command for the start screen, and one waiting for the viewer to confirm leaving.
  const [startCommand, setStartCommand] = useState<MenuCommand | null>(null);
  const [leaveFor, setLeaveFor] = useState<MenuCommand | null>(null);
  const [shortcuts, setShortcuts] = useState(false);

  const refresh = () =>
    bridge.getProfile().then((p) => {
      setProfile(p);
      // Styles that depend on the platform and on the window's material.
      document.documentElement.dataset.platform = p.platform;
      document.documentElement.classList.toggle('translucent', p.translucent);
      return p;
    });

  useEffect(() => bridge.onMenu(dispatchMenu), [bridge]);
  useMenu((command) => {
    if (command.id === 'shortcuts') return setShortcuts(true);
    // Changed in the settings window.
    if (command.id === 'profile-changed') return void refresh();
    if (!START_COMMANDS.has(command.id)) return;
    if (screen.kind === 'start') setStartCommand(command);
    else if (screen.kind === 'viewer') setLeaveFor(command);
    // A hosted session answers these itself: one session at a time.
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

  return (
    <>
      {screen.kind === 'host' && <HostScreen onEnded={() => refresh().then(() => setScreen({ kind: 'start' }))} />}
      {screen.kind === 'viewer' && (
        <ViewerScreen
          code={screen.code}
          onLeave={(retry) => setScreen(retry ? { kind: 'start', code: retry.code, joinError: retry.message } : { kind: 'start' })}
        />
      )}
      {/* Without a toolbar the top of the window is where it is dragged (macOS title bar). */}
      {screen.kind === 'start' && <div className="drag-strip" data-tauri-drag-region />}
      {screen.kind === 'start' && (
        <StartScreen
          profile={profile}
          onProfile={setProfile}
          onHosting={() => {
            setScreen({ kind: 'host' });
            void refresh();
          }}
          onJoin={(code) => setScreen({ kind: 'viewer', code })}
          initialCode={screen.code}
          joinError={screen.joinError}
          command={startCommand}
          onCommandDone={() => setStartCommand(null)}
        />
      )}
      {leaveFor && (
        <ConfirmDialog
          title="Leave this session?"
          confirm="Leave"
          onCancel={() => setLeaveFor(null)}
          onConfirm={() => {
            void bridge.leaveSession();
            setScreen({ kind: 'start' });
            setStartCommand(leaveFor);
            setLeaveFor(null);
          }}
        >
          <p>You stop watching. You can join again with the invite code while the session runs.</p>
        </ConfirmDialog>
      )}
      {shortcuts && <ShortcutsDialog onClose={() => setShortcuts(false)} />}
    </>
  );
}

/** Menu commands that start something from the start screen. */
const START_COMMANDS = new Set(['host', 'join', 'recent']);
