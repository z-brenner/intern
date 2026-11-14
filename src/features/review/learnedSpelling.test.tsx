import { fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { App } from '../../App';
import { ReviewInspector } from '../../components/ReviewInspector';
import { openRow, queueCell } from '../queue/gridTesting';
import { createInMemoryBridge, createLearnedSpellingBridge } from '../../lib/inMemoryBridge';
import { humanizeReason } from '../../lib/reasons';
import type { QueueItem } from '../../types';

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
    expect(note).toHaveTextContent('Uses your spelling: Contoso Worldwide, Inc. written as Contoso. Forget it in Settings, under Learned spellings.');
    // Settings has no way to change a spelling, so the note does not send anyone there for one.
    expect(note.textContent).not.toMatch(/change/i);
    expect(screen.getByLabelText('Filename')).toHaveValue('2026-04-01 Statement of Work between Ridgeline Cartography LLC and Contoso');
    fireEvent.click(screen.getByText('Analysis details'));
    const evidence = screen.getByRole('heading', { name: 'Evidence' }).closest('section')!;
    expect(within(evidence).getByText('Contoso Worldwide, Inc.')).toBeVisible();
  });

  // A spelling ending in its own period ("Inc.") is the end of the list:
  // the sentence takes no second one. It read "Receiver, Inc.. Change or
  // forget it under Settings." - and Settings has no way to change one.
  it('ends the note with one period after a spelling that ends in one', async () => {
    const abbreviated: QueueItem = { ...respelled, houseRules: [{ kind: 'party', from: 'Contoso Worldwide, Inc.', to: 'Contoso, Inc.' }] };
    render(<App bridge={createInMemoryBridge({ items: [abbreviated] })} />);
    await selectRow(await screen.findByRole('row', { name: /SOW final v3.pdf/i }));

    const note = screen.getByRole('note', { name: 'Learned spellings applied' });
    expect(note).toHaveTextContent('Uses your spelling: Contoso Worldwide, Inc. written as Contoso, Inc. Forget it in Settings, under Learned spellings.');
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

/** The held statement of work of `?learnedSpellings=held`, first in the demo's queue, with what the bridge is asked watched. */
function heldBridge() {
  const base = createLearnedSpellingBridge();
  return {
    ...base,
    approve: vi.fn(base.approve), keepOriginal: vi.fn(base.keepOriginal), remove: vi.fn(base.remove),
    houseRulesList: vi.fn(base.houseRulesList), houseRuleCheck: vi.fn(base.houseRuleCheck),
  };
}

const HELD_NAME = '2026-04-01 SOW between Ridgeline Cartography and Contoso';

/**
 * `text` typed a key at a time wherever the focus is, as a person types it:
 * the key, then - into the field that has the focus once the key is down -
 * the input it makes, unless the key or the input was refused.
 */
function typeAtTheFocus(text: string) {
  for (const data of text) {
    if (!fireEvent.keyDown(document.activeElement ?? document.body, { key: data })) continue;
    const field = document.activeElement;
    if (!(field instanceof HTMLInputElement || field instanceof HTMLTextAreaElement)) continue;
    if (!fireEvent(field, new InputEvent('beforeinput', { inputType: 'insertText', data, bubbles: true, cancelable: true }))) continue;
    fireEvent.input(field, { target: { value: field.value + data }, inputType: 'insertText', data });
  }
}

/*
  No approval checks a learned spelling, and the review window checks none
  either. "Use on its own" is asked in Settings only: asked here, the
  spelling typed into it reached review's own keys, which move on, put the
  caret in a document's name and file it. A document held for a spelling
  nobody has checked says why it waits and where "Use on its own" is, and
  is reviewed like any other.
*/
describe('a document held for a spelling nobody has checked', () => {
  it('asks nothing and offers no field for a spelling, and says why it waits and where "Use on its own" is', () => {
    render(<ReviewInspector item={held} drawer={false} {...handlers()} />);
    expect(screen.queryByRole('button', { name: /Use on its own/ })).not.toBeInTheDocument();
    expect(screen.queryByRole('group', { name: /Use on its own/ })).not.toBeInTheDocument();
    // The name and the description, and no field to type a spelling into.
    expect(screen.getAllByRole('textbox')).toEqual([screen.getByRole('textbox', { name: 'Filename' }), screen.getByRole('textbox', { name: 'Description' })]);
    const why = screen.getByRole('note', { name: 'Why the name waits' });
    expect(why).toHaveTextContent('This name waits for you because Intern has not been told to use every one of these spellings on its own: Use on its own is in Settings, under Learned spellings.');
    // It does not say that approving the name would do instead.
    expect(why.textContent).not.toMatch(/approv/i);
    const applied = screen.getByRole('note', { name: 'Learned spellings applied' });
    expect(applied).toHaveTextContent('Forget any of them in Settings, under Learned spellings.');
    expect(applied.textContent).not.toMatch(/change/i);
  });

  it('says "this spelling" under a note of one', () => {
    render(<ReviewInspector item={{ ...held, houseRules: [held.houseRules![0]] }} drawer={false} {...handlers()} />);
    expect(screen.getByRole('note', { name: 'Why the name waits' })).toHaveTextContent('This name waits for you because Intern has not been told to use this spelling on its own: Use on its own is in Settings, under Learned spellings.');
  });

  it('says nothing of it for a name in your spelling that is not held for it', () => {
    const { rerender } = render(<ReviewInspector item={{ ...held, status: 'ready', errorCode: undefined, reasonCodes: undefined, reason: undefined }} drawer={false} {...handlers()} />);
    expect(screen.queryByRole('note', { name: 'Why the name waits' })).not.toBeInTheDocument();
    rerender(<ReviewInspector item={{ ...held, errorCode: 'LOW_CONFIDENCE', reasonCodes: ['LOW_CONFIDENCE'], reason: humanizeReason('LOW_CONFIDENCE') }} drawer={false} {...handlers()} />);
    expect(screen.queryByRole('note', { name: 'Why the name waits' })).not.toBeInTheDocument();
    expect(screen.getByRole('note', { name: 'Learned spellings applied' })).toBeInTheDocument();
  });

  // Whatever is pressed in the panel, and whatever is typed - the spellings
  // themselves, at the panel, in its fields, and with Enter - no spelling is
  // read or checked from the review window, and approving the name files it.
  it('reads and checks no spelling from the review window, whatever is pressed or typed there', async () => {
    const bridge = heldBridge();
    render(<App bridge={bridge} />);
    const panel = await selectRow(await screen.findByRole('row', { name: /SOW final v3.pdf/i }));
    const name = within(panel).getByRole('textbox', { name: 'Filename' });
    const pressable = within(panel).getAllByRole('button')
      .filter((button) => !button.closest('.inspector-actions') && !['Close review', 'Next undecided'].includes(button.getAttribute('aria-label') ?? button.textContent ?? ''));
    for (const button of pressable) {
      fireEvent.pointerDown(button, { pointerType: 'mouse' });
      fireEvent.click(button, { detail: 1 });
    }
    fireEvent.click(within(panel).getByText('Analysis details'));
    // The spellings typed, and Enter, on the panel's heading and on nothing.
    for (const focusOn of [() => within(panel).getByRole('heading', { name: 'Review item' }).focus(), () => (document.activeElement as HTMLElement).blur()]) {
      for (const spelling of ['Contoso', 'Ridgeline Cartography']) {
        focusOn();
        typeAtTheFocus(spelling);
        fireEvent.keyDown(document.activeElement ?? document.body, { key: 'Enter' });
      }
    }
    expect(panel).toHaveTextContent('SOW final v3.pdf');
    expect(name).toHaveValue(HELD_NAME);
    // In its fields, and the name filed with Enter: an approval, which checks nothing.
    within(panel).getByRole('textbox', { name: 'Description' }).focus();
    typeAtTheFocus(' Contoso');
    name.focus();
    typeAtTheFocus(' Contoso');
    fireEvent.keyDown(name, { key: 'Enter' });
    await waitFor(() => expect(bridge.approve).toHaveBeenCalledWith('sow', `${HELD_NAME} Contoso.pdf`, `${held.description} Contoso`));
    expect(bridge.approve).toHaveBeenCalledOnce();
    expect(bridge.houseRuleCheck).not.toHaveBeenCalled();
    expect(bridge.houseRulesList).not.toHaveBeenCalled();
  });

  describe('is reviewed from the keyboard as any other document is', () => {
    const press = (key: string, init: Partial<KeyboardEventInit> = {}) => fireEvent.keyDown(document.activeElement ?? document.body, { key, ...init });
    const sow = () => queueCell('SOW final v3.pdf');
    async function start() {
      const bridge = heldBridge();
      render(<App bridge={bridge} />);
      const panel = await selectRow(await screen.findByRole('row', { name: /SOW final v3.pdf/i }));
      sow().focus();
      return { bridge, panel, name: () => within(panel).getByRole('textbox', { name: 'Filename' }) };
    }

    it('J and K move from it and back, F2 and Enter go to its name, "/" to the filter, Alt+N on, and Delete asks', async () => {
      const { bridge, panel, name } = await start();
      press('j');
      await waitFor(() => expect(panel).not.toHaveTextContent('SOW final v3.pdf'));
      press('k');
      await waitFor(() => expect(sow()).toHaveFocus());
      expect(panel).toHaveTextContent('SOW final v3.pdf');
      press('F2');
      expect(name()).toHaveFocus();
      expect(name()).toHaveValue(HELD_NAME);
      sow().focus();
      press('Enter');
      await waitFor(() => expect(name()).toHaveFocus());
      sow().focus();
      press('/');
      expect(screen.getByRole('searchbox', { name: 'Filter queue' })).toHaveFocus();
      sow().focus();
      press('Delete');
      expect(within(panel).getByRole('group', { name: 'Confirm removal' })).toBeInTheDocument();
      fireEvent.click(within(panel).getByRole('button', { name: 'Cancel' }));
      name().focus();
      press('n', { code: 'KeyN', altKey: true });
      await waitFor(() => expect(panel).not.toHaveTextContent('SOW final v3.pdf'));
      expect(bridge.approve).not.toHaveBeenCalled();
      expect(bridge.keepOriginal).not.toHaveBeenCalled();
      expect(bridge.remove).not.toHaveBeenCalled();
      expect(bridge.houseRuleCheck).not.toHaveBeenCalled();
    });

    it('Ctrl+Enter approves it under the name shown', async () => {
      const { bridge } = await start();
      press('Enter', { ctrlKey: true });
      await waitFor(() => expect(bridge.approve).toHaveBeenCalledWith('sow', `${HELD_NAME}.pdf`, held.description));
      expect(bridge.houseRuleCheck).not.toHaveBeenCalled();
    });

    it('Alt+K keeps its original name', async () => {
      const { bridge } = await start();
      press('k', { code: 'KeyK', altKey: true });
      await waitFor(() => expect(bridge.keepOriginal).toHaveBeenCalledWith('sow'));
      expect(bridge.approve).not.toHaveBeenCalled();
      expect(bridge.houseRuleCheck).not.toHaveBeenCalled();
    });

    it('a name typed into it, and Enter, renames it under that name', async () => {
      const { bridge, name } = await start();
      press('F2');
      fireEvent.change(name(), { target: { value: '2026-04-01 Statement of Work between Ridgeline Cartography LLC and Contoso Worldwide' } });
      fireEvent.keyDown(name(), { key: 'Enter' });
      await waitFor(() => expect(bridge.approve).toHaveBeenCalledWith('sow', '2026-04-01 Statement of Work between Ridgeline Cartography LLC and Contoso Worldwide.pdf', held.description));
      expect(bridge.houseRuleCheck).not.toHaveBeenCalled();
    });
  });

});
