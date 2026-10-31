// A fake bridge with a realistic session, for looking at the UI in a browser (`pnpm dev`, then
// open /?demo=start, /?demo=host or /?demo=viewer) and for component tests. Never used by the app.

import type { Bridge, TerminalSink } from './bridge';
import type { HostEvent, Participant, Profile, TimelineEvent, ViewEvent } from './types';

const t0 = Date.UTC(2026, 9, 31, 13, 2);

export const demoProfile: Profile = {
  username: 'Nick',
  recentFolders: [
    { path: '/Users/nick/projects/webshop', name: 'webshop' },
    { path: '/Users/nick/projects/specshare', name: 'specshare' },
    { path: '/Users/nick/work/billing-api', name: 'billing-api' },
  ],
  lastAgent: 'claude-code',
  notifications: true,
  view: null,
  translucent: false,
  agents: [
    { id: 'claude-code', name: 'Claude Code', installed: true, hasTimeline: true },
    { id: 'open-code', name: 'OpenCode', installed: true, hasTimeline: false },
    { id: 'omp', name: 'omp', installed: false, hasTimeline: false },
    { id: 'terminal', name: 'Terminal', installed: true, hasTimeline: false },
  ],
  hosting: false,
  platform: 'macos',
};

export const demoParticipants: Participant[] = [
  { id: 0, username: 'Nick', role: 'Host', label: 'Nick' },
  { id: 1, username: 'Anna de Vries', role: 'Viewer', label: 'Anna de Vries' },
  { id: 2, username: 'Bram', role: 'Viewer', label: 'Bram' },
  { id: 3, username: 'Anna de Vries', role: 'Viewer', label: 'Anna de Vries (2)' },
];

let seq = 0;
const at = (minutes: number) => ({ outputSeq: seq++, at: t0 + minutes * 60_000 });

export const demoTimeline: TimelineEvent[] = [
  { ...at(0), kind: 'session-started' },
  { ...at(0), kind: 'git', folder: 'webshop', branch: 'feature/SHOP-412_health-endpoint', dirty: true, ahead: 2, behind: 0 },
  {
    ...at(4),
    kind: 'status-line',
    model: 'Opus 5.5',
    contextWindow: 1_000_000,
    contextTokens: 178_200,
    effort: 'medium',
    fast: false,
    fiveHour: { usedPercent: 23, resetsAt: Math.floor(Date.now() / 1000) + 2 * 3600 + 14 * 60 },
    sevenDay: { usedPercent: 41, resetsAt: Math.floor(Date.now() / 1000) + 3 * 86_400 + 2 * 3600 },
  },
  { ...at(1), kind: 'prompt', promptId: 'p1', text: 'Add a /health endpoint to the API that reports the database status.' },
  { ...at(1), kind: 'message', promptId: 'p1', messageId: 'm1', text: "I'll look at how the routes are set up first.", final: true },
  { ...at(1), kind: 'tool-call', promptId: 'p1', toolUseId: 't1', toolName: 'Read', input: { file_path: 'src/routes/index.ts' } },
  { ...at(1), kind: 'tool-result', promptId: 'p1', toolUseId: 't1', status: 'succeeded' },
  { ...at(1), kind: 'file-read', promptId: 'p1', toolUseId: 't1', path: 'src/routes/index.ts' },
  { ...at(2), kind: 'tool-call', promptId: 'p1', toolUseId: 't2', toolName: 'Write', input: { file_path: 'src/routes/health.ts' } },
  { ...at(2), kind: 'tool-result', promptId: 'p1', toolUseId: 't2', status: 'succeeded' },
  {
    ...at(2),
    kind: 'file-change',
    promptId: 'p1',
    toolUseId: 't2',
    path: 'src/routes/health.ts',
    change: 'created',
    hunks: [
      {
        oldStart: 0,
        oldLines: 0,
        newStart: 1,
        newLines: 9,
        lines: [
          "+import { Router } from 'express';",
          "+import { db } from '../db';",
          '+',
          '+export const health = Router();',
          '+',
          "+health.get('/health', async (_req, res) => {",
          '+  const database = await db.ping().then(() => "up", () => "down");',
          '+  res.status(database === "up" ? 200 : 503).json({ status: "ok", database });',
          '+});',
        ],
      },
    ],
  },
  { ...at(2), kind: 'tool-call', promptId: 'p1', toolUseId: 't3', toolName: 'Edit', input: { file_path: 'src/routes/index.ts' } },
  { ...at(2), kind: 'tool-result', promptId: 'p1', toolUseId: 't3', status: 'succeeded' },
  {
    ...at(2),
    kind: 'file-change',
    promptId: 'p1',
    toolUseId: 't3',
    path: 'src/routes/index.ts',
    change: 'edited',
    hunks: [
      {
        oldStart: 3,
        oldLines: 4,
        newStart: 3,
        newLines: 6,
        lines: [" import { orders } from './orders';", "+import { health } from './health';", ' ', ' export const routes = Router();', "+routes.use(health);", " routes.use('/orders', orders);"],
      },
    ],
  },
  { ...at(3), kind: 'tool-call', promptId: 'p1', toolUseId: 't4', toolName: 'Bash', input: { command: 'npm test -- health' } },
  { ...at(3), kind: 'tool-result', promptId: 'p1', toolUseId: 't4', status: 'failed', error: 'Exit code 1: no test files match "health"' },
  {
    ...at(3),
    kind: 'response',
    promptId: 'p1',
    text: 'Added **`GET /health`** in `src/routes/health.ts` and registered it in the router.\n\n- Returns `200` with `{ status: "ok", database: "up" }` when the database answers\n- Returns `503` when it does not\n\nThere are no tests for it yet; want me to add one?',
  },
  { ...at(3), kind: 'usage', tokens: { input: 14, output: 2310, cacheRead: 96400, cacheWrite: 21800 }, contextTokens: 24180, contextWindow: 200_000 },
  { ...at(4), kind: 'prompt', promptId: 'p2', text: 'Yes, add a test with a stubbed database.' },
  { ...at(4), kind: 'message', promptId: 'p2', messageId: 'm2', text: 'Writing a test that stubs `db.ping` for both cases.', final: false },
  { ...at(4), kind: 'tool-call', promptId: 'p2', toolUseId: 't5', toolName: 'Bash', input: { command: 'npm test -- health' } },
  { ...at(4), kind: 'usage', tokens: { input: 19, output: 2890, cacheRead: 148200, cacheWrite: 26500 }, contextTokens: 151320, contextWindow: 200_000 },
];

