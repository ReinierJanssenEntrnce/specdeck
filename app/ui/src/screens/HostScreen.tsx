import { useCallback, useEffect, useReducer, useRef, useState } from 'react';
import { AlertTriangle, Copy, Link2, Power, RefreshCw } from 'lucide-react';
import { errorMessage, useBridge } from '../api/bridge';
import type { AgentId, AgentStatus, HostEvent, Participant, SharingState, TimelineEvent } from '../api/types';
import { ConfirmDialog } from '../components/ConfirmDialog';
import { StatusPill } from '../components/StatusPill';
import { Toast, useToast } from '../components/Toast';
import { formatCode, joinLink } from '../model/code';
import { Workspace, type WorkspaceHandle } from './Workspace';

export const AGENT_NAMES: Record<AgentId, string> = { 'claude-code': 'Claude Code', 'open-code': 'OpenCode', omp: 'omp' };

interface State {
  agent: AgentId | null;
  folder: string;
  timelineSupported: boolean;
  sharing: SharingState;
  code: string | null;
  participants: Participant[];
  status: AgentStatus;
  viewerLimit: number;
  exited: number | null;
  timeline: TimelineEvent[];
}

const initial: State = {
  agent: null,
  folder: '',
  timelineSupported: false,
  sharing: 'connecting',
  code: null,
  participants: [],
  status: 'starting',
  viewerLimit: 20,
  exited: null,
  timeline: [],
};

export function hostReducer(state: State, event: HostEvent | { type: 'limit'; limit: number }): State {
  switch (event.type) {
    case 'snapshot':
      return { ...state, ...event, timeline: event.timeline };
    case 'sharing':
      return { ...state, sharing: event.state, code: event.code ?? (event.state === 'connecting' ? null : state.code) };
    case 'participants':
      return { ...state, participants: event.participants };
    case 'status':
      return { ...state, status: event.status };
    case 'timeline':
      return { ...state, timeline: [...state.timeline, event.event] };
    case 'exited':
      return { ...state, exited: event.code, status: 'exited' };
    case 'limit':
      return { ...state, viewerLimit: event.limit };
  }
}

