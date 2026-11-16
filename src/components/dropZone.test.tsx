import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { DropZone } from './DropZone';

describe('the drop target', () => {
  it('names the input folder a drop is copied into', () => {
    render(<DropZone variant="hero" inputFolder="Input" outputFolder="Output" onDrop={() => {}} />);
    expect(screen.getByRole('region', { name: 'Drag files or folders here to copy them into Input' })).toBeVisible();
    expect(screen.getByText(/Nothing is renamed until you approve it\. Your originals stay where they are\./)).toBeVisible();
    expect(screen.queryByText(/Microsoft confirms/)).not.toBeInTheDocument();
  });

  // With automatic renaming on, what is renamed without asking is what is
  // ready, which the review level in Settings decides: at Balanced that is
  // more than the documents Intern is sure about, so the line says "ready".
  it('says ready documents are renamed on their own when automatic renaming is on', () => {
    render(<DropZone variant="hero" inputFolder="Input" outputFolder="Output" automaticRename onDrop={() => {}} />);
    expect(screen.getByText(/Ready documents are renamed on their own; the rest wait for you to approve them\. Your originals stay where they are\./)).toBeVisible();
    expect(screen.queryByText(/sure about|Nothing is renamed until/)).not.toBeInTheDocument();
  });

  // Only the verified team Inbox waits for Microsoft; a build that merely
  // carries the deployment, watching some other folder, does not.
  it('says a copy waits for Microsoft only for the verified team Inbox', () => {
    render(<DropZone variant="hero" inputFolder="Inbox" verifiedInbox onDrop={() => {}} />);
    expect(screen.getByText('A copy joins the queue once OneDrive has uploaded it and Microsoft confirms it is yours.')).toBeVisible();
  });
});
