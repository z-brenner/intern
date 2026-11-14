import { act, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { App } from '../App';
import { createInMemoryBridge } from '../lib/inMemoryBridge';
import type { DesktopBridge } from '../lib/bridge';
import type { QueueBridgeEvent, QueueEventSource } from '../lib/tauriBridge';
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
    // Before Settings opens, an undo in the name is the browser's.
    name.focus();
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
    name.focus();
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
      behind.focus();
      expect(allowed(behind, 'historyUndo')).toBe(true);

      fireEvent.click(await screen.findByRole('button', { name: 'History' }));
      const dialog = await screen.findByRole('dialog', { name: 'Rename history' });
      within(dialog).getByRole('button', { name: 'Close history' }).focus();
      for (const inputType of ['historyUndo', 'historyRedo', 'insertText']) expect(allowed(behind, inputType), inputType).toBe(false);

      fireEvent.keyDown(document.activeElement!, { key: 'Escape' });
      await waitFor(() => expect(dialog).not.toBeInTheDocument());
      behind.focus();
      expect(allowed(behind, 'historyUndo')).toBe(true);
    } finally {
      behind.remove();
    }
  });
});

/*
  The browser's undo goes to the field edited last wherever the focus is,
  with no dialog open too. In the review panel, Ctrl+Z in the Description,
  in the queue's filter or after a press on the panel's text undid the
  Filename's last edit and put the caret in it, and the next Enter - a new
  line, in the Description - filed the document under the name half
  undone. An undo or a redo changes only the field with the focus.
*/
describe('an undo in the review window', () => {
  it('lets no undo or redo from elsewhere in the review window reach the name', async () => {
    render(<App bridge={createInMemoryBridge()} />);
    const panel = await openRow(await screen.findByRole('row', { name: /Lease Agreement - 123 Main St.pdf/i }));
    const name = within(panel).getByRole('textbox', { name: 'Filename' });
    const description = within(panel).getByRole('textbox', { name: 'Description' });
    fireEvent.change(name, { target: { value: `${(name as HTMLTextAreaElement).value} v2` } });
    for (const [where, focusOn] of [
      ['in the Description', () => description.focus()],
      ['in the queue filter', () => screen.getByRole('searchbox', { name: 'Filter queue' }).focus()],
      ['on the panel heading', () => within(panel).getByRole('heading', { name: 'Review item' }).focus()],
      ['on nothing, after a press on the panel text', () => (document.activeElement as HTMLElement).blur()],
    ] as const) {
      focusOn();
      for (const inputType of ['historyUndo', 'historyRedo']) {
        expect(allowed(name, inputType), `${inputType} ${where}`).toBe(false);
        expect(allowed(description, inputType), `${inputType} in the description, ${where}`).toBe(where === 'in the Description');
      }
    }
    // In the field with the focus, undo and redo are the person's.
    name.focus();
    expect(allowed(name, 'historyUndo')).toBe(true);
    expect(allowed(name, 'historyRedo')).toBe(true);
    expect(name).toHaveValue('2023-09-15 Lease Agreement between ABC Properties LLC and TenantCo Inc v2');
  });
});

/**
 * jsdom has no inert: focus() puts the focus in an inert subtree. A browser
 * refuses it, and so does this, for the tests below; restored after each.
 */
function focusRefusedWhereInert() {
  const focus = HTMLElement.prototype.focus;
  return vi.spyOn(HTMLElement.prototype, 'focus').mockImplementation(function (this: HTMLElement, options?: FocusOptions) {
    if (!this.closest('[inert]')) focus.call(this, options);
  });
}

