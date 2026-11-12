import { fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { App } from '../../App';
import { openRow } from '../queue/gridTesting';
import { createInMemoryBridge } from '../../lib/inMemoryBridge';
import type { QueueItem } from '../../types';

const selectRow = openRow;

/*
  A request letter the model read as sent to the Department of Justice, and
  the name gives the addressee; the reviewer wants it filed under the law
  firm that sent it, which the document names too. The fix used to be
  retyping the party - fifteen to forty characters, here twenty-seven.
*/
const letter: QueueItem = {
  id: 'letter',
  originalFilename: 'Scan 0201.pdf',
  status: 'review',
  proposedFilename: '2012-06-19 Request Letter to U.S. Department of Justice.pdf',
  confidence: 0.92,
  description: 'Business review request to the U.S. Department of Justice on behalf of STARS Alliance LLC.',
  evidence: { date: 'June 19, 2012', type: 'Request Letter', parties: 'U.S. Department of Justice' },
  reason: 'The model was unsure of its reading. Check the name against the document, starting with the date and the parties.',
  errorCode: 'LOW_CONFIDENCE',
  partyChoices: ['Morgan, Lewis & Bockius LLP', 'STARS Alliance LLC'],
  nameParties: ['U.S. Department of Justice'],
};

const choicesGroup = () => screen.getByRole('group', { name: 'Other parties to choose from' });

describe('other parties the document names', () => {
  it('puts a chosen party in place of the one the name gives, and Enter files it from the name', async () => {
    const baseBridge = createInMemoryBridge({ items: [letter] });
    const approve = vi.fn(baseBridge.approve);
    render(<App bridge={{ ...baseBridge, approve }} />);
    await selectRow(await screen.findByRole('row', { name: /Scan 0201.pdf/i }));
    const filename = screen.getByLabelText('Filename');

    const choices = choicesGroup();
    expect(choices).toHaveTextContent('Other parties the document names');
    const firm = within(choices).getByRole('button', { name: 'Morgan, Lewis & Bockius LLP' });
    // Said for a screen reader as what pressing it does, and none is pressed yet.
    expect(firm).toHaveAccessibleDescription('Puts this party in the filename in place of the one it gives.');
    expect(firm).toHaveAttribute('aria-pressed', 'false');

    firm.focus();
    fireEvent.click(firm);
    expect(filename).toHaveValue('2012-06-19 Request Letter to Morgan, Lewis & Bockius LLP');
    expect(filename).toHaveFocus();
    // The choice stays on screen, now the one the name gives.
    expect(within(choicesGroup()).getByRole('button', { name: 'Morgan, Lewis & Bockius LLP' })).toHaveAttribute('aria-pressed', 'true');

    fireEvent.keyDown(filename, { key: 'Enter' });
    await waitFor(() => expect(approve).toHaveBeenCalledWith('letter', '2012-06-19 Request Letter to Morgan, Lewis & Bockius LLP.pdf', letter.description));
  });

  /*
    A chip used to set the name from code, which empties a browser field's
    undo history: a party fixed by hand, or words typed after it, were gone
    once a chip was clicked, and Ctrl+Z gave nothing back. It goes in
    through the browser's editing command now, as typed text does. jsdom
    has no such command; this one does what Chromium's does, replacing the
    selection and announcing it as input.
  */
  describe('put in as the browser puts typed text', () => {
    const browserInsertText = (command: string, _ui?: boolean, text = '') => {
      const field = document.activeElement;
      if (command !== 'insertText' || !(field instanceof HTMLTextAreaElement)) return false;
      field.setRangeText(text, field.selectionStart, field.selectionEnd, 'end');
      field.dispatchEvent(new InputEvent('input', { bubbles: true, inputType: 'insertText', data: text }));
      return true;
    };
    const insertText = vi.fn(browserInsertText);
    beforeEach(() => {
      insertText.mockClear();
      Object.defineProperty(document, 'execCommand', { configurable: true, value: insertText });
    });
    afterEach(() => { Reflect.deleteProperty(document, 'execCommand'); });

    it('replaces the whole name with the editing command, keeping what was typed after the party', async () => {
      const baseBridge = createInMemoryBridge({ items: [letter] });
      const approve = vi.fn(baseBridge.approve);
      render(<App bridge={{ ...baseBridge, approve }} />);
      await selectRow(await screen.findByRole('row', { name: /Scan 0201.pdf/i }));
      const filename = screen.getByLabelText<HTMLTextAreaElement>('Filename');
      fireEvent.change(filename, { target: { value: '2012-06-19 Request Letter to U.S. Department of Justice re Merger' } });

      let selected: [number | null, number | null, boolean] | undefined;
      insertText.mockImplementationOnce((command, ui, text) => {
        selected = [filename.selectionStart, filename.selectionEnd, document.activeElement === filename];
        return browserInsertText(command, ui, text);
      });
      fireEvent.click(within(choicesGroup()).getByRole('button', { name: 'Morgan, Lewis & Bockius LLP' }));

      expect(insertText).toHaveBeenCalledTimes(1);
      expect(insertText).toHaveBeenCalledWith('insertText', false, '2012-06-19 Request Letter to Morgan, Lewis & Bockius LLP re Merger');
      // The whole name was selected in the field, so the one command replaced all of it.
      expect(selected).toEqual([0, '2012-06-19 Request Letter to U.S. Department of Justice re Merger'.length, true]);
      expect(filename).toHaveValue('2012-06-19 Request Letter to Morgan, Lewis & Bockius LLP re Merger');
      expect(filename).toHaveFocus();
      expect(within(choicesGroup()).getByRole('button', { name: 'Morgan, Lewis & Bockius LLP' })).toHaveAttribute('aria-pressed', 'true');

      fireEvent.keyDown(filename, { key: 'Enter' });
      await waitFor(() => expect(approve).toHaveBeenCalledWith('letter', '2012-06-19 Request Letter to Morgan, Lewis & Bockius LLP re Merger.pdf', letter.description));
    });

    it('sets the name from code when the command is refused, and does nothing for the party already given', async () => {
      render(<App bridge={createInMemoryBridge({ items: [letter] })} />);
      await selectRow(await screen.findByRole('row', { name: /Scan 0201.pdf/i }));
      const filename = screen.getByLabelText('Filename');

      insertText.mockReturnValueOnce(false);
      fireEvent.click(within(choicesGroup()).getByRole('button', { name: 'STARS Alliance LLC' }));
      expect(insertText).toHaveBeenCalledTimes(1);
      expect(filename).toHaveValue('2012-06-19 Request Letter to STARS Alliance LLC');
      expect(filename).toHaveFocus();
      expect(within(choicesGroup()).getByRole('button', { name: 'STARS Alliance LLC' })).toHaveAttribute('aria-pressed', 'true');

      // Choosing it again changes nothing, and leaves no step to undo.
      fireEvent.click(within(choicesGroup()).getByRole('button', { name: 'STARS Alliance LLC' }));
      expect(insertText).toHaveBeenCalledTimes(1);
      expect(filename).toHaveValue('2012-06-19 Request Letter to STARS Alliance LLC');
      expect(filename).toHaveFocus();
    });
  });

  it('replaces an earlier choice, and only the second side of a "between"', async () => {
    const notice: QueueItem = {
      ...letter,
      id: 'notice',
      originalFilename: 'Scan 0202.pdf',
      proposedFilename: '2008-01-24 Notice of Voluntary Dismissal between PA Advisors, LLC and WPP Group PLC.pdf',
      partyChoices: ['Google Inc', 'Yahoo! Inc'],
      nameParties: ['PA Advisors, LLC', 'WPP Group PLC'],
    };
    render(<App bridge={createInMemoryBridge({ items: [notice] })} />);
    await selectRow(await screen.findByRole('row', { name: /Scan 0202.pdf/i }));
    const filename = screen.getByLabelText('Filename');

    fireEvent.click(within(choicesGroup()).getByRole('button', { name: 'Google Inc' }));
    expect(filename).toHaveValue('2008-01-24 Notice of Voluntary Dismissal between PA Advisors, LLC and Google Inc');
    fireEvent.click(within(choicesGroup()).getByRole('button', { name: 'Yahoo! Inc' }));
    expect(filename).toHaveValue('2008-01-24 Notice of Voluntary Dismissal between PA Advisors, LLC and Yahoo! Inc');
    expect(within(choicesGroup()).getByRole('button', { name: 'Google Inc' })).toHaveAttribute('aria-pressed', 'false');
    expect(within(choicesGroup()).getByRole('button', { name: 'Yahoo! Inc' })).toHaveAttribute('aria-pressed', 'true');
  });

  it('keeps the type\'s own words when the party is replaced in a name that has no date yet', async () => {
    const undated: QueueItem = {
      ...letter,
      id: 'application',
      originalFilename: 'Scan 0203.pdf',
      proposedFilename: 'Application for Certification for City of Victorville.pdf',
      partyChoices: ['California Energy Commission'],
      nameParties: ['City of Victorville'],
    };
    render(<App bridge={createInMemoryBridge({ items: [undated] })} />);
    await selectRow(await screen.findByRole('row', { name: /Scan 0203.pdf/i }));

    fireEvent.click(within(choicesGroup()).getByRole('button', { name: 'California Energy Commission' }));
    expect(screen.getByLabelText('Filename')).toHaveValue('Application for Certification for California Energy Commission');
  });

  it('adds a party after the whole type of a name that gave none, and leaves a collision suffix off', async () => {
    const notice: QueueItem = {
      ...letter,
      id: 'notice',
      originalFilename: 'Scan 0205.pdf',
      // The party was left out for length, and the type holds "with".
      proposedFilename: '2022-03-17 Notice in Accordance with Section 57C of the Railways Act 1993 (2).pdf',
      partyChoices: ['Department for Transport'],
      nameParties: undefined,
    };
    render(<App bridge={createInMemoryBridge({ items: [notice] })} />);
    await selectRow(await screen.findByRole('row', { name: /Scan 0205.pdf/i }));

    fireEvent.click(within(choicesGroup()).getByRole('button', { name: 'Department for Transport' }));
    const filename = screen.getByLabelText('Filename');
    expect(filename).toHaveValue('2022-03-17 Notice in Accordance with Section 57C of the Railways Act 1993 - Department for Transport');
    expect(filename).toHaveFocus();
    expect(within(choicesGroup()).getByRole('button', { name: 'Department for Transport' })).toHaveAttribute('aria-pressed', 'true');
  });

  it('shows nothing when the document names no other party', async () => {
    const { partyChoices: _choices, nameParties: _named, ...plain } = letter;
    render(<App bridge={createInMemoryBridge({ items: [plain] })} />);
    await selectRow(await screen.findByRole('row', { name: /Scan 0201.pdf/i }));

    expect(screen.getByLabelText('Filename')).toHaveValue('2012-06-19 Request Letter to U.S. Department of Justice');
    expect(screen.queryByRole('group', { name: 'Other parties to choose from' })).not.toBeInTheDocument();
    expect(screen.queryByText('Other parties the document names')).not.toBeInTheDocument();
  });

  it('offers them only while the name is in review, as the backend sends them', async () => {
    const bridge = createInMemoryBridge({ items: [letter, { ...letter, id: 'ready', originalFilename: 'Scan 0204.pdf', status: 'ready', errorCode: undefined, reason: undefined }] });
    const [inReview, ready] = await bridge.listItems();
    expect(inReview).toMatchObject({ partyChoices: letter.partyChoices, nameParties: letter.nameParties });
    expect(ready).not.toHaveProperty('partyChoices');
    expect(ready).not.toHaveProperty('nameParties');

    await bridge.approve('letter', '2012-06-19 Request Letter to STARS Alliance LLC.pdf', letter.description!);
    const [filed] = await bridge.listItems();
    expect(filed.status).toBe('completed');
    expect(filed).not.toHaveProperty('partyChoices');
  });
});
