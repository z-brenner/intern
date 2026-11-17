import { act, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { App } from '../../App';
import { UseOnItsOwnQuestion, typedOut } from '../../components/UseOnItsOwn';
import { createInMemoryBridge, createLearnedSpellingBridge } from '../../lib/inMemoryBridge';
import { openRow } from '../queue/gridTesting';
import { clearField, commitComposition, composeInto, fillWithoutTyping, insertAtOnce, typeInto } from './typingTesting';
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

describe('the spellings ATLAS has learned', () => {
  it('says there is nothing to show until review has taught something', async () => {
    render(<App bridge={createInMemoryBridge()} />);
    const dialog = await openSettings();
    expect(await within(dialog).findByText(/Nothing yet\. Correct a name in review/)).toBeVisible();
    expect(within(dialog).queryByRole('list', { name: 'Learned spellings' })).not.toBeInTheDocument();
    // What the section offers, and no more: Settings has no way to change a spelling.
    expect(within(dialog).getByText(/^When you correct a name, ATLAS remembers/)).toHaveTextContent('When you correct a name, ATLAS remembers the spelling (for example “Contoso Worldwide, Inc.” → “Contoso”). New spellings wait for your OK before ATLAS applies them by itself.');
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
    expect(rows[0]).toHaveTextContent('Name');
    expect(rows[0]).toHaveTextContent('Contoso Worldwide, Inc. written as Contoso');
    expect(rows[0]).toHaveTextContent('Seen once · used after one more correction');
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
  // waits for a person until they choose "Always use" for it: the list
  // says it asks before filing. "Use now" puts a spelling in use, no more
  // checked than one learned from two edits. It counted as the check, and
  // one click on a wrong spelling filed the next name it changed without
  // review.
  it('says a spelling in use asks before filing until it is used on its own', async () => {
    const unchecked: LearnedRule = { ...learned[1], confirmed: 0 };
    render(<App bridge={createInMemoryBridge({ learnedRules: [learned[0], unchecked] })} />);
    const dialog = await openSettings();
    const list = await within(dialog).findByRole('list', { name: 'Learned spellings' });
    expect(within(list).getAllByRole('listitem')[1]).toHaveTextContent('In use · waits for your OK');

    fireEvent.click(within(within(list).getAllByRole('listitem')[0]).getByRole('button', { name: /Use now:/ }));
    await waitFor(() => expect(within(list).getAllByRole('listitem')[0]).toHaveTextContent('In use · waits for your OK'));
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

  // No approval checks a spelling: "Always use" is the only way. It is
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
    const action = () => within(rows()[2]).getByRole('button', { name: 'Always use: Acme written as Acme Widgets International, Inc.' });
    // Not for a spelling seen once, nor for one already checked.
    expect(within(rows()[0]).queryByRole('button', { name: /Always use/ })).not.toBeInTheDocument();
    expect(within(rows()[1]).queryByRole('button', { name: /Always use/ })).not.toBeInTheDocument();

    fireEvent.click(action());
    const question = within(rows()[2]).getByRole('group', { name: 'Always use: Acme written as Acme Widgets International, Inc.' });
    expect(question).toHaveAccessibleDescription('From now on ATLAS will write Acme as Acme Widgets International, Inc., and may file the name without asking you. Only do this if it is right every time.');
    expect(within(question).getByRole('textbox', { name: 'Type Acme Widgets International, Inc. to always use it' })).toHaveFocus();
    expect(within(question).getByRole('button', { name: 'Always use' })).toBeDisabled();
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
    expect(rows()[2]).toHaveTextContent('In use · waits for your OK');

    fireEvent.click(action());
    typeInto(within(rows()[2]).getByRole('textbox'), 'Acme Widgets International, Inc.');
    fireEvent.click(within(rows()[2]).getByRole('button', { name: 'Always use' }));
    await waitFor(() => expect(houseRuleCheck).toHaveBeenCalledWith('3', { kind: 'party', from: 'Acme', to: 'Acme Widgets International, Inc.' }));
    await waitFor(() => expect(rows()[2]).not.toHaveTextContent('waits for your OK'));
    expect(rows()[2]).toHaveTextContent('In use');
    expect(within(rows()[2]).queryByRole('button', { name: /Always use/ })).not.toBeInTheDocument();
    await waitFor(() => expect(rows()[2]).toHaveFocus());
    // "Use now" is another thing: it puts a spelling seen once in use, unchecked.
    expect(houseRuleUse).not.toHaveBeenCalled();
  });

  // "Use now" and "Forget" take away the button that was pressed: Use now
  // turns into "Always use", Forget takes the row. The focus fell to the
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
    expect(rows()[0]).toHaveTextContent('In use · waits for your OK');

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

  // The yes answers only once the spelling is typed out, which no press
  // does. A press on another spelling's "Always use" closed the question
  // above it and opened the next with its yes under the pointer, and rules
  // on which presses the yes would take kept being got round.
  it('answers only once the spelling is typed out: until then its yes waits and Enter in its field does nothing, and Enter never saves Settings', async () => {
    const third: LearnedRule = { id: '3', kind: 'party', from: 'Northwind Traders', to: 'Northwind', seen: 2, active: true, confirmed: 0, learnedAt: 1716369300 };
    const base = createInMemoryBridge({ learnedRules: [learned[0], third] });
    const houseRuleCheck = vi.fn(base.houseRuleCheck);
    const saveSettings = vi.fn(base.saveSettings);
    render(<App bridge={{ ...base, houseRuleCheck, saveSettings }} />);
    const dialog = await openSettings();
    const list = await within(dialog).findByRole('list', { name: 'Learned spellings' });
    fireEvent.click(within(list).getByRole('button', { name: /^Always use:/ }));
    const question = within(list).getByRole('group', { name: 'Always use: Northwind Traders written as Northwind' });
    const field = within(question).getByRole('textbox', { name: 'Type Northwind to always use it' });
    const yes = within(question).getByRole('button', { name: 'Always use' });
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
      fireEvent.click(await within(list).findByRole('button', { name: `Always use: ${rule.from} written as ${rule.to}` }));
      const question = within(list).getByRole('group');
      const field = within(question).getByRole('textbox');
      const yes = within(question).getByRole('button', { name: 'Always use' });
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
    const contoso = () => within(list).getByRole('button', { name: 'Always use: Contoso Worldwide, Inc. written as Contoso' });
    const asked = () => {
      const question = within(list).getByRole('group');
      return { field: within(question).getByRole('textbox'), yes: within(question).getByRole('button', { name: 'Always use' }), cancel: within(question).getByRole('button', { name: 'Cancel' }) };
    };

    fireEvent.click(contoso());
    expect(fillWithoutTyping(asked().field, 'Contoso')).toEqual([]);
    expect(asked().field).toHaveValue('');
    expect(asked().yes).toBeDisabled();
    expect(asked().field).toHaveAttribute('autocomplete', 'off');
    expect(asked().field).toHaveAttribute('autocorrect', 'off');
    expect(asked().field).toHaveAttribute('autocapitalize', 'off');
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
    fireEvent.click(within(list).getByRole('button', { name: 'Always use: Yamada Taro written as 山田太郎' }));
    expect(asked().field).toHaveValue('');
    composeInto(asked().field, ['やまだ', '山田', '山田太郎']);
    expect(asked().yes).toBeEnabled();
    commitComposition(asked().field);
    // WebKit ends the composition before the Enter that commits it, a key it marks as the input method's.
    fireEvent.keyDown(asked().field, { key: 'Enter', keyCode: 229 });
    expect(houseRuleCheck).not.toHaveBeenCalled();
    fireEvent.keyDown(asked().field, { key: 'Enter' });
    await waitFor(() => expect(houseRuleCheck).toHaveBeenCalledWith('9', { kind: 'party', from: 'Yamada Taro', to: '山田太郎' }));
    expect(houseRuleCheck).toHaveBeenCalledOnce();
  });

  // While one spelling is asked about, the other spellings' actions wait. A
  // press on another's "Always use" closed the question above it and
  // opened its own where the pointer was.
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
    fireEvent.click(within(rows()[0]).getByRole('button', { name: /^Always use:/ }));
    const question = within(rows()[0]).getByRole('group');
    const others = [...within(rows()[1]).getAllByRole('button'), ...within(rows()[2]).getAllByRole('button')];
    expect(others.map((button) => button.getAttribute('aria-label'))).toEqual([
      'Always use: Ridgeline Cartography LLC written as Ridgeline Cartography', 'Forget: Ridgeline Cartography LLC written as Ridgeline Cartography',
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
      const action = within(list).getByRole('button', { name: /Always use/ });
      fireEvent.pointerDown(action, { pointerType: 'mouse' });
      fireEvent.click(action, { detail: 1 });
      const field = within(list).getByRole('textbox');
      expect(field).toHaveFocus();
      expect(focus.mock.calls.filter((_, index) => focus.mock.contexts[index] === field)).toEqual([[{ preventScroll: true }]]);
      expect(scrollIntoView).not.toHaveBeenCalled();
      fireEvent.keyDown(field, { key: 'Escape' });

      const again = await within(list).findByRole('button', { name: /Always use/ });
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
    const names = within(dialog).getByRole('textbox', { name: "Your organizations" });
    fireEvent.change(names, { target: { value: 'Fabrikam LLP' } });
    const action = () => within(list).getByRole('button', { name: /^Always use:/ });
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
  // question's field, and the spelling typed next went nowhere. What is
  // typed is the question's wherever the focus is in the dialog, but in
  // another field.
  it('takes a character typed after the focus left its field into the field', async () => {
    const base = createInMemoryBridge({ learnedRules: [{ ...learned[1], confirmed: 0 }] });
    const houseRuleCheck = vi.fn(base.houseRuleCheck);
    const onSave = vi.fn(base.saveSettings);
    render(<App bridge={{ ...base, houseRuleCheck, saveSettings: onSave }} />);
    const dialog = await openSettings();
    const list = await within(dialog).findByRole('list', { name: 'Learned spellings' });
    const names = within(dialog).getByRole('textbox', { name: "Your organizations" });
    fireEvent.click(within(list).getByRole('button', { name: /^Always use:/ }));
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
  // the spelling then went. Once the question had been answered or taken
  // back, Ctrl+Z on its row or on Save - to take back the check - undid the
  // unsaved edit instead, and Save stored it. Before, while and after it
  // asks, an undo changes only the field with the focus.
  it('changes no setting but the one with the focus, before, while and after it asks: no undo or redo reaches an unsaved edit', async () => {
    const base = createInMemoryBridge({ learnedRules: [{ ...learned[1], confirmed: 0 }] });
    const houseRuleCheck = vi.fn(base.houseRuleCheck);
    const saveSettings = vi.fn(base.saveSettings);
    render(<App bridge={{ ...base, houseRuleCheck, saveSettings }} />);
    const dialog = await openSettings();
    const list = await within(dialog).findByRole('list', { name: 'Learned spellings' });
    const machine = within(dialog).getByRole('textbox', { name: "This machine's name" });
    const names = within(dialog).getByRole('textbox', { name: "Your organizations" });
    fireEvent.change(machine, { target: { value: 'Front desk' } });
    /** Whether the browser may make the undo or redo it is about to in `field`: an event nobody refused. */
    const undoes = (field: HTMLElement, inputType: string) => fireEvent(field, new InputEvent('beforeinput', { inputType, bubbles: true, cancelable: true }));
    const action = within(list).getByRole('button', { name: /^Always use:/ });
    action.focus();
    expect(undoes(machine, 'historyUndo')).toBe(false);
    // In the field with the focus, the browser's undo is the person's.
    machine.focus();
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
    // Asking no more, an undo on Save or on the row changes no setting,
    // and in this machine's name it is the browser's.
    for (const focus of [() => within(dialog).getByRole('button', { name: 'Save settings' }).focus(), () => within(list).getByRole('listitem').focus()]) {
      focus();
      for (const [other, inputType] of [[machine, 'historyUndo'], [machine, 'historyRedo'], [names, 'historyUndo']] as const) expect(undoes(other, inputType), inputType).toBe(false);
    }
    machine.focus();
    expect(undoes(machine, 'historyUndo')).toBe(true);
  });

  it('changes no setting but the one with the focus once the question is taken back', async () => {
    render(<App bridge={createInMemoryBridge({ learnedRules: [{ ...learned[1], confirmed: 0 }] })} />);
    const dialog = await openSettings();
    const list = await within(dialog).findByRole('list', { name: 'Learned spellings' });
    const machine = within(dialog).getByRole('textbox', { name: "This machine's name" });
    fireEvent.change(machine, { target: { value: 'Front desk' } });
    const undoes = (field: HTMLElement, inputType: string) => fireEvent(field, new InputEvent('beforeinput', { inputType, bubbles: true, cancelable: true }));
    fireEvent.click(within(list).getByRole('button', { name: /^Always use:/ }));
    const field = within(list).getByRole('textbox');
    typeInto(field, 'Meet');
    fireEvent.keyDown(field, { key: 'Escape' });
    expect(within(list).queryByRole('group')).not.toBeInTheDocument();
    for (const focus of [() => within(list).getByRole('listitem').focus(), () => within(list).getByRole('button', { name: /^Always use:/ }).focus(), () => (document.activeElement as HTMLElement).blur()]) {
      focus();
      for (const inputType of ['historyUndo', 'historyRedo']) expect(undoes(machine, inputType), inputType).toBe(false);
    }
    machine.focus();
    expect(undoes(machine, 'historyUndo')).toBe(true);
    expect(machine).toHaveValue('Front desk');
  });

  // Closing the section over an open question takes the question back: it
  // is not left asking where it cannot be seen.
  it('takes back the question when its section is closed', async () => {
    render(<App bridge={createInMemoryBridge({ learnedRules: [{ ...learned[1], confirmed: 0 }] })} />);
    const dialog = await openSettings();
    const list = await within(dialog).findByRole('list', { name: 'Learned spellings' });
    fireEvent.click(within(list).getByRole('button', { name: /^Always use:/ }));
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
    fireEvent.click(within(list).getByRole('button', { name: /Always use/ }));
    expect(within(list).getByRole('group')).toHaveAccessibleDescription(/^From now on ATLAS will call Quarterly Operations Review documents Meeting Minutes, and may file the name without asking you\./);
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
    fireEvent.click(within(list).getByRole('button', { name: /Always use/ }));
    typeInto(within(list).getByRole('textbox'), 'Meeting Minutes');
    fireEvent.click(within(list).getByRole('button', { name: 'Always use' }));
    expect(await within(dialog).findByRole('alert')).toHaveTextContent('That spelling changed since the list was shown');
    expect(houseRuleCheck).toHaveBeenCalledWith('2', { kind: 'document_type', from: 'Quarterly Operations Review', to: 'Meeting Minutes' });
    await waitFor(() => expect(within(list).getAllByRole('listitem')[0]).toHaveTextContent('Quarterly Operations Review written as Board Minutes'));
  });

  // A spelling forgotten since the list was shown - by putting a document's
  // own words back in review - stayed in the list as in use, its "Use on
  // its own" refused again however often it was tried.
  it('takes a spelling forgotten since the list was shown out of the list when using it on its own finds it gone', async () => {
    const unchecked: LearnedRule = { id: '3', kind: 'party', from: 'Northwind Traders', to: 'Northwind', seen: 2, active: true, confirmed: 0, learnedAt: 1716369300 };
    const base = createInMemoryBridge({ learnedRules: [learned[0], unchecked, learned[1]] });
    const houseRuleCheck = vi.fn(base.houseRuleCheck);
    render(<App bridge={{ ...base, houseRuleCheck }} />);
    const dialog = await openSettings();
    const list = await within(dialog).findByRole('list', { name: 'Learned spellings' });
    const rows = () => within(list).getAllByRole('listitem');
    await base.houseRuleForget('3');
    fireEvent.click(within(rows()[1]).getByRole('button', { name: 'Always use: Northwind Traders written as Northwind' }));
    typeInto(within(rows()[1]).getByRole('textbox'), 'Northwind');
    fireEvent.click(within(rows()[1]).getByRole('button', { name: 'Always use' }));
    expect(await within(dialog).findByRole('alert')).toHaveTextContent('That spelling is no longer in the list');
    expect(houseRuleCheck).toHaveBeenCalledWith('3', { kind: 'party', from: 'Northwind Traders', to: 'Northwind' });
    await waitFor(() => expect(rows()).toHaveLength(2));
    expect(list).not.toHaveTextContent('Northwind');
    // The focus goes to the row that took its place, as when it is forgotten here.
    await waitFor(() => expect(rows()[1]).toHaveFocus());
    expect(rows()[1]).toHaveTextContent('Quarterly Operations Review written as Meeting Minutes');
  });

  // An Enter that did not answer said nothing, and a screen reader heard
  // silence: Tab from the field passes the yes, which is disabled.
  it('says why when Enter does not answer, until the field changes', async () => {
    const base = createInMemoryBridge({ learnedRules: [{ ...learned[1], confirmed: 0 }] });
    const houseRuleCheck = vi.fn(base.houseRuleCheck);
    render(<App bridge={{ ...base, houseRuleCheck }} />);
    const dialog = await openSettings();
    const list = await within(dialog).findByRole('list', { name: 'Learned spellings' });
    fireEvent.click(within(list).getByRole('button', { name: /^Always use:/ }));
    const question = within(list).getByRole('group');
    const field = within(question).getByRole('textbox');
    const status = within(question).getByRole('status');
    expect(status).toHaveTextContent(/^$/);
    fireEvent.keyDown(field, { key: 'Enter' });
    expect(status).toHaveTextContent(/^$/);
    typeInto(field, 'Quarterly Operations Review');
    expect(status).toHaveTextContent(/^$/);
    fireEvent.keyDown(field, { key: 'Enter' });
    expect(status).toHaveTextContent('That is not Meeting Minutes: type it as it is shown.');
    clearField(field);
    expect(status).toHaveTextContent(/^$/);
    typeInto(field, 'Meeting Minute');
    fireEvent.keyDown(field, { key: 'Enter' });
    expect(status).toHaveTextContent('That is not Meeting Minutes: type it as it is shown.');
    typeInto(field, 's');
    expect(status).toHaveTextContent(/^$/);
    expect(houseRuleCheck).not.toHaveBeenCalled();
    fireEvent.keyDown(field, { key: 'Enter' });
    await waitFor(() => expect(houseRuleCheck).toHaveBeenCalledOnce());
    expect(dialog).toBeInTheDocument();
  });

  // A key types one character. A word put in at once, with no keys for
  // it - an input method's clipboard, an on-screen keyboard's, an emoji
  // keyboard's - was taken as typed, and one press checked the spelling.
  // An input method that commits what its keys spelled, without showing a
  // composition, puts in no more than the keys pressed for it.
  it('takes no more characters at once than keys were pressed for them', async () => {
    const rules: LearnedRule[] = [
      { id: '1', kind: 'party', from: 'Contoso Worldwide, Inc.', to: 'Contoso', seen: 2, active: true, confirmed: 0, learnedAt: 1 },
      { id: '9', kind: 'party', from: 'Yamada Taro', to: '山田太郎', seen: 2, active: true, confirmed: 0, learnedAt: 9 },
    ];
    const base = createInMemoryBridge({ learnedRules: rules });
    const houseRuleCheck = vi.fn(base.houseRuleCheck);
    render(<App bridge={{ ...base, houseRuleCheck }} />);
    const dialog = await openSettings();
    const list = await within(dialog).findByRole('list', { name: 'Learned spellings' });
    const asked = () => {
      const question = within(list).getByRole('group');
      return { field: within(question).getByRole('textbox'), yes: within(question).getByRole('button', { name: 'Always use' }), cancel: within(question).getByRole('button', { name: 'Cancel' }) };
    };
    fireEvent.click(within(list).getByRole('button', { name: 'Always use: Contoso Worldwide, Inc. written as Contoso' }));
    const { field, yes, cancel } = asked();
    for (const keys of [0, 1, 2, 6]) {
      insertAtOnce(field, 'Contoso', keys);
      expect(field, `${keys} keys`).toHaveValue('');
      expect(yes, `${keys} keys`).toBeDisabled();
    }
    // Keys that type nothing earn nothing: an arrow, Enter, Home, a
    // function key, Backspace, a modifier, a key held with Ctrl or Cmd.
    const typeNothing = [
      { key: 'ArrowLeft' }, { key: 'ArrowLeft', repeat: true }, { key: 'Enter' }, { key: 'Home' }, { key: 'F5' },
      { key: 'Backspace' }, { key: 'Shift' }, { key: 'a', ctrlKey: true }, { key: 'v', metaKey: true },
    ];
    for (const key of typeNothing) {
      for (let pressed = 0; pressed < 7; pressed += 1) fireEvent.keyDown(field, key);
      insertAtOnce(field, 'Contoso', 0);
      expect(field, JSON.stringify(key)).toHaveValue('');
    }
    // A character key does, and so do an input method's keys, a dead key, Alt (Option) and AltGr (Ctrl+Alt).
    const typeSomething = [
      { key: 'c' }, { key: ' ' }, { key: 'Unidentified', keyCode: 229 }, { key: 'Dead' }, { key: 'å', altKey: true }, { key: 'ł', ctrlKey: true, altKey: true },
    ];
    for (const key of typeSomething) {
      for (let pressed = 0; pressed < 7; pressed += 1) fireEvent.keyDown(field, key);
      insertAtOnce(field, 'Contoso', 0);
      expect(field, JSON.stringify(key)).toHaveValue('Contoso');
      clearField(field);
    }
    // Keys pressed before a character went in are spent on it.
    for (let key = 0; key < 7; key += 1) fireEvent.keyDown(field, { key: 'Process', keyCode: 229 });
    typeInto(field, 'C');
    insertAtOnce(field, 'ontoso', 0);
    expect(field).toHaveValue('C');
    clearField(field);
    insertAtOnce(field, 'Contoso', 7);
    expect(field).toHaveValue('Contoso');
    expect(yes).toBeEnabled();
    fireEvent.click(cancel);

    fireEvent.click(within(list).getByRole('button', { name: 'Always use: Yamada Taro written as 山田太郎' }));
    // "yamadatarou" and the key that commits it.
    insertAtOnce(asked().field, '山田太郎', 12);
    expect(asked().field).toHaveValue('山田太郎');
    fireEvent.click(asked().yes);
    await waitFor(() => expect(houseRuleCheck).toHaveBeenCalledWith('9', { kind: 'party', from: 'Yamada Taro', to: '山田太郎' }));
    expect(houseRuleCheck).toHaveBeenCalledOnce();
  });

  // Opened by a press where it reaches past what can be seen, the question
  // stayed there while the spelling was typed into a field out of sight,
  // "Save settings" under it. A key is no aim of the pointer: the first one
  // brings it into sight.
  it('is brought into sight by a key typed in it after a press opened it', async () => {
    const scrollIntoView = vi.fn();
    Object.defineProperty(Element.prototype, 'scrollIntoView', { value: scrollIntoView, configurable: true, writable: true });
    try {
      render(<App bridge={createInMemoryBridge({ learnedRules: [{ ...learned[1], confirmed: 0 }] })} />);
      const dialog = await openSettings();
      const list = await within(dialog).findByRole('list', { name: 'Learned spellings' });
      const action = within(list).getByRole('button', { name: /^Always use:/ });
      fireEvent.pointerDown(action, { pointerType: 'mouse' });
      fireEvent.click(action, { detail: 1 });
      const question = within(list).getByRole('group');
      expect(scrollIntoView).not.toHaveBeenCalled();
      typeInto(within(question).getByRole('textbox'), 'M');
      expect(scrollIntoView.mock.contexts).toEqual([question]);
      expect(scrollIntoView).toHaveBeenLastCalledWith({ block: 'nearest' });
    } finally {
      Reflect.deleteProperty(Element.prototype, 'scrollIntoView');
    }
  });

  // Settings opens over the review window, whose shortcuts are off while it
  // is open. Keys typed in the question's field stay in it besides: none
  // approves, keeps or moves on the document behind the dialog, and none
  // saves or closes Settings.
  it('keeps every key typed in its field to itself, with a held document open behind Settings', async () => {
    const base = createLearnedSpellingBridge();
    const bridge = { ...base, approve: vi.fn(base.approve), keepOriginal: vi.fn(base.keepOriginal), remove: vi.fn(base.remove), houseRuleCheck: vi.fn(base.houseRuleCheck), saveSettings: vi.fn(base.saveSettings) };
    render(<App bridge={bridge} />);
    const panel = await openRow(await screen.findByRole('row', { name: /SOW final v3.pdf/i }));
    const dialog = await openSettings();
    const list = await within(dialog).findByRole('list', { name: 'Learned spellings' });
    fireEvent.click(within(list).getByRole('button', { name: 'Always use: Contoso Worldwide, Inc. written as Contoso' }));
    const field = within(list).getByRole('textbox');
    for (const typed of ['Cont', 'oso']) {
      typeInto(field, typed);
      for (const key of [
        { key: 'Enter', ctrlKey: true }, { key: 'Enter', metaKey: true }, { key: 'k', code: 'KeyK', altKey: true },
        { key: 'n', code: 'KeyN', altKey: true }, { key: 'F2' }, { key: 'Delete' }, { key: '/' }, { key: 'j' },
      ]) fireEvent.keyDown(field, key);
    }
    expect(field).toHaveFocus();
    expect(field).toHaveValue('Contoso');
    expect(panel).toHaveTextContent('SOW final v3.pdf');
    expect(dialog).toBeInTheDocument();
    expect(bridge.approve).not.toHaveBeenCalled();
    expect(bridge.keepOriginal).not.toHaveBeenCalled();
    expect(bridge.remove).not.toHaveBeenCalled();
    expect(bridge.houseRuleCheck).not.toHaveBeenCalled();
    expect(bridge.saveSettings).not.toHaveBeenCalled();
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

// What the question takes as the spelling typed out, and how it keeps the
// keys while it asks, wherever it is put.
describe('the question "Always use" asks', () => {
  it('is the spelling\'s letters and digits in order, whatever the case, accents, spaces and punctuation', () => {
    expect(typedOut('contoso ltd', 'Contoso Ltd.')).toBe(true);
    expect(typedOut('CONTOSO-LTD', 'Contoso Ltd.')).toBe(true);
    expect(typedOut('Muller', 'Müller')).toBe(true);
    expect(typedOut('Ｃｏｎｔｏｓｏ', 'Contoso')).toBe(true);
    expect(typedOut('山田太郎', '山田太郎')).toBe(true);
    expect(typedOut('3M', '3M Company')).toBe(false);
    expect(typedOut('Mller', 'Müller')).toBe(false);
    expect(typedOut('Mueller', 'Müller')).toBe(false);
    expect(typedOut('Contoso Ltd. 2', 'Contoso Ltd.')).toBe(false);
    expect(typedOut('Ltd. Contoso', 'Contoso Ltd.')).toBe(false);
    expect(typedOut('山田', '山田太郎')).toBe(false);
    expect(typedOut('', 'Contoso')).toBe(false);
    expect(typedOut(' .', 'Contoso')).toBe(false);
    // A spelling with no letter or digit is typed exactly, spaces at its ends aside.
    expect(typedOut(' & ', '&')).toBe(true);
    expect(typedOut('+', '&')).toBe(false);
    expect(typedOut('', ' ')).toBe(false);
    expect(typedOut(' ', ' ')).toBe(false);
  });

  // An accent may be left off, as a keyboard without it does. A mark that
  // makes another letter or syllable may not: typing the document's own
  // カトウ confirmed ガトウ, and राम confirmed रमा.
  it('lets go of accents, and of no mark that makes another letter', () => {
    expect(typedOut('Nguyen', 'Nguyễn')).toBe(true);
    expect(typedOut('Dvorak', 'Dvořák')).toBe(true);
    expect(typedOut('ELENI', 'ΕΛΕΝΗ')).toBe(false);
    expect(typedOut('Ελενη', 'Ελένη')).toBe(true);
    expect(typedOut('ΕΛΕΝΗΣ', 'Ελένης')).toBe(true);
    expect(typedOut('Ελένησ', 'Ελένης')).toBe(true);
    // Hebrew and Arabic are written without their vowel points as a rule.
    expect(typedOut('שלום', 'שָׁלוֹם')).toBe(true);
    expect(typedOut('محمد', 'مُحَمَّد')).toBe(true);
    // Elsewhere a mark is a letter's own.
    expect(typedOut('ガトウ', 'ガトウ')).toBe(true);
    expect(typedOut('ｶﾞﾄｳ', 'ガトウ')).toBe(true);
    expect(typedOut('カトウ', 'ガトウ')).toBe(false);
    expect(typedOut('さとう', 'さどう')).toBe(false);
    expect(typedOut('ホンタ', 'ホンダ')).toBe(false);
    expect(typedOut('ポンダ', 'ホンダ')).toBe(false);
    expect(typedOut('राम', 'रमा')).toBe(false);
    expect(typedOut('कमल', 'कमला')).toBe(false);
    expect(typedOut('करन', 'किरन')).toBe(false);
    expect(typedOut('कषितिज', 'क्षितिज')).toBe(false);
    expect(typedOut('क्षितिज', 'क्षितिज')).toBe(true);
    expect(typedOut('Иошкар', 'Йошкар')).toBe(false);
    expect(typedOut('Ежов', 'Ёжов')).toBe(false);
    expect(typedOut('Йошкар', 'Йошкар')).toBe(true);
    expect(typedOut('สดา', 'สุดา')).toBe(false);
    expect(typedOut('கம', 'காம')).toBe(false);
  });

  // A keyboard without ø, ß or ł types o, ss and l, and the spelling's
  // symbols are none of its letters: "Acme" is "Acme™" as it is "Acme®".
  it('takes a letter a keyboard may lack as it is typed without it, and leaves out what is no letter as written', () => {
    for (const [typed, spelling] of [
      ['Moller & Son A/S', 'Møller & Søn A/S'], ['Soren', 'Søren'], ['Strasse', 'Straße'], ['STRASSE', 'Straße'], ['strasse', 'STRAẞE'],
      ['Lodz', 'Łódź'], ['Dorde Dordevic', 'Đorđe Đorđević'], ['Caesar', 'Cæsar'], ['Aero', 'Ærø'], ['Oeuvre', 'Œuvre'],
      ['Kirikkale', 'Kırıkkale'], ['KIRSEHIR', 'Kırşehir'], ['Thor', 'Þór'], ['Gudmundsdottir', 'Guðmundsdóttir'], ['Hamrun', 'Ħamrun'],
      ['Acme', 'Acme™'], ['Acme', 'Acme®'], ['Acme', 'Acme℠'], ['finance', 'ﬁnance'], ['Hawaii', 'Hawaiʻi'], ['Abd al-Aziz', 'ʿAbd al-ʿAzīz'],
      ['葛飾', '葛\u{E0100}飾'],
    ]) expect(typedOut(typed, spelling), `${typed} for ${spelling}`).toBe(true);
    for (const [typed, spelling] of [
      ['Moler', 'Møller'], ['Mpller', 'Møller'], ['Stase', 'Straße'], ['Strase', 'Straße'], ['Acmetm', 'Acme™'], ['Acm', 'Acme™'],
      ['Contoso Ltd', 'Contoso'], ['Contos', 'Contoso'], ['Thr', 'Þór'],
    ]) expect(typedOut(typed, spelling), `${typed} for ${spelling}`).toBe(false);
    // A spelling that is all symbols is typed as it is.
    expect(typedOut('™', '™')).toBe(true);
    expect(typedOut('tm', '™')).toBe(false);
    expect(typedOut('', '™')).toBe(false);
  });

  it('starts empty again when the spelling it asks about changes while it asks', () => {
    const onConfirm = vi.fn();
    type Shown = Pick<LearnedRule, 'id' | 'kind' | 'from' | 'to'>;
    const rule: Shown = { id: '1', kind: 'party', from: 'Contoso Worldwide, Inc.', to: 'Contoso' };
    const ask = (shown: Shown) => <UseOnItsOwnQuestion rule={shown} describedBy="asked" onConfirm={onConfirm} onCancel={() => undefined} />;
    const { rerender } = render(ask(rule));
    typeInto(screen.getByRole('textbox'), 'Contoso');
    expect(screen.getByRole('button', { name: 'Always use' })).toBeEnabled();
    const changes: Shown[] = [{ ...rule, id: '5' }, { ...rule, kind: 'document_type' }, { ...rule, from: 'Contoso Worldwide' }, { ...rule, to: 'Contoso Ltd.' }];
    for (const changed of changes) {
      rerender(ask(changed));
      expect(screen.getByRole('textbox'), JSON.stringify(changed)).toHaveValue('');
      expect(screen.getByRole('textbox')).toHaveFocus();
      expect(screen.getByRole('button', { name: 'Always use' })).toBeDisabled();
      typeInto(screen.getByRole('textbox'), changed.to);
      expect(screen.getByRole('button', { name: 'Always use' })).toBeEnabled();
    }
    expect(onConfirm).not.toHaveBeenCalled();
  });

  it('takes itself back on Escape from its field, and lets Tab through, but not the Escape an input method takes', () => {
    const onCancel = vi.fn();
    render(<UseOnItsOwnQuestion rule={{ id: '1', kind: 'party', from: 'Contoso Worldwide, Inc.', to: 'Contoso' }} describedBy="asked" onConfirm={vi.fn()} onCancel={onCancel} />);
    const field = screen.getByRole('textbox');
    const beyond = vi.fn();
    document.addEventListener('keydown', beyond);
    try {
      fireEvent.keyDown(field, { key: 'Escape', isComposing: true });
      expect(onCancel).not.toHaveBeenCalled();
      fireEvent.keyDown(field, { key: 'Escape' });
      expect(onCancel).toHaveBeenCalledOnce();
      expect(beyond).not.toHaveBeenCalled();
      // Tab goes on to the dialog's own keys, which keep the focus in it.
      fireEvent.keyDown(field, { key: 'Tab' });
      expect(beyond).toHaveBeenCalledOnce();
    } finally {
      document.removeEventListener('keydown', beyond);
    }
  });

  // A press on its sentence or its waiting yes takes the focus off its
  // field. What is typed next is its own: a character typed in the dialog
  // it is in, or on nothing, goes into its field - but not one typed in
  // another field, nor one typed outside its dialog, and nothing once it
  // has gone.
  it('takes a character typed in its dialog or on nothing into its field, and no other key, nor one typed in another field or outside its dialog', () => {
    const scrollIntoView = vi.fn();
    Object.defineProperty(Element.prototype, 'scrollIntoView', { value: scrollIntoView, configurable: true, writable: true });
    const beyond = vi.fn();
    document.addEventListener('keydown', beyond);
    try {
      const view = render(<div>
        <button type="button">Outside</button>
        <section role="dialog" aria-modal="true" aria-label="Settings">
          <input aria-label="Another field" />
          <button type="button">Save</button>
          <UseOnItsOwnQuestion rule={{ id: '1', kind: 'party', from: 'Contoso Worldwide, Inc.', to: 'Contoso' }} describedBy="asked" onConfirm={vi.fn()} onCancel={vi.fn()} />
        </section>
      </div>);
      const group = screen.getByRole('group');
      const field = screen.getByRole('textbox', { name: 'Type Contoso to always use it' });
      const cancel = screen.getByRole('button', { name: 'Cancel' });
      const save = screen.getByRole('button', { name: 'Save' });
      const outside = screen.getByRole('button', { name: 'Outside' });
      const other = screen.getByRole('textbox', { name: 'Another field' });
      /** `init` pressed at `target`: whether it went on to the page's own listeners, and where the focus is after it. */
      const pressAt = (target: HTMLElement, init: KeyboardEventInit) => {
        target.focus();
        if (target === document.body) (document.activeElement as HTMLElement).blur();
        beyond.mockClear();
        scrollIntoView.mockClear();
        fireEvent.keyDown(target, init);
        return { reached: beyond.mock.calls.length === 1, focus: document.activeElement };
      };
      // A character on nothing, on its Cancel, or on another control of its dialog: into its field, brought into sight, and to nothing else.
      for (const target of [document.body, cancel, save]) {
        expect(pressAt(target, { key: 'k' }), target.tagName).toEqual({ reached: false, focus: field });
        expect(scrollIntoView.mock.contexts).toEqual([group]);
      }
      // Not the space bar, which presses Cancel; nor a key held with Ctrl, nor an input method's, nor a named key.
      expect(pressAt(cancel, { key: ' ' })).toEqual({ reached: true, focus: cancel });
      for (const init of [{ key: 'k', ctrlKey: true }, { key: 'k', isComposing: true }, { key: 'Process', keyCode: 229 }, { key: 'Dead' }, { key: 'Enter' }, { key: 'ArrowDown' }]) {
        expect(pressAt(document.body, init), JSON.stringify(init)).toEqual({ reached: true, focus: document.body });
      }
      // Not a key typed in another field: that field's. Nor one typed outside its dialog.
      expect(pressAt(other, { key: 'k' })).toEqual({ reached: true, focus: other });
      expect(pressAt(outside, { key: 'k' })).toEqual({ reached: true, focus: outside });
      // Gone, it leaves nothing behind.
      view.unmount();
      view.container.remove();
      expect(pressAt(document.body, { key: 'k' })).toEqual({ reached: true, focus: document.body });
    } finally {
      document.removeEventListener('keydown', beyond);
      Reflect.deleteProperty(Element.prototype, 'scrollIntoView');
    }
  });
});
