import { Ban, CalendarPlus, ChevronRight, ClipboardCopy, Ellipsis, ExternalLink, FileCheck2, FileText, FolderOpen, RefreshCw, RotateCcw, Trash2, X } from 'lucide-react';
import { useEffect, useId, useImperativeHandle, useLayoutEffect, useRef, useState } from 'react';
import type { KeyboardEvent, MouseEvent, Ref } from 'react';
import { ConfidenceMeter } from './ConfidenceMeter';
import { FileKindIcon } from './FileKindIcon';
import { Icon } from './Icon';
import { StatusCell } from './StatusCell';
import { itemActions, keptOriginal } from '../features/review/actions';
import type { ReviewInspectorHandle } from '../features/review/useReviewShortcuts';
import { filenameExtension, joinFilename, leadingDate, splitFilename, validateFilename, withLeadingDate } from '../lib/filenames';
import type { QueueItem } from '../types';

/** What the inspector says when a name has no date; the backend says the same in DATE_REQUIRED. */
export const DATE_REQUIRED_MESSAGE = 'Every rename needs a date. Start the filename with the document\'s date as YYYY-MM-DD.';

/**
 * What is wrong with an edited stem, checked as the person types: the
 * backend's own rules for the whole name, and one it cannot see - a stem that
 * already ends in the extension, as a name pasted whole from Explorer does,
 * would be filed as "Lease.pdf.pdf". An empty stem is left alone until
 * Approve, so clearing the field to retype it is not an error.
 */
function stemProblem(stem: string, extension: string | undefined, sourceExtension: string | undefined): string | undefined {
  if (!stem.trim()) return undefined;
  if (extension !== undefined && stem.trim().toLowerCase().endsWith(`.${extension.toLowerCase()}`)) {
    return `Leave \u201c.${extension}\u201d off: Intern adds the extension itself.`;
  }
  return validateFilename(joinFilename(stem, extension), sourceExtension);
}

/** Line breaks have no place in a filename; one pasted in becomes a space, so the words either side stay apart. */
const withoutLineBreaks = (value: string) => value.replace(/[\r\n]+/g, ' ');

/**
 * The pipeline joins several parties into one string with semicolons. Shown as
 * one run of text it reads like a database field; split back out, each party
 * is its own quotation and a reader can check them one at a time.
 */
function quotations(value?: string): string[] {
  return (value ?? '').split(';').map((part) => part.trim()).filter((part) => part.length > 0);
}

/**
 * A click that decides the item - approve, keep, remove. Review moves on as
 * soon as the backend answers, which is quicker than a double-click, and the
 * next item's button is enabled in the same place in the commit that shows
 * it. The second click of a double-click therefore landed on a document
 * nobody had looked at and decided that too. Only a click's first press
 * decides; one from the keyboard has no count (detail 0) and is a press.
 */
const decides = (run: () => void) => (event: MouseEvent<HTMLButtonElement>) => { if (event.detail <= 1) run(); };

/** A key's name beside the button it presses; read out through aria-keyshortcuts instead. */
const Shortcut = ({ keys }: { keys: string }) => <kbd className="shortcut" aria-hidden="true">{keys}</kbd>;

