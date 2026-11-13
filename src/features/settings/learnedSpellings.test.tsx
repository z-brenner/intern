import { act, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { App } from '../../App';
import { createInMemoryBridge } from '../../lib/inMemoryBridge';
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
