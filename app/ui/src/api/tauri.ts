// The real bridge: Tauri commands from app/src-tauri/src/lib.rs.

import { Channel, invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { open } from '@tauri-apps/plugin-dialog';
import type { Bridge } from './bridge';
import type { MenuCommand } from '../menu/menu';
import type { HostEvent, JoinLink, Profile, ViewEvent } from './types';

function channel<T>(handler: (message: T) => void): Channel<T> {
  const ch = new Channel<T>();
  ch.onmessage = handler;
  return ch;
}

export const tauriBridge: Bridge = {
  getProfile: () => invoke<Profile>('get_profile'),
  setUsername: (username) => invoke<Profile>('set_username', { username }),
  setNotifications: (enabled) => invoke<Profile>('set_notifications', { enabled }),
  setView: (view) => invoke<Profile>('set_view', { view }),
  removeRecentFolder: (path) => invoke<Profile>('remove_recent_folder', { path }),
  chooseFolder: async () => {
    const folder = await open({ directory: true, multiple: false, title: 'Choose the folder the agent works in' });
    return typeof folder === 'string' ? folder : null;
  },
  startHosting: (agent, folder, cols, rows) => invoke('start_hosting', { agent, folder, cols, rows }),
  hostAttach: (terminal, events) =>
    invoke('host_attach', { terminal: channel<ArrayBuffer>(terminal), events: channel<HostEvent>(events) }),
  hostInput: (data) => invoke('host_input', { data }),
  hostResize: (cols, rows) => invoke('host_resize', { cols, rows }),
  hostSetColors: (foreground, background) => invoke('host_set_colors', { foreground, background }),
  hostNewCode: () => invoke('host_new_code'),
  hostRemoveViewer: (viewerId) => invoke('host_remove_viewer', { viewerId }),
  hostSetViewerLimit: (limit) => invoke('host_set_viewer_limit', { limit }),
  hostClose: () => invoke('host_close'),
  joinSession: (code, terminal, events) =>
    invoke('join_session', { code, terminal: channel<ArrayBuffer>(terminal), events: channel<ViewEvent>(events) }),
  leaveSession: () => invoke('leave_session'),
  takeJoinLink: () => invoke<JoinLink | null>('take_join_link'),
  copyText: (text) => navigator.clipboard.writeText(text),
  onMenu: (handler) => {
    const stop = listen<MenuCommand>('menu', (event) => handler(event.payload));
    return () => void stop.then((unlisten) => unlisten());
  },
};
