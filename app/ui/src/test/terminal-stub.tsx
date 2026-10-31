// Stands in for TerminalView in tests: records what the screens ask of the terminal and lets a
// test play the terminal's side (typing, scrolling, search results).

import { forwardRef, useImperativeHandle, useEffect } from 'react';
import { vi } from 'vitest';
import type { TerminalHandle } from '../terminal/TerminalView';

export const terminal = {
  handle: {
    write: vi.fn(),
    reset: vi.fn(),
    findNext: vi.fn(),
    findPrevious: vi.fn(),
    clearSearch: vi.fn(),
    scrollToSeq: vi.fn(),
    scrollToBottom: vi.fn(),
    focus: vi.fn(),
  } satisfies TerminalHandle,
  props: {} as Record<string, ((...args: never[]) => void) | string | undefined>,
};

export const TerminalViewStub = forwardRef<TerminalHandle, Record<string, unknown>>(function TerminalViewStub(props, ref) {
  useImperativeHandle(ref, () => terminal.handle);
  useEffect(() => {
    terminal.props = props as typeof terminal.props;
  });
  return <div data-testid="terminal" data-mode={props.mode as string} />;
});
