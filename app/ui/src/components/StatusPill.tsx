import type { AgentStatus } from '../api/types';

const LABELS: Record<AgentStatus, string> = {
  starting: 'Starting',
  working: 'Working',
  waiting: 'Waiting for input',
  exited: 'Agent exited',
  unknown: 'Live',
};

interface Props {
  status: AgentStatus;
  agentName: string;
  terminal?: boolean;
  /** What the agent waits on, for sessions whose timeline cannot show it. */
  request?: string | null;
}

/** What the agent is doing, at a glance. */
export function StatusPill({ status, agentName, terminal = false, request }: Props) {
  const className = status === 'working' ? 'working' : status === 'waiting' ? 'waiting' : status === 'exited' ? 'ended' : '';
  // In a terminal, a working or waiting status comes from whatever program reports it.
  const label = status === 'working' && !terminal ? `${agentName} is working` : terminal && status === 'exited' ? 'Shell exited' : LABELS[status];
  const shownRequest = status === 'waiting' ? request : null;
  // A shell has no activity to share, so there is nothing to explain.
  const title = shownRequest ?? (status === 'unknown' && !terminal ? `${agentName}'s activity is not shared` : undefined);
  return (
    <span className={`pill ${className}`} role="status" title={title}>
      <span className="dot" aria-hidden />
      {label}
      {shownRequest && ': '}
      {shownRequest && <span className="pill-request">{shownRequest}</span>}
    </span>
  );
}
