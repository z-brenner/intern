import { readFile } from 'node:fs/promises';
import { expect, it } from 'vitest';
import { SUPPORTED_EXTENSIONS } from '../src/lib/formats';

// The file picker's Documents filter and the in-memory bridge's skipping both
// read SUPPORTED_EXTENSIONS, the frontend's copy of the list the backend
// admits. A format the backend gains and the copy lacks is hidden by the
// filter and refused by the demo, and no other test would notice - so the
// copy is held to the backend's own declaration, in intern-core, which every
// crate that admits a file reads. When the list moves, this has to move with
// it.
it('offers exactly the extensions the backend admits', async () => {
  const source = await readFile('crates/intern-core/src/snapshot.rs', 'utf8');
  const declared = /pub const SUPPORTED_EXTENSIONS: &\[&str\] = &\[([^\]]*)\];/.exec(source)?.[1];
  expect(declared, 'SUPPORTED_EXTENSIONS is declared in intern-core').toBeDefined();
  const backend = [...declared!.matchAll(/"([^"]+)"/g)].map((match) => match[1]);
  expect(backend.length).toBeGreaterThan(0);

  expect(new Set(SUPPORTED_EXTENSIONS).size, 'no extension is listed twice').toBe(SUPPORTED_EXTENSIONS.length);
  expect([...SUPPORTED_EXTENSIONS].sort()).toEqual([...backend].sort());
});
