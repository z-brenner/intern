import { Ban, CalendarPlus, CircleAlert, ChevronRight, ClipboardCopy, Ellipsis, ExternalLink, FileCheck2, FileText, FolderOpen, RefreshCw, RotateCcw, Trash2, X } from 'lucide-react';
import { useEffect, useId, useImperativeHandle, useLayoutEffect, useRef, useState } from 'react';
import type { FocusEvent, KeyboardEvent, MouseEvent, Ref } from 'react';
import { ConfidenceMeter } from './ConfidenceMeter';
import { FileKindIcon } from './FileKindIcon';
import { Icon } from './Icon';
import { StatusCell } from './StatusCell';
import { isUnreadable, itemActions, keptOriginal, withoutReviewPrefix } from '../features/review/actions';
import { itemFlaggedFields, itemReasonCodes, nameParts, partyGiven, withParty } from '../features/queue/check';
import type { ReasonField } from '../lib/reasons';
import type { ReviewInspectorHandle } from '../features/review/useReviewShortcuts';
import { filenameExtension, joinFilename, leadingDate, splitFilename, validateFilename, withLeadingDate } from '../lib/filenames';
import type { DesktopBridge } from '../lib/bridge';
import type { HouseRule, LearnedRule, ModelReading, QueueItem } from '../types';
import { UseOnItsOwnQuestion, failureCode, spellingLabel } from './UseOnItsOwn';
import { afterAPress } from '../lib/lastInput';

/**
 * Below this the model's own figure already says it was unsure, and is by
 * itself why the engine sends a reading to review (READY_CONFIDENCE in
 * validate.rs, which a test holds this to). At or above it, a reading
 * flagged as unsure was flagged by what Intern measured, and the figure
 * needs a word of explanation.
 */
export const SELF_RATED_UNSURE = 0.6;

/** Where "Analysis details" being open or closed is remembered, between documents and between launches. */
export const ANALYSIS_DETAILS_KEY = 'intern.analysisDetails';

/**
 * Closed unless a person left it open. Storage can be missing or refuse to
 * be read (a locked-down profile); that is a closed section, not an error.
 */
function readAnalysisDetailsPreference(): boolean {
  try { return window.localStorage.getItem(ANALYSIS_DETAILS_KEY) === 'open'; } catch { return false; }
}

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

/** Whether a learned spelling is the one a name used: the same kind, word and spelling, exactly. */
const sameSpelling = (learned: LearnedRule, used: HouseRule) => learned.kind === used.kind && learned.from === used.from && learned.to === used.to;

/** Why a spelling could not be used on its own, in a sentence. */
function spellingFailure(failure: unknown): string {
  const code = failureCode(failure);
  if (code === 'RULE_CHANGED') return 'This spelling changed - look at it again.';
  if (code === 'RULE_NOT_FOUND') return 'This spelling was forgotten - look at the name again.';
  const message = failure instanceof Error ? failure.message : typeof failure === 'object' && failure && 'message' in failure && typeof failure.message === 'string' ? failure.message : '';
  return message.trim() ? `The spelling could not be used on its own: ${message.trim()}${code ? ` (${code})` : ''}` : 'The spelling could not be used on its own.';
}

interface Props {
  item: QueueItem;
  /** For the Details toggle's aria-controls. */
  id?: string;
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
  /**
   * The learned spellings, for a document held because a spelling nobody
   * has checked changed its name (LEARNED_SPELLING_UNCONFIRMED): each such
   * spelling the name used is offered "Use on its own" here, after the
   * question Settings asks. Without it, only the pointer to Settings.
   */
  spellings?: Pick<DesktopBridge, 'houseRulesList' | 'houseRuleCheck'>;
  /** A spelling was used on its own from here: the queue may have changed. */
  onSpellingChecked?(): void;
}
/** The name and description being edited, for one item and one revision of its proposal (`for`). */
interface Draft { for: string; stem: string; description: string; edited: boolean }