/*
  Nothing behind Settings or Rename history takes the focus while it is
  open. Shift+Tab after a press on the dialog's heading walked out to the
  review panel's Approve & rename; the queue put the focus on Rename all
  ready behind Settings when the selected document left it by itself; the
  review drawer, opened again under Settings by a narrower window, put the
  caret in the document's name. The keys typed next - the space bar,
  Enter, a spelling - filed documents with Settings still open.
*/
describe('the window behind a dialog', () => {
  it('is inert while Settings or Rename history is open, and the focus goes back to what opened it', async () => {
    const refused = focusRefusedWhereInert();
    try {
      const { container } = render(<App bridge={createInMemoryBridge({ items: [filed, { id: 'lease', originalFilename: 'Lease Agreement - 123 Main St.pdf', status: 'review', proposedFilename: '2023-09-15 Lease.pdf' }] })} />);
      await openRow(await screen.findByRole('row', { name: /Lease Agreement - 123 Main St.pdf/i }));
      const header = container.querySelector('.app-header')!;
      const workspace = container.querySelector('.workspace')!;
      const settings = screen.getAllByRole('button', { name: 'Settings' })[0];
      settings.focus();
      fireEvent.click(settings);
      const dialog = await screen.findByRole('dialog', { name: 'Settings' });
      expect(header).toHaveAttribute('inert');
      expect(workspace).toHaveAttribute('inert');
      expect(dialog.closest('[inert]')).toBeNull();
      fireEvent.keyDown(document.activeElement!, { key: 'Escape' });
      await waitFor(() => expect(dialog).not.toBeInTheDocument());
      expect(header).not.toHaveAttribute('inert');
      expect(workspace).not.toHaveAttribute('inert');
      expect(settings).toHaveFocus();

      fireEvent.click(await screen.findByRole('button', { name: /^Completed/ }));
      const history = await screen.findByRole('button', { name: 'History' });
      history.focus();
      fireEvent.click(history);
      const historyDialog = await screen.findByRole('dialog', { name: 'Rename history' });
      expect(header).toHaveAttribute('inert');
      expect(workspace).toHaveAttribute('inert');
      expect(historyDialog.closest('[inert]')).toBeNull();
      fireEvent.click(within(historyDialog).getByRole('button', { name: 'Close history' }));
      await waitFor(() => expect(historyDialog).not.toBeInTheDocument());
      expect(workspace).not.toHaveAttribute('inert');
      expect(history).toHaveFocus();
    } finally {
      refused.mockRestore();
    }
  });

  it('keeps the focus in Settings when the selected document leaves the queue by itself', async () => {
    const refused = focusRefusedWhereInert();
    try {
      const base = createInMemoryBridge({ liveEvents: true }) as DesktopBridge & QueueEventSource;
      let changed: ((event: QueueBridgeEvent) => void) | undefined;
      const bridge = { ...base, subscribeQueue: async (listener: (event: QueueBridgeEvent) => void) => { changed = listener; return base.subscribeQueue(listener); } };
      render(<App bridge={bridge} />);
      const panel = await openRow(await screen.findByRole('row', { name: /Lease Agreement - 123 Main St.pdf/i }));
      await waitFor(() => expect(changed).toBeDefined());
      fireEvent.click(screen.getAllByRole('button', { name: 'Settings' })[0]);
      const dialog = await screen.findByRole('dialog', { name: 'Settings' });
      const machine = within(dialog).getByRole('textbox', { name: "This machine's name" });
      machine.focus();
      // Its file taken from the Queue folder, by a person or another machine.
      await base.remove('lease');
      act(() => changed!({ type: 'changed' }));
      await waitFor(() => expect(panel).not.toBeInTheDocument());
      await act(async () => { await new Promise((resolve) => setTimeout(resolve, 20)); });
      expect(machine).toHaveFocus();
      expect(screen.getByRole('button', { name: 'Rename all ready' })).not.toHaveFocus();
    } finally {
      refused.mockRestore();
    }
  });

  it('keeps Tab and Shift+Tab in the dialog from nothing and from behind it', async () => {
    render(<App bridge={createInMemoryBridge({ items: [filed, { id: 'lease', originalFilename: 'Lease Agreement - 123 Main St.pdf', status: 'review', proposedFilename: '2023-09-15 Lease.pdf' }] })} />);
    const panel = await openRow(await screen.findByRole('row', { name: /Lease Agreement - 123 Main St.pdf/i }));
    const name = within(panel).getByRole('textbox', { name: 'Filename' });
    for (const [label, open] of [
      ['Settings', () => fireEvent.click(screen.getAllByRole('button', { name: 'Settings' })[0])],
      ['Rename history', async () => { fireEvent.click(await screen.findByRole('button', { name: /^Completed/ })); fireEvent.click(await screen.findByRole('button', { name: 'History' })); }],
    ] as const) {
      await open();
      const dialog = await screen.findByRole('dialog', { name: label });
      // Its first control is Close, and Shift+Tab from it goes round to its last.
      const first = within(dialog).getAllByRole('button')[0];
      first.focus();
      fireEvent.keyDown(first, { key: 'Tab', shiftKey: true });
      const last = document.activeElement;
      expect(dialog.contains(last) && last !== first).toBe(true);
      for (const [where, focusOn] of [
        ['on nothing, after a press on its heading', () => (document.activeElement as HTMLElement).blur()],
        ['behind it', () => (label === 'Settings' ? name : screen.getByRole('button', { name: /^Queue/ })).focus()],
      ] as const) {
        focusOn();
        expect(dialog.contains(document.activeElement), where).toBe(false);
        fireEvent.keyDown(document.activeElement ?? document.body, { key: 'Tab', shiftKey: true });
        expect(document.activeElement, `Shift+Tab ${where}, in ${label}`).toBe(last);
        focusOn();
        fireEvent.keyDown(document.activeElement ?? document.body, { key: 'Tab' });
        expect(document.activeElement, `Tab ${where}, in ${label}`).toBe(first);
      }
      fireEvent.keyDown(document.activeElement!, { key: 'Escape' });
      await waitFor(() => expect(dialog).not.toBeInTheDocument());
    }
  });
});
