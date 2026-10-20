import type { LocalModelId, LocalModelOption } from '../types';

/** Exact size of the default local model's file, Bonsai 4B, as model-manifest.json pins it. */
export const PINNED_MODEL_BYTES = 1_146_465_120;
/** Exact size of the smaller local model's file, Bonsai 2B. */
export const PINNED_SMALL_MODEL_BYTES = 517_312_992;

export const DEFAULT_LOCAL_MODEL: LocalModelId = 'bonsai-4b';

/**
 * The two local models as src-tauri/resources/model-manifest.json pins them.
 * The backend's `local_models` is the authority; this is what the screens
 * fall back to when it cannot be read, so an error message can still name
 * the files it accepts. A test keeps it equal to the manifest.
 */
export const LOCAL_MODELS: ReadonlyArray<Pick<LocalModelOption, 'id' | 'label' | 'file' | 'bytes'>> = [
  { id: 'bonsai-4b', label: 'Bonsai 4B', file: 'Ternary-Bonsai-2-4B-PQ2_0.gguf', bytes: PINNED_MODEL_BYTES },
  { id: 'bonsai-2b', label: 'Bonsai 2B', file: 'Ternary-Bonsai-2-2B-PQ2_0.gguf', bytes: PINNED_SMALL_MODEL_BYTES },
];

/**
 * What a person weighs between the two, from Intern's own evaluation
 * (docs/bonsai-native-windows-evaluation.md: twelve held-out documents on a
 * 4 GiB, two-processor Windows machine). 4B named 11 of the 12 exactly
 * right, at a median 54 s a document and 1.96 GB of memory at its peak; 2B
 * named 8, at 22 s and 0.94 GB. The smaller one is a trade, not an upgrade,
 * so it is only ever the person's choice.
 */
export const MODEL_TRADE: Record<LocalModelId, string> = {
  'bonsai-4b': 'It names more documents correctly (11 of 12 in Intern\'s evaluation, against 8 of 12), but is a larger download, takes more than twice as long per document and needs about twice the memory.',
  'bonsai-2b': 'It is a smaller download, more than twice as fast and needs about half the memory, but names fewer documents correctly: 8 of 12 in Intern\'s evaluation, against 11 of 12.',
};

/** The backend's list, or the pinned facts when it has not answered. */
export function knownModels(models: LocalModelOption[] | undefined): ReadonlyArray<Pick<LocalModelOption, 'id' | 'label' | 'file' | 'bytes'>> {
  return models?.length ? models : LOCAL_MODELS;
}

/** The other local model than `id`. */
export const otherModel = (id: LocalModelId): LocalModelId => id === 'bonsai-4b' ? 'bonsai-2b' : 'bonsai-4b';

/**
 * Whether this computer has less than about 8 GB of memory, as far as the
 * webview will say. `navigator.deviceMemory` is rounded down to a power of
 * two and capped, so 4 means somewhere under 8 GB. It is a hint for the
 * person and nothing more; where the browser does not report it, nothing is
 * shown.
 */
export function lowMemory(): boolean {
  const memory = typeof navigator === 'undefined' ? undefined : (navigator as Navigator & { deviceMemory?: unknown }).deviceMemory;
  return typeof memory === 'number' && memory > 0 && memory <= 4;
}
