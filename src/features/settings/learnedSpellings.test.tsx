import { act, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { App } from '../../App';
import { PRESS_SETTLES_MS } from '../../components/UseOnItsOwn';
import { createInMemoryBridge } from '../../lib/inMemoryBridge';
import { drawYes, moveTo, offTheYes, onTheYes, pressAt, yesAt } from '../review/pointerTesting';
import type { Point } from '../review/pointerTesting';
import type { HouseRule, LearnedRule } from '../../types';

async function openSettings() {
  fireEvent.click((await screen.findAllByRole('button', { name: 'Settings' }))[0]);
  const dialog = await screen.findByRole('dialog', { name: 'Settings' });
  fireEvent.click(within(dialog).getByText('Learned spellings', { selector: 'summary' }));
  return dialog;
}

const learned: LearnedRule[] = [
  { id: '1', kind: 'party', from: 'Contoso Worldwide, Inc.', to: 'Contoso', seen: 1, active: false, confirmed: 0, learnedAt: 1716282900 },
  { id: '2', kind: 'document_type', from: 'Quarterly Operations Review', to: 'Meeting Minutes', seen: 2, active: true, confirmed: 1, learnedAt: 1716196500 },
];

describe('the spellings Intern has learned', () => {
  it('says there is nothing to show until review has taught something', async () => {
    render(<App bridge={createInMemoryBridge()} />);
    const dialog = await openSettings();
    expect(await within(dialog).findByText(/Nothing yet\. Change a party's name or a document type in review/)).toBeVisible();
    expect(within(dialog).queryByRole('list', { name: 'Learned spellings' })).not.toBeInTheDocument();
  });

  it('lists each spelling with its state, uses one at once, and forgets one', async () => {
    const base = createInMemoryBridge({ learnedRules: learned });
    const houseRuleUse = vi.fn(base.houseRuleUse);
    const houseRuleForget = vi.fn(base.houseRuleForget);
    render(<App bridge={{ ...base, houseRuleUse, houseRuleForget }} />);
    const dialog = await openSettings();
    const list = await within(dialog).findByRole('list', { name: 'Learned spellings' });
    const rows = within(list).getAllByRole('listitem');
    expect(rows).toHaveLength(2);
    expect(rows[0]).toHaveTextContent('Party');
    expect(rows[0]).toHaveTextContent('Contoso Worldwide, Inc. written as Contoso');
    expect(rows[0]).toHaveTextContent('Seen once · in use after one more edit');
    expect(rows[1]).toHaveTextContent('Type');
    expect(rows[1]).toHaveTextContent('In use');
    // A spelling already in use has nothing to hurry.
    expect(within(rows[1]).queryByRole('button', { name: /Use now/ })).not.toBeInTheDocument();

    fireEvent.click(within(rows[0]).getByRole('button', { name: 'Use now: Contoso Worldwide, Inc. written as Contoso' }));
    await waitFor(() => expect(houseRuleUse).toHaveBeenCalledWith('1'));
    await waitFor(() => expect(within(list).getAllByRole('listitem')[0]).toHaveTextContent('In use'));
    expect(within(within(list).getAllByRole('listitem')[0]).queryByRole('button', { name: /Use now/ })).not.toBeInTheDocument();

    fireEvent.click(within(within(list).getAllByRole('listitem')[1]).getByRole('button', { name: 'Forget: Quarterly Operations Review written as Meeting Minutes' }));
    await waitFor(() => expect(houseRuleForget).toHaveBeenCalledWith('2'));
    await waitFor(() => expect(within(list).getAllByRole('listitem')).toHaveLength(1));
    expect(within(list).getAllByRole('listitem')[0]).toHaveTextContent('Contoso');
  });

  // A spelling learned from two edits is in use, but every name it changes
  // waits for a person until they choose "Use on its own" for it: the list
  // says it asks before filing. "Use now" puts a spelling in use, no more
  // checked than one learned from two edits. It counted as the check, and
  // one click on a wrong spelling filed the next name it changed without
  // review.
  it('says a spelling in use asks before filing until it is used on its own', async () => {
    const unchecked: LearnedRule = { ...learned[1], confirmed: 0 };
    render(<App bridge={createInMemoryBridge({ learnedRules: [learned[0], unchecked] })} />);
    const dialog = await openSettings();
    const list = await within(dialog).findByRole('list', { name: 'Learned spellings' });
    expect(within(list).getAllByRole('listitem')[1]).toHaveTextContent('In use · asks before filing');

    fireEvent.click(within(within(list).getAllByRole('listitem')[0]).getByRole('button', { name: /Use now:/ }));
    await waitFor(() => expect(within(list).getAllByRole('listitem')[0]).toHaveTextContent('In use · asks before filing'));
  });

  // Forgetting is instant and the row disappears, so a second click landing
  // before React rerenders used to reach a rule that was already gone and
  // report RULE_NOT_FOUND - an error about nothing, over a change that worked.
  it('forgets a spelling once when the button is clicked twice quickly', async () => {
    const base = createInMemoryBridge({ learnedRules: learned });
    const houseRuleForget = vi.fn(base.houseRuleForget);
    render(<App bridge={{ ...base, houseRuleForget }} />);
    const dialog = await openSettings();
    const list = await within(dialog).findByRole('list', { name: 'Learned spellings' });
    const forget = within(within(list).getAllByRole('listitem')[0]).getByRole('button', { name: /Forget:/ });

    act(() => { fireEvent.click(forget); fireEvent.click(forget); });

    await waitFor(() => expect(within(list).getAllByRole('listitem')).toHaveLength(1));
    expect(houseRuleForget).toHaveBeenCalledOnce();
    expect(within(dialog).queryByRole('alert')).not.toBeInTheDocument();
  });

  // No approval checks a spelling: "Use on its own" is the only way. It is
  // offered for a spelling in use that nobody has checked, says what it
  // does to every later document before doing it, and checks the spelling
  // exactly as the person was shown it - kind, word and spelling. Answered
  // or taken back, the focus goes back to the spelling's row: it went to
  // the page, and a keyboard user had to find the list again.
  it('uses an unchecked spelling on its own only after saying what that does, and keeps the focus on its row', async () => {
    const unchecked: LearnedRule = { id: '3', kind: 'party', from: 'Acme', to: 'Acme Widgets International, Inc.', seen: 2, active: true, confirmed: 0, learnedAt: 1716369300 };
    const base = createInMemoryBridge({ learnedRules: [learned[0], learned[1], unchecked] });
    const houseRuleCheck = vi.fn(base.houseRuleCheck);
    const houseRuleUse = vi.fn(base.houseRuleUse);
    render(<App bridge={{ ...base, houseRuleCheck, houseRuleUse }} />);
    const dialog = await openSettings();
    const list = await within(dialog).findByRole('list', { name: 'Learned spellings' });
    const rows = () => within(list).getAllByRole('listitem');
    const action = () => within(rows()[2]).getByRole('button', { name: 'Use on its own: Acme written as Acme Widgets International, Inc.' });
    // Not for a spelling seen once, nor for one already checked.
    expect(within(rows()[0]).queryByRole('button', { name: /Use on its own/ })).not.toBeInTheDocument();
    expect(within(rows()[1]).queryByRole('button', { name: /Use on its own/ })).not.toBeInTheDocument();

    fireEvent.click(action());
    const question = within(rows()[2]).getByRole('group', { name: 'Use on its own: Acme written as Acme Widgets International, Inc.' });
    expect(question).toHaveAccessibleDescription('Every later document Intern reads as naming Acme will name Acme Widgets International, Inc. instead, and may be filed under that name without your check. Do this only if that is right for every one of them.');
    expect(within(question).getByRole('button', { name: 'Cancel' })).toHaveFocus();
    expect(houseRuleCheck).not.toHaveBeenCalled();

    // Cancel, and Escape, take the question back, change nothing, and give
    // the focus back to the action.
    fireEvent.click(within(question).getByRole('button', { name: 'Cancel' }));
    expect(within(rows()[2]).queryByRole('group')).not.toBeInTheDocument();
    await waitFor(() => expect(action()).toHaveFocus());
    fireEvent.click(action());
    fireEvent.keyDown(within(rows()[2]).getByRole('button', { name: 'Cancel' }), { key: 'Escape' });
    expect(within(rows()[2]).queryByRole('group')).not.toBeInTheDocument();
    await waitFor(() => expect(action()).toHaveFocus());
    expect(dialog).toBeInTheDocument();
    expect(houseRuleCheck).not.toHaveBeenCalled();
    expect(rows()[2]).toHaveTextContent('In use · asks before filing');

    fireEvent.click(action());
    fireEvent.click(within(rows()[2]).getByRole('button', { name: 'Use on its own' }));
    await waitFor(() => expect(houseRuleCheck).toHaveBeenCalledWith('3', { kind: 'party', from: 'Acme', to: 'Acme Widgets International, Inc.' }));
    await waitFor(() => expect(rows()[2]).not.toHaveTextContent('asks before filing'));
    expect(rows()[2]).toHaveTextContent('In use');
    expect(within(rows()[2]).queryByRole('button', { name: /Use on its own/ })).not.toBeInTheDocument();
    await waitFor(() => expect(rows()[2]).toHaveFocus());
    // "Use now" is another thing: it puts a spelling seen once in use, unchecked.
    expect(houseRuleUse).not.toHaveBeenCalled();
  });

  // "Use now" and "Forget" take away the button that was pressed: Use now
  // turns into "Use on its own", Forget takes the row. The focus fell to the
  // page, outside the dialog, and a screen reader lost its place. It goes to
  // the spelling's row, the row that took a forgotten one's place, the one
  // before it at the end of the list, or the section's heading when none is
  // left; a Forget refused leaves it on Forget.
  it('keeps the focus in the list when Use now or Forget takes away the button pressed', async () => {
    const third: LearnedRule = { id: '3', kind: 'party', from: 'Northwind Traders', to: 'Northwind', seen: 2, active: true, confirmed: 0, learnedAt: 1716369300 };
    render(<App bridge={createInMemoryBridge({ learnedRules: [learned[0], learned[1], third] })} />);
    const dialog = await openSettings();
    const list = await within(dialog).findByRole('list', { name: 'Learned spellings' });
    const rows = () => within(list).getAllByRole('listitem');
    // Pressed from the keyboard: the button has the focus when it is pressed.
    const press = (button: HTMLElement) => { button.focus(); fireEvent.click(button); };

    press(within(rows()[0]).getByRole('button', { name: 'Use now: Contoso Worldwide, Inc. written as Contoso' }));
    await waitFor(() => expect(within(rows()[0]).queryByRole('button', { name: /Use now/ })).not.toBeInTheDocument());
    await waitFor(() => expect(rows()[0]).toHaveFocus());
    expect(rows()[0]).toHaveTextContent('In use · asks before filing');

    // The first of three: the row that took its place.
    press(within(rows()[0]).getByRole('button', { name: /^Forget:/ }));
    await waitFor(() => expect(rows()).toHaveLength(2));
    await waitFor(() => expect(rows()[0]).toHaveFocus());
    expect(rows()[0]).toHaveTextContent('Quarterly Operations Review written as Meeting Minutes');

    // The last: the one before it.
    press(within(rows()[1]).getByRole('button', { name: /^Forget:/ }));
    await waitFor(() => expect(rows()).toHaveLength(1));
    await waitFor(() => expect(rows()[0]).toHaveFocus());

    // The only one: the section's heading, with "Nothing yet" under it.
    press(within(rows()[0]).getByRole('button', { name: /^Forget:/ }));
    expect(await within(dialog).findByText(/Nothing yet\./)).toBeVisible();
    await waitFor(() => expect(within(dialog).getByText('Learned spellings', { selector: 'summary' })).toHaveFocus());
    expect(dialog).toContainElement(document.activeElement as HTMLElement);
  });

  it('leaves the focus on Forget when forgetting is refused', async () => {
    const base = createInMemoryBridge({ learnedRules: learned });
    const houseRuleForget = vi.fn(async () => { throw { code: 'DATABASE_UNAVAILABLE', message: 'the queue database is locked' }; });
    render(<App bridge={{ ...base, houseRuleForget }} />);
    const dialog = await openSettings();
    const list = await within(dialog).findByRole('list', { name: 'Learned spellings' });
    const forget = within(within(list).getAllByRole('listitem')[1]).getByRole('button', { name: /^Forget:/ });
    forget.focus();
    fireEvent.click(forget);
    expect(await within(dialog).findByRole('alert')).toHaveTextContent('The spelling could not be changed');
    await act(async () => { await Promise.resolve(); });
    expect(forget).toHaveFocus();
  });

  // The same question as in the review window, with the same yes: a press
  // that is part of a double-click, or comes straight after the question
  // appeared, is no answer, even from a pointer that came onto it. Only a
  // press meant for it uses a spelling on its own.
  it('takes as a yes only a press meant for it', async () => {
    const clock = { now: 1000 };
    vi.spyOn(performance, 'now').mockImplementation(() => clock.now);
    drawYes();
    try {
      const unchecked: LearnedRule = { ...learned[1], confirmed: 0 };
      const base = createInMemoryBridge({ learnedRules: [unchecked] });
      const houseRuleCheck = vi.fn(base.houseRuleCheck);
      render(<App bridge={{ ...base, houseRuleCheck }} />);
      const dialog = await openSettings();
      const list = await within(dialog).findByRole('list', { name: 'Learned spellings' });
      pressAt(within(list).getByRole('button', { name: /Use on its own/ }), offTheYes);
      const yes = within(list).getByRole('button', { name: 'Use on its own' });
      moveTo(onTheYes);
      // A press where the pointer is, on the yes, as Chromium makes one.
      const press = (presses: number) => {
        fireEvent.pointerDown(yes, { ...onTheYes, pointerType: 'mouse' });
        if (fireEvent.mouseDown(yes, onTheYes)) yes.focus();
        fireEvent.click(yes, { ...onTheYes, detail: presses });
      };
      // Straight after the question appeared; the second press of a slow
      // double-click; straight after a press it did not take.
      press(1);
      clock.now += PRESS_SETTLES_MS * 4;
      press(2);
      clock.now += PRESS_SETTLES_MS - 1;
      press(1);
      expect(within(list).getByRole('button', { name: 'Cancel' })).toHaveFocus();
      expect(houseRuleCheck).not.toHaveBeenCalled();

      clock.now += PRESS_SETTLES_MS;
      press(1);
      await waitFor(() => expect(houseRuleCheck).toHaveBeenCalledOnce());
      await waitFor(() => expect(within(list).getAllByRole('listitem')[0]).not.toHaveTextContent('asks before filing'));
    } finally {
      vi.restoreAllMocks();
    }
  });

  // The yes takes a press of the pointer only once the pointer came onto it
  // after the question appeared. In Settings a press on another spelling's
  // "Use on its own" closed the question above it, its row moved up, and
  // the new question opened with its yes under the pointer: a press there
  // again used the spelling on its own with its sentence never read.
  it('takes a press only from a pointer that came onto the yes', async () => {
    const clock = { now: 1000 };
    vi.spyOn(performance, 'now').mockImplementation(() => clock.now);
    const box = drawYes();
    try {
      const base = createInMemoryBridge({ learnedRules: [{ ...learned[1], confirmed: 0 }] });
      const houseRuleCheck = vi.fn(base.houseRuleCheck);
      render(<App bridge={{ ...base, houseRuleCheck }} />);
      const dialog = await openSettings();
      const list = await within(dialog).findByRole('list', { name: 'Learned spellings' });
      const ask = (at: Point, pointerType = 'mouse') => {
        pressAt(within(list).getByRole('button', { name: /^Use on its own:/ }), at, pointerType);
        clock.now += PRESS_SETTLES_MS * 2;
        const question = within(list).getByRole('group');
        return { question, yes: within(question).getByRole('button', { name: 'Use on its own' }), cancel: within(question).getByRole('button', { name: 'Cancel' }) };
      };
      const refused = ({ question, cancel }: ReturnType<typeof ask>) => {
        expect(question).toBeInTheDocument();
        expect(cancel).toHaveFocus();
        expect(houseRuleCheck).not.toHaveBeenCalled();
      };

      // Opened under a pointer that has not moved since.
      let asked = ask(onTheYes);
      pressAt(asked.yes, onTheYes);
      refused(asked);
      fireEvent.keyDown(asked.cancel, { key: 'Escape' });

      // The yes moved under a pointer that stayed put: no move, the move a
      // browser sends to where the pointer was, a pixel's move on the yes.
      asked = ask(offTheYes);
      box.current = new DOMRect(100, 160, 120, 30);
      pressAt(asked.yes, offTheYes);
      refused(asked);
      clock.now += PRESS_SETTLES_MS * 2;
      moveTo(offTheYes);
      pressAt(asked.yes, offTheYes);
      refused(asked);
      clock.now += PRESS_SETTLES_MS * 2;
      moveTo({ clientX: offTheYes.clientX, clientY: offTheYes.clientY + 1 });
      pressAt(asked.yes, { clientX: offTheYes.clientX, clientY: offTheYes.clientY + 1 });
      refused(asked);
      fireEvent.keyDown(asked.cancel, { key: 'Escape' });
      box.current = yesAt;

      // A finger: a tap where the tap that opened it was.
      asked = ask(onTheYes, 'touch');
      pressAt(asked.yes, { clientX: onTheYes.clientX + 2, clientY: onTheYes.clientY }, 'touch');
      refused(asked);
      // A tap on the sentence, then on the yes.
      clock.now += PRESS_SETTLES_MS * 2;
      fireEvent.pointerDown(asked.question.querySelector('p')!, { ...offTheYes, pointerType: 'touch' });
      pressAt(asked.yes, onTheYes, 'touch');
      await waitFor(() => expect(houseRuleCheck).toHaveBeenCalledOnce());
    } finally {
      vi.restoreAllMocks();
    }
  });

  it('takes a yes from the pointer moved onto it, or from the keyboard at once', async () => {
    const clock = { now: 1000 };
    vi.spyOn(performance, 'now').mockImplementation(() => clock.now);
    drawYes();
    try {
      const base = createInMemoryBridge({ learnedRules: [{ ...learned[1], confirmed: 0 }, { id: '3', kind: 'party', from: 'Northwind Traders', to: 'Northwind', seen: 2, active: true, confirmed: 0, learnedAt: 1716369300 }] });
      const houseRuleCheck = vi.fn(base.houseRuleCheck);
      render(<App bridge={{ ...base, houseRuleCheck }} />);
      const dialog = await openSettings();
      const list = await within(dialog).findByRole('list', { name: 'Learned spellings' });

      pressAt(within(list).getByRole('button', { name: /^Use on its own: Quarterly/ }), offTheYes);
      clock.now += PRESS_SETTLES_MS * 2;
      moveTo(onTheYes);
      pressAt(within(list).getByRole('button', { name: 'Use on its own' }), onTheYes);
      await waitFor(() => expect(houseRuleCheck).toHaveBeenCalledWith('2', { kind: 'document_type', from: 'Quarterly Operations Review', to: 'Meeting Minutes' }));

      // From the keyboard, straight after the question appeared.
      const action = await within(list).findByRole('button', { name: /^Use on its own: Northwind/ });
      action.focus();
      fireEvent.keyDown(action, { key: 'Enter' });
      fireEvent.click(action, { detail: 0 });
      const yes = within(list).getByRole('button', { name: 'Use on its own' });
      fireEvent.keyDown(document.activeElement!, { key: 'Tab', shiftKey: true });
      yes.focus();
      fireEvent.keyDown(yes, { key: 'Enter' });
      fireEvent.click(yes, { detail: 0 });
      await waitFor(() => expect(houseRuleCheck).toHaveBeenCalledWith('3', { kind: 'party', from: 'Northwind Traders', to: 'Northwind' }));
    } finally {
      vi.restoreAllMocks();
    }
  });

  // As in the review window: while one spelling is asked about, the other
  // spellings' actions wait. A press on another's "Use on its own" closed
  // the question above it and opened its own where the pointer was.
  it('leaves every other spelling\'s actions waiting while it asks', async () => {
    const rules: LearnedRule[] = [
      { id: '1', kind: 'party', from: 'Contoso Worldwide, Inc.', to: 'Contoso', seen: 2, active: true, confirmed: 0, learnedAt: 1 },
      { id: '2', kind: 'party', from: 'Ridgeline Cartography LLC', to: 'Ridgeline Cartography', seen: 2, active: true, confirmed: 0, learnedAt: 2 },
      { id: '3', kind: 'party', from: 'Northwind Traders', to: 'Northwind', seen: 1, active: false, confirmed: 0, learnedAt: 3 },
    ];
    const base = createInMemoryBridge({ learnedRules: rules });
    const houseRuleUse = vi.fn(base.houseRuleUse);
    const houseRuleForget = vi.fn(base.houseRuleForget);
    render(<App bridge={{ ...base, houseRuleUse, houseRuleForget }} />);
    const dialog = await openSettings();
    const list = await within(dialog).findByRole('list', { name: 'Learned spellings' });
    const rows = () => within(list).getAllByRole('listitem');
    fireEvent.click(within(rows()[0]).getByRole('button', { name: /^Use on its own:/ }));
    const question = within(rows()[0]).getByRole('group');
    const others = [...within(rows()[1]).getAllByRole('button'), ...within(rows()[2]).getAllByRole('button')];
    expect(others.map((button) => button.getAttribute('aria-label'))).toEqual([
      'Use on its own: Ridgeline Cartography LLC written as Ridgeline Cartography', 'Forget: Ridgeline Cartography LLC written as Ridgeline Cartography',
      'Use now: Northwind Traders written as Northwind', 'Forget: Northwind Traders written as Northwind',
    ]);
    for (const button of others) {
      expect(button).toBeDisabled();
      fireEvent.click(button);
    }
    expect(question).toBeInTheDocument();
    expect(within(list).getAllByRole('group')).toHaveLength(1);
    expect(houseRuleUse).not.toHaveBeenCalled();
    expect(houseRuleForget).not.toHaveBeenCalled();

    fireEvent.click(within(question).getByRole('button', { name: 'Cancel' }));
    for (const button of others) expect(button).toBeEnabled();
  });

  // Opened by a press, the question takes the focus where it is: the
  // browser scrolled its Cancel into view when it opened past the bottom of
  // the dialog's body, and brought whatever the scroll brought under the
  // pointer for the next press. A press on its sentence leaves the focus on
  // Cancel, not on the row, where Escape closed Settings.
  it('moves nothing under the pointer that opened the question, and keeps Escape for the question', async () => {
    const focus = vi.spyOn(HTMLElement.prototype, 'focus');
    try {
      const unchecked: LearnedRule = { ...learned[1], confirmed: 0 };
      const base = createInMemoryBridge({ learnedRules: [unchecked] });
      const houseRuleCheck = vi.fn(base.houseRuleCheck);
      render(<App bridge={{ ...base, houseRuleCheck }} />);
      const dialog = await openSettings();
      const list = await within(dialog).findByRole('list', { name: 'Learned spellings' });
      const action = within(list).getByRole('button', { name: /Use on its own/ });
      fireEvent.pointerDown(action);
      fireEvent.click(action, { detail: 1 });
      const question = within(list).getByRole('group');
      const cancel = within(question).getByRole('button', { name: 'Cancel' });
      expect(cancel).toHaveFocus();
      expect(focus.mock.calls.filter((_, index) => focus.mock.contexts[index] === cancel).map((call) => call[0])).toEqual([{ preventScroll: true }]);

      const sentence = question.querySelector('p')!;
      // A press on the sentence focuses the row, as it does in Chromium.
      fireEvent.pointerDown(sentence);
      within(list).getByRole('listitem').focus();
      fireEvent.click(sentence, { detail: 1 });
      expect(cancel).toHaveFocus();
      fireEvent.keyDown(cancel, { key: 'Escape' });
      expect(within(list).queryByRole('group')).not.toBeInTheDocument();
      expect(dialog).toBeInTheDocument();
      expect(houseRuleCheck).not.toHaveBeenCalled();
    } finally {
      focus.mockRestore();
    }
  });

  // A press on the sentence let go off the question clicks nothing in it,
  // and leaves the focus on the row. Escape there closed Settings, and with
  // it whatever had not been saved. Wherever the focus is in the dialog,
  // Escape takes back the question and nothing else.
  it('takes back the question, and only the question, on Escape wherever the focus is in the dialog', async () => {
    const base = createInMemoryBridge({ learnedRules: [{ ...learned[1], confirmed: 0 }] });
    const houseRuleCheck = vi.fn(base.houseRuleCheck);
    const onSave = vi.fn(base.saveSettings);
    render(<App bridge={{ ...base, houseRuleCheck, saveSettings: onSave }} />);
    const dialog = await openSettings();
    const list = await within(dialog).findByRole('list', { name: 'Learned spellings' });
    const names = within(dialog).getByRole('textbox', { name: "Your organization's names" });
    fireEvent.change(names, { target: { value: 'Fabrikam LLP' } });
    const action = () => within(list).getByRole('button', { name: /^Use on its own:/ });
    for (const focusElsewhere of [
      () => within(list).getByRole('listitem').focus(),
      () => names.focus(),
      () => within(dialog).getByRole('button', { name: 'Save settings' }).focus(),
    ]) {
      pressAt(action(), offTheYes);
      const question = within(list).getByRole('group');
      fireEvent.pointerDown(question.querySelector('p')!, { ...offTheYes, pointerType: 'mouse' });
      focusElsewhere();
      fireEvent.keyDown(document.activeElement!, { key: 'Escape' });
      expect(question).not.toBeInTheDocument();
      expect(dialog).toBeInTheDocument();
      await waitFor(() => expect(action()).toHaveFocus());
    }
    expect(names).toHaveValue('Fabrikam LLP');
    expect(houseRuleCheck).not.toHaveBeenCalled();
    expect(onSave).not.toHaveBeenCalled();
    // With no question open, Escape closes Settings as before.
    fireEvent.keyDown(action(), { key: 'Escape' });
    await waitFor(() => expect(dialog).not.toBeInTheDocument());
  });

  // Closing the section over an open question takes the question back: it
  // is not left asking where it cannot be seen.
  it('takes back the question when its section is closed', async () => {
    render(<App bridge={createInMemoryBridge({ learnedRules: [{ ...learned[1], confirmed: 0 }] })} />);
    const dialog = await openSettings();
    const list = await within(dialog).findByRole('list', { name: 'Learned spellings' });
    fireEvent.click(within(list).getByRole('button', { name: /^Use on its own:/ }));
    expect(within(list).getByRole('group')).toBeInTheDocument();
    fireEvent.click(within(dialog).getByText('Learned spellings', { selector: 'summary' }));
    await waitFor(() => expect(within(list).queryByRole('group')).not.toBeInTheDocument());
    expect(dialog).toBeInTheDocument();
  });

  it('says a document type spelling renames every later document of that type, in its own words', async () => {
    const type: LearnedRule = { ...learned[1], confirmed: 0 };
    render(<App bridge={createInMemoryBridge({ learnedRules: [type] })} />);
    const dialog = await openSettings();
    const list = await within(dialog).findByRole('list', { name: 'Learned spellings' });
    fireEvent.click(within(list).getByRole('button', { name: /Use on its own/ }));
    expect(within(list).getByRole('group')).toHaveAccessibleDescription(/^Every later document Intern reads as Quarterly Operations Review will be called Meeting Minutes instead, and may be filed under that name without your check\./);
  });

  // The list can be older than the spelling: an approval in the queue can
  // respell it while Settings is open. Checking the spelling the person was
  // not shown would let Intern file names they never agreed to.
  it('checks nothing when the spelling changed since the list was shown', async () => {
    const shown: LearnedRule = { ...learned[1], confirmed: 0 };
    const base = createInMemoryBridge({ learnedRules: [shown] });
    let current = shown;
    const houseRulesList = vi.fn(async () => [current]);
    const houseRuleCheck = vi.fn(async (id: string, spelling: HouseRule) => {
      if (spelling.to !== current.to) throw { code: 'RULE_CHANGED', message: 'learned spelling is not in use as it was shown' };
      return base.houseRuleCheck(id, spelling);
    });
    render(<App bridge={{ ...base, houseRulesList, houseRuleCheck }} />);
    const dialog = await openSettings();
    const list = await within(dialog).findByRole('list', { name: 'Learned spellings' });
    current = { ...shown, to: 'Board Minutes' };
    fireEvent.click(within(list).getByRole('button', { name: /Use on its own/ }));
    fireEvent.click(within(list).getByRole('button', { name: 'Use on its own' }));
    expect(await within(dialog).findByRole('alert')).toHaveTextContent('That spelling changed since the list was shown');
    expect(houseRuleCheck).toHaveBeenCalledWith('2', { kind: 'document_type', from: 'Quarterly Operations Review', to: 'Meeting Minutes' });
    await waitFor(() => expect(within(list).getAllByRole('listitem')[0]).toHaveTextContent('Quarterly Operations Review written as Board Minutes'));
  });

  it('reports a spelling that could not be changed and keeps the list', async () => {
    const base = createInMemoryBridge({ learnedRules: learned });
    const houseRuleForget = vi.fn(async () => { throw { code: 'DATABASE_UNAVAILABLE', message: 'the queue database is locked' }; });
    render(<App bridge={{ ...base, houseRuleForget }} />);
    const dialog = await openSettings();
    const list = await within(dialog).findByRole('list', { name: 'Learned spellings' });
    fireEvent.click(within(within(list).getAllByRole('listitem')[0]).getByRole('button', { name: /Forget:/ }));
    expect(await within(dialog).findByRole('alert')).toHaveTextContent('The spelling could not be changed');
    expect(within(list).getAllByRole('listitem')).toHaveLength(2);
  });
});
