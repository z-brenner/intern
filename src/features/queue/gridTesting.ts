import { fireEvent, screen, waitFor } from '@testing-library/react';
import { expect } from 'vitest';
import type { ColumnKey } from './grid';

/*
  Test helpers for the queue table and the Details panel. Selecting a row no
  longer opens the panel, so a test that reviews an item selects its row and
  then presses Details, as a person does.
*/

/** The cell of a row, found by the row's original filename. */
export function queueCell(originalFilename: string, column: ColumnKey = 'original'): HTMLElement {
  const original = screen.getAllByRole('gridcell', { name: originalFilename }).filter((cell) => cell.dataset.column === 'original');
  if (original.length !== 1) throw new Error(`Expected one row named ${originalFilename}, found ${original.length}`);
  if (column === 'original') return original[0];
  const cell = original[0].closest('tr')?.querySelector<HTMLElement>(`[data-column="${column}"]`);
  if (!cell) throw new Error(`No ${column} cell in the row of ${originalFilename}`);
  return cell;
}

/** Click a cell, as a person selecting it does (the table selects on mouse down). */
export function clickCell(cell: HTMLElement, init: MouseEventInit = {}) {
  fireEvent.mouseDown(cell, { button: 0, buttons: 1, ...init });
  fireEvent.mouseUp(cell, { button: 0, ...init });
  fireEvent.click(cell, { button: 0, ...init });
}

/** Select a row by clicking its original filename. */
export function selectRow(originalFilename: string) {
  clickCell(queueCell(originalFilename));
}

/** Select a table row and open the Details panel for it. Resolves with the panel. */
export async function openRow(row: HTMLElement): Promise<HTMLElement> {
  const cell = row.querySelector<HTMLElement>('[data-column="original"]');
  if (!cell) throw new Error('Not a queue row');
  return openDetails(cell.textContent ?? undefined);
}

export const detailsToggle = () => screen.getByRole('button', { name: 'Details' });

/**
 * Select a row (when named) and open the Details panel for it, unless it is
 * open already. Resolves with the panel.
 */
export async function openDetails(originalFilename?: string): Promise<HTMLElement> {
  if (originalFilename) selectRow(originalFilename);
  // Generous: under a loaded full run the first render of a file can take a while.
  await waitFor(() => expect(detailsToggle()).toBeEnabled(), { timeout: 3000 });
  const toggle = detailsToggle();
  if (toggle.getAttribute('aria-expanded') !== 'true') fireEvent.click(toggle);
  let panel: HTMLElement | undefined;
  await waitFor(() => {
    panel = screen.getByRole(window.matchMedia?.('(max-width: 1100px)').matches ? 'dialog' : 'complementary', { name: 'Review item' });
    if (originalFilename) expect(panel).toHaveTextContent(originalFilename);
  }, { timeout: 3000 });
  return panel!;
}
