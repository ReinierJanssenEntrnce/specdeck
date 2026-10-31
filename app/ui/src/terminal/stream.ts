// Decodes the terminal stream from app/src-tauri/src/terminal.rs: `1, len (u32 BE), bytes` for
// output and `2, cols (u16 BE), rows (u16 BE)` for a resize.

export type TerminalStep = { output: Uint8Array } | { resize: [number, number] };

export function decodeTerminalStream(buffer: ArrayBuffer): TerminalStep[] {
  const view = new DataView(buffer);
  const bytes = new Uint8Array(buffer);
  const steps: TerminalStep[] = [];
  let i = 0;
  while (i < bytes.length) {
    const kind = bytes[i];
    if (kind === 1 && i + 5 <= bytes.length) {
      const length = view.getUint32(i + 1);
      steps.push({ output: bytes.subarray(i + 5, i + 5 + length) });
      i += 5 + length;
    } else if (kind === 2 && i + 5 <= bytes.length) {
      steps.push({ resize: [view.getUint16(i + 1), view.getUint16(i + 3)] });
      i += 5;
    } else {
      break;
    }
  }
  return steps;
}
