// Types shared with the Rust side (app/src-tauri). Names follow its serde output.

export type AgentId = 'claude-code' | 'open-code' | 'omp';
export type AgentStatus = 'starting' | 'working' | 'waiting' | 'exited' | 'unknown';

export interface AgentInfo {
  id: AgentId;
  name: string;
  installed: boolean;
  hasTimeline: boolean;
}

export interface RecentFolder {
  path: string;
  name: string;
}

export interface Profile {
  username: string | null;
  recentFolders: RecentFolder[];
  lastAgent: AgentId | null;
  agents: AgentInfo[];
  hosting: boolean;
  /** Whether to show OS notifications while the window is not in front. */
  notifications: boolean;
  /** The window has the platform's material behind it (macOS vibrancy, Windows Mica). */
  translucent: boolean;
  platform: string;
}

export interface Participant {
  id: number;
  username: string;
  role: 'Host' | 'Viewer';
  label: string;
}

export interface DiffHunk {
  oldStart: number;
  oldLines: number;
  newStart: number;
  newLines: number;
  lines: string[];
}

type Stamp = { outputSeq: number; at: number };

/** Use of one of the host account's rate-limit windows. */
export interface RateLimit {
  usedPercent: number;
  /** Seconds since the Unix epoch. */
  resetsAt?: number;
}

/** Tokens used in a session, summed over every model call, subagents included. */
export interface TokenCounts {
  input: number;
  output: number;
  cacheRead: number;
  cacheWrite: number;
}

export type TimelineEvent = Stamp &
  (
    | { kind: 'session-started' }
    | { kind: 'session-ended' }
    | { kind: 'prompt'; promptId: string; text: string }
    | { kind: 'message'; promptId: string; messageId: string; text: string; final: boolean }
    | { kind: 'response'; promptId: string; text: string }
    | { kind: 'tool-call'; promptId: string; toolUseId: string; toolName: string; input: Record<string, unknown> }
    | { kind: 'tool-result'; promptId: string; toolUseId: string; status: 'succeeded' | 'failed' | 'not-run'; error?: string }
    | { kind: 'file-change'; promptId: string; toolUseId: string; path: string; change: 'created' | 'edited'; hunks: DiffHunk[] }
    | { kind: 'file-read'; promptId: string; toolUseId: string; path: string }
    | { kind: 'attention'; message: string }
    | { kind: 'timeline-unavailable'; reason: string }
    | { kind: 'timeline-available' }
    | { kind: 'usage'; tokens: TokenCounts; contextTokens?: number; contextWindow?: number }
    | { kind: 'usage-unavailable'; reason: string }
    | {
        kind: 'status-line';
        model?: string;
        contextWindow?: number;
        contextTokens?: number;
        effort?: string;
        fast: boolean;
        fiveHour?: RateLimit;
        sevenDay?: RateLimit;
      }
    | { kind: 'git'; folder: string; branch?: string; dirty: boolean; ahead: number; behind: number; worktree?: string }
  );

export type SharingState = 'connecting' | 'ready' | 'unavailable';

export type HostEvent =
  | {
      type: 'snapshot';
      agent: AgentId;
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
  | { type: 'sharing'; state: SharingState; code: string | null }
  | { type: 'participants'; participants: Participant[] }
  | { type: 'status'; status: AgentStatus }
  | { type: 'timeline'; event: TimelineEvent }
  | { type: 'exited'; code: number };

export type EndReason =
  | 'closed'
  | 'removed'
  | 'full'
  | 'invalid-code'
  | 'too-many-attempts'
  | 'service-unavailable'
  | 'network-blocked'
  | 'update-app'
  | 'host-update';

export type ViewEvent =
  | { type: 'joined'; viewerId: number; agent: AgentId; timelineSupported: boolean; cols: number; rows: number }
  | { type: 'reconnecting' }
  | { type: 'reconnected' }
  | { type: 'timeline'; event: TimelineEvent }
  | { type: 'participants'; participants: Participant[] }
  | { type: 'status'; status: AgentStatus }
  | { type: 'ended'; reason: EndReason };
