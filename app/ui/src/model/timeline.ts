// Turns the flat list of timeline events into what the timeline panel shows: one turn per
// prompt, with the agent's text and tool calls in order, and the files it touched.

import type { DiffHunk, TimelineEvent, TokenCounts } from '../api/types';

export type ToolStatus = 'running' | 'succeeded' | 'failed' | 'not-run';

export interface FileChange {
  path: string;
  change: 'created' | 'edited';
  hunks: DiffHunk[];
  promptId: string;
  toolUseId: string;
  at: number;
  outputSeq: number;
}

export interface TextItem {
  kind: 'text';
  id: string;
  text: string;
  outputSeq: number;
}

export interface ToolItem {
  kind: 'tool';
  id: string;
  toolName: string;
  input: Record<string, unknown>;
  status: ToolStatus;
  error?: string;
  changes: FileChange[];
  outputSeq: number;
}

export interface Turn {
  promptId: string;
  prompt: string;
  at: number;
  outputSeq: number;
  items: (TextItem | ToolItem)[];
  /** The final response, once the turn is over. */
  response: string | null;
  /** Tokens the agent used for this prompt so far, once usage is known. */
  tokens: number | null;
}

/** The session's token usage, as the agent last reported it. */
export type Usage =
  | { available: true; tokens: TokenCounts; total: number; contextTokens?: number; contextWindow?: number }
  | { available: false; reason: string };

export interface TouchedFile {
  path: string;
  /** Read but never changed. */
  readOnly: boolean;
  changes: FileChange[];
}

export interface TimelineModel {
  turns: Turn[];
  files: TouchedFile[];
  /** Why the timeline is unavailable right now, or null. */
  unavailable: string | null;
  /** Null until the agent reports usage, and for agents without a timeline. */
  usage: Usage | null;
}

export function totalTokens(tokens: TokenCounts): number {
  return tokens.input + tokens.output + tokens.cacheRead + tokens.cacheWrite;
}

export function buildTimeline(events: TimelineEvent[]): TimelineModel {
  const turns: Turn[] = [];
  const byPrompt = new Map<string, Turn>();
  const tools = new Map<string, ToolItem>();
  const texts = new Map<string, TextItem>();
  const files = new Map<string, TouchedFile>();
  let unavailable: string | null = null;
  let usage: Usage | null = null;
  // The session total when each turn started; a turn's tokens are what was added since.
  let sessionTotal = 0;
  const startTotals = new Map<Turn, number>();

  const turnFor = (promptId: string, at: number, outputSeq: number): Turn => {
    let turn = byPrompt.get(promptId);
    if (!turn) {
      // Events for a prompt that came before this viewer's history; show them under a blank prompt.
      turn = { promptId, prompt: '', at, outputSeq, items: [], response: null, tokens: null };
      startTotals.set(turn, sessionTotal);
      byPrompt.set(promptId, turn);
      turns.push(turn);
    }
    return turn;
  };
  const touched = (path: string): TouchedFile => {
    let file = files.get(path);
    if (!file) {
      file = { path, readOnly: true, changes: [] };
      files.set(path, file);
    }
    return file;
  };

  for (const event of events) {
    switch (event.kind) {
      case 'prompt': {
        const turn = turnFor(event.promptId, event.at, event.outputSeq);
        turn.prompt = event.text;
        turn.at = event.at;
        turn.outputSeq = event.outputSeq;
        break;
      }
      case 'message': {
        // Live text arrives in pieces per message.
        const existing = texts.get(event.messageId);
        if (existing) {
          existing.text += event.text;
        } else {
          const item: TextItem = { kind: 'text', id: event.messageId, text: event.text, outputSeq: event.outputSeq };
          texts.set(event.messageId, item);
          turnFor(event.promptId, event.at, event.outputSeq).items.push(item);
        }
        break;
      }
      case 'response':
        turnFor(event.promptId, event.at, event.outputSeq).response = event.text;
        break;
      case 'tool-call': {
        const item: ToolItem = {
          kind: 'tool',
          id: event.toolUseId,
          toolName: event.toolName,
          input: event.input,
          status: 'running',
          changes: [],
          outputSeq: event.outputSeq,
        };
        tools.set(event.toolUseId, item);
        turnFor(event.promptId, event.at, event.outputSeq).items.push(item);
        break;
      }
      case 'tool-result': {
        const item = tools.get(event.toolUseId);
        if (item) {
          item.status = event.status;
          if (event.error) item.error = event.error;
        }
        break;
      }
      case 'file-change': {
        const change: FileChange = {
          path: event.path,
          change: event.change,
          hunks: event.hunks,
          promptId: event.promptId,
          toolUseId: event.toolUseId,
          at: event.at,
          outputSeq: event.outputSeq,
        };
        tools.get(event.toolUseId)?.changes.push(change);
        const file = touched(event.path);
        file.readOnly = false;
        file.changes.push(change);
        break;
      }
      case 'file-read':
        touched(event.path);
        break;
      case 'timeline-unavailable':
        unavailable = event.reason;
        break;
      case 'timeline-available':
        unavailable = null;
        break;
      case 'usage': {
        const total = totalTokens(event.tokens);
        usage = { available: true, tokens: event.tokens, total, contextTokens: event.contextTokens, contextWindow: event.contextWindow };
        sessionTotal = total;
        const turn = turns.at(-1);
        if (turn) turn.tokens = total - (startTotals.get(turn) ?? total);
        break;
      }
      case 'usage-unavailable':
        usage = { available: false, reason: event.reason };
        break;
      case 'session-started':
      case 'session-ended':
        break;
    }
  }

  // Changed files first, then files that were only read; each group by name.
  const sorted = [...files.values()].sort(
    (a, b) => Number(a.readOnly) - Number(b.readOnly) || a.path.localeCompare(b.path),
  );
  return { turns, files: sorted, unavailable, usage };
}

/** The usage the latest usage entry reports, without building the whole timeline. */
export function latestUsage(events: TimelineEvent[]): Usage | null {
  for (let i = events.length - 1; i >= 0; i--) {
    const event = events[i];
    if (event?.kind === 'usage') {
      return { available: true, tokens: event.tokens, total: totalTokens(event.tokens), contextTokens: event.contextTokens, contextWindow: event.contextWindow };
    }
    if (event?.kind === 'usage-unavailable') return { available: false, reason: event.reason };
  }
  return null;
}

/** A short line describing what a tool call does, for its timeline entry. */
export function toolSummary(item: ToolItem): string {
  const input = item.input;
  const text = (key: string) => (typeof input[key] === 'string' ? (input[key] as string) : null);
  switch (item.toolName) {
    case 'Bash':
      return text('command') ?? 'Run a command';
    case 'Read':
    case 'Write':
    case 'Edit':
    case 'MultiEdit':
    case 'NotebookEdit':
      return text('file_path') ?? item.toolName;
    case 'Grep':
    case 'Glob':
      return text('pattern') ?? item.toolName;
    case 'WebFetch':
      return text('url') ?? 'Fetch a page';
    case 'WebSearch':
      return text('query') ?? 'Search the web';
    case 'Task':
      return text('description') ?? 'Start a subagent';
    default: {
      const first = Object.values(input).find((value) => typeof value === 'string');
      return typeof first === 'string' ? first : item.toolName;
    }
  }
}

/** The text a search looks through for a turn. */
export function turnText(turn: Turn): string {
  const parts = [turn.prompt, turn.response ?? ''];
  for (const item of turn.items) {
    parts.push(item.kind === 'text' ? item.text : `${item.toolName} ${toolSummary(item)}`);
  }
  return parts.join('\n');
}
