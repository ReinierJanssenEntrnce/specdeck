import { act, fireEvent, render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { demoParticipants, demoTimeline } from '../api/demo';
import { terminal, TerminalViewStub } from '../test/terminal-stub';
import { Workspace } from './Workspace';

vi.mock('../terminal/TerminalView', () => ({ TerminalView: TerminalViewStub }));

function renderWorkspace(props: Partial<Parameters<typeof Workspace>[0]> = {}) {
  return render(
    <Workspace
      mode="viewer"
      agentName="Claude Code"
      timelineSupported
      timeline={demoTimeline}
      participants={demoParticipants}
      me={2}
      {...props}
    />,
  );
}

beforeEach(() => vi.clearAllMocks());

describe('timeline panel', () => {
  it('shows prompts with the agent text, tool calls with their outcome, and diffs', () => {
    renderWorkspace();
    const first = screen.getByRole('article', { name: 'Prompt 1' });
    expect(first).toHaveTextContent('Add a /health endpoint');
    expect(within(first).getByText('Write')).toBeInTheDocument();
    expect(within(first).getByLabelText('Created src/routes/health.ts')).toHaveTextContent("+import { Router } from 'express';");
    expect(within(first).getByText('Failed')).toBeInTheDocument();
    expect(within(first).getByText(/no test files match/)).toBeInTheDocument();
    expect(screen.getByRole('article', { name: 'Prompt 2' })).toHaveTextContent('Running');
  });

  it('jumps the terminal to where an entry happened', async () => {
    renderWorkspace();
    await userEvent.click(screen.getByRole('button', { name: /Prompt 2/ }));
    const prompt2 = demoTimeline.find((e) => e.kind === 'prompt' && e.promptId === 'p2')!;
    expect(terminal.handle.scrollToSeq).toHaveBeenCalledWith(prompt2.outputSeq);
  });

  it('lists every prompt to jump to', async () => {
    renderWorkspace();
    await userEvent.selectOptions(screen.getByLabelText('Jump to'), 'p1');
    const prompt1 = demoTimeline.find((e) => e.kind === 'prompt' && e.promptId === 'p1')!;
    expect(terminal.handle.scrollToSeq).toHaveBeenCalledWith(prompt1.outputSeq);
  });

  it('says so for an agent without a timeline', () => {
    renderWorkspace({ timelineSupported: false, agentName: 'OpenCode', timeline: [] });
    // Opens on People, the only tab with content; the timeline explains itself.
    expect(screen.getByRole('tab', { name: /People/ })).toHaveAttribute('aria-selected', 'true');
    fireEvent.click(screen.getByRole('tab', { name: /Timeline/ }));
    expect(screen.getByText('No timeline for OpenCode')).toBeInTheDocument();
  });

  it('shows when the timeline is paused', () => {
    renderWorkspace({
      timeline: [...demoTimeline, { outputSeq: 99, at: 0, kind: 'timeline-unavailable', reason: 'No events arrived from the agent.' }],
    });
    expect(screen.getByText(/Timeline paused: No events arrived from the agent./)).toBeInTheDocument();
  });
});

describe('files panel', () => {
  it('lists touched files and opens every diff of a changed file', async () => {
    renderWorkspace();
    await userEvent.click(screen.getByRole('tab', { name: /Files/ }));
    const list = screen.getByRole('list', { name: 'Touched files' });
    expect(within(list).getAllByRole('button').map((b) => b.getAttribute('aria-label'))).toEqual([
      'src/routes/health.ts, 1 change',
      'src/routes/index.ts, 1 change',
    ]);
    await userEvent.click(screen.getByRole('button', { name: 'src/routes/index.ts, 1 change' }));
    expect(screen.getByText('Change 1 of 1 · show in terminal')).toBeInTheDocument();
    expect(screen.getByText('routes.use(health);')).toBeInTheDocument();
    await userEvent.click(screen.getByRole('button', { name: /All files/ }));
    expect(screen.getByRole('list', { name: 'Touched files' })).toBeInTheDocument();
  });
});

describe('search', () => {
  it('opens with the find shortcut and searches the terminal and the timeline', async () => {
    renderWorkspace();
    fireEvent.keyDown(window, { key: 'f', metaKey: true, ctrlKey: false });
    fireEvent.keyDown(window, { key: 'f', ctrlKey: true });
    const field = await screen.findByLabelText('Find in session');
    await userEvent.type(field, 'health');
    expect(terminal.handle.findNext).toHaveBeenLastCalledWith('health');
    act(() => (terminal.props.onSearchResults as (r: { index: number; count: number }) => void)({ index: 0, count: 4 }));
    expect(screen.getByRole('search')).toHaveTextContent('1/4 in terminal · 2 in timeline');
    // Both prompts mention it; stepping highlights them in turn.
    await userEvent.type(field, '{Enter}');
    expect(screen.getByRole('article', { name: 'Prompt 1' })).toHaveClass('current');
    await userEvent.type(field, '{Escape}');
    expect(screen.queryByRole('search')).not.toBeInTheDocument();
    expect(terminal.handle.clearSearch).toHaveBeenCalled();
  });

  it('says when there are no matches', async () => {
    renderWorkspace();
    fireEvent.keyDown(window, { key: 'f', metaKey: true });
    fireEvent.keyDown(window, { key: 'f', ctrlKey: true });
    await userEvent.type(await screen.findByLabelText('Find in session'), 'zebra');
    expect(screen.getByRole('search')).toHaveTextContent('No matches');
  });
});

describe('following the output', () => {
  it('offers to jump back to live output after scrolling away', async () => {
    renderWorkspace();
    expect(screen.queryByRole('button', { name: /Jump to live/ })).not.toBeInTheDocument();
    act(() => (terminal.props.onFollowChange as (f: boolean) => void)(false));
    act(() => (terminal.props.onNewOutput as () => void)());
    await userEvent.click(screen.getByRole('button', { name: 'New output · jump to live' }));
    expect(terminal.handle.scrollToBottom).toHaveBeenCalled();
    act(() => (terminal.props.onFollowChange as (f: boolean) => void)(true));
    expect(screen.queryByRole('button', { name: /Jump to live/ })).not.toBeInTheDocument();
  });
});
