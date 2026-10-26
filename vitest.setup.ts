import '@testing-library/jest-dom/vitest';
import { cleanup } from '@testing-library/react';
import { afterEach } from 'vitest';

afterEach(() => {
  cleanup();
  // Each test starts as a first launch: the Details panel remembers being
  // left open, and one test's choice must not open it for the next.
  try { window.localStorage?.clear(); } catch { /* No storage in this environment. */ }
});
