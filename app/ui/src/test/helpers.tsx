// Rendering screens with a fake bridge and a stub terminal (xterm.js needs a real canvas).

import { render } from '@testing-library/react';
import { vi } from 'vitest';
import type { ReactElement } from 'react';
import { BridgeContext, type Bridge } from '../api/bridge';
import { demoBridge } from '../api/demo';

/** A demo bridge whose methods are spies, with any overrides. */
export function fakeBridge(screen = 'start', overrides: Partial<Bridge> = {}): Bridge {
  const base = demoBridge(screen);
  const spied = Object.fromEntries(Object.entries(base).map(([key, fn]) => [key, vi.fn(fn as (...args: unknown[]) => unknown)])) as unknown as Bridge;
  return { ...spied, ...overrides };
}

export function renderWith(bridge: Bridge, ui: ReactElement) {
  return render(<BridgeContext.Provider value={bridge}>{ui}</BridgeContext.Provider>);
}
