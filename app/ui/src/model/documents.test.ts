import { describe, expect, it } from 'vitest';
import type { TimelineEvent } from '../api/types';
import { buildDocuments, compareBlocks, splitBlocks } from './documents';

describe('document blocks', () => {
  it('splits Markdown into headings, paragraphs, list items and whole code blocks', () => {
    const text = '# Title\n\nFirst paragraph\nstill first.\n\n- one\n- two\n  continued\n\n```\na\n\nb\n```\n## Next\nText';
    expect(splitBlocks(text)).toEqual(['# Title', 'First paragraph\nstill first.', '- one', '- two\n  continued', '```\na\n\nb\n```', '## Next', 'Text']);
  });

  it('marks blocks that are new or changed since the version read', () => {
    const read = '# Design\n\n## Shell\n\nzsh, bash, sh.\n\n## Texts\n\nA confirmation step.';
    const now = '# Design\n\n## Shell\n\nzsh, bash, sh.\n\n## Texts\n\nNo extra step.\n\n## Risks\n\nNone.';
    expect(compareBlocks(read, now).map(({ mark }) => mark)).toEqual([null, null, null, null, 'changed', 'new', 'new']);
  });

  it('marks nothing for a first read', () => {
    expect(compareBlocks(null, '# A\n\nB').every(({ mark }) => mark === null)).toBe(true);
  });
});

describe('documents and plans from the timeline', () => {
  const at = (n: number) => ({ outputSeq: n, at: n });
  const events: TimelineEvent[] = [
    { ...at(1), kind: 'document', promptId: 'p1', toolUseId: 't1', path: 'openspec/changes/a/proposal.md', text: '# A' },
    {
      ...at(2),
      kind: 'plan',
      change: 'a',
      documents: [{ id: 'proposal', name: 'Proposal', paths: ['openspec/changes/a/proposal.md'], exists: true }],
      tasks: { done: 0, total: 3 },
    },
    { ...at(3), kind: 'document', promptId: 'p2', toolUseId: 't2', path: 'openspec/changes/a/proposal.md', text: '# A, revised' },
    { ...at(4), kind: 'plan', change: 'b', documents: [], tasks: { done: 0, total: 0 } },
    { ...at(5), kind: 'plan', change: 'a', documents: [], tasks: { done: 1, total: 3 } },
    { ...at(6), kind: 'document', promptId: 'p2', toolUseId: 't3', path: 'big.md', text: null },
  ];

  it('keeps every version of a document, and the latest plan per change, most recent first', () => {
    const { documents, plans } = buildDocuments(events);
    expect(documents.get('openspec/changes/a/proposal.md')!.map((v) => v.text)).toEqual(['# A', '# A, revised']);
    expect(documents.get('big.md')![0]!.text).toBeNull();
    expect(plans.map((plan) => [plan.change, plan.tasks.done])).toEqual([
      ['a', 1],
      ['b', 0],
    ]);
  });
});
