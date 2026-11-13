import { cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { App } from '../../App';
import { ReviewInspector } from '../../components/ReviewInspector';
import { openRow } from '../queue/gridTesting';
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
