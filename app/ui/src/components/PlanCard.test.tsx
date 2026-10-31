import { render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import type { DocumentVersion, Plan } from '../model/documents';
import { PlanCard, planReaderDocuments } from './PlanCard';

const dir = 'openspec/changes/host-terminal-session';
const defaultPlan: Plan = {
  change: 'host-terminal-session',
  documents: [
    { id: 'proposal', name: 'Proposal', paths: [`${dir}/proposal.md`], exists: true },
    { id: 'specs', name: 'Specs', paths: [`${dir}/specs/session-hosting/spec.md`, `${dir}/specs/viewer-experience/spec.md`], exists: true },
    { id: 'design', name: 'Design', paths: [], exists: false },
    { id: 'tasks', name: 'Tasks', paths: [`${dir}/tasks.md`], exists: true },
  ],
  tasks: { done: 2, total: 8 },
};
const version = (n: number, text: string): DocumentVersion => ({ version: n, text, promptId: 'p', toolUseId: 't' });
const versions = new Map([
  [`${dir}/proposal.md`, [version(0, '# Host a plain terminal session\n\nWhy.')]],
  [`${dir}/tasks.md`, [version(0, '- [ ] 1.1'), version(1, '- [x] 1.1')]],
]);

describe('plan card', () => {
  it('shows the plan with its documents in schema order and task progress', async () => {
    const onOpen = vi.fn();
    render(<PlanCard plan={defaultPlan} versions={versions} lastRead={new Map([[`${dir}/tasks.md`, 0]])} onOpen={onOpen} />);
    const card = screen.getByRole('region', { name: 'Plan: Host a plain terminal session' });
    expect(card).toHaveTextContent('host-terminal-session');
    expect(within(card).getAllByRole('button').map((b) => b.textContent)).toEqual(['Proposalnew', 'Specs (2)', 'Design', 'Taskschanged']);
    expect(within(card).getByRole('button', { name: /Design/ })).toBeDisabled();
    expect(within(card).getByRole('progressbar', { name: 'Tasks' })).toHaveAttribute('aria-valuenow', '2');
    expect(card).toHaveTextContent('2 of 8 tasks done');
    await userEvent.click(within(card).getByRole('button', { name: /Specs/ }));
    expect(onOpen).toHaveBeenCalledWith(`${dir}/specs/session-hosting/spec.md`);
  });

  it('follows a custom schema, with its own names and order', () => {
    const plan: Plan = {
      change: 'checkout',
      documents: [
        { id: 'feature-brief', name: 'Feature brief', paths: ['openspec/changes/checkout/brief.md'], exists: true },
        { id: 'proposal', name: 'Proposal', paths: [], exists: false },
      ],
      tasks: { done: 0, total: 0 },
    };
    render(<PlanCard plan={plan} versions={new Map()} lastRead={new Map()} onOpen={vi.fn()} />);
    const card = screen.getByRole('region', { name: 'Plan: Checkout' });
    expect(within(card).getAllByRole('button').map((b) => b.textContent)).toEqual(['Feature brief', 'Proposal']);
    expect(within(card).queryByRole('progressbar')).not.toBeInTheDocument();
  });

  it('updates when a task is ticked', () => {
    const { rerender } = render(<PlanCard plan={defaultPlan} versions={versions} lastRead={new Map()} onOpen={vi.fn()} />);
    rerender(<PlanCard plan={{ ...defaultPlan, tasks: { done: 3, total: 8 } }} versions={versions} lastRead={new Map()} onOpen={vi.fn()} />);
    expect(screen.getByRole('progressbar', { name: 'Tasks' })).toHaveAttribute('aria-valuenow', '3');
  });

  it('gives the reader one tab per file, specs by capability', () => {
    expect(planReaderDocuments(defaultPlan).map((doc) => doc.title)).toEqual(['Proposal', 'Spec: session-hosting', 'Spec: viewer-experience', 'Tasks']);
  });
});
