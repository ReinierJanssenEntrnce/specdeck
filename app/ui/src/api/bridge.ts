// Everything the UI asks of the Rust side. Components get it from BridgeContext, so tests can
// pass a fake.

import type { MenuCommand } from '../menu/menu';
import { createContext, useContext } from 'react';
import type { AgentId, HostEvent, JoinLink, Profile, ViewEvent } from './types';

/** Receives terminal data as the binary stream described in src/terminal/stream.ts. */
export type TerminalSink = (data: ArrayBuffer) => void;

export interface Bridge {
  getProfile(): Promise<Profile>;
  setUsername(username: string): Promise<Profile>;
  setNotifications(enabled: boolean): Promise<Profile>;
  removeRecentFolder(path: string): Promise<Profile>;
  /** Opens the system folder picker; null when cancelled. */
  chooseFolder(): Promise<string | null>;
  startHosting(agent: AgentId, folder: string, cols: number, rows: number): Promise<void>;
  hostAttach(terminal: TerminalSink, events: (event: HostEvent) => void): Promise<void>;
  hostInput(data: string): Promise<void>;
  hostResize(cols: number, rows: number): Promise<void>;
  hostSetColors(foreground: string, background: string): Promise<void>;
  hostNewCode(): Promise<void>;
  hostRemoveViewer(viewerId: number): Promise<void>;
  hostSetViewerLimit(limit: number): Promise<void>;
  hostClose(): Promise<void>;
  joinSession(code: string, terminal: TerminalSink, events: (event: ViewEvent) => void): Promise<void>;
  leaveSession(): Promise<void>;
  /** The join link that opened the app or came in since, once; null when there is none. */
  takeJoinLink(): Promise<JoinLink | null>;
  copyText(text: string): Promise<void>;
  /** Calls `handler` for each native menu command; returns a function that stops listening. */
  onMenu(handler: (command: MenuCommand) => void): () => void;
}

export const BridgeContext = createContext<Bridge | null>(null);

export function useBridge(): Bridge {
  const bridge = useContext(BridgeContext);
  if (!bridge) throw new Error('BridgeContext is missing');
  return bridge;
}

/** Error text from a failed command, for showing to the user. */
export function errorMessage(error: unknown): string {
  if (typeof error === 'string') return error;
  if (error instanceof Error) return error.message;
  return 'Something went wrong.';
}
