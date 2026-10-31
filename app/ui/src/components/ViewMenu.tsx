import { useEffect, useRef, useState } from 'react';
import { Eye, MessagesSquare, SquareTerminal } from 'lucide-react';
import type { View } from '../api/types';

/** The first answer of the first-join question: everything technical shown. */
export const DEVELOPER_VIEW: View = { terminal: true, steps: 'details' };
export const OVERVIEW_VIEW: View = { terminal: false, steps: 'summary' };

interface MenuProps {
  view: View;
  onChange: (view: View) => void;
  /** The host types into the terminal, so it cannot be hidden, and is never asked how to follow. */
  host?: boolean;
  onAskAgain?: () => void;
}

/** How this person follows sessions: the terminal on or off, and steps in words or in detail. */
export function ViewMenu({ view, onChange, host = false, onAskAgain }: MenuProps) {
  const [open, setOpen] = useState(false);
  const button = useRef<HTMLButtonElement>(null);
  const panel = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!open) return;
    const onKey = (event: KeyboardEvent) => {
      if (event.key !== 'Escape') return;
      setOpen(false);
      button.current?.focus();
    };
    const onDown = (event: MouseEvent) => {
      const target = event.target as Node;
      if (!panel.current?.contains(target) && !button.current?.contains(target)) setOpen(false);
    };
    window.addEventListener('keydown', onKey);
    window.addEventListener('mousedown', onDown);
    return () => {
      window.removeEventListener('keydown', onKey);
      window.removeEventListener('mousedown', onDown);
    };
  }, [open]);

  return (
    <div className="view-menu">
      <button ref={button} className="btn" aria-expanded={open} aria-controls="view-menu-panel" onClick={() => setOpen((o) => !o)}>
        <Eye aria-hidden /> View
      </button>
      {open && (
        <div ref={panel} id="view-menu-panel" className="view-menu-panel">
          {!host && (
            <label className="check">
              <input type="checkbox" checked={view.terminal} onChange={() => onChange({ ...view, terminal: !view.terminal })} />
              Show terminal
            </label>
          )}
          <fieldset>
            <legend>Steps</legend>
            <label className="check">
              <input type="radio" name="view-steps" checked={view.steps === 'summary'} onChange={() => onChange({ ...view, steps: 'summary' })} />
              <span>
                Summary <span className="muted">· what the agent did, in words</span>
              </span>
            </label>
            <label className="check">
              <input type="radio" name="view-steps" checked={view.steps === 'details'} onChange={() => onChange({ ...view, steps: 'details' })} />
              <span>
                Details <span className="muted">· tools, commands, diffs</span>
              </span>
            </label>
          </fieldset>
          <p className="muted">
            Remembered for every session.
            {!host && onAskAgain && (
              <>
                {' '}
                <button
                  className="link"
                  onClick={() => {
                    setOpen(false);
                    onAskAgain();
                  }}
                >
                  Show the first-join question again
                </button>
              </>
            )}
          </p>
        </div>
      )}
    </div>
  );
}

/** Asked the first time someone joins a session: how they want to follow it. */
export function FirstJoinDialog({ onChoose }: { onChoose: (view: View) => void }) {
  const first = useRef<HTMLButtonElement>(null);
  useEffect(() => first.current?.focus(), []);
  return (
    <div className="overlay">
      <div className="dialog first-join" role="dialog" aria-modal="true" aria-labelledby="first-join-title" aria-describedby="first-join-hint">
        <h2 id="first-join-title">How do you want to follow sessions?</h2>
        <p id="first-join-hint">You can change this any time under View.</p>
        <div className="choices">
          <button ref={first} className="choice" onClick={() => onChoose(OVERVIEW_VIEW)}>
            <MessagesSquare aria-hidden />
            <strong>Overview</strong>
            <span>The conversation and the plan: what was asked, what the agent did in plain words, and the documents it wrote.</span>
          </button>
          <button className="choice" onClick={() => onChoose(DEVELOPER_VIEW)}>
            <SquareTerminal aria-hidden />
            <strong>Developer</strong>
            <span>The terminal next to every tool call, command and diff.</span>
          </button>
        </div>
      </div>
    </div>
  );
}
