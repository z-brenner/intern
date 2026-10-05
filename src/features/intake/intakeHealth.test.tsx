import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { createInMemoryBridge } from '../../lib/inMemoryBridge';
import type { IntakeStatus } from '../../types';
import { IntakeHealthNotice, intakeActivity } from './IntakeHealth';

const status = (overrides: Partial<IntakeStatus> = {}): IntakeStatus => ({
  enabled: true, watching: true, folder: 'C:\\Users\\pat\\OneDrive\\Scans', machineId: 'm', machineName: 'PC',
  cloud: { provider: 'onedrive_personal', displayName: 'OneDrive' }, machines: [], heldForOthers: 0, syncConflicts: 0,
  awaitingHydration: 0, unreadableFolders: 0, claimedByOthers: 0, processedHere: 0, lastScanAt: null, error: null,
  oneDriveRunning: true, ...overrides,
});

describe('folder activity', () => {
  // arriving_and_unreadable_counts_render
  it('renders the arriving and could-not-be-read counts beside the folder health', () => {
    render(<IntakeHealthNotice status={status({ arriving: 2, unreadableDocuments: 3 })} myFolder bridge={createInMemoryBridge()} />);

    expect(screen.getByRole('status', { name: 'Folder health' })).toHaveTextContent('Synced Up to date.');
    const activity = screen.getByRole('status', { name: 'Folder activity' });
    expect(activity).toHaveTextContent('2 documents arriving…');
    expect(activity).toHaveTextContent('3 documents could not be read.');
  });

  it('shows the counts for a folder OneDrive does not keep, and nothing when there are none', () => {
    const local = status({ cloud: null, oneDriveRunning: null });
    const view = render(<IntakeHealthNotice status={{ ...local, arriving: 1, unreadableDocuments: 1 }} myFolder={false} bridge={createInMemoryBridge()} />);
    expect(screen.queryByRole('status', { name: 'Folder health' })).toBeNull();
    expect(screen.getByRole('status', { name: 'Folder activity' })).toHaveTextContent('1 document arriving… 1 document could not be read. Intern keeps trying; check that it opens on this computer.');
    view.unmount();

    // A backend from before these counts existed sends neither.
    render(<IntakeHealthNotice status={local} myFolder={false} bridge={createInMemoryBridge()} />);
    expect(screen.queryByRole('status', { name: 'Folder activity' })).toBeNull();
    expect(intakeActivity(status({ enabled: false, arriving: 4 }))).toEqual([]);
  });
});
