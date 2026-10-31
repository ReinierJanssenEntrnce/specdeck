import { useState } from 'react';
import { ArrowLeft, Eye, FilePen, FilePlus, FileX, Files } from 'lucide-react';
import type { TouchedFile } from '../model/timeline';
import { DiffView } from './DiffView';

interface Props {
  files: TouchedFile[];
  timelineSupported: boolean;
  agentName: string;
  /** A terminal session, which never lists files. */
  terminal?: boolean;
  onJump: (outputSeq: number) => void;
}

/** Every file the agent read or changed; a changed file opens to all its diffs, in order. */
export function FilesPanel({ files, timelineSupported, agentName, terminal = false, onJump }: Props) {
  const [open, setOpen] = useState<string | null>(null);
  const file = files.find((f) => f.path === open);

  if (!timelineSupported) {
    return (
      <div className="empty">
        <Files aria-hidden />
        <strong>{terminal ? 'No file list for a terminal session' : `No file list for ${agentName}`}</strong>
        <span>Touched files are listed for Claude Code sessions.</span>
      </div>
    );
  }
  if (files.length === 0) {
    return (
      <div className="empty">
        <Files aria-hidden />
        <strong>No files touched yet</strong>
        <span>Files the agent reads or changes are listed here, with their diffs.</span>
      </div>
    );
  }
  if (file && !file.readOnly) {
    return (
      <div className="file-detail">
        <button className="btn ghost small" onClick={() => setOpen(null)} style={{ justifySelf: 'start' }}>
          <ArrowLeft aria-hidden /> All files
        </button>
        <strong className="selectable" style={{ fontFamily: 'var(--mono)', fontSize: 12.5 }}>
          {file.path}
        </strong>
        {file.changes.map((change, i) => (
          <div key={i} style={{ display: 'grid', gap: 4 }}>
            <button className="btn ghost small" style={{ justifySelf: 'start' }} onClick={() => onJump(change.outputSeq)}>
              Change {i + 1} of {file.changes.length} · show in terminal
            </button>
            <DiffView change={change} showPath={false} />
          </div>
        ))}
      </div>
    );
  }
  return (
    <ul className="files" aria-label="Touched files">
      {files.map((f) => {
        const created = f.changes.some((c) => c.change === 'created');
        const deleted = f.changes.at(-1)?.change === 'deleted';
        const Icon = f.readOnly ? Eye : deleted ? FileX : created ? FilePlus : FilePen;
        const tag = f.readOnly
          ? 'read'
          : deleted
            ? 'deleted'
            : f.changes.length === 1
              ? created
                ? 'new'
                : '1 change'
              : `${f.changes.length} changes`;
        return (
          <li key={f.path}>
            <button onClick={() => !f.readOnly && setOpen(f.path)} disabled={f.readOnly} aria-label={`${f.path}, ${tag === 'new' ? '1 change' : tag}`}>
              <Icon aria-hidden />
              <span className="path" title={f.path}>
                {f.path}
              </span>
              <span className="tag">{tag}</span>
            </button>
          </li>
        );
      })}
    </ul>
  );
}
