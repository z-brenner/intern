import { fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { App } from '../../App';
import { SettingsDialog } from '../../components/SettingsDialog';
import { createInMemoryBridge } from '../../lib/inMemoryBridge';
import type { AppSettings } from '../../types';
import { openDetails, queueCell } from '../queue/gridTesting';

/*
  What counts as ready is a choice in Settings: Strict, as it always was, or
  Balanced. It stands above "Automatically rename ready documents" and is not
  a part of it, because it also decides what Rename all ready renames.
*/

async function openSettings() {
  fireEvent.click((await screen.findAllByRole('button', { name: 'Settings' }))[0]);
  const dialog = await screen.findByRole('dialog', { name: 'Settings' });
  fireEvent.click(within(dialog).getByText('How Intern names files', { selector: 'summary' }));
  return dialog;
}

const level = (dialog: HTMLElement) => within(dialog).getByRole('group', { name: 'Which documents are ready' });
const strict = (dialog: HTMLElement) => within(level(dialog)).getByRole('radio', { name: /^Strict \(default\)/ });
const balanced = (dialog: HTMLElement) => within(level(dialog)).getByRole('radio', { name: /^Balanced/ });

/** Settings as an older backend sends them: no review level at all. */
const older: AppSettings = { destination: 'D:\\Filed', destinationLayout: 'flat', startMinimized: false, automaticRename: false, intakeFolder: 'D:\\In', intakeEnabled: true, intakeLocalOnly: true, processOthersUploads: false, machineLabel: 'Front desk', runInBackground: false, startAtLogin: false, recordDescriptions: false, modelSource: 'local', hostedProvider: 'openai_compatible', hostedBaseUrl: '', hostedModel: '' };

function renderDialog(settings: AppSettings) {
  const onSave = vi.fn(async (_settings: AppSettings) => undefined);
  const props = { bridge: createInMemoryBridge(), onSave, onClose: vi.fn(), onCheckForUpdate: async () => ({ state: 'unsupported' as const }), onInstallUpdate: async () => undefined };
  const view = render(<SettingsDialog settings={settings} {...props} />);
  return { onSave, rerender: (next: AppSettings) => view.rerender(<SettingsDialog settings={next} {...props} />) };
}

describe('which documents are ready, in Settings', () => {
  it('offers Strict and Balanced, Strict by default, and says what each does without a figure', async () => {
    render(<App bridge={createInMemoryBridge()} />);
    const dialog = await openSettings();
    const group = level(dialog);

    expect(strict(dialog)).toBeChecked();
    expect(balanced(dialog)).not.toBeChecked();
    expect(group).toBeVisible();
    // Strict: only what passes every check.
    expect(strict(dialog)).toHaveAccessibleName(expect.stringMatching(/^Strict \(default\)\s*Ready only when every check passes\. Everything else waits for you\.$/));
    // Balanced: what can also be ready, what still waits, and what it costs.
    const words = balanced(dialog).closest('label')!;
    expect(words).toHaveTextContent('Also ready when the only doubts are its type, its description or the model’s own confidence.');
    expect(words).toHaveTextContent('Still waits when the date or a party cannot be confirmed, the name would have no document type, or Intern could not read it.');
    // What it costs is said on a line of its own, not lost at the end of the paragraph.
    const cost = within(words).getByText('More documents are renamed without you looking, and more of those names will be wrong.');
    expect(cost).toHaveClass('review-level-cost');
    expect(balanced(dialog)).toHaveAccessibleName(expect.stringMatching(/more of those names will be wrong\.$/));
    // At either level, the quick model's readings wait.
    expect(group).toHaveTextContent('Either way, a document the quick model read always waits for you.');
    // No figure anywhere in it: how many more are wrong depends on the documents.
    expect(group.textContent).not.toMatch(/\d/);
  });

  it('says it decides Rename all ready too, and stands before the automatic renaming it is not a part of', async () => {
    render(<App bridge={createInMemoryBridge()} />);
    const dialog = await openSettings();
    const group = level(dialog);
    // Read out with the group: what ready means for the button and for automatic renaming, and the quick model.
    expect(group).toHaveAccessibleDescription('Ready documents are the ones Rename all ready renames, or that Intern renames by itself if automatic renaming is on. Either way, a document the quick model read always waits for you.');
    const automatic = within(dialog).getByRole('checkbox', { name: 'Automatically rename ready documents' });
    expect(group.contains(automatic)).toBe(false);
    expect(group.compareDocumentPosition(automatic) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
    // The checkbox no longer speaks of confidence, which at Balanced is not what ready means.
    expect(dialog).not.toHaveTextContent(/high-confidence|less sure about/);
    expect(dialog).toHaveTextContent('Anything not ready still waits for you.');
    // The level is there whether automatic renaming is on or off.
    expect(automatic).not.toBeChecked();
    expect(strict(dialog)).toBeEnabled();
    expect(balanced(dialog)).toBeEnabled();
    fireEvent.click(automatic);
    expect(balanced(dialog)).toBeEnabled();
  });

  it('saves the level chosen, and shows it again when Settings is opened next', async () => {
    const base = createInMemoryBridge();
    const saveSettings = vi.fn(base.saveSettings);
    render(<App bridge={{ ...base, saveSettings }} />);
    let dialog = await openSettings();

    fireEvent.click(balanced(dialog));
    expect(balanced(dialog)).toBeChecked();
    expect(strict(dialog)).not.toBeChecked();
    fireEvent.click(within(dialog).getByRole('button', { name: 'Save settings' }));
    await waitFor(() => expect(saveSettings).toHaveBeenCalledWith(expect.objectContaining({ reviewLevel: 'balanced', automaticRename: false })));
    await waitFor(() => expect(screen.queryByRole('dialog', { name: 'Settings' })).not.toBeInTheDocument());
    expect((await base.getSettings()).reviewLevel).toBe('balanced');

    dialog = await openSettings();
    expect(balanced(dialog)).toBeChecked();
    fireEvent.click(strict(dialog));
    fireEvent.click(within(dialog).getByRole('button', { name: 'Save settings' }));
    await waitFor(() => expect(saveSettings).toHaveBeenLastCalledWith(expect.objectContaining({ reviewLevel: 'strict' })));
    await waitFor(() => expect(screen.queryByRole('dialog', { name: 'Settings' })).not.toBeInTheDocument());
    expect((await base.getSettings()).reviewLevel).toBe('strict');
  });

  it('throws the choice away when Settings is closed without saving', async () => {
    const base = createInMemoryBridge();
    const saveSettings = vi.fn(base.saveSettings);
    render(<App bridge={{ ...base, saveSettings }} />);
    let dialog = await openSettings();
    fireEvent.click(balanced(dialog));
    fireEvent.keyDown(balanced(dialog), { key: 'Escape' });
    await waitFor(() => expect(screen.queryByRole('dialog', { name: 'Settings' })).not.toBeInTheDocument());
    expect(saveSettings).not.toHaveBeenCalled();
    expect((await base.getSettings()).reviewLevel).toBe('strict');

    dialog = await openSettings();
    expect(strict(dialog)).toBeChecked();
  });

  it('shows a saved Balanced, and Strict for settings that say no level or one it does not know', async () => {
    const view = renderDialog({ ...older, reviewLevel: 'balanced' });
    expect(balanced(await screen.findByRole('dialog', { name: 'Settings' }))).toBeChecked();
    view.rerender({ ...older, reviewLevel: 'strict' });
    expect(strict(screen.getByRole('dialog', { name: 'Settings' }))).toBeChecked();
    view.rerender(older);
    expect(strict(screen.getByRole('dialog', { name: 'Settings' }))).toBeChecked();
    view.rerender({ ...older, reviewLevel: 'lenient' } as unknown as AppSettings);
    expect(strict(screen.getByRole('dialog', { name: 'Settings' }))).toBeChecked();
    expect(balanced(screen.getByRole('dialog', { name: 'Settings' }))).not.toBeChecked();
  });
});

describe('saving with the level as it was', () => {
  // Settings sends the whole settings object. One loaded with no level - an
  // older settings file - is saved with none, so a save that changed
  // something else does not also record a level nobody chose.
  it('adds no level to settings that had none', async () => {
    const { onSave } = renderDialog(older);
    const dialog = await screen.findByRole('dialog', { name: 'Settings' });
    expect(strict(dialog)).toBeChecked();
    fireEvent.click(within(dialog).getByRole('button', { name: 'Save settings' }));
    await waitFor(() => expect(onSave).toHaveBeenCalledOnce());
    expect(onSave).toHaveBeenCalledWith(older);
    expect('reviewLevel' in onSave.mock.calls[0][0]).toBe(false);
  });

  it('sends the level back unchanged beside another edit, and a level chosen and taken back is no edit', async () => {
    const loaded: AppSettings = { ...older, reviewLevel: 'balanced' };
    const { onSave } = renderDialog(loaded);
    const dialog = await screen.findByRole('dialog', { name: 'Settings' });
    fireEvent.click(strict(dialog));
    fireEvent.click(balanced(dialog));
    fireEvent.click(within(dialog).getByRole('checkbox', { name: 'Automatically rename ready documents' }));
    fireEvent.click(within(dialog).getByRole('button', { name: 'Save settings' }));
    await waitFor(() => expect(onSave).toHaveBeenCalledWith({ ...loaded, automaticRename: true }));
  });

  it('takes back a level chosen over settings that had none, leaving them with none', async () => {
    const { onSave } = renderDialog(older);
    const dialog = await screen.findByRole('dialog', { name: 'Settings' });
    fireEvent.click(balanced(dialog));
    fireEvent.click(strict(dialog));
    fireEvent.click(within(dialog).getByRole('button', { name: 'Save settings' }));
    await waitFor(() => expect(onSave).toHaveBeenCalledOnce());
    expect(onSave).toHaveBeenCalledWith(older);
    expect('reviewLevel' in onSave.mock.calls[0][0]).toBe(false);
  });

  // App reads the settings a render after it mounts. A level that was not
  // touched follows the settings that arrive; one that was chosen stays.
  it('follows settings that arrive after it opened while the level is untouched', async () => {
    const { onSave, rerender } = renderDialog({ ...older, reviewLevel: 'strict' });
    const dialog = await screen.findByRole('dialog', { name: 'Settings' });
    rerender({ ...older, reviewLevel: 'balanced' });
    expect(balanced(dialog)).toBeChecked();
    fireEvent.click(within(dialog).getByRole('button', { name: 'Save settings' }));
    await waitFor(() => expect(onSave).toHaveBeenCalledWith({ ...older, reviewLevel: 'balanced' }));
  });

  it('keeps a level the person chose when the settings arrive after it', async () => {
    const { onSave, rerender } = renderDialog({ ...older, reviewLevel: 'strict' });
    const dialog = await screen.findByRole('dialog', { name: 'Settings' });
    fireEvent.click(balanced(dialog));
    rerender({ ...older, machineLabel: 'Reception' });
    expect(balanced(dialog)).toBeChecked();
    fireEvent.click(within(dialog).getByRole('button', { name: 'Save settings' }));
    await waitFor(() => expect(onSave).toHaveBeenCalledWith({ ...older, machineLabel: 'Reception', reviewLevel: 'balanced' }));
  });

  // A level this build does not know is shown as Strict, and Strict is what
  // a save then writes: never a value outside the two.
  it('writes an unknown stored level as the Strict it showed', async () => {
    const { onSave } = renderDialog({ ...older, reviewLevel: 'lenient' } as unknown as AppSettings);
    const dialog = await screen.findByRole('dialog', { name: 'Settings' });
    expect(strict(dialog)).toBeChecked();
    fireEvent.click(within(dialog).getByRole('button', { name: 'Save settings' }));
    await waitFor(() => expect(onSave).toHaveBeenCalledWith({ ...older, reviewLevel: 'strict' }));
  });
});

describe('the level from the keyboard', () => {
  // Settings moves Tab itself, one control at a time, as it does through
  // the local model's choices: the two levels, then the checkbox after them.
  it('is reached by Tab between the organization\'s names and automatic renaming, each level a stop', async () => {
    render(<App bridge={createInMemoryBridge()} />);
    const dialog = await openSettings();
    const names = within(dialog).getByLabelText('Your organizations');
    names.focus();
    fireEvent.keyDown(names, { key: 'Tab' });
    expect(strict(dialog)).toHaveFocus();
    fireEvent.keyDown(strict(dialog), { key: 'Tab' });
    expect(balanced(dialog)).toHaveFocus();
    fireEvent.keyDown(balanced(dialog), { key: 'Tab' });
    expect(within(dialog).getByRole('checkbox', { name: 'Automatically rename ready documents' })).toHaveFocus();
    fireEvent.keyDown(document.activeElement!, { key: 'Tab', shiftKey: true });
    expect(balanced(dialog)).toHaveFocus();
    // Focus alone chooses nothing.
    expect(strict(dialog)).toBeChecked();
  });

  it('is not reached while its section is closed', async () => {
    render(<App bridge={createInMemoryBridge()} />);
    fireEvent.click((await screen.findAllByRole('button', { name: 'Settings' }))[0]);
    const dialog = await screen.findByRole('dialog', { name: 'Settings' });
    const summary = within(dialog).getByText('How Intern names files', { selector: 'summary' });
    summary.focus();
    fireEvent.keyDown(summary, { key: 'Tab' });
    expect(within(dialog).getByText('Learned spellings', { selector: 'summary' })).toHaveFocus();
  });

  it('is a group of two radio buttons of one name, so the arrow keys move between them and the space bar chooses', async () => {
    render(<App bridge={createInMemoryBridge()} />);
    const dialog = await openSettings();
    const radios = within(level(dialog)).getAllByRole('radio') as HTMLInputElement[];
    expect(radios).toHaveLength(2);
    expect(new Set(radios.map((radio) => radio.name))).toEqual(new Set(['review-level']));
    expect(radios.map((radio) => radio.value)).toEqual(['strict', 'balanced']);
    // Neither is a control that acts on a document: choosing one changes a draft, and Save settings saves it.
    for (const radio of radios) expect(radio.closest('[data-acts]')).toBeNull();
    expect(level(dialog).closest('[data-acts]')).toBeNull();
  });
});

/*
  Saving another level judges the documents that wait again, in the backend.
  The window reads the queue after any save of the settings, so what the
  level changed is on screen at once.
*/
describe('the queue after the level is saved', () => {
  const LEASE = 'Lease Agreement - 123 Main St.pdf';
  const save = async (choose: (dialog: HTMLElement) => HTMLElement) => {
    const dialog = await openSettings();
    fireEvent.click(choose(dialog));
    fireEvent.click(within(dialog).getByRole('button', { name: 'Save settings' }));
    await waitFor(() => expect(screen.queryByRole('dialog', { name: 'Settings' })).not.toBeInTheDocument());
  };

  it('shows the demo\'s lease ready with a note at Balanced, and waiting again at Strict', async () => {
    const base = createInMemoryBridge();
    const approve = vi.fn(base.approve);
    render(<App bridge={{ ...base, approve }} />);
    await screen.findByRole('grid', { name: 'Queue' });
    expect(queueCell(LEASE, 'status')).toHaveTextContent('Needs review');
    const strictCheck = queueCell(LEASE, 'check').textContent;
    expect(screen.getByRole('button', { name: '1 needs review' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: '3 ready' })).toBeInTheDocument();

    await save(balanced);
    await waitFor(() => expect(queueCell(LEASE, 'status')).toHaveTextContent('Ready with a note'));
    expect(queueCell(LEASE, 'check')).toHaveTextContent('Note: Type uncertain +1 more');
    expect(screen.queryByRole('button', { name: /needs? review/ })).not.toBeInTheDocument();
    expect(screen.getByRole('button', { name: '4 ready, 1 with a note' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Rename all ready' })).toHaveTextContent('Rename all ready 4');
    // Saving a level renames nothing.
    expect(approve).not.toHaveBeenCalled();
    expect(screen.getByRole('button', { name: 'Completed, 1' })).toBeInTheDocument();

    await save(strict);
    await waitFor(() => expect(queueCell(LEASE, 'status')).toHaveTextContent('Needs review'));
    expect(queueCell(LEASE, 'check').textContent).toBe(strictCheck);
    expect(screen.getByRole('button', { name: '1 needs review' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: '3 ready' })).toBeInTheDocument();
    expect(approve).not.toHaveBeenCalled();
  });

  // The level changes the row, not the proposal: a name typed for the
  // document is still in its field, and Rename all ready still leaves a
  // document with something typed for the person.
  it('keeps what was typed for a document the level made ready', async () => {
    const base = createInMemoryBridge();
    const approve = vi.fn(base.approve);
    render(<App bridge={{ ...base, approve }} />);
    await screen.findByRole('grid', { name: 'Queue' });
    const panel = await openDetails(LEASE);
    fireEvent.change(within(panel).getByLabelText('Filename'), { target: { value: '2023-09-15 Lease of 123 Main St' } });
    expect(within(panel).getByRole('heading', { name: 'What to check' })).toBeInTheDocument();

    await save(balanced);
    await waitFor(() => expect(within(panel).getByRole('region', { name: "Intern's note" })).toBeInTheDocument());
    expect(within(panel).queryByRole('heading', { name: 'What to check' })).not.toBeInTheDocument();
    expect(within(panel).getByLabelText('Filename')).toHaveValue('2023-09-15 Lease of 123 Main St');
    expect(within(panel).getByRole('button', { name: /^Apply rename/ })).toBeEnabled();
    // Four are ready; the button renames the three nobody typed over.
    expect(screen.getByRole('button', { name: '4 ready, 1 with a note' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Rename all ready' })).toHaveTextContent('Rename all ready 3');
    fireEvent.click(screen.getByRole('button', { name: 'Rename all ready' }));
    await waitFor(() => expect(approve).toHaveBeenCalledTimes(3));
    expect(approve).not.toHaveBeenCalledWith('lease', expect.anything(), expect.anything());
    expect(await screen.findByRole('group', { name: 'Action result' })).toHaveTextContent('Renamed 3 documents. 1 was left for you');
    // None of the three had a note, so nothing is said of notes.
    expect(screen.getByRole('group', { name: 'Action result' })).not.toHaveTextContent('had a note');
    expect(within(panel).getByLabelText('Filename')).toHaveValue('2023-09-15 Lease of 123 Main St');
  });
});
