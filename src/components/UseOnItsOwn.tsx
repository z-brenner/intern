import { useEffect, useLayoutEffect, useRef, useState } from 'react';
import type { ReactNode } from 'react';
import { afterAPress } from '../lib/lastInput';
import { isTextField } from '../lib/textField';
import type { HouseRule, LearnedRule } from '../types';

/**
 * What "Use on its own" does to every later document, in plain words, with
 * the document's word and the spelling exactly as they are: the one action
 * that lets Intern file a name a learned spelling changed without a person
 * looking at it. An approval never does that.
 */
export function onItsOwnConsequence(rule: HouseRule): ReactNode {
  return rule.kind === 'party'
    ? <>Every later document Intern reads as naming <q>{rule.from}</q> will name <q>{rule.to}</q> instead, and may be filed under that name without your check.</>
    : <>Every later document Intern reads as <q>{rule.from}</q> will be called <q>{rule.to}</q> instead, and may be filed under that name without your check.</>;
}

/** How a learned spelling is named to a screen reader: "Acme written as Acme Corp". */
export const spellingLabel = (rule: HouseRule) => `${rule.from} written as ${rule.to}`;

/**
 * The scripts whose marks are accents a person may leave off: Latin and
 * Greek, and Hebrew and Arabic, which as a rule write their vowel points not
 * at all. Elsewhere a mark makes another letter or syllable - ガ is not カ,
 * Й is not И, रमा is not राम - and is typed with it.
 */
const ACCENTED = /[\p{sc=Latin}\p{sc=Greek}\p{sc=Hebrew}\p{sc=Arabic}]/u;

/**
 * Letters that no decomposition takes apart, as a keyboard without them
 * types them: "Søren" is "Soren", "Straße" is "Strasse", "Kırşehir" is
 * "Kirsehir". A word's final sigma is its sigma.
 */
const PLAIN: Record<string, string> = { ß: 'ss', æ: 'ae', œ: 'oe', ø: 'o', ł: 'l', đ: 'd', ð: 'd', þ: 'th', ı: 'i', ħ: 'h', ŧ: 't', ς: 'σ' };

/** What is no letter or digit as written: punctuation, spaces, a symbol, an apostrophe written as a letter (ʻ, ʼ). */
const NOT_LETTERS = /[^\p{L}\p{M}\p{N}]|[\u02B9-\u02BF]/gu;
/** Marks that choose how a character is drawn, not which it is. */
const VARIATIONS = /[\uFE00-\uFE0F\u{E0100}-\u{E01EF}]/gu;

/**
 * The letters and digits of `text`, in order, as a person types them: in
 * any case, without accents (`ACCENTED`, `PLAIN`), and without what is no
 * letter as written - "Müller & Co." is "mullerco", and "Acme™" is "acme",
 * not the "acmetm" its decomposition spells.
 */
const lettersOf = (text: string) => text
  .replace(VARIATIONS, '')
  .replace(NOT_LETTERS, '')
  .normalize('NFKD')
  .replace(/(\P{M})(\p{M}+)/gu, (_, letter: string, marks: string) => ACCENTED.test(letter) ? letter : letter + marks)
  .toLowerCase()
  .replace(/[ßæœøłđðþıħŧς]/gu, (letter) => PLAIN[letter])
  .replace(NOT_LETTERS, '');

/**
 * Whether `typed` is `spelling` typed out: the same letters and digits in
 * the same order, whatever the case, the accents, the spaces and the
 * punctuation - "contoso ltd" is "Contoso Ltd.", "Muller" is "Müller",
 * "Soren" is "Søren" - and a letter missing or added is not, nor a mark
 * that makes another letter. A spelling with no letter or digit in it is
 * typed exactly, spaces at its ends aside.
 */
export function typedOut(typed: string, spelling: string): boolean {
  const letters = lettersOf(spelling);
  if (letters !== '') return lettersOf(typed) === letters;
  return typed.trim() !== '' && typed.trim() === spelling.trim();
}

/** How many characters `text` is as it is seen: "é" is one however it is encoded, and so is "👍🏽". */
function characters(text: string): number {
  if (typeof Intl.Segmenter !== 'function') return [...text].length;
  return [...new Intl.Segmenter(undefined, { granularity: 'grapheme' }).segment(text)].length;
}

