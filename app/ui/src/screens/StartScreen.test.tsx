import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { demoProfile } from '../api/demo';
import type { Profile } from '../api/types';
import { fakeBridge, renderWith } from '../test/helpers';
import { StartScreen } from './StartScreen';

function setup(profile: Partial<Profile> = {}, bridgeOverrides = {}) {
  const bridge = fakeBridge('start', bridgeOverrides);
  const handlers = { onProfile: vi.fn(), onHosting: vi.fn(), onJoin: vi.fn() };
  renderWith(bridge, <StartScreen profile={{ ...demoProfile, ...profile }} {...handlers} />);
  return { bridge, ...handlers };
}

describe('start screen', () => {
  it('asks for a name before anything else', async () => {
    const { bridge, onProfile } = setup({ username: null });
    expect(screen.queryByText('Host a session')).not.toBeInTheDocument();
    await userEvent.click(screen.getByRole('button', { name: 'Continue' }));
    expect(screen.getByRole('alert')).toHaveTextContent('Enter the name others should see.');
    await userEvent.type(screen.getByLabelText('What should others call you?'), 'Anna');
    await userEvent.click(screen.getByRole('button', { name: 'Continue' }));
    expect(bridge.setUsername).toHaveBeenCalledWith('Anna');
    await waitFor(() => expect(onProfile).toHaveBeenCalled());
  });

  it('shows which agents are installed and preselects the last used one', () => {
    setup({ lastAgent: 'open-code' });
    const agents = screen.getAllByRole('radio');
    expect(agents.map((a) => a.textContent)).toEqual(['Claude CodeTimeline and diffs', 'OpenCodeTerminal only', 'ompNot installed']);
    expect(screen.getByRole('radio', { name: /OpenCode/ })).toHaveAttribute('aria-checked', 'true');
    expect(screen.getByRole('radio', { name: /omp/ })).toBeDisabled();
  });

  it('starts a session in a recent folder without a dialog', async () => {
    const { bridge, onHosting } = setup();
    await userEvent.click(screen.getByRole('button', { name: 'Start a session in webshop' }));
    expect(bridge.chooseFolder).not.toHaveBeenCalled();
    expect(bridge.startHosting).toHaveBeenCalledWith('claude-code', '/Users/nick/projects/webshop', expect.any(Number), expect.any(Number));
    await waitFor(() => expect(onHosting).toHaveBeenCalled());
  });

  it('starts in a chosen folder', async () => {
    const { bridge, onHosting } = setup({ recentFolders: [] });
    await userEvent.click(screen.getByRole('button', { name: /Choose a folder and start/ }));
    expect(bridge.chooseFolder).toHaveBeenCalled();
    await waitFor(() => expect(onHosting).toHaveBeenCalled());
  });

  it('explains why a session could not start', async () => {
    const startHosting = vi.fn().mockRejectedValue('Claude Code is not installed. Install it and log in, then start the session again.');
    const { onHosting } = setup({}, { startHosting });
    await userEvent.click(screen.getByRole('button', { name: 'Start a session in webshop' }));
    expect(await screen.findByRole('alert')).toHaveTextContent('Claude Code is not installed.');
    expect(onHosting).not.toHaveBeenCalled();
  });

  it('removes a folder from the recent list', async () => {
    const { bridge } = setup();
    await userEvent.click(screen.getByRole('button', { name: 'Remove webshop from recent folders' }));
    expect(bridge.removeRecentFolder).toHaveBeenCalledWith('/Users/nick/projects/webshop');
  });

  it('formats the code as it is typed and joins when it is complete', async () => {
    const { onJoin } = setup();
    const field = screen.getByLabelText('Invite code');
    const join = screen.getByRole('button', { name: 'Join' });
    await userEvent.type(field, '7k2m9qxh4tp');
    expect(field).toHaveValue('7K2M-9QXH-4TP');
    expect(join).toBeDisabled();
    await userEvent.type(field, 'a');
    expect(join).toBeEnabled();
    await userEvent.click(join);
    expect(onJoin).toHaveBeenCalledWith('7K2M-9QXH-4TPA');
  });

  it('takes a pasted join link', async () => {
    setup();
    const field = screen.getByLabelText('Invite code');
    await userEvent.click(field);
    await userEvent.paste('specdeck://join/7k2m-9qxh-4tpa');
    expect(field).toHaveValue('7K2M-9QXH-4TPA');
  });

  it('shows why the last join did not work', () => {
    const bridge = fakeBridge();
    renderWith(
      bridge,
      <StartScreen profile={demoProfile} onProfile={vi.fn()} onHosting={vi.fn()} onJoin={vi.fn()} initialCode="7K2M9QXH4TPA" joinError="That code does not work" />,
    );
    expect(screen.getByLabelText('Invite code')).toHaveValue('7K2M-9QXH-4TPA');
    expect(screen.getByRole('alert')).toHaveTextContent('That code does not work');
  });

  it('lets the user change their name', async () => {
    const { bridge } = setup();
    await userEvent.click(screen.getByRole('button', { name: 'Change' }));
    const field = screen.getByLabelText('Your name');
    await userEvent.clear(field);
    await userEvent.type(field, 'Nicolaas{Enter}');
    expect(bridge.setUsername).toHaveBeenCalledWith('Nicolaas');
  });
});
