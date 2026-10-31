import { useEffect, useState, type FormEvent } from 'react';
import { errorMessage, useBridge } from '../api/bridge';
import type { Profile } from '../api/types';

/** The settings window: changes apply at once, as is usual for app settings. */
export function SettingsScreen() {
  const bridge = useBridge();
  const [profile, setProfile] = useState<Profile | null>(null);
  const [name, setName] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [saved, setSaved] = useState(false);

  useEffect(() => {
    void bridge.getProfile().then((p) => {
      setProfile(p);
      setName(p.username ?? '');
    });
  }, [bridge]);

  if (!profile) return <div className="settings" aria-busy="true" />;

  const saveName = async (event?: FormEvent) => {
    event?.preventDefault();
    if (name.trim() === (profile.username ?? '')) return;
    if (!name.trim()) return setError('Enter the name others should see.');
    try {
      setProfile(await bridge.setUsername(name));
      setError(null);
      setSaved(true);
    } catch (e) {
      setError(errorMessage(e));
    }
  };

  return (
    <main className="settings">
      <form className="row" onSubmit={saveName}>
        <label htmlFor="settings-name">Your name</label>
        <div>
          <input
            id="settings-name"
            className="field"
            maxLength={64}
            value={name}
            onChange={(event) => {
              setName(event.target.value);
              setSaved(false);
            }}
            onBlur={() => void saveName()}
            aria-describedby="settings-name-hint"
            aria-invalid={error ? true : undefined}
          />
          <span id="settings-name-hint" className="field-hint">
            {error ? (
              <span className="error-text" role="alert">
                {error}
              </span>
            ) : saved ? (
              'Saved. Sessions you start or join from now on use this name.'
            ) : (
              'Shown to everyone in a session. It is a label, not a login.'
            )}
          </span>
        </div>
      </form>
      <div className="row">
        <span id="settings-notifications">Notifications</span>
        <div>
          <label className="check">
            <input
              type="checkbox"
              checked={profile.notifications}
              aria-labelledby="settings-notifications"
              aria-describedby="settings-notifications-hint"
              onChange={async (event) => setProfile(await bridge.setNotifications(event.target.checked))}
            />
            Show notifications
          </label>
          <span id="settings-notifications-hint" className="field-hint">
            When someone joins or leaves your session, when the agent waits for you, and when a session you watch ends. Only
            while SpecDeck is not in front.
          </span>
        </div>
      </div>
    </main>
  );
}