/**
 * What typing puts in a field, or takes out: keys, an input method's
 * composition, a deletion. A key types one character, and an input method
 * that shows no composition commits what its keys spelled, so text put in
 * outside a composition is taken one character at a time, or as many at
 * once as keys that type (`typesText`) were pressed for it since the last
 * input (`keys`) - an arrow, Enter or Ctrl+A earns nothing. Not a
 * paste, a drop, a suggestion taken (a spelling, an autocorrection,
 * autofill, which says no input type at all), a yank, an undo, nor a word
 * put in at once with no keys for it: an input method's clipboard, an
 * on-screen keyboard's clipboard or word, an emoji keyboard's.
 */
const typing = (event: InputEvent, keys: number) => {
  const type = event.inputType;
  if (type === 'insertText') return event.data === null || characters(event.data) <= Math.max(1, keys);
  return type === 'insertCompositionText' || type === 'insertFromComposition' || type.startsWith('delete');
};

/** A key's name, not the character it types: Enter, ArrowLeft, F5, Shift, Process. */
const NAMED_KEY = /^[A-Z][A-Za-z0-9]+$/;
/** The named keys an input method or a dead key types with. */
const TYPING_KEYS = new Set(['Process', 'Unidentified', 'Dead']);

/**
 * Whether a key types: a character key, or an input method's key or a
 * dead key - not an arrow, Enter, Home, Backspace, a function key or a
 * modifier, nor a key held with Ctrl or Cmd (Ctrl with Alt is AltGr on
 * Windows, which types). Only these earn the text put in at once (`typing`).
 */
const typesText = (event: { key: string; ctrlKey: boolean; altKey: boolean; metaKey: boolean }) =>
  (!NAMED_KEY.test(event.key) || TYPING_KEYS.has(event.key)) && (!event.ctrlKey || event.altKey) && !event.metaKey;

/** A key that types a character itself: not an input method's or a dead key, nor the space bar, which presses what has the focus. */
const typesACharacter = (event: KeyboardEvent) => typesText(event) && !NAMED_KEY.test(event.key) && event.key !== ' ';

/** A question asking: where it is, its field, and how it takes a character typed outside its field. */
interface Asking { group: HTMLElement; field: HTMLInputElement; take(): void }

/** The questions asking, the last opened last. */
const asking: Asking[] = [];

/**
 * Whether "Use on its own" asks, in the review window or in Settings. While
 * it does, no key is a shortcut (useReviewShortcuts), wherever the focus is.
 */
export const askingNow = (): boolean => asking.length > 0;

/** Whether a dialog that keeps the keys to itself is open over `group`. */
const underADialog = (group: Element) => [...group.ownerDocument.querySelectorAll('[aria-modal="true"]')].some((dialog) => !dialog.contains(group));

/**
 * Whether a key pressed at `target` is `question`'s: anywhere but in a text
 * field - on nothing, on its sentence, on a button - unless a dialog is open
 * over it. In a field the person put the focus in, a key is that field's.
 */
const isOwn = (question: Asking, target: Element | null) => !underADialog(question.group) && !isTextField(target);

/** What a person puts the focus on to use: a button, a link, a field, a table's cell. */
const CONTROLS = 'a[href], button, input, select, textarea, summary, [contenteditable="true"], [role="gridcell"]';

/**
 * Where a question went away by itself - not answered, not taken back -
 * while the keys typed were its own: the app it was in. Until the person
 * presses, or presses Tab or Escape, the keys after it go nowhere.
 */
let goneFrom: Element | undefined;
/** Whether the last key pressed was Tab or Escape, which leave a question. */
let leftByKey = false;

/** The element `element` is in that is the page's own: the app's root. */
function appOf(element: Element): Element {
  let app = element;
  while (app.parentElement && app.parentElement !== element.ownerDocument.body) app = app.parentElement;
  return app;
}

/**
 * Every key, before anything else on the page sees it. While a question
 * asks, a character typed anywhere its own (`isOwn`) goes into its field,
 * with the focus, and to nothing else: the browser types it where the focus
 * is once the key is down. Once a question went away by itself
 * (`goneFrom`), every key but Tab and Escape goes nowhere.
 */
