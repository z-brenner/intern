import { render } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { FlaggedName } from './FlaggedName';

describe('FlaggedName', () => {
  it('marks the organization a name leads with as one of its parties', () => {
    const name = '2025-03-26 Vistage Malaysia - Employment Termination Letter for Johnny Appleseed.pdf';
    const { container } = render(<FlaggedName name={name} flagged={['parties']} />);
    expect(container).toHaveTextContent(name);
    const marks = [...container.querySelectorAll('mark.flagged')];
    expect(marks.map((mark) => mark.textContent)).toEqual(['Vistage Malaysia', 'Johnny Appleseed']);
    expect(marks[0]).toHaveAttribute('title', 'Check the parties');
  });

  it('marks the type between the organization and the other side', () => {
    const name = '2026-10-16 Vistage International - Host Chair SOW for John Smith.pdf';
    const { container } = render(<FlaggedName name={name} flagged={['type']} />);
    const marks = [...container.querySelectorAll('mark.flagged')];
    expect(marks.map((mark) => mark.textContent)).toEqual(['Host Chair SOW']);
    expect(marks[0]).toHaveAttribute('title', 'Check the document type');
  });

  it('is the plain name when nothing in it is in doubt', () => {
    const name = '2025-04-30 Nimbus Orchard Supply - Invoice.pdf';
    const { container } = render(<FlaggedName name={name} flagged={[]} />);
    expect(container.querySelector('mark')).toBeNull();
    expect(container).toHaveTextContent(name);
  });
});
