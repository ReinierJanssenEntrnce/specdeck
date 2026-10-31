import { useEffect, useState } from 'react';
import { Folder, GitBranch } from 'lucide-react';
import type { RateLimit } from '../api/types';
import { formatTokens, timeLeft, type StatusBarInfo } from '../model/statusbar';

const COUNT = new Intl.NumberFormat();

/** Re-renders now and then, so times until a reset stay current. */
function useNow(intervalMs: number): number {
  const [now, setNow] = useState(() => Date.now());
  useEffect(() => {
    const timer = setInterval(() => setNow(Date.now()), intervalMs);
    return () => clearInterval(timer);
  }, [intervalMs]);
  return now;
}

/** Calm below half, amber past half, red past 80%, as in the host's own status line. */
function level(percent: number): string {
  return percent >= 80 ? 'hot' : percent >= 50 ? 'warm' : 'calm';
}

function Gauge({ percent, label }: { percent: number; label: string }) {
  const shown = Math.min(100, Math.max(0, percent));
  return (
    <span
      className={`gauge ${level(percent)}`}
      role="meter"
      aria-label={label}
      aria-valuemin={0}
      aria-valuemax={100}
      aria-valuenow={Math.round(shown)}
    >
      <span style={{ width: `${shown}%` }} />
    </span>
  );
}

function Limit({ name, title, limit, now }: { name: string; title: string; limit: RateLimit; now: number }) {
  const left = limit.resetsAt === undefined ? '' : timeLeft(limit.resetsAt, now);
  return (
    <span className="item" title={`${title}: ${limit.usedPercent}% used${left ? `, resets in ${left}` : ''}`}>
      <span className="label">{name}</span>
      <Gauge percent={limit.usedPercent} label={title} />
      <span className="value">{limit.usedPercent}%</span>
      {left && <span className="faint">{left}</span>}
    </span>
  );
}

/** The branch name with its issue key (PROJ-123) picked out, as the host's status line does. */
function Branch({ name }: { name: string }) {
  const key = /[A-Z][A-Z0-9]+-\d+/.exec(name);
  if (!key) return <span className="branch">{name}</span>;
  return (
    <span className="branch">
      {name.slice(0, key.index)}
      <span className="key">{key[0]}</span>
      {name.slice(key.index + key[0].length)}
    </span>
  );
}

interface Props {
  info: StatusBarInfo;
  /** The folder name to show until the session reports it. */
  folder?: string;
}

/** A quiet line at the bottom of the session: where the agent works and what it has used. */
export function StatusBar({ info, folder }: Props) {
  const now = useNow(30_000);
  const name = info.folder ?? folder;
  const { git, contextTokens, contextWindow } = info;
  const contextPercent = contextTokens !== undefined && contextWindow ? (contextTokens / contextWindow) * 100 : undefined;

  const left = [
    info.model && (
      <span className="item" key="model">
        <span className="strong">{info.model}</span>
        {contextWindow !== undefined && <span className="faint">{formatTokens(contextWindow)}</span>}
        {info.effort && <span className="faint">{info.effort}</span>}
        {info.fast && <span className="faint">fast</span>}
      </span>
    ),
    name && (
      <span className="item folder" key="folder">
        <Folder aria-hidden />
        {name}
      </span>
    ),
    git && (
      <span
        className="item"
        key="git"
        title={[
          `Branch ${git.branch}`,
          git.dirty ? 'uncommitted changes' : null,
          git.ahead ? `${git.ahead} ahead` : null,
          git.behind ? `${git.behind} behind` : null,
        ]
          .filter(Boolean)
          .join(', ')}
      >
        <GitBranch aria-hidden />
        <Branch name={git.branch} />
        {git.dirty && (
          <span className="dirty" aria-label="uncommitted changes">
            ●
          </span>
        )}
        {(git.ahead > 0 || git.behind > 0) && (
          <span className="faint">
            {git.ahead > 0 && `↑${git.ahead}`}
            {git.ahead > 0 && git.behind > 0 && ' '}
            {git.behind > 0 && `↓${git.behind}`}
          </span>
        )}
      </span>
    ),
    git?.worktree && (
      <span className="item worktree" key="worktree" title={`Linked worktree ${git.worktree}`}>
        ⑂ {git.worktree}
      </span>
    ),
  ].filter(Boolean);

  const right = [
    contextTokens !== undefined && (
      <span
        className="item"
        key="context"
        title={
          contextWindow
            ? `Context: ${COUNT.format(contextTokens)} of ${COUNT.format(contextWindow)} tokens`
            : `Context: ${COUNT.format(contextTokens)} tokens`
        }
      >
        <span className="label">ctx</span>
        {contextPercent !== undefined && (
          <>
            <Gauge percent={contextPercent} label="Context window" />
            <span className="value">{contextPercent.toFixed(1)}%</span>
          </>
        )}
        <span className="faint">
          {formatTokens(contextTokens)}
          {contextWindow ? `/${formatTokens(contextWindow)}` : ''}
        </span>
      </span>
    ),
    info.total !== undefined && !info.usageUnavailable && (
      <span
        className="item"
        key="total"
        title={`${COUNT.format(info.total)} tokens this session, subagents included. Most of it is context read again from the cache on every call.`}
      >
        <span className="value">{formatTokens(info.total)}</span>
        <span className="faint">tokens</span>
      </span>
    ),
    info.usageUnavailable && (
      <span className="item faint" key="unavailable" title={`Token usage cannot be read: ${info.usageUnavailable}`}>
        usage unavailable
      </span>
    ),
    info.fiveHour && <Limit key="5h" name="5h" title="5-hour limit" limit={info.fiveHour} now={now} />,
    info.sevenDay && <Limit key="7d" name="7d" title="7-day limit" limit={info.sevenDay} now={now} />,
  ].filter(Boolean);

  if (left.length === 0 && right.length === 0) return null;
  return (
    <footer className="statusbar" aria-label="Session details">
      <div className="side">{left}</div>
      <div className="side">{right}</div>
    </footer>
  );
}
