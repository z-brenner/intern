import { fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { App } from '../../App';
import { createInMemoryBridge } from '../../lib/inMemoryBridge';
import { describeNotes, humanizeReason } from '../../lib/reasons';
import { TauriBridge } from '../../lib/tauriBridge';
import type { HistoryEntry, QueueItem } from '../../types';
import { openDetails, queueCell } from '../queue/gridTesting';

/*
  At the Balanced review level a reading can be ready while it carries review
  codes, and a document Intern then renames by itself is filed with them. The
  window shows such a row as ready, or completed, with a note: the person can
  see what Intern noted, nothing of it reads as a problem, and nothing that
  is only for a document in review is offered.
*/

const codes = ['TYPE_INFERRED', 'LOW_CONFIDENCE'];
const noted: QueueItem = {
  id: 'noted', originalFilename: 'Scan 0301.pdf', status: 'ready',
  proposedFilename: '2024-03-08 Letter to Fabrikam Inc.pdf', confidence: 1,
  description: 'Letter to Fabrikam Inc. about renewing its service agreement.',
  evidence: { date: 'March 8, 2024', type: 'Dear Ms. Okafor', parties: 'Fabrikam Inc.' },
  reason: humanizeReason(codes.join(', ')), reasonCodes: codes,
};
const plain: QueueItem = { id: 'plain', originalFilename: 'Scan 0302.pdf', status: 'ready', proposedFilename: '2024-03-09 Invoice INV-7 from Litware Inc.pdf', confidence: 0.97, description: 'Invoice INV-7 from Litware Inc.' };
const inReview: QueueItem = { id: 'review', originalFilename: 'Scan 0303.pdf', status: 'review', proposedFilename: '2024-03-10 Notice to Contoso Ltd.pdf', confidence: 0.9, description: 'Notice to Contoso Ltd.', reason: humanizeReason('DATE_AMBIGUOUS'), reasonCodes: ['DATE_AMBIGUOUS'] };
/** As an automatic rename at Balanced leaves one: renamed, its codes still on it. */
const renamed: QueueItem = { ...noted, id: 'renamed', originalFilename: 'Scan 0298.pdf', status: 'completed', filedName: noted.proposedFilename, filedPath: `C:\\Filed\\${noted.proposedFilename}`, undoable: true, keptOriginal: false };
const renamedPlain: QueueItem = { ...plain, id: 'renamed-plain', originalFilename: 'Scan 0299.pdf', status: 'completed', filedName: plain.proposedFilename, undoable: true, keptOriginal: false };

async function start(items: QueueItem[], extra: Parameters<typeof createInMemoryBridge>[0] = {}) {
  const base = createInMemoryBridge({ items, ...extra });
  const bridge = { ...base, approve: vi.fn(base.approve) };
  render(<App bridge={bridge} />);
  // The view switch is there whatever the queue holds; the table only when the Queue view has rows.
  await screen.findByRole('button', { name: /^Completed, / }, { timeout: 5000 });
  return bridge;
}

describe('a ready row with a note, in the queue', () => {
  it('reads as ready with a note: its status, what the filename is described by, and the note in the Check column', async () => {
    await start([noted, plain, inReview]);
    expect(queueCell('Scan 0301.pdf', 'status')).toHaveTextContent('Ready with a note');
    expect(queueCell('Scan 0301.pdf', 'status').querySelector('.status')).toHaveClass('ready');
    // The arrow keys step over Status, so the filename's cell carries it.
    expect(queueCell('Scan 0301.pdf')).toHaveAccessibleDescription('Ready with a note');
    const check = queueCell('Scan 0301.pdf', 'check');
    expect(check).toHaveTextContent('Note: Type supplied by Intern +1 more');
    expect(check.querySelector('.check-label')).toHaveClass('noted');
    // Not the stylesheet's boxed notice, which `note` is.
    expect(check.querySelector('.check-label')).not.toHaveClass('note');
    expect(check).toHaveAttribute('title', describeNotes(codes));
    // Not a review's amber, and no part of the name marked to check.
    expect(check.querySelector('.check-label')).not.toHaveClass('review');
    expect(queueCell('Scan 0301.pdf', 'proposed').querySelector('mark')).toBeNull();
    // A ready row with nothing noted is as it was.
    expect(queueCell('Scan 0302.pdf', 'status')).toHaveTextContent(/^Ready$/);
    expect(queueCell('Scan 0302.pdf')).toHaveAccessibleDescription('Ready');
    expect(queueCell('Scan 0302.pdf', 'check')).toHaveTextContent('All checks passed');
  });

  it('is counted with the ready and never with those that need review', async () => {
    await start([noted, plain, inReview]);
    expect(screen.getByRole('button', { name: '1 needs review' })).toBeInTheDocument();
    // Rename all ready renames it with the rest, and the count beside it says how many have a note.
    const ready = screen.getByRole('button', { name: '2 ready, 1 with a note' });
    expect(screen.getByRole('button', { name: 'Rename all ready' })).toHaveTextContent('Rename all ready 2');
    expect(screen.getByRole('status', { name: 'Queue status' })).toHaveTextContent('Queue active. 2 ready, 1 needs review.');
    // "Need review" goes to the review row, not to the note.
    fireEvent.click(screen.getByRole('button', { name: '1 needs review' }));
    expect(queueCell('Scan 0303.pdf').closest('tr')).toHaveAttribute('aria-current', 'true');
    fireEvent.click(screen.getByRole('button', { name: '1 needs review' }));
    expect(queueCell('Scan 0303.pdf').closest('tr')).toHaveAttribute('aria-current', 'true');
    // Show only ready lists it, with its note.
    fireEvent.click(ready);
    const rows = screen.getAllByRole('row').slice(1);
    expect(rows.map((row) => row.querySelector('[data-column="original"]')?.textContent)).toEqual(['Scan 0301.pdf', 'Scan 0302.pdf']);
    expect(rows[0]).toHaveTextContent('Note: Type supplied by Intern');
  });

  it('says nothing of notes on the ready count when no ready row has one', async () => {
    await start([plain, inReview]);
    expect(screen.getByRole('button', { name: '1 ready' })).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: /with a note/ })).not.toBeInTheDocument();
  });

  it('is not what Intern selects at launch: that is the first document needing review', async () => {
    await start([noted, plain, inReview]);
    await waitFor(() => expect(queueCell('Scan 0303.pdf').closest('tr')).toHaveAttribute('aria-current', 'true'));
  });

  it('says how many of the documents Rename all ready renamed had a note', async () => {
    const bridge = await start([noted, plain, inReview]);
    fireEvent.click(screen.getByRole('button', { name: 'Rename all ready' }));
    await waitFor(() => expect(bridge.approve).toHaveBeenCalledTimes(2));
    expect(bridge.approve).toHaveBeenCalledWith('noted', noted.proposedFilename, noted.description);
    const result = await screen.findByRole('group', { name: 'Action result' });
    expect(result.querySelector('p')).toHaveTextContent(/^Renamed 2 documents\. 1 had a note from Intern\.$/);
    expect(screen.getByRole('status', { name: 'Action status' })).toHaveTextContent('Renamed 2 documents. 1 had a note from Intern.');
    // An approval clears a document's reasons, as the backend's does: renamed by the person, it carries no note.
    fireEvent.click(screen.getByRole('button', { name: /^Completed, / }));
    expect(queueCell('Scan 0301.pdf', 'status')).toHaveTextContent(/^Completed$/);
    expect(screen.queryByRole('button', { name: /with a note/ })).not.toBeInTheDocument();
  });

  it('says nothing of notes after Rename all ready when none of the documents had one', async () => {
    await start([plain, inReview]);
    fireEvent.click(screen.getByRole('button', { name: 'Rename all ready' }));
    const result = await screen.findByRole('group', { name: 'Action result' });
    expect(result.querySelector('p')).toHaveTextContent(/^Renamed 1 document\.$/);
  });
});

