// What the status bar at the bottom of a session shows, from the latest entries of each kind.

import type { RateLimit, TimelineEvent } from '../api/types';
import { totalTokens } from './timeline';

export interface GitState {
  branch: string;
  dirty: boolean;
  ahead: number;
  behind: number;
  worktree?: string;
}

export interface StatusBarInfo {
  folder?: string;
  /** Absent when the folder is not in a git repository. */
  git?: GitState;
  model?: string;
  effort?: string;
  fast: boolean;
  contextTokens?: number;
  contextWindow?: number;
  /** Tokens used in the session. */
  total?: number;
  /** Why usage cannot be read, when it cannot. */
  usageUnavailable?: string;
  fiveHour?: RateLimit;
  sevenDay?: RateLimit;
}

/** `agentFigures` is false for agents that report no usage: only the folder and git state count. */
export function statusBarInfo(events: TimelineEvent[], agentFigures = true): StatusBarInfo {
  const info: StatusBarInfo = { fast: false };
  // Claude Code's own figures for the context, once its status line reported them; the
  // transcript's are the fallback (its window size is a guess).
  let statusLineContext = false;
  for (const event of events) {
    if (!agentFigures && event.kind !== 'git') continue;
    switch (event.kind) {
      case 'git':
        info.folder = event.folder;
        info.git = event.branch === undefined ? undefined : { branch: event.branch, dirty: event.dirty, ahead: event.ahead, behind: event.behind, worktree: event.worktree };
        break;
      case 'status-line':
        info.model = event.model;
        info.effort = event.effort;
        info.fast = event.fast;
        info.fiveHour = event.fiveHour;
        info.sevenDay = event.sevenDay;
        if (event.contextTokens !== undefined) {
          statusLineContext = true;
          info.contextTokens = event.contextTokens;
          info.contextWindow = event.contextWindow;
        }
        break;
      case 'usage':
        info.total = totalTokens(event.tokens);
        info.usageUnavailable = undefined;
        if (!statusLineContext && event.contextTokens !== undefined) {
          info.contextTokens = event.contextTokens;
          info.contextWindow = event.contextWindow;
        }
        break;
      case 'usage-unavailable':
        info.usageUnavailable = event.reason;
        break;
      default:
        break;
    }
  }
  return info;
}

/** A token count in a few characters: 950, 12.3k, 245k, 1.2M. */
export function formatTokens(tokens: number): string {
  if (tokens < 1000) return String(tokens);
  if (tokens < 100_000) return `${(tokens / 1000).toFixed(1).replace(/\.0$/, '')}k`;
  if (tokens < 1_000_000) return `${Math.round(tokens / 1000)}k`;
  return `${(tokens / 1_000_000).toFixed(1).replace(/\.0$/, '')}M`;
}

/**
 * Time left until `resetsAt` (seconds since the epoch): 3d2h, 2h14m, 9m. Truncated, never rounded
 * up, so it reads as "at least this long"; under a minute shows 1m. Empty once it has passed.
 */
export function timeLeft(resetsAt: number, nowMs: number): string {
  const s = Math.floor(resetsAt - nowMs / 1000);
  if (s <= 0) return '';
  if (s >= 86_400) return `${Math.floor(s / 86_400)}d${Math.floor((s % 86_400) / 3600)}h`;
  if (s >= 3600) return `${Math.floor(s / 3600)}h${String(Math.floor((s % 3600) / 60)).padStart(2, '0')}m`;
  if (s >= 60) return `${Math.floor(s / 60)}m`;
  return '1m';
}
