import type { AgentStatus, TimelineEvent } from '../api/types';

/**
 * The request the agent waits on, after a timeline event or status change: the message of the
 * latest notice, until the agent stops waiting. Sessions without a timeline show it with the
 * status, since they have nowhere else to show it.
 */
export function nextRequest(request: string | null, event: { type: 'timeline'; event: TimelineEvent } | { type: 'status'; status: AgentStatus }): string | null {
  if (event.type === 'status') return event.status === 'waiting' ? request : null;
  return event.event.kind === 'attention' ? event.event.message : request;
}

/** The request in a whole timeline, as replayed when the host's window opens again. */
export function requestIn(timeline: TimelineEvent[], status: AgentStatus): string | null {
  if (status !== 'waiting') return null;
  const notice = timeline.findLast((event) => event.kind === 'attention');
  return notice?.kind === 'attention' ? notice.message : null;
}
