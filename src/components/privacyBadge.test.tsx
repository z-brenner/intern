import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { PrivacyBadge } from './PrivacyBadge';
import type { PrivacyFacts } from './PrivacyBadge';

const local: PrivacyFacts = { hosted: false, modelDownloaded: true, automaticUpdateChecks: true };

describe('the privacy badge', () => {
  it('opens to the list of what Intern sends, and closes with Escape back to itself', () => {
    render(<PrivacyBadge facts={local} onOpenSettings={vi.fn()} onOpenGuide={vi.fn()} />);
    const badge = screen.getByRole('button', { name: /Private · On this device/ });
    expect(badge).toHaveAttribute('aria-expanded', 'false');

    fireEvent.click(badge);
    const panel = screen.getByRole('dialog', { name: 'What Intern sends' });
    expect(badge).toHaveAttribute('aria-expanded', 'true');
    expect(panel).toHaveTextContent('Your documents stay on this computer');
    expect(panel).toHaveTextContent(/Hosted model\s*Off/);
    expect(panel).toHaveTextContent(/Model download\s*Done/);
    expect(panel).toHaveTextContent(/Update checks\s*Automatic/);

    fireEvent.keyDown(panel, { key: 'Escape' });
    expect(screen.queryByRole('dialog', { name: 'What Intern sends' })).not.toBeInTheDocument();
    expect(badge).toHaveFocus();
  });

  it('says the opposite, plainly, while a hosted model is on', () => {
    render(<PrivacyBadge facts={{ ...local, hosted: true, hostedService: 'Anthropic', automaticUpdateChecks: false }} onOpenSettings={vi.fn()} onOpenGuide={vi.fn()} />);
    fireEvent.click(screen.getByRole('button', { name: /Hosted model · Text leaves this device/ }));
    const panel = screen.getByRole('dialog', { name: 'What Intern sends' });
    expect(panel).toHaveTextContent('Document text is leaving this computer');
    expect(panel).toHaveTextContent('sent to Anthropic');
    expect(panel).toHaveTextContent(/Update checks\s*Off/);
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
