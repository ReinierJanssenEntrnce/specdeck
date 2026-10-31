import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { formatTokens, UsageMeter } from './UsageMeter';

const tokens = { input: 20, output: 4_000, cacheRead: 1_150_000, cacheWrite: 60_000 };

describe('usage meter', () => {
  it('shows how full the context is and the session total', () => {
    render(<UsageMeter usage={{ available: true, tokens, total: 1_214_020, contextTokens: 50_000, contextWindow: 200_000 }} />);
    expect(screen.getByRole('meter', { name: 'Context window' })).toHaveAttribute('aria-valuenow', '25');
    expect(screen.getByText('25% context')).toBeInTheDocument();
    expect(screen.getByText('1.2M tokens')).toBeInTheDocument();
    const details = screen.getByText('25% context').parentElement!.title;
    expect(details).toContain('Context: 50,000 of 200,000 tokens (25%)');
    expect(details).toContain('Cache read 1,150,000');
  });

  it('warns when the context is nearly full', () => {
    const { container } = render(<UsageMeter usage={{ available: true, tokens, total: 1, contextTokens: 190_000, contextWindow: 200_000 }} />);
    expect(container.firstElementChild).toHaveClass('usage', 'full');
  });

  it('shows the context in tokens when the window size is unknown', () => {
    render(<UsageMeter usage={{ available: true, tokens, total: 1_214_020, contextTokens: 12_300 }} />);
    expect(screen.queryByRole('meter')).toBeNull();
    expect(screen.getByText('12.3k in context')).toBeInTheDocument();
  });

  it('says when usage cannot be read', () => {
    render(<UsageMeter usage={{ available: false, reason: 'The transcript was not found' }} />);
    expect(screen.getByText('Usage unavailable').closest('span')).toHaveAttribute('title', expect.stringContaining('The transcript was not found'));
  });

  it('shows nothing before the agent reports usage', () => {
    const { container } = render(<UsageMeter usage={null} />);
    expect(container).toBeEmptyDOMElement();
  });

  it('formats token counts compactly', () => {
    expect([950, 1_000, 12_345, 245_600, 1_000_000, 1_250_000].map(formatTokens)).toEqual(['950', '1k', '12.3k', '246k', '1M', '1.3M']);
  });
});
