import { fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { App } from '../App';
import { createInMemoryBridge } from '../lib/inMemoryBridge';
import { openRow } from '../features/queue/gridTesting';
import type { QueueItem } from '../types';

/*
  The browser's undo and redo go to the field edited last in the page,
  wherever the focus is, and take the focus there. With Settings open over
  the review window, Ctrl+Z in a field of Settings with nothing of its own
  to undo undid the review panel's Filename behind the dialog and put the
  caret in it, where Enter filed the document. While Settings or Rename
  history is open, an input aimed outside it is refused, and once it closes
  it is not.
*/

/** Whether the browser may make the input `inputType` it is about to make in `field`: an event nobody refused. */
const allowed = (field: HTMLElement, inputType: string) =>
  fireEvent(field, new InputEvent('beforeinput', { inputType, data: inputType === 'insertText' ? 'x' : null, bubbles: true, cancelable: true }));

const filed: QueueItem = { id: 'filed', originalFilename: 'lease.pdf', status: 'completed', proposedFilename: '2026-01-02 Lease.pdf', filedName: '2026-01-02 Lease.pdf' };

describe('a dialog over the review window', () => {
  it('lets no undo, redo or typing reach a field behind Settings while it is open', async () => {
    render(<App bridge={createInMemoryBridge()} />);
    const panel = await openRow(await screen.findByRole('row', { name: /Lease Agreement - 123 Main St.pdf/i }));
    const name = within(panel).getByRole('textbox', { name: 'Filename' });
    const description = within(panel).getByRole('textbox', { name: 'Description' });
    fireEvent.change(name, { target: { value: `${(name as HTMLTextAreaElement).value} v2` } });
    // Before Settings opens, an undo is the browser's.
    expect(allowed(name, 'historyUndo')).toBe(true);

    fireEvent.click(screen.getAllByRole('button', { name: 'Settings' })[0]);
    const dialog = await screen.findByRole('dialog', { name: 'Settings' });
    const machine = within(dialog).getByRole('textbox', { name: "This machine's name" });
    for (const focusOn of [() => machine.focus(), () => within(dialog).getByRole('button', { name: 'Close settings' }).focus(), () => (document.activeElement as HTMLElement).blur()]) {
      focusOn();
      for (const inputType of ['historyUndo', 'historyRedo', 'insertText']) {
        expect(allowed(name, inputType), `${inputType} in the name`).toBe(false);
        expect(allowed(description, inputType), `${inputType} in the description`).toBe(false);
      }
    }
    // Inside the dialog, editing is the person's.
    machine.focus();
    expect(allowed(machine, 'historyUndo')).toBe(true);
    expect(allowed(machine, 'insertText')).toBe(true);
    expect(name).toHaveValue('2023-09-15 Lease Agreement between ABC Properties LLC and TenantCo Inc v2');

    fireEvent.keyDown(machine, { key: 'Escape' });
    await waitFor(() => expect(dialog).not.toBeInTheDocument());
    expect(allowed(name, 'historyUndo')).toBe(true);
    expect(allowed(name, 'insertText')).toBe(true);
  });

  it('lets no undo, redo or typing reach a field behind Rename history while it is open', async () => {
    // A field of the page behind it, edited before the dialog opened.
    const behind = document.body.appendChild(document.createElement('input'));
    try {
      render(<App bridge={createInMemoryBridge({ items: [filed] })} />);
      fireEvent.click(await screen.findByRole('button', { name: /^Completed/ }));
      fireEvent.change(behind, { target: { value: 'edited' } });
      expect(allowed(behind, 'historyUndo')).toBe(true);

      fireEvent.click(await screen.findByRole('button', { name: 'History' }));
      const dialog = await screen.findByRole('dialog', { name: 'Rename history' });
      within(dialog).getByRole('button', { name: 'Close history' }).focus();
      for (const inputType of ['historyUndo', 'historyRedo', 'insertText']) expect(allowed(behind, inputType), inputType).toBe(false);

      fireEvent.keyDown(document.activeElement!, { key: 'Escape' });
      await waitFor(() => expect(dialog).not.toBeInTheDocument());
      expect(allowed(behind, 'historyUndo')).toBe(true);
    } finally {
      behind.remove();
    }
  });
});
