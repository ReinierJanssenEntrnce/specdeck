import type { AgentStatus } from '../api/types';

const LABELS: Record<AgentStatus, string> = {
  starting: 'Starting',
  working: 'Working',
  waiting: 'Waiting for input',
  exited: 'Agent exited',
  unknown: 'Live',
};

/** What the agent is doing, at a glance. */
export function StatusPill({ status, agentName }: { status: AgentStatus; agentName: string }) {
  const className = status === 'working' ? 'working' : status === 'waiting' ? 'waiting' : status === 'exited' ? 'ended' : '';
  const label = status === 'working' ? `${agentName} is working` : LABELS[status];
  return (
    <span className={`pill ${className}`} role="status" title={status === 'unknown' ? `${agentName}'s activity is not shared` : undefined}>
      <span className="dot" aria-hidden />
      {label}
    </span>
  );
}
