import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { StatusPill } from './StatusPill';

describe('the status pill', () => {
  it('shows what an agent without a timeline waits on', () => {
    render(<StatusPill status="waiting" agentName="omp" request="omp needs permission: approve Write: src/main.rs" />);
    const pill = screen.getByRole('status');
    expect(pill).toHaveTextContent('Waiting for input: omp needs permission: approve Write: src/main.rs');
    expect(pill).toHaveAttribute('title', 'omp needs permission: approve Write: src/main.rs');
  });

  it('drops the request once the agent no longer waits', () => {
    render(<StatusPill status="working" agentName="omp" request="omp needs permission" />);
    expect(screen.getByRole('status')).toHaveTextContent(/^omp is working$/);
  });

  it('says a program in a terminal is working or waiting, without naming the shell', () => {
    const { rerender } = render(<StatusPill status="working" agentName="Terminal" terminal />);
    expect(screen.getByRole('status')).toHaveTextContent(/^Working$/);
    rerender(<StatusPill status="waiting" agentName="Terminal" terminal request="Claude Code asks: Tea?" />);
    expect(screen.getByRole('status')).toHaveTextContent('Waiting for input: Claude Code asks: Tea?');
    rerender(<StatusPill status="unknown" agentName="Terminal" terminal />);
    expect(screen.getByRole('status')).toHaveTextContent(/^Live$/);
    expect(screen.getByRole('status')).not.toHaveAttribute('title');
  });

  it('explains an unknown activity for an agent that does not report it', () => {
    render(<StatusPill status="unknown" agentName="OpenCode" />);
    expect(screen.getByRole('status')).toHaveAttribute('title', "OpenCode's activity is not shared");
  });
});
