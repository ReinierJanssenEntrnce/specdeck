import { useEffect, useRef, useState, type FormEvent } from 'react';
import { Folder, FolderOpen, LogIn, X } from 'lucide-react';
import { errorMessage, useBridge } from '../api/bridge';
import type { AgentId, AgentInfo, Profile } from '../api/types';
import { initials } from '../components/PeoplePanel';
import { formatCode, isCompleteCode, isJoinLink } from '../model/code';
import type { MenuCommand } from '../menu/menu';

interface Props {
  profile: Profile;
  onProfile: (profile: Profile) => void;
  onHosting: () => void;
  /** With the pasted join link, which may name another sharing service. */
  onJoin: (code: string, link?: string) => void;
  /** A code to show in the join field, for example after a failed join. */
  initialCode?: string;
  /** A message from a join that did not work. */
  joinError?: string;
  /** A host, join or recent-folder command from the menu, until `onCommandDone`. */
  command?: MenuCommand | null;
  onCommandDone?: () => void;
}

/** Choose to host or to join, and set your name the first time. */
export function StartScreen({ profile, onProfile, onHosting, onJoin, initialCode = '', joinError, command = null, onCommandDone = () => {} }: Props) {
  const bridge = useBridge();
  useEffect(() => void bridge.startScreenShown().catch(() => {}), [bridge]);
  // Without a name there is nothing to host or join yet.
  useEffect(() => {
    if (command && !profile.username) onCommandDone();
  }, [command, profile.username, onCommandDone]);
  if (!profile.username) return <Welcome onProfile={onProfile} />;
  return (
    <main className="start">
      <div className="start-inner">
        <header className="brand">
          <div>
            <h1>SpecDeck</h1>
            <p>Run a coding agent with your team watching along.</p>
          </div>
          <NameBadge profile={profile} onProfile={onProfile} />
        </header>
        <div className="cards">
          <HostCard profile={profile} onProfile={onProfile} onHosting={onHosting} command={command} onCommandDone={onCommandDone} />
          <JoinCard onJoin={onJoin} initialCode={initialCode} error={joinError} command={command} onCommandDone={onCommandDone} />
        </div>
      </div>
    </main>
  );
}

function Welcome({ onProfile }: { onProfile: (profile: Profile) => void }) {
  const bridge = useBridge();
  const [name, setName] = useState('');
  const [error, setError] = useState<string | null>(null);
  const submit = async (event: FormEvent) => {
    event.preventDefault();
    if (!name.trim()) return setError('Enter the name others should see.');
    try {
      onProfile(await bridge.setUsername(name));
    } catch (e) {
      setError(errorMessage(e));
    }
  };
  return (
    <main className="start">
      <form className="welcome" onSubmit={submit}>
        <Logo />
        <h1>Welcome to SpecDeck</h1>
        <p>Run a coding agent on your computer while your team follows along, or join someone else's session.</p>
        <label htmlFor="username" style={{ textAlign: 'left', fontWeight: 600 }}>
          What should others call you?
        </label>
        <input
          id="username"
          className="field"
          autoFocus
          maxLength={64}
          placeholder="Your name"
          value={name}
          onChange={(event) => setName(event.target.value)}
          aria-describedby="username-hint"
        />
        <span id="username-hint" className="field-hint" style={{ textAlign: 'left' }}>
          Shown to everyone in a session. It is a label, not a login.
        </span>
        {error && <span className="error-text" role="alert">{error}</span>}
        <button className="btn primary large" type="submit">
          Continue
        </button>
      </form>
    </main>
  );
}

function NameBadge({ profile, onProfile }: { profile: Profile; onProfile: (profile: Profile) => void }) {
  const bridge = useBridge();
  const [editing, setEditing] = useState(false);
  const [name, setName] = useState(profile.username ?? '');
  if (editing) {
    return (
      <form
        className="me"
        onSubmit={async (event) => {
          event.preventDefault();
          if (!name.trim()) return;
          onProfile(await bridge.setUsername(name));
          setEditing(false);
        }}
      >
        <input className="field" style={{ width: 180, height: 30 }} autoFocus maxLength={64} aria-label="Your name" value={name} onChange={(e) => setName(e.target.value)} />
        <button className="btn small primary" type="submit">
          Save
        </button>
        <button className="btn small ghost" type="button" onClick={() => setEditing(false)}>
          Cancel
        </button>
      </form>
    );
  }
  return (
    <div className="me">
      <span className="avatar" aria-hidden>
        {initials(profile.username ?? '')}
      </span>
      <span>
        You appear as <strong style={{ color: 'var(--text)' }}>{profile.username}</strong>
      </span>
      <button className="btn small ghost" onClick={() => setEditing(true)}>
        Change
      </button>
    </div>
  );
}

