// Commands from the native menu (app/src-tauri/src/menu.rs). The app forwards each one as a
// window event, so the screen or panel it concerns can handle it where its state lives.

import { useEffect, useRef } from 'react';

export interface MenuCommand {
  id: string;
  /** The folder, for `recent`. */
  path?: string;
}

const EVENT = 'specdeck:menu';

export function dispatchMenu(command: MenuCommand) {
  window.dispatchEvent(new CustomEvent<MenuCommand>(EVENT, { detail: command }));
}

/** Calls `handler` for every menu command while the component is mounted. */
export function useMenu(handler: (command: MenuCommand) => void) {
  const current = useRef(handler);
  useEffect(() => {
    current.current = handler;
  });
  useEffect(() => {
    const listener = (event: Event) => current.current((event as CustomEvent<MenuCommand>).detail);
    window.addEventListener(EVENT, listener);
    return () => window.removeEventListener(EVENT, listener);
  }, []);
}

export const isMac = typeof navigator !== 'undefined' && /Mac/.test(navigator.platform);
