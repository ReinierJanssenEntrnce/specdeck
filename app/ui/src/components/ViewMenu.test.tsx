import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { FirstJoinDialog, ViewMenu } from './ViewMenu';

describe('first-join question', () => {
  it('sets both settings from either answer', async () => {
    const onChoose = vi.fn();
    render(<FirstJoinDialog onChoose={onChoose} />);
    expect(screen.getByRole('dialog', { name: 'How do you want to follow sessions?' })).toHaveTextContent('You can change this any time under View');
    await userEvent.click(screen.getByRole('button', { name: /Overview/ }));
    expect(onChoose).toHaveBeenLastCalledWith({ terminal: false, steps: 'summary' });
    await userEvent.click(screen.getByRole('button', { name: /Developer/ }));
    expect(onChoose).toHaveBeenLastCalledWith({ terminal: true, steps: 'details' });
  });
});

describe('view menu', () => {
  it('lets a viewer switch the terminal and the steps, and ask the question again', async () => {
    const onChange = vi.fn();
    const onAskAgain = vi.fn();
    render(<ViewMenu view={{ terminal: true, steps: 'details' }} onChange={onChange} onAskAgain={onAskAgain} />);
    expect(screen.queryByLabelText('Show terminal')).not.toBeInTheDocument();
    await userEvent.click(screen.getByRole('button', { name: 'View' }));
    expect(screen.getByLabelText('Show terminal')).toBeChecked();
    await userEvent.click(screen.getByLabelText('Show terminal'));
    expect(onChange).toHaveBeenLastCalledWith({ terminal: false, steps: 'details' });
    await userEvent.click(screen.getByRole('radio', { name: /Summary/ }));
    expect(onChange).toHaveBeenLastCalledWith({ terminal: true, steps: 'summary' });
    await userEvent.click(screen.getByRole('button', { name: 'Show the first-join question again' }));
    expect(onAskAgain).toHaveBeenCalled();
  });

  it('closes with Escape and gives the host no terminal choice', async () => {
    render(<ViewMenu host view={{ terminal: false, steps: 'summary' }} onChange={vi.fn()} />);
    await userEvent.click(screen.getByRole('button', { name: 'View' }));
    expect(screen.queryByLabelText('Show terminal')).not.toBeInTheDocument();
    expect(screen.getByRole('radio', { name: /Summary/ })).toBeChecked();
    expect(screen.queryByRole('button', { name: /first-join question/ })).not.toBeInTheDocument();
    await userEvent.keyboard('{Escape}');
    expect(screen.queryByRole('radio', { name: /Summary/ })).not.toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'View' })).toHaveFocus();
  });
});