const encoder = new TextEncoder();
const esc = '\x1b[';
const terminalText = [
  `${esc}38;5;245m╭──────────────────────────────────────────────────╮${esc}0m`,
  `${esc}38;5;245m│${esc}0m ${esc}38;2;217;119;87m✻${esc}0m Welcome to ${esc}1mClaude Code${esc}0m!                       ${esc}38;5;245m│${esc}0m`,
  `${esc}38;5;245m│${esc}0m   ${esc}2mcwd: /Users/nick/projects/webshop${esc}0m              ${esc}38;5;245m│${esc}0m`,
  `${esc}38;5;245m╰──────────────────────────────────────────────────╯${esc}0m`,
  '',
  `${esc}2m>${esc}0m Add a /health endpoint to the API that reports the database status.`,
  '',
  `${esc}38;2;217;119;87m●${esc}0m I'll look at how the routes are set up first.`,
  '',
  `${esc}32m●${esc}0m ${esc}1mRead${esc}0m(src/routes/index.ts)`,
  `  ⎿  Read 24 lines`,
  `${esc}32m●${esc}0m ${esc}1mWrite${esc}0m(src/routes/health.ts)`,
  `  ⎿  Wrote 9 lines to src/routes/health.ts`,
  `${esc}32m●${esc}0m ${esc}1mUpdate${esc}0m(src/routes/index.ts)`,
  `  ⎿  Updated src/routes/index.ts with 2 additions`,
  `${esc}31m●${esc}0m ${esc}1mBash${esc}0m(npm test -- health)`,
  `  ⎿  ${esc}31mError: no test files match "health"${esc}0m`,
  '',
  `${esc}38;2;217;119;87m●${esc}0m Added ${esc}1mGET /health${esc}0m in src/routes/health.ts and registered it.`,
  '  There are no tests for it yet; want me to add one?',
  '',
  `${esc}2m>${esc}0m Yes, add a test with a stubbed database.`,
  '',
  `${esc}38;2;217;119;87m✻${esc}0m ${esc}38;2;217;119;87mWriting tests…${esc}0m ${esc}2m(12s · esc to interrupt)${esc}0m`,
].join('\r\n');

function terminalStream(): ArrayBuffer {
  const bytes = encoder.encode(terminalText);
  const out = new Uint8Array(5 + bytes.length);
  out[0] = 1;
  new DataView(out.buffer).setUint32(1, bytes.length);
  out.set(bytes, 5);
  return out.buffer;
}

export function demoBridge(screen: string): Bridge {
  const profile = { ...demoProfile, hosting: screen === 'host' };
  return {
    getProfile: async () => profile,
    setUsername: async (username) => ({ ...profile, username }),
    setNotifications: async (notifications) => ({ ...profile, notifications }),
    setView: async (view) => ({ ...profile, view }),
    removeRecentFolder: async (path) => ({ ...profile, recentFolders: profile.recentFolders.filter((f) => f.path !== path) }),
    chooseFolder: async () => '/Users/nick/projects/webshop',
    startHosting: async () => {},
    hostAttach: async (terminal: TerminalSink, events: (event: HostEvent) => void) => {
      events({
        type: 'snapshot',
        agent: 'claude-code',
        folder: '/Users/nick/projects/webshop',
        timelineSupported: true,
        sharing: 'ready',
        code: '7K2M9QXH4TPA',
        participants: demoParticipants,
        status: 'working',
        viewerLimit: 20,
        exited: null,
        timeline: demoTimeline,
      });
      setTimeout(() => terminal(terminalStream()), 50);
    },
    hostInput: async () => {},
    hostResize: async () => {},
    hostSetColors: async () => {},
    hostNewCode: async () => {},
    hostRemoveViewer: async () => {},
    hostSetViewerLimit: async () => {},
    hostClose: async () => {},
    joinSession: async (_code, terminal: TerminalSink, events: (event: ViewEvent) => void) => {
      setTimeout(() => {
        events({ type: 'joined', viewerId: 2, agent: 'claude-code', timelineSupported: true, cols: 100, rows: 32 });
        events({ type: 'participants', participants: demoParticipants });
        events({ type: 'status', status: 'working' });
        for (const event of demoTimeline) events({ type: 'timeline', event });
        terminal(terminalStream());
      }, 300);
    },
    leaveSession: async () => {},
    takeJoinLink: async () => null,
    copyText: async () => {},
    onMenu: () => () => {},
  };
}
