import { describe, expect, it } from 'vitest';
import type { TimelineEvent } from '../api/types';
import { formatTokens, statusBarInfo, timeLeft } from './statusbar';

let at = 1;
const stamp = () => ({ outputSeq: 0, at: at++ });

describe('status bar info', () => {
  it('takes the latest of each kind', () => {
    const events: TimelineEvent[] = [
      { ...stamp(), kind: 'git', folder: 'webshop', branch: 'main', dirty: false, ahead: 0, behind: 0 },
      { ...stamp(), kind: 'usage', tokens: { input: 10, output: 5, cacheRead: 1000, cacheWrite: 100 }, contextTokens: 900, contextWindow: 200_000 },
      {
        ...stamp(),
        kind: 'status-line',
        model: 'Opus 5.5',
        contextWindow: 1_000_000,
        contextTokens: 1110,
        effort: 'medium',
        fast: false,
        fiveHour: { usedPercent: 23, resetsAt: 100 },
      },
      { ...stamp(), kind: 'git', folder: 'webshop', branch: 'feature/PROJ-1', dirty: true, ahead: 2, behind: 0 },
    ];
    expect(statusBarInfo(events)).toEqual({
      folder: 'webshop',
      git: { branch: 'feature/PROJ-1', dirty: true, ahead: 2, behind: 0, worktree: undefined },
      model: 'Opus 5.5',
      effort: 'medium',
      fast: false,
      contextTokens: 1110,
      contextWindow: 1_000_000,
      total: 1115,
      usageUnavailable: undefined,
      fiveHour: { usedPercent: 23, resetsAt: 100 },
      sevenDay: undefined,
    });
  });

  it("prefers Claude Code's own context figures to the transcript's", () => {
    const info = statusBarInfo([
      { ...stamp(), kind: 'status-line', fast: false, contextTokens: 151_000, contextWindow: 1_000_000 },
      { ...stamp(), kind: 'usage', tokens: { input: 1, output: 1, cacheRead: 0, cacheWrite: 0 }, contextTokens: 152_000, contextWindow: 200_000 },
    ]);
    expect(info).toMatchObject({ contextTokens: 151_000, contextWindow: 1_000_000, total: 2 });
    const transcriptOnly = statusBarInfo([
      { ...stamp(), kind: 'usage', tokens: { input: 1, output: 1, cacheRead: 0, cacheWrite: 0 }, contextTokens: 152_000, contextWindow: 200_000 },
    ]);
    expect(transcriptOnly).toMatchObject({ contextTokens: 152_000, contextWindow: 200_000 });
  });

  it('has no git state outside a repository, and keeps the folder', () => {
    const info = statusBarInfo([{ ...stamp(), kind: 'git', folder: 'notes', dirty: false, ahead: 0, behind: 0 }]);
    expect(info.folder).toBe('notes');
    expect(info.git).toBeUndefined();
  });

  it('remembers why usage is unavailable until it is back', () => {
    const unavailable: TimelineEvent = { ...stamp(), kind: 'usage-unavailable', reason: 'not found' };
    expect(statusBarInfo([unavailable]).usageUnavailable).toBe('not found');
    const back: TimelineEvent = { ...stamp(), kind: 'usage', tokens: { input: 1, output: 1, cacheRead: 0, cacheWrite: 0 } };
    expect(statusBarInfo([unavailable, back]).usageUnavailable).toBeUndefined();
  });

  it('formats tokens and time left compactly', () => {
    expect([950, 1_000, 12_345, 245_600, 1_000_000, 1_250_000].map(formatTokens)).toEqual(['950', '1k', '12.3k', '246k', '1M', '1.3M']);
    const now = 1_000_000_000;
    expect(timeLeft(now / 1000 + 3 * 86_400 + 2 * 3600 + 59, now)).toBe('3d2h');
    expect(timeLeft(now / 1000 + 2 * 3600 + 14 * 60 + 59, now)).toBe('2h14m');
    expect(timeLeft(now / 1000 + 3599, now)).toBe('59m');
    expect(timeLeft(now / 1000 + 20, now)).toBe('1m');
    expect(timeLeft(now / 1000 - 1, now)).toBe('');
  });
});
