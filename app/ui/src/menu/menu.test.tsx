import { act, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { App } from '../App';
import type { Bridge } from '../api/bridge';
import { fakeBridge, renderWith } from '../test/helpers';
import { terminal } from '../test/terminal-stub';
import type { MenuCommand } from './menu';

vi.mock('../terminal/TerminalView', async () => ({ TerminalView: (await import('../test/terminal-stub')).TerminalViewStub }));

let menu: (command: MenuCommand) => void = () => {};

/** The app with a bridge whose native menu the test can use. */
async function setup(screenName = 'start', overrides: Partial<Bridge> = {}) {
  const bridge = fakeBridge(screenName, {
    onMenu: (handler) => {
      menu = handler;
      return () => {};
    },
    ...overrides,
  });
  renderWith(bridge, <App />);
  await screen.findAllByText(screenName === 'host' ? 'webshop' : 'SpecDeck');
  return bridge;
}

beforeEach(() => vi.clearAllMocks());

describe('menu commands', () => {
  it('File > Host a Session… chooses a folder and starts', async () => {
    const bridge = await setup();
    act(() => menu({ id: 'host' }));
    await waitFor(() => expect(bridge.startHosting).toHaveBeenCalledWith(expect.any(String), '/Users/nick/projects/webshop', expect.any(Number), expect.any(Number)));
  });

  it('File > Open Recent starts in that folder', async () => {
    const bridge = await setup();
    act(() => menu({ id: 'recent', path: '/Users/nick/projects/api' }));
    await waitFor(() => expect(bridge.startHosting).toHaveBeenCalledWith(expect.any(String), '/Users/nick/projects/api', expect.any(Number), expect.any(Number)));
    expect(bridge.chooseFolder).not.toHaveBeenCalled();
  });

  it('File > Join a Session… puts the cursor in the code field', async () => {
    await setup();
    act(() => menu({ id: 'join' }));
    expect(screen.getByLabelText('Invite code')).toHaveFocus();
  });

  it('asks a viewer before leaving the session for another', async () => {
    const bridge = await setup('start');
    await userEvent.type(screen.getByLabelText('Invite code'), '7K2M9QXH4TPA');
    await userEvent.click(screen.getByRole('button', { name: /Join/ }));
    act(() => menu({ id: 'join' }));
    const dialog = screen.getByRole('alertdialog', { name: 'Leave this session?' });
    await userEvent.click(within(dialog).getByRole('button', { name: 'Cancel' }));
    expect(bridge.leaveSession).not.toHaveBeenCalled();

    act(() => menu({ id: 'join' }));
    await userEvent.click(within(screen.getByRole('alertdialog')).getByRole('button', { name: 'Leave' }));
    expect(bridge.leaveSession).toHaveBeenCalled();
    expect(screen.getByLabelText('Invite code')).toHaveFocus();
  });

  it('keeps a hosted session: one session at a time', async () => {
    const bridge = await setup('host');
    act(() => menu({ id: 'join' }));
    expect(await screen.findByText('End this session before you host or join another.')).toBeInTheDocument();
    expect(bridge.leaveSession).not.toHaveBeenCalled();
  });

  it('File > Copy Invite Code copies it while hosting', async () => {
    const bridge = await setup('host');
    act(() => menu({ id: 'copy-code' }));
    await waitFor(() => expect(bridge.copyText).toHaveBeenCalledWith('7K2M-9QXH-4TPA'));
  });

  it('goes back to the start screen when the session is ended from the tray', async () => {
    await setup('host');
    act(() => menu({ id: 'session-ended' }));
    expect(await screen.findByRole('heading', { name: 'Host a session' })).toBeInTheDocument();
  });

  it('View and Edit > Find act on the session', async () => {
    await setup('host');
    act(() => menu({ id: 'tab:people' }));
    expect(screen.getByRole('tab', { name: /People/ })).toHaveAttribute('aria-selected', 'true');
    act(() => menu({ id: 'find' }));
    expect(screen.getByLabelText('Find in session')).toHaveFocus();
    act(() => menu({ id: 'jump-to-live' }));
    expect(terminal.handle.scrollToBottom).toHaveBeenCalled();
  });

  it('Help > Keyboard Shortcuts lists them', async () => {
    await setup();
    act(() => menu({ id: 'shortcuts' }));
    const dialog = screen.getByRole('dialog', { name: 'Keyboard shortcuts' });
    expect(dialog).toHaveTextContent('Find in the terminal and timeline');
    await userEvent.keyboard('{Escape}');
    expect(screen.queryByRole('dialog')).toBeNull();
  });
});
