import { useEffect, useRef, useState } from 'react';
import type { ReactNode } from 'react';
import { afterAPress } from '../lib/lastInput';
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

/** The letters and digits of `text`, in order, without case or accents: "Müller & Co." is "mullerco". */
const lettersOf = (text: string) => text.normalize('NFKD').replace(/\p{M}/gu, '').toLowerCase().replace(/[^\p{L}\p{N}]/gu, '');

/**
 * Whether `typed` is `spelling` typed out: the same letters and digits in
 * the same order, whatever the case, the accents, the spaces and the
 * punctuation - "contoso ltd" is "Contoso Ltd.", "Muller" is "Müller", and
 * a letter missing or added is not. A spelling with no letter or digit in
 * it is typed exactly, spaces at its ends aside.
 */
export function typedOut(typed: string, spelling: string): boolean {
  const letters = lettersOf(spelling);
  if (letters !== '') return lettersOf(typed) === letters;
  return typed.trim() !== '' && typed.trim() === spelling.trim();
}

/**
 * What typing puts in a field, or takes out: keys, an input method,
 * dictation, a deletion. Not a paste, a drop, a suggestion taken (a
 * spelling, an autocorrection, autofill, which says no input type at all),
 * a yank, or an undo.
 */
const typing = (inputType: string | undefined) =>
  inputType === 'insertText' || inputType === 'insertCompositionText' || inputType === 'insertFromComposition' || inputType?.startsWith('delete') === true;

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
 * typing fills the field: a paste, a drop, a suggestion taken and an undo
 * are refused, and the browser is asked for no suggestions to take. Once
 * the spelling is typed, the yes is an ordinary button.
 *
 * Keys typed in the field stay there, but for Tab, and Escape: Ctrl+Enter,
 * Alt+K and Alt+N neither approve, keep nor move on in review, and Enter
 * saves no settings. The field starts empty for every question, and again
 * when the spelling it asks about changes while it asks. Opened by a press,
 * the question takes the focus in its field and moves nothing under the
 * pointer (`afterAPress`); opened from the keyboard, it is brought into
 * sight.
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
  const ready = !disabled && typedOut(typed, rule.to);
  useEffect(() => {
    const input = field.current;
    if (!afterAPress()) group.current?.scrollIntoView?.({ block: 'nearest' });
    input?.focus({ preventScroll: true });
    const refuse = (event: InputEvent) => { if (!typing(event.inputType)) event.preventDefault(); };
    input?.addEventListener('beforeinput', refuse);
    return () => input?.removeEventListener('beforeinput', refuse);
  }, []);
  const answer = () => { if (ready) onConfirm(); };
  return <div ref={group} className="rule-confirm" role="group" aria-label={`Use on its own: ${spellingLabel(rule)}`} aria-describedby={describedBy} onKeyDown={(event) => {
    if (event.key === 'Escape' && !event.nativeEvent.isComposing) { event.preventDefault(); event.stopPropagation(); onCancel(); }
  }}>
    <p id={describedBy}>{onItsOwnConsequence(rule)} Do this only if that is right for every one of them.{note ? <> {note}</> : null}</p>
    <label className="rule-confirm-field">Type <q>{rule.to}</q> to use it on its own
      <input ref={field} type="text" name={name} value={typed} autoComplete="off" autoCorrect="off" autoCapitalize="off" spellCheck={false}
        onChange={(event) => { const input = event.nativeEvent; if (input instanceof InputEvent && typing(input.inputType)) setTyped(event.target.value); }}
        onPaste={(event) => event.preventDefault()}
        onDrop={(event) => event.preventDefault()}
        onKeyDown={(event) => {
          // An input method's keys, the Enter that commits what it composed among them.
          const composing = event.nativeEvent.isComposing || event.keyCode === 229;
          if ((event.key === 'Tab' || event.key === 'Escape') && !composing) return;
          event.stopPropagation();
          if (event.key !== 'Enter' || composing) return;
          event.preventDefault();
          if (!event.repeat && !event.altKey && !event.ctrlKey && !event.metaKey && !event.shiftKey) answer();
        }} />
    </label>
    <span className="rule-confirm-buttons"><button type="button" className="primary" disabled={!ready} onClick={answer}>Use on its own</button>
      <button type="button" onClick={onCancel}>Cancel</button></span>
  </div>;
}

/** The code of a failure the backend reported, when it has one. */
export function failureCode(error: unknown): string | undefined {
  return typeof error === 'object' && error && 'code' in error && typeof error.code === 'string' ? error.code : undefined;
}