function routeKey(event: KeyboardEvent) {
  leftByKey = event.key === 'Tab' || event.key === 'Escape';
  if (asking.length === 0) {
    if (!goneFrom) return;
    if (leftByKey || !goneFrom.isConnected) { goneFrom = undefined; return; }
    event.preventDefault();
    event.stopImmediatePropagation();
    return;
  }
  if (event.isComposing || event.keyCode === 229 || !typesACharacter(event)) return;
  const target = event.target instanceof Element ? event.target : null;
  const question = [...asking].reverse().find((candidate) => isOwn(candidate, target));
  if (!question) return;
  event.stopImmediatePropagation();
  question.take();
}

if (typeof window !== 'undefined') {
  window.addEventListener('keydown', routeKey, { capture: true });
  window.addEventListener('pointerdown', () => { goneFrom = undefined; leftByKey = false; }, { capture: true, passive: true });
}

/**
 * The question "Use on its own" asks before it checks a spelling, where the
 * action was: the consequence, a field to type the spelling into, then Use
 * on its own and Cancel. `note` is said after the consequence. Escape takes
 * back the question and nothing else; the dialog or the drawer it is in
 * does the same wherever the focus is in them.
 *
 * The yes answers only once the spelling is typed in the field (`typedOut`):
 * until then it is disabled, and Enter in the field does nothing. Checking a
 * spelling is for good - every name it changes is filed without review from
 * then on - and what can go wrong is a wrong spelling. Typing it out makes
 * the person read it, letter by letter, and no press of any pointer types:
 * not the second press of a double-click on the action, not a press after
 * a scroll or a change of layout brought the yes under the pointer, not a
 * tap through a menu or a dialog over it. Rules that tried to tell a press
 * meant for the yes from one that was not kept being found wanting. So only
 * typing fills the field (`typing`): a paste, a drop, a suggestion taken, an
 * undo, and a word put in at once with no keys for it are refused, and the
 * browser is asked for no suggestions to take. An Enter the field does not
 * answer says why. Once the spelling is typed, the yes is an ordinary
 * button.
 *
 * Keys typed in the field stay there, but for Tab, and Escape: Ctrl+Enter,
 * Alt+K and Alt+N neither approve, keep nor move on in review, and Enter
 * saves no settings. Nor are keys typed elsewhere anyone else's while it
 * asks (`routeKey`): a press on its sentence or its waiting yes, or the
 * second press of a double-click on its action, takes the focus off its
 * field, and the spelling typed next went to the review shortcuts - the K
 * of "Deutsche Bank" moved review to the document before, the drawer put
 * the caret in that one's name, and the Enter meant for the question filed
 * it. A character typed anywhere but in another field, and not from a
 * dialog over the question, goes into its field, with the focus; no other
 * key is a shortcut (`askingNow`). Gone by itself - the document read
 * again, the spelling changed - while what was typed was its own, it
 * leaves the keys after it going nowhere until a press, Tab or Escape:
 * the person typing has not seen it go. While it asks, no field changes
 * but the one with the focus: an undo or a redo, which goes to the field
 * edited last wherever the focus is and takes the focus there, changes no
 * name nor setting behind the question. The field starts empty for every question, and again
 * when the spelling it asks about changes while it asks. Opened by a press,
 * the question takes the focus in its field and moves nothing under the
 * pointer (`afterAPress`); opened from the keyboard, it is brought into
 * sight, and so it is by the first key typed in it after a press opened it.
 */
export function UseOnItsOwnQuestion(props: QuestionProps) {
  const { rule } = props;
  return <Question key={[rule.id, rule.kind, rule.from, rule.to].join('\n')} {...props} />;
}

type QuestionProps = {
  rule: Pick<LearnedRule, 'id' | 'kind' | 'from' | 'to'>;
  /** The id of the sentence, for the group's description. */
  describedBy: string;
  note?: ReactNode;
  disabled?: boolean;
  onConfirm(): void;
  onCancel(): void;
};

