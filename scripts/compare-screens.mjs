// Replays two terminal record streams in xterm.js and reports whether they end on the same screen
// (content, colours and attributes, cursor, active buffer). Used by the mirror-exactness test
// (task 5.5): one stream as the host's terminal received it, one as a viewer did.
//
//   node scripts/compare-screens.mjs <cols> <rows> <host.json> <viewer.json>
//
// Each file is a JSON array of steps: {"data": "<base64>"} or {"resize": [cols, rows]}.

import { readFileSync } from 'node:fs';
import xterm from '@xterm/headless';
import serialize from '@xterm/addon-serialize';

const [cols, rows, ...files] = process.argv.slice(2);

async function replay(file) {
  const terminal = new xterm.Terminal({ cols: Number(cols), rows: Number(rows), scrollback: 100_000, allowProposedApi: true });
  const serializer = new serialize.SerializeAddon();
  terminal.loadAddon(serializer);
  for (const step of JSON.parse(readFileSync(file, 'utf8'))) {
    if (step.resize) terminal.resize(step.resize[0], step.resize[1]);
    else await new Promise((resolve) => terminal.write(Buffer.from(step.data, 'base64'), resolve));
  }
  const buffer = terminal.buffer.active;
  const screen = {
    serialized: serializer.serialize(),
    cursor: [buffer.cursorX, buffer.cursorY],
    buffer: buffer.type,
    size: [terminal.cols, terminal.rows],
  };
  terminal.dispose();
  return screen;
}

const [host, viewer] = await Promise.all(files.map(replay));
const differences = Object.keys(host).filter((key) => JSON.stringify(host[key]) !== JSON.stringify(viewer[key]));
console.log(JSON.stringify({ same: differences.length === 0, differences, size: host.size, cursor: host.cursor, buffer: host.buffer, serializedBytes: host.serialized.length }));
process.exit(differences.length === 0 ? 0 : 1);