describe('a ready row with a note, in the Details panel', () => {
  it('says first that it is ready and what Intern noted, as a note and not as what to check', async () => {
    await start([noted, plain, inReview]);
    const panel = await openDetails('Scan 0301.pdf');
    const note = within(panel).getByRole('region', { name: "Intern's note" });
    expect(within(note).getByRole('heading', { name: 'Ready with a note' })).toBeVisible();
    expect(note).toHaveTextContent('This document is ready: nothing holds it. Intern noted this about its reading.');
    expect(note).toHaveTextContent(describeNotes(codes));
    // Before the name it is about.
    expect(note.compareDocumentPosition(within(panel).getByLabelText('Filename')) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
    // Nothing of a review: no "What to check", no instruction, no part of the name or the evidence marked.
    expect(within(panel).queryByRole('heading', { name: 'What to check' })).not.toBeInTheDocument();
    expect(panel).not.toHaveTextContent(/Check that it names the document|Check the name against the document/);
    expect(within(panel).queryByRole('note', { name: 'Part of the name to check' })).not.toBeInTheDocument();
    expect(panel.querySelector('mark.flagged')).toBeNull();
    // Said once: no second "Note" under the details.
    expect(within(panel).queryByRole('heading', { name: 'Note' })).not.toBeInTheDocument();
    expect(panel.querySelector('.inspector > .status, .source-file .status')).toHaveTextContent('Ready with a note');
    fireEvent.click(within(panel).getByText('Analysis details'));
    expect(panel.querySelector('.evidence-item.flagged')).toBeNull();
    expect(panel).not.toHaveTextContent('(check this)');
  });

  it('offers what a ready document is offered, and nothing that is only for one in review', async () => {
    await start([{ ...noted, partyChoices: ['Northwind Traders'], nameParties: ['Fabrikam Inc'] }, plain, inReview]);
    const panel = await openDetails('Scan 0301.pdf');
    // The ready action, under its ready name.
    expect(within(panel).getByRole('button', { name: /^Apply rename/ })).toBeEnabled();
    expect(within(panel).queryByRole('button', { name: /Approve & rename/ })).not.toBeInTheDocument();
    expect(within(panel).getByRole('button', { name: /^Keep original/ })).toBeEnabled();
    fireEvent.click(within(panel).getByRole('button', { name: 'More review actions' }));
    const more = within(panel).getByRole('group', { name: 'More review actions' });
    expect(within(more).getAllByRole('button').map((button) => button.textContent)).toEqual(['Analyze again', 'Remove from queueDel']);
    // No retry of any kind, and no other parties to choose: those go with a review.
    expect(within(panel).queryByRole('button', { name: /Retry|Process anyway|Check again/ })).not.toBeInTheDocument();
    expect(within(panel).queryByRole('group', { name: 'Other parties to choose from' })).not.toBeInTheDocument();
    expect(within(panel).queryByRole('note', { name: 'Why the name waits' })).not.toBeInTheDocument();
    // It is among what is left to decide, after those needing review, as any ready document is.
    expect(panel).toHaveTextContent('2 of 3 to decide');
  });

  it('files it under Apply rename like any ready document', async () => {
    const bridge = await start([noted, inReview]);
    const panel = await openDetails('Scan 0301.pdf');
    fireEvent.click(within(panel).getByRole('button', { name: /^Apply rename/ }));
    await waitFor(() => expect(bridge.approve).toHaveBeenCalledWith('noted', noted.proposedFilename, noted.description));
    expect(await screen.findByRole('group', { name: 'Action result' })).toHaveTextContent('Renamed 1 document.');
  });

  // "The model was unsure of its reading" in the note, and "100%" under
  // Analysis details: the same pair a review has, explained the same way.
  it('explains the model\'s own figure beside a note that the model was unsure', async () => {
    await start([noted, { ...noted, id: 'typed', originalFilename: 'Scan 0304.pdf', reasonCodes: ['TYPE_INFERRED'], reason: humanizeReason('TYPE_INFERRED') }]);
    const panel = await openDetails('Scan 0301.pdf');
    fireEvent.click(within(panel).getByText('Analysis details'));
    expect(within(panel).getByText("Model's own confidence").closest('p')).toHaveTextContent('100%');
    expect(within(panel).getByRole('note', { name: "About the model's own confidence" })).toBeInTheDocument();
    // A note about the type alone has nothing to reconcile.
    await openDetails('Scan 0304.pdf');
    expect(within(panel).getByRole('region', { name: "Intern's note" })).toHaveTextContent(describeNotes(['TYPE_INFERRED']));
    expect(within(panel).queryByRole('note', { name: "About the model's own confidence" })).not.toBeInTheDocument();
  });

  it('shows a ready document with nothing noted, and one in review, as it always did', async () => {
    await start([noted, plain, inReview]);
    const panel = await openDetails('Scan 0302.pdf');
    expect(within(panel).queryByRole('region', { name: "Intern's note" })).not.toBeInTheDocument();
    expect(panel.querySelector('.source-file .status')).toHaveTextContent(/^Ready$/);
    await openDetails('Scan 0303.pdf');
    expect(within(panel).queryByRole('region', { name: "Intern's note" })).not.toBeInTheDocument();
    expect(within(panel).getByRole('heading', { name: 'What to check' }).closest('section')).toHaveTextContent(humanizeReason('DATE_AMBIGUOUS'));
    expect(within(panel).getByRole('button', { name: /Approve & rename/ })).toBeEnabled();
  });

  it('reads the note from what the desktop backend sends for a ready row', async () => {
    // queue_item_dto joins the record's reasons whatever the row's status.
    const sent = [{ id: 9, originalFilename: 'Scan 0401.pdf', status: 'ready', proposedFilename: '2025-05-01 Letter to Halvorsen Fixture Works LLC.pdf', confidence: 1, description: 'Letter to Halvorsen Fixture Works LLC.', reason: 'TYPE_INFERRED, LOW_CONFIDENCE', proposalRevision: '1', approved: false }];
    const items = await new TauriBridge({ invoke: async <T,>() => sent as T, listen: async () => () => {} }).listItems();
    expect(items[0]).toMatchObject({ status: 'ready', reasonCodes: codes });
    await start(items);
    expect(queueCell('Scan 0401.pdf', 'status')).toHaveTextContent('Ready with a note');
    expect(queueCell('Scan 0401.pdf', 'check')).toHaveTextContent('Note: Type supplied by Intern +1 more');
    const panel = await openDetails('Scan 0401.pdf');
    expect(within(panel).getByRole('region', { name: "Intern's note" })).toHaveTextContent(describeNotes(codes));
  });
});

describe('a document renamed with a note, afterwards', () => {
  const receipt = (item: QueueItem, receiptId: string, direction: 'apply' | 'undo' = 'apply'): HistoryEntry => ({ receiptId, queueItemId: item.id, at: 1716369300 + Number(receiptId), direction, kind: 'rename', stage: 'complete', originalPath: `C:\\In\\${item.originalFilename}`, newPath: `C:\\Filed\\${item.filedName}` });

  it('is marked in Completed: its status, its Check cell, and its panel', async () => {
    await start([renamed, renamedPlain, inReview]);
    fireEvent.click(screen.getByRole('button', { name: /^Completed, / }));
    expect(queueCell('Scan 0298.pdf', 'status')).toHaveTextContent('Completed with a note');
    expect(queueCell('Scan 0298.pdf')).toHaveAccessibleDescription('Completed with a note');
    const check = queueCell('Scan 0298.pdf', 'check');
    expect(check).toHaveTextContent('Note: Type supplied by Intern +1 more');
    expect(check).toHaveAttribute('title', describeNotes(codes));
    expect(queueCell('Scan 0299.pdf', 'status')).toHaveTextContent(/^Completed$/);
    expect(queueCell('Scan 0299.pdf', 'check')).toHaveTextContent('—');

    const panel = await openDetails('Scan 0298.pdf');
    const note = within(panel).getByRole('region', { name: "Intern's note" });
    expect(within(note).getByRole('heading', { name: 'Renamed with a note' })).toBeVisible();
    expect(note).toHaveTextContent('Intern had this noted about its reading when the document was renamed.');
    expect(note).toHaveTextContent(describeNotes(codes));
    expect(within(panel).queryByRole('heading', { name: 'Note' })).not.toBeInTheDocument();
    // What a filed document is offered, and no more.
    expect(within(panel).getByRole('button', { name: 'Undo' })).toBeEnabled();
    expect(within(panel).queryByRole('button', { name: /rename|Keep original/i })).not.toBeInTheDocument();
  });

  it('is marked from what the desktop backend sends for a filed row: its reasons, and that it was renamed', async () => {
    // queue_item_dto, for a document Intern renamed by itself, one a person approved (its reasons cleared), and one kept under its own name.
    const sent = [
      { id: 21, originalFilename: 'Scan 0501.pdf', status: 'completed', proposedFilename: '2025-05-01 Letter to Halvorsen Fixture Works LLC.pdf', filedName: '2025-05-01 Letter to Halvorsen Fixture Works LLC.pdf', filedPath: 'C:\\Filed\\2025-05-01 Letter to Halvorsen Fixture Works LLC.pdf', keptOriginal: false, parked: false, undoable: true, approved: false, reason: 'TYPE_INFERRED, LOW_CONFIDENCE', proposalRevision: '1' },
      { id: 22, originalFilename: 'Scan 0502.pdf', status: 'completed', proposedFilename: '2025-05-02 Invoice.pdf', filedName: '2025-05-02 Invoice.pdf', filedPath: 'C:\\Filed\\2025-05-02 Invoice.pdf', keptOriginal: false, parked: false, undoable: true, approved: false, proposalRevision: '2' },
      { id: 23, originalFilename: 'Scan 0503.pdf', status: 'completed', proposedFilename: '2025-05-03 Notice.pdf', filedName: null, filedPath: null, keptOriginal: true, parked: false, undoable: false, approved: false, reason: 'TYPE_INFERRED, LOW_CONFIDENCE', proposalRevision: '1' },
    ];
    const items = await new TauriBridge({ invoke: async <T,>() => sent as T, listen: async () => () => {} }).listItems();
    await start(items);
    fireEvent.click(screen.getByRole('button', { name: /^Completed, / }));
    expect(queueCell('Scan 0501.pdf', 'status')).toHaveTextContent('Completed with a note');
    expect(queueCell('Scan 0501.pdf', 'check')).toHaveTextContent('Note: Type supplied by Intern +1 more');
    expect(queueCell('Scan 0502.pdf', 'status')).toHaveTextContent(/^Completed$/);
    expect(queueCell('Scan 0503.pdf', 'status')).toHaveTextContent(/^Completed$/);
    expect(queueCell('Scan 0503.pdf', 'check').querySelector('.check-label')).toHaveClass('quiet');
    expect(screen.getByRole('button', { name: '1 with a note' })).toBeInTheDocument();
  });

  it('can be listed alone in Completed, and the list lets go when none is left', async () => {
    await start([renamed, renamedPlain, inReview]);
    // Not offered in the queue, where nothing is filed.
    expect(screen.queryByRole('button', { name: '1 with a note' })).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: /^Completed, / }));
    const toggle = screen.getByRole('button', { name: '1 with a note' });
    expect(toggle).toHaveAttribute('aria-pressed', 'false');
    expect(screen.getAllByRole('row').slice(1)).toHaveLength(2);
    expect(screen.getByText('2 items')).toBeInTheDocument();

    fireEvent.click(toggle);
    expect(toggle).toHaveAttribute('aria-pressed', 'true');
    const rows = screen.getAllByRole('row').slice(1);
    expect(rows).toHaveLength(1);
    expect(rows[0]).toHaveTextContent('Scan 0298.pdf');
    expect(screen.getByText('1 of 2 items')).toBeInTheDocument();
    // It does not act on a document: a filter, like "ready".
    expect(toggle.closest('[data-acts]')).toBeNull();

    // Undone, the document is back in the queue and nothing is left to list.
    const panel = await openDetails('Scan 0298.pdf');
    fireEvent.click(within(panel).getByRole('button', { name: 'Undo' }));
    await waitFor(() => expect(screen.queryByRole('button', { name: /with a note/ })).not.toBeInTheDocument());
    expect(await screen.findByText('Scan 0299.pdf')).toBeInTheDocument();
    expect(screen.getByText('1 item')).toBeInTheDocument();
    // Back in the queue it waits for a decision, its codes and the undo as why.
    fireEvent.click(screen.getByRole('button', { name: /^Queue, / }));
    expect(queueCell('Scan 0298.pdf', 'status')).toHaveTextContent('Needs review');
    expect(queueCell('Scan 0298.pdf', 'check')).toHaveTextContent('Type supplied by Intern +2 more');
  });

  it('is not offered when no filed document has a note: one kept under its original name, with a review\'s codes, has none', async () => {
    const kept: QueueItem = { ...inReview, id: 'kept', originalFilename: 'Scan 0297.pdf', status: 'completed', keptOriginal: true, undoable: false };
    await start([kept, renamedPlain]);
    fireEvent.click(screen.getByRole('button', { name: /^Completed, / }));
    expect(screen.queryByRole('button', { name: /with a note/ })).not.toBeInTheDocument();
    expect(queueCell('Scan 0297.pdf', 'status')).toHaveTextContent(/^Completed$/);
    // As before: the codes it kept, quietly.
    expect(queueCell('Scan 0297.pdf', 'check')).toHaveTextContent('Date could be day- or month-first');
    expect(queueCell('Scan 0297.pdf', 'check').querySelector('.check-label')).toHaveClass('quiet');
  });

  it('is marked in Rename history on the row of its rename, while it is still in the queue', async () => {
    const history = [receipt(renamedPlain, '3'), receipt(renamed, '2'), receipt({ ...renamed, id: 'cleared', filedName: '2023-01-02 Letter.pdf' }, '1')];
    await start([renamed, renamedPlain], { history });
    fireEvent.click(screen.getByRole('button', { name: /^Completed, / }));
    fireEvent.click(screen.getByRole('button', { name: 'History' }));
    const dialog = await screen.findByRole('dialog', { name: 'Rename history' });
    const marks = await within(dialog).findAllByRole('note');
    expect(marks).toHaveLength(1);
    expect(marks[0]).toHaveAccessibleName(`Intern's note on ${renamed.filedName}`);
    expect(marks[0]).toHaveTextContent('Note: Type supplied by Intern (+1 more)');
    expect(marks[0]).toHaveAttribute('title', describeNotes(codes));
    expect(marks[0].closest('tr')).toHaveTextContent('Scan 0298.pdf');
  });

  it('is not marked in Rename history on an undo, or on a rename since undone', async () => {
    // Undone and renamed again by a person: the newest row is the rename that stands, and the row has no note left.
    const undone: QueueItem = { ...renamed, status: 'review', undoable: false, filedName: undefined, filedPath: undefined, keptOriginal: undefined };
    const history = [receipt(renamed, '5', 'undo'), receipt(renamed, '4')];
    await start([undone, renamedPlain], { history });
    fireEvent.click(screen.getByRole('button', { name: /^Completed, / }));
    fireEvent.click(screen.getByRole('button', { name: 'History' }));
    const dialog = await screen.findByRole('dialog', { name: 'Rename history' });
    await within(dialog).findByText('Undone');
    expect(within(dialog).queryByRole('note')).not.toBeInTheDocument();
  });
});
