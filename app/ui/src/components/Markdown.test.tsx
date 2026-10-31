import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { Markdown } from './Markdown';

describe('markdown', () => {
  it('leaves out HTML comments, such as the notes in OpenSpec templates', () => {
    const { container } = render(<Markdown text={'### New Capabilities\n\n<!-- None: documentation only. -->\n\nText <!-- inline --> after.'} />);
    expect(container).not.toHaveTextContent('<!--');
    expect(container).not.toHaveTextContent('documentation only');
    expect(screen.getByText(/Text\s+after\./)).toBeInTheDocument();
  });

  it('keeps other angle-bracket text the agent wrote', () => {
    const { container } = render(<Markdown text={'Try "write a test for <filepath>"'} />);
    expect(container).toHaveTextContent('write a test for <filepath>');
  });
});
