import { act, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import type { ViewEvent } from '../api/types';
import { demoParticipants } from '../api/demo';
import { fakeBridge, renderWith } from '../test/helpers';
import { terminal, TerminalViewStub } from '../test/terminal-stub';
import { ViewerScreen } from './ViewerScreen';

vi.mock('../terminal/TerminalView', () => ({ TerminalView: TerminalViewStub }));

let emit: (event: ViewEvent) => void = () => {};

function setup() {
  const bridge = fakeBridge('viewer');
  bridge.joinSession = vi.fn(async (_code, _terminal, events) => {
    emit = events;
  });
  const onLeave = vi.fn();
  renderWith(bridge, <ViewerScreen code="7K2M9QXH4TPA" onLeave={onLeave} />);
  return { bridge, onLeave };
}

function join() {
  act(() => {
    emit({ type: 'joined', viewerId: 2, agent: 'claude-code', timelineSupported: true, cols: 100, rows: 30 });
    emit({ type: 'participants', participants: demoParticipants });
    emit({ type: 'status', status: 'waiting' });
  });
}

beforeEach(() => vi.clearAllMocks());

describe('viewer screen', () => {
  it('shows that it is joining, then the host session read-only', () => {
    const { bridge } = setup();
    expect(bridge.joinSession).toHaveBeenCalledWith('7K2M9QXH4TPA', expect.any(Function), expect.any(Function));
    expect(screen.getByText('Joining the session…')).toBeInTheDocument();
    join();
    expect(screen.queryByText('Joining the session…')).not.toBeInTheDocument();
    expect(screen.getByText("Nick's session")).toBeInTheDocument();
    expect(screen.getByText('Read-only')).toBeInTheDocument();
    expect(screen.getByText('Waiting for input')).toBeInTheDocument();
    // Nothing to type into the session with.
    expect(terminal.props.mode).toBe('viewer');
    expect(screen.queryByRole('textbox')).not.toBeInTheDocument();
  });

  it('marks you in the participant list', async () => {
    setup();
    join();
    await userEvent.click(screen.getByRole('tab', { name: /People/ }));
    expect(screen.getByText('Bram (you)')).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: /Remove/ })).not.toBeInTheDocument();
  });

  it('shows a reconnecting banner while the connection is back', () => {
    setup();
    join();
    act(() => emit({ type: 'reconnecting' }));
    expect(screen.getByText(/Connection lost. Reconnecting/)).toBeInTheDocument();
    act(() => emit({ type: 'reconnected' }));
    expect(screen.queryByText(/Connection lost/)).not.toBeInTheDocument();
  });

  it('lets the viewer keep reading after the host ends the session', () => {
    setup();
    join();
    act(() => emit({ type: 'ended', reason: 'closed' }));
    expect(screen.getByText(/The host ended the session/)).toBeInTheDocument();
    expect(screen.getByText('Ended')).toBeInTheDocument();
    expect(screen.getByTestId('terminal')).toBeInTheDocument();
  });

  it('explains a code that does not work and offers to try again', async () => {
    const { onLeave } = setup();
    act(() => emit({ type: 'ended', reason: 'invalid-code' }));
    expect(screen.getByRole('alert')).toHaveTextContent('That code does not work');
    await userEvent.click(screen.getByRole('button', { name: 'Try again' }));
    expect(onLeave).toHaveBeenCalledWith({ code: '7K2M9QXH4TPA', message: 'That code does not work' });
  });

  it('explains a network that blocks the connection', () => {
    setup();
    act(() => emit({ type: 'ended', reason: 'network-blocked' }));
    expect(screen.getByRole('alert')).toHaveTextContent('UDP');
  });

  it('leaves', async () => {
    const { bridge, onLeave } = setup();
    join();
    await userEvent.click(screen.getByRole('button', { name: /Leave/ }));
    expect(bridge.leaveSession).toHaveBeenCalled();
    await waitFor(() => expect(onLeave).toHaveBeenCalledWith());
  });

  it('shows the session usage to viewers', () => {
    setup();
    join();
    act(() =>
      emit({
        type: 'timeline',
        event: { outputSeq: 0, at: 1, kind: 'usage', tokens: { input: 1, output: 1, cacheRead: 0, cacheWrite: 0 }, contextTokens: 100_000, contextWindow: 200_000 },
      }),
    );
    expect(screen.getByRole('meter', { name: 'Context window' })).toHaveAttribute('aria-valuenow', '50');
  });

  it('shows no usage for an agent without a timeline', () => {
    setup();
    act(() => {
      emit({ type: 'joined', viewerId: 2, agent: 'open-code', timelineSupported: false, cols: 100, rows: 30 });
      emit({ type: 'timeline', event: { outputSeq: 0, at: 1, kind: 'usage', tokens: { input: 1, output: 1, cacheRead: 0, cacheWrite: 0 } } });
    });
    expect(screen.queryByText(/tokens/)).toBeNull();
  });
});
