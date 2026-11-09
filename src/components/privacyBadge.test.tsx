import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { PrivacyBadge } from './PrivacyBadge';
import type { PrivacyFacts } from './PrivacyBadge';

const local: PrivacyFacts = { modelDownloaded: true, automaticUpdateChecks: true };

describe('the privacy badge', () => {
  it('opens to the list of what Intern sends, and closes with Escape back to itself', () => {
    render(<PrivacyBadge facts={local} onOpenSettings={vi.fn()} onOpenGuide={vi.fn()} />);
    const badge = screen.getByRole('button', { name: /Private · On this device/ });
    expect(badge).toHaveAttribute('aria-expanded', 'false');

    fireEvent.click(badge);
    const panel = screen.getByRole('dialog', { name: 'What Intern sends' });
    expect(badge).toHaveAttribute('aria-expanded', 'true');
    expect(panel).toHaveTextContent('Your documents stay on this computer');
    expect(panel).toHaveTextContent('you choose what to share in your browser');
    expect(panel).toHaveTextContent(/Model download\s*Done/);
    expect(panel).toHaveTextContent(/Update checks\s*Automatic/);

    fireEvent.keyDown(panel, { key: 'Escape' });
    expect(screen.queryByRole('dialog', { name: 'What Intern sends' })).not.toBeInTheDocument();
    expect(badge).toHaveFocus();
  });

  it('hands Settings the badge, so closing Settings comes back to it', () => {
    const onOpenSettings = vi.fn();
    render(<PrivacyBadge facts={local} onOpenSettings={onOpenSettings} onOpenGuide={vi.fn()} />);
    const badge = screen.getByRole('button', { name: /On this device/ });
    fireEvent.click(badge);
    fireEvent.click(screen.getByRole('button', { name: 'Change in Settings' }));
    expect(onOpenSettings).toHaveBeenCalledWith(badge);
    expect(screen.queryByRole('dialog', { name: 'What Intern sends' })).not.toBeInTheDocument();
  });
});
