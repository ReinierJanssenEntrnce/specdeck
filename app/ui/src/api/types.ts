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
    | { kind: 'timeline-unavailable'; reason: string }
    | { kind: 'timeline-available' }
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