interface Props {
  item: QueueItem;
  drawer: boolean;
  busy?: boolean;
  /** How the keyboard shortcuts reach the panel. */
  ref?: Ref<ReviewInspectorHandle>;
  /** Where this item stands among those still to decide: "2 of 5". */
  position?: { index?: number; total: number };
  /** Go to the next item still to decide, when there is another. */
  onNext?(): void;
  onClose(): void;
  onApprove(filename: string, description: string): void;
  onKeep(): void;
  onCancel(): void;
  onRetry(): void;
  onReanalyze(): void;
  /** `confirmed`: the person said they resolved a parked item's files themselves. */
  onRemove(confirmed: boolean): void;
  onUndo(): void;
  onOpen(): void;
  onReveal(): void;
}
export function ReviewInspector({ item, drawer, busy, ref, position, onNext, onClose, onApprove, onKeep, onCancel, onRetry, onReanalyze, onRemove, onUndo, onOpen, onReveal }: Props) {
  // Exactly what the backend accepts for this item. Retry on an ordinary
  // review item, and nothing at all for a ready or waiting one, were the
  // two ways this panel used to be wrong (FRONTEND_UX-3, FRONTEND_UX-4).
  const actions = itemActions(item);
  // The extension is the source's and the backend refuses any other, so it
  // is not part of what can be edited: the field holds the name before it,
  // and the extension sits after the field, locked.
  const sourceExtension = filenameExtension(item.originalFilename);
  const { stem: proposedStem, extension } = splitFilename(item.proposedFilename ?? '', sourceExtension);
  // The name and description being edited belong to one item, and to one
  // revision of its proposal. The panel stays mounted as review moves on, and
  // they used to be put back to the new item's in a passive effect: a task
  // after the commit that showed the new item. A key pressed in between - a
  // held or quick second Ctrl+Enter, or Enter in the name - approved the item
  // on screen under the name and description of the one just decided. They
  // are put back while rendering instead, so no commit, and no handler or
  // shortcut taken from one, ever pairs an item with another's draft.
  const draftFor = `${item.id}\n${item.proposalRevision ?? ''}`;
  const [draft, setDraft] = useState({ for: draftFor, stem: proposedStem, description: item.description ?? '' });
  const [error, setError] = useState('');
  const [moreOpen, setMoreOpen] = useState(false);
  const [confirmingRemove, setConfirmingRemove] = useState(false);
  if (draft.for !== draftFor) {
    setDraft({ for: draftFor, stem: proposedStem, description: item.description ?? '' });
    setError(''); setMoreOpen(false); setConfirmingRemove(false);
  }
  const { stem, description } = draft;
  const setStem = (value: string) => setDraft((current) => ({ ...current, stem: value }));
  const setDescription = (value: string) => setDraft((current) => ({ ...current, description: value }));
  const inspectorRef = useRef<HTMLElement>(null);
  const filenameRef = useRef<HTMLTextAreaElement>(null);
  const confirmRef = useRef<HTMLButtonElement>(null);
  const headingRef = useRef<HTMLHeadingElement>(null);
  const fieldId = useId();
  useEffect(() => { if (confirmingRemove) confirmRef.current?.focus(); }, [confirmingRemove]);
  // The whole name, always: a single-line field showed about half of a
  // typical proposal, and the caret only its tail. The field grows to fit
  // instead. Chromium sizes it from its content (field-sizing, in app.css);
  // this is for an engine that cannot.
  useLayoutEffect(() => {
    const field = filenameRef.current;
    if (!field || typeof CSS === 'undefined' || CSS.supports?.('field-sizing', 'content')) return;
    field.style.height = 'auto';
    field.style.height = `${field.scrollHeight}px`;
  }, [stem]);
  useEffect(() => {
    if (!drawer) return;
    const filenameInput = filenameRef.current;
    if (filenameInput && !filenameInput.disabled) filenameInput.focus();
    else inspectorRef.current?.querySelector<HTMLElement>('button:not(:disabled)')?.focus();
  }, [drawer, item.id]);
  const dated = leadingDate(stem) !== undefined;
  // Every check the backend makes, before the round trip, so a refusal names
  // the character at fault next to the field instead of arriving as jargon.
  const submit = (nextStem: string) => {
    if (!nextStem.trim()) { setError('Filename is required'); return; }
    const problem = stemProblem(nextStem, extension, sourceExtension);
    if (problem) { setError(problem); return; }
    const name = joinFilename(nextStem, extension);
    if (!leadingDate(name)) { setError(DATE_REQUIRED_MESSAGE); return; }
    onApprove(name, description);
  };
  const approve = () => submit(stem);
  const editStem = (value: string) => { setStem(value); setError(stemProblem(value, extension, sourceExtension) ?? ''); };
  // The model's date, when Intern could not find it written in the document
  // and so left it out of the name. Offered until the name carries a date.
  const suggestedDate = item.suggestedDate && !dated ? item.suggestedDate : undefined;
  const useSuggestedDate = () => editStem(withLeadingDate(stem, suggestedDate!));
  const acceptSuggestedDate = () => submit(withLeadingDate(stem, suggestedDate!));
  // When the name has no date and the model offered none worth showing, the
  // document's own dates - and, last, the file's - are one click each.
  const dateChoices = !dated ? (item.datesInDocument ?? []) : [];
  const fileDate = !dated ? item.fileModifiedDate : undefined;
  const chooseDate = (date: string) => editStem(withLeadingDate(stem, date));
  // Enter files the name, as it does in any one-line field; the IME's Enter
  // that ends a composition is left to the IME. A held Enter repeats, and
  // after the first press the caret is in the next item's name: only the
  // press itself files a name.
  const onFilenameKeyDown = (event: KeyboardEvent<HTMLTextAreaElement>) => {
    if (event.key !== 'Enter' || event.altKey || event.nativeEvent.isComposing) return;
    event.preventDefault();
    if (!busy && !event.repeat) approve();
  };
  const onKeyDown = (event: KeyboardEvent<HTMLElement>) => {
    // The same for Enter held on a button, which presses it again with every
    // repeat - by then the next item's button, in the same place.
    if (event.repeat && event.key === 'Enter' && event.target instanceof HTMLButtonElement) { event.preventDefault(); return; }
    if (!drawer) return;
    if (event.key === 'Escape') { event.preventDefault(); onClose(); return; }
    if (event.key !== 'Tab') return;
    const focusable = [...(inspectorRef.current?.querySelectorAll<HTMLElement>('button:not(:disabled), input:not(:disabled), textarea:not(:disabled), select:not(:disabled), [tabindex]:not([tabindex="-1"])') ?? [])];
    const first = focusable.at(0);
    const last = focusable.at(-1);
    if (!first || !last) return;
    if (event.shiftKey && document.activeElement === first) { event.preventDefault(); last.focus(); }
    else if (!event.shiftKey && document.activeElement === last) { event.preventDefault(); first.focus(); }
  };
  // The name can be edited only where it can be approved: a parked item's
  // files must be checked first, and an item flagged before analysis has no
  // proposal for the backend to approve.
  const editable = actions.approve;
  const omitted = item.omittedParties ?? [];
  const completed = item.status === 'completed';
  const filedAs = item.filedName ?? item.proposedFilename;
  const remove = actions.remove;
  // A parked item's files were left part-way through a rename, so removing it
  // waits for the person to say they put them right; anything else goes at
  // once from the menu, and asks first only from the Delete key.
  const chooseRemove = () => { setMoreOpen(false); if (remove?.resolvedFiles) setConfirmingRemove(true); else onRemove(false); };
  const cancelRemove = () => { setConfirmingRemove(false); queueMicrotask(() => inspectorRef.current?.querySelector<HTMLElement>('.inspector-actions button:not(:disabled)')?.focus()); };
  // The shortcuts act only where the matching button is offered and enabled,
  // so a key can never do what a click could not.
  useImperativeHandle(ref, () => ({
    approve: () => { if (actions.approve && !busy && !confirmingRemove) approve(); },
    keep: () => { if (actions.keep && !busy && !confirmingRemove) onKeep(); },
    requestRemove: () => { if (remove && !busy) { setMoreOpen(false); setConfirmingRemove(true); } },
    focus: (target) => {
      if (target === 'filename' && filenameRef.current) filenameRef.current.focus();
      else headingRef.current?.focus();
    },
  }));
  // Each row is a claim the proposed filename makes, paired with the text in
  // the document that supports it. A bare definition list read as metadata
  // about the file; attributing a quotation to the part of the name it
  // justifies is what makes it evidence a person can check.
  const evidence = [
    { label: 'Date', quotes: quotations(item.evidence?.date) },
    { label: 'Type', quotes: quotations(item.evidence?.type) },
    { label: 'Parties', quotes: quotations(item.evidence?.parties) },
  ].filter((entry) => entry.quotes.length > 0);
  return <aside ref={inspectorRef} className="inspector" aria-label="Review item" role={drawer ? 'dialog' : 'complementary'} aria-modal={drawer || undefined} onKeyDown={onKeyDown}>
    {/* A filed document's name is no longer its "current" one; it said so anyway (FRONTEND_UX-8). */}
    {/* Focusable so selecting a row can bring a keyboard user here, past the rest of the queue. */}
    <div className="inspector-title"><h2 ref={headingRef} tabIndex={-1}>{completed ? 'Filed document' : 'Review item'}</h2><button type="button" className="icon-button" onClick={onClose} aria-label="Close review"><Icon icon={X} /></button></div>
    {position && position.total > 0 && <div className="review-position">
      <span>{position.index ? `${position.index} of ${position.total} to decide` : `${position.total} to decide`}</span>
      {onNext && <button type="button" className="link-button" onClick={onNext}>Next undecided<Icon icon={ChevronRight} /></button>}
    </div>}
    <div className="source-file">
      <p className="field-label">{completed ? 'Original name' : 'Current name'}</p>
      <p className="selected-file"><FileKindIcon filename={item.originalFilename} />{item.originalFilename}</p>
      {item.status === 'ready' && item.approved && <p className="filed-outcome">Approved. It will be renamed when the queue is free.</p>}
      {completed && (keptOriginal(item)
        ? <p className="filed-outcome">Kept its original name</p>
        : filedAs && <p className="filed-outcome">Renamed to <strong>{filedAs}</strong></p>)}
      <StatusCell item={item} />
      {/*
        The reviewer is asked to name a document they could not open from
        here, and once it was filed they could not find it. Before filing
        these reach the file as it arrived; after, the filed copy.
      */}
      {(actions.open || actions.reveal) && <div className="document-actions" role="group" aria-label="Document">
        {actions.open && <button type="button" onClick={onOpen}><Icon icon={ExternalLink} />Open</button>}
        {actions.reveal && <button type="button" onClick={onReveal}><Icon icon={FolderOpen} />Show in folder</button>}
      </div>}
    </div>
    {editable && <section className="proposal">
      <h3>Proposed rename</h3>
      <label className="filename-label">Filename
        <span className="filename-field">
          <textarea ref={filenameRef} aria-label="Filename" className="filename-input" rows={1} spellCheck={false} autoComplete="off" value={stem}
            onChange={(event) => editStem(withoutLineBreaks(event.target.value))} onKeyDown={onFilenameKeyDown}
            aria-invalid={Boolean(error)} aria-describedby={[extension !== undefined ? `${fieldId}-extension` : '', error ? `${fieldId}-error` : ''].filter(Boolean).join(' ') || undefined} />
          {extension !== undefined && <span className="filename-extension" title="The extension stays as it is" aria-hidden="true">.{extension}</span>}
        </span>
      </label>
      {extension !== undefined && <span className="sr-only" id={`${fieldId}-extension`}>The name ends in .{extension}, which cannot change.</span>}
      {error && <p className="form-error" role="alert" id={`${fieldId}-error`}>{error}</p>}
      {!error && stem.trim() && !dated && !suggestedDate && <p className="check-hint date-hint">{DATE_REQUIRED_MESSAGE}</p>}
      {/*
        A name that differs from the evidence under it looks like a mistake
        unless the reason is on screen: the reviewer's own spelling, learned
        from their edits, was applied. Said here, beside the name, with the
        document's words kept in the evidence below.
      */}
      {item.houseRules && item.houseRules.length > 0 && <p className="check-hint house-style-hint" role="note" aria-label="Learned spellings applied">
        Uses your spelling: {item.houseRules.map((rule, index) => <span key={`${rule.kind}-${rule.from}`}>{index > 0 ? '; ' : ''}<q>{rule.from}</q> written as <strong>{rule.to}</strong></span>)}. Change or forget it under Settings.
      </p>}
      {/*
        The same for a party the evidence names and the name does not: the
        person's own organisation, left out so the name says who the
        document is with.
      */}
      {omitted.length > 0 && <p className="check-hint own-names-hint" role="note" aria-label="Filed by the other side">
        Filed by the other side: {omitted.map((name, index) => <span key={`${index}-${name}`}>{index === 0 ? '' : index === omitted.length - 1 ? ' and ' : ', '}<q>{name}</q></span>)} {omitted.length === 1 ? 'is your organisation, so it is' : 'are your organisation, so they are'} left out of the name. Change this under Settings.
      </p>}
      {/*
        The gate above would otherwise be a dead end for the commonest review:
        the model read a date the document never states verbatim, so the name
        has none. The model's reading is shown, said to be unverified, and
        accepted in one click - into the field, or straight through to the
        rename.
      */}
      {(dateChoices.length > 0 || fileDate) && <div className="date-choices" role="group" aria-label="Dates to choose from">
        <p className="field-label">{dateChoices.length > 0 ? 'Dates the document states' : 'The document states no date'}</p>
        <div className="chips">
          {dateChoices.map((date) => <button type="button" key={date} className="chip" disabled={busy} onClick={() => chooseDate(date)}>{date}</button>)}
          {fileDate && <button type="button" className="chip chip--file" disabled={busy} title="The file's last-modified date on this computer, not a date from the document" onClick={() => chooseDate(fileDate)}>{fileDate} · file date</button>}
        </div>
      </div>}
      {suggestedDate && <div className="suggestion" role="group" aria-label="Suggested date">
        <p><Icon icon={CalendarPlus} />The model read <strong>{suggestedDate}</strong> as the document's date, but Intern could not find it written in the document. Every rename needs a date.</p>
        <div className="suggestion-actions">
          <button type="button" disabled={busy} onClick={useSuggestedDate}>Use this date</button>
          <button type="button" className="primary" disabled={busy} onClick={decides(acceptSuggestedDate)}>Use date &amp; rename</button>
        </div>
      </div>}
      {item.confidence !== undefined && <p className={`confidence-readout confidence ${item.status}`}><span className="field-label">Confidence</span><ConfidenceMeter value={item.confidence} status={item.status} variant="panel" /></p>}
      <label>Description<textarea value={description} onChange={(event) => setDescription(event.target.value)} /></label>
    </section>}
    {/*
      A renamed file becomes `completed`, which made `editable` false and took
      the description off screen with it. The sentence describing the document
      is the other half of what Intern produces, and it was unreachable the
      moment it was most useful. Read-only here because the proposal is settled,
      with a copy action so it can go somewhere else.
    */}
    {!editable && item.description && <section><h3>Description</h3><p className="settled-description">{item.description}</p>
      <button type="button" className="copy-description" onClick={() => void navigator.clipboard?.writeText(item.description ?? '')}><Icon icon={ClipboardCopy} />Copy description</button></section>}
    {evidence.length > 0 && <section className="evidence"><h3>Evidence</h3>
      <p className="section-lead">Text found in the document that supports this name.</p>
      <dl className="evidence-list">{evidence.map(({ label, quotes }) => <div className="evidence-item" key={label}>
        <dt className="field-label">{label}</dt>
        <dd>{quotes.map((quote) => <q key={quote}>{quote}</q>)}</dd>
      </div>)}</dl></section>}
    {/* Settled, a reason is no longer why the item waits - "already had this name" is a note about what happened. */}
    {item.reason && <section className={`note${item.status === 'failed' ? ' note--failed' : completed ? '' : ' note--review'}`}><h3>{item.status === 'failed' ? 'Failure details' : completed ? 'Note' : 'Reason for review'}</h3><p>{item.reason}</p>
      {/*
        The sentence says "a document that was filed already"; this says
        which one, so the person can open it and compare rather than take
        Intern's word for it.
      */}
      {item.nearDuplicateOf && <p className="near-duplicate" aria-label="Filed already as">Filed already as <q>{item.nearDuplicateOf}</q>.</p>}
    </section>}
    <div className="inspector-actions">
      {confirmingRemove && remove ? <div className="remove-confirm" role="group" aria-label="Confirm removal" onKeyDown={(event) => {
        // Escape takes back the question, not the whole drawer.
        if (event.key === 'Escape') { event.preventDefault(); event.stopPropagation(); cancelRemove(); }
      }}>
        <p>{remove.resolvedFiles
          ? 'This rename stopped part-way, so its files may need putting right by hand. Remove it only once you have: Intern will not touch them again.'
          : <>Remove <q>{item.originalFilename}</q> from the queue? The file itself stays where it is.</>}</p>
        <button ref={confirmRef} type="button" className="primary" disabled={busy} onClick={decides(() => { setConfirmingRemove(false); onRemove(remove.resolvedFiles); })}>{remove.resolvedFiles ? 'I have resolved the files myself' : remove.label}</button>
        <button type="button" disabled={busy} onClick={cancelRemove}>Cancel</button>
      </div> : <>
        {actions.approve && <button type="button" className="primary" disabled={busy} onClick={decides(approve)} aria-keyshortcuts="Control+Enter"><Icon icon={FileCheck2} />{item.status === 'ready' ? 'Apply rename' : 'Approve & rename'}<Shortcut keys="Ctrl+Enter" /></button>}
        {actions.retry?.primary && <button type="button" className="primary" disabled={busy} onClick={onRetry}><Icon icon={RotateCcw} />{actions.retry.label}</button>}
        {actions.keep && <>
          <button type="button" className="secondary-action" disabled={busy} onClick={decides(onKeep)} aria-keyshortcuts="Alt+K"><Icon icon={FileText} />Keep original<Shortcut keys="Alt+K" /></button>
          <button type="button" className="icon-button more-actions" disabled={busy} aria-label="More review actions" aria-expanded={moreOpen} onClick={() => setMoreOpen(!moreOpen)}><Icon icon={Ellipsis} /></button>
          {moreOpen && <div className="review-menu" role="group" aria-label="More review actions">
            {actions.retry && <button type="button" disabled={busy} onClick={() => { setMoreOpen(false); onRetry(); }}><Icon icon={RotateCcw} />{actions.retry.label}</button>}
            {actions.reanalyze && <button type="button" disabled={busy} onClick={() => { setMoreOpen(false); onReanalyze(); }}><Icon icon={RefreshCw} />Analyze again</button>}
            {remove && <button type="button" disabled={busy} onClick={decides(chooseRemove)} aria-keyshortcuts="Delete"><Icon icon={Trash2} />{remove.label}<Shortcut keys="Del" /></button>}
          </div>}
        </>}
        {/* Parked, failed or waiting: no keep and no menu, so the rest stand on their own. */}
        {!actions.keep && actions.retry && !actions.retry.primary && <button type="button" disabled={busy} onClick={onRetry}><Icon icon={RotateCcw} />{actions.retry.label}</button>}
        {!actions.keep && remove && <button type="button" className={actions.retry?.primary ? 'secondary-action' : undefined} disabled={busy} onClick={decides(chooseRemove)} aria-keyshortcuts="Delete"><Icon icon={Trash2} />{remove.label}<Shortcut keys="Del" /></button>}
        {actions.cancel && <button type="button" disabled={busy} onClick={onCancel}><Icon icon={Ban} />Cancel processing</button>}
        {actions.undo && <button type="button" disabled={busy} onClick={onUndo}><Icon icon={RotateCcw} />Undo</button>}
      </>}
    </div>
  </aside>;
}