/** The host's session: the agent's terminal, the invite code and everything viewers see. */
export function HostScreen({ onEnded }: { onEnded: () => void }) {
  const bridge = useBridge();
  const workspace = useRef<WorkspaceHandle>(null);
  const [state, dispatch] = useReducer(hostReducer, initial);
  const [confirmEnd, setConfirmEnd] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [toast, showToast] = useToast();

  const attached = useRef(false);
  useEffect(() => {
    // Attach once, also when React mounts the screen twice in development. The window gets the
    // whole session from the start, also after it was closed and reopened.
    if (attached.current) return;
    attached.current = true;
    workspace.current?.resetTerminal();
    bridge.hostAttach((data) => workspace.current?.terminalData(data), dispatch).catch((e) => setError(errorMessage(e)));
  }, [bridge]);

  const onInput = useCallback((data: string) => void bridge.hostInput(data).catch(() => {}), [bridge]);
  const onResize = useCallback((cols: number, rows: number) => void bridge.hostResize(cols, rows).catch(() => {}), [bridge]);
  const onThemeColors = useCallback((fg: string, bg: string) => void bridge.hostSetColors(fg, bg).catch(() => {}), [bridge]);

  const agentName = state.agent ? AGENT_NAMES[state.agent] : 'Agent';
  const folderName = state.folder.split(/[\\/]/).filter(Boolean).at(-1) ?? state.folder;
  const viewers = state.participants.filter((p) => p.role === 'Viewer').length;

  const copy = async (text: string, message: string) => {
    try {
      await bridge.copyText(text);
      showToast(message);
    } catch {
      showToast('Could not copy. Select the code and copy it yourself.');
    }
  };

  const end = async () => {
    setConfirmEnd(false);
    await bridge.hostClose();
    onEnded();
  };

  return (
    <div className="session">
      <header className="toolbar">
        <div className="title">
          <strong title={state.folder}>{folderName || 'Starting…'}</strong>
          <span>
            {agentName} · {state.folder}
          </span>
        </div>
        <StatusPill status={state.status} agentName={agentName} />
        <div className="spacer" />
        <InviteCode
          sharing={state.sharing}
          code={state.code}
          onCopyCode={() => state.code && copy(formatCode(state.code), 'Invite code copied')}
          onCopyLink={() => state.code && copy(joinLink(state.code), 'Join link copied')}
          onNewCode={() => bridge.hostNewCode()}
        />
        <button className="btn danger" onClick={() => setConfirmEnd(true)}>
          <Power aria-hidden /> End session
        </button>
      </header>
      <div>
        {error && (
          <div className="banner danger" role="alert">
            <AlertTriangle aria-hidden /> {error}
          </div>
        )}
        {state.sharing === 'unavailable' && (
          <div className="banner warning" role="status">
            <AlertTriangle aria-hidden />
            Sharing is unavailable: SpecDeck's sharing service cannot be reached. Your session keeps running here, and SpecDeck keeps
            trying; the invite code appears once it can be shared.
          </div>
        )}
        {state.exited !== null && (
          <div className="banner info" role="status">
            {agentName} has exited. Viewers can still read the session; end it when you are done.
          </div>
        )}
      </div>
      <Workspace
        ref={workspace}
        mode="host"
        agentName={agentName}
        timelineSupported={state.timelineSupported}
        timeline={state.timeline}
        participants={state.participants}
        me={0}
        onInput={onInput}
        onResize={onResize}
        onThemeColors={onThemeColors}
        onRemove={(p) => bridge.hostRemoveViewer(p.id)}
        viewerLimit={state.viewerLimit}
        onViewerLimit={(limit) => {
          dispatch({ type: 'limit', limit });
          void bridge.hostSetViewerLimit(limit);
        }}
      />
      {confirmEnd && (
        <ConfirmDialog title="End the session for everyone?" confirm="End session" danger onConfirm={end} onCancel={() => setConfirmEnd(false)}>
          <p>
            {agentName} stops
            {viewers > 0 ? `, and ${viewers === 1 ? '1 viewer is' : `${viewers} viewers are`} disconnected` : ''}. The invite code stops
            working.
          </p>
        </ConfirmDialog>
      )}
      <Toast message={toast} />
    </div>
  );
}

function InviteCode(props: {
  sharing: SharingState;
  code: string | null;
  onCopyCode: () => void;
  onCopyLink: () => void;
  onNewCode: () => void;
}) {
  const [confirmNew, setConfirmNew] = useState(false);
  if (props.sharing !== 'ready' || !props.code) {
    return (
      <div className="invite" aria-live="polite">
        <span className="pending">{props.sharing === 'unavailable' ? 'Sharing unavailable' : 'Getting an invite code…'}</span>
        {props.sharing === 'connecting' && <span className="spinner" style={{ margin: '0 6px' }} />}
      </div>
    );
  }
  return (
    <div className="invite" aria-label="Invite code">
      <span className="label">Invite code</span>
      <code className="selectable">{formatCode(props.code)}</code>
      <button className="btn ghost small" onClick={props.onCopyCode} title="Copy the code">
        <Copy aria-hidden /> Copy
      </button>
      <button className="btn ghost small icon" onClick={props.onCopyLink} aria-label="Copy join link" title="Copy a join link">
        <Link2 aria-hidden />
      </button>
      <button className="btn ghost small icon" onClick={() => setConfirmNew(true)} aria-label="Make a new code" title="New code">
        <RefreshCw aria-hidden />
      </button>
      {confirmNew && (
        <ConfirmDialog
          title="Make a new invite code?"
          confirm="New code"
          onCancel={() => setConfirmNew(false)}
          onConfirm={() => {
            setConfirmNew(false);
            props.onNewCode();
          }}
        >
          <p>The current code stops working. People already watching stay; share the new code with anyone still to join.</p>
        </ConfirmDialog>
      )}
    </div>
  );
}
