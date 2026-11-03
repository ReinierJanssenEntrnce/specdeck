import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { DiffView } from './DiffView';
import type { FileChange } from '../model/timeline';

const change = (path: string): FileChange => ({
  path,
  change: 'edited',
  hunks: [],
  promptId: 'p',
  toolUseId: 't',
  at: 0,
  outputSeq: 0,
});

describe('a diff without hunks', () => {
  it('says the file is outside the session folder when the host shows it from outside', () => {
    for (const path of ['~/.claude/memory.md', '/etc/hosts', 'C:\\Users\\nick\\notes.md']) {
      const { unmount } = render(<DiffView change={change(path)} />);
      expect(screen.getByText('Outside the session folder')).toBeInTheDocument();
      unmount();
    }
  });

  it('says there is no diff for a file inside the session folder', () => {
    render(<DiffView change={change('assets/logo.png')} />);
    expect(screen.getByText('No diff to show')).toBeInTheDocument();
  });
});
