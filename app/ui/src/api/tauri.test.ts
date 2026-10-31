import { describe, expect, it } from 'vitest';
import { asArrayBuffer } from './tauri';

describe('binary channel messages', () => {
  const bytes = [1, 0, 0, 0, 2, 104, 105];

  it('passes an ArrayBuffer through', () => {
    const buffer = new Uint8Array(bytes).buffer;
    expect(asArrayBuffer(buffer)).toBe(buffer);
  });

  // When the IPC protocol is unavailable, Tauri falls back to postMessage and sends raw bytes as a
  // JSON array of numbers.
  it('turns the number array of the postMessage fallback into an ArrayBuffer', () => {
    expect([...new Uint8Array(asArrayBuffer(bytes))]).toEqual(bytes);
  });

  it('takes the viewed bytes of a typed array', () => {
    const view = new Uint8Array([9, ...bytes, 9]).subarray(1, bytes.length + 1);
    expect([...new Uint8Array(asArrayBuffer(view))]).toEqual(bytes);
  });
});
