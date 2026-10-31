import type { AgentStatus } from '../api/types';

const LABELS: Record<AgentStatus, string> = {
  starting: 'Starting',
  working: 'Working',
  waiting: 'Waiting for input',
  exited: 'Agent exited',
  unknown: 'Live',
};

/** What the agent is doing, at a glance. */
export function StatusPill({ status, agentName, terminal = false }: { status: AgentStatus; agentName: string; terminal?: boolean }) {
  const className = status === 'working' ? 'working' : status === 'waiting' ? 'waiting' : status === 'exited' ? 'ended' : '';
  const label = status === 'working' ? `${agentName} is working` : terminal && status === 'exited' ? 'Shell exited' : LABELS[status];
  // A shell has no activity to share, so there is nothing to explain.
  const title = status === 'unknown' && !terminal ? `${agentName}'s activity is not shared` : undefined;
  return (
    <span className={`pill ${className}`} role="status" title={title}>
      <span className="dot" aria-hidden />
      {label}
    </span>
  );
}
