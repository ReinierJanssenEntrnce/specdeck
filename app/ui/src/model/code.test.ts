import { describe, expect, it } from 'vitest';
import { codeCharacters, formatCode, isCompleteCode, joinLink } from './code';

describe('invite codes', () => {
  it('formats in groups of four as the user types', () => {
    expect(formatCode('7k2m')).toBe('7K2M');
    expect(formatCode('7k2m9')).toBe('7K2M-9');
    expect(formatCode('7k2m9qxh4tpa')).toBe('7K2M-9QXH-4TPA');
  });

  it('ignores dashes, spaces and characters that cannot be in a code', () => {
    expect(formatCode(' 7K2M - 9QXH - 4TPA!! ')).toBe('7K2M-9QXH-4TPA');
    expect(codeCharacters('u-u-u')).toBe('');
  });

  it('reads look-alike letters as digits', () => {
    expect(codeCharacters('OIL0')).toBe('0110');
  });

  it('takes the code from a pasted join link', () => {
    expect(formatCode('specdeck://join/7K2M-9QXH-4TPA')).toBe('7K2M-9QXH-4TPA');
    expect(isCompleteCode('Join me: specdeck://join/7k2m9qxh4tpa')).toBe(true);
  });

  it('knows when a code is complete', () => {
    expect(isCompleteCode('7K2M-9QXH-4TP')).toBe(false);
    expect(isCompleteCode('7K2M-9QXH-4TPA')).toBe(true);
  });

  it('makes join links', () => {
    expect(joinLink('7k2m9qxh4tpa')).toBe('specdeck://join/7K2M-9QXH-4TPA');
  });
});
