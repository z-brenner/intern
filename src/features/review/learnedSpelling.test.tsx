import { cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { App } from '../../App';
import { ReviewInspector } from '../../components/ReviewInspector';
import { PRESS_SETTLES_MS } from '../../components/UseOnItsOwn';
import { openRow } from '../queue/gridTesting';
import { drawYes, moveTo, nextFrame, offTheYes, onTheYes, pressAt, yesAt } from './pointerTesting';
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

/** A press on the yes where the pointer is, on it, as Chromium makes one: the pointer down, the mouse down, and a click counting `presses`. */
const press = (button: HTMLElement, presses: number) => {
  fireEvent.pointerDown(button, { ...onTheYes, pointerType: 'mouse' });
  if (fireEvent.mouseDown(button, onTheYes)) button.focus();
  fireEvent.pointerUp(button, { ...onTheYes, pointerType: 'mouse' });
  fireEvent.click(button, { ...onTheYes, detail: presses });
};

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
      fireEvent.pointerUp(yes, { ...onTheYes, pointerType: 'mouse' });
      fireEvent.click(yes, { detail: 1 });
      expect(question).toBeInTheDocument();
      expect(cancel).toHaveFocus();
      expect(bridge.houseRuleCheck).not.toHaveBeenCalled();

      // One begun once the sentence had time to be read is a yes.
      clock.now += PRESS_SETTLES_MS;
      fireEvent.pointerDown(yes, { ...onTheYes, pointerType: 'mouse' });
      yes.focus();
      fireEvent.pointerUp(yes, { ...onTheYes, pointerType: 'mouse' });
      fireEvent.click(yes, { detail: 1 });
      await waitFor(() => expect(bridge.houseRuleCheck).toHaveBeenCalledOnce());
    });

    // A double-click is timed from its first press to its second. A
    // question opened by a press timed its half second from when it
    // appeared, after that press was let go, and refused a person quick to
    // move onto the yes and press it.
    it('times the half second from when the press that opened the question began', async () => {
      const bridge = spellingBridge(spellings);
      const clock = { now: 1000 };
      vi.spyOn(performance, 'now').mockImplementation(() => clock.now);
      drawYes();
      render(<ReviewInspector item={held} drawer={false} {...handlers()} spellings={bridge} />);
      const label = 'Use on its own: Contoso Worldwide, Inc. written as Contoso';
      const open = async (held = 150) => {
        const began = clock.now;
        const action = await screen.findByRole('button', { name: label });
        fireEvent.pointerDown(action, { ...offTheYes, pointerType: 'mouse' });
        // Held as a hand holds a press, and let go.
        clock.now += held;
        fireEvent.pointerUp(action, { ...offTheYes, pointerType: 'mouse' });
        fireEvent.click(action, { ...offTheYes, detail: 1 });
        moveTo(onTheYes);
        const question = screen.getByRole('group', { name: label });
        return { began, question, yes: within(question).getByRole('button', { name: 'Use on its own' }) };
      };
      // Within the half second of the press that opened it: no answer.
      let asked = await open();
      clock.now = asked.began + PRESS_SETTLES_MS - 50;
      press(asked.yes, 1);
      expect(asked.question).toBeInTheDocument();
      fireEvent.keyDown(within(asked.question).getByRole('button', { name: 'Cancel' }), { key: 'Escape' });
      // A press held longer than the half second is no double-click begun:
      // the half second runs from when the question appeared.
      clock.now += PRESS_SETTLES_MS * 4;
      asked = await open(PRESS_SETTLES_MS * 2);
      clock.now += 100;
      press(asked.yes, 1);
      expect(asked.question).toBeInTheDocument();
      fireEvent.keyDown(within(asked.question).getByRole('button', { name: 'Cancel' }), { key: 'Escape' });
      // After it, though less than half a second after the question appeared: a yes.
      clock.now += PRESS_SETTLES_MS * 4;
      asked = await open();
      clock.now = asked.began + PRESS_SETTLES_MS + 50;
      press(asked.yes, 1);
      await waitFor(() => expect(bridge.houseRuleCheck).toHaveBeenCalledOnce());
    });

    // A press refused because the pointer had not come onto the yes said
    // nothing: a person who meant it pressed again and again a yes that did
    // nothing. It says what to do, beside the buttons. A press refused for
    // coming too soon needs nothing done but to press again.
    it('says what to do when the pointer had not come onto the yes, and only then', async () => {
      const bridge = spellingBridge(spellings);
      const clock = { now: 1000 };
      const { question, yes } = await ask(bridge, clock);
      const again = within(question).getByRole('status');
      press(yes, 1);
      expect(question).toBeInTheDocument();
      expect(again).toBeEmptyDOMElement();
      clock.now += PRESS_SETTLES_MS * 2;
      moveTo({ clientX: onTheYes.clientX + 1, clientY: onTheYes.clientY }, document.body);
      press(yes, 1);
      expect(question).toBeInTheDocument();
      expect(again).toHaveTextContent('Move well off Use on its own, then back.');
      expect(bridge.houseRuleCheck).not.toHaveBeenCalled();
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
  // under a resting mouse. The yes takes a press of the pointer only from
  // a pointer that came onto it after the question appeared - found on the
  // yes itself, from where that pointer was last seen off it, the yes
  // standing still - however it got there.
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
      fireEvent.pointerUp(yes, { ...onTheYes, pointerType: 'mouse' });
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

    // A pen that does not report hovering is seen only where it touches
    // down, as a finger is.
    it('with a pen that does not hover, not a tap where the tap that opened it was, but a tap on it after a tap off it', async () => {
      const bridge = spellingBridge(spellings);
      const { clock, question, yes, cancel } = await ask(bridge, onTheYes, 'pen');
      pressAt(yes, { clientX: onTheYes.clientX + 2, clientY: onTheYes.clientY + 1 }, 'pen');
      expect(question).toBeInTheDocument();
      expect(cancel).toHaveFocus();
      expect(bridge.houseRuleCheck).not.toHaveBeenCalled();
      clock.now += PRESS_SETTLES_MS * 2;
      const sentence = question.querySelector('p')!;
      fireEvent.pointerDown(sentence, { ...offTheYes, pointerType: 'pen' });
      fireEvent.click(sentence, { detail: 1 });
      pressAt(yes, onTheYes, 'pen');
      await waitFor(() => expect(bridge.houseRuleCheck).toHaveBeenCalledOnce());
    });

    // A pen that hovers is held to what a mouse is: a yes that came to rest
    // a pixel from where it hovered, and a touch down across its edge, is
    // the yes coming to the pen.
    it('with a pen that hovers, not a touch down from where it hovered beside a yes that moved there since', async () => {
      const bridge = spellingBridge(spellings);
      const { box, clock, question, yes } = await ask(bridge, offTheYes);
      const hover = (at: Point) => fireEvent.pointerMove(document.body, { ...at, pointerType: 'pen', buttons: 0 });
      hover(offTheYes);
      box.current = new DOMRect(100, offTheYes.clientY + 1, 120, 30);
      const across = { clientX: offTheYes.clientX, clientY: offTheYes.clientY + 2 };
      pressAt(yes, across, 'pen');
      expect(question).toBeInTheDocument();
      expect(bridge.houseRuleCheck).not.toHaveBeenCalled();
      // Hovering where it was, beside the yes where it is now, then touching
      // down on it: still a pen the yes came to.
      clock.now += PRESS_SETTLES_MS * 2;
      hover({ clientX: offTheYes.clientX + 1, clientY: offTheYes.clientY });
      pressAt(yes, across, 'pen');
      expect(question).toBeInTheDocument();
      expect(bridge.houseRuleCheck).not.toHaveBeenCalled();
      // Hovering away from the yes where it is now, then touching down on it.
      clock.now += PRESS_SETTLES_MS * 2;
      hover({ clientX: offTheYes.clientX, clientY: offTheYes.clientY - 40 });
      pressAt(yes, across, 'pen');
      await waitFor(() => expect(bridge.houseRuleCheck).toHaveBeenCalledOnce());
    });

    it('says to a finger refused what to do', async () => {
      const bridge = spellingBridge(spellings);
      const { question, yes } = await ask(bridge, onTheYes, 'touch');
      pressAt(yes, onTheYes, 'touch');
      expect(within(question).getByRole('status')).toHaveTextContent('Tap well away, then Use on its own.');
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

    // What is under the pointer is what the browser finds there, not the
    // yes's box: the More menu, or the decision grown over the yes, is not
    // the yes. A pointer moved onto Keep original where it lay over the
    // yes's box came onto the yes; a press closed the menu, and the yes it
    // uncovered under the still pointer took the press after it.
    it('not from a pointer that came onto something drawn over it, nor one with something drawn over it since', async () => {
      const bridge = spellingBridge(spellings);
      const { clock, question, yes, cancel } = await ask(bridge, offTheYes);
      const over = document.createElement('button');
      document.body.append(over);
      moveTo(onTheYes, over);
      over.remove();
      pressAt(yes, onTheYes);
      expect(question).toBeInTheDocument();
      expect(cancel).toHaveFocus();
      // On the yes, then onto something drawn over it there, and it goes.
      clock.now += PRESS_SETTLES_MS * 2;
      moveTo(offTheYes);
      moveTo(onTheYes);
      document.body.append(over);
      moveTo({ clientX: onTheYes.clientX + 1, clientY: onTheYes.clientY }, over);
      over.remove();
      pressAt(yes, { clientX: onTheYes.clientX + 1, clientY: onTheYes.clientY });
      expect(question).toBeInTheDocument();
      expect(bridge.houseRuleCheck).not.toHaveBeenCalled();
    });

    // A report of the pointer where it already was says what is under it
    // now. When that is not the yes, the pointer is no longer on it.
    it('not from a pointer reported where it was on something drawn over the yes since it came', async () => {
      const bridge = spellingBridge(spellings);
      const { question, yes } = await ask(bridge, offTheYes);
      moveTo(onTheYes);
      const over = document.createElement('div');
      document.body.append(over);
      moveTo(onTheYes, over);
      over.remove();
      pressAt(yes, onTheYes);
      expect(question).toBeInTheDocument();
      expect(bridge.houseRuleCheck).not.toHaveBeenCalled();
    });

    // A scroll or a change of layout left the yes's edge a pixel from a
    // still pointer, and a twitch of the hand crossed it. The pointer moved,
    // but onto a yes that had come to it: it has to be seen where it is
    // with the yes where it is, and then come onto it.
    it('not from a pointer that crossed onto a yes that moved since it was last seen', async () => {
      const bridge = spellingBridge(spellings);
      const { box, clock, question, yes, cancel } = await ask(bridge, offTheYes);
      box.current = new DOMRect(100, offTheYes.clientY + 1, 120, 30);
      const twitch = { clientX: offTheYes.clientX, clientY: offTheYes.clientY + 2 };
      moveTo(twitch);
      pressAt(yes, twitch);
      expect(question).toBeInTheDocument();
      expect(cancel).toHaveFocus();
      expect(bridge.houseRuleCheck).not.toHaveBeenCalled();
      // Seen off it where it is now, but beside it, and onto it: a hand that
      // drifts a pixel one way and two the other while it presses.
      clock.now += PRESS_SETTLES_MS * 2;
      moveTo({ clientX: offTheYes.clientX + 1, clientY: offTheYes.clientY });
      moveTo(twitch);
      pressAt(yes, twitch);
      expect(question).toBeInTheDocument();
      expect(bridge.houseRuleCheck).not.toHaveBeenCalled();
      // Seen away from it where it is now, and onto it.
      clock.now += PRESS_SETTLES_MS * 2;
      moveTo({ clientX: offTheYes.clientX, clientY: offTheYes.clientY - 40 });
      moveTo(twitch);
      pressAt(yes, twitch);
      await waitFor(() => expect(bridge.houseRuleCheck).toHaveBeenCalledOnce());
    });

    // The pointer came onto the yes, and the yes moved - a scroll away and
    // back, a change of layout - under the pointer that stayed put.
    it('not once the yes moved under a pointer that came onto it, away and back as well', async () => {
      const bridge = spellingBridge(spellings);
      const { box, clock, question, yes } = await ask(bridge, offTheYes);
      moveTo(onTheYes);
      box.current = new DOMRect(100, 400, 120, 30);
      await nextFrame();
      box.current = yesAt;
      await nextFrame();
      pressAt(yes, onTheYes);
      expect(question).toBeInTheDocument();
      // Moved by a few pixels, still under the pointer, and pressed before the next frame.
      clock.now += PRESS_SETTLES_MS * 2;
      moveTo(offTheYes);
      moveTo(onTheYes);
      box.current = new DOMRect(100, 205, 120, 30);
      pressAt(yes, onTheYes);
      expect(question).toBeInTheDocument();
      expect(bridge.houseRuleCheck).not.toHaveBeenCalled();
    });

    // A touch screen and a mouse: each pointer is its own. The mouse rests
    // where the yes opens, and a finger taps the action. A hand back on the
    // mouse, a pixel's move, is not the mouse coming onto the yes; nor does
    // a finger that came onto it let a press of the mouse through.
    it('not from a mouse that never moved onto it, after a finger', async () => {
      const bridge = spellingBridge(spellings);
      moveTo(onTheYes);
      const { clock, question, yes, cancel } = await ask(bridge, offTheYes, 'touch');
      const nudged = { clientX: onTheYes.clientX + 1, clientY: onTheYes.clientY };
      moveTo(nudged);
      pressAt(yes, nudged);
      expect(question).toBeInTheDocument();
      expect(cancel).toHaveFocus();
      // A finger touches down on the yes from off it, and goes on to scroll: no click.
      clock.now += PRESS_SETTLES_MS * 2;
      fireEvent.pointerDown(yes, { ...onTheYes, pointerType: 'touch' });
      fireEvent.pointerCancel(yes, { ...onTheYes, pointerType: 'touch' });
      pressAt(yes, nudged);
      expect(question).toBeInTheDocument();
      expect(bridge.houseRuleCheck).not.toHaveBeenCalled();
    });

    // A click on the yes that did not begin there is no press on it,
    // whatever pointer came onto it: a finger's tap on Keep original over
    // the yes closed the More menu, and its click landed on the yes the
    // menu uncovered, under a mouse that had come onto it.
    it('not a click that did not begin on it, whatever pointer is on it', async () => {
      const bridge = spellingBridge(spellings);
      const { question, yes } = await ask(bridge, offTheYes);
      moveTo(onTheYes);
      const keep = document.createElement('button');
      document.body.append(keep);
      fireEvent.pointerDown(keep, { ...onTheYes, pointerType: 'touch' });
      keep.remove();
      fireEvent.click(yes, { ...onTheYes, detail: 1 });
      expect(question).toBeInTheDocument();
      expect(bridge.houseRuleCheck).not.toHaveBeenCalled();
    });

    // A press with another button, or one let go off the yes, clicks
    // nothing, so nothing refused it, and the focus Chromium gives the
    // button pressed stayed on the yes: the next Enter, or Space, was a yes
    // from the keyboard. A press never gives the yes the focus.
    it('never takes the focus from a press, so a key after it is no yes', async () => {
      const bridge = spellingBridge(spellings);
      const { question, yes, cancel } = await ask(bridge, onTheYes);
      for (const button of [2, 1, 0]) {
        fireEvent.pointerDown(yes, { ...onTheYes, pointerType: 'mouse', button });
        if (fireEvent.mouseDown(yes, { ...onTheYes, button })) yes.focus();
        fireEvent.pointerUp(document.body, { pointerType: 'mouse', button });
        fireEvent.mouseUp(document.body, { button });
        expect(cancel).toHaveFocus();
      }
      fireEvent.keyDown(document.activeElement!, { key: 'Enter' });
      fireEvent.click(document.activeElement!, { detail: 0 });
      expect(question).not.toBeInTheDocument();
      expect(bridge.houseRuleCheck).not.toHaveBeenCalled();
    });

    // A press that clicks nothing - with the middle button, or the right -
    // left the press on the yes standing, under a mouse that had come onto
    // it and stayed. A finger's tap that began on Keep original, over the
    // yes, closed the More menu, and its click landed on the yes: it was
    // taken as the end of the mouse's press. The press on the yes is the
    // last press anywhere, and only when it began on the yes.
    it('not a click on it that ends a press begun elsewhere, after a press on it that clicked nothing', async () => {
      const bridge = spellingBridge(spellings);
      const { question, yes } = await ask(bridge, offTheYes);
      moveTo(onTheYes);
      fireEvent.pointerDown(yes, { ...onTheYes, pointerType: 'mouse', button: 1, buttons: 4 });
      fireEvent.mouseDown(yes, { ...onTheYes, button: 1 });
      fireEvent.pointerUp(yes, { ...onTheYes, pointerType: 'mouse', button: 1 });
      fireEvent.mouseUp(yes, { ...onTheYes, button: 1 });
      const keep = document.createElement('button');
      document.body.append(keep);
      fireEvent.pointerDown(keep, { ...onTheYes, pointerType: 'touch' });
      keep.remove();
      fireEvent.pointerUp(yes, { ...onTheYes, pointerType: 'touch' });
      fireEvent.click(yes, { ...onTheYes, detail: 1 });
      expect(question).toBeInTheDocument();
      expect(bridge.houseRuleCheck).not.toHaveBeenCalled();
    });

    // The page sees a mouse only while it is over the page with nothing of
    // the system's over it. Over another window lying over the panel - the
    // document the spelling is checked against - a menu of the system's or
    // a notification, or with Intern behind another window, it moves where
    // the page cannot see; when that goes away, it is where the page did not
    // see it go, the yes maybe. A window snapped by a key moves the page
    // under a mouse that stays put. Where it is next seen is where it was
    // found, not a move onto the yes, and a mouse that had come onto the yes
    // is no longer known to be on it, nor to have been away from it. Away
    // from the yes and back, where the page sees it, it is answered.
    it.each([
      ['it left the page', () => { fireEvent.pointerOut(document.body, { pointerType: 'mouse', relatedTarget: null }); }],
      ['the window lost the focus', () => { fireEvent.blur(window); }],
      ['the system\'s menu opened', () => { fireEvent.contextMenu(document.body); }],
      ['the browser took it for a drag', () => { fireEvent.pointerCancel(document.body, { pointerType: 'mouse' }); }],
      ['a press began that the page did not see end', () => { fireEvent.pointerDown(document.body, { pointerType: 'mouse', buttons: 1 }); }],
      ['the window moved on the screen under it', () => { vi.spyOn(window, 'screenX', 'get').mockReturnValue(window.screenX + 300); }],
    ])('not from a mouse found on it once the page lost sight of it: %s', async (_how, lose) => {
      const bridge = spellingBridge(spellings);
      const { clock, question, yes } = await ask(bridge, offTheYes);
      // Seen away from it, lost, and found on it.
      lose();
      moveTo(onTheYes);
      pressAt(yes, onTheYes);
      expect(question).toBeInTheDocument();
      // Come onto it, lost, and found on it a pixel over.
      clock.now += PRESS_SETTLES_MS * 2;
      moveTo(offTheYes);
      moveTo(onTheYes);
      lose();
      const nudged = { clientX: onTheYes.clientX + 1, clientY: onTheYes.clientY };
      moveTo(nudged);
      pressAt(yes, nudged);
      expect(question).toBeInTheDocument();
      expect(within(question).getByRole('status')).toHaveTextContent('Move well off Use on its own, then back.');
      // Seen away from it, lost, found beside it, and onto it.
      clock.now += PRESS_SETTLES_MS * 2;
      moveTo(offTheYes);
      lose();
      moveTo({ clientX: onTheYes.clientX, clientY: yesAt.top - 2 });
      moveTo(onTheYes);
      pressAt(yes, onTheYes);
      expect(question).toBeInTheDocument();
      expect(bridge.houseRuleCheck).not.toHaveBeenCalled();
      // Away from it, seen, and back onto it, and it is answered.
      clock.now += PRESS_SETTLES_MS * 2;
      moveTo(offTheYes);
      moveTo(onTheYes);
      pressAt(yes, onTheYes);
      await waitFor(() => expect(bridge.houseRuleCheck).toHaveBeenCalledOnce());
    });

    it('but a menu the page shows for itself, not the system\'s, loses nothing', async () => {
      const bridge = spellingBridge(spellings);
      const { yes } = await ask(bridge, offTheYes);
      const itsOwn = (event: Event) => event.preventDefault();
      document.addEventListener('contextmenu', itsOwn);
      fireEvent.contextMenu(document.body);
      document.removeEventListener('contextmenu', itsOwn);
      moveTo(onTheYes);
      pressAt(yes, onTheYes);
      await waitFor(() => expect(bridge.houseRuleCheck).toHaveBeenCalledOnce());
    });

    // A finger leaves the page's sight each time it lifts, as a mouse
    // leaving the page does, but each touch is where it was aimed: nothing
    // of a finger is lost.
    it('with a finger, a tap on it after a tap away from it, the finger lifting between', async () => {
      const bridge = spellingBridge(spellings);
      const { question, yes } = await ask(bridge, onTheYes, 'touch');
      const sentence = question.querySelector('p')!;
      fireEvent.pointerDown(sentence, { ...offTheYes, pointerType: 'touch' });
      fireEvent.pointerUp(sentence, { ...offTheYes, pointerType: 'touch' });
      fireEvent.pointerOut(sentence, { ...offTheYes, pointerType: 'touch', relatedTarget: null });
      fireEvent.click(sentence, { detail: 1 });
      pressAt(yes, onTheYes, 'touch');
      await waitFor(() => expect(bridge.houseRuleCheck).toHaveBeenCalledOnce());
    });

    // A question opened from the keyboard, its yes appearing with its edge a
    // pixel from where the mouse rests, and a hand on the mouse that drifts
    // two pixels as it presses: the mouse moved onto the yes, but the yes
    // came to it. A pointer comes onto the yes from away from it - at least
    // the yes's own height off it - with the yes standing where it is.
    it('not from a mouse that drifted onto it from beside it, where it appeared', async () => {
      const bridge = spellingBridge(spellings);
      const clock = { now: 1000 };
      vi.spyOn(performance, 'now').mockImplementation(() => clock.now);
      drawYes();
      render(<ReviewInspector item={held} drawer={false} {...handlers()} spellings={bridge} />);
      const label = 'Use on its own: Contoso Worldwide, Inc. written as Contoso';
      const action = await screen.findByRole('button', { name: label });
      moveTo({ clientX: 150, clientY: 100 });
      moveTo({ clientX: 150, clientY: yesAt.top - 1 });
      fireEvent.keyDown(action, { key: 'Enter' });
      fireEvent.click(action, { detail: 0 });
      const question = screen.getByRole('group', { name: label });
      const yes = within(question).getByRole('button', { name: 'Use on its own' });
      clock.now += PRESS_SETTLES_MS * 2;
      const drifted = { clientX: 151, clientY: yesAt.top + 1 };
      moveTo(drifted);
      pressAt(yes, drifted);
      expect(question).toBeInTheDocument();
      expect(within(question).getByRole('status')).toHaveTextContent('Move well off Use on its own, then back.');
      // Off it by less than it is tall, and back: still beside it.
      clock.now += PRESS_SETTLES_MS * 2;
      moveTo({ clientX: 150, clientY: yesAt.top - yesAt.height + 1 });
      moveTo(drifted);
      pressAt(yes, drifted);
      expect(question).toBeInTheDocument();
      expect(bridge.houseRuleCheck).not.toHaveBeenCalled();
      // Its height off it, and back onto it.
      clock.now += PRESS_SETTLES_MS * 2;
      moveTo({ clientX: 150, clientY: yesAt.top - yesAt.height });
      moveTo(drifted);
      pressAt(yes, drifted);
      await waitFor(() => expect(bridge.houseRuleCheck).toHaveBeenCalledOnce());
    });

    // A finger swipes the panel, and the yes comes to rest with its edge a
    // pixel from where the finger lifted; a tap a few pixels from there lands
    // on it. A finger comes onto the yes from a touch away from it as it is.
    it('with a finger, not a tap a few pixels from its last, across the edge of a yes that came to rest between them', async () => {
      const bridge = spellingBridge(spellings);
      const { box, clock, question, yes } = await ask(bridge, offTheYes, 'touch');
      const lifted = { clientX: 150, clientY: 300 };
      fireEvent.pointerDown(document.body, { ...lifted, pointerType: 'touch' });
      fireEvent.pointerCancel(document.body, { ...lifted, pointerType: 'touch' });
      box.current = new DOMRect(100, lifted.clientY + 1, 120, 30);
      const again = { clientX: 151, clientY: lifted.clientY + 3 };
      pressAt(yes, again, 'touch');
      expect(question).toBeInTheDocument();
      expect(within(question).getByRole('status')).toHaveTextContent('Tap well away, then Use on its own.');
      expect(bridge.houseRuleCheck).not.toHaveBeenCalled();
      // A tap away from it, on the sentence above it, and one on it.
      clock.now += PRESS_SETTLES_MS * 2;
      const sentence = question.querySelector('p')!;
      const start = { clientX: 110, clientY: lifted.clientY - 40 };
      fireEvent.pointerDown(sentence, { ...start, pointerType: 'touch' });
      fireEvent.pointerUp(sentence, { ...start, pointerType: 'touch' });
      fireEvent.click(sentence, { detail: 1 });
      pressAt(yes, again, 'touch');
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