function Question({ rule, describedBy, note, disabled, onConfirm, onCancel }: QuestionProps) {
  const group = useRef<HTMLDivElement>(null);
  const field = useRef<HTMLInputElement>(null);
  // Its own name, so the browser has nothing typed before to suggest for it.
  const [name] = useState(() => `use-on-its-own-${Date.now().toString(36)}-${Math.random().toString(36).slice(2)}`);
  const [typed, setTyped] = useState('');
  // The keys pressed in the field since its last input, taken or refused: how much text put in at once may be.
  const keys = useRef(0);
  // An Enter the field did not answer, said until the field changes.
  const [notYet, setNotYet] = useState(false);
  const matches = typedOut(typed, rule.to);
  const ready = !disabled && matches;
  // Answered, or taken back by its Cancel or Escape: not gone by itself.
  const ended = useRef(false);
  useLayoutEffect(() => {
    const own: Asking = {
      group: group.current!,
      field: field.current!,
      take: () => {
        own.field.focus({ preventScroll: true });
        own.group.scrollIntoView?.({ block: 'nearest' });
      },
    };
    asking.push(own);
    goneFrom = undefined;
    return () => {
      asking.splice(asking.indexOf(own), 1);
      // Still on the page as it goes: where the focus is, keys were its own.
      const at = own.group.ownerDocument.activeElement;
      if (!ended.current && !leftByKey && (at === own.field || (isOwn(own, at) && !at?.matches(CONTROLS)))) goneFrom = appOf(own.group);
    };
  }, []);
  useEffect(() => {
    const input = field.current;
    if (!afterAPress()) group.current?.scrollIntoView?.({ block: 'nearest' });
    input?.focus({ preventScroll: true });
    // Keys a refused input was put in after are spent on it.
    const refuse = (event: InputEvent) => { if (!typing(event, keys.current)) { keys.current = 0; event.preventDefault(); } };
    input?.addEventListener('beforeinput', refuse);
    // While it asks, no field changes but the one with the focus. An undo or
    // a redo goes to the field edited last, wherever the focus is, and takes
    // the focus there: pressed in this field, or after a press on the
    // sentence took the focus from it, it undid the document's name, and the
    // spelling and its Enter went into the name.
    const elsewhere = (event: InputEvent) => { if (event.target !== document.activeElement) event.preventDefault(); };
    document.addEventListener('beforeinput', elsewhere, true);
    return () => {
      input?.removeEventListener('beforeinput', refuse);
      document.removeEventListener('beforeinput', elsewhere, true);
    };
  }, []);
  const answer = () => {
    if (ready) { ended.current = true; onConfirm(); }
    else if (!matches && typed.trim() !== '') setNotYet(true);
  };
  const cancel = () => { ended.current = true; onCancel(); };
  return <div ref={group} className="rule-confirm" role="group" aria-label={`Use on its own: ${spellingLabel(rule)}`} aria-describedby={describedBy} onKeyDown={(event) => {
    if (event.key === 'Escape' && !event.nativeEvent.isComposing) { event.preventDefault(); event.stopPropagation(); cancel(); }
  }}>
    <p id={describedBy}>{onItsOwnConsequence(rule)} Do this only if that is right for every one of them.{note ? <> {note}</> : null}</p>
    <label className="rule-confirm-field">Type <q>{rule.to}</q> to use it on its own
      <input ref={field} type="text" name={name} value={typed} autoComplete="off" autoCorrect="off" autoCapitalize="off" spellCheck={false}
        onChange={(event) => {
          const input = event.nativeEvent;
          const taken = input instanceof InputEvent && typing(input, keys.current);
          keys.current = 0;
          if (!taken) return;
          setTyped(event.target.value);
          setNotYet(false);
        }}
        onPaste={(event) => event.preventDefault()}
        onDrop={(event) => event.preventDefault()}
        onKeyDown={(event) => {
          if (typesText(event)) keys.current += 1;
          // An input method's keys, the Enter that commits what it composed among them.
          const composing = event.nativeEvent.isComposing || event.keyCode === 229;
          if ((event.key === 'Tab' || event.key === 'Escape') && !composing) return;
          event.stopPropagation();
          // A key is no aim of the pointer: what it types is brought into sight.
          group.current?.scrollIntoView?.({ block: 'nearest' });
          if (event.key !== 'Enter' || composing) return;
          event.preventDefault();
          if (!event.repeat && !event.altKey && !event.ctrlKey && !event.metaKey && !event.shiftKey) answer();
        }} />
    </label>
    <span className="rule-confirm-buttons"><button type="button" className="primary" disabled={!ready} onClick={answer}>Use on its own</button>
      <button type="button" onClick={cancel}>Cancel</button></span>
    <span className="rule-confirm-not-yet" role="status">{notYet && !matches ? <>That is not <q>{rule.to}</q>: type it as it is shown.</> : null}</span>
  </div>;
}

/** The code of a failure the backend reported, when it has one. */
export function failureCode(error: unknown): string | undefined {
  return typeof error === 'object' && error && 'code' in error && typeof error.code === 'string' ? error.code : undefined;
}
