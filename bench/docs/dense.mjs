/// Helpers for the long documents built to test retrieval: each places a
/// defining fact on a chosen page, so the layout has to land each section
/// exactly where the gold says it is. Sections of fixed text are laid out
/// as they come; the schedules between them are made of many small blocks,
/// each different, laid out until the next section's page is reached.

/// Lays out blocks from `next` until the flow is on page `page`, then
/// returns: either the last block spilled onto that page (the next section
/// starts below it), or the page before it is full to within `room`
/// points and a page break starts the next section at its top. `next`
/// lays out one block and returns false when it has none left.
export function fillTo(flow, page, next, { room = 110, id = 'document' } = {}) {
  let guard = 0;
  while (flow.pageNumber < page - 1 || (flow.pageNumber === page - 1 && flow.bottom - flow.y > room)) {
    guard += 1;
    if (guard > 10000 || !next()) throw new Error(`${id}: ran out of blocks before page ${page} (on page ${flow.pageNumber})`);
  }
  if (flow.pageNumber > page) throw new Error(`${id}: a block ran past page ${page} to page ${flow.pageNumber}`);
  if (flow.pageNumber === page - 1) flow.pageBreak();
}

/// A `next` for [`fillTo`] that lays out `items` in order with `emit`.
export function blocks(items, emit) {
  let index = 0;
  return () => {
    if (index >= items.length) return false;
    emit(items[index], index);
    index += 1;
    return true;
  };
}

/// A `next` that runs each of `sources` to exhaustion in turn.
export function chain(...sources) {
  let current = 0;
  return () => {
    while (current < sources.length) {
      if (sources[current]()) return true;
      current += 1;
    }
    return false;
  };
}

/// The pages (1-based) whose text contains `needle`, whitespace collapsed.
export function pagesContaining(text, needle) {
  return text.flatMap((page, index) => (page.replace(/\s+/g, ' ').includes(needle) ? [index + 1] : []));
}

/// Refuses a layout where `needle` is not printed on exactly `expected`.
export function expectPages(id, text, needle, expected) {
  const found = pagesContaining(text, needle);
  if (JSON.stringify(found) !== JSON.stringify(expected)) {
    throw new Error(`${id}: "${needle}" is printed on pages [${found}], expected [${expected}]`);
  }
}

/// A `next` that lays out `rows` as one table a few rows at a time, so a
/// schedule can stop wherever the page it fills ends. The title, the
/// introduction and the header come with the first rows.
export function tableBlocks(flow, columns, rows, { chunk = 4, title = null, intro = null, size = 8.5 } = {}) {
  let index = 0;
  return () => {
    if (index >= rows.length) return false;
    if (index === 0) {
      if (title) flow.heading(title, { level: 2 });
      if (intro) flow.paragraph(intro, { size: 9.5 });
    }
    const last = index + chunk >= rows.length;
    flow.table(columns, rows.slice(index, index + chunk), { size, header: index === 0, after: last ? 8 : 0 });
    index += chunk;
    return true;
  };
}
