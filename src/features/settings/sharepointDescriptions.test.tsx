import { act, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { App } from '../../App';
import { SUPPORT_LINKS } from '../../lib/bridge';
import type { DesktopBridge } from '../../lib/bridge';
import { createInMemoryBridge } from '../../lib/inMemoryBridge';
import type { DescriptionsStatus } from '../../types';
import { SharePointDescriptions } from './SharePointDescriptions';

const on: DescriptionsStatus = { enabled: true, folder: 'C:\\Filed\\.intern\\descriptions', recordedThisSession: 3, lastRecordedAt: 1775001600, lastError: null };
const off: DescriptionsStatus = { enabled: false, folder: 'C:\\Filed\\.intern\\descriptions', recordedThisSession: 0, lastRecordedAt: null, lastError: null };

function bridgeWith(overrides: Partial<DesktopBridge> = {}): DesktopBridge {
  return { ...createInMemoryBridge(), descriptionsStatus: vi.fn(async () => on), ...overrides };
}

function renderSection(bridge: DesktopBridge, props: Partial<{ savedEnabled: boolean; draftEnabled: boolean; onDraftEnabledChange: (enabled: boolean) => void }> = {}) {
  const onDraftEnabledChange = props.onDraftEnabledChange ?? vi.fn();
  render(<SharePointDescriptions bridge={bridge} savedEnabled={props.savedEnabled ?? true} draftEnabled={props.draftEnabled ?? true} onDraftEnabledChange={onDraftEnabledChange} />);
  return { onDraftEnabledChange };
}

describe('Settings: Descriptions for SharePoint', () => {
  it('says what it does, whether records are on, where they go, and when the last one was written', async () => {
    renderSection(bridgeWith());

    expect(screen.getByRole('heading', { name: 'Descriptions for SharePoint' })).toBeVisible();
    expect(screen.getByText(/one-sentence description of each filed document in your SharePoint library's Description column/)).toBeVisible();
    expect(screen.getByText(/Intern itself sends nothing/)).toBeVisible();
    const status = await screen.findByRole('status', { name: 'Description records status' });
    expect(status).toHaveTextContent('On · writing to C:\\Filed\\.intern\\descriptions · 3 records written since Intern started · Last recorded:');
    expect(status).toHaveTextContent(new Date(1775001600 * 1000).toLocaleString());
  });

  it('reports records off without claiming a folder is being written', async () => {
    renderSection(bridgeWith({ descriptionsStatus: vi.fn(async () => off) }), { savedEnabled: false, draftEnabled: false });

    const status = await screen.findByRole('status', { name: 'Description records status' });
    expect(status).toHaveTextContent('Off · no records are being written');
    expect(status).not.toHaveTextContent('C:\\Filed');
    expect(screen.getByLabelText('Write a description record for each filed document')).not.toBeChecked();
  });

  it('shows the last write failure in the operating system’s words', async () => {
    renderSection(bridgeWith({ descriptionsStatus: vi.fn(async () => ({ ...on, lastError: 'DESCRIPTION_WRITE_FAILED: Access is denied.' })) }));

    expect(await screen.findByRole('alert')).toHaveTextContent('The last description record could not be written. DESCRIPTION_WRITE_FAILED: Access is denied.');
  });

  it('follows live status updates from the backend', async () => {
    let push: ((status: DescriptionsStatus) => void) | undefined;
    const bridge = { ...bridgeWith(), subscribeDescriptions: (listener: (status: DescriptionsStatus) => void) => { push = listener; return () => { push = undefined; }; } };
    renderSection(bridge);
    const status = await screen.findByRole('status', { name: 'Description records status' });
    await waitFor(() => expect(push).toBeDefined());

    act(() => push!({ ...on, recordedThisSession: 4 }));
    expect(status).toHaveTextContent('4 records written since Intern started');
  });

  it('edits only the draft setting from the checkbox', () => {
    const { onDraftEnabledChange } = renderSection(bridgeWith(), { draftEnabled: false });

    fireEvent.click(screen.getByLabelText('Write a description record for each filed document'));
    expect(onDraftEnabledChange).toHaveBeenCalledWith(true);
  });

  it('lists the four steps in order', () => {
    renderSection(bridgeWith());

    const steps = within(screen.getByRole('list', { name: "Fill your library's Description column automatically" })).getAllByRole('listitem');
    expect(steps).toHaveLength(4);
    expect(steps[0]).toHaveTextContent('In the library, add a column named Description (Multiple lines of text).');
    expect(within(steps[1]).getByRole('button', { name: 'Open the flow download' })).toBeVisible();
    expect(steps[2]).toHaveTextContent('My flows → Import → Import Package (Legacy)');
    expect(steps[2]).toHaveTextContent('choose your site and library');
    expect(within(steps[3]).getByRole('button', { name: 'Write records for documents already filed' })).toBeVisible();
  });

  // The flow is fetched by the person's browser from the published guide.
  // The app hands the fixed address to the shell and requests nothing itself.
  it('opens the flow download in the browser through the fixed link, and makes no request of its own', async () => {
    const fetch = vi.spyOn(globalThis, 'fetch');
    const openSupportLink = vi.fn(async () => {});
    renderSection(bridgeWith({ openSupportLink }));

    fireEvent.click(screen.getByRole('button', { name: 'Open the flow download' }));
    await waitFor(() => expect(openSupportLink).toHaveBeenCalledWith('description-flow'));
    expect(SUPPORT_LINKS['description-flow']).toBe('https://z-brenner.github.io/intern/guide.html#sharepoint-descriptions');
    expect(fetch).not.toHaveBeenCalled();
    fetch.mockRestore();
  });

  it('shows the address when the browser cannot be opened', async () => {
    renderSection(bridgeWith({ openSupportLink: vi.fn(async () => { throw new Error('no handler'); }) }));

    fireEvent.click(screen.getByRole('button', { name: 'Open the flow download' }));
    expect(await screen.findByRole('alert')).toHaveTextContent(`The guide could not be opened. You can reach it at ${SUPPORT_LINKS['description-flow']}.`);
  });

  it('offers the backfill only once records are saved on', () => {
    renderSection(bridgeWith({ descriptionsStatus: vi.fn(async () => off) }), { savedEnabled: false, draftEnabled: true });

    expect(screen.getByRole('button', { name: 'Write records for documents already filed' })).toBeDisabled();
    expect(screen.getByText('Save with records on first.')).toBeVisible();
  });

  it('reports what a backfill wrote and refreshes the status', async () => {
    const descriptionsStatus = vi.fn(async () => on);
    renderSection(bridgeWith({ descriptionsStatus, descriptionsBackfill: vi.fn(async () => ({ written: 2, failed: 1 })) }));
    await screen.findByRole('status', { name: 'Description records status' });

    fireEvent.click(screen.getByRole('button', { name: 'Write records for documents already filed' }));
    expect(await screen.findByRole('status', { name: 'Backfill status' })).toHaveTextContent('2 records written; 1 could not be written.');
    expect(descriptionsStatus).toHaveBeenCalledTimes(2);
  });

  it('explains a refused backfill in plain words', async () => {
    renderSection(bridgeWith({ descriptionsBackfill: vi.fn(async () => { throw { code: 'DESCRIPTIONS_DISABLED', message: 'turn on description records' }; }) }));

    fireEvent.click(screen.getByRole('button', { name: 'Write records for documents already filed' }));
    expect(await screen.findByRole('alert')).toHaveTextContent('Turn on description records and save settings first; then records can be written for documents already filed. (DESCRIPTIONS_DISABLED)');
  });

  it('sits in the Settings dialog and saves the setting with the rest', async () => {
    const base = createInMemoryBridge();
    // The output folder is already set, so the test does not depend on how the folder fields are laid out.
    await base.saveSettings({ ...(await base.getSettings()), destination: 'C:\\Filed', recordDescriptions: false });
    const saveSettings = vi.fn(base.saveSettings);
    render(<App bridge={{ ...base, saveSettings }} />);
    fireEvent.click((await screen.findAllByRole('button', { name: 'Settings' }))[0]);
    const dialog = await screen.findByRole('dialog', { name: 'Settings' });

    const section = within(dialog).getByRole('region', { name: 'Descriptions for SharePoint' });
    fireEvent.click(within(section).getByLabelText('Write a description record for each filed document'));
    fireEvent.click(within(dialog).getByRole('button', { name: 'Save settings' }));
    await waitFor(() => expect(saveSettings).toHaveBeenCalledWith(expect.objectContaining({ recordDescriptions: true, destination: 'C:\\Filed' })));
  });
});
