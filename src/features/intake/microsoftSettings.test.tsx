import { act, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { createInMemoryBridge } from '../../lib/inMemoryBridge';
import { MicrosoftIntakeSettings } from './MicrosoftIntakeSettings';
import type { MicrosoftIntakeBridge, MicrosoftIntakeStatus, MicrosoftAccount } from './microsoft';
const account: MicrosoftAccount = { tenantId: 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', id: 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', displayName: 'Zachary Brenner', email: 'zack@example.test' };
const initial: MicrosoftIntakeStatus = { connected: false, account: null, binding: null, documents: [], error: null };
function bridge(status = initial) {
  let current = { ...status };
  const microsoft: MicrosoftIntakeBridge = {
    microsoftIntakeStatus: vi.fn(async () => current),
    microsoftSignInStart: vi.fn(async () => ({ userCode: 'ABCD-EFGH', verificationUri: 'https://microsoft.com/devicelogin', intervalSeconds: 5, expiresAt: 9999999999 })),
    microsoftSignInPoll: vi.fn(async () => ({ state: 'pending' as const, intervalSeconds: 5 })),
    microsoftDisconnect: vi.fn(async () => { current = { ...current, connected: false, account: null }; }),
    microsoftBindIntake: vi.fn(async () => {
      const binding = { localFolder: 'C:/Intake', driveId: 'fixed-drive', folderId: 'fixed-inbox', webUrl: 'https://teamcontoso.sharepoint.com/sites/InternTestSite/Files/Inbox', tenantId: account.tenantId };
      current = { ...current, binding }; return binding;
    }),
    microsoftOpenSignIn: vi.fn(async () => {}),
  };
  return { ...createInMemoryBridge(), ...microsoft };
}
describe('Microsoft upload identity setup', () => {
  it('explains that manual pairing is off in a build that sets up SharePoint itself, whichever shape the refusal takes', async () => {
    for (const refusal of [{ code: 'MICROSOFT_MANUAL_PAIRING_DISABLED', message: 'manual pairing is disabled' }, 'Manual folder pairing is disabled for this deployment. (MICROSOFT_MANUAL_PAIRING_DISABLED)']) {
      const api = bridge({ ...initial, connected: true, account });
      vi.mocked(api.microsoftBindIntake).mockRejectedValueOnce(refusal);
      const view = render(<MicrosoftIntakeSettings bridge={api} savedFolder="C:/Intake" unsavedFolder={false} />);
      const pair = await screen.findByRole('button', { name: 'Verify folder pairing' });
      await waitFor(() => expect(pair).toBeEnabled());
      fireEvent.click(pair);
      const alert = await screen.findByRole('alert');
      expect(alert).toHaveTextContent(/use the SharePoint connection/i);
      expect(alert).toHaveTextContent('(MICROSOFT_MANUAL_PAIRING_DISABLED)');
      view.unmount();
    }
  });
  // Every shipping build lacks the deployment. The panel used to show the
  // backend's sentence as a red error, a Connect button that could never be
  // enabled, steps starting at 2, and a "browser preview" hint in the
  // installed app; now it says one true thing.
  it('says plainly that verification is not part of a build without the deployment', async () => {
    const unavailable = 'SharePoint deployment configuration is unavailable: provisioned identifiers are not available in this build.';
    render(<MicrosoftIntakeSettings bridge={bridge({ ...initial, error: unavailable })} savedFolder="C:/Intake" unsavedFolder={false} />);

    expect(await screen.findByRole('status', { name: 'Microsoft connection status' })).toHaveTextContent('Microsoft upload verification is not part of this build.');
    expect(screen.queryByRole('group', { name: 'Microsoft upload identity' })).not.toBeInTheDocument();
    expect(screen.queryByRole('button', { name: 'Connect my Microsoft account' })).not.toBeInTheDocument();
    expect(screen.queryByText(/deployment configuration/i)).not.toBeInTheDocument();
    expect(screen.queryByText(/browser preview/i)).not.toBeInTheDocument();
    expect(screen.queryByRole('alert')).not.toBeInTheDocument();
  });

  it('keeps any other failure as persistent status, numbered from the first step', async () => {
    const broken = 'Private Microsoft upload snapshots are unavailable.';
    render(<MicrosoftIntakeSettings bridge={bridge({ ...initial, error: broken })} savedFolder="C:/Intake" unsavedFolder={false} />);

    expect(await screen.findByRole('status', { name: 'Microsoft connection status' })).toHaveTextContent(broken);
    expect(screen.queryByRole('alert')).not.toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Connect my Microsoft account' })).toBeDisabled();
    expect(screen.getAllByRole('heading', { level: 4 }).map((heading) => heading.textContent)).toEqual([
      'Microsoft upload identity', '1. Connect your Microsoft account', '2. Pair the intake folder',
    ]);
  });

  it('calls only the browser build a browser preview', async () => {
    const withoutMicrosoft = { ...bridge(), microsoftIntakeStatus: undefined, microsoftSignInStart: undefined };
    const view = render(<MicrosoftIntakeSettings bridge={withoutMicrosoft} savedFolder="C:/Intake" unsavedFolder={false} />);
    expect(screen.getByText(/This browser preview cannot verify any uploads/)).toBeVisible();
    view.unmount();

    const runtime = window as unknown as Record<string, unknown>;
    runtime.__TAURI_INTERNALS__ = {};
    try {
      render(<MicrosoftIntakeSettings bridge={withoutMicrosoft} savedFolder="C:/Intake" unsavedFolder={false} />);
      expect(screen.getByRole('group', { name: 'Microsoft upload identity' })).toBeVisible();
      expect(screen.queryByText(/browser preview/i)).not.toBeInTheDocument();
    } finally {
      delete runtime.__TAURI_INTERNALS__;
    }
  });

  it('describes the proof as Microsoft metadata about who created and last modified a document', async () => {
    render(<MicrosoftIntakeSettings bridge={bridge()} savedFolder="C:/Intake" unsavedFolder={false} />);
    const lead = screen.getByText(/Unverified uploads are never processed/);
    expect(lead).toHaveTextContent(/created/);
    expect(lead).toHaveTextContent(/last modified/);
    expect(lead).not.toHaveTextContent(/upload activity/);
    await act(async () => {});
  });

  it('uses one managed connection action without audit consent or identifier fields', async () => {
    const api = bridge();
    render(<MicrosoftIntakeSettings bridge={api} savedFolder="C:/Intake" unsavedFolder={false} />);
    const connect = screen.getByRole('button', { name: 'Connect my Microsoft account' });
    expect(connect).toBeEnabled();
    expect(screen.queryByLabelText(/tenant ID|application ID|drive ID|folder ID/i)).not.toBeInTheDocument();
    expect(screen.queryByLabelText(/understand the Microsoft permissions/i)).not.toBeInTheDocument();
    expect(screen.getByRole('note')).toHaveTextContent('profile and file metadata');
    expect(screen.getByRole('note')).not.toHaveTextContent(/audit/i);
    fireEvent.click(connect);
    await screen.findByRole('status', { name: 'Microsoft sign-in' });
    expect(api.microsoftSignInStart).toHaveBeenCalledWith();
  });
  it('shows the authenticated identity, with no editable name that could grant ownership', async () => {
    render(<MicrosoftIntakeSettings bridge={bridge({ ...initial, connected: true, account })} savedFolder="C:/Intake" unsavedFolder={false} />);
    expect(await screen.findByText('Zachary Brenner')).toBeVisible();
    expect(screen.getByText('zack@example.test')).toBeVisible();
    expect(screen.queryByRole('textbox', { name: /name|email/i })).not.toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Disconnect Microsoft' })).toBeEnabled();
  });
  it('uses a fixed sign-in action and cancels pending authorization when closed', async () => {
    const api = bridge();
    const { unmount } = render(<MicrosoftIntakeSettings bridge={api} savedFolder="C:/Intake" unsavedFolder={false} />);
    fireEvent.click(screen.getByRole('button', { name: 'Connect my Microsoft account' }));
    expect(await screen.findByRole('status', { name: 'Microsoft sign-in' })).toHaveTextContent('ABCD-EFGH');
    fireEvent.click(screen.getByRole('button', { name: 'Open Microsoft sign-in' }));
    await waitFor(() => expect(api.microsoftOpenSignIn).toHaveBeenCalledWith());
    unmount();
    expect(api.microsoftDisconnect).toHaveBeenCalledOnce();
  });
  it('never pairs a draft folder that has not been saved', async () => {
    render(<MicrosoftIntakeSettings bridge={bridge({ ...initial, connected: true, account })} savedFolder="C:/Old" unsavedFolder />);
    await screen.findByText('Zachary Brenner');
    expect(screen.getByRole('button', { name: 'Verify folder pairing' })).toBeDisabled();
    expect(screen.getByText('Save your changed intake folder before pairing it.')).toBeVisible();
  });
  it('pairs saved intake through the backend and shows the resolved Microsoft folder', async () => {
    const api = bridge({ ...initial, connected: true, account });
    render(<MicrosoftIntakeSettings bridge={api} savedFolder="C:/Intake" unsavedFolder={false} />);
    await screen.findByText('Zachary Brenner');
    fireEvent.click(screen.getByRole('button', { name: 'Verify folder pairing' }));
    expect(await screen.findByRole('status', { name: 'Microsoft folder pairing' })).toHaveTextContent('https://teamcontoso.sharepoint.com/sites/InternTestSite/Files/Inbox');
    expect(api.microsoftBindIntake).toHaveBeenCalledWith();
  });
  it('separates the uploader, processor, and unknown holds without an override button', async () => {
    const processor = { ...account, id: 'dddddddd-dddd-dddd-dddd-dddddddddddd', displayName: 'John Smith', email: 'john@example.test' };
    const api = bridge({ ...initial, connected: true, account, documents: [
      { path: 'one', filename: 'agreement.pdf', state: 'processed', reason: 'Analysis completed.', uploader: account, processedBy: processor, filedAs: null, checkedAt: 0 },
      { path: 'two', filename: 'unknown.pdf', state: 'unknown', reason: 'Upload event missing.', uploader: null, processedBy: null, filedAs: null, checkedAt: 0 },
    ] });
    render(<MicrosoftIntakeSettings bridge={api} savedFolder="C:/Intake" unsavedFolder={false} />);
    expect(await screen.findByRole('status', { name: 'Uploader counts' })).toHaveTextContent('1 verified · 0 belonging to others · 1 uploader unknown');
    expect(screen.getByText(/Processed by John Smith/)).toBeVisible();
    expect(screen.getByText('Held: uploader unknown')).toBeVisible();
    expect(screen.queryByRole('button', { name: /process anyway|ignore verification/i })).not.toBeInTheDocument();
  });
  it('displays a failed connection without claiming that uploads were verified', async () => {
    const api = { ...bridge(), microsoftSignInStart: vi.fn(async () => { throw new Error('Organization denied consent.'); }) };
    render(<MicrosoftIntakeSettings bridge={api} savedFolder="C:/Intake" unsavedFolder={false} />);
    fireEvent.click(screen.getByRole('button', { name: 'Connect my Microsoft account' }));
    expect(await screen.findByRole('alert')).toHaveTextContent('Organization denied consent.');
    expect(screen.queryByText('Processing for')).not.toBeInTheDocument();
  });
  it('does not issue duplicate device-code requests from same-tick clicks', async () => {
    let resolve!: (value: Awaited<ReturnType<MicrosoftIntakeBridge['microsoftSignInStart']>>) => void;
    const api = { ...bridge(), microsoftSignInStart: vi.fn(() => new Promise<Awaited<ReturnType<MicrosoftIntakeBridge['microsoftSignInStart']>>>((done) => { resolve = done; })) };
    const { unmount } = render(<MicrosoftIntakeSettings bridge={api} savedFolder="C:/Intake" unsavedFolder={false} />);
    const button = screen.getByRole('button', { name: 'Connect my Microsoft account' });
    act(() => { fireEvent.click(button); fireEvent.click(button); });
    expect(api.microsoftSignInStart).toHaveBeenCalledOnce();
    await act(async () => { resolve({ userCode: 'CODE', verificationUri: 'https://microsoft.com/devicelogin', intervalSeconds: 5, expiresAt: 9999999999 }); });
    unmount();
  });
});
