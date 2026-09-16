import { act, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { App } from '../../App';
import type { DesktopBridge } from '../../lib/bridge';
import { PINNED_MODEL_BYTES, createInMemoryBridge } from '../../lib/inMemoryBridge';
import type { InMemoryBridgeOptions } from '../../lib/inMemoryBridge';

type FakeSharePoint = NonNullable<InMemoryBridgeOptions['sharePointFake']>;

const account = { displayName: 'Pat Lee', email: 'pat.lee@contoso.test' };

function enabledBridge(fake: FakeSharePoint = {}, options: InMemoryBridgeOptions = {}) {
  return createInMemoryBridge({ sharePoint: 'fake', ...options, sharePointFake: { account, ...fake } });
}

async function onboarding() {
  await screen.findByRole('button', { name: 'Set up Intern' });
  return screen.getByRole('main', { name: 'Intern setup' });
}

async function begin() {
  fireEvent.click(await screen.findByRole('button', { name: 'Set up Intern' }));
}

/** Connect is enabled once the Microsoft status has loaded; a click before then does nothing. */
async function connect() {
  const button = await screen.findByRole('button', { name: 'Connect Microsoft account' });
  await waitFor(() => expect(button).toBeEnabled());
  fireEvent.click(button);
}

/** Focus moves in an effect after the step renders. */
async function expectFocused(element: HTMLElement) {
  await waitFor(() => expect(element).toHaveFocus());
}

async function confirmAccount() {
  fireEvent.click(await screen.findByRole('button', { name: 'Yes, this is my account' }));
}

/** Welcome through Finished for an account and library that are already in place. */
async function walkToFinished() {
  await begin();
  await confirmAccount();
  fireEvent.click(await screen.findByRole('button', { name: 'Turn on filing' }));
  await screen.findByRole('heading', { name: 'Watching Files/Inbox. Filing your documents into Files/Filed.' });
}

function expectNoIdentifierFields() {
  expect(screen.queryAllByRole('textbox')).toHaveLength(0);
  expect(screen.queryByLabelText(/tenant|client|drive|folder|email|path|site/i)).not.toBeInTheDocument();
}

afterEach(() => {
  vi.useRealTimers();
});

describe('guided onboarding routing', () => {
  it('leaves today\'s app untouched when the SharePoint deployment is disabled', async () => {
    const bridge = createInMemoryBridge();
    render(<App bridge={bridge} />);

    expect(await screen.findByRole('main', { name: 'Intern' })).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: 'Set up Intern' })).not.toBeInTheDocument();
  });

  it('keeps the existing model setup screen when the deployment is disabled', async () => {
    render(<App bridge={createInMemoryBridge({ setup: { state: 'required', downloadedBytes: 0, totalBytes: PINNED_MODEL_BYTES } })} />);

    expect(await screen.findByRole('button', { name: 'Download model' })).toBeVisible();
    expect(screen.queryByRole('button', { name: 'Set up Intern' })).not.toBeInTheDocument();
    expect(screen.queryByRole('list', { name: 'Setup steps' })).not.toBeInTheDocument();
  });

  it('never routes to onboarding when the backend reports it required without a deployment', async () => {
    const base = createInMemoryBridge();
    const bridge: DesktopBridge = { ...base, getOnboarding: async () => ({ currentVersion: 1, completedVersion: 0, required: true, sharePointAvailable: false }) };
    render(<App bridge={bridge} />);

    expect(await screen.findByRole('main', { name: 'Intern' })).toBeInTheDocument();
  });

  it('shows an inert loading state until onboarding status settles', () => {
    const base = enabledBridge();
    const listItems = vi.spyOn(base, 'listItems');
    render(<App bridge={{ ...base, getOnboarding: () => new Promise<never>(() => {}) }} />);

    expect(screen.getByRole('status', { name: 'Loading setup' })).toBeVisible();
    expect(screen.queryByRole('button')).not.toBeInTheDocument();
    expect(listItems).not.toHaveBeenCalled();
  });

  it('skips onboarding once the current version is completed', async () => {
    render(<App bridge={enabledBridge({ connected: true, phase: 'active' }, { completedOnboardingVersion: 1 })} />);

    expect(await screen.findByRole('main', { name: 'Intern' })).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: 'Set up Intern' })).not.toBeInTheDocument();
  });

  it('shows a legacy install the new workflow once, then opens the app directly', async () => {
    const bridge = enabledBridge({ connected: true, phase: 'ready_to_activate' }, { completedOnboardingVersion: 0 });
    const first = render(<App bridge={bridge} />);

    await onboarding();
    await walkToFinished();
    fireEvent.click(screen.getByRole('button', { name: 'Open Intern' }));
    expect(await screen.findByRole('main', { name: 'Intern' })).toBeInTheDocument();
    first.unmount();

    render(<App bridge={bridge} />);
    expect(await screen.findByRole('main', { name: 'Intern' })).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: 'Set up Intern' })).not.toBeInTheDocument();
  });

  it('does not read or subscribe to the queue until onboarding completes', async () => {
    const bridge = enabledBridge({ connected: true, phase: 'ready_to_activate' });
    const listItems = vi.spyOn(bridge, 'listItems');
    render(<App bridge={bridge} />);

    await walkToFinished();
    expect(listItems).not.toHaveBeenCalled();

    fireEvent.click(screen.getByRole('button', { name: 'Open Intern' }));
    await screen.findByRole('main', { name: 'Intern' });
    await waitFor(() => expect(listItems).toHaveBeenCalled());
  });

  it('offers a plain retry when onboarding status cannot be read, instead of entering the app', async () => {
    const bridge = enabledBridge();
    const listItems = vi.spyOn(bridge, 'listItems');
    vi.spyOn(bridge, 'getOnboarding').mockRejectedValueOnce({ code: 'ONBOARDING_STATE_UNREADABLE', message: 'ui-state.json is corrupt' });
    render(<App bridge={bridge} />);

    const alert = await screen.findByRole('alert');
    expect(alert).toHaveTextContent(/try again/i);
    expect(screen.getByText('ONBOARDING_STATE_UNREADABLE')).toBeInTheDocument();
    expect(screen.queryByRole('main', { name: 'Intern' })).not.toBeInTheDocument();
    expect(listItems).not.toHaveBeenCalled();

    fireEvent.click(screen.getByRole('button', { name: 'Try again' }));
    expect(await screen.findByRole('button', { name: 'Set up Intern' })).toBeVisible();
  });
});

