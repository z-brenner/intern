import { useEffect, useRef } from 'react';
import type { MouseEvent, ReactNode } from 'react';
import { afterAPress } from '../lib/lastInput';
import type { HouseRule } from '../types';

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
 * How long after the question appears, or after a press its yes did not
 * take, a press that begins on the yes is still the end of the press that
 * came before it: about the time a system gives a double-click.
 */
export const PRESS_SETTLES_MS = 500;

/**
 * The question "Use on its own" asks before it checks a spelling, where the
 * action was: the consequence, then Use on its own and Cancel, Cancel
 * holding the focus so that Enter pressed on is not a yes. Escape takes
 * back the question and nothing else - not the dialog or the drawer it is
 * in. `note` is said after the consequence.
 *
 * The yes takes a press only when the press was meant for it. Opened by a
 * press, the question opens where its action was, sentence first, and
 * nothing moves under the pointer (`afterAPress`): the Cancel takes the
 * focus where it is, and the panel around it does not scroll. The review
 * panel used to scroll the question clear of the decision stuck to its
 * bottom, which could bring the yes under the pointer that pressed the
 * action: a double-click a little slower than the system's, or a press
 * again because nothing seemed to happen, used a spelling on its own with
 * the sentence never read. So the next press lands on the sentence, and a
 * press on the question that misses its buttons leaves the focus on
 * Cancel, where Escape still takes back the question and nothing else.
 * And a press on the yes is no answer when it is part of a double-click or
 * more, or when it began within PRESS_SETTLES_MS of the question appearing
 * or of a press the yes did not take, however long it was held; the focus
 * goes back to Cancel. A yes from the keyboard - the focus moved to it on
 * purpose - is taken at once.
 */
export function UseOnItsOwnQuestion({ rule, describedBy, note, disabled, onConfirm, onCancel }: {
  rule: HouseRule;
  /** The id of the sentence, for the group's description. */
  describedBy: string;
  note?: ReactNode;
  disabled?: boolean;
  onConfirm(): void;
  onCancel(): void;
}) {
  const cancel = useRef<HTMLButtonElement>(null);
  /** When a press on the yes can first be an answer. */
  const settled = useRef(Number.POSITIVE_INFINITY);
  /** When the press now on the yes began. */
  const pressBegan = useRef<number | undefined>(undefined);
  useEffect(() => {
    settled.current = performance.now() + PRESS_SETTLES_MS;
    cancel.current?.focus({ preventScroll: afterAPress() });
  }, []);
  const answer = (event: MouseEvent<HTMLButtonElement>) => {
    // A click from the keyboard, or from assistive technology, counts no presses.
    const now = performance.now();
    const began = pressBegan.current ?? now;
    pressBegan.current = undefined;
    if (event.detail > 0 && (event.detail > 1 || began < settled.current)) {
      settled.current = now + PRESS_SETTLES_MS;
      cancel.current?.focus({ preventScroll: true });
      return;
    }
    onConfirm();
  };
  // A press on the sentence focuses the row or the group around the
  // question, where Escape closes the drawer or the dialog: the focus goes
  // back to Cancel.
  const missed = (event: MouseEvent<HTMLDivElement>) => {
    if (event.target instanceof Element && event.target.closest('button')) return;
    cancel.current?.focus({ preventScroll: true });
  };
  return <div className="rule-confirm" role="group" aria-label={`Use on its own: ${spellingLabel(rule)}`} aria-describedby={describedBy} onClick={missed} onKeyDown={(event) => {
    if (event.key === 'Escape') { event.preventDefault(); event.stopPropagation(); onCancel(); }
  }}>
    <p id={describedBy}>{onItsOwnConsequence(rule)} Do this only if that is right for every one of them.{note ? <> {note}</> : null}</p>
    <button type="button" className="primary" disabled={disabled} onPointerDown={() => { pressBegan.current = performance.now(); }} onClick={answer}>Use on its own</button>
    <button ref={cancel} type="button" onClick={onCancel}>Cancel</button>
  </div>;
}

/** The code of a failure the backend reported, when it has one. */
export function failureCode(error: unknown): string | undefined {
  return typeof error === 'object' && error && 'code' in error && typeof error.code === 'string' ? error.code : undefined;
}
