import { describe, expect, it } from 'vitest';
import { isNewlineKey } from './keys';

const key = (modifiers: Partial<KeyboardEvent> = {}) => ({
  key: 'Enter',
  shiftKey: false,
  ctrlKey: false,
  altKey: false,
  metaKey: false,
  isComposing: false,
  ...modifiers,
});

describe('newline key', () => {
  it('is Shift+Enter', () => {
    expect(isNewlineKey(key({ shiftKey: true }))).toBe(true);
  });

  it('is not Enter alone or with another modifier', () => {
    expect(isNewlineKey(key())).toBe(false);
    expect(isNewlineKey(key({ altKey: true }))).toBe(false);
    expect(isNewlineKey(key({ ctrlKey: true }))).toBe(false);
    expect(isNewlineKey(key({ ctrlKey: true, shiftKey: true }))).toBe(false);
    expect(isNewlineKey(key({ metaKey: true, shiftKey: true }))).toBe(false);
  });

  it('leaves Shift+Enter to an input method while it composes', () => {
    expect(isNewlineKey(key({ shiftKey: true, isComposing: true }))).toBe(false);
  });

  it('is not another key with Shift', () => {
    expect(isNewlineKey(key({ key: 'a', shiftKey: true }))).toBe(false);
  });
});
