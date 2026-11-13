import { cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { App } from '../../App';
import { ReviewInspector } from '../../components/ReviewInspector';
import { PRESS_SETTLES_MS } from '../../components/UseOnItsOwn';
import { openRow } from '../queue/gridTesting';
import { drawYes, moveTo, offTheYes, onTheYes, pressAt } from './pointerTesting';
import type { Point } from './pointerTesting';
import { createInMemoryBridge } from '../../lib/inMemoryBridge';
import { humanizeReason } from '../../lib/reasons';
import type { HouseRule, LearnedRule, QueueItem } from '../../types';

const selectRow = openRow;

const respelled: QueueItem = {
  id: 'sow',
  originalFilename: 'SOW final v3.pdf',
  status: 'ready',
  proposedFilename: '2026-04-01 Statement of Work between Ridgeline Cartography LLC and Contoso.pdf',
  confidence: 0.93,
  description: 'Statement of work between Ridgeline Cartography LLC and Contoso Worldwide, Inc. for the 2026 member-map engagement.',
  evidence: { date: 'effective as of April 1, 2026', type: 'STATEMENT OF WORK', parties: 'Ridgeline Cartography LLC; Contoso Worldwide, Inc.' },
  houseRules: [{ kind: 'party', from: 'Contoso Worldwide, Inc.', to: 'Contoso' }],
};

describe('a name in the reviewer\'s own spelling', () => {
  it('says which spelling was applied while the evidence keeps the document\'s words', async () => {
    render(<App bridge={createInMemoryBridge({ items: [respelled] })} />);
    await selectRow(await screen.findByRole('row', { name: /SOW final v3.pdf/i }));

    const note = screen.getByRole('note', { name: 'Learned spellings applied' });
    expect(note).toHaveTextContent('Uses your spelling: Contoso Worldwide, Inc. written as Contoso. Change or forget it under Settings.');
    expect(screen.getByLabelText('Filename')).toHaveValue('2026-04-01 Statement of Work between Ridgeline Cartography LLC and Contoso');
    fireEvent.click(screen.getByText('Analysis details'));
    const evidence = screen.getByRole('heading', { name: 'Evidence' }).closest('section')!;
    expect(within(evidence).getByText('Contoso Worldwide, Inc.')).toBeVisible();
  });

  // A spelling ending in its own period ("Inc.") is the end of the list:
  // the sentence takes no second one. It read "Receiver, Inc.. Change or
  // forget it under Settings."
  it('ends the note with one period after a spelling that ends in one', async () => {
    const abbreviated: QueueItem = { ...respelled, houseRules: [{ kind: 'party', from: 'Contoso Worldwide, Inc.', to: 'Contoso, Inc.' }] };
    render(<App bridge={createInMemoryBridge({ items: [abbreviated] })} />);
    await selectRow(await screen.findByRole('row', { name: /SOW final v3.pdf/i }));

    const note = screen.getByRole('note', { name: 'Learned spellings applied' });
    expect(note).toHaveTextContent('Uses your spelling: Contoso Worldwide, Inc. written as Contoso, Inc. Change or forget it under Settings.');
    expect(note.textContent).not.toContain('..');
  });

  it('shows no note for a name in the document\'s own words', async () => {
    render(<App bridge={createInMemoryBridge()} />);
    await selectRow(await screen.findByRole('row', { name: /Lease Agreement - 123 Main St.pdf/i }));
    expect(screen.queryByRole('note', { name: 'Learned spellings applied' })).not.toBeInTheDocument();
  });
});

function handlers() {
  return {
    onClose: vi.fn(), onApprove: vi.fn(), onKeep: vi.fn(), onCancel: vi.fn(), onRetry: vi.fn(),
    onReanalyze: vi.fn(), onRemove: vi.fn(), onUndo: vi.fn(), onOpen: vi.fn(), onReveal: vi.fn(),
  };
}

/** A press with the pointer where it is: it focuses the button and counts `presses`. */
const press = (button: HTMLElement, presses: number) => { button.focus(); fireEvent.click(button, { detail: presses }); };

/** A statement of work held for a spelling nobody has checked, as the backend lists it. */
const held: QueueItem = {
  ...respelled,
  status: 'review',
  errorCode: 'LEARNED_SPELLING_UNCONFIRMED',
  reasonCodes: ['LEARNED_SPELLING_UNCONFIRMED'],
  reason: humanizeReason('LEARNED_SPELLING_UNCONFIRMED'),
  proposedFilename: '2026-04-01 SOW between Ridgeline Cartography and Contoso.pdf',
  houseRules: [
    { kind: 'party', from: 'Contoso Worldwide, Inc.', to: 'Contoso' },
    { kind: 'party', from: 'Ridgeline Cartography LLC', to: 'Ridgeline Cartography' },
    { kind: 'document_type', from: 'Statement of Work', to: 'SOW' },
  ],
};

/** The learned spellings: the first two the held name used, unchecked; the third used and checked; one more unchecked that it did not use. */
const spellings: LearnedRule[] = [
  { id: '1', kind: 'party', from: 'Contoso Worldwide, Inc.', to: 'Contoso', seen: 2, active: true, confirmed: 0, learnedAt: 1 },
  { id: '2', kind: 'party', from: 'Ridgeline Cartography LLC', to: 'Ridgeline Cartography', seen: 2, active: true, confirmed: 0, learnedAt: 2 },
  { id: '3', kind: 'document_type', from: 'Statement of Work', to: 'SOW', seen: 2, active: true, confirmed: 1, learnedAt: 3 },
  { id: '4', kind: 'party', from: 'Northwind Traders', to: 'Northwind', seen: 2, active: true, confirmed: 0, learnedAt: 4 },
];

/** A bridge holding `rules`, whose check behaves as the backend's: exact, in use, or refused. */
function spellingBridge(rules: LearnedRule[]) {
  let current = rules.map((rule) => ({ ...rule }));
  const houseRulesList = vi.fn(async () => current.map((rule) => ({ ...rule })));
  const houseRuleCheck = vi.fn(async (id: string, shown: HouseRule) => {
    const rule = current.find((candidate) => candidate.id === id);
    if (!rule) throw { code: 'RULE_NOT_FOUND', message: 'learned spelling does not exist' };
    if (!rule.active || rule.kind !== shown.kind || rule.from !== shown.from || rule.to !== shown.to) throw { code: 'RULE_CHANGED', message: 'learned spelling is not in use as it was shown' };
    current = current.map((candidate) => candidate.id === id ? { ...candidate, confirmed: candidate.confirmed + 1 } : candidate);
  });
  return { houseRulesList, houseRuleCheck, respell(id: string, to: string) { current = current.map((rule) => rule.id === id ? { ...rule, to } : rule); } };
}

// No approval checks a learned spelling. A document held for one shows,
// beside "Uses your spelling", "Use on its own" for each spelling nobody has
// checked that its name used - the only way to stop being asked - after the
// question Settings asks. It checks the spelling, as shown, and nothing else:
// the document is neither approved nor filed.
describe('using a held document\'s spelling on its own', () => {
  afterEach(() => cleanup());

  it('is offered only for the unchecked spellings the name used, beside the pointer to Settings', async () => {
    const bridge = spellingBridge(spellings);
    render(<ReviewInspector item={held} drawer={false} {...handlers()} spellings={bridge} />);
    expect(await screen.findByRole('button', { name: 'Use on its own: Contoso Worldwide, Inc. written as Contoso' })).toBeVisible();
    expect(screen.getByRole('button', { name: 'Use on its own: Ridgeline Cartography LLC written as Ridgeline Cartography' })).toBeVisible();
    // Not the checked one, nor one the name did not use.
    expect(screen.getAllByRole('button', { name: /^Use on its own:/ })).toHaveLength(2);
    expect(screen.getByRole('note', { name: 'Learned spellings applied' })).toHaveTextContent('Change or forget it under Settings.');
  });

  it('is not offered for a document not held for a spelling', async () => {
    const bridge = spellingBridge(spellings);
    const { rerender } = render(<ReviewInspector item={{ ...held, status: 'ready', errorCode: undefined, reasonCodes: undefined, reason: undefined }} drawer={false} {...handlers()} spellings={bridge} />);
    rerender(<ReviewInspector item={{ ...held, errorCode: 'LOW_CONFIDENCE', reasonCodes: ['LOW_CONFIDENCE'], reason: humanizeReason('LOW_CONFIDENCE') }} drawer={false} {...handlers()} spellings={bridge} />);
    await Promise.resolve();
    expect(screen.queryByRole('button', { name: /Use on its own/ })).not.toBeInTheDocument();
    expect(bridge.houseRulesList).not.toHaveBeenCalled();
  });

  it('asks first, and Cancel or Escape take the question back and nothing else', async () => {
    const bridge = spellingBridge(spellings);
    const actions = handlers();
    render(<ReviewInspector item={held} drawer {...actions} spellings={bridge} />);
    const action = await screen.findByRole('button', { name: 'Use on its own: Contoso Worldwide, Inc. written as Contoso' });
    fireEvent.click(action);
    const question = screen.getByRole('group', { name: 'Use on its own: Contoso Worldwide, Inc. written as Contoso' });
    expect(question).toHaveAccessibleDescription('Every later document Intern reads as naming Contoso Worldwide, Inc. will name Contoso instead, and may be filed under that name without your check. Do this only if that is right for every one of them. Choosing this does not approve this document: it still waits for you.');
    expect(within(question).getByRole('button', { name: 'Cancel' })).toHaveFocus();

    fireEvent.click(within(question).getByRole('button', { name: 'Cancel' }));
    expect(screen.queryByRole('group', { name: /^Use on its own:/ })).not.toBeInTheDocument();
    await waitFor(() => expect(screen.getByRole('button', { name: 'Use on its own: Contoso Worldwide, Inc. written as Contoso' })).toHaveFocus());

    fireEvent.click(screen.getByRole('button', { name: 'Use on its own: Contoso Worldwide, Inc. written as Contoso' }));
    fireEvent.keyDown(screen.getByRole('button', { name: 'Cancel' }), { key: 'Escape' });
    expect(screen.queryByRole('group', { name: /^Use on its own:/ })).not.toBeInTheDocument();
    await waitFor(() => expect(screen.getByRole('button', { name: 'Use on its own: Contoso Worldwide, Inc. written as Contoso' })).toHaveFocus());
    // The drawer stays open: Escape took back the question only.
    expect(actions.onClose).not.toHaveBeenCalled();
    expect(bridge.houseRuleCheck).not.toHaveBeenCalled();
  });

  it('checks the spelling as shown, and neither approves nor files the document', async () => {
    const bridge = spellingBridge(spellings);
    const actions = handlers();
    const onSpellingChecked = vi.fn();
    render(<ReviewInspector item={held} drawer={false} {...actions} spellings={bridge} onSpellingChecked={onSpellingChecked} />);
    fireEvent.click(await screen.findByRole('button', { name: 'Use on its own: Ridgeline Cartography LLC written as Ridgeline Cartography' }));
    fireEvent.click(screen.getByRole('button', { name: 'Use on its own' }));
    await waitFor(() => expect(bridge.houseRuleCheck).toHaveBeenCalledWith('2', { kind: 'party', from: 'Ridgeline Cartography LLC', to: 'Ridgeline Cartography' }));
    expect(await screen.findByRole('status')).toHaveTextContent('Intern will use \u201cRidgeline Cartography\u201d on its own from now on. This document still waits for you.');
    await waitFor(() => expect(screen.queryByRole('button', { name: /Use on its own: Ridgeline/ })).not.toBeInTheDocument());
    expect(screen.getByRole('button', { name: /Use on its own: Contoso/ })).toBeVisible();
    expect(onSpellingChecked).toHaveBeenCalledOnce();
    expect(actions.onApprove).not.toHaveBeenCalled();
    expect(bridge.houseRuleCheck).toHaveBeenCalledOnce();
    await waitFor(() => expect(screen.getByRole('group', { name: 'Use a learned spelling on its own' })).toHaveFocus());
  });

  // The panel used to scroll the question clear of the decision, which
  // brought its yes under the pointer that pressed the action. The second
  // press of a double-click, or a press straight after the first, used the
  // spelling on its own with the sentence never read: from then on a wrong
  // spelling's names were filed without review. The yes now takes a press
  // only from a pointer that came onto it (below); these are the rules
  // behind that one, each with the pointer moved onto the yes first.
  describe('takes as a yes only a press meant for it', () => {
    afterEach(() => vi.restoreAllMocks());

    async function ask(bridge: ReturnType<typeof spellingBridge>, clock: { now: number }) {
      vi.spyOn(performance, 'now').mockImplementation(() => clock.now);
      drawYes();
      render(<ReviewInspector item={held} drawer={false} {...handlers()} spellings={bridge} />);
      pressAt(await screen.findByRole('button', { name: 'Use on its own: Contoso Worldwide, Inc. written as Contoso' }), offTheYes);
      const question = screen.getByRole('group', { name: 'Use on its own: Contoso Worldwide, Inc. written as Contoso' });
      const yes = within(question).getByRole('button', { name: 'Use on its own' });
      moveTo(onTheYes);
      return { question, yes, cancel: within(question).getByRole('button', { name: 'Cancel' }) };
    }

    it('not the second press of a double-click, however slow', async () => {
      const bridge = spellingBridge(spellings);
      const clock = { now: 1000 };
      const { question, yes, cancel } = await ask(bridge, clock);
      clock.now += PRESS_SETTLES_MS * 4;
      press(yes, 2);
      press(yes, 3);
      expect(question).toBeInTheDocument();
      expect(cancel).toHaveFocus();
      expect(bridge.houseRuleCheck).not.toHaveBeenCalled();
    });

    it('not a press straight after the question appeared, nor straight after a press it did not take', async () => {
      const bridge = spellingBridge(spellings);
      const clock = { now: 1000 };
      const { question, yes, cancel } = await ask(bridge, clock);
      clock.now += PRESS_SETTLES_MS - 1;
      press(yes, 1);
      expect(cancel).toHaveFocus();
      // Clicking on and on is no answer either.
      clock.now += PRESS_SETTLES_MS - 1;
      press(yes, 1);
      expect(question).toBeInTheDocument();
      expect(cancel).toHaveFocus();
      expect(bridge.houseRuleCheck).not.toHaveBeenCalled();

      // A press once the sentence had time to be read is a yes.
      clock.now += PRESS_SETTLES_MS;
      press(yes, 1);
      await waitFor(() => expect(bridge.houseRuleCheck).toHaveBeenCalledWith('1', { kind: 'party', from: 'Contoso Worldwide, Inc.', to: 'Contoso' }));
      expect(await screen.findByRole('status')).toHaveTextContent('Intern will use \u201cContoso\u201d on its own from now on.');
    });

    it('not a press begun straight after the question appeared, however long it was held', async () => {
      const bridge = spellingBridge(spellings);
      const clock = { now: 1000 };
      const { question, yes, cancel } = await ask(bridge, clock);
      clock.now += 100;
      fireEvent.pointerDown(yes, { ...onTheYes, pointerType: 'mouse' });
      yes.focus();
      // Let go well after the half second.
      clock.now += PRESS_SETTLES_MS + 100;
      fireEvent.click(yes, { detail: 1 });
      expect(question).toBeInTheDocument();
      expect(cancel).toHaveFocus();
      expect(bridge.houseRuleCheck).not.toHaveBeenCalled();

      // One begun once the sentence had time to be read is a yes.
      clock.now += PRESS_SETTLES_MS;
      fireEvent.pointerDown(yes, { ...onTheYes, pointerType: 'mouse' });
      yes.focus();
      fireEvent.click(yes, { detail: 1 });
      await waitFor(() => expect(bridge.houseRuleCheck).toHaveBeenCalledOnce());
    });

    it('but a yes from the keyboard at once', async () => {
      const bridge = spellingBridge(spellings);
      const { yes } = await ask(bridge, { now: 1000 });
      // Tab to the button, and Enter on it, which clicks it with no presses counted.
      fireEvent.keyDown(document.activeElement!, { key: 'Tab' });
      yes.focus();
      fireEvent.keyDown(yes, { key: 'Enter' });
      fireEvent.click(yes, { detail: 0 });
      await waitFor(() => expect(bridge.houseRuleCheck).toHaveBeenCalledOnce());
    });
  });

  // Every rule above was about where the yes was put under the pointer, and
  // each round found another way it came there: the panel's scroll, a row
  // moving up in Settings, a question opened from the keyboard scrolled
  // under a resting mouse. The yes takes a press of the pointer only once
  // the pointer came onto it after the question appeared - moved onto it
  // from off it, as it is at that moment - however it got there.
  describe('takes a press only from a pointer that came onto the yes', () => {
    afterEach(() => vi.restoreAllMocks());

    /** Open the question with a press at `at`, then wait out the half second. */
    async function ask(bridge: ReturnType<typeof spellingBridge>, at: Point, pointerType = 'mouse') {
      const clock = { now: 1000 };
      vi.spyOn(performance, 'now').mockImplementation(() => clock.now);
      const box = drawYes();
      render(<ReviewInspector item={held} drawer={false} {...handlers()} spellings={bridge} />);
      pressAt(await screen.findByRole('button', { name: 'Use on its own: Contoso Worldwide, Inc. written as Contoso' }), at, pointerType);
      const question = screen.getByRole('group', { name: 'Use on its own: Contoso Worldwide, Inc. written as Contoso' });
      clock.now += PRESS_SETTLES_MS * 2;
      return { box, clock, question, yes: within(question).getByRole('button', { name: 'Use on its own' }), cancel: within(question).getByRole('button', { name: 'Cancel' }) };
    }

    it('not when the question opened under a pointer that has not moved since', async () => {
      const bridge = spellingBridge(spellings);
      const { question, yes, cancel } = await ask(bridge, onTheYes);
      pressAt(yes, onTheYes);
      expect(question).toBeInTheDocument();
      expect(cancel).toHaveFocus();
      expect(bridge.houseRuleCheck).not.toHaveBeenCalled();
    });

    it('when the pointer moved onto it from off it', async () => {
      const bridge = spellingBridge(spellings);
      const { yes } = await ask(bridge, offTheYes);
      moveTo(onTheYes);
      pressAt(yes, onTheYes);
      await waitFor(() => expect(bridge.houseRuleCheck).toHaveBeenCalledWith('1', { kind: 'party', from: 'Contoso Worldwide, Inc.', to: 'Contoso' }));
    });

    it('not when the yes moved under a pointer that stayed put, whatever the browser says of it', async () => {
      const bridge = spellingBridge(spellings);
      const { box, clock, question, yes, cancel } = await ask(bridge, offTheYes);
      // The layout changes and the yes comes under the pointer.
      box.current = new DOMRect(100, 160, 120, 30);
      pressAt(yes, offTheYes);
      expect(cancel).toHaveFocus();
      // The move a browser sends to where the pointer already was.
      clock.now += PRESS_SETTLES_MS * 2;
      moveTo(offTheYes);
      pressAt(yes, offTheYes);
      // Nor a hand on the mouse, a pixel from where it was, still on the yes.
      clock.now += PRESS_SETTLES_MS * 2;
      moveTo({ clientX: offTheYes.clientX + 1, clientY: offTheYes.clientY });
      pressAt(yes, { clientX: offTheYes.clientX + 1, clientY: offTheYes.clientY });
      expect(question).toBeInTheDocument();
      expect(cancel).toHaveFocus();
      expect(bridge.houseRuleCheck).not.toHaveBeenCalled();

      // Off it and back onto it, where it is now, is a pointer that came onto it.
      clock.now += PRESS_SETTLES_MS * 2;
      moveTo({ clientX: 60, clientY: 120 });
      moveTo(offTheYes);
      pressAt(yes, offTheYes);
      await waitFor(() => expect(bridge.houseRuleCheck).toHaveBeenCalledOnce());
    });

    // A mouse or a pen hovers: it comes onto the yes by moving. A press where
    // it was not seen to move to - the window moved under it - is not one.
    it('not a mouse press on it where the pointer was not seen to move to', async () => {
      const bridge = spellingBridge(spellings);
      const { question, yes, cancel } = await ask(bridge, offTheYes);
      pressAt(yes, onTheYes);
      expect(question).toBeInTheDocument();
      expect(cancel).toHaveFocus();
      expect(bridge.houseRuleCheck).not.toHaveBeenCalled();
    });

    // The yes opened under the pointer and moved off it; the pointer, never
    // seen off the yes, follows it there. It has to be seen off it first.
    it('not from a pointer never seen off the yes since it appeared, even one the yes moved away from', async () => {
      const bridge = spellingBridge(spellings);
      const { box, clock, question, yes, cancel } = await ask(bridge, onTheYes);
      box.current = new DOMRect(100, 240, 120, 30);
      moveTo({ clientX: 150, clientY: 250 });
      pressAt(yes, { clientX: 150, clientY: 250 });
      expect(question).toBeInTheDocument();
      expect(cancel).toHaveFocus();
      expect(bridge.houseRuleCheck).not.toHaveBeenCalled();
      // Seen off it, and back on it.
      clock.now += PRESS_SETTLES_MS * 2;
      moveTo(offTheYes);
      moveTo({ clientX: 150, clientY: 250 });
      pressAt(yes, { clientX: 150, clientY: 250 });
      await waitFor(() => expect(bridge.houseRuleCheck).toHaveBeenCalledOnce());
    });

    it('not a press whose click counts none when it began on the yes', async () => {
      const bridge = spellingBridge(spellings);
      const { question, yes, cancel } = await ask(bridge, onTheYes);
      fireEvent.pointerDown(yes, { ...onTheYes, pointerType: 'mouse' });
      yes.focus();
      fireEvent.click(yes, { detail: 0 });
      expect(question).toBeInTheDocument();
      expect(cancel).toHaveFocus();
      expect(bridge.houseRuleCheck).not.toHaveBeenCalled();
    });

    it('with a finger, not a tap where the tap that opened it was, but a tap on it after a tap off it', async () => {
      const bridge = spellingBridge(spellings);
      const { clock, question, yes, cancel } = await ask(bridge, onTheYes, 'touch');
      const near = { clientX: onTheYes.clientX + 3, clientY: onTheYes.clientY + 2 };
      pressAt(yes, near, 'touch');
      expect(question).toBeInTheDocument();
      expect(cancel).toHaveFocus();
      expect(bridge.houseRuleCheck).not.toHaveBeenCalled();

      // A tap on the sentence, then on the yes.
      clock.now += PRESS_SETTLES_MS * 2;
      const sentence = question.querySelector('p')!;
      fireEvent.pointerDown(sentence, { ...offTheYes, pointerType: 'touch' });
      fireEvent.click(sentence, { detail: 1 });
      pressAt(yes, onTheYes, 'touch');
      await waitFor(() => expect(bridge.houseRuleCheck).toHaveBeenCalledOnce());
    });

    it('but Enter on the yes it was moved to at once', async () => {
      const bridge = spellingBridge(spellings);
      const { yes } = await ask(bridge, onTheYes);
      fireEvent.keyDown(document.activeElement!, { key: 'Tab' });
      yes.focus();
      fireEvent.keyDown(yes, { key: 'Enter' });
      fireEvent.click(yes, { detail: 0 });
      await waitFor(() => expect(bridge.houseRuleCheck).toHaveBeenCalledOnce());
    });
  });

  // Opened by a press, the question opens where its action was, sentence
  // first, and nothing moves under the pointer: the panel used to scroll its
  // Cancel clear of the decision, which brought its yes under the pointer
  // for the next press. The next press lands on the sentence, and the focus
  // stays on Cancel, where Escape takes back the question only.
  describe('opened by a press, moves nothing under the pointer', () => {
    afterEach(() => vi.restoreAllMocks());

    /** The options each focus of `element` was given. */
    const spyOnFocus = () => vi.spyOn(HTMLElement.prototype, 'focus');
    const focusesOf = (focus: ReturnType<typeof spyOnFocus>, element: HTMLElement) => focus.mock.calls.filter((_, index) => focus.mock.contexts[index] === element).map((call) => call[0]);

    it('takes the focus without scrolling when a press opened it, and may scroll when a key did', async () => {
      const focus = spyOnFocus();
      render(<ReviewInspector item={held} drawer={false} {...handlers()} spellings={spellingBridge(spellings)} />);
      const action = await screen.findByRole('button', { name: 'Use on its own: Contoso Worldwide, Inc. written as Contoso' });
      fireEvent.pointerDown(action);
      fireEvent.click(action, { detail: 1 });
      const cancel = within(screen.getByRole('group', { name: 'Use on its own: Contoso Worldwide, Inc. written as Contoso' })).getByRole('button', { name: 'Cancel' });
      expect(cancel).toHaveFocus();
      expect(focusesOf(focus, cancel)).toEqual([{ preventScroll: true }]);
      fireEvent.keyDown(cancel, { key: 'Escape' });

      const again = await screen.findByRole('button', { name: 'Use on its own: Contoso Worldwide, Inc. written as Contoso' });
      fireEvent.keyDown(again, { key: 'Enter' });
      fireEvent.click(again, { detail: 0 });
      const fromKey = within(screen.getByRole('group', { name: 'Use on its own: Contoso Worldwide, Inc. written as Contoso' })).getByRole('button', { name: 'Cancel' });
      expect(fromKey).toHaveFocus();
      expect(focusesOf(focus, fromKey)).toEqual([{ preventScroll: false }]);
    });

    it('leaves the focus on Cancel after a press on the sentence, where Escape takes back the question and not the drawer', async () => {
      const bridge = spellingBridge(spellings);
      const actions = handlers();
      render(<ReviewInspector item={held} drawer {...actions} spellings={bridge} />);
      const action = await screen.findByRole('button', { name: 'Use on its own: Contoso Worldwide, Inc. written as Contoso' });
      fireEvent.pointerDown(action);
      fireEvent.click(action, { detail: 1 });
      const question = screen.getByRole('group', { name: 'Use on its own: Contoso Worldwide, Inc. written as Contoso' });
      const sentence = question.querySelector('p')!;
      // A press on the sentence focuses the group around the question, as it does in Chromium.
      fireEvent.pointerDown(sentence);
      screen.getByRole('group', { name: 'Use a learned spelling on its own' }).focus();
      fireEvent.click(sentence, { detail: 1 });
      expect(within(question).getByRole('button', { name: 'Cancel' })).toHaveFocus();
      fireEvent.keyDown(document.activeElement!, { key: 'Escape' });
      expect(question).not.toBeInTheDocument();
      expect(actions.onClose).not.toHaveBeenCalled();
      expect(bridge.houseRuleCheck).not.toHaveBeenCalled();
    });

    // A press on the sentence let go off the question clicks nothing in it,
    // and leaves the focus on the group around it. Escape there closed the
    // drawer, question and all.
    it('takes back the question, and only the question, on Escape wherever the focus is in the drawer', async () => {
      const bridge = spellingBridge(spellings);
      const actions = handlers();
      render(<ReviewInspector item={held} drawer {...actions} spellings={bridge} />);
      const label = 'Use on its own: Contoso Worldwide, Inc. written as Contoso';
      for (const focusElsewhere of [
        () => screen.getByRole('group', { name: 'Use a learned spelling on its own' }).focus(),
        () => screen.getByLabelText('Filename').focus(),
      ]) {
        const action = await screen.findByRole('button', { name: label });
        pressAt(action, offTheYes);
        const question = screen.getByRole('group', { name: label });
        fireEvent.pointerDown(question.querySelector('p')!, { ...offTheYes, pointerType: 'mouse' });
        focusElsewhere();
        fireEvent.keyDown(document.activeElement!, { key: 'Escape' });
        expect(question).not.toBeInTheDocument();
        await waitFor(() => expect(screen.getByRole('button', { name: label })).toHaveFocus());
      }
      expect(actions.onClose).not.toHaveBeenCalled();
      expect(bridge.houseRuleCheck).not.toHaveBeenCalled();
      // With no question open, Escape closes the drawer as before.
      fireEvent.keyDown(screen.getByRole('button', { name: label }), { key: 'Escape' });
      expect(actions.onClose).toHaveBeenCalledOnce();
    });
  });

  // The record's spelling can be older than the spelling: an approval of
  // another document can respell it while this one is on screen. Checking a
  // spelling the person was not shown would let Intern file names they
  // never agreed to.
  it('says when the spelling changed since it was shown, and checks nothing', async () => {
    const bridge = spellingBridge(spellings);
    const onSpellingChecked = vi.fn();
    render(<ReviewInspector item={held} drawer={false} {...handlers()} spellings={bridge} onSpellingChecked={onSpellingChecked} />);
    fireEvent.click(await screen.findByRole('button', { name: 'Use on its own: Contoso Worldwide, Inc. written as Contoso' }));
    bridge.respell('1', 'Contoso Ltd');
    fireEvent.click(screen.getByRole('button', { name: 'Use on its own' }));
    expect(await screen.findByRole('alert')).toHaveTextContent('This spelling changed - look at it again.');
    expect(bridge.houseRuleCheck).toHaveBeenCalledWith('1', { kind: 'party', from: 'Contoso Worldwide, Inc.', to: 'Contoso' });
    expect(onSpellingChecked).not.toHaveBeenCalled();
    // As it reads now, it is not the spelling the name used: nothing to offer for it.
    await waitFor(() => expect(screen.queryByRole('button', { name: /Use on its own: Contoso/ })).not.toBeInTheDocument());
  });

  // The note above lists every spelling the name used. A bare "Use on its
  // own" under it - the only unchecked one of three, or the one left after
  // the other was checked and named in the line below - read as an action
  // for whichever spelling was nearest. Each says which it is on screen
  // whenever the note lists more than one.
  it('names the spelling beside "Use on its own" when it is the one unchecked of several the note lists', async () => {
    const oneUnchecked = spellingBridge(spellings.map((rule) => rule.id === '2' ? { ...rule, confirmed: 1 } : rule));
    render(<ReviewInspector item={held} drawer={false} {...handlers()} spellings={oneUnchecked} />);
    await screen.findByRole('button', { name: 'Use on its own: Contoso Worldwide, Inc. written as Contoso' });
    const group = screen.getByRole('group', { name: 'Use a learned spelling on its own' });
    expect(within(group).getAllByRole('button', { name: /^Use on its own:/ })).toHaveLength(1);
    expect(group).toHaveTextContent('Contoso Worldwide, Inc. written as Contoso');
  });

  it('names the spelling beside the "Use on its own" left once the other was checked', async () => {
    const bridge = spellingBridge(spellings);
    render(<ReviewInspector item={held} drawer={false} {...handlers()} spellings={bridge} />);
    fireEvent.click(await screen.findByRole('button', { name: 'Use on its own: Contoso Worldwide, Inc. written as Contoso' }));
    fireEvent.click(within(screen.getByRole('group', { name: /^Use on its own:/ })).getByRole('button', { name: 'Use on its own' }));
    await screen.findByRole('status');
    await waitFor(() => expect(screen.queryByRole('button', { name: /Use on its own: Contoso/ })).not.toBeInTheDocument());
    const left = screen.getByRole('group', { name: 'Use a learned spelling on its own' });
    expect(within(left).getAllByRole('button', { name: /^Use on its own:/ })).toHaveLength(1);
    expect(left).toHaveTextContent('Ridgeline Cartography LLC written as Ridgeline Cartography');
  });

  it('shows the action alone under a note of one spelling', async () => {
    const one: QueueItem = { ...held, houseRules: [held.houseRules![0]] };
    render(<ReviewInspector item={one} drawer={false} {...handlers()} spellings={spellingBridge(spellings)} />);
    await screen.findByRole('button', { name: 'Use on its own: Contoso Worldwide, Inc. written as Contoso' });
    expect(screen.getByRole('group', { name: 'Use a learned spelling on its own' })).toHaveTextContent(/^Use on its own$/);
  });

  it('is offered from the review window of the app, which reads the spellings from the bridge', async () => {
    const base = createInMemoryBridge({ items: [held], learnedRules: spellings });
    const houseRuleCheck = vi.fn(base.houseRuleCheck);
    render(<App bridge={{ ...base, houseRuleCheck }} />);
    await selectRow(await screen.findByRole('row', { name: /SOW final v3.pdf/i }));
    fireEvent.click(await screen.findByRole('button', { name: 'Use on its own: Contoso Worldwide, Inc. written as Contoso' }));
    fireEvent.click(screen.getByRole('button', { name: 'Use on its own' }));
    await waitFor(() => expect(houseRuleCheck).toHaveBeenCalledWith('1', { kind: 'party', from: 'Contoso Worldwide, Inc.', to: 'Contoso' }));
    expect((await base.houseRulesList()).find((rule) => rule.id === '1')?.confirmed).toBe(1);
  });
});
