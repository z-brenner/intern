import { Download, TriangleAlert } from 'lucide-react';
import { formatBytes } from '../lib/format';
import type { SetupState } from '../types';
import { DownloadProgress } from './DownloadProgress';
import { Icon } from './Icon';

interface SetupScreenProps {
  setup?: SetupState;
  busy: boolean;
  canChooseExisting: boolean;
  operationError?: string;
  onStart(): void;
  onCancel(): void;
  onChooseExisting(): void;
  /** Open Settings to configure a hosted model instead of the download. */
  onUseHostedModel?(): void;
}

export function SetupScreen({ setup, busy, canChooseExisting, operationError, onStart, onCancel, onChooseExisting, onUseHostedModel }: SetupScreenProps) {
  if (!setup) return <main className="setup-screen" aria-label="Intern setup"><section><p role="status" aria-label="Loading setup" aria-live="polite">Loading setup</p></section></main>;
  const failed = setup.state === 'failed';
  const downloading = setup.state === 'downloading';
  const canceled = setup.state === 'required' && setup.error === 'MODEL_DOWNLOAD_CANCELED';
  const resumable = setup.downloadedBytes > 0 && setup.downloadedBytes < setup.totalBytes;
  const showProgress = downloading || resumable || failed && setup.downloadedBytes > 0;
  const first = setup.tiers?.find((tier) => tier.tier === 'fast');
  const accurate = setup.tiers?.find((tier) => tier.tier === 'accurate');
  const size = setup.totalBytes > 0 ? formatBytes(setup.totalBytes) : '';
  // Downloaded in full but not yet ready: the files are being checked against
  // their pinned sizes and checksums. Said as its own step, so a bar at 100%
  // never looks stuck.
  const checking = downloading && setup.totalBytes > 0 && setup.downloadedBytes >= setup.totalBytes;
  const step = checking ? 1 : 0;
  const steps = [
    `Download the model${size ? ` (${size})` : ''}`,
    'Check the files against their published checksums',
    'Start naming documents',
  ];
  return <main className="setup-screen" aria-label="Intern setup"><section>
    <Icon icon={failed ? TriangleAlert : Download} /><h1>{failed ? 'Model setup needs attention' : 'Set up Intern'}</h1>
    {/*
      The size comes from the setup state, which the backend fills from
      model-manifest.json before the first byte is fetched, so it cannot drift
      from what will actually be downloaded. It used to be the hardcoded string
      "approximately 3.27 GB" - the combined size of a model and a vision
      projector, from a design this pipeline no longer uses. The first screen a
      new user saw overstated the download by more than two and a half times.

      The screen leads with what the download is for, in plain words; which
      models, by name, is a detail for the few who want it, below.
    */}
    <p className="setup-lead">Intern reads your documents with a model that runs on this computer. It needs a one-time download{size ? ` of ${size}` : ''} first. After that, naming documents works without an internet connection.</p>
    <ol className="setup-steps" aria-label="Model setup steps">
      {steps.map((text, index) => <li key={text} className={index < step ? 'done' : index === step && downloading ? 'current' : undefined} aria-current={index === step && downloading ? 'step' : undefined}>
        <span className="setup-step-mark" aria-hidden="true">{index + 1}</span>{text}{index === 1 && checking ? '…' : ''}
      </li>)}
    </ol>
    {operationError && <p className="setup-error" role="alert">{operationError}</p>}
    {canceled && !operationError && <p role="status" aria-label="Setup status" aria-live="polite">Download canceled. Your progress was saved.</p>}
    {showProgress && <DownloadProgress setup={setup} />}
    <div className="setup-actions">
      <button type="button" className="primary" onClick={onStart} disabled={downloading || busy}>{downloading ? checking ? 'Checking files…' : 'Downloading model…' : failed ? 'Try download again' : resumable ? 'Resume download' : 'Download model'}</button>
      {downloading && <button type="button" onClick={onCancel} disabled={busy}>Cancel setup</button>}
      <button type="button" onClick={onChooseExisting} disabled={downloading || busy || !canChooseExisting}>Choose existing model files</button>
    </div>
    <p className="setup-privacy">Your documents and filenames stay on this device. After setup, processing runs fully locally with no network dependency.</p>
    {/*
      The second model is part of what Download fetches, so it is said here,
      before anything is fetched - that nobody waits for it, and that until
      it arrives every name waits for a check, which is what the quick
      model's readings always do.
    */}
    <details className="setup-details">
      <summary>What gets downloaded</summary>
      <p>{first
        ? `The quick model, ${first.displayName}${size ? ` (${size})` : ''}, to start naming documents`
        : `${size ? `${size} of ` : ''}model files`}, or the matching files if you already have them on this computer.</p>
      {accurate && accurate.state !== 'ready' && <p>{accurate.displayName}{accurate.totalBytes > 0 ? ` (${formatBytes(accurate.totalBytes)})` : ''}, the accurate model, then follows in the background while you work. Until it arrives, every name waits for you to check it.</p>}
    </details>
    {/*
      The other way in. A hosted model needs no download, but it sends the
      text of every document to the service named in Settings - the one
      thing the paragraph above promises Intern does not do - so it is
      offered as the smaller, plainly-labeled choice, never the default.
    */}
    {onUseHostedModel && <div className="setup-alternative">
      <p>Prefer not to download? A hosted model, under your own API key, needs no download, but document text is sent to that service.</p>
      <button type="button" disabled={downloading || busy} onClick={onUseHostedModel}>Set up a hosted model</button>
    </div>}
  </section></main>;
}
