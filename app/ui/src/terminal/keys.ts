// Keys the host's terminal sends differently from xterm.js.

type Key = Pick<KeyboardEvent, 'key' | 'shiftKey' | 'ctrlKey' | 'altKey' | 'metaKey' | 'isComposing'>;

/**
 * Shift+Enter, which starts a new line in an agent's prompt. xterm sends `\r` for it, as for
 * Enter, so the agent would send the prompt instead.
 */
export function isNewlineKey(event: Key): boolean {
  return event.key === 'Enter' && event.shiftKey && !event.ctrlKey && !event.altKey && !event.metaKey && !event.isComposing;
}
