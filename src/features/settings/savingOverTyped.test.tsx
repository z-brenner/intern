import { act, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { App } from '../../App';
import { SettingsDialog } from '../../components/SettingsDialog';
import { createInMemoryBridge } from '../../lib/inMemoryBridge';
import { TauriBridge } from '../../lib/tauriBridge';
import type { QueueEventSource } from '../../lib/tauriBridge';
import type { AppSettings, QueueItem } from '../../types';
import { openDetails, queueCell } from '../queue/gridTesting';
import { keptText, typedAndNotFiled } from '../review/drafts';
import type { EarlierDrafts, ReviewDrafts } from '../review/drafts';
import { NOTHING_TYPED, renamedOverTyped, typedWaiting, typedWarning } from './savingOverTyped';
import type { TypedWaiting } from './savingOverTyped';

/*
  With automatic renaming on, the backend renames every ready document nobody
  has approved, and does not know what a person typed in review and did not
  file. Two saves in Settings can so rename a document under the very name
  its person was correcting: automatic renaming switched on while a ready
  document has something typed for it, and Strict changed to Balanced with
  automatic renaming on, which makes ready some of the documents waiting in
  review. Settings says so, above Save, before either is saved.
*/

const ONE = '1 document has a name or a description you typed and did not file. When you save, Intern can rename it under the name it proposed. You can file it first, or take back what you typed. What you typed is kept with the document, but is not filed.';
const several = (count: number) => `${count} documents have a name or a description you typed and did not file. When you save, Intern can rename them under the names it proposed. You can file them first, or take back what you typed. What you typed is kept with each document, but is not filed.`;

/** Settings as an older backend sends them: no review level at all. */
const older: AppSettings = { destination: 'D:\\Filed', destinationLayout: 'flat', startMinimized: false, automaticRename: false, intakeFolder: 'D:\\In', intakeEnabled: true, intakeLocalOnly: true, processOthersUploads: false, machineLabel: 'Front desk', runInBackground: false, startAtLogin: false, recordDescriptions: false, modelSource: 'local', hostedProvider: 'openai_compatible', hostedBaseUrl: '', hostedModel: '' };

describe('the documents a save could rename over what was typed, counted', () => {
  const row = (id: string, status: QueueItem['status'], more: Partial<QueueItem> = {}): QueueItem => ({ id, originalFilename: `${id}.pdf`, status, proposedFilename: `2024-05-01 ${id}.pdf`, ...more });
  const items: QueueItem[] = [
    row('ready-typed', 'ready'),
    row('ready-plain', 'ready'),
    row('ready-noted-typed', 'ready', { reasonCodes: ['TYPE_INFERRED'] }),
    row('review-typed', 'review', { reasonCodes: ['LOW_CONFIDENCE'] }),
    row('review-plain', 'review'),
    row('review-set-aside', 'review'),
    row('completed-typed', 'completed', { filedName: '2024-05-01 completed-typed.pdf', undoable: true }),
    row('reading-typed', 'processing', { stage: 'reading' }),
    row('renaming-typed', 'processing', { stage: 'filing', cancelable: false }),
    row('waiting-typed', 'waiting', { proposedFilename: undefined }),
    row('failed-typed', 'failed'),
  ];
  const typedFor = (...ids: string[]) => (id: string) => ids.includes(id);

  it('counts ready documents and documents in review apart, and only those with something typed', () => {
    expect(typedWaiting(items, typedFor('ready-typed', 'ready-noted-typed', 'review-typed'))).toEqual({ ready: 2, review: 1 });
    expect(typedWaiting(items, typedFor('review-typed', 'review-set-aside'))).toEqual({ ready: 0, review: 2 });
    expect(typedWaiting(items, typedFor())).toEqual(NOTHING_TYPED);
    expect(typedWaiting([], typedFor('ready-typed'))).toEqual(NOTHING_TYPED);
  });

  it('does not count a document being read, renamed, failed, completed or not yet read, whatever was typed for it', () => {
    expect(typedWaiting(items, typedFor('completed-typed', 'reading-typed', 'renaming-typed', 'waiting-typed', 'failed-typed'))).toEqual(NOTHING_TYPED);
    // Everything typed for: still only the three ready and the three in review.
    expect(typedWaiting(items, () => true)).toEqual({ ready: 3, review: 3 });
  });

  it('does not count what Intern has no name for or will not name by itself: an unreadable document, an approved one, a staged copy', () => {
    const others: QueueItem[] = [
      // Could not be read: it waits in review with no proposal, and a person types its name.
      // The demo and an older backend leave the name out.
      row('unreadable', 'review', { proposedFilename: undefined, errorCode: 'PASSWORD_PROTECTED' }),
      // The desktop backend sends the key all the same, holding null, and the bridge hands it on.
      row('unreadable-as-sent', 'review', { proposedFilename: null as unknown as undefined, errorCode: 'PASSWORD_PROTECTED' }),
      row('unverified-as-sent', 'review', { proposedFilename: null as unknown as undefined, errorCode: 'UPLOADER_UNVERIFIED' }),
      row('ready-with-no-name', 'ready', { proposedFilename: null as unknown as undefined }),
      // An empty name is no name either.
      row('empty-name', 'review', { proposedFilename: '' }),
      // Approved by a person, waiting for the queue to be free: filed under what they approved.
      row('approved', 'ready', { approved: true }),
      // A duplicate staged as a copy: named after its original, by nobody.
      row('copy', 'ready', { copyOf: '2024-05-01 original.pdf' }),
    ];
    expect(typedWaiting(others, () => true)).toEqual(NOTHING_TYPED);
    // Each alone, so that no one of them is let through by another.
    for (const other of others) expect(typedWaiting([other], () => true), other.id).toEqual(NOTHING_TYPED);
    // Beside ones that do count, they change nothing.
    expect(typedWaiting([...others, row('review-typed', 'review'), row('ready-typed', 'ready')], () => true)).toEqual({ ready: 1, review: 1 });
  });

  it('takes typed as the window does: in the fields now, or set aside when the document changed, and never Intern\'s own proposal', () => {
    const drafts: ReviewDrafts = new Map();
    const earlier: EarlierDrafts = new Map();
    const typed = (id: string) => typedAndNotFiled(drafts, earlier, id);
    expect(typedWaiting(items, typed)).toEqual(NOTHING_TYPED);
    // Shown in the panel and not changed: no edit.
    drafts.set('ready-plain', { item: 'ready-plain', for: 'x', stem: '2024-05-01 ready-plain', description: '', edited: false });
    drafts.set('review-plain', { item: 'review-plain', for: 'x', stem: '2024-05-01 review-plain', description: '', edited: false });
    // Kept only to hold the caret when a field went: Intern's proposal, not typed.
    earlier.set('review-plain', [keptText('2024-05-01 review-plain', false)]);
    expect(typedWaiting(items, typed)).toEqual(NOTHING_TYPED);
    drafts.set('ready-typed', { item: 'ready-typed', for: 'x', stem: '2024-05-01 My own name', description: '', edited: true });
    drafts.set('review-typed', { item: 'review-typed', for: 'x', stem: '2024-05-01 Mine too', description: 'A sentence of mine.', edited: true });
    expect(typedWaiting(items, typed)).toEqual({ ready: 1, review: 1 });
    // Set aside when the document was read again, and still not filed.
    earlier.set('review-set-aside', [keptText('2024-05-01 Typed before it was read again', true)]);
    earlier.set('completed-typed', [keptText('2024-05-01 Typed before it was filed elsewhere', true)]);
    expect(typedWaiting(items, typed)).toEqual({ ready: 1, review: 2 });
    // Filed, taken back or discarded: no longer typed.
    drafts.delete('review-typed');
    earlier.delete('review-set-aside');
    expect(typedWaiting(items, typed)).toEqual({ ready: 1, review: 0 });
  });
});

describe('whether a save lets Intern rename them', () => {
  const typed: TypedWaiting = { ready: 2, review: 5 };
  const settings = (automaticRename: boolean, reviewLevel?: unknown): AppSettings => (reviewLevel === undefined ? { ...older, automaticRename } : { ...older, automaticRename, reviewLevel } as AppSettings);
  // [automatic before, level before, automatic after, level after, the documents counted]
  const cases: Array<[boolean, unknown, boolean, unknown, number]> = [
    // Automatic renaming off once saved: nothing is renamed by itself.
    [false, 'strict', false, 'strict', 0],
    [false, 'strict', false, 'balanced', 0],
    [false, 'balanced', false, 'strict', 0],
    [true, 'strict', false, 'strict', 0],
    [true, 'strict', false, 'balanced', 0],
    [true, 'balanced', false, 'balanced', 0],
    // Switched on by this save: the ready ones; and those in review too when the level goes to Balanced.
    [false, 'strict', true, 'strict', 2],
    [false, 'strict', true, 'balanced', 7],
    [false, 'balanced', true, 'balanced', 2],
    [false, 'balanced', true, 'strict', 2],
    // On already: only the level going from Strict to Balanced makes anything ready.
    [true, 'strict', true, 'strict', 0],
    [true, 'strict', true, 'balanced', 5],
    [true, 'balanced', true, 'balanced', 0],
    [true, 'balanced', true, 'strict', 0],
    // No level, or one this build does not know, is Strict.
    [true, undefined, true, 'balanced', 5],
    [false, undefined, true, undefined, 2],
    [true, 'lenient', true, 'balanced', 5],
    [true, 'balanced', true, 'lenient', 0],
    [true, 'strict', true, 'lenient', 0],
  ];
  it.each(cases)('automatic %s at %s, saved as automatic %s at %s: %i', (automaticBefore, levelBefore, automaticAfter, levelAfter, count) => {
    expect(renamedOverTyped(settings(automaticBefore, levelBefore), settings(automaticAfter, levelAfter), typed)).toBe(count);
  });

  it('counts none where nothing is typed, whatever is saved', () => {
    for (const [automaticBefore, levelBefore, automaticAfter, levelAfter] of cases) {
      expect(renamedOverTyped(settings(automaticBefore, levelBefore), settings(automaticAfter, levelAfter), NOTHING_TYPED)).toBe(0);
    }
  });

  it('counts each kind only for the change that lets it be renamed', () => {
    expect(renamedOverTyped(settings(false, 'strict'), settings(true, 'strict'), { ready: 0, review: 4 })).toBe(0);
    expect(renamedOverTyped(settings(true, 'strict'), settings(true, 'balanced'), { ready: 4, review: 0 })).toBe(0);
    expect(renamedOverTyped(settings(false, 'strict'), settings(true, 'balanced'), { ready: 1, review: 0 })).toBe(1);
    expect(renamedOverTyped(settings(false, 'strict'), settings(true, 'balanced'), { ready: 0, review: 1 })).toBe(1);
  });
});

describe('the warning\'s words', () => {
  it('says how many documents, that Intern can rename them under the names it proposed, what to do first, and where what was typed is afterwards', () => {
    expect(typedWarning(1)).toBe(ONE);
    expect(typedWarning(2)).toBe(several(2));
    expect(typedWarning(14)).toBe(several(14));
  });

  it('is nothing for no document', () => {
    expect(typedWarning(0)).toBe('');
    expect(typedWarning(-1)).toBe('');
  });

  it('says "can", since not every document in review becomes ready, and has no long dash', () => {
    for (const words of [ONE, several(3)]) {
      expect(words).toContain('Intern can rename');
      expect(words).not.toMatch(/\bwill\b/);
      expect(words).not.toMatch(/[\u2012-\u2015]/);
    }
  });
});

const level = (dialog: HTMLElement) => within(dialog).getByRole('group', { name: 'Which documents are ready' });
const strict = (dialog: HTMLElement) => within(level(dialog)).getByRole('radio', { name: /^Strict \(default\)/ });
const balanced = (dialog: HTMLElement) => within(level(dialog)).getByRole('radio', { name: /^Balanced/ });
const automatic = (dialog: HTMLElement) => within(dialog).getByRole('checkbox', { name: 'Automatically rename ready documents' });
const saveButton = (dialog: HTMLElement) => within(dialog).getByRole('button', { name: 'Save settings' });
/** The warning, or nothing: the one alert in Settings that speaks of what was typed. */
const warning = (dialog: HTMLElement) => within(dialog).queryAllByRole('alert').find((alert) => alert.textContent?.includes('you typed and did not file')) ?? null;

function renderDialog(settings: AppSettings, typed?: TypedWaiting) {
  const onSave = vi.fn(async (_settings: AppSettings) => undefined);
  const props = { bridge: createInMemoryBridge(), onSave, onClose: vi.fn(), onCheckForUpdate: async () => ({ state: 'unsupported' as const }), onInstallUpdate: async () => undefined };
  const view = render(typed ? <SettingsDialog settings={settings} typed={typed} {...props} /> : <SettingsDialog settings={settings} {...props} />);
  return { onSave, unmount: view.unmount, rerender: (next: AppSettings, counts: TypedWaiting) => view.rerender(<SettingsDialog settings={next} typed={counts} {...props} />) };
}

describe('the warning in Settings', () => {
  it('comes and goes with the two choices: Strict to Balanced with automatic renaming on, for a document in review', async () => {
    renderDialog({ ...older, reviewLevel: 'strict' }, { ready: 0, review: 1 });
    const dialog = await screen.findByRole('dialog', { name: 'Settings' });
    expect(warning(dialog)).toBeNull();
    // Balanced alone renames nothing by itself.
    fireEvent.click(balanced(dialog));
    expect(warning(dialog)).toBeNull();
    fireEvent.click(automatic(dialog));
    expect(warning(dialog)).toHaveTextContent(ONE);
    // Either choice taken back, it goes.
    fireEvent.click(strict(dialog));
    expect(warning(dialog)).toBeNull();
    fireEvent.click(balanced(dialog));
    expect(warning(dialog)).toHaveTextContent(ONE);
    fireEvent.click(automatic(dialog));
    expect(warning(dialog)).toBeNull();
  });

  it('comes and goes with automatic renaming switched on, for ready documents, and counts those in review once Balanced is chosen too', async () => {
    renderDialog(older, { ready: 2, review: 1 });
    const dialog = await screen.findByRole('dialog', { name: 'Settings' });
    expect(warning(dialog)).toBeNull();
    fireEvent.click(automatic(dialog));
    expect(warning(dialog)).toHaveTextContent(several(2));
    fireEvent.click(balanced(dialog));
    expect(warning(dialog)).toHaveTextContent(several(3));
    fireEvent.click(strict(dialog));
    expect(warning(dialog)).toHaveTextContent(several(2));
    fireEvent.click(automatic(dialog));
    expect(warning(dialog)).toBeNull();
  });

  it('counts only documents in review when automatic renaming was on already', async () => {
    renderDialog({ ...older, automaticRename: true, reviewLevel: 'strict' }, { ready: 4, review: 1 });
    const dialog = await screen.findByRole('dialog', { name: 'Settings' });
    // Nothing this save changes: the ready ones are renamed with or without it.
    expect(warning(dialog)).toBeNull();
    fireEvent.click(balanced(dialog));
    expect(warning(dialog)).toHaveTextContent(ONE);
  });

  it('never warns of Balanced back to Strict, of automatic renaming switched off, or of a save that changes neither', async () => {
    renderDialog({ ...older, automaticRename: true, reviewLevel: 'balanced' }, { ready: 3, review: 3 });
    const dialog = await screen.findByRole('dialog', { name: 'Settings' });
    expect(warning(dialog)).toBeNull();
    fireEvent.click(strict(dialog));
    expect(warning(dialog)).toBeNull();
    fireEvent.click(automatic(dialog));
    expect(warning(dialog)).toBeNull();
    fireEvent.click(balanced(dialog));
    expect(warning(dialog)).toBeNull();
    // Another setting changed: still nothing of this.
    fireEvent.change(within(dialog).getByLabelText("This machine's name"), { target: { value: 'Reception' } });
    expect(warning(dialog)).toBeNull();
  });

  it('is absent where nothing is typed, and where Settings is given no count', async () => {
    const first = renderDialog(older, NOTHING_TYPED);
    let dialog = await screen.findByRole('dialog', { name: 'Settings' });
    fireEvent.click(balanced(dialog));
    fireEvent.click(automatic(dialog));
    expect(warning(dialog)).toBeNull();
    expect(within(dialog).queryByRole('alert')).not.toBeInTheDocument();
    first.unmount();

    renderDialog(older);
    dialog = await screen.findByRole('dialog', { name: 'Settings' });
    fireEvent.click(balanced(dialog));
    fireEvent.click(automatic(dialog));
    expect(within(dialog).queryByRole('alert')).not.toBeInTheDocument();
  });

  it('follows the count while Settings is open, and leaves the choices as they are', async () => {
    const { rerender } = renderDialog(older, { ready: 0, review: 1 });
    const dialog = await screen.findByRole('dialog', { name: 'Settings' });
    fireEvent.click(balanced(dialog));
    fireEvent.click(automatic(dialog));
    expect(warning(dialog)).toHaveTextContent(ONE);
    // A document becomes ready with something typed for it.
    rerender(older, { ready: 1, review: 1 });
    expect(warning(dialog)).toHaveTextContent(several(2));
    // Both are read again, or filed from another computer.
    rerender(older, NOTHING_TYPED);
    expect(warning(dialog)).toBeNull();
    // One comes back from being read.
    rerender(older, { ready: 0, review: 1 });
    expect(warning(dialog)).toHaveTextContent(ONE);
    expect(balanced(dialog)).toBeChecked();
    expect(automatic(dialog)).toBeChecked();
  });

  // App reads the settings a render after it mounts. What was loaded is what
  // arrived, not the placeholders the dialog opened with.
  it('goes by the settings that arrive after it opened: automatic renaming that was on already is not this save\'s doing', async () => {
    const { rerender } = renderDialog({ ...older, reviewLevel: 'strict' }, { ready: 1, review: 0 });
    const dialog = await screen.findByRole('dialog', { name: 'Settings' });
    fireEvent.click(automatic(dialog));
    expect(warning(dialog)).toHaveTextContent(ONE);
    rerender({ ...older, automaticRename: true, reviewLevel: 'strict' }, { ready: 1, review: 0 });
    expect(automatic(dialog)).toBeChecked();
    expect(warning(dialog)).toBeNull();
  });

  it('is heard when it appears and when its count changes, and stands above Save, outside the choice of level', async () => {
    const { rerender } = renderDialog(older, { ready: 1, review: 0 });
    const dialog = await screen.findByRole('dialog', { name: 'Settings' });
    fireEvent.click(automatic(dialog));
    const shown = warning(dialog)!;
    // An alert, as the dialog's other notices that matter before going on: read out when it is put on the page.
    expect(shown).toHaveAttribute('role', 'alert');
    expect(shown).toHaveClass('save-warning');
    // An error's red is for something that went wrong.
    expect(shown).not.toHaveClass('form-error');
    // In the footer, straight before Save, which stays in view wherever the sections are scrolled to.
    expect(shown.parentElement).toHaveClass('dialog-actions');
    expect(shown.nextElementSibling).toBe(saveButton(dialog));
    expect(dialog.querySelector('.dialog-body')!.contains(shown)).toBe(false);
    // Not in the choice of level, whose words hold no figure.
    expect(level(dialog).contains(shown)).toBe(false);
    expect(level(dialog).textContent).not.toMatch(/\d/);
    // The same element says the new count, so the change is read out too.
    rerender(older, { ready: 3, review: 0 });
    expect(warning(dialog)).toBe(shown);
    expect(shown).toHaveTextContent(several(3));
  });

  // The page has no layout here, so the heights are given: one document's
  // warning takes four lines, several documents' five. What is checked is
  // what Settings does with them. That nothing moves, and that nothing comes
  // under a pointer at rest where the warning was, is run in a browser
  // (tests/e2e/savingOverTyped.spec.ts).
  it('never gives back the room it took while Settings is open: a shorter count keeps the taller box, and its room stays above Save, blank, once it has gone', async () => {
    const rect = (height: number) => ({ top: 400, bottom: 400 + height, left: 0, right: 0, x: 0, y: 400, width: 0, height, toJSON: () => ({}) }) as DOMRect;
    const drawn = (box: HTMLElement) => Math.max(box.textContent?.startsWith('1 document') ? 88 : 106, parseFloat(box.style.minHeight) || 0);
    const layout = vi.spyOn(HTMLElement.prototype, 'getBoundingClientRect').mockImplementation(function (this: HTMLElement) {
      return rect(this.classList.contains('save-warning') ? drawn(this) : 0);
    });
    try {
      const { rerender } = renderDialog(older, { ready: 1, review: 0 });
      const dialog = await screen.findByRole('dialog', { name: 'Settings' });
      const sections = dialog.querySelector<HTMLElement>('.dialog-body')!;
      const footer = dialog.querySelector<HTMLElement>('.dialog-actions')!;
      const room = () => footer.querySelector<HTMLElement>(':scope > div[aria-hidden="true"]');
      // The sections are never given anything of it: they end as they did before it was first shown.
      const sectionsEnd = () => `${sections.lastElementChild?.lastElementChild?.textContent}, ${sections.querySelectorAll('[aria-hidden="true"]:empty').length} blank`;
      const ended = sectionsEnd();
      expect(ended).toBe('Opens in your browser., 0 blank');
      expect(room()).toBeNull();
      fireEvent.click(automatic(dialog));
      expect(warning(dialog)).toHaveTextContent(ONE);
      expect(warning(dialog)).toHaveStyle({ minHeight: '88px' });
      // More documents, more lines: the box grows, and that is its room from then on.
      rerender(older, { ready: 3, review: 0 });
      expect(warning(dialog)).toHaveTextContent(several(3));
      expect(warning(dialog)).toHaveStyle({ minHeight: '106px' });
      // Fewer lines again: the box stays as tall as it was.
      rerender(older, { ready: 1, review: 0 });
      expect(warning(dialog)).toHaveTextContent(ONE);
      expect(warning(dialog)).toHaveStyle({ minHeight: '106px' });
      expect(room()).toBeNull();
      expect(footer.children).toHaveLength(2);

      // Gone by itself: its room is still straight above Save, as tall as its box was, with nothing in it,
      // and nothing of it is read out. So nothing that was scrolled behind the warning is uncovered.
      rerender(older, NOTHING_TYPED);
      expect(warning(dialog)).toBeNull();
      expect(within(dialog).queryByRole('alert')).not.toBeInTheDocument();
      expect(room()).toHaveStyle({ height: '106px' });
      expect(room()).toBeEmptyDOMElement();
      expect(room()).toHaveClass('save-warning-room');
      expect(room()!.nextElementSibling).toBe(saveButton(dialog));
      expect(footer.children).toHaveLength(2);
      expect(sectionsEnd()).toBe(ended);

      // Back by itself: in the room it had, which is the warning's again.
      rerender(older, { ready: 1, review: 0 });
      expect(warning(dialog)).toHaveStyle({ minHeight: '106px' });
      expect(warning(dialog)!.nextElementSibling).toBe(saveButton(dialog));
      expect(room()).toBeNull();
      expect(footer.children).toHaveLength(2);

      // Gone by the choice taken back: the same room is kept, and kept when the other choice is changed too.
      fireEvent.click(automatic(dialog));
      expect(warning(dialog)).toBeNull();
      expect(room()).toHaveStyle({ height: '106px' });
      fireEvent.click(balanced(dialog));
      expect(room()).toHaveStyle({ height: '106px' });
      expect(footer.children).toHaveLength(2);
      expect(sectionsEnd()).toBe(ended);
    } finally {
      layout.mockRestore();
    }
  });

  // Places given, as above: the warning is 88px tall with Save 12px under it,
  // so the footer grows by 100px; the sections' view ended at 708 before, and
  // ends at 608 once it has. The checkbox's line is 26px, and the checkbox
  // itself the 16px that end 5px above the line's end, as it is drawn. At
  // first the line ends at 628, 20px behind the footer.
  it('scrolls the sections to show the line of the control just used when the person\'s own choice brings the warning and that control was in view, and never when it comes by itself or the control was out of view already', async () => {
    const rect = (top: number, height: number) => ({ top, bottom: top + height, left: 0, right: 0, x: 0, y: top, width: 0, height, toJSON: () => ({}) }) as DOMRect;
    let lineEnds = 628;
    const layout = vi.spyOn(HTMLElement.prototype, 'getBoundingClientRect').mockImplementation(function (this: HTMLElement) {
      if (this.classList.contains('save-warning')) return rect(623, 88);
      if (this.classList.contains('dialog-body')) return rect(91, 517);
      if (this.tagName === 'LABEL' && this.textContent === 'Automatically rename ready documents') return rect(lineEnds - 26, 26);
      if (this.tagName === 'INPUT' && this.closest('label')?.textContent === 'Automatically rename ready documents') return rect(lineEnds - 21, 16);
      // Save is under the warning, in the footer, 12px below its box.
      if (this.textContent === 'Save settings') return rect(723, 42);
      // Not where it is drawn: given the checkbox's place, to see that a control outside the sections is not
      // scrolled for, wherever it is.
      if (this.getAttribute('aria-label') === 'Close settings') return rect(lineEnds - 26, 26);
      return rect(0, 0);
    });
    const open = async (typed: TypedWaiting) => {
      const view = renderDialog(older, typed);
      const dialog = await screen.findByRole('dialog', { name: 'Settings' });
      return { ...view, dialog, sections: dialog.querySelector<HTMLElement>('.dialog-body')! };
    };
    try {
      // The person's own choice, with the focus on the checkbox they ticked: its line and 4px under it are shown.
      let settings = await open({ ready: 1, review: 0 });
      automatic(settings.dialog).focus();
      fireEvent.click(automatic(settings.dialog));
      expect(warning(settings.dialog)).toHaveTextContent(ONE);
      expect(settings.sections.scrollTop).toBe(24);
      // Taken back and chosen again: the room is the footer's already, so nothing is covered anew and nothing scrolls.
      fireEvent.click(automatic(settings.dialog));
      fireEvent.click(automatic(settings.dialog));
      expect(warning(settings.dialog)).toHaveTextContent(ONE);
      expect(settings.sections.scrollTop).toBe(24);
      settings.unmount();

      // By itself: the choice was made with nothing to warn of, and a document comes back to wait. The focus is
      // still on the checkbox, behind the footer now, and the sections do not move for it.
      settings = await open(NOTHING_TYPED);
      automatic(settings.dialog).focus();
      fireEvent.click(automatic(settings.dialog));
      expect(warning(settings.dialog)).toBeNull();
      settings.rerender(older, { ready: 1, review: 0 });
      expect(warning(settings.dialog)).toHaveTextContent(ONE);
      expect(settings.sections.scrollTop).toBe(0);
      // Nor when a later choice of the person's changes the warning without making it taller.
      fireEvent.click(balanced(settings.dialog));
      expect(settings.sections.scrollTop).toBe(0);
      settings.unmount();

      // The focus on a control that is not in the sections: nothing of theirs to keep in view.
      for (const outside of ['Save settings', 'Close settings']) {
        settings = await open({ ready: 1, review: 0 });
        within(settings.dialog).getByRole('button', { name: outside }).focus();
        fireEvent.click(automatic(settings.dialog));
        expect(warning(settings.dialog)).toHaveTextContent(ONE);
        expect(settings.sections.scrollTop, outside).toBe(0);
        settings.unmount();
      }

      // The checkbox's line whole above the footer: it is in view as it is.
      lineEnds = 600;
      settings = await open({ ready: 1, review: 0 });
      automatic(settings.dialog).focus();
      fireEvent.click(automatic(settings.dialog));
      expect(warning(settings.dialog)).toHaveTextContent(ONE);
      expect(settings.sections.scrollTop).toBe(0);
      settings.unmount();

      // What decides is whether the checkbox itself was in view before the footer grew, when the view ended at
      // 708. Its line need not have been: the scroll is by all that is cut of the line, and 4px.
      for (const [ends, scrolled, how] of [
        [708, 104, 'the line just covered, to the last pixel the footer took'],
        [709, 105, 'the line cut off by a pixel before, the checkbox itself in view'],
        // Where the keyboard leaves it: the browser shows the checkbox the focus comes to and no more, so it
        // ends at the view's end, 708, and 5px of its line are cut off. The scroll is more than the footer grew.
        [713, 109, 'the checkbox ending at the view\'s end, its line cut off under it'],
        // One pixel past the end still counts as in view: the browser scrolls by whole pixels, and can leave a
        // control it has brought into view a part of one past the end.
        [714, 110, 'the checkbox one pixel past the view\'s end'],
        // Two pixels past: it was out of view before the footer grew, and the sections are not sent after it.
        [715, 0, 'the checkbox itself cut off before'],
      ] as const) {
        lineEnds = ends;
        settings = await open({ ready: 1, review: 0 });
        automatic(settings.dialog).focus();
        fireEvent.click(automatic(settings.dialog));
        expect(warning(settings.dialog)).toHaveTextContent(ONE);
        expect(settings.sections.scrollTop, how).toBe(scrolled);
        settings.unmount();
      }

      // The warning grown by a line, by the person's own choice. Balanced itself is the 16px at the top of its
      // line, 2px down, and its words stand under it.
      lineEnds = 620;
      const lines = vi.spyOn(HTMLElement.prototype, 'getBoundingClientRect').mockImplementation(function (this: HTMLElement) {
        if (this.classList.contains('save-warning')) return this.textContent?.startsWith('1 document') ? rect(623, 88) : rect(605, 106);
        if (this.classList.contains('dialog-body')) return rect(91, this.closest('.settings-dialog')?.querySelector('.save-warning')?.textContent?.startsWith('1 document') ? 517 : 499);
        if (this.tagName === 'LABEL' && this.textContent?.startsWith('Balanced')) return rect(lineEnds - 145, 145);
        if (this.tagName === 'INPUT' && this.closest('label')?.textContent?.startsWith('Balanced')) return rect(lineEnds - 143, 16);
        if (this.textContent === 'Save settings') return rect(723, 42);
        return rect(0, 0);
      });
      try {
        settings = await open({ ready: 1, review: 2 });
        fireEvent.click(automatic(settings.dialog));
        expect(warning(settings.dialog)).toHaveTextContent(ONE);
        expect(settings.sections.scrollTop).toBe(0);
        // Balanced adds the two in review: five lines, 18px more of footer, and the view ends at 590. Its line
        // ends at 620, 30px behind the footer, of which the footer took only 18 just now: 12px were cut off
        // before. Balanced itself was in view, so its line is shown whole: 34px, more than the footer grew.
        balanced(settings.dialog).focus();
        fireEvent.click(balanced(settings.dialog));
        expect(warning(settings.dialog)).toHaveTextContent(several(3));
        expect(settings.sections.scrollTop).toBe(34);
        settings.unmount();
        // Ending at 600, the line is 10px behind the footer, all of it just taken: shown again.
        lineEnds = 600;
        settings = await open({ ready: 1, review: 2 });
        fireEvent.click(automatic(settings.dialog));
        balanced(settings.dialog).focus();
        fireEvent.click(balanced(settings.dialog));
        expect(warning(settings.dialog)).toHaveTextContent(several(3));
        expect(settings.sections.scrollTop).toBe(14);
        settings.unmount();
        // Balanced itself out of view before the footer grew, when the view ended at 608: with its line ending
        // at 737 it ended at 610, 2px past, one more than the pixel allowed, and it is left where it is. One
        // pixel higher it counts as in view, and its line is shown whole.
        for (const [ends, scrolled] of [[737, 0], [736, 150]] as const) {
          lineEnds = ends;
          settings = await open({ ready: 1, review: 2 });
          fireEvent.click(automatic(settings.dialog));
          balanced(settings.dialog).focus();
          fireEvent.click(balanced(settings.dialog));
          expect(warning(settings.dialog)).toHaveTextContent(several(3));
          expect(settings.sections.scrollTop, `Balanced's line ending at ${ends}`).toBe(scrolled);
          settings.unmount();
        }
      } finally {
        lines.mockRestore();
      }
    } finally {
      layout.mockRestore();
    }
  });

  it('takes no room before it has been shown', async () => {
    renderDialog(older, { ready: 1, review: 0 });
    const dialog = await screen.findByRole('dialog', { name: 'Settings' });
    expect(dialog.querySelector('.dialog-body div[aria-hidden="true"]')).toBeNull();
    expect(dialog.querySelector('.dialog-actions')!.children).toHaveLength(1);
    expect(dialog.querySelector('.save-warning-room')).toBeNull();
    // Nor for a choice that warns of nothing: Balanced alone, with a document typed for.
    fireEvent.click(balanced(dialog));
    expect(dialog.querySelector('.dialog-actions')!.children).toHaveLength(1);
  });

  it('is words only: no button, nothing that acts, and Save is not held back', async () => {
    renderDialog(older, { ready: 1, review: 2 });
    const dialog = await screen.findByRole('dialog', { name: 'Settings' });
    const buttons = within(dialog).getAllByRole('button').length;
    fireEvent.click(balanced(dialog));
    fireEvent.click(automatic(dialog));
    const shown = warning(dialog)!;
    expect(shown).toHaveTextContent(several(3));
    expect(shown.querySelector('button, a, input, [tabindex]')).toBeNull();
    expect(shown.closest('[data-acts]')).toBeNull();
    expect(dialog.querySelector('[data-acts]')).toBeNull();
    expect(within(dialog).getAllByRole('button')).toHaveLength(buttons);
    expect(saveButton(dialog)).toBeEnabled();
    // Tab goes past it: from the last control of the sections to Save, as before.
    const guide = within(dialog).getByRole('button', { name: 'Open the guide' });
    guide.focus();
    fireEvent.keyDown(guide, { key: 'Tab' });
    expect(saveButton(dialog)).toHaveFocus();
  });

  it('changes nothing of what Save sends', async () => {
    const choose = async () => {
      const dialog = await screen.findByRole('dialog', { name: 'Settings' });
      fireEvent.click(balanced(dialog));
      fireEvent.click(automatic(dialog));
      return dialog;
    };
    // As it was before the warning: Settings given no count.
    const before = renderDialog(older);
    let dialog = await choose();
    expect(warning(dialog)).toBeNull();
    fireEvent.click(saveButton(dialog));
    await waitFor(() => expect(before.onSave).toHaveBeenCalledOnce());
    before.unmount();

    const warned = renderDialog(older, { ready: 2, review: 3 });
    dialog = await choose();
    expect(warning(dialog)).toHaveTextContent(several(5));
    fireEvent.click(saveButton(dialog));
    await waitFor(() => expect(warned.onSave).toHaveBeenCalledOnce());
    expect(warned.onSave.mock.calls).toEqual(before.onSave.mock.calls);
    // The whole settings object with the two choices, and nothing beside it.
    expect(warned.onSave.mock.calls[0]).toEqual([{ ...older, reviewLevel: 'balanced', automaticRename: true }]);
  });
});

async function openSettings() {
  fireEvent.click((await screen.findAllByRole('button', { name: 'Settings' }))[0]);
  const dialog = await screen.findByRole('dialog', { name: 'Settings' });
  fireEvent.click(within(dialog).getByText('Naming and filing', { selector: 'summary' }));
  return dialog;
}
const closeSettings = async (dialog: HTMLElement) => {
  fireEvent.keyDown(saveButton(dialog), { key: 'Escape' });
  await waitFor(() => expect(screen.queryByRole('dialog', { name: 'Settings' })).not.toBeInTheDocument());
};

describe('the count Settings is given by the window', () => {
  const LEASE = 'Lease Agreement - 123 Main St.pdf';
  const EMPLOYMENT = 'Employment Agreement - John Smith.pdf';
  const typeName = async (document: string, name: string) => {
    const panel = await openDetails(document);
    fireEvent.change(within(panel).getByLabelText('Filename'), { target: { value: name } });
    return panel;
  };

  it('is of the documents with something typed and not filed: one waiting in review, then a ready one as well', async () => {
    render(<App bridge={createInMemoryBridge()} />);
    await screen.findByRole('grid', { name: 'Queue' });
    // Nothing typed: no warning, whatever is chosen.
    let dialog = await openSettings();
    fireEvent.click(balanced(dialog));
    fireEvent.click(automatic(dialog));
    expect(warning(dialog)).toBeNull();
    await closeSettings(dialog);

    await typeName(LEASE, '2023-09-15 Lease of 123 Main St');
    dialog = await openSettings();
    expect(warning(dialog)).toBeNull();
    // The lease waits in review: automatic renaming alone does not reach it.
    fireEvent.click(automatic(dialog));
    expect(warning(dialog)).toBeNull();
    fireEvent.click(balanced(dialog));
    expect(warning(dialog)).toHaveTextContent(ONE);
    fireEvent.click(strict(dialog));
    expect(warning(dialog)).toBeNull();
    await closeSettings(dialog);

    await typeName(EMPLOYMENT, '2024-04-12 Employment Agreement of John Smith');
    dialog = await openSettings();
    fireEvent.click(automatic(dialog));
    expect(warning(dialog)).toHaveTextContent(ONE);
    fireEvent.click(balanced(dialog));
    expect(warning(dialog)).toHaveTextContent(several(2));
  });

  it('no longer counts a document whose typed name was taken back, or filed', async () => {
    const base = createInMemoryBridge();
    render(<App bridge={base} />);
    await screen.findByRole('grid', { name: 'Queue' });
    let panel = await typeName(EMPLOYMENT, '2024-04-12 Employment Agreement of John Smith');
    let dialog = await openSettings();
    fireEvent.click(automatic(dialog));
    expect(warning(dialog)).toHaveTextContent(ONE);
    await closeSettings(dialog);

    fireEvent.click(within(panel).getByRole('button', { name: "Back to Intern's proposal" }));
    dialog = await openSettings();
    fireEvent.click(automatic(dialog));
    expect(warning(dialog)).toBeNull();
    await closeSettings(dialog);

    // Typed again and filed under what was typed: the person's own rename.
    panel = await typeName(EMPLOYMENT, '2024-04-12 Employment Agreement of John Smith');
    fireEvent.click(within(panel).getByRole('button', { name: /^Apply rename/ }));
    await waitFor(async () => expect((await base.listItems()).find((item) => item.id === 'employment')?.status).toBe('completed'));
    dialog = await openSettings();
    fireEvent.click(automatic(dialog));
    expect(warning(dialog)).toBeNull();
  });

  // The queue changes with nobody asking while Settings is open, and the
  // warning follows it: what was typed stays typed, set aside, but a document
  // being read is not one Intern can rename.
  it('follows the queue while Settings is open: gone while the document is read again, back when it waits again', async () => {
    const base = createInMemoryBridge({ liveEvents: true }) as ReturnType<typeof createInMemoryBridge> & QueueEventSource;
    const changes = new Map<string, Partial<QueueItem>>();
    let queueChanged = () => {};
    const bridge = {
      ...base,
      listItems: async () => (await base.listItems()).map((item) => ({ ...item, ...changes.get(item.id) }) as QueueItem),
      subscribeQueue: (listener: Parameters<QueueEventSource['subscribeQueue']>[0]) => { queueChanged = () => listener({ type: 'changed' }); return base.subscribeQueue(listener); },
    };
    const byItself = async (id: string, change: Partial<QueueItem>) => {
      changes.set(id, change);
      await act(async () => { queueChanged(); await new Promise((resolve) => setTimeout(resolve, 0)); });
    };
    render(<App bridge={bridge} />);
    await screen.findByRole('grid', { name: 'Queue' });
    await typeName(LEASE, '2023-09-15 Lease of 123 Main St');
    const dialog = await openSettings();
    fireEvent.click(balanced(dialog));
    fireEvent.click(automatic(dialog));
    expect(warning(dialog)).toHaveTextContent(ONE);

    await byItself('lease', { status: 'processing', stage: 'reading', proposedFilename: undefined, description: undefined, reason: undefined });
    await waitFor(() => expect(warning(dialog)).toBeNull());
    // The choices stand, and Settings is still open.
    expect(balanced(dialog)).toBeChecked();
    expect(automatic(dialog)).toBeChecked();

    await byItself('lease', { status: 'review', proposedFilename: '2023-09-15 Lease between ABC Properties and TenantCo.pdf', description: 'A lease of 123 Main St.', proposalRevision: '1' });
    await waitFor(() => expect(warning(dialog)).toHaveTextContent(ONE));

    // Filed from another computer: completed, and no longer counted.
    await byItself('lease', { status: 'completed', filedName: '2023-09-15 Lease between ABC Properties and TenantCo.pdf', undoable: true });
    await waitFor(() => expect(warning(dialog)).toBeNull());
  });
});

/*
  A queue row as the desktop backend sends it (QueueItemDto in
  src-tauri/src/commands.rs, filled by queue_item_dto). Every field without a
  skip attribute is there, and holds null where the backend has nothing: a
  document with no proposal has a `proposedFilename` key, and it is null. The
  demo leaves such keys out, which is how a count that looked only for a
  missing key came to count a document Intern has no name for.
*/
type Sent = Record<string, unknown> & { id: string };
const sentRow = (id: number, originalFilename: string, status: string, more: Record<string, unknown> = {}): Sent => ({
  id: String(id), originalFilename, status,
  proposedFilename: null, confidence: null, description: null, evidence: null, reason: null, errorCode: null,
  undoable: false, approved: false, proposalRevision: null, reconciliation: null,
  filedPath: null, filedName: null, keptOriginal: false, parked: false,
  ...more,
});
/** What a reading adds to a row: a name, a sentence, and where in the document each part was found. */
const sentProposal = (proposedFilename: string, description: string, confidence: number) => ({ proposedFilename, description, confidence, proposalRevision: '1', evidence: { date: null, type: null, parties: null } });
const desktopOver = (sent: () => Sent[]) => new TauriBridge({ invoke: async <T,>(command: string) => { if (command !== 'queue_list') throw new Error(`not played out here: ${command}`); return sent().map((row) => ({ ...row })) as T; }, listen: async () => () => {} });

describe('the count over rows as the desktop backend sends them', () => {
  it('leaves out a document with no proposal, whose name arrives as null, and counts the others as ever', async () => {
    const sent = [
      sentRow(1, 'Scan 0601.pdf', 'needs_review', { ...sentProposal('2025-05-01 Letter to Halvorsen Fixture Works LLC.pdf', 'A letter.', 1), reason: 'TYPE_INFERRED, LOW_CONFIDENCE' }),
      sentRow(2, 'Scan 0602.pdf', 'ready', sentProposal('2025-05-02 Invoice from Brightwater Supply Co.pdf', 'An invoice.', 0.98)),
      sentRow(3, 'Scan 0603.pdf', 'needs_review', { ...sentProposal('2025-05-03 Notice to Contoso Ltd.pdf', 'A notice.', 0.9), reason: 'DATE_AMBIGUOUS' }),
      // In review with no proposal at all: could not be read, sent by someone not verified, a duplicate found before reading.
      sentRow(4, 'NEEDS REVIEW - Lease scan.pdf', 'needs_review', { errorCode: 'PASSWORD_PROTECTED', fileModifiedDate: '2026-01-02' }),
      sentRow(5, 'Scan 0605.pdf', 'needs_review', { errorCode: 'UPLOADER_UNVERIFIED' }),
      sentRow(6, 'Scan 0606.pdf', 'needs_review', { errorCode: 'DUPLICATE', reason: 'Duplicate of 2025-05-02 Invoice from Brightwater Supply Co.pdf' }),
      // Ready, but not Intern's to rename by itself: approved by a person, and a staged copy.
      sentRow(7, 'Scan 0607.pdf', 'ready', { ...sentProposal('2025-05-07 Memo.pdf', 'A memo.', 0.97), approved: true }),
      sentRow(8, 'Scan 0608.pdf', 'ready', { ...sentProposal('COPY - 2025-05-02 Invoice from Brightwater Supply Co.pdf', '', 0), confidence: null, copyOf: '2025-05-02 Invoice from Brightwater Supply Co.pdf' }),
      // Not waiting for a name.
      sentRow(9, 'Scan 0609.pdf', 'completed', { ...sentProposal('2025-05-09 Receipt.pdf', 'A receipt.', 0.99), filedName: '2025-05-09 Receipt.pdf', filedPath: 'C:\\Filed\\2025-05-09 Receipt.pdf', undoable: true }),
      sentRow(10, 'Scan 0610.pdf', 'failed', { errorCode: 'MODEL_FAILED' }),
      sentRow(11, 'Scan 0611.pdf', 'analyzing'),
      sentRow(12, 'Scan 0612.pdf', 'queued'),
    ];
    const items = await desktopOver(() => sent).listItems();
    expect(items).toHaveLength(12);
    // Everything typed for: only the letter and the notice in review, and the invoice that is ready.
    expect(typedWaiting(items, () => true)).toEqual({ ready: 1, review: 2 });
    const typedFor = (...ids: number[]) => (id: string) => ids.map(String).includes(id);
    expect(typedWaiting(items, typedFor(4, 5, 6))).toEqual(NOTHING_TYPED);
    expect(typedWaiting(items, typedFor(4))).toEqual(NOTHING_TYPED);
    expect(typedWaiting(items, typedFor(5))).toEqual(NOTHING_TYPED);
    expect(typedWaiting(items, typedFor(1, 4))).toEqual({ ready: 0, review: 1 });
    expect(typedWaiting(items, typedFor(7, 8, 9, 10, 11, 12))).toEqual(NOTHING_TYPED);
  });
});

/*
  What the warning says of afterwards, run: a document Intern renames by
  itself while it has something typed for it. The demo does not act out
  automatic renaming, so the queue here is rows shaped as the desktop backend
  sends them (`sentRow`: every key it always sends, null where it has
  nothing), read through TauriBridge, with the backend's part played out by
  hand: judged again as ready, renamed under Intern's name with its reasons
  kept, and an undo that puts it back in review. No backend ran.
*/
describe('a document Intern renamed by itself while it had something typed', () => {
  const TYPED_NAME = '2025-05-01 Letter to Halvorsen about the fixtures';
  const TYPED_SENTENCE = 'Their reply to our notice of 12 April.';

  async function start() {
    const sent: Sent[] = [
      sentRow(31, 'Scan 0601.pdf', 'needs_review', { ...sentProposal('2025-05-01 Letter to Halvorsen Fixture Works LLC.pdf', 'Letter to Halvorsen Fixture Works LLC.', 1), reason: 'TYPE_INFERRED, LOW_CONFIDENCE' }),
      sentRow(32, 'Scan 0602.pdf', 'ready', sentProposal('2025-05-02 Invoice from Brightwater Supply Co.pdf', 'Invoice from Brightwater Supply Co.', 0.98)),
      sentRow(33, 'Scan 0603.pdf', 'needs_review', { ...sentProposal('2025-05-03 Notice to Contoso Ltd.pdf', 'Notice to Contoso Ltd.', 0.9), reason: 'DATE_AMBIGUOUS' }),
      // Could not be read: it waits in review with no proposal, so its name is null.
      sentRow(34, 'NEEDS REVIEW - Lease scan.pdf', 'needs_review', { errorCode: 'PASSWORD_PROTECTED', fileModifiedDate: '2026-01-02' }),
    ];
    const change = (id: number, to: Record<string, unknown>) => { const index = sent.findIndex((row) => row.id === String(id)); sent[index] = { ...sent[index], ...to }; };
    const desktop = desktopOver(() => sent);
    const base = createInMemoryBridge({ liveEvents: true }) as ReturnType<typeof createInMemoryBridge> & QueueEventSource;
    let settings: AppSettings = { ...(await base.getSettings()) };
    let queueChanged = () => {};
    const saveSettings = vi.fn(async (next: AppSettings) => {
      settings = { ...next };
      // Judged again at Balanced: its type and the model's doubt no longer hold it.
      if (next.reviewLevel === 'balanced') change(31, { status: 'ready' });
    });
    const bridge = {
      ...base,
      listItems: () => desktop.listItems(),
      getSettings: async () => ({ ...settings }),
      saveSettings,
      approve: vi.fn(base.approve),
      // As an undone rename is left: back under its own name, waiting, with UNDONE beside what it carried.
      undo: vi.fn(async (id: string) => {
        const row = sent.find((each) => each.id === id)!;
        change(Number(row.id), { status: 'needs_review', reason: [row.reason, 'UNDONE'].filter(Boolean).join(', '), filedName: null, filedPath: null, undoable: false });
      }),
      subscribeQueue: (listener: Parameters<QueueEventSource['subscribeQueue']>[0]) => { queueChanged = () => listener({ type: 'changed' }); return base.subscribeQueue(listener); },
    };
    /** The backend does something with nobody asking, and says the queue changed. */
    const byItself = async (run: () => void) => { run(); await act(async () => { queueChanged(); await new Promise((resolve) => setTimeout(resolve, 0)); }); };
    const renaming = (id: number) => change(id, { status: 'applying' });
    const renamed = (id: number) => { const row = sent.find((each) => each.id === String(id))!; change(id, { status: 'completed', filedName: row.proposedFilename, filedPath: `C:\\Filed\\${String(row.proposedFilename)}`, keptOriginal: false, undoable: true }); };
    render(<App bridge={bridge} />);
    await screen.findByRole('grid', { name: 'Queue' });
    return { bridge, byItself, renaming, renamed };
  }
  const saveWith = async (choose: (dialog: HTMLElement) => void) => {
    const dialog = await openSettings();
    choose(dialog);
    const said = warning(dialog)?.textContent;
    fireEvent.click(saveButton(dialog));
    await waitFor(() => expect(screen.queryByRole('dialog', { name: 'Settings' })).not.toBeInTheDocument());
    await act(async () => { await new Promise((resolve) => setTimeout(resolve, 0)); });
    return said;
  };
  const kept = () => screen.queryAllByLabelText<HTMLTextAreaElement>('What you had typed');

  it('keeps what was typed with the document, read-only, under Intern\'s name: from review, by way of Balanced', async () => {
    const { bridge, byItself, renaming, renamed } = await start();
    const panel = await openDetails('Scan 0601.pdf');
    fireEvent.change(within(panel).getByLabelText('Filename'), { target: { value: TYPED_NAME } });
    fireEvent.change(within(panel).getByLabelText('Description'), { target: { value: TYPED_SENTENCE } });

    // The warning said so before the save, and the save went through as chosen.
    expect(await saveWith((dialog) => { fireEvent.click(balanced(dialog)); fireEvent.click(automatic(dialog)); })).toBe(ONE);
    expect(bridge.saveSettings).toHaveBeenCalledWith(expect.objectContaining({ reviewLevel: 'balanced', automaticRename: true }));
    // Ready now, and what was typed is still in its fields.
    await waitFor(() => expect(queueCell('Scan 0601.pdf', 'status')).toHaveTextContent('Ready with a note'));
    expect(within(panel).getByLabelText('Filename')).toHaveValue(TYPED_NAME);
    expect(kept()).toHaveLength(0);

    // Intern renames it. The fields go; what they held is set aside, where typing changes nothing.
    await byItself(() => renaming(31));
    await waitFor(() => expect(within(panel).queryByLabelText('Filename')).not.toBeInTheDocument());
    expect(kept()).toHaveLength(1);
    expect(kept()[0]).toHaveValue(`${TYPED_NAME}\n\n${TYPED_SENTENCE}`);
    expect(kept()[0]).toHaveAttribute('readonly');

    await byItself(() => renamed(31));
    await waitFor(() => expect(panel).toHaveTextContent('Renamed to 2025-05-01 Letter to Halvorsen Fixture Works LLC.pdf'));
    // Under Intern's name, not the one typed; and nobody approved anything.
    expect(panel).not.toHaveTextContent(`Renamed to ${TYPED_NAME}`);
    expect(bridge.approve).not.toHaveBeenCalled();
    expect(kept()).toHaveLength(1);
    expect(kept()[0]).toHaveValue(`${TYPED_NAME}\n\n${TYPED_SENTENCE}`);
    expect(panel).toContainElement(kept()[0]);
    expect(within(panel).getByRole('group', { name: 'Kept from before this document changed' })).toHaveTextContent('They are kept here as you had them, name first, and nothing here is filed.');
    // The window says it happened, where it says what an action did, and keeps nothing above the queue.
    expect(screen.getByRole('group', { name: 'Renamed without you' })).toHaveTextContent("Scan 0601.pdf was renamed under Intern's proposal. What you typed for it was not filed, and is kept in its details.");
    expect(screen.queryByRole('group', { name: /^Kept from Scan/ })).not.toBeInTheDocument();

    // It is with the document wherever that is listed: in Completed, opened again.
    fireEvent.click(screen.getByRole('button', { name: /^Completed, / }));
    expect(queueCell('Scan 0601.pdf', 'status')).toHaveTextContent('Completed with a note');
    const filed = await openDetails('Scan 0601.pdf');
    expect(within(filed).getByLabelText('What you had typed')).toHaveValue(`${TYPED_NAME}\n\n${TYPED_SENTENCE}`);
    expect(within(filed).getByRole('button', { name: 'Undo' })).toBeEnabled();
  });

  it('says so when the document is not on screen, keeps what was typed all the same, and still has it after Undo: a ready one, by way of automatic renaming', async () => {
    const { bridge, byItself, renamed } = await start();
    const panel = await openDetails('Scan 0602.pdf');
    fireEvent.change(within(panel).getByLabelText('Filename'), { target: { value: '2025-05-02 Invoice 4471 from Brightwater' } });
    fireEvent.click(screen.getByRole('button', { name: 'Details' }));
    await waitFor(() => expect(screen.queryByRole('complementary', { name: 'Review item' })).not.toBeInTheDocument());

    expect(await saveWith((dialog) => { fireEvent.click(automatic(dialog)); })).toBe(ONE);
    await byItself(() => renamed(32));
    await waitFor(() => expect(screen.getByRole('button', { name: 'Completed, 1' })).toBeInTheDocument());
    // Said where the outcome of an action is; nothing else: the document has only left the queue for Completed.
    expect(screen.getByRole('group', { name: 'Renamed without you' })).toHaveTextContent("Scan 0602.pdf was renamed under Intern's proposal. What you typed for it was not filed, and is kept in its details.");
    expect([...document.querySelector('.toasts')!.children].map((each) => each.getAttribute('aria-label'))).toEqual(['Renamed without you']);
    expect(kept()).toHaveLength(0);
    expect(screen.queryByRole('group', { name: /^Kept from/ })).not.toBeInTheDocument();

    fireEvent.click(screen.getByRole('button', { name: 'Completed, 1' }));
    const filed = await openDetails('Scan 0602.pdf');
    expect(filed).toHaveTextContent('Renamed to 2025-05-02 Invoice from Brightwater Supply Co.pdf');
    expect(within(filed).getByLabelText('What you had typed')).toHaveValue('2025-05-02 Invoice 4471 from Brightwater\n\nInvoice from Brightwater Supply Co.');

    // Undone, it waits in review under Intern's proposal, and what was typed is still beside it, not in the field.
    fireEvent.click(within(filed).getByRole('button', { name: 'Undo' }));
    await waitFor(() => expect(bridge.undo).toHaveBeenCalledWith('32'));
    await waitFor(() => expect(screen.getByRole('button', { name: 'Completed, 0' })).toBeInTheDocument());
    // No longer renamed, so no longer said to be.
    expect(screen.queryByRole('group', { name: 'Renamed without you' })).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: /^Queue, / }));
    const again = await openDetails('Scan 0602.pdf');
    expect(queueCell('Scan 0602.pdf', 'status')).toHaveTextContent('Needs review');
    expect(within(again).getByLabelText('Filename')).toHaveValue('2025-05-02 Invoice from Brightwater Supply Co');
    expect(within(again).getByLabelText('What you had typed')).toHaveValue('2025-05-02 Invoice 4471 from Brightwater\n\nInvoice from Brightwater Supply Co.');
  });

  // Intern proposed no name for it, so there is none to rename it under, at
  // either level: the warning would say something untrue of it.
  it('is not warned of for a document Intern could not read, whatever is typed for it: its name arrives as null', async () => {
    await start();
    const lease = await openDetails('NEEDS REVIEW - Lease scan.pdf');
    fireEvent.change(within(lease).getByLabelText('Filename'), { target: { value: '2026-01-02 Lease typed by hand' } });
    let dialog = await openSettings();
    fireEvent.click(automatic(dialog));
    expect(warning(dialog)).toBeNull();
    fireEvent.click(balanced(dialog));
    expect(warning(dialog)).toBeNull();
    expect(within(dialog).queryByRole('alert')).not.toBeInTheDocument();
    await closeSettings(dialog);

    // A letter in review that Intern does have a name for, typed for as well: that one is counted, and only it.
    const letter = await openDetails('Scan 0601.pdf');
    fireEvent.change(within(letter).getByLabelText('Filename'), { target: { value: TYPED_NAME } });
    dialog = await openSettings();
    fireEvent.click(automatic(dialog));
    expect(warning(dialog)).toBeNull();
    fireEvent.click(balanced(dialog));
    expect(warning(dialog)).toHaveTextContent(ONE);
  });
});
