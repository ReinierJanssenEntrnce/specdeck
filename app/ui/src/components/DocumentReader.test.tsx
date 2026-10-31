import { render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import type { DocumentVersion } from '../model/documents';
import type { FileChange } from '../model/timeline';
import { DocumentReader } from './DocumentReader';

const version = (n: number, text: string | null): DocumentVersion => ({ version: n, text, promptId: 'p', toolUseId: `t${n}` });
const design = [
  version(0, '# Design\n\n## Texts\n\nA confirmation step.'),
  version(1, '# Design\n\n## Texts\n\nNo extra step.\n\n## Risks\n\nNone.'),
];

function open(lastRead: Map<string, number>, extra: Partial<Parameters<typeof DocumentReader>[0]> = {}) {
  const onRead = vi.fn((path: string, n: number) => lastRead.set(path, n));
  const onClose = vi.fn();
  const view = render(
    <DocumentReader
      heading="Plan · demo"
      docs={[
        { title: 'Design', path: 'design.md' },
        { title: 'Proposal', path: 'proposal.md' },
      ]}
      initial="design.md"
      versions={new Map([
        ['design.md', design],
        ['proposal.md', [version(0, '# Proposal')]],
      ])}
      lastRead={lastRead}
      onRead={onRead}
      changes={[]}
      onClose={onClose}
      {...extra}
    />,
  );
  return { onRead, onClose, ...view };
}

describe('document reader', () => {
  it('shows a document read for the first time without marks', () => {
    const { onRead } = open(new Map());
    const dialog = screen.getByRole('dialog', { name: 'Design' });
    expect(within(dialog).getByRole('heading', { name: 'Design', level: 1 })).toBeInTheDocument();
    expect(within(dialog).queryByText('Changed')).not.toBeInTheDocument();
    expect(within(dialog).queryByText('New')).not.toBeInTheDocument();
    expect(onRead).toHaveBeenCalledWith('design.md', 1);
  });

  it('marks what changed since this person last read it, and can hide the marks', async () => {
    open(new Map([['design.md', 0]]));
    const changed = screen.getByText('No extra step.').closest('.doc-block')!;
    expect(changed).toHaveTextContent('Changed');
    expect(screen.getByText('None.').closest('.doc-block')).toHaveTextContent('New');
    await userEvent.click(screen.getByLabelText('Highlight changes since you last read it'));
    expect(screen.queryByText('Changed')).not.toBeInTheDocument();
  });

  it('keeps last read per participant: someone who read the latest sees no marks', () => {
    open(new Map([['design.md', 1]]));
    expect(screen.queryByText('Changed')).not.toBeInTheDocument();
  });

  it('switches documents and closes with Escape', async () => {
    const { onClose } = open(new Map());
    await userEvent.click(screen.getByRole('button', { name: 'Proposal' }));
    expect(screen.getByRole('dialog', { name: 'Proposal' })).toBeInTheDocument();
    await userEvent.keyboard('{Escape}');
    expect(onClose).toHaveBeenCalled();
  });

  it('offers the diffs of a document too large to show whole', () => {
    const change: FileChange = {
      path: 'big.md',
      change: 'edited',
      hunks: [{ oldStart: 1, oldLines: 1, newStart: 1, newLines: 1, lines: ['-old', '+new'] }],
      promptId: 'p',
      toolUseId: 't',
      at: 0,
      outputSeq: 0,
    };
    open(new Map(), {
      docs: [{ title: 'big.md', path: 'big.md' }],
      initial: 'big.md',
      versions: new Map([['big.md', [version(0, null)]]]),
      changes: [change],
    });
    expect(screen.getByText('This document is too large to show whole. Its changes:')).toBeInTheDocument();
    expect(screen.getByLabelText('Changed big.md')).toHaveTextContent('new');
  });
});
