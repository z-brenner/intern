import type { AppSettings, ModelPreference, ModelTier, SetupState, TierSetup } from '../../types';
import { formatBytes } from '../../lib/format';
import { DownloadProgress } from '../../components/DownloadProgress';
import { describeSetupError } from '../setup/useModelSetup';

/** What each choice does, in a line. The names come from the backend when it says them. */
function choices(fast: string, accurate: string): { value: ModelPreference; label: string; hint: string }[] {
  return [
    { value: 'automatic', label: 'Automatic (recommended)', hint: `${accurate} when this computer has the memory to spare, ${fast} when it does not.` },
    { value: 'accurate', label: 'Most accurate', hint: `${accurate} for every document. About twice as slow.` },
    { value: 'fast', label: 'Fastest', hint: `${fast} for every document. Quicker and lighter, a little less accurate.` },
  ];
}

interface LocalModelProps {
  next: AppSettings;
  onChange(preference: ModelPreference): void;
  setup?: SetupState;
  busy: boolean;
  onStartTier(tier: ModelTier): void;
  onPauseTier(tier: ModelTier): void;
}

/**
 * Which local model reads documents, and where each one is. Calm and short:
 * three choices, and a line per model with the one action it needs, if any.
 */
export function LocalModel({ next, onChange, setup, busy, onStartTier, onPauseTier }: LocalModelProps) {
  const tiers = setup?.tiers ?? [];
  const name = (tier: ModelTier, fallback: string) => tiers.find((entry) => entry.tier === tier)?.displayName || fallback;
  const preference = next.modelPreference ?? 'automatic';
  return <div className="local-model" role="group" aria-label="Local model">
    {choices(name('fast', 'The fast model'), name('accurate', 'The accurate model')).map((choice) => <label key={choice.value} className="check-label">
      <input type="radio" name="model-preference" value={choice.value} checked={preference === choice.value} onChange={() => onChange(choice.value)} />
      <span>{choice.label}<span className="check-hint">{choice.hint}</span></span>
    </label>)}
    {tiers.length > 0 && <ul className="local-model-tiers" aria-label="Local models">
      {tiers.map((tier) => <TierLine key={tier.tier} tier={tier} busy={busy} onStart={() => onStartTier(tier.tier)} onPause={() => onPauseTier(tier.tier)} />)}
    </ul>}
  </div>;
}

function TierLine({ tier, busy, onStart, onPause }: { tier: TierSetup; busy: boolean; onStart(): void; onPause(): void }) {
  const size = tier.totalBytes > 0 ? ` · ${formatBytes(tier.totalBytes)}` : '';
  const started = tier.downloadedBytes > 0 && tier.downloadedBytes < tier.totalBytes;
  return <li aria-label={tier.displayName}>
    <span>{tier.displayName}{size} · {tierStatus(tier)}</span>
    {tier.state === 'downloading' && <DownloadProgress setup={tier} label={`${tier.displayName} download`} />}
    {tier.state === 'failed' && tier.error && <p className="form-error">{tier.tier !== 'fast' && tier.error === 'MODEL_SELF_TEST_FAILED'
      ? 'This model did not pass its check on this computer, so the fast model reads instead.'
      : describeSetupError(tier.error)}</p>}
    {tier.tier !== 'fast' && <>
      {tier.state === 'downloading' && <button type="button" disabled={busy} onClick={onPause}>Pause download</button>}
      {tier.state === 'required' && <button type="button" disabled={busy} onClick={onStart}>{started ? 'Resume download' : 'Download'}</button>}
      {tier.state === 'failed' && <button type="button" disabled={busy} onClick={onStart}>Download again</button>}
    </>}
  </li>;
}

function tierStatus(tier: TierSetup): string {
  switch (tier.state) {
    case 'ready': return 'Ready';
    case 'downloading': return 'Downloading';
    case 'failed': return 'Not usable';
    case 'required': return tier.downloadedBytes > 0 ? 'Paused' : 'Not downloaded';
  }
}

/**
 * The main window's one line about the accurate model while it arrives:
 * "Improving accuracy: downloading Bonsai 2 4B, 41%". Nothing once it is
 * ready, and nothing that asks for a decision.
 */
export function accuracyStatus(setup: SetupState | undefined): string | undefined {
  const accurate = setup?.tiers?.find((tier) => tier.tier === 'accurate');
  if (!accurate || accurate.state !== 'downloading') return undefined;
  const percent = accurate.totalBytes > 0 ? Math.min(100, Math.floor((accurate.downloadedBytes / accurate.totalBytes) * 100)) : undefined;
  return `Improving accuracy: downloading ${accurate.displayName}${percent === undefined ? '' : `, ${percent}%`}`;
}
