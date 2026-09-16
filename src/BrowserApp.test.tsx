import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

const picker = vi.hoisted(() => vi.fn(async () => []));

vi.mock('./lib/inMemoryBridge', async (importOriginal) => {
  const actual = await importOriginal<typeof import('./lib/inMemoryBridge')>();
  return {
    ...actual,
    createBrowserSelectionBoundary: () => ({ pickFiles: picker, pickFolder: async () => undefined, pickExistingModelFiles: async () => undefined, resolveDrop: async () => ({}) }),
  };
});

import { BrowserApp } from './BrowserApp';

describe('BrowserApp', () => {
  it('starts the fixture E2E adapter empty so dropped files drive the run', async () => {
    window.history.replaceState({}, '', '/?fixtureBatch=1');

    render(<BrowserApp />);

    expect(await screen.findByText('0 items')).toBeVisible();
    expect(screen.queryByText('Lease Agreement - 123 Main St.pdf')).not.toBeInTheDocument();
    window.history.replaceState({}, '', '/');
  });

  it('runs guided onboarding against the fake SharePoint deployment only when asked', async () => {
    window.history.replaceState({}, '', '/?sharePoint=fake');

    render(<BrowserApp />);

    expect(await screen.findByRole('button', { name: 'Set up Intern' })).toBeVisible();
    expect(screen.queryByRole('main', { name: 'Intern' })).not.toBeInTheDocument();
    window.history.replaceState({}, '', '/');
  });

  it('opens the ordinary demo app by default, with no onboarding', async () => {
    render(<BrowserApp />);

    expect(await screen.findByRole('main', { name: 'Intern' })).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: 'Set up Intern' })).not.toBeInTheDocument();
  });

  it('injects the browser selection boundary into the platform-neutral App', async () => {
    render(<BrowserApp />);

    fireEvent.click(await screen.findByRole('button', { name: 'Add files' }));

    await waitFor(() => expect(picker).toHaveBeenCalledOnce());
  });
});
