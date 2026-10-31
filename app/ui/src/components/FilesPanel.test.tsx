import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import type { FileChange, TouchedFile } from '../model/timeline';
import { FilesPanel } from './FilesPanel';

const change = (path: string, kind: FileChange['change'], lines: string[]): FileChange => ({
  path,
  change: kind,
  hunks: lines.length ? [{ oldStart: 1, oldLines: lines.length, newStart: 0, newLines: 0, lines }] : [],
  promptId: 'p1',
  toolUseId: 't1',
  at: 0,
  outputSeq: 0,
});

describe('files panel', () => {
  it('shows a file a shell command deleted, and says when a change has no diff', async () => {
    const files: TouchedFile[] = [
      { path: 'old.txt', readOnly: false, changes: [change('old.txt', 'deleted', ['-bye'])] },
      { path: 'logo.png', readOnly: false, changes: [change('logo.png', 'edited', [])] },
    ];
    render(<FilesPanel files={files} timelineSupported agentName="Claude Code" onJump={vi.fn()} />);
    expect(screen.getByRole('button', { name: 'old.txt, deleted' })).toHaveTextContent('deleted');

    await userEvent.click(screen.getByRole('button', { name: 'old.txt, deleted' }));
    expect(screen.getByLabelText('Deleted old.txt')).toHaveTextContent('bye');

    await userEvent.click(screen.getByRole('button', { name: /All files/ }));
    await userEvent.click(screen.getByRole('button', { name: 'logo.png, 1 change' }));
    expect(screen.getByLabelText('Changed logo.png')).toHaveTextContent('No diff to show');
  });
});
