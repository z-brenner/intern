import { act, cleanup, fireEvent, render, screen } from '@testing-library/react';
import { createRef } from 'react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { ReviewInspector } from '../../components/ReviewInspector';
import { describeAddReport } from '../../lib/addReport';
import { createInMemoryBridge } from '../../lib/inMemoryBridge';
import { humanizeReason } from '../../lib/reasons';
import type { QueueItem } from '../../types';
import { isUnreadable, itemActions, retryAccepted, withoutReviewPrefix } from './actions';
import type { ReviewInspectorHandle } from './useReviewShortcuts';

function handlers() {
  return {
    onClose: vi.fn(), onApprove: vi.fn(), onKeep: vi.fn(), onCancel: vi.fn(), onRetry: vi.fn(),
    onReanalyze: vi.fn(), onRemove: vi.fn(), onUndo: vi.fn(), onOpen: vi.fn(), onReveal: vi.fn(),
  };
}

/** A document Intern could not read, as the backend lists it: in review, renamed, with no proposal. */
const unreadable: QueueItem = {
  id: 'locked', originalFilename: 'NEEDS REVIEW - Lease scan.pdf', status: 'review',
  errorCode: 'PASSWORD_PROTECTED', reason: humanizeReason('PASSWORD_PROTECTED'), fileModifiedDate: '2026-01-02',
};

describe('a document Intern could not read', () => {
  afterEach(() => cleanup());

  it('can be retried or named by hand, and is not offered Analyze again beside Retry', () => {
    expect(isUnreadable(unreadable)).toBe(true);
    expect(retryAccepted(unreadable)).toBe(true);
    expect(itemActions(unreadable)).toMatchObject({ approve: true, keep: true, reanalyze: false, retry: { label: 'Retry', primary: false }, open: true, reveal: true });
    for (const code of ['UNSUPPORTED_CONTENT', 'DOCUMENT_TOO_LARGE', 'EXTRACTION_FAILED', 'MODEL_DECLINED', 'MODEL_OUTPUT_INVALID', 'ANALYSIS_FAILED', 'MODEL_FAILED']) {
      expect(retryAccepted({ ...unreadable, errorCode: code }), code).toBe(true);
    }
    // A failed item with the same code is not one of these: it is failed.
    expect(isUnreadable({ ...unreadable, status: 'failed' })).toBe(false);
    // Machine problems never reach review under these codes.
    expect(isUnreadable({ ...unreadable, errorCode: 'OCR_UNAVAILABLE' })).toBe(false);
  });

  it('is named starting from the name it arrived with, and approved under the name typed', () => {
    const actions = handlers();
    render(<ReviewInspector item={unreadable} drawer={false} {...actions} />);
    expect(screen.getByRole('heading', { name: 'Name it yourself' })).toBeVisible();
    const field = screen.getByRole('textbox', { name: 'Filename' });
    expect(field).toHaveValue('Lease scan');
    // No date yet: the file's own date is one click.
    fireEvent.click(screen.getByRole('button', { name: /2026-01-02/ }));
    expect(field).toHaveValue('2026-01-02 Lease scan');
    fireEvent.click(screen.getByRole('button', { name: /Approve & rename/ }));
    expect(actions.onApprove).toHaveBeenCalledWith('2026-01-02 Lease scan.pdf', '');
  });

  it('says what Remove leaves behind when it asks first', () => {
    const ref = createRef<ReviewInspectorHandle>();
    render(<ReviewInspector ref={ref} item={unreadable} drawer={false} {...handlers()} />);
    act(() => ref.current?.requestRemove());
    expect(screen.getByRole('group', { name: 'Confirm removal' })).toHaveTextContent('stays where it is, under this name, and Intern leaves it alone');
  });

  it('has sentences that point at what can be done now', () => {
    for (const code of ['PASSWORD_PROTECTED', 'UNSUPPORTED_CONTENT', 'DOCUMENT_TOO_LARGE', 'EXTRACTION_FAILED', 'MODEL_FAILED', 'MODEL_OUTPUT_INVALID', 'ANALYSIS_FAILED', 'MODEL_DECLINED']) {
      const sentence = humanizeReason(code);
      expect(sentence, code).toMatch(/name it yourself/i);
      expect(sentence, code).not.toMatch(/retry or remove it|add it again/i);
    }
  });

  it('keeps its own name back without the prefix, in the in-memory bridge as in the backend', async () => {
    const bridge = createInMemoryBridge({ items: [{ ...unreadable }] });
    await bridge.keepOriginal('locked');
    expect((await bridge.listItems())[0]).toMatchObject({ status: 'completed', keptOriginal: true, originalFilename: 'Lease scan.pdf' });
    expect(withoutReviewPrefix('needs review - a.pdf')).toBe('a.pdf');
    expect(withoutReviewPrefix('a.pdf')).toBe('a.pdf');
  });

  it('is never taken in again when added', async () => {
    const bridge = createInMemoryBridge({ items: [] });
    const report = await bridge.addFiles([{ path: 'C:\\Inbox\\NEEDS REVIEW - scan.pdf', displayName: 'NEEDS REVIEW - scan.pdf' }]);
    expect(report).toEqual({ added: 0, alreadyQueued: 0, skipped: [{ name: 'NEEDS REVIEW - scan.pdf', code: 'NEEDS_REVIEW_FILE' }] });
    expect(describeAddReport(report)).toContain('its name marks it as waiting for review');
  });
});

