import { useEffect, useRef, useState } from 'react';
import type { DesktopBridge } from '../lib/bridge';
import { formatBytes } from '../lib/format';
import { MODEL_TRADE, knownModels, lowMemory, otherModel } from '../lib/localModels';
import type { LegacyModel, LocalModelId, LocalModelOption } from '../types';

/**
 * The way to the other local model from setup: the smaller one while the
 * default is chosen, and back again once the smaller one is. One sentence
 * says what is traded, and the button's click both chooses it and starts its
 * download - nothing is downloaded before that.
 */
export function ModelSwitchOption({ chosen, models, disabled, onSwitch, actionsClassName, textClassName }: {
  chosen: LocalModelId;
  models?: LocalModelOption[];
  disabled: boolean;
  onSwitch(model: LocalModelId): void;
  actionsClassName?: string;
  textClassName?: string;
}) {
  const target = otherModel(chosen);
  const bytes = knownModels(models).find((model) => model.id === target)?.bytes;
  const size = bytes ? ` (${formatBytes(bytes)})` : '';
  const label = target === 'bonsai-2b' ? `Use the smaller model${size} instead` : `Use the recommended model${size} instead`;
  return <>
    <p className={textClassName}>{target === 'bonsai-2b' ? 'For a computer with little memory. ' : 'The model Intern recommends. '}{MODEL_TRADE[target]}</p>
    <div className={actionsClassName}>
      <button type="button" onClick={() => onSwitch(target)} disabled={disabled}>{label}</button>
    </div>
  </>;
}

/**
 * Said only when the webview reports less than about 8 GB of memory and the
 * larger model is chosen. It suggests; choosing stays with the person.
 */
export function LowMemoryHint({ chosen, where, className }: { chosen: LocalModelId; where: string; className?: string }) {
  if (chosen !== 'bonsai-4b' || !lowMemory()) return null;
  return <p className={className} role="note">This computer has less than 8 GB of memory. The smaller model {where} needs about half as much, and may suit it better.</p>;
}

function describeLegacyError(error: unknown): string {
  const code = typeof error === 'object' && error && 'code' in error && typeof error.code === 'string' ? error.code : typeof error === 'string' ? error : undefined;
  switch (code) {
    case 'LEGACY_MODEL_NOT_FOUND': return 'The previous model is no longer in Intern\'s model folder, so there is nothing to remove. (LEGACY_MODEL_NOT_FOUND)';
    case 'LEGACY_MODEL_MISMATCH': return 'The file in Intern\'s model folder is not exactly the previous model, so Intern left it alone. (LEGACY_MODEL_MISMATCH)';
    case 'LEGACY_MODEL_REMOVE_FAILED': return 'The previous model could not be removed. Close anything that may be using it and try again. (LEGACY_MODEL_REMOVE_FAILED)';
  }
  const message = error instanceof Error ? error.message : typeof error === 'object' && error && 'message' in error && typeof error.message === 'string' ? error.message : '';
  return message.trim() ? `The previous model could not be removed: ${message.trim()}` : 'The previous model could not be removed.';
}

/**
 * The model file an earlier version of Intern downloaded and this one no
 * longer reads. Intern never deletes it on its own: it is offered here, with
 * its size, and goes only when a person clicks.
 */
export function LegacyModelNotice({ bridge, className, actionsClassName }: { bridge: DesktopBridge; className?: string; actionsClassName?: string }) {
  const [legacy, setLegacy] = useState<LegacyModel | null>(null);
  const [removing, setRemoving] = useState(false);
  const [error, setError] = useState('');
  const [removed, setRemoved] = useState('');
  const mounted = useRef(true);
  useEffect(() => { mounted.current = true; return () => { mounted.current = false; }; }, []);
  useEffect(() => {
    let active = true;
    // A convenience: a bridge that cannot say leaves nothing to offer.
    void Promise.resolve().then(() => bridge.getLegacyModel())
      .then((found) => { if (active) setLegacy(found ?? null); })
      .catch(() => {});
    return () => { active = false; };
  }, [bridge]);
  const remove = async () => {
    if (!legacy || removing) return;
    setRemoving(true);
    setError('');
    try {
      await bridge.removeLegacyModel();
      if (!mounted.current) return;
      setRemoved(`Removed the previous model. ${formatBytes(legacy.bytes)} freed.`);
      setLegacy(null);
    } catch (cause) {
      if (!mounted.current) return;
      setError(describeLegacyError(cause));
      // The file may have gone, or turned out not to be the old model; ask again.
      try { const found = await bridge.getLegacyModel(); if (mounted.current) setLegacy(found ?? null); } catch { /* Keep what is shown. */ }
    } finally {
      if (mounted.current) setRemoving(false);
    }
  };
  if (!legacy && !error && !removed) return null;
  return <div className={className} role="group" aria-label="Previous model">
    {legacy && <>
      <p>An earlier version of Intern downloaded {legacy.file}. This version no longer uses it.</p>
      <div className={actionsClassName}>
        <button type="button" onClick={() => void remove()} disabled={removing}>{removing ? 'Removing the previous model…' : `Remove the previous model (${formatBytes(legacy.bytes)})`}</button>
      </div>
    </>}
    {error && <p className="form-error" role="alert">{error}</p>}
    {removed && <p role="status" aria-live="polite">{removed}</p>}
  </div>;
}
