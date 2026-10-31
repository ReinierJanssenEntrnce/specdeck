import { describe, expect, it } from 'vitest';
import { decodeTerminalStream } from './stream';

describe('terminal stream', () => {
  it('decodes output and resizes in order', () => {
    // Same bytes as the Rust encoder's test.
    const buffer = new Uint8Array([1, 0, 0, 0, 2, 104, 105, 2, 0, 100, 0, 30]).buffer;
    const steps = decodeTerminalStream(buffer);
    expect(steps).toHaveLength(2);
    expect(new TextDecoder().decode((steps[0] as { output: Uint8Array }).output)).toBe('hi');
    expect(steps[1]).toEqual({ resize: [100, 30] });
  });

  it('stops at a truncated step instead of misreading it', () => {
    expect(decodeTerminalStream(new Uint8Array([1, 0, 0]).buffer)).toEqual([]);
  });
});
