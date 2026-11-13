import { useEffect, useRef } from 'react';
import type { MouseEvent, ReactNode } from 'react';
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
 * take, a press on the yes is still the end of the press that came before
 * it: about the time a system gives a double-click.
 */
export const PRESS_SETTLES_MS = 500;

/**
 * The question "Use on its own" asks before it checks a spelling, where the
 * action was: the consequence, then Use on its own and Cancel, Cancel
 * holding the focus so that Enter pressed on is not a yes. Escape takes
 * back the question and nothing else - not the dialog or the drawer it is
 * in. `note` is said after the consequence.
 *
 * The yes takes a press only when the press was meant for it. The question
 * opens where its action was, and the review panel then scrolls it clear of
 * the decision stuck to its bottom, which can bring the yes under the
 * pointer that pressed the action: the second press of a double-click on
 * the action, or a press straight after it, used a spelling on its own with
 * the sentence never read. So a press that is part of a double-click or
 * more, or that comes within PRESS_SETTLES_MS of the question appearing or
 * of a press the yes did not take, is no answer, and the focus goes back to
 * Cancel. A yes from the keyboard - the focus moved to it on purpose - is
 * taken at once.
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
  useEffect(() => {
    settled.current = performance.now() + PRESS_SETTLES_MS;
    cancel.current?.focus();
  }, []);
  const answer = (event: MouseEvent<HTMLButtonElement>) => {
    // A click from the keyboard, or from assistive technology, counts no presses.
    const now = performance.now();
    if (event.detail > 0 && (event.detail > 1 || now < settled.current)) {
      settled.current = now + PRESS_SETTLES_MS;
      cancel.current?.focus();
      return;
    }
    onConfirm();
  };
  return <div className="rule-confirm" role="group" aria-label={`Use on its own: ${spellingLabel(rule)}`} aria-describedby={describedBy} onKeyDown={(event) => {
    if (event.key === 'Escape') { event.preventDefault(); event.stopPropagation(); onCancel(); }
  }}>
    <p id={describedBy}>{onItsOwnConsequence(rule)} Do this only if that is right for every one of them.{note ? <> {note}</> : null}</p>
    <button type="button" className="primary" disabled={disabled} onClick={answer}>Use on its own</button>
    <button ref={cancel} type="button" onClick={onCancel}>Cancel</button>
  </div>;
}

/** The code of a failure the backend reported, when it has one. */
export function failureCode(error: unknown): string | undefined {
  return typeof error === 'object' && error && 'code' in error && typeof error.code === 'string' ? error.code : undefined;
}
