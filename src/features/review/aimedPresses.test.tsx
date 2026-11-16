import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { App } from '../../App';
import { ReviewInspector } from '../../components/ReviewInspector';
import { Toast } from '../../components/Toast';
import { TypedKept } from '../../components/TypedKept';
import { SETTLE_MS, takeOnlyAimedPresses } from '../../lib/aimedPress';
import { createInMemoryBridge, createReviewBatchBridge } from '../../lib/inMemoryBridge';
import { openDetails } from '../queue/gridTesting';
import type { QueueItem } from '../../types';

/*
  A press of the pointer on a control that acts is taken only when it can
  have been aimed at it (src/lib/aimedPress.ts), and what acts is said where
  each control is drawn (`data-acts`). So that the marks cannot fall behind
  the window, every button the queue and the review panel can show is
  listed here as acting or not, and a button nobody has listed fails.
*/

/** Whether a button acts - decides, files, removes, discards, cancels, retries or renames - by its accessible name. */
const ACTS: Array<[RegExp, boolean]> = [
  [/^(Approve & rename|Apply rename|Keep original|Analyze again|Check again|Process anyway|Retry|Retry item|Cancel processing|Undo)$/, true],
  [/^(Remove from queue|Remove item|I have resolved the files myself)$/, true],
  [/^(Use this date|Use date & rename|Back to Intern's proposal|Back to the name it came with)$/, true],
  [/^(Discard what I had|Done)$/, true],
  // A date or a party to put in the name.
  [/^(\d{4}-\d{2}-\d{2}( · file date)?|Contoso Ltd|Fabrikam Inc)$/, true],
  [/^(Rename all ready|Discard waiting items|Clear history|Undo this rename|Undo these \d+ renames)$/, true],
  // A rename by itself over what was typed: its Undo, and its Done above.
  [/^(Undo the rename of .+|Undo the renames of these \d+ documents)$/, true],
  [/^Install .+ and restart$/, true],
  // These only move the view, open something, or take back a question.
  [/^(Close review|Next undecided|Open|Show in folder|More review actions|Copy description|Cancel|Dismiss|Not now)$/, false],
  [/^(Queue, \d+|Completed, \d+|\d+ needs? review|\d+ ready|Copy table|Copy selection|Details|More queue actions|History)$/, false],
  [/^(Add files|Add folder|Pause queue|Resume queue|Help & support|Settings|Private · .*)$/, false],
];

/** A button's accessible name: its label, or its text without the hidden parts (icons, key hints, counts). */
function nameOf(button: HTMLElement): string {
  return button.getAttribute('aria-label') ?? [...button.childNodes]
    .filter((node) => !(node instanceof Element && node.getAttribute('aria-hidden') === 'true'))
    .map((node) => node.textContent).join('').trim();
}

/** Every button on the page that is marked otherwise than the list says, or is not on the list. */
function mismarked(): string[] {
  return screen.getAllByRole('button', { hidden: true }).concat(screen.queryAllByRole('menuitem')).flatMap((button) => {
    const name = nameOf(button);
    const listed = ACTS.find(([names]) => names.test(name));
    if (!listed) return [`"${name}" is not said to act or not`];
    return button.hasAttribute('data-acts') === listed[1] ? [] : [`"${name}" ${listed[1] ? 'acts and is not marked' : 'is marked and does not act'}`];
  });
}

function handlers() {
  return {
    onClose: vi.fn(), onApprove: vi.fn(), onKeep: vi.fn(), onCancel: vi.fn(), onRetry: vi.fn(),
    onReanalyze: vi.fn(), onRemove: vi.fn(), onUndo: vi.fn(), onOpen: vi.fn(), onReveal: vi.fn(),
  };
}

const proposal = { proposedFilename: '2024-03-01 Lease Agreement with Northwind Traders.pdf', confidence: 0.8, description: 'A lease.' };
const review: QueueItem = { id: 'review', originalFilename: 'scan.pdf', status: 'review', ...proposal, reason: 'The model was unsure of its reading.', errorCode: 'LOW_CONFIDENCE' };

describe('the controls that act', () => {
  afterEach(() => cleanup());

  it('are marked, each one, in every state the review panel shows a document in', () => {
    const states: Array<[string, QueueItem]> = [
      ['needs review', review],
      ['ready', { ...review, status: 'ready', reason: undefined, errorCode: undefined }],
      ['its files need checking', { ...review, parked: true, errorCode: 'FILE_CHANGED' }],
      ['a duplicate, which can be retried', { ...review, errorCode: 'DUPLICATE' }],
      ['could not be read, with no proposal', { id: 'unreadable', originalFilename: 'NEEDS REVIEW - scan.pdf', status: 'review', errorCode: 'PASSWORD_PROTECTED', reason: 'It is protected by a password.' }],
      ['no date in the name: one suggested, others stated, and the file\'s', { ...review, proposedFilename: 'Lease Agreement with Northwind Traders.pdf', suggestedDate: '2024-03-01' }],
      ['no date in the name, and none suggested', { ...review, proposedFilename: 'Lease Agreement with Northwind Traders.pdf', datesInDocument: ['2024-03-01', '2024-04-15'], fileModifiedDate: '2024-05-02' }],
      ['other parties to choose from', { ...review, partyChoices: ['Contoso Ltd', 'Fabrikam Inc'], nameParties: ['Northwind Traders'] }],
      ['being read', { id: 'processing', originalFilename: 'scan.pdf', status: 'processing', stage: 'reading', cancelable: true }],
      ['waiting', { id: 'waiting', originalFilename: 'scan.pdf', status: 'waiting' }],
      ['failed', { id: 'failed', originalFilename: 'scan.pdf', status: 'failed', errorCode: 'MODEL_FAILED', reason: 'The model stopped.' }],
      ['filed', { ...review, status: 'completed', filedName: proposal.proposedFilename, undoable: true, reason: undefined, errorCode: undefined }],
    ];
    const found = new Set<string>();
    for (const [state, item] of states) {
      const { container, unmount } = render(<ReviewInspector item={item} drawer={false} position={{ index: 1, total: 3 }} onNext={vi.fn()} {...handlers()} />);
      // With what was typed, the menu open, and then the question before a removal.
      const name = container.querySelector<HTMLTextAreaElement>('textarea[aria-label="Filename"]');
      if (name) fireEvent.change(name, { target: { value: `${name.value} signed` } });
      const more = screen.queryByRole('button', { name: 'More review actions' });
      if (more) fireEvent.click(more);
      expect(mismarked(), state).toEqual([]);
      screen.getAllByRole('button').forEach((button) => found.add(nameOf(button)));
      const remove = screen.queryByRole('button', { name: /^Remove/ });
      if (remove && item.parked) {
        fireEvent.click(remove);
        expect(mismarked(), `${state}, asked before removing`).toEqual([]);
        screen.getAllByRole('button').forEach((button) => found.add(nameOf(button)));
      }
      unmount();
    }
    // The states above do show what the list is about: none of these went unseen.
    for (const name of ['Approve & rename', 'Apply rename', 'Keep original', 'Analyze again', 'Check again', 'Process anyway', 'Retry', 'Retry item', 'Cancel processing', 'Undo', 'Remove from queue', 'Remove item', 'I have resolved the files myself', 'Use this date', 'Use date & rename', "Back to Intern's proposal", 'Back to the name it came with', '2024-03-01', '2024-05-02 · file date', 'Contoso Ltd']) expect([...found], name).toContain(name);
  });

  it('are marked on what is kept of a name, on a result that can be undone, and across the queue\'s own window', async () => {
    const kept = { key: 1, text: 'A name\n\nA description', edited: true };
    render(<TypedKept kept={kept} label="Kept from before this document changed" dismiss="Discard what I had" onDismiss={vi.fn()}>Kept.</TypedKept>);
    render(<TypedKept kept={{ ...kept, key: 2 }} label="Kept from scan.pdf" dismiss="Done" onDismiss={vi.fn()}>Kept.</TypedKept>);
    render(<Toast tone="success" label="Action result" action={{ label: 'Undo', name: 'Undo these 3 renames', onClick: vi.fn() }} onDismiss={vi.fn()}>Renamed 3 documents.</Toast>);
    render(<Toast byItself tone="success" label="Renamed without you" action={{ label: 'Undo', name: 'Undo the rename of scan.pdf', onClick: vi.fn() }} onDismiss={vi.fn()}>scan.pdf was renamed.</Toast>);
    render(<Toast byItself tone="success" label="Renamed without you" action={{ label: 'Undo', name: 'Undo the renames of these 2 documents', onClick: vi.fn() }} onDismiss={vi.fn()}>2 documents were renamed.</Toast>);
    expect(mismarked()).toEqual([]);
    cleanup();

    render(<App bridge={createInMemoryBridge({ update: { state: 'available', currentVersion: '0.1.0', version: '0.2.0' } })} />);
    await screen.findByRole('gridcell', { name: 'Lease Agreement - 123 Main St.pdf' });
    await openDetails('Lease Agreement - 123 Main St.pdf');
    await screen.findByRole('button', { name: /^Install .+ and restart$/ });
    fireEvent.click(screen.getByRole('button', { name: 'More queue actions' }));
    expect(mismarked()).toEqual([]);
    for (const name of ['Rename all ready', 'Discard waiting items']) expect(screen.getAllByRole('button').concat(screen.getAllByRole('menuitem')).map(nameOf)).toContain(name);
    fireEvent.click(screen.getByRole('button', { name: /^Completed, / }));
    await waitFor(() => expect(screen.getByRole('button', { name: 'Clear history' })).toBeInTheDocument());
    expect(mismarked()).toEqual([]);
  });

  it('name Approve & rename and Keep original as the same two buttons for every document, and no other so', () => {
    render(<ReviewInspector item={review} drawer={false} {...handlers()} />);
    fireEvent.click(screen.getByRole('button', { name: 'More review actions' }));

    const named = screen.getAllByRole('button').filter((button) => button.hasAttribute('data-acts') && button.getAttribute('data-acts') !== 'true');
    expect(named.map((button) => `${nameOf(button)}: ${button.getAttribute('data-acts')}`)).toEqual(['Approve & rename: approve', 'Keep original: keep']);
    // Each says which document it is drawn for: back after a moment for the same one, it has arrived (aimedPress).
    expect(named.map((button) => button.getAttribute('data-acts-for'))).toEqual(['review', 'review']);
    expect(screen.getAllByRole('button').filter((button) => button.hasAttribute('data-acts-for'))).toEqual(named);
  });
});

/*
  Approve, keep, remove, discard and "Back to Intern's proposal" act on a
  first press only: the second click of a double-click lands on whatever the
  first brought under the pointer. A press that was refused as not aimed did
  nothing, and the browser counts the press made again at once as the second
  click of a run. That one is a first press (`firstPress`). The rule is
  applied to the window here as the app applies it, with a clock the test
  sets; a page that lays nothing out (jsdom) has every control arrive when
  it is first drawn.
*/
describe('a button that acts on a first press only', () => {
  let now = 0;
  let stop = () => {};
  beforeEach(() => { now = 0; stop = takeOnlyAimedPresses(window, () => now); });
  afterEach(() => { stop(); cleanup(); });

  /** A whole press of the mouse on a control, as a browser sends it, counted `count` in its run of clicks. Whether the press went down. */
  const press = (control: Element, count: number) => {
    const at = { clientX: 10, clientY: 10, button: 0, buttons: 1, pointerId: 1, isPrimary: true, pointerType: 'mouse' };
    // A pointerdown that is refused sends no mouse events after it.
    const down = fireEvent.pointerDown(control, at);
    if (down) fireEvent.mouseDown(control, { ...at, detail: count });
    fireEvent.pointerUp(control, { ...at, buttons: 0 });
    if (down) fireEvent.mouseUp(control, { ...at, buttons: 0, detail: count });
    fireEvent.click(control, { ...at, buttons: 0, detail: count });
    return down;
  };
  /** The window is told of what was drawn, as a browser tells it. */
  const drawn = async () => { await Promise.resolve(); };

  it('approves once when the press is refused and made again at once: the press the browser counts second acts, and the third does not', async () => {
    const on = handlers();
    now = 10_000;
    render(<ReviewInspector item={review} drawer={false} {...on} />);
    await drawn();
    const approve = screen.getByRole('button', { name: /Approve & rename/ });
    const keep = screen.getByRole('button', { name: /Keep original/ });

    // The button has only just come there: refused.
    now = 10_300;
    expect(press(approve, 1)).toBe(false);
    expect(on.onApprove).not.toHaveBeenCalled();
    // Made again within the double-click time, and once more: on Keep original, as if that had come under the pointer.
    now = 10_700;
    expect(press(approve, 2)).toBe(true);
    expect(on.onApprove).toHaveBeenCalledTimes(1);
    now = 10_900;
    expect(press(approve, 3)).toBe(true);
    expect(press(keep, 3)).toBe(true);
    expect(on.onApprove).toHaveBeenCalledTimes(1);
    expect(on.onKeep).not.toHaveBeenCalled();
  });

  it('approves one document at a double-click, however slow the second click', async () => {
    const base = createReviewBatchBridge();
    const bridge = { ...base, approve: vi.fn(base.approve), keepOriginal: vi.fn(base.keepOriginal) };
    render(<App bridge={bridge} />);
    await openDetails();
    const panel = () => screen.getByRole('complementary', { name: 'Review item' });
    await waitFor(() => expect(panel()).toHaveTextContent('Scan 0101.pdf'));
    const approve = () => screen.getByRole('button', { name: /Approve & rename/ });

    now = 10_000;
    expect(press(approve(), 1)).toBe(true);
    await waitFor(() => expect(panel()).toHaveTextContent('Scan 0102.pdf'));
    await waitFor(() => expect(approve()).toBeEnabled());
    // The second click, after the settle time (a person whose double-click time is longer): the rule takes the press.
    now = 10_000 + SETTLE_MS + 100;
    expect(press(approve(), 2)).toBe(true);
    expect(press(screen.getByRole('button', { name: /Keep original/ }), 2)).toBe(true);
    expect(bridge.approve).toHaveBeenCalledTimes(1);
    expect(bridge.keepOriginal).not.toHaveBeenCalled();
    expect(panel()).toHaveTextContent('Scan 0102.pdf');
  });

  it('discards what was kept, and puts a name back as proposed, at the press made again after a refused one', async () => {
    const dismissed = vi.fn();
    now = 10_000;
    render(<TypedKept kept={{ key: 1, text: 'A name\n\nA description', edited: true }} label="Kept from before this document changed" dismiss="Discard what I had" onDismiss={dismissed}>Kept.</TypedKept>);
    await drawn();
    const discard = screen.getByRole('button', { name: 'Discard what I had' });
    now = 10_300;
    expect(press(discard, 1)).toBe(false);
    expect(dismissed).not.toHaveBeenCalled();
    now = 10_700;
    expect(press(discard, 2)).toBe(true);
    expect(dismissed).toHaveBeenCalledTimes(1);
    now = 10_900;
    expect(press(discard, 3)).toBe(true);
    expect(dismissed).toHaveBeenCalledTimes(1);
    cleanup();

    now = 20_000;
    const { container } = render(<ReviewInspector item={review} drawer={false} {...handlers()} />);
    const name = () => container.querySelector<HTMLTextAreaElement>('textarea[aria-label="Filename"]')!;
    const proposed = name().value;
    fireEvent.change(name(), { target: { value: `${proposed} signed` } });
    await drawn();
    const back = screen.getByRole('button', { name: "Back to Intern's proposal" });
    now = 20_300;
    expect(press(back, 1)).toBe(false);
    expect(name()).toHaveValue(`${proposed} signed`);
    now = 20_700;
    expect(press(back, 2)).toBe(true);
    expect(name()).toHaveValue(proposed);
  });
});