describe('guided onboarding steps', () => {
  it('explains what Intern does before asking for anything', async () => {
    render(<App bridge={enabledBridge()} />);

    const main = await onboarding();
    await expectFocused(within(main).getByRole('heading', { level: 1 }));
    expect(main).toHaveTextContent(/watches your team's Inbox/i);
    expect(main).toHaveTextContent(/privately on this computer/i);
    expect(main).toHaveTextContent(/moves it to Filed/i);
    expect(main).toHaveTextContent(/only documents you upload/i);
    expect(within(main).getByRole('list', { name: 'Setup steps' })).toBeInTheDocument();
    expect(within(main).getAllByRole('button')).toHaveLength(1);
    expectNoIdentifierFields();
  });

  it('downloads the local model as a step and moves on when it is ready', async () => {
    render(<App bridge={enabledBridge({}, { setup: { state: 'required', downloadedBytes: 0, totalBytes: PINNED_MODEL_BYTES }, downloadIntervalMs: 5 })} />);

    await begin();
    const heading = await screen.findByRole('heading', { name: 'Get the local model' });
    await expectFocused(heading);
    const advanced = screen.getByText('Other ways to get the model');
    expect(advanced.closest('details')).not.toHaveAttribute('open');
    expect(screen.getByRole('button', { name: 'Choose existing model files' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Use a hosted model instead' })).toBeInTheDocument();

    fireEvent.click(screen.getByRole('button', { name: 'Download model' }));
    await expectFocused(await screen.findByRole('heading', { name: 'Connect your Microsoft account' }, { timeout: 3000 }));
  });

  it('resumes a partial model download', async () => {
    render(<App bridge={enabledBridge({}, { setup: { state: 'required', downloadedBytes: PINNED_MODEL_BYTES / 2, totalBytes: PINNED_MODEL_BYTES } })} />);

    await begin();
    expect(await screen.findByRole('button', { name: 'Resume download' })).toBeVisible();
  });

  it('skips the model step when the model is already ready', async () => {
    render(<App bridge={enabledBridge()} />);

    await begin();
    expect(await screen.findByRole('heading', { name: 'Connect your Microsoft account' })).toBeVisible();
    expect(screen.queryByRole('heading', { name: 'Get the local model' })).not.toBeInTheDocument();
  });

  it('signs in with a device code and asks the person to confirm the verified account', async () => {
    vi.useFakeTimers({ shouldAdvanceTime: true });
    const bridge = enabledBridge({ signInPolls: 1 });
    const open = vi.spyOn(bridge, 'microsoftOpenSignIn');
    render(<App bridge={bridge} />);

    await begin();
    await connect();
    const waiting = await screen.findByRole('status', { name: 'Microsoft sign-in' });
    expect(within(screen.getByRole('main')).getByText('ABCD-EFGH')).toBeVisible();
    expect(waiting).toHaveAttribute('aria-live', 'polite');
    fireEvent.click(screen.getByRole('button', { name: 'Open Microsoft sign-in' }));
    await waitFor(() => expect(open).toHaveBeenCalledWith());

    await act(async () => { await vi.advanceTimersByTimeAsync(5000); });
    expect(await screen.findByText('Pat Lee')).toBeVisible();
    expect(screen.getByText('pat.lee@contoso.test')).toBeVisible();
    expect(screen.getByText(/Is this the account you use to upload documents/i)).toBeVisible();
    expect(screen.getByRole('button', { name: 'Yes, this is my account' })).toBeVisible();
    expectNoIdentifierFields();
  });

  it('disconnects when the person wants a different account', async () => {
    const bridge = enabledBridge({ connected: true });
    const disconnect = vi.spyOn(bridge, 'microsoftDisconnect');
    render(<App bridge={bridge} />);

    await begin();
    fireEvent.click(await screen.findByRole('button', { name: 'Use a different account' }));
    expect(await screen.findByRole('button', { name: 'Connect Microsoft account' })).toBeVisible();
    expect(disconnect).toHaveBeenCalledOnce();
  });

  it('resumes from the backend state: an account and library that already exist go straight to activation', async () => {
    const bridge = enabledBridge({ connected: true, phase: 'ready_to_activate' });
    const start = vi.spyOn(bridge, 'microsoftSignInStart');
    const sync = vi.spyOn(bridge, 'startSharePointSync');
    render(<App bridge={bridge} />);

    await begin();
    await confirmAccount();
    await expectFocused(await screen.findByRole('heading', { name: 'Turn on filing' }));
    expect(start).not.toHaveBeenCalled();
    expect(sync).not.toHaveBeenCalled();
  });

  it('skips activation when the library is already active', async () => {
    render(<App bridge={enabledBridge({ connected: true, phase: 'active' })} />);

    await begin();
    await confirmAccount();
    expect(await screen.findByRole('heading', { name: 'Watching Files/Inbox. Filing your documents into Files/Filed.' })).toBeVisible();
  });

  it('asks OneDrive to sync the library, then rescans until it appears', async () => {
    vi.useFakeTimers({ shouldAdvanceTime: true });
    const bridge = enabledBridge({ connected: true, phase: 'enrollment_pending', pendingRescans: 2 });
    const sync = vi.spyOn(bridge, 'startSharePointSync');
    const rescan = vi.spyOn(bridge, 'getSharePointSetup');
    const openSupportLink = vi.spyOn(bridge, 'openSupportLink').mockResolvedValue();
    render(<App bridge={bridge} />);

    await begin();
    await confirmAccount();
    await expectFocused(await screen.findByRole('heading', { name: 'Sync the Files library' }));
    fireEvent.click(screen.getByRole('button', { name: 'Sync Files with OneDrive' }));
    await waitFor(() => expect(sync).toHaveBeenCalledOnce());

    const status = await screen.findByRole('status', { name: 'Library sync' });
    expect(status).toHaveTextContent(/waiting for OneDrive/i);
    expect(screen.getByText(/OneDrive may ask you to confirm/i)).toBeVisible();
    expect(screen.getByRole('button', { name: 'Try again' })).toBeVisible();
    // A link would do nothing inside the desktop app; the bridge hands the fixed site to the shell.
    expect(screen.queryByRole('link')).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: 'Open SharePoint' }));
    await waitFor(() => expect(openSupportLink).toHaveBeenCalledWith('sharepoint-site'));

    const before = rescan.mock.calls.length;
    await act(async () => { await vi.advanceTimersByTimeAsync(5000); });
    await waitFor(() => expect(rescan.mock.calls.length).toBeGreaterThan(before));
    expect(screen.getByRole('status', { name: 'Library sync' })).toBeInTheDocument();
    // Two rescans still pending; the third finds the library.
    await act(async () => { await vi.advanceTimersByTimeAsync(5000); });
    expect(screen.getByRole('status', { name: 'Library sync' })).toBeInTheDocument();
    await act(async () => { await vi.advanceTimersByTimeAsync(5000); });
    expect(await screen.findByRole('heading', { name: 'Turn on filing' })).toBeVisible();
  });

  it('stops polling Microsoft and rescanning once the window content unmounts', async () => {
    vi.useFakeTimers({ shouldAdvanceTime: true });
    const bridge = enabledBridge({ signInPolls: 99 });
    const poll = vi.spyOn(bridge, 'microsoftSignInPoll');
    const disconnect = vi.spyOn(bridge, 'microsoftDisconnect');
    const view = render(<App bridge={bridge} />);

    await begin();
    await connect();
    await screen.findByRole('status', { name: 'Microsoft sign-in' });
    view.unmount();
    expect(disconnect).toHaveBeenCalledOnce();

    await act(async () => { await vi.advanceTimersByTimeAsync(30_000); });
    expect(poll).not.toHaveBeenCalled();
  });

  it('activates, then summarizes what Intern is doing and for whom', async () => {
    const bridge = enabledBridge({ connected: true, phase: 'ready_to_activate' });
    render(<App bridge={bridge} />);

    await walkToFinished();
    const main = screen.getByRole('main', { name: 'Intern setup' });
    await expectFocused(screen.getByRole('heading', { name: 'Watching Files/Inbox. Filing your documents into Files/Filed.' }));
    expect(main).toHaveTextContent('Pat Lee');
    expect(main).toHaveTextContent('pat.lee@contoso.test');
    expect(main).toHaveTextContent(/starts automatically/i);
    expect(main).toHaveTextContent(/system tray/i);
    expect(within(main).getAllByRole('button')).toHaveLength(1);
    expectNoIdentifierFields();
  });
});

describe('guided onboarding failures', () => {
  it('asks for the work account when the connected account is from another organization', async () => {
    const bridge = enabledBridge({ connected: true, failures: { getSharePointSetup: [{ code: 'MICROSOFT_ACCOUNT_WRONG_TENANT', message: 'The connected Microsoft account is outside the provisioned Contoso tenant.' }] } });
    const disconnect = vi.spyOn(bridge, 'microsoftDisconnect');
    render(<App bridge={bridge} />);

    await begin();
    await confirmAccount();
    const alert = await screen.findByRole('alert');
    expect(alert).toHaveTextContent(/work account/);
    expect(screen.getByText('MICROSOFT_ACCOUNT_WRONG_TENANT')).toBeInTheDocument();

    fireEvent.click(screen.getByRole('button', { name: 'Use a different account' }));
    expect(await screen.findByRole('button', { name: 'Connect Microsoft account' })).toBeVisible();
    expect(disconnect).toHaveBeenCalledOnce();
  });

  it('explains blocked organization consent and stays on the connection step', async () => {
    vi.useFakeTimers({ shouldAdvanceTime: true });
    const bridge = enabledBridge({ failures: { microsoftSignInPoll: ['Microsoft sign-in was declined, expired, or blocked by organization policy. Files remain held.'] } });
    render(<App bridge={bridge} />);

    await begin();
    await connect();
    await screen.findByRole('status', { name: 'Microsoft sign-in' });
    await act(async () => { await vi.advanceTimersByTimeAsync(5000); });

    const alert = await screen.findByRole('alert');
    expect(alert).toHaveTextContent(/organization/i);
    expect(alert).toHaveTextContent(/cannot safely watch the shared Inbox/i);
    expect(screen.getByRole('heading', { name: 'Connect your Microsoft account' })).toBeVisible();
    expect(screen.getByRole('button', { name: 'Connect Microsoft account' })).toBeEnabled();
  });

  it('recognizes the backend consent-blocked token in a sign-in failure', async () => {
    vi.useFakeTimers({ shouldAdvanceTime: true });
    const message = 'Your Microsoft organization has blocked Intern from connecting. Ask your IT administrator to allow Intern. (MICROSOFT_CONSENT_BLOCKED)';
    const bridge = enabledBridge({ failures: { microsoftSignInPoll: [message] } });
    render(<App bridge={bridge} />);

    await begin();
    await connect();
    await screen.findByRole('status', { name: 'Microsoft sign-in' });
    await act(async () => { await vi.advanceTimersByTimeAsync(5000); });

    expect(await screen.findByRole('alert')).toHaveTextContent(/organization blocked/i);
    expect(screen.getByText('MICROSOFT_CONSENT_BLOCKED')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Connect Microsoft account' })).toBeEnabled();
  });

  it('maps an explicit consent-blocked code to the organization explanation', async () => {
    const bridge = enabledBridge({ failures: { microsoftSignInStart: [{ code: 'MICROSOFT_CONSENT_BLOCKED', message: 'AADSTS65001' }] } });
    render(<App bridge={bridge} />);

    await begin();
    await connect();
    expect(await screen.findByRole('alert')).toHaveTextContent(/organization blocked/i);
    expect(screen.getByText('MICROSOFT_CONSENT_BLOCKED')).toBeInTheDocument();
  });

  it('tells the person to install or open OneDrive and never claims sync is active', async () => {
    const bridge = enabledBridge({ connected: true, failures: { startSharePointSync: [{ code: 'ONEDRIVE_MISSING', message: 'OneDrive is not installed.' }] } });
    const openSupportLink = vi.spyOn(bridge, 'openSupportLink').mockResolvedValue();
    render(<App bridge={bridge} />);

    await begin();
    await confirmAccount();
    fireEvent.click(await screen.findByRole('button', { name: 'Sync Files with OneDrive' }));
    const alert = await screen.findByRole('alert');
    expect(alert).toHaveTextContent(/^Install or open OneDrive/);
    expect(screen.getByText('ONEDRIVE_MISSING')).toBeInTheDocument();
    expect(screen.queryByRole('status', { name: 'Library sync' })).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: 'Get OneDrive' }));
    await waitFor(() => expect(openSupportLink).toHaveBeenCalledWith('onedrive-download'));
    expect(screen.getByRole('alert')).toBe(alert);
  });

  it('gives the address to type when the system browser cannot be opened', async () => {
    const bridge = enabledBridge({ connected: true, failures: { startSharePointSync: [{ code: 'ONEDRIVE_MISSING', message: 'OneDrive is not installed.' }] } });
    vi.spyOn(bridge, 'openSupportLink').mockRejectedValue(new Error('opener denied'));
    render(<App bridge={bridge} />);

    await begin();
    await confirmAccount();
    fireEvent.click(await screen.findByRole('button', { name: 'Sync Files with OneDrive' }));
    fireEvent.click(await screen.findByRole('button', { name: 'Get OneDrive' }));
    expect(await screen.findByText(/could not be opened.*https:\/\/www\.microsoft\.com\/microsoft-365\/onedrive\/download/)).toBeVisible();
    expect(screen.getByText('ONEDRIVE_MISSING')).toBeInTheDocument();
  });

  it('reports activation failure plainly and lets the person try again', async () => {
    const bridge = enabledBridge({ connected: true, phase: 'ready_to_activate', failures: { activateOnboarding: [{ code: 'FILED_UNWRITABLE', message: 'Filed is read-only' }] } });
    render(<App bridge={bridge} />);

    await begin();
    await confirmAccount();
    fireEvent.click(await screen.findByRole('button', { name: 'Turn on filing' }));
    const alert = await screen.findByRole('alert');
    expect(alert).toHaveTextContent(/Files\/Filed/);
    expect(screen.getByText('FILED_UNWRITABLE')).toBeInTheDocument();

    fireEvent.click(screen.getByRole('button', { name: 'Turn on filing' }));
    expect(await screen.findByRole('heading', { name: 'Watching Files/Inbox. Filing your documents into Files/Filed.' })).toBeVisible();
  });

  it('does not enter the app when completion cannot be saved, and retries', async () => {
    const bridge = enabledBridge({ connected: true, phase: 'ready_to_activate', failures: { completeOnboarding: [{ code: 'ONBOARDING_STATE_WRITE_FAILED', message: 'could not write ui-state.json' }] } });
    const listItems = vi.spyOn(bridge, 'listItems');
    render(<App bridge={bridge} />);

    await walkToFinished();
    fireEvent.click(screen.getByRole('button', { name: 'Open Intern' }));
    expect(await screen.findByRole('alert')).toHaveTextContent(/try again/i);
    expect(screen.getByText('ONBOARDING_STATE_WRITE_FAILED')).toBeInTheDocument();
    expect(screen.queryByRole('main', { name: 'Intern' })).not.toBeInTheDocument();
    expect(listItems).not.toHaveBeenCalled();

    fireEvent.click(screen.getByRole('button', { name: 'Open Intern' }));
    expect(await screen.findByRole('main', { name: 'Intern' })).toBeInTheDocument();
  });

  it('falls back to plain language for an unknown code and keeps the code for support', async () => {
    const bridge = enabledBridge({ connected: true, failures: { getSharePointSetup: [{ code: 'SOMETHING_NEW', message: 'raw backend detail' }] } });
    render(<App bridge={bridge} />);

    await begin();
    await confirmAccount();
    const alert = await screen.findByRole('alert');
    expect(alert).toHaveTextContent(/^Try again/);
    expect(alert).not.toHaveTextContent('raw backend detail');
    expect(screen.getByText('SOMETHING_NEW')).toBeInTheDocument();
  });

  it('keeps the verifier gap honest instead of pretending the library is ready', async () => {
    const bridge = enabledBridge({ connected: true, failures: { getSharePointSetup: [{ code: 'SHAREPOINT_ROOT_VERIFIER_UNAVAILABLE', message: 'verifier unavailable' }] } });
    render(<App bridge={bridge} />);

    await begin();
    await confirmAccount();
    expect(await screen.findByRole('alert')).toHaveTextContent(/support/i);
    expect(screen.queryByRole('button', { name: 'Turn on filing' })).not.toBeInTheDocument();
  });
});
