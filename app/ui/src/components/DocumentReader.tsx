import { useEffect, useRef, useState } from 'react';
import { FileText, X } from 'lucide-react';
import { compareBlocks, type DocumentVersion } from '../model/documents';
import type { FileChange } from '../model/timeline';
import { DiffView } from './DiffView';
import { Markdown } from './Markdown';

export interface ReaderDocument {
  title: string;
  path: string;
}

interface Props {
  /** What the documents belong to, such as "Plan · viewer-views". */
  heading: string;
  docs: ReaderDocument[];
  initial: string;
  versions: Map<string, DocumentVersion[]>;
  /** The version this participant last read, by path; kept by the caller across readers. */
  lastRead: Map<string, number>;
  onRead: (path: string, version: number) => void;
  /** File changes, for a document too large to show whole. */
  changes: FileChange[];
  onClose: () => void;
}

/** Reads documents the agent wrote, with what changed since this participant last read them. */
export function DocumentReader({ heading, docs, initial, versions, lastRead, onRead, changes, onClose }: Props) {
  const [path, setPath] = useState(initial);
  const [marks, setMarks] = useState(true);
  // What each document is compared against: the version read before this reader showed it.
  const [bases, setBases] = useState(() => new Map([[initial, lastRead.get(initial) ?? null]]));
  const close = useRef<HTMLButtonElement>(null);

  const latest = versions.get(path)?.at(-1) ?? null;
  const base = bases.has(path) ? bases.get(path)! : (lastRead.get(path) ?? null);

  // Shown is read.
  useEffect(() => {
    if (latest && lastRead.get(path) !== latest.version) onRead(path, latest.version);
  }, [path, latest, lastRead, onRead]);

  const show = (next: string) => {
    if (!bases.has(next)) setBases((known) => new Map(known).set(next, lastRead.get(next) ?? null));
    setPath(next);
  };

  useEffect(() => {
    close.current?.focus();
    const onKey = (event: KeyboardEvent) => event.key === 'Escape' && onClose();
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [onClose]);

  const title = docs.find((doc) => doc.path === path)?.title ?? path;
  const baseText = marks && base !== null ? (versions.get(path)?.[base]?.text ?? null) : null;

  return (
    <div className="overlay" onMouseDown={(event) => event.target === event.currentTarget && onClose()}>
      <div className="dialog reader" role="dialog" aria-modal="true" aria-labelledby="reader-title">
        <header>
          <div className="reader-title">
            <span className="muted">{heading}</span>
            <strong id="reader-title">{title}</strong>
          </div>
          <label className="check">
            <input type="checkbox" checked={marks} onChange={() => setMarks((on) => !on)} />
            Highlight changes since you last read it
          </label>
          <button ref={close} className="btn ghost icon" aria-label="Close" onClick={onClose}>
            <X aria-hidden />
          </button>
        </header>
        {docs.length > 1 && (
          <nav aria-label="Documents" className="reader-tabs">
            {docs.map((doc) => (
              <button key={doc.path} className="btn ghost small" aria-current={doc.path === path ? 'page' : undefined} onClick={() => show(doc.path)}>
                {doc.title}
              </button>
            ))}
          </nav>
        )}
        <div className="reader-body selectable">
          {!latest ? (
            <div className="empty">
              <FileText aria-hidden />
              <strong>Not written yet</strong>
            </div>
          ) : latest.text === null ? (
            <>
              <p>This document is too large to show whole. Its changes:</p>
              {changes
                .filter((change) => change.path === path)
                .map((change, i) => (
                  <DiffView key={i} change={change} />
                ))}
            </>
          ) : (
            compareBlocks(baseText, latest.text).map(({ block, mark }, i) => (
              <div key={i} className={`doc-block${mark ? ` mark-${mark}` : ''}`}>
                {mark && <span className="mark-tag">{mark === 'new' ? 'New' : 'Changed'}</span>}
                <Markdown text={block} />
              </div>
            ))
          )}
        </div>
      </div>
    </div>
  );
}