describe('an exact duplicate', () => {
  const filed: QueueItem = {
    id: 'lease', originalFilename: 'lease scan.pdf', status: 'completed', proposedFilename: '2026-01-02 Lease.pdf',
    filedName: '2026-01-02 Lease.pdf', filedPath: 'C:\\Filed\\2026\\2026-01-02 Lease.pdf', undoable: true, keptOriginal: false,
  };

  it('is filed at once as a copy named after the original, numbered after the first', async () => {
    const bridge = createInMemoryBridge({ items: [{ ...filed }], settings: { destination: 'C:\\Filed' } });
    await bridge.addFiles([{ path: 'C:\\Inbox\\lease scan.pdf', displayName: 'lease scan.pdf' }]);
    await bridge.addFiles([{ path: 'C:\\Inbox\\again\\lease scan.pdf', displayName: 'lease scan.pdf' }]);
    const [, first, second] = await bridge.listItems();
    expect(first).toMatchObject({ status: 'completed', filedName: 'COPY - 2026-01-02 Lease.pdf', filedPath: 'C:\\Filed\\2026\\COPY - 2026-01-02 Lease.pdf', undoable: true, keptOriginal: false, reason: humanizeReason('EXACT_COPY') });
    expect(second).toMatchObject({ status: 'completed', filedName: 'COPY - 2026-01-02 Lease (2).pdf' });
  });

  it('waits in review as a duplicate once undone, and Process anyway reads it', async () => {
    const bridge = createInMemoryBridge({ items: [{ ...filed }], settings: { destination: 'C:\\Filed' } });
    await bridge.addFiles([{ path: 'C:\\Inbox\\lease scan.pdf', displayName: 'lease scan.pdf' }]);
    const copy = (await bridge.listItems())[1];
    await bridge.undo(copy.id);
    const undone = (await bridge.listItems())[1];
    expect(undone).toMatchObject({ status: 'review', errorCode: 'DUPLICATE' });
    expect(undone.reason).toContain('You undid this rename');
    expect(itemActions(undone)).toMatchObject({ approve: false, retry: { label: 'Process anyway' } });
    await bridge.retry(copy.id);
    expect((await bridge.listItems())[1]).toMatchObject({ status: 'waiting' });
  });

  it('stays in review untouched with no output folder to file a copy into', async () => {
    const bridge = createInMemoryBridge({ items: [{ ...filed }], settings: { destination: '' } });
    await bridge.addFiles([{ path: 'C:\\Inbox\\lease scan.pdf', displayName: 'lease scan.pdf' }]);
    expect((await bridge.listItems())[1]).toMatchObject({ status: 'review', errorCode: 'DUPLICATE', reason: 'Duplicate of 2026-01-02 Lease.pdf' });
  });

  it('explains the duplicate left where it is without promising it was never filed elsewhere', () => {
    expect(humanizeReason('DUPLICATE')).toContain('so Intern did not file it again');
    expect(humanizeReason('EXACT_COPY')).toContain('filed it as a copy');
  });
});
