import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { StatusBar } from './StatusBar';

describe('status bar', () => {
  it('shows the model, folder, branch, context and rate limits', () => {
    const inTwoHours = Date.now() / 1000 + 2 * 3600 + 14 * 60 + 30;
    render(
      <StatusBar
        info={{
          model: 'Opus 5.5',
          effort: 'medium',
          fast: false,
          folder: 'webshop',
          git: { branch: 'main', dirty: true, ahead: 2, behind: 1 },
          contextTokens: 178_000,
          contextWindow: 1_000_000,
          total: 140_200_000,
          fiveHour: { usedPercent: 23, resetsAt: inTwoHours },
          sevenDay: { usedPercent: 85 },
        }}
      />,
    );
    const bar = screen.getByRole('contentinfo', { name: 'Session details' });
    expect(bar).toHaveTextContent('Opus 5.51Mmedium');
    expect(bar).toHaveTextContent('webshop');
    expect(bar).toHaveTextContent('main●↑2 ↓1');
    expect(screen.getByRole('meter', { name: 'Context window' })).toHaveAttribute('aria-valuenow', '18');
    expect(bar).toHaveTextContent('17.8%178k/1M');
    expect(bar).toHaveTextContent('140.2Mtokens');
    expect(bar).toHaveTextContent('23%2h14m');
    expect(screen.getByRole('meter', { name: '7-day limit' })).toHaveClass('hot');
  });

  it('picks out the issue key in the branch', () => {
    render(<StatusBar info={{ fast: false, git: { branch: 'feature/PROJ-2287_fix', dirty: false, ahead: 0, behind: 0 } }} />);
    expect(screen.getByText('PROJ-2287')).toHaveClass('key');
  });

  it('shows only the folder and git state for an agent without figures', () => {
    render(<StatusBar info={{ fast: false, folder: 'notes', git: { branch: 'main', dirty: false, ahead: 0, behind: 0 } }} />);
    expect(screen.getByRole('contentinfo')).toHaveTextContent(/^notesmain$/);
    expect(screen.queryByRole('meter')).toBeNull();
  });

  it('falls back to the folder it was given until the session reports one', () => {
    render(<StatusBar info={{ fast: false }} folder="webshop" />);
    expect(screen.getByRole('contentinfo')).toHaveTextContent('webshop');
  });

  it('says when usage cannot be read', () => {
    render(<StatusBar info={{ fast: false, total: 5, usageUnavailable: 'not found' }} />);
    expect(screen.getByText('usage unavailable')).toHaveAttribute('title', expect.stringContaining('not found'));
    expect(screen.queryByText('tokens')).toBeNull();
  });
});
