import { Gauge } from 'lucide-react';
import type { Usage } from '../model/timeline';

const COUNT = new Intl.NumberFormat();

/** A token count in a few characters: 950, 12.3k, 245k, 1.2M. */
export function formatTokens(tokens: number): string {
  if (tokens < 1000) return String(tokens);
  if (tokens < 100_000) return `${(tokens / 1000).toFixed(1).replace(/\.0$/, '')}k`;
  if (tokens < 1_000_000) return `${Math.round(tokens / 1000)}k`;
  return `${(tokens / 1_000_000).toFixed(1).replace(/\.0$/, '')}M`;
}

/** How full the agent's context window is, and the tokens used this session. */
export function UsageMeter({ usage }: { usage: Usage | null }) {
  if (!usage) return null;
  if (!usage.available) {
    return (
      <span className="pill" title={`Token usage cannot be read: ${usage.reason}`}>
        <Gauge size={13} aria-hidden /> Usage unavailable
      </span>
    );
  }

  const { contextTokens, contextWindow, tokens, total } = usage;
  const percent = contextTokens !== undefined && contextWindow ? Math.min(100, Math.round((contextTokens / contextWindow) * 100)) : null;
  const level = percent === null ? '' : percent >= 90 ? ' full' : percent >= 75 ? ' high' : '';
  const details = [
    contextTokens === undefined
      ? null
      : contextWindow
        ? `Context: ${COUNT.format(contextTokens)} of ${COUNT.format(contextWindow)} tokens (${percent}%)`
        : `Context: ${COUNT.format(contextTokens)} tokens`,
    `This session: ${COUNT.format(total)} tokens`,
    `Input ${COUNT.format(tokens.input)} · Output ${COUNT.format(tokens.output)} · Cache read ${COUNT.format(tokens.cacheRead)} · Cache write ${COUNT.format(tokens.cacheWrite)}`,
    'Most of the session total is context the agent reads again from its cache on every call.',
  ].filter(Boolean);

  return (
    <div className={`usage${level}`} title={details.join('\n')}>
      {percent !== null && (
        <span
          className="bar"
          role="meter"
          aria-label="Context window"
          aria-valuemin={0}
          aria-valuemax={100}
          aria-valuenow={percent}
          aria-valuetext={`${percent}% of the context window used`}
        >
          <span style={{ width: `${percent}%` }} />
        </span>
      )}
      {percent !== null ? (
        <span>{percent}% context</span>
      ) : (
        contextTokens !== undefined && <span>{formatTokens(contextTokens)} in context</span>
      )}
      <span className="total">{formatTokens(total)} tokens</span>
    </div>
  );
}