function defaultAgent(profile: Profile): AgentId | null {
  const installed = profile.agents.filter((agent) => agent.installed);
  if (profile.lastAgent && installed.some((agent) => agent.id === profile.lastAgent)) return profile.lastAgent;
  return installed[0]?.id ?? null;
}

interface CommandProps {
  command: MenuCommand | null;
  onCommandDone: () => void;
}

function HostCard({
  profile,
  onProfile,
  onHosting,
  command,
  onCommandDone,
}: { profile: Profile; onProfile: (p: Profile) => void; onHosting: () => void } & CommandProps) {
  const bridge = useBridge();
  const [agent, setAgent] = useState<AgentId | null>(() => defaultAgent(profile));
  const [starting, setStarting] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  // Terminal is always there; the coding agents may all be missing.
  const noCodingAgents = profile.agents.every((a) => a.id === 'terminal' || !a.installed);

  const start = async (folder: string | null) => {
    if (!folder || !agent) return;
    setError(null);
    setStarting(folder);
    try {
      // A first guess at the terminal size; the host's terminal fits its pane right after.
      const cols = Math.max(80, Math.floor((window.innerWidth - 420) / 7.8));
      const rows = Math.max(24, Math.floor((window.innerHeight - 120) / 17));
      await bridge.startHosting(agent, folder, cols, rows);
      onHosting();
    } catch (e) {
      setError(errorMessage(e));
      setStarting(null);
    }
  };

  // File > Host a Session… and File > Open Recent.
  useEffect(() => {
    if (command?.id === 'host') {
      onCommandDone();
      void bridge.chooseFolder().then(start);
    } else if (command?.id === 'recent' && command.path) {
      const path = command.path;
      onCommandDone();
      // Started after this effect, like a click would be.
      void Promise.resolve().then(() => start(path));
    }
    // `start` changes every render; the command is what matters.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [command]);

  return (
    <section className="card" aria-labelledby="host-title">
      <div>
        <h2 id="host-title">Host a session</h2>
        <p className="lead">Start a coding agent or a terminal in a project folder. You type; people you invite watch.</p>
      </div>
      <div style={{ display: 'grid', gap: 8 }}>
        <span className="section-label" id="agent-label">
          Agent
        </span>
        <div className="agents" role="radiogroup" aria-labelledby="agent-label">
          {profile.agents.map((info) => (
            <AgentChoice key={info.id} info={info} selected={agent === info.id} onSelect={() => setAgent(info.id)} />
          ))}
        </div>
        {agent === 'terminal' && <span className="field-hint">Everyone you invite sees all output, including any secrets a command prints.</span>}
        {noCodingAgents && (
          <span className="field-hint">No coding agent is installed. Install Claude Code, OpenCode or omp for a timeline; you can still share a terminal.</span>
        )}
      </div>
      {profile.recentFolders.length > 0 && (
        <div style={{ display: 'grid', gap: 6 }}>
          <span className="section-label">Recent folders</span>
          <ul className="recent">
            {profile.recentFolders.map((folder) => (
              <li key={folder.path}>
                <button
                  className="open"
                  onClick={() => start(folder.path)}
                  disabled={!agent || starting !== null}
                  title={`Start in ${folder.path}`}
                  aria-label={`Start a session in ${folder.name}`}
                >
                  {starting === folder.path ? <span className="spinner" aria-label="Starting" /> : <Folder aria-hidden />}
                  <span className="name">{folder.name}</span>
                  <span className="path">{folder.path}</span>
                </button>
                <button
                  className="btn ghost icon small remove"
                  aria-label={`Remove ${folder.name} from recent folders`}
                  title="Remove from this list (the folder is not touched)"
                  onClick={async () => onProfile(await bridge.removeRecentFolder(folder.path))}
                >
                  <X aria-hidden />
                </button>
              </li>
            ))}
          </ul>
        </div>
      )}
      {error && (
        <span className="error-text" role="alert">
          {error}
        </span>
      )}
      <button className="btn primary large" disabled={!agent || starting !== null} onClick={async () => start(await bridge.chooseFolder())}>
        {starting && !profile.recentFolders.some((f) => f.path === starting) ? <span className="spinner" /> : <FolderOpen aria-hidden />}
        {profile.recentFolders.length > 0 ? 'Choose another folder…' : 'Choose a folder and start…'}
      </button>
    </section>
  );
}

function AgentChoice({ info, selected, onSelect }: { info: AgentInfo; selected: boolean; onSelect: () => void }) {
  const detail = !info.installed ? 'Not installed' : info.id === 'terminal' ? 'Your shell' : info.hasTimeline ? 'Timeline and diffs' : 'Terminal only';
  return (
    <button
      className="agent"
      role="radio"
      aria-checked={selected}
      disabled={!info.installed}
      onClick={onSelect}
      title={info.installed ? undefined : `${info.name} was not found. Install it and log in to use it.`}
    >
      <strong>{info.name}</strong>
      <span>{detail}</span>
    </button>
  );
}

function JoinCard({
  onJoin,
  initialCode,
  error,
  command,
  onCommandDone,
}: { onJoin: (code: string, link?: string) => void; initialCode: string; error?: string } & CommandProps) {
  const [code, setCode] = useState(formatCode(initialCode));
  // The field shows only the code; the link it came from is kept for its sharing service.
  const [link, setLink] = useState<string | null>(null);
  const input = useRef<HTMLInputElement>(null);
  // File > Join a Session…
  useEffect(() => {
    if (command?.id !== 'join') return;
    onCommandDone();
    input.current?.focus();
    input.current?.select();
  }, [command, onCommandDone]);
  const complete = isCompleteCode(code);
  return (
    <section className="card" aria-labelledby="join-title">
      <div>
        <h2 id="join-title">Join a session</h2>
        <p className="lead">Follow someone else's agent. You see everything; only the host can type.</p>
      </div>
      <form
        style={{ display: 'grid', gap: 10 }}
        onSubmit={(event) => {
          event.preventDefault();
          if (!complete) return;
          if (link && formatCode(link) === code) onJoin(code, link.trim());
          else onJoin(code);
        }}
      >
        <label htmlFor="invite-code" className="section-label">
          Invite code
        </label>
        <input
          ref={input}
          id="invite-code"
          className="field code"
          placeholder="XXXX-XXXX-XXXX"
          autoComplete="off"
          spellCheck={false}
          value={code}
          onChange={(event) => setCode(formatCode(event.target.value))}
          onPaste={(event) => {
            event.preventDefault();
            const text = event.clipboardData.getData('text');
            setCode(formatCode(text));
            setLink(isJoinLink(text) ? text : null);
          }}
          aria-describedby="code-hint"
          aria-invalid={error ? true : undefined}
        />
        <span id="code-hint" className="field-hint">
          Paste the code or a specdeck:// link from the host. Upper or lower case does not matter.
        </span>
        {error && (
          <span className="error-text" role="alert">
            {error}
          </span>
        )}
        <button className="btn primary large" type="submit" disabled={!complete}>
          <LogIn aria-hidden /> Join
        </button>
      </form>
    </section>
  );
}

export function Logo() {
  return (
    <svg className="logo" viewBox="0 0 1024 1024" aria-hidden>
      <rect width="1024" height="1024" rx="224" fill="#1f2a44" />
      <rect x="300" y="250" width="470" height="560" rx="56" fill="#3d5a99" transform="rotate(-8 535 530)" />
      <rect x="254" y="214" width="470" height="560" rx="56" fill="#f4f1ea" />
      <path d="M348 380 l90 80 -90 80" fill="none" stroke="#1f2a44" strokeWidth="44" strokeLinecap="round" strokeLinejoin="round" />
      <path d="M478 560 h140" stroke="#1f2a44" strokeWidth="44" strokeLinecap="round" />
    </svg>
  );
}
