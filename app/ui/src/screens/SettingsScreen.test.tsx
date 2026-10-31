import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it } from 'vitest';
import { fakeBridge, renderWith } from '../test/helpers';
import { SettingsScreen } from './SettingsScreen';

describe('settings window', () => {
  it('changes the name when the field loses focus or on Enter', async () => {
    const bridge = fakeBridge();
    renderWith(bridge, <SettingsScreen />);
    const name = await screen.findByLabelText('Your name');
    expect(name).toHaveValue('Nick');
    await userEvent.clear(name);
    await userEvent.type(name, 'Anna{Enter}');
    expect(bridge.setUsername).toHaveBeenCalledWith('Anna');
    expect(await screen.findByText(/Saved/)).toBeInTheDocument();
  });

  it('does not accept an empty name', async () => {
    const bridge = fakeBridge();
    renderWith(bridge, <SettingsScreen />);
    const name = await screen.findByLabelText('Your name');
    await userEvent.clear(name);
    await userEvent.tab();
    expect(screen.getByRole('alert')).toHaveTextContent('Enter the name others should see.');
    expect(bridge.setUsername).not.toHaveBeenCalled();
  });

  it('turns notifications off and on', async () => {
    const bridge = fakeBridge();
    renderWith(bridge, <SettingsScreen />);
    const toggle = await screen.findByRole('checkbox', { name: 'Notifications' });
    expect(toggle).toBeChecked();
    await userEvent.click(toggle);
    expect(bridge.setNotifications).toHaveBeenCalledWith(false);
    await waitFor(() => expect(toggle).not.toBeChecked());
  });
});
