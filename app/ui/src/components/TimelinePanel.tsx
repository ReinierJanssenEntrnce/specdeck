import { useEffect, useRef } from 'react';
import { AlertTriangle, Check, CircleSlash, History, Loader, X } from 'lucide-react';
import type { TimelineModel, ToolItem, Turn } from '../model/timeline';
import { toolSummary } from '../model/timeline';
import { DiffView } from './DiffView';
import { Markdown } from './Markdown';

interface Props {
  model: TimelineModel;
  timelineSupported: boolean;
  agentName: string;
  /** Turns matching the current search, by prompt id. */
  matches: Set<string>;
  /** The turn the search stepped to. */
  current: string | null;
  /** Scrolls the terminal to where an entry happened. */
  onJump: (outputSeq: number) => void;
}

const TIME = new Intl.DateTimeFormat(undefined, { hour: '2-digit', minute: '2-digit' });

export function TimelinePanel({ model, timelineSupported, agentName, matches, current, onJump }: Props) {
  const list = useRef<HTMLDivElement>(null);
  const atBottom = useRef(true);

  // Follow new entries while the reader is at the end of the list.
  useEffect(() => {
    const element = list.current?.parentElement;
    if (!element) return;
    const onScroll = () => {
      atBottom.current = element.scrollHeight - element.scrollTop - element.clientHeight < 40;
    };
    element.addEventListener('scroll', onScroll);
    return () => element.removeEventListener('scroll', onScroll);
  }, []);
  useEffect(() => {
    const element = list.current?.parentElement;
    if (element && atBottom.current) element.scrollTo({ top: element.scrollHeight });
  }, [model]);

  useEffect(() => {
    if (current) document.getElementById(`turn-${current}`)?.scrollIntoView({ block: 'center' });
  }, [current]);

  if (!timelineSupported) {
    return (
      <div className="empty">
        <History aria-hidden />
        <strong>No timeline for {agentName}</strong>
        <span>
          SpecDeck builds the timeline of prompts, tool calls and file changes from Claude Code's events. For {agentName}, the
          terminal shows everything.
        </span>
      </div>
    );
  }

  const jumpTo = (promptId: string) => {
    const turn = model.turns.find((t) => t.promptId === promptId);
    if (!turn) return;
    document.getElementById(`turn-${promptId}`)?.scrollIntoView({ block: 'start' });
    onJump(turn.outputSeq);
  };

  return (
    <div ref={list}>
      {model.turns.length > 1 && (
        <div className="prompt-list">
          <label htmlFor="jump-prompt" className="field-hint">
            Jump to
          </label>
          <select id="jump-prompt" value="" onChange={(event) => jumpTo(event.target.value)}>
            <option value="" disabled>
              Prompt…
            </option>
            {model.turns.map((turn, i) => (
              <option key={turn.promptId} value={turn.promptId}>
                {i + 1}. {turn.prompt.slice(0, 60) || 'Earlier prompt'}
              </option>
            ))}
          </select>
        </div>
      )}
      {model.unavailable && (
        <div className="banner warning" role="status">
          <AlertTriangle aria-hidden />
          <span>Timeline paused: {model.unavailable} The terminal keeps working.</span>
        </div>
      )}
      {model.turns.length === 0 ? (
        <div className="empty">
          <History aria-hidden />
          <strong>No prompts yet</strong>
          <span>Prompts, tool calls and file changes appear here as they happen.</span>
        </div>
      ) : (
        <div className="timeline">
          {model.turns.map((turn, i) => (
            <TurnView
              key={turn.promptId}
              turn={turn}
              number={i + 1}
              match={matches.has(turn.promptId)}
              current={current === turn.promptId}
              onJump={onJump}
            />
          ))}
        </div>
      )}
    </div>
  );
}

function TurnView({ turn, number, match, current, onJump }: { turn: Turn; number: number; match: boolean; current: boolean; onJump: (seq: number) => void }) {
  const texts = turn.items.filter((item) => item.kind === 'text');
  const showResponse = turn.response && !texts.some((item) => item.text.trim() === turn.response?.trim());
  return (
    <article id={`turn-${turn.promptId}`} className={`turn${match ? ' match' : ''}${current ? ' current' : ''}`} aria-label={`Prompt ${number}`}>
      <button className="prompt" onClick={() => onJump(turn.outputSeq)} title="Show in the terminal">
        <span className="meta">
          <span>Prompt {number}</span>
          <span>{turn.at ? TIME.format(turn.at) : ''}</span>
        </span>
        <span className="text selectable">{turn.prompt || 'Earlier prompt'}</span>
      </button>
      {turn.items.map((item) =>
        item.kind === 'text' ? <Markdown key={item.id} text={item.text} /> : <ToolView key={item.id} item={item} onJump={onJump} />,
      )}
      {showResponse && <Markdown text={turn.response ?? ''} />}
    </article>
  );
}

const STATUS_TEXT = { running: 'Running', succeeded: 'Done', failed: 'Failed', 'not-run': 'Did not run' } as const;

function ToolView({ item, onJump }: { item: ToolItem; onJump: (seq: number) => void }) {
  const Icon = { running: Loader, succeeded: Check, failed: X, 'not-run': CircleSlash }[item.status];
  return (
    <details className={`tool ${item.status}`} open={item.changes.length > 0}>
      <summary onClick={() => onJump(item.outputSeq)}>
        <Icon className="status-icon" aria-hidden />
        <span className="name">{item.toolName}</span>
        <span className="what" title={toolSummary(item)}>
          {toolSummary(item)}
        </span>
        <span className="state">{STATUS_TEXT[item.status]}</span>
      </summary>
      <div className="tool-body">
        {item.error && <div className="error-text selectable">{item.error}</div>}
        {item.changes.length > 0 ? (
          item.changes.map((change, i) => <DiffView key={i} change={change} />)
        ) : (
          <pre className="tool-input selectable">{JSON.stringify(item.input, null, 2)}</pre>
        )}
      </div>
    </details>
  );
}

