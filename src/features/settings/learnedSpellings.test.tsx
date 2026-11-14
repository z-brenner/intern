import { act, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { App } from '../../App';
import { createInMemoryBridge } from '../../lib/inMemoryBridge';
import { clearField, commitComposition, composeInto, fillWithoutTyping, typeInto } from '../review/typingTesting';
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
    expect(within(question).getByRole('textbox', { name: 'Type Acme Widgets International, Inc. to use it on its own' })).toHaveFocus();
    expect(within(question).getByRole('button', { name: 'Use on its own' })).toBeDisabled();
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
    typeInto(within(rows()[2]).getByRole('textbox'), 'Acme Widgets International, Inc.');
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

  // The same question as in the review window: the yes answers only once the
  // spelling is typed out, which no press does. A press on another
  // spelling's "Use on its own" closed the question above it and opened the
  // next with its yes under the pointer, and rules on which presses the yes
  // would take kept being got round.
  it('answers only once the spelling is typed out: until then its yes waits and Enter in its field does nothing, and Enter never saves Settings', async () => {
    const third: LearnedRule = { id: '3', kind: 'party', from: 'Northwind Traders', to: 'Northwind', seen: 2, active: true, confirmed: 0, learnedAt: 1716369300 };
    const base = createInMemoryBridge({ learnedRules: [learned[0], third] });
    const houseRuleCheck = vi.fn(base.houseRuleCheck);
    const saveSettings = vi.fn(base.saveSettings);
    render(<App bridge={{ ...base, houseRuleCheck, saveSettings }} />);
    const dialog = await openSettings();
    const list = await within(dialog).findByRole('list', { name: 'Learned spellings' });
    fireEvent.click(within(list).getByRole('button', { name: /^Use on its own:/ }));
    const question = within(list).getByRole('group', { name: 'Use on its own: Northwind Traders written as Northwind' });
    const field = within(question).getByRole('textbox', { name: 'Type Northwind to use it on its own' });
    const yes = within(question).getByRole('button', { name: 'Use on its own' });
    expect(field).toHaveFocus();
    for (const typed of ['', 'Northwin', 'Northwinds', 'Northwind Traders']) {
      clearField(field);
      typeInto(field, typed);
      expect(yes, typed).toBeDisabled();
      fireEvent.pointerDown(yes, { pointerType: 'mouse' });
      fireEvent.click(yes, { detail: 1 });
      fireEvent.click(yes, { detail: 0 });
      fireEvent.keyDown(field, { key: 'Enter' });
      expect(question, typed).toBeInTheDocument();
    }
    clearField(field);
    typeInto(field, 'northwind');
    expect(yes).toBeEnabled();
    // Ctrl+Enter is not Enter.
    fireEvent.keyDown(field, { key: 'Enter', ctrlKey: true });
    expect(question).toBeInTheDocument();
    expect(houseRuleCheck).not.toHaveBeenCalled();
    fireEvent.keyDown(field, { key: 'Enter' });
    await waitFor(() => expect(houseRuleCheck).toHaveBeenCalledWith('3', { kind: 'party', from: 'Northwind Traders', to: 'Northwind' }));
    expect(houseRuleCheck).toHaveBeenCalledOnce();
    await waitFor(() => expect(within(list).getAllByRole('listitem')[1]).toHaveFocus());
    expect(dialog).toBeInTheDocument();
    expect(saveSettings).not.toHaveBeenCalled();
  });

  it('takes the spelling typed in any case, with or without its accents, spaces and punctuation, and not a letter missing or added', async () => {
    const rules: LearnedRule[] = [
      { id: '7', kind: 'party', from: 'Contoso Limited', to: 'Contoso Ltd.', seen: 2, active: true, confirmed: 0, learnedAt: 7 },
      { id: '8', kind: 'party', from: 'Mueller GmbH', to: 'Müller', seen: 2, active: true, confirmed: 0, learnedAt: 8 },
    ];
    const base = createInMemoryBridge({ learnedRules: rules });
    const houseRuleCheck = vi.fn(base.houseRuleCheck);
    render(<App bridge={{ ...base, houseRuleCheck }} />);
    const dialog = await openSettings();
    const list = await within(dialog).findByRole('list', { name: 'Learned spellings' });
    for (const [rule, right, wrong] of [[rules[0], 'contoso ltd', ['contoso lt', 'contoso ltdd']], [rules[1], 'Muller', ['Mller', 'Mueller']]] as const) {
      fireEvent.click(await within(list).findByRole('button', { name: `Use on its own: ${rule.from} written as ${rule.to}` }));
      const question = within(list).getByRole('group');
      const field = within(question).getByRole('textbox');
      const yes = within(question).getByRole('button', { name: 'Use on its own' });
      for (const typed of wrong) {
        clearField(field);
        typeInto(field, typed);
        expect(yes, typed).toBeDisabled();
      }
      clearField(field);
      typeInto(field, right);
      expect(yes, right).toBeEnabled();
      fireEvent.click(yes);
      await waitFor(() => expect(houseRuleCheck).toHaveBeenLastCalledWith(rule.id, { kind: rule.kind, from: rule.from, to: rule.to }));
    }
  });

  it('takes nothing into its field but typing, an input method\'s included, and starts it empty every time it asks', async () => {
    const rules: LearnedRule[] = [
      { id: '1', kind: 'party', from: 'Contoso Worldwide, Inc.', to: 'Contoso', seen: 2, active: true, confirmed: 0, learnedAt: 1 },
      { id: '9', kind: 'party', from: 'Yamada Taro', to: '山田太郎', seen: 2, active: true, confirmed: 0, learnedAt: 9 },
    ];
    const base = createInMemoryBridge({ learnedRules: rules });
    const houseRuleCheck = vi.fn(base.houseRuleCheck);
    render(<App bridge={{ ...base, houseRuleCheck }} />);
    const dialog = await openSettings();
    const list = await within(dialog).findByRole('list', { name: 'Learned spellings' });
    const contoso = () => within(list).getByRole('button', { name: 'Use on its own: Contoso Worldwide, Inc. written as Contoso' });
    const asked = () => {
      const question = within(list).getByRole('group');
      return { field: within(question).getByRole('textbox'), yes: within(question).getByRole('button', { name: 'Use on its own' }), cancel: within(question).getByRole('button', { name: 'Cancel' }) };
    };

    fireEvent.click(contoso());
    expect(fillWithoutTyping(asked().field, 'Contoso')).toEqual([]);
    expect(asked().field).toHaveValue('');
    expect(asked().yes).toBeDisabled();
    expect(asked().field).toHaveAttribute('autocomplete', 'off');
    expect(asked().field).toHaveAttribute('spellcheck', 'false');
    const name = asked().field.getAttribute('name');

    // Typed, then taken back: asked again, the field is empty.
    typeInto(asked().field, 'Contoso');
    expect(asked().yes).toBeEnabled();
    fireEvent.keyDown(asked().field, { key: 'Escape' });
    fireEvent.click(contoso());
    expect(asked().field).toHaveValue('');
    expect(asked().yes).toBeDisabled();
    expect(asked().field.getAttribute('name')).not.toBe(name);
    typeInto(asked().field, 'Contoso');
    fireEvent.click(asked().cancel);

    // Another spelling's question starts empty too, and takes a name composed with an input method.
    fireEvent.click(within(list).getByRole('button', { name: 'Use on its own: Yamada Taro written as 山田太郎' }));
    expect(asked().field).toHaveValue('');
    composeInto(asked().field, ['やまだ', '山田', '山田太郎']);
    expect(asked().yes).toBeEnabled();
    commitComposition(asked().field);
    expect(houseRuleCheck).not.toHaveBeenCalled();
    fireEvent.keyDown(asked().field, { key: 'Enter' });
    await waitFor(() => expect(houseRuleCheck).toHaveBeenCalledWith('9', { kind: 'party', from: 'Yamada Taro', to: '山田太郎' }));
    expect(houseRuleCheck).toHaveBeenCalledOnce();
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

  // Opened by a press, the question takes the focus in its field where it
  // is: the browser scrolled the question into view when it opened past the
  // bottom of the dialog's body, and brought whatever the scroll brought
  // under the pointer for the next press. Opened from the keyboard, it is
  // brought into sight.
  it('moves nothing under the pointer that opened the question, and is brought into sight when a key opened it', async () => {
    const focus = vi.spyOn(HTMLElement.prototype, 'focus');
    // jsdom scrolls nothing, and has no scrollIntoView to call.
    const scrollIntoView = vi.fn();
    Object.defineProperty(Element.prototype, 'scrollIntoView', { value: scrollIntoView, configurable: true, writable: true });
    try {
      const unchecked: LearnedRule = { ...learned[1], confirmed: 0 };
      render(<App bridge={createInMemoryBridge({ learnedRules: [unchecked] })} />);
      const dialog = await openSettings();
      const list = await within(dialog).findByRole('list', { name: 'Learned spellings' });
      const action = within(list).getByRole('button', { name: /Use on its own/ });
      fireEvent.pointerDown(action, { pointerType: 'mouse' });
      fireEvent.click(action, { detail: 1 });
      const field = within(list).getByRole('textbox');
      expect(field).toHaveFocus();
      expect(focus.mock.calls.filter((_, index) => focus.mock.contexts[index] === field)).toEqual([[{ preventScroll: true }]]);
      expect(scrollIntoView).not.toHaveBeenCalled();
      fireEvent.keyDown(field, { key: 'Escape' });

      const again = await within(list).findByRole('button', { name: /Use on its own/ });
      fireEvent.keyDown(again, { key: 'Enter' });
      fireEvent.click(again, { detail: 0 });
      expect(within(list).getByRole('textbox')).toHaveFocus();
      expect(scrollIntoView.mock.contexts).toEqual([within(list).getByRole('group')]);
      expect(scrollIntoView).toHaveBeenCalledWith({ block: 'nearest' });
    } finally {
      focus.mockRestore();
      Reflect.deleteProperty(Element.prototype, 'scrollIntoView');
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
      () => within(list).getByRole('button', { name: 'Cancel' }).focus(),
    ]) {
      fireEvent.pointerDown(action(), { pointerType: 'mouse' });
      fireEvent.click(action(), { detail: 1 });
      const question = within(list).getByRole('group');
      typeInto(within(question).getByRole('textbox'), 'Meeting');
      fireEvent.pointerDown(question.querySelector('p')!, { pointerType: 'mouse' });
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

  // A press on the sentence, or on the waiting yes, took the focus off the
  // question's field, and the spelling typed next went nowhere - or, in
  // the review window, to its shortcuts. What is typed is the question's
  // wherever the focus is in the dialog, but in another field.
  it('takes a character typed after the focus left its field into the field', async () => {
    const base = createInMemoryBridge({ learnedRules: [{ ...learned[1], confirmed: 0 }] });
    const houseRuleCheck = vi.fn(base.houseRuleCheck);
    const onSave = vi.fn(base.saveSettings);
    render(<App bridge={{ ...base, houseRuleCheck, saveSettings: onSave }} />);
    const dialog = await openSettings();
    const list = await within(dialog).findByRole('list', { name: 'Learned spellings' });
    const names = within(dialog).getByRole('textbox', { name: "Your organization's names" });
    fireEvent.click(within(list).getByRole('button', { name: /^Use on its own:/ }));
    const field = within(list).getByRole('textbox');
    for (const [where, focusAway] of [
      ['on its row', () => within(list).getByRole('listitem').focus()],
      ['on nothing', () => (document.activeElement as HTMLElement).blur()],
      ['on Save', () => within(dialog).getByRole('button', { name: 'Save settings' }).focus()],
      ['on its Cancel', () => within(list).getByRole('button', { name: 'Cancel' }).focus()],
    ] as const) {
      focusAway();
      expect(field, where).not.toHaveFocus();
      expect(fireEvent.keyDown(document.activeElement ?? document.body, { key: 'M' }), where).toBe(true);
      expect(field, where).toHaveFocus();
    }
    // Not from another field, whose keys are its own.
    names.focus();
    fireEvent.keyDown(names, { key: 'M' });
    expect(names).toHaveFocus();
    field.focus();
    typeInto(field, 'Meeting Minutes');
    fireEvent.keyDown(field, { key: 'Enter' });
    await waitFor(() => expect(houseRuleCheck).toHaveBeenCalledWith('2', { kind: 'document_type', from: 'Quarterly Operations Review', to: 'Meeting Minutes' }));
    expect(onSave).not.toHaveBeenCalled();
    expect(dialog).toBeInTheDocument();
  });

  // An undo or a redo goes to the field edited last, wherever the focus
  // is, and takes the focus there: Ctrl+Z in the question's field undid an
  // unsaved edit to this machine's name and moved the focus into it, where
  // the spelling then went.
  it('changes no setting but the one with the focus while it asks: no undo or redo reaches an unsaved edit behind it', async () => {
    const base = createInMemoryBridge({ learnedRules: [{ ...learned[1], confirmed: 0 }] });
    const houseRuleCheck = vi.fn(base.houseRuleCheck);
    const saveSettings = vi.fn(base.saveSettings);
    render(<App bridge={{ ...base, houseRuleCheck, saveSettings }} />);
    const dialog = await openSettings();
    const list = await within(dialog).findByRole('list', { name: 'Learned spellings' });
    const machine = within(dialog).getByRole('textbox', { name: "This machine's name" });
    const names = within(dialog).getByRole('textbox', { name: "Your organization's names" });
    fireEvent.change(machine, { target: { value: 'Front desk' } });
    /** Whether the browser may make the undo or redo it is about to in `field`: an event nobody refused. */
    const undoes = (field: HTMLElement, inputType: string) => fireEvent(field, new InputEvent('beforeinput', { inputType, bubbles: true, cancelable: true }));
    const action = within(list).getByRole('button', { name: /^Use on its own:/ });
    action.focus();
    expect(undoes(machine, 'historyUndo')).toBe(true);

    fireEvent.click(action);
    const question = within(list).getByRole('group');
    const field = within(question).getByRole('textbox');
    const focusOn: [string, () => void][] = [
      ['in its field', () => field.focus()],
      ['on its Cancel', () => within(question).getByRole('button', { name: 'Cancel' }).focus()],
      ['on nothing, after a press on its sentence', () => (document.activeElement as HTMLElement).blur()],
    ];
    for (const [where, focus] of focusOn) {
      focus();
      for (const [other, inputType] of [[machine, 'historyUndo'], [machine, 'historyRedo'], [names, 'historyUndo']] as const) {
        expect(undoes(other, inputType), `${inputType}, the focus ${where}`).toBe(false);
      }
    }
    // In the field with the focus, the browser's undo is the person's.
    machine.focus();
    expect(undoes(machine, 'historyUndo')).toBe(true);
    field.focus();
    typeInto(field, 'Meeting Minutes');
    fireEvent.keyDown(field, { key: 'Enter' });
    await waitFor(() => expect(houseRuleCheck).toHaveBeenCalledWith('2', { kind: 'document_type', from: 'Quarterly Operations Review', to: 'Meeting Minutes' }));
    expect(question).not.toBeInTheDocument();
    expect(machine).toHaveValue('Front desk');
    expect(dialog).toBeInTheDocument();
    expect(saveSettings).not.toHaveBeenCalled();
    // Asking no more, it leaves undo to the browser again.
    within(dialog).getByRole('button', { name: 'Save settings' }).focus();
    expect(undoes(machine, 'historyUndo')).toBe(true);
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
    typeInto(within(list).getByRole('textbox'), 'Meeting Minutes');
    fireEvent.click(within(list).getByRole('button', { name: 'Use on its own' }));
    expect(await within(dialog).findByRole('alert')).toHaveTextContent('That spelling changed since the list was shown');
    expect(houseRuleCheck).toHaveBeenCalledWith('2', { kind: 'document_type', from: 'Quarterly Operations Review', to: 'Meeting Minutes' });
    await waitFor(() => expect(within(list).getAllByRole('listitem')[0]).toHaveTextContent('Quarterly Operations Review written as Board Minutes'));
  });

  // A spelling forgotten since the list was shown - in the review window,
  // or by putting a document's own words back - stayed in the list as in
  // use, its "Use on its own" refused again however often it was tried.
  it('takes a spelling forgotten since the list was shown out of the list when using it on its own finds it gone', async () => {
    const unchecked: LearnedRule = { id: '3', kind: 'party', from: 'Northwind Traders', to: 'Northwind', seen: 2, active: true, confirmed: 0, learnedAt: 1716369300 };
    const base = createInMemoryBridge({ learnedRules: [learned[0], unchecked, learned[1]] });
    const houseRuleCheck = vi.fn(base.houseRuleCheck);
    render(<App bridge={{ ...base, houseRuleCheck }} />);
    const dialog = await openSettings();
    const list = await within(dialog).findByRole('list', { name: 'Learned spellings' });
    const rows = () => within(list).getAllByRole('listitem');
    await base.houseRuleForget('3');
    fireEvent.click(within(rows()[1]).getByRole('button', { name: 'Use on its own: Northwind Traders written as Northwind' }));
    typeInto(within(rows()[1]).getByRole('textbox'), 'Northwind');
    fireEvent.click(within(rows()[1]).getByRole('button', { name: 'Use on its own' }));
    expect(await within(dialog).findByRole('alert')).toHaveTextContent('That spelling is no longer in the list');
    expect(houseRuleCheck).toHaveBeenCalledWith('3', { kind: 'party', from: 'Northwind Traders', to: 'Northwind' });
    await waitFor(() => expect(rows()).toHaveLength(2));
    expect(list).not.toHaveTextContent('Northwind');
    // The focus goes to the row that took its place, as when it is forgotten here.
    await waitFor(() => expect(rows()[1]).toHaveFocus());
    expect(rows()[1]).toHaveTextContent('Quarterly Operations Review written as Meeting Minutes');
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