export function ReviewInspector({ item, id, drawer, busy, ref, position, onNext, onClose, onApprove, onKeep, onCancel, onRetry, onReanalyze, onRemove, onUndo, onOpen, onReveal, spellings, onSpellingChecked }: Props) {
  // Exactly what the backend accepts for this item. Retry on an ordinary
  // review item, and nothing at all for a ready or waiting one, were the
  // two ways this panel used to be wrong (FRONTEND_UX-3, FRONTEND_UX-4).
  const actions = itemActions(item);
  // The extension is the source's and the backend refuses any other, so it
  // is not part of what can be edited: the field holds the name before it,
  // and the extension sits after the field, locked.
  const sourceExtension = filenameExtension(item.originalFilename);
  // A document Intern could not read has no proposal; the person names it,
  // starting from the name it arrived with - without the NEEDS REVIEW prefix
  // Intern gave the file, which the filed name leaves behind.
  const unreadable = isUnreadable(item) && item.proposedFilename === undefined;
  const startingName = item.proposedFilename ?? (unreadable ? withoutReviewPrefix(item.originalFilename) : '');
  const { stem: proposedStem, extension } = splitFilename(startingName, sourceExtension);
  // The name and description being edited belong to one item, and to one
  // revision of its proposal. The panel stays mounted as review moves on, and
  // they used to be put back to the new item's in a passive effect: a task
  // after the commit that showed the new item. A key pressed in between - a
  // held or quick second Ctrl+Enter, or Enter in the name - approved the item
  // on screen under the name and description of the one just decided. They
  // are put back while rendering instead, so no commit, and no handler or
  // shortcut taken from one, ever pairs an item with another's draft.
  const draftFor = `${item.id}\n${item.proposalRevision ?? ''}`;
  const [draft, setDraft] = useState<Draft>({ for: draftFor, stem: proposedStem, description: item.description ?? '', edited: false });
  // What was typed for an item left undecided - Alt+N, Next undecided, or
  // another row - is set aside under the same item and revision, and is in
  // the fields again when review comes back to it. It used to be thrown
  // away, and Alt+N is pressed from inside the name, where corrections are
  // typed. A new reading is a new revision: its own proposal is shown, not
  // an edit of the one before.
  const [setAside, setSetAside] = useState<ReadonlyMap<string, Draft>>(() => new Map());
  const [error, setError] = useState('');
  // "Use on its own" for a held document's spelling: which one is being
  // asked about, and what came of the last answer.
  const [askingSpelling, setAskingSpelling] = useState<string>();
  const [spellingNote, setSpellingNote] = useState<{ failed: boolean; text: string }>();
  const [moreOpen, setMoreOpen] = useState(false);
  const [confirmingRemove, setConfirmingRemove] = useState(false);
  // "Analysis details" - the evidence, the model's own figure, who read it -
  // stays as the person left it. It used to close itself for every document,
  // so checking each name against its evidence cost Tab, Tab, Enter (or a
  // click) per document, on a queue where nearly every document is reviewed.
  // Closed until someone opens it; remembered for the next launch too.
  const [analysisOpen, setAnalysisOpen] = useState(readAnalysisDetailsPreference);
  const rememberAnalysis = (open: boolean) => {
    setAnalysisOpen(open);
    try { window.localStorage.setItem(ANALYSIS_DETAILS_KEY, open ? 'open' : 'closed'); } catch { /* Not remembered; it still opens now. */ }
  };
  const filenameRef = useRef<HTMLTextAreaElement>(null);
  const descriptionRef = useRef<HTMLTextAreaElement>(null);
  // The name's field and the description's are one item's, and one
  // revision's: each is keyed by the draft (below), so the browser's undo
  // steps go with the field they were made in. One field served every item,
  // and an undo step could outlive its item: once the field's whole text
  // had been replaced - by a party chip, or by Ctrl+A and typing or Delete -
  // Ctrl+Z in the next document's field put the last one's old text after
  // its own (Chromium; WebView2 is Chromium). In the name that was one
  // Enter from filing; in the description, another matter's sentence went
  // into this document's record, the description ledger and the shared
  // filed index. A caret in the old field moves to the new one, at the end,
  // where the reused field had it.
  const nameHadFocus = useRef(false);
  const descriptionHadFocus = useRef(false);
  if (draft.for !== draftFor) {
    nameHadFocus.current = filenameRef.current !== null && document.activeElement === filenameRef.current;
    descriptionHadFocus.current = descriptionRef.current !== null && document.activeElement === descriptionRef.current;
    const kept = draft.edited ? new Map(setAside).set(draft.for, draft) : setAside;
    if (kept !== setAside) setSetAside(kept);
    setDraft(kept.get(draftFor) ?? { for: draftFor, stem: proposedStem, description: item.description ?? '', edited: false });
    setError(''); setMoreOpen(false); setConfirmingRemove(false); setAskingSpelling(undefined); setSpellingNote(undefined);
  }
  // The More menu is drawn only beside Keep original. An item that can no
  // longer be kept - a refresh showing it being read again - took the menu
  // off screen and left it open, and the next Escape closed that unseen
  // menu instead of the drawer. It closes with what it belonged to.
  if (moreOpen && !actions.keep) setMoreOpen(false);
  const { stem, description } = draft;
  const setStem = (value: string) => setDraft((current) => ({ ...current, stem: value, edited: true }));
  const setDescription = (value: string) => setDraft((current) => ({ ...current, description: value, edited: true }));
  const inspectorRef = useRef<HTMLElement>(null);
  const confirmRef = useRef<HTMLButtonElement>(null);
  const headingRef = useRef<HTMLHeadingElement>(null);
  const moreButtonRef = useRef<HTMLButtonElement>(null);
  const moreMenuRef = useRef<HTMLDivElement>(null);
  const actionsRef = useRef<HTMLDivElement>(null);
  const fieldId = useId();
  useEffect(() => { if (confirmingRemove) confirmRef.current?.focus(); }, [confirmingRemove]);
  // Text with a line break - a cell copied from a spreadsheet, a line from a
  // document - goes into the name with a space for each break, as typed,
  // through the browser's editing command. Pasted as it was, the field held
  // the break until the change set the name from code, and Chromium's undo
  // of that paste then put the old name after the new one: Ctrl+A, paste
  // "2024-02-01 Engagement Letter with Contoso\r\n", Ctrl+Z gave "2024-02-01
  // Engagement Letter with Contoso 2024-02-01 Engagement Letter with
  // Northwind Traders", one Enter from filing. Registered on each render,
  // so the field of the draft on screen has it.
  useLayoutEffect(() => {
    const field = filenameRef.current;
    if (!field) return;
    const asTyped = (event: InputEvent) => {
      const text = event.data ?? event.dataTransfer?.getData('text/plain') ?? null;
      if (text === null || !/[\r\n]/.test(text) || typeof document.execCommand !== 'function') return;
      event.preventDefault();
      if (document.execCommand('insertText', false, withoutLineBreaks(text))) return;
      editStem(withoutLineBreaks(`${field.value.slice(0, field.selectionStart)}${text}${field.value.slice(field.selectionEnd)}`));
    };
    field.addEventListener('beforeinput', asTyped);
    return () => field.removeEventListener('beforeinput', asTyped);
  });
  // The More menu closes on a press anywhere outside it, as the toolbar's
  // does. It used to stay open over the panel until its own button was
  // pressed again or one of its actions taken.
  useEffect(() => {
    if (!moreOpen) return;
    const close = (event: PointerEvent) => {
      const target = event.target as Node | null;
      if (target && (moreMenuRef.current?.contains(target) || moreButtonRef.current?.contains(target))) return;
      setMoreOpen(false);
    };
    document.addEventListener('pointerdown', close);
    return () => document.removeEventListener('pointerdown', close);
  }, [moreOpen]);
  // The whole name, always: a single-line field showed about half of a
  // typical proposal, and the caret only its tail. The field grows to fit
  // instead. Chromium sizes it from its content (field-sizing, in app.css);
  // this is for an engine that cannot.
  useLayoutEffect(() => {
    const field = filenameRef.current;
    if (!field || typeof CSS === 'undefined' || CSS.supports?.('field-sizing', 'content')) return;
    field.style.height = 'auto';
    field.style.height = `${field.scrollHeight}px`;
  }, [stem, draft.for]);
  useLayoutEffect(() => {
    const field = nameHadFocus.current ? filenameRef.current : descriptionHadFocus.current ? descriptionRef.current : null;
    nameHadFocus.current = false;
    descriptionHadFocus.current = false;
    if (!field) return;
    field.focus();
    field.setSelectionRange(field.value.length, field.value.length);
  }, [draft.for]);
  // The decision sticks to the bottom of the panel over whatever scrolls
  // under it, and the browser scrolls a focused control into view only when
  // none of it is in the panel, under the buttons or not. A control focused
  // under them, or half under them, stayed there: "Analysis details" reached
  // by Tab, the question "Use on its own" asks with its buttons cut off. A
  // control focused from the keyboard is scrolled clear of the buttons,
  // however tall they are now, never so far that its top leaves the panel;
  // a control of that question brings the whole question clear, and so
  // does a key typed in it. After a press nothing is scrolled until a key
  // is pressed (`afterAPress`): not the control the press focused, so that
  // the press lands on what was pressed, and not the question a press
  // opened, whose scroll clear of the buttons brought its yes under the
  // pointer for the press that came next. Left where it opened, though,
  // that question took the spelling typed into it in a field under the
  // buttons; the first key brings it clear.
  const keepFocusInSight = (event: FocusEvent<HTMLElement> | KeyboardEvent<HTMLElement>) => {
    const panel = inspectorRef.current;
    const decision = actionsRef.current;
    const focused = event.target as HTMLElement;
    if (afterAPress() || !panel || !decision || decision.offsetHeight === 0 || decision.contains(focused)) return;
    const box = (focused.closest('.rule-confirm') ?? focused).getBoundingClientRect();
    const under = box.bottom + 12 - decision.getBoundingClientRect().top;
    const room = box.top - 12 - panel.getBoundingClientRect().top;
    if (under > 0 && room > 0) panel.scrollTop += Math.min(under, room);
  };
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
  // A name a click puts in - a party chip, a date - goes in as typing would,
  // through the browser's own editing command, so one Ctrl+Z in the name
  // gives back what the click replaced. A value set from code empties the
  // field's undo history, and a party the reviewer had corrected by hand
  // could only be typed again. Only the words that differ are replaced, not
  // the whole text: Chromium's undo of a whole-text replacement puts the
  // old text back after whatever the field holds by then, so after a click
  // that set the name from code - a date chip did, until it came here too -
  // Ctrl+Z gave the name with its old self after it, one Enter from filing.
  // Where the command is missing or refused (jsdom has none), or leaves
  // anything but the new name, the value is set from code. The caret stays
  // in the name, at the end, where Enter files it.
  const typeName = (next: string) => {
    const field = filenameRef.current;
    field?.focus();
    if (next === stem) return;
    const { start, end, text } = changedWords(stem, next);
    field?.setSelectionRange(start, end);
    const typed = field !== null && document.activeElement === field && typeof document.execCommand === 'function' && document.execCommand('insertText', false, text);
    if (!typed || field.value !== next) editStem(next);
    else field.setSelectionRange(next.length, next.length);
  };
  // The model's date, when Intern could not find it written in the document
  // and so left it out of the name. Offered until the name carries a date.
  const suggestedDate = item.suggestedDate && !dated ? item.suggestedDate : undefined;
  // A date put into the name takes the offer off screen, and the button that
  // was pressed with it: focus fell to nothing, and a keyboard user had to
  // find the panel again to file the name they had just completed. It goes
  // to the name, which now starts with that date; Enter there files it.
  const putDate = (date: string) => typeName(withLeadingDate(stem, date));
  const useSuggestedDate = () => putDate(suggestedDate!);
  const acceptSuggestedDate = () => submit(withLeadingDate(stem, suggestedDate!));
  // When the name has no date and the model offered none worth showing, the
  // document's own dates - and, last, the file's - are one click each.
  const dateChoices = !dated ? (item.datesInDocument ?? []) : [];
  const fileDate = !dated ? item.fileModifiedDate : undefined;
  const chooseDate = putDate;
  // The other parties the document names, when the name gives the wrong
  // one: a click puts one in place of the party the name gives - which
  // used to mean retyping fifteen to forty characters - as typed (see
  // typeName), and the caret stays in the name, where Enter files it. The
  // one the name now gives is pressed.
  const partyChoices = item.partyChoices ?? [];
  const partyOffer = { proposed: proposedStem, nameParties: item.nameParties ?? [], choices: partyChoices };
  const chooseParty = (party: string) => typeName(withParty(stem, party, partyOffer));
  const partyInName = partyChoices.length > 0 ? partyGiven(stem, partyOffer) : undefined;
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
    // Escape takes back the last thing opened: the More menu first, with
    // focus back on its button. Only with no menu open does it reach the
    // drawer, which it closes. It used to close the whole drawer from under
    // an open menu, and beside the queue it did nothing at all.
    if (event.key === 'Escape' && moreOpen && !event.nativeEvent.isComposing) {
      event.preventDefault();
      event.stopPropagation();
      // Focus on one of the menu's actions would go with the menu.
      if (moreMenuRef.current?.contains(document.activeElement)) moreButtonRef.current?.focus();
      setMoreOpen(false);
      return;
    }
    // Then the question "Use on its own" asks, wherever the focus is in the
    // panel: a press on its sentence let go off the question leaves the
    // focus on the group around it, and Escape there closed the drawer.
    if (event.key === 'Escape' && askingSpelling !== undefined && !event.nativeEvent.isComposing) {
      event.preventDefault();
      event.stopPropagation();
      takeBackSpelling(askingSpelling);
      return;
    }
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
  // The name can be edited only where it can be approved: an item flagged
  // before analysis has no proposal for the backend to approve. A parked
  // item's can be, since approving it checks its files first.
  const editable = actions.approve;
  const omitted = item.omittedParties ?? [];
  /*
    A document held because a learned spelling nobody has checked changed
    its name offers, for each such spelling the name used, "Use on its
    own": the only way a spelling is checked - approving this name checks
    nothing - after the question Settings asks, with the word and the
    spelling exactly as the backend will check them. It approves and files
    nothing: the document still waits for its own decision. Whether a
    spelling is checked is the backend's list's to say, read again for each
    item and revision, and after every answer.
  */
  const heldForSpelling = item.status === 'review' && itemReasonCodes(item).includes('LEARNED_SPELLING_UNCONFIRMED');
  const usedRules = item.houseRules ?? [];
  const [learned, setLearned] = useState<LearnedRule[]>();
  const learnedFor = heldForSpelling && spellings && usedRules.length > 0
    ? [item.id, item.proposalRevision ?? '', ...usedRules.flatMap((rule) => [rule.kind, rule.from, rule.to])].join('\n')
    : undefined;
  useEffect(() => {
    if (learnedFor === undefined || !spellings) { setLearned(undefined); return; }
    let active = true;
    spellings.houseRulesList().then((rules) => { if (active) setLearned(rules); }, () => { if (active) setLearned(undefined); });
    return () => { active = false; };
  }, [learnedFor, spellings]);
  const unchecked = heldForSpelling && learned
    ? usedRules.flatMap((rule) => learned.filter((candidate) => sameSpelling(candidate, rule) && candidate.active && candidate.confirmed === 0).slice(0, 1))
    : [];
  // Answered or taken back, the focus goes back where the action was: its
  // button, or the group once the spelling is checked and the button gone.
  const spellingChecks = useRef<HTMLDivElement>(null);
  const [spellingFocus, setSpellingFocus] = useState<{ rule?: string }>();
  useEffect(() => {
    if (!spellingFocus || askingSpelling !== undefined) return;
    const trigger = [...(spellingChecks.current?.querySelectorAll<HTMLElement>('button[data-rule]') ?? [])].find((button) => button.dataset.rule === spellingFocus.rule);
    (trigger ?? spellingChecks.current)?.focus();
    setSpellingFocus(undefined);
  }, [spellingFocus, askingSpelling, learned]);
  const takeBackSpelling = (rule: string) => { setAskingSpelling(undefined); setSpellingFocus({ rule }); };
  const checkSpelling = async (rule: LearnedRule) => {
    if (!spellings) return;
    setAskingSpelling(undefined);
    try {
      await spellings.houseRuleCheck(rule.id, { kind: rule.kind, from: rule.from, to: rule.to });
      setSpellingNote({ failed: false, text: `Intern will use \u201c${rule.to}\u201d on its own from now on. This document still waits for you.` });
      onSpellingChecked?.();
    } catch (failure) {
      setSpellingNote({ failed: true, text: spellingFailure(failure) });
    }
    await spellings.houseRulesList().then(setLearned, () => undefined);
    setSpellingFocus({});
  };
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
  const flagged = itemFlaggedFields(item);
  const evidence = [
    { label: 'Date', field: 'date' as const, quotes: quotations(item.evidence?.date) },
    { label: 'Type', field: 'type' as const, quotes: quotations(item.evidence?.type) },
    { label: 'Parties', field: 'parties' as const, quotes: quotations(item.evidence?.parties) },
  ].filter((entry) => entry.quotes.length > 0);
  /*
    Whether the evidence is the document's own lines. A local reading's is:
    the engine looks up the lines the model cited and shows those, never the
    model's words. A hosted model's, and a reading stored before Intern
    recorded which model made it, is the model's own wording of the line,
    which small models paraphrase. Shown in quotation marks, a paraphrase
    reads as something the document says - to a lawyer above all - so only
    the document's own lines get them.
  */
  const verbatim = item.readBy !== undefined;
  // In review because the model was unsure (LOW_CONFIDENCE) while the model's
  // own figure, shown under Analysis details, says it was sure.
  const sureByItsOwnFigure = item.status === 'review' && item.confidence !== undefined && item.confidence >= SELF_RATED_UNSURE
    && itemReasonCodes(item).includes('LOW_CONFIDENCE');
  /*
    The Parties row is another matter whoever read it: what the backend
    sends there is the names the proposal settled on, not the lines that
    name them. Set in quotation marks under "Lines from the document" it
    claimed a line no document has - and a party is the part of a name most
    often in doubt. It is shown as the names it is, and the lead says so.
  */
  const quoted = (field: ReasonField) => verbatim && field !== 'parties';
  // The parts of the name being edited that the review is about, as they now read.
  const nameFlags = editable ? flaggedNameParts(joinFilename(stem, extension), flagged) : [];
  const reviewNote = item.reason && (item.status === 'review' || item.status === 'failed');
  return <aside ref={inspectorRef} id={id} className="inspector" aria-label="Review item" role={drawer ? 'dialog' : 'complementary'} aria-modal={drawer || undefined} onKeyDown={onKeyDown} onFocus={keepFocusInSight}
    onKeyDownCapture={(event) => { if ((event.target as HTMLElement).closest('.rule-confirm')) keepFocusInSight(event); }}>
    {/* A filed document's name is no longer its "current" one; it said so anyway (FRONTEND_UX-8). */}
    {/* Focusable so selecting a row can bring a keyboard user here, past the rest of the queue. */}
    <div className="inspector-title"><h2 ref={headingRef} tabIndex={-1}>{completed ? 'Filed document' : 'Review item'}</h2><button type="button" className="icon-button" onClick={onClose} aria-label="Close review"><Icon icon={X} /></button></div>
    {position && position.total > 0 && <div className="review-position">
      <span>{position.index ? `${position.index} of ${position.total} to decide` : `${position.total} to decide`}</span>
      {/* Its key is in the tooltip and read out with it; no badge here, where the line has no room for one. */}
      {onNext && <button type="button" className="link-button" onClick={onNext} aria-keyshortcuts="Alt+N" title="Leave this one for now and go to the next still to decide (Alt+N)">Next undecided<Icon icon={ChevronRight} /></button>}
    </div>}
    {/*
      What to check comes first. It used to be the last thing in the panel,
      under the name, the description and the evidence, so for the documents
      that most needed it the reason was below the fold.
    */}
    {reviewNote && <section className={`check-first${item.status === 'failed' ? ' failed' : ''}`}>
      <h3><Icon icon={CircleAlert} />{item.status === 'failed' ? 'Failure details' : 'What to check'}</h3>
      <p>{item.reason}</p>
      {/*
        The sentence says "a document that was filed already"; this says
        which one, so the person can open it and compare rather than take
        Intern's word for it.
      */}
      {item.nearDuplicateOf && <p className="near-duplicate" aria-label="Filed already as">Filed already as <q>{item.nearDuplicateOf}</q>.</p>}
    </section>}
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
      <h3>{unreadable ? 'Name it yourself' : 'Proposed rename'}</h3>
      <label className="filename-label">Filename
        <span className="filename-field">
          <textarea key={draft.for} ref={filenameRef} aria-label="Filename" className="filename-input" rows={1} spellCheck={false} autoComplete="off" value={stem}
            onChange={(event) => editStem(withoutLineBreaks(event.target.value))} onKeyDown={onFilenameKeyDown}
            aria-invalid={Boolean(error)} aria-describedby={[extension !== undefined ? `${fieldId}-extension` : '', error ? `${fieldId}-error` : ''].filter(Boolean).join(' ') || undefined} />
          {extension !== undefined && <span className="filename-extension" title="The extension stays as it is" aria-hidden="true">.{extension}</span>}
        </span>
      </label>
      {extension !== undefined && <span className="sr-only" id={`${fieldId}-extension`}>The name ends in .{extension}, which cannot change.</span>}
      {error && <p className="form-error" role="alert" id={`${fieldId}-error`}>{error}</p>}
      {!error && stem.trim() && !dated && !suggestedDate && <p className="check-hint date-hint">{DATE_REQUIRED_MESSAGE}</p>}
      {!error && nameFlags.length > 0 && <p className="flag-line" role="note" aria-label="Part of the name to check">
        Check the {listOf(nameFlags.map(({ field }) => FIELD_WORDS[field]))}: {nameFlags.map(({ text }, index) => <span key={index}>{index > 0 ? ', ' : ''}<mark className="flagged">{text.trim()}</mark></span>)}
      </p>}
      {/*
        A name that differs from the evidence under it looks like a mistake
        unless the reason is on screen: the reviewer's own spelling, learned
        from their edits, was applied. Said here, beside the name, with the
        document's words kept in the evidence below. A spelling ending in
        its own period ("Inc.") ends the list without a second one.
      */}
      {item.houseRules && item.houseRules.length > 0 && <p className="check-hint house-style-hint" role="note" aria-label="Learned spellings applied">
        Uses your spelling: {item.houseRules.map((rule, index) => <span key={`${rule.kind}-${rule.from}`}>{index > 0 ? '; ' : ''}<q>{rule.from}</q> written as <strong>{rule.to}</strong></span>)}{item.houseRules[item.houseRules.length - 1].to.trimEnd().endsWith('.') ? '' : '.'} Change or forget it under Settings.
      </p>}
      {/*
        Each action says on screen which spelling it is for whenever the note
        above lists more than one: a bare "Use on its own" under three
        spellings, one of them just checked and named in the line below it,
        read as an action for that one.
      */}
      {(unchecked.length > 0 || spellingNote) && <div ref={spellingChecks} className="spelling-checks" role="group" aria-label="Use a learned spelling on its own" tabIndex={-1}>
        {unchecked.map((rule) => askingSpelling === rule.id
          ? <UseOnItsOwnQuestion key={rule.id} rule={rule} describedBy={`${fieldId}-spelling-${rule.id}`} disabled={busy}
            note="Choosing this does not approve this document: it still waits for you."
            onConfirm={() => void checkSpelling(rule)} onCancel={() => takeBackSpelling(rule.id)} />
          : <div key={rule.id} className="spelling-check">
            {usedRules.length > 1 && <span><q>{rule.from}</q> written as <strong>{rule.to}</strong></span>}
            <button type="button" data-rule={rule.id} disabled={busy || askingSpelling !== undefined} aria-label={`Use on its own: ${spellingLabel(rule)}`} onClick={() => { setSpellingNote(undefined); setAskingSpelling(rule.id); }}>Use on its own</button>
          </div>)}
        {spellingNote && <p className={spellingNote.failed ? 'form-error' : 'spelling-checked'} role={spellingNote.failed ? 'alert' : 'status'}>{spellingNote.text}</p>}
      </div>}
      {/*
        The same for a party the evidence names and the name does not: the
        person's own organization, left out so the name says who the
        document is with.
      */}
      {omitted.length > 0 && <p className="check-hint own-names-hint" role="note" aria-label="Filed by the other side">
        Filed by the other side: {omitted.map((name, index) => <span key={`${index}-${name}`}>{index === 0 ? '' : index === omitted.length - 1 ? ' and ' : ', '}<q>{name}</q></span>)} {omitted.length === 1 ? 'is your organization, so it is' : 'are your organization, so they are'} left out of the name. Change this under Settings.
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
      {partyChoices.length > 0 && <div className="party-choices" role="group" aria-label="Other parties to choose from">
        <p className="field-label">Other parties the document names</p>
        <span className="sr-only" id={`${fieldId}-party-choices`}>Puts this party in the filename in place of the one it gives.</span>
        <div className="chips">
          {partyChoices.map((party) => <button type="button" key={party} className="chip" disabled={busy} aria-pressed={party === partyInName} aria-describedby={`${fieldId}-party-choices`} onClick={() => chooseParty(party)}>{party}</button>)}
        </div>
      </div>}
      {suggestedDate && <div className="suggestion" role="group" aria-label="Suggested date">
        <p><Icon icon={CalendarPlus} />Suggested date: <strong>{suggestedDate}</strong> (unverified). Check it against the document before using it.</p>
        <div className="suggestion-actions">
          <button type="button" disabled={busy} onClick={useSuggestedDate}>Use this date</button>
          <button type="button" className="primary" disabled={busy} onClick={decides(acceptSuggestedDate)}>Use date &amp; rename</button>
        </div>
      </div>}
      <label>Description<textarea key={draft.for} ref={descriptionRef} value={description} onChange={(event) => setDescription(event.target.value)} /></label>
      {flagged.includes('description') && <p className="flag-line" role="note" aria-label="Part of the name to check">Check the description against the document.</p>}
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
    {/*
      Opened and closed here rather than by the browser, so the panel knows
      which it is and the next document's section is drawn the same way. A
      section the browser opens by itself (find in page) is taken note of
      for now, and not as the person's choice.
    */}
    {(evidence.length > 0 || item.confidence !== undefined || item.readBy) && <details className="analysis-details" open={analysisOpen}
      onToggle={(event) => { if (event.currentTarget.open !== analysisOpen) setAnalysisOpen(event.currentTarget.open); }}>
      <summary onClick={(event) => { event.preventDefault(); rememberAnalysis(!analysisOpen); }}>Analysis details</summary>
      {/*
        The model's figure for its own reading. It no longer has a column:
        a small model overstates it, and what decides review is the checks
        above. Kept here for anyone who wants it, and named as the model's
        own so it is not read as Intern's.
      */}
      {item.confidence !== undefined && <p className={`confidence-readout confidence ${item.status}`}><span className="field-label">Model's own confidence</span><ConfidenceMeter value={item.confidence} status={item.status} variant="panel" /></p>}
      {/*
        "The model was unsure of its reading" above, and "100%" here: both
        true, and a contradiction unless the panel says why. The review
        reason comes from what Intern measured as the model wrote the date
        and the parties, or from this figure when it is low - and then the
        two agree and there is nothing to explain. The line says only that
        they are different measures, not which one flagged this reading.
      */}
      {sureByItsOwnFigure && <p className="check-hint confidence-note" role="note" aria-label="About the model's own confidence">
        This is the model's rating of its whole answer, and small models rate themselves high. Intern also measures how sure the model was of each word of the date and the parties as it wrote them, so a reading can be flagged as unsure while this figure is high.
      </p>}
      {/*
        Which local model read it, in a quiet line: worth knowing when two
        models can, and the reason a reading may change by itself - one the
        fast model made in the accurate one's place is read again once the
        accurate one can, unless someone acts on it first.
      */}
      {item.readBy && <p className="check-hint read-by" aria-label="Read by">{readByLine(item.readBy, !item.approved)}</p>}
      {evidence.length > 0 && <section className="evidence"><h3>Evidence</h3>
        <p className="section-lead">{verbatim
          ? evidenceLead(evidence.map((entry) => entry.field))
          : 'The model\'s reading of the document, in its own words. These are not lines matched in the document, so check them against it.'}</p>
        <dl className="evidence-list">{evidence.map(({ label, field, quotes }) => {
          const doubted = flagged.includes(field);
          return <div className={`evidence-item${doubted ? ' flagged' : ''}`} key={label}>
            <dt className="field-label">{doubted && <Icon icon={CircleAlert} />}{label}{doubted && <span className="sr-only"> (check this)</span>}</dt>
            <dd>{quotes.map((quote) => quoted(field) ? <q key={quote}>{quote}</q> : <span className="reading" key={quote}>{quote}</span>)}</dd>
          </div>;
        })}</dl></section>}
    </details>}
    {/* Settled, a reason is no longer why the item waits - "already had this name" is a note about what happened. */}
    {/* A waiting document is not in review: its reason says why it waits (a document in doubt, to be read again once Intern has shown it can read). */}
    {item.reason && !reviewNote && <section className="note"><h3>{completed ? 'Note' : item.status === 'waiting' ? 'Why it is waiting' : 'Note'}</h3><p>{item.reason}</p>
      {/*
        The sentence says "a document that was filed already"; this says
        which one, so the person can open it and compare rather than take
        Intern's word for it.
      */}
      {item.nearDuplicateOf && <p className="near-duplicate" aria-label="Filed already as">Filed already as <q>{item.nearDuplicateOf}</q>.</p>}
    </section>}
    <div ref={actionsRef} className="inspector-actions">
      {confirmingRemove && remove ? <div className="remove-confirm" role="group" aria-label="Confirm removal" onKeyDown={(event) => {
        // Escape takes back the question, not the whole drawer.
        if (event.key === 'Escape') { event.preventDefault(); event.stopPropagation(); cancelRemove(); }
      }}>
        <p>{remove.resolvedFiles
          ? 'This rename stopped part-way, so its files may need putting right by hand. Remove it only once you have: Intern will not touch them again.'
          : <>Remove <q>{item.originalFilename}</q> from the queue? The file itself stays where it is{isUnreadable(item) ? ', under this name, and Intern leaves it alone' : ''}.</>}</p>
        <button ref={confirmRef} type="button" className="primary" disabled={busy} onClick={decides(() => { setConfirmingRemove(false); onRemove(remove.resolvedFiles); })}>{remove.resolvedFiles ? 'I have resolved the files myself' : remove.label}</button>
        <button type="button" disabled={busy} onClick={cancelRemove}>Cancel</button>
      </div> : <>
        {/* A parked item leads with checking its files; approving, which checks them first too, comes after it. */}
        {actions.retry?.primary && <button type="button" className="primary" disabled={busy} onClick={onRetry}><Icon icon={RotateCcw} />{actions.retry.label}</button>}
        {actions.approve && <button type="button" className={actions.retry?.primary ? undefined : 'primary'} disabled={busy} onClick={decides(approve)} aria-keyshortcuts="Control+Enter"><Icon icon={FileCheck2} />{item.status === 'ready' ? 'Apply rename' : 'Approve & rename'}<Shortcut keys="Ctrl+Enter" /></button>}
        {actions.keep && <>
          <button type="button" className="secondary-action" disabled={busy} onClick={decides(onKeep)} aria-keyshortcuts="Alt+K"><Icon icon={FileText} />Keep original<Shortcut keys="Alt+K" /></button>
          <button ref={moreButtonRef} type="button" className="icon-button more-actions" disabled={busy} aria-label="More review actions" aria-expanded={moreOpen} onClick={() => setMoreOpen(!moreOpen)}><Icon icon={Ellipsis} /></button>
          {moreOpen && <div ref={moreMenuRef} className="review-menu" role="group" aria-label="More review actions">
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

/** "Read by Qwen3.5 9B", and anything a reviewer should know about that reading. */
export function readByLine(reading: ModelReading, pending: boolean): string {
  const read = `Read by ${reading.displayName}`;
  if (reading.escalationFailed) return `${read}. The accurate model could not read it again.`;
  if (reading.escalated) return `${read}, after a second look.`;
  if (reading.provisional && pending) return `${read}. Provisional — will be re-read by the accurate model.`;
  return read;
}

const FIELD_WORDS: Record<ReasonField, string> = { date: 'date', type: 'document type', parties: 'parties', description: 'description' };

/**
 * What a local reading's evidence is, said for the rows it has: the date and
 * the document type are lines found in the document; the parties are names.
 */
function evidenceLead(fields: readonly ReasonField[]): string {
  const lines = fields.filter((field) => field !== 'parties').map((field) => FIELD_WORDS[field]);
  const names = fields.includes('parties');
  return [
    lines.length > 0 ? `Lines from the document behind the ${listOf(lines)}.` : '',
    names ? `The parties are the names Intern read${lines.length > 0 ? ', not lines from the document' : '. They are not lines from the document'}.` : '',
  ].filter(Boolean).join(' ');
}

/** "date", "date and parties", "date, document type and parties". */
function listOf(words: string[]): string {
  return words.length < 2 ? words.join('') : `${words.slice(0, -1).join(', ')} and ${words.at(-1)}`;
}

/** The flagged parts of a name, one per field, in the order the name reads. */
function flaggedNameParts(name: string, flagged: readonly ReasonField[]): Array<{ field: ReasonField; text: string }> {
  return nameParts(name, flagged).flatMap((part) => part.flagged && part.field && part.text.trim() ? [{ field: part.field, text: part.text }] : []);
}

/**
 * Where two names differ: the span of `from`, `start` to `end`, that `text`
 * takes the place of to make `to`. The words both share at the start and at
 * the end are left out, whole: the span begins and ends at a space or at an
 * end of `from`, so it splits no character - a letter and its accent, or a
 * character of two code units. Chromium's editing never splits one, and
 * given a span that does, edits another span than the one meant.
 */
function changedWords(from: string, to: string): { start: number; end: number; text: string } {
  let start = 0;
  while (start < from.length && start < to.length && from[start] === to[start]) start += 1;
  // `end` is where the shared ending begins in `from`; `to.length - (from.length - end)` is where it begins in `to`.
  let end = from.length;
  while (end > start && to.length - (from.length - end) > start && from[end - 1] === to[to.length - (from.length - end) - 1]) end -= 1;
  const boundary = (at: number) => at === 0 || at === from.length || from[at - 1] === ' ' || from[at] === ' ';
  while (!boundary(start)) start -= 1;
  while (!boundary(end)) end += 1;
  return { start, end, text: to.slice(start, to.length - (from.length - end)) };
}
