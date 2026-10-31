// Measures what replaying a recorded session costs a late joiner (design decision 4, task 2.4):
// raw and zstd-compressed size, and how long xterm.js takes to process the whole log.
//
// Record a session with the macOS/Linux `script` command, without resizing the terminal:
//   script -q ~/specdeck-recordings/claude-1.raw claude
// then measure it:
//   node scripts/measure-replay.mjs ~/specdeck-recordings/claude-1.raw [cols] [rows]

import { readFileSync, statSync } from 'node:fs';
import { performance } from 'node:perf_hooks';
import { constants, zstdCompressSync } from 'node:zlib';
import xterm from '@xterm/headless';

const [file, cols = '120', rows = '40'] = process.argv.slice(2);
if (!file) {
  console.error('usage: node scripts/measure-replay.mjs <recording> [cols] [rows]');
  process.exit(1);
}

const raw = readFileSync(file);
const compressStart = performance.now();
const compressed = zstdCompressSync(raw, { params: { [constants.ZSTD_c_compressionLevel]: 3 } });
const compressMs = performance.now() - compressStart;

// Same batching as the output log: 16 ms batches, at most 64 KB each. Without timing data the
// recording is split into 64 KB chunks, which is the worst case for the number of writes.
const terminal = new xterm.Terminal({ cols: Number(cols), rows: Number(rows), scrollback: 1_000_000, allowProposedApi: true });
const replayStart = performance.now();
for (let offset = 0; offset < raw.length; offset += 64 * 1024) {
  await new Promise((resolve) => terminal.write(raw.subarray(offset, offset + 64 * 1024), resolve));
}
const replayMs = performance.now() - replayStart;

const mb = (bytes) => (bytes / 1024 / 1024).toFixed(2);
console.log(`recording:      ${file} (modified ${statSync(file).mtime.toISOString()})`);
console.log(`raw size:       ${mb(raw.length)} MB`);
console.log(`zstd size:      ${mb(compressed.length)} MB (level 3, ${(raw.length / compressed.length).toFixed(1)}x, ${compressMs.toFixed(0)} ms)`);
console.log(`xterm replay:   ${replayMs.toFixed(0)} ms at ${cols}x${rows}, ${terminal.buffer.active.length} lines of scrollback`);
// Transfer time on a 20 Mbit/s connection, a modest broadband upload for the host.
const transferMs = (compressed.length * 8) / (20 * 1000 * 1000) * 1000;
console.log(`catch-up:       ~${((transferMs + replayMs) / 1000).toFixed(1)} s (transfer at 20 Mbit/s + replay; target 5 s)`);
terminal.dispose();
