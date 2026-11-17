import { fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { App } from '../../App';
import { createInMemoryBridge } from '../../lib/inMemoryBridge';
import { describeNotes, humanizeReason, noteWhy, reasonLines } from '../../lib/reasons';
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

/**
 * What a note says, chip by chip: each code's few words, and the sentence it
 * opens, which is its tooltip as well.
 */
function noteSays(note: HTMLElement): Array<{ label: string; why: string }> {
  return within(within(note).getByRole('list', { name: 'ATLAS noted' })).getAllByRole('button').map((chip) => ({
    label: chip.textContent ?? '',
    why: chip.getAttribute('title') ?? '',
  }));
}

/** The chips `codes` make, as a note says them. */
const chipsOf = (labels: string[], noteCodes: string[]) => noteCodes.map((code, index) => ({ label: labels[index], why: noteWhy(code) }));

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
    expect(check).toHaveTextContent('Note: Type guessed +1 more');
    expect(check.querySelector('.check-label')).toHaveClass('noted');
    // Not the stylesheet's boxed notice, which `note` is.
    expect(check.querySelector('.check-label')).not.toHaveClass('note');
    expect(check).toHaveAttribute('title', reasonLines(codes, { note: true }));
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
    expect(rows[0]).toHaveTextContent('Note: Type guessed');
  });

  it('says nothing of notes on the ready count when no ready row has one', async () => {
    await start([plain, inReview]);
    expect(screen.getByRole('button', { name: '1 ready' })).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: /with a note/ })).not.toBeInTheDocument();
  });

  it('is not what ATLAS selects at launch: that is the first document needing review', async () => {
    await start([noted, plain, inReview]);
    await waitFor(() => expect(queueCell('Scan 0303.pdf').closest('tr')).toHaveAttribute('aria-current', 'true'));
  });

  it('says how many of the documents Rename all ready renamed had a note', async () => {
    const bridge = await start([noted, plain, inReview]);
    fireEvent.click(screen.getByRole('button', { name: 'Rename all ready' }));
    await waitFor(() => expect(bridge.approve).toHaveBeenCalledTimes(2));
    expect(bridge.approve).toHaveBeenCalledWith('noted', noted.proposedFilename, noted.description);
    const result = await screen.findByRole('group', { name: 'Action result' });
    expect(result.querySelector('p')).toHaveTextContent(/^Renamed 2 documents\. 1 had a note from ATLAS\.$/);
    expect(screen.getByRole('status', { name: 'Action status' })).toHaveTextContent('Renamed 2 documents. 1 had a note from ATLAS.');
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
  it('says first that it is ready and what ATLAS noted, as a note and not as what to check', async () => {
    await start([noted, plain, inReview]);
    const panel = await openDetails('Scan 0301.pdf');
    const note = within(panel).getByRole('region', { name: "ATLAS's note" });
    expect(within(note).getByRole('heading', { name: 'Ready with a note' })).toBeVisible();
    expect(note).toHaveTextContent('Nothing holds it. ATLAS only noted this about its reading.');
    // A few words a note, each opening its sentence: the note's, never a review's instruction.
    expect(noteSays(note)).toEqual(chipsOf(['Type guessed', 'Model unsure'], codes));
    fireEvent.click(within(note).getByRole('button', { name: 'Model unsure' }));
    expect(within(note).getByText(/If you look at this name, start with the date and the parties\./)).toBeVisible();
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

  // The confidence is one figure, consistent with the status (the
  // backend's): beside a note that the model was unsure it is shown once,
  // and no paragraph about the model's own figure is needed.
  it('shows the confidence once beside a note that the model was unsure, and explains nothing about it', async () => {
    await start([noted, { ...noted, id: 'typed', originalFilename: 'Scan 0304.pdf', reasonCodes: ['TYPE_INFERRED'], reason: humanizeReason('TYPE_INFERRED') }]);
    const panel = await openDetails('Scan 0301.pdf');
    fireEvent.click(within(panel).getByText('Analysis details'));
    expect(within(panel).getAllByText('Confidence')).toHaveLength(1);
    expect(within(panel).getByText('Confidence').closest('p')).toHaveTextContent('100%');
    expect(panel).not.toHaveTextContent(/model's own/i);
    await openDetails('Scan 0304.pdf');
    expect(noteSays(within(panel).getByRole('region', { name: "ATLAS's note" }))).toEqual(chipsOf(['Type guessed'], ['TYPE_INFERRED']));
  });

  it('shows a ready document with nothing noted, and one in review, as it always did', async () => {
    await start([noted, plain, inReview]);
    const panel = await openDetails('Scan 0302.pdf');
    expect(within(panel).queryByRole('region', { name: "ATLAS's note" })).not.toBeInTheDocument();
    expect(panel.querySelector('.source-file .status')).toHaveTextContent(/^Ready$/);
    await openDetails('Scan 0303.pdf');
    expect(within(panel).queryByRole('region', { name: "ATLAS's note" })).not.toBeInTheDocument();
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
    expect(queueCell('Scan 0401.pdf', 'check')).toHaveTextContent('Note: Type guessed +1 more');
    const panel = await openDetails('Scan 0401.pdf');
    expect(noteSays(within(panel).getByRole('region', { name: "ATLAS's note" }))).toEqual(chipsOf(['Type guessed', 'Model unsure'], codes));
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
    expect(check).toHaveTextContent('Note: Type guessed +1 more');
    expect(check).toHaveAttribute('title', reasonLines(codes, { note: true }));
    expect(queueCell('Scan 0299.pdf', 'status')).toHaveTextContent(/^Completed$/);
    expect(queueCell('Scan 0299.pdf', 'check')).toHaveTextContent('—');

    const panel = await openDetails('Scan 0298.pdf');
    const note = within(panel).getByRole('region', { name: "ATLAS's note" });
    expect(within(note).getByRole('heading', { name: 'Renamed with a note' })).toBeVisible();
    expect(note).toHaveTextContent('ATLAS noted this about its reading when it was renamed.');
    expect(noteSays(note)).toEqual(chipsOf(['Type guessed', 'Model unsure'], codes));
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
    expect(queueCell('Scan 0501.pdf', 'check')).toHaveTextContent('Note: Type guessed +1 more');
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
    expect(queueCell('Scan 0298.pdf', 'check')).toHaveTextContent('Type guessed +2 more');
  });

  it('is not offered when no filed document has a note: one kept under its original name, with a review\'s codes, has none', async () => {
    const kept: QueueItem = { ...inReview, id: 'kept', originalFilename: 'Scan 0297.pdf', status: 'completed', keptOriginal: true, undoable: false };
    await start([kept, renamedPlain]);
    fireEvent.click(screen.getByRole('button', { name: /^Completed, / }));
    expect(screen.queryByRole('button', { name: /with a note/ })).not.toBeInTheDocument();
    expect(queueCell('Scan 0297.pdf', 'status')).toHaveTextContent(/^Completed$/);
    // As before: the codes it kept, quietly.
    expect(queueCell('Scan 0297.pdf', 'check')).toHaveTextContent('Date order unclear');
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
    expect(marks[0]).toHaveAccessibleName(`ATLAS's note on ${renamed.filedName}`);
    expect(marks[0]).toHaveTextContent('Note: Type guessed (+1 more)');
    expect(marks[0]).toHaveAttribute('title', reasonLines(codes, { note: true }));
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

/*
  TYPE_MISSING is raised for a type that is a general word alone ("Letter",
  "Agreement", "Notice"), and at Balanced such a reading is ready only when
  its name has a type. The Check cell said "Note: No document type" beside a
  name that had one, in the queue, in the copied text and in Rename history.
*/
describe('a note that the type is a general word', () => {
  const fromBackend = (sent: unknown[]) => new TauriBridge({ invoke: async <T,>() => sent as T, listen: async () => () => {} }).listItems();
  const name = '2024-03-08 Letter to Fabrikam Inc.pdf';

  it('never says the name has no type: ready, renamed, and in Rename history', async () => {
    const items = await fromBackend([
      { id: 31, originalFilename: 'Scan 0601.pdf', status: 'ready', proposedFilename: name, confidence: 0.97, description: 'Letter to Fabrikam Inc.', reason: 'TYPE_MISSING', proposalRevision: '1', approved: false },
      { id: 32, originalFilename: 'Scan 0602.pdf', status: 'completed', proposedFilename: '2024-03-09 Agreement with Contoso Ltd.pdf', filedName: '2024-03-09 Agreement with Contoso Ltd.pdf', filedPath: 'C:\\Filed\\2024-03-09 Agreement with Contoso Ltd.pdf', keptOriginal: false, parked: false, undoable: true, approved: false, reason: 'TYPE_MISSING, LOW_CONFIDENCE', proposalRevision: '1' },
    ]);
    const history: HistoryEntry[] = [{ receiptId: '7', queueItemId: '32', at: 1716369307, direction: 'apply', kind: 'rename', stage: 'complete', originalPath: 'C:\\In\\Scan 0602.pdf', newPath: 'C:\\Filed\\2024-03-09 Agreement with Contoso Ltd.pdf' }];
    await start(items, { history, settings: { reviewLevel: 'balanced' } });
    expect(queueCell('Scan 0601.pdf', 'status')).toHaveTextContent('Ready with a note');
    expect(queueCell('Scan 0601.pdf', 'proposed')).toHaveTextContent(name);
    expect(queueCell('Scan 0601.pdf', 'check')).toHaveTextContent(/^Note: No specific type$/);
    expect(queueCell('Scan 0601.pdf', 'check')).toHaveAttribute('title', 'No specific type: No specific document type was identified.');
    const panel = await openDetails('Scan 0601.pdf');
    expect(within(panel).getByRole('region', { name: "ATLAS's note" })).toHaveTextContent('No specific document type was identified.');
    expect(document.body).not.toHaveTextContent('No document type');

    fireEvent.click(screen.getByRole('button', { name: /^Completed, / }));
    expect(queueCell('Scan 0602.pdf', 'status')).toHaveTextContent('Completed with a note');
    expect(queueCell('Scan 0602.pdf', 'check')).toHaveTextContent('Note: No specific type +1 more');
    fireEvent.click(screen.getByRole('button', { name: 'History' }));
    const dialog = await screen.findByRole('dialog', { name: 'Rename history' });
    expect(await within(dialog).findByRole('note')).toHaveTextContent('Note: No specific type (+1 more)');
    expect(document.body).not.toHaveTextContent('No document type');
  });

  it('is still "No document type" on a document in review, where the name may have none', async () => {
    const items = await fromBackend([{ id: 33, originalFilename: 'Scan 0603.pdf', status: 'needs_review', proposedFilename: '2024-03-08 Document.pdf', confidence: 0.9, reason: 'TYPE_MISSING', proposalRevision: '1', approved: false }]);
    await start(items);
    expect(queueCell('Scan 0603.pdf', 'status')).toHaveTextContent('Needs review');
    expect(queueCell('Scan 0603.pdf', 'check')).toHaveTextContent(/^No document type$/);
  });
});

/*
  A document with notes is not always ready or renamed. While Intern renames
  it the backend calls it applying, and one that already had exactly the name
  Intern would give it is completed under that name, ALREADY_NAMED added to
  the reasons it carried. Neither waits for a person, and the panel's closing
  note told the person to check both.
*/
describe('a document with notes that is being renamed, or was completed under the name it had', () => {
  const fromBackend = (sent: unknown[]) => new TauriBridge({ invoke: async <T,>() => sent as T, listen: async () => () => {} }).listItems();
  const instruction = /Check that it names the document|Check the name against the document|Check the type/;
  const alreadyNamed = 'This document already had this name, so nothing was renamed.';
  const sent = [
    { id: 41, originalFilename: 'Scan 0701.pdf', status: 'applying', proposedFilename: '2024-03-09 Letter to Contoso Ltd.pdf', confidence: 1, reason: 'TYPE_INFERRED, LOW_CONFIDENCE', proposalRevision: '1', approved: false },
    { id: 42, originalFilename: '2024-03-08 Letter to Fabrikam Inc.pdf', status: 'completed', proposedFilename: '2024-03-08 Letter to Fabrikam Inc.pdf', confidence: 1, filedName: null, filedPath: null, keptOriginal: true, parked: false, undoable: false, approved: false, reason: 'TYPE_INFERRED, LOW_CONFIDENCE, ALREADY_NAMED', proposalRevision: '2' },
    // Already named after a person's approval, or at Strict: the one reason, as before.
    { id: 43, originalFilename: '2024-03-07 Invoice INV-7 from Litware Inc.pdf', status: 'completed', proposedFilename: '2024-03-07 Invoice INV-7 from Litware Inc.pdf', filedName: null, filedPath: null, keptOriginal: true, parked: false, undoable: false, approved: false, reason: 'ALREADY_NAMED', proposalRevision: '2' },
    // Kept under its own name by a person, from review: the review's words, as before.
    { id: 44, originalFilename: 'Scan 0704.pdf', status: 'completed', proposedFilename: '2024-03-06 Notice.pdf', filedName: null, filedPath: null, keptOriginal: true, parked: false, undoable: false, approved: false, reason: 'TYPE_INFERRED, LOW_CONFIDENCE', proposalRevision: '1' },
  ];

  it('says what ATLAS noted while the document is renamed, and tells nobody to check it', async () => {
    const items = await fromBackend(sent);
    expect(items[0]).toMatchObject({ status: 'processing', stage: 'filing', reasonCodes: codes });
    await start(items);
    expect(queueCell('Scan 0701.pdf', 'status')).toHaveTextContent('Renaming');
    const panel = await openDetails('Scan 0701.pdf');
    const note = within(panel).getByRole('heading', { name: 'Note' }).closest('section')!;
    expect(note).toHaveTextContent(describeNotes(codes));
    expect(panel).not.toHaveTextContent(instruction);
    // Not ready and not renamed: no "with a note" of either kind, and nothing of a review.
    expect(within(panel).queryByRole('region', { name: "ATLAS's note" })).not.toBeInTheDocument();
    expect(within(panel).queryByRole('heading', { name: 'What to check' })).not.toBeInTheDocument();
  });

  it('says first that the document already had its name, then the notes, in the panel and the Check cell', async () => {
    await start(await fromBackend(sent));
    fireEvent.click(screen.getByRole('button', { name: /^Completed, / }));
    const row = '2024-03-08 Letter to Fabrikam Inc.pdf';
    expect(queueCell(row, 'status')).toHaveTextContent(/^Completed$/);
    const check = queueCell(row, 'check');
    expect(check).toHaveTextContent('Already had this name +2 more');
    expect(check.querySelector('.check-label')).toHaveClass('quiet');
    expect(check).toHaveAttribute('title', `Already had this name: ${alreadyNamed}\n${reasonLines(codes, { note: true })}`);
    // Nothing was renamed, so it is not among those renamed with a note.
    expect(screen.queryByRole('button', { name: /with a note/ })).not.toBeInTheDocument();

    const panel = await openDetails(row);
    expect(panel).toHaveTextContent('Kept its original name');
    const note = within(panel).getByRole('heading', { name: 'Note' }).closest('section')!;
    expect(note).toHaveTextContent(`${alreadyNamed} ${describeNotes(codes)}`);
    expect(panel).not.toHaveTextContent(instruction);
    // The confidence beside it is one figure, with nothing to explain.
    fireEvent.click(within(panel).getByText('Analysis details'));
    expect(within(panel).getAllByText('Confidence')).toHaveLength(1);
    expect(panel).not.toHaveTextContent(/model's own/i);
  });

  it('leaves a document already named with nothing noted, and one a person kept, as they were', async () => {
    await start(await fromBackend(sent));
    fireEvent.click(screen.getByRole('button', { name: /^Completed, / }));
    expect(queueCell('2024-03-07 Invoice INV-7 from Litware Inc.pdf', 'check')).toHaveTextContent(/^Already had this name$/);
    const panel = await openDetails('2024-03-07 Invoice INV-7 from Litware Inc.pdf');
    expect(within(panel).getByRole('heading', { name: 'Note' }).closest('section')).toHaveTextContent(new RegExp(`^Note${alreadyNamed.replace(/[.]/g, '\\.')}$`));
    // A person's Keep original leaves the review's codes on the record; that row is the released one's.
    expect(queueCell('Scan 0704.pdf', 'check')).toHaveTextContent('Type guessed +1 more');
    expect(queueCell('Scan 0704.pdf', 'check')).toHaveAttribute('title', reasonLines(codes));
    await openDetails('Scan 0704.pdf');
    expect(within(panel).getByRole('heading', { name: 'Note' }).closest('section')).toHaveTextContent(humanizeReason(codes.join(', ')));
  });
});
