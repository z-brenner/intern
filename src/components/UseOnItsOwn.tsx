import { useEffect, useLayoutEffect, useRef } from 'react';
import type { MouseEvent, ReactNode } from 'react';
import { afterAPress, lastPointer, watchPointer } from '../lib/lastInput';
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
 * in, which take it back on Escape wherever the focus is in them. `note` is
 * said after the consequence.
 *
 * The yes takes a press of the pointer only once the pointer has come onto
 * it since the question appeared: moved onto it from a point off it, both
 * as the yes is at that moment, after the pointer was seen off it - or, a
 * finger having no hover, touched down on it from a point off it.
 * A yes that came under a pointer that did not move to it is no yes: the
 * question opening under the pointer that pressed its action, the row
 * below a closed question moving up, a scroll, a change of layout. Each
 * time the review panel or Settings was made to stop moving the yes under
 * the pointer, another way was found; this does not depend on how the
 * yes got there. Where the pointer was last seen is lastInput's
 * (`lastPointer`, `watchPointer`), and only a pointer seen somewhere new
 * counts: a move to where it already was is a browser saying what is now
 * under it. A pointer not seen yet, and one on the yes when the question
 * appeared, has to move off it and back.
 *
 * Two older rules stay behind it. A press on the yes is no answer when it
 * is part of a double-click or more, or when it began within
 * PRESS_SETTLES_MS of the question appearing or of a press the yes did not
 * take, however long it was held. And opened by a press, the question opens
 * where its action was, sentence first, with nothing moved under the
 * pointer (`afterAPress`): Cancel takes the focus where it is, and the
 * panel around it does not scroll, so the next press lands on the sentence.
 * A press on the question that misses its buttons leaves the focus on
 * Cancel. A press the yes refuses checks nothing and puts the focus back on
 * Cancel. A yes from the keyboard, or from assistive technology - the focus
 * moved to it on purpose - is taken at once.
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
  const yes = useRef<HTMLButtonElement>(null);
  const cancel = useRef<HTMLButtonElement>(null);
  /** When a press on the yes can first be an answer. */
  const settled = useRef(Number.POSITIVE_INFINITY);
  /** When the press now on the yes began. */
  const pressBegan = useRef<number | undefined>(undefined);
  /** Whether the pointer has come onto the yes since the question appeared. */
  const arrived = useRef(false);
  // Before the question is painted, and before its Cancel takes the focus
  // and may scroll it: where the yes is when the question appears.
  useLayoutEffect(() => {
    const onTheYes = (at: { x: number; y: number }) => {
      const box = yes.current?.getBoundingClientRect();
      return box !== undefined && at.x >= box.left && at.x < box.right && at.y >= box.top && at.y < box.bottom;
    };
    const appearedAt = lastPointer();
    let seenOff = appearedAt !== undefined && !onTheYes(appearedAt);
    arrived.current = false;
    return watchPointer((seen, before) => {
      if (!onTheYes(seen)) { seenOff = true; arrived.current = false; return; }
      // A mouse or a pen hovers, and comes onto the yes by moving: a press
      // where it was not seen to move to is not that. A finger has no
      // hover, and comes onto it by touching down.
      const coming = !seen.press || seen.type === 'touch';
      if (coming && seenOff && before !== undefined && !onTheYes(before)) arrived.current = true;
    });
  }, []);
  useEffect(() => {
    settled.current = performance.now() + PRESS_SETTLES_MS;
    cancel.current?.focus({ preventScroll: afterAPress() });
  }, []);
  const answer = (event: MouseEvent<HTMLButtonElement>) => {
    const now = performance.now();
    const began = pressBegan.current ?? now;
    // A click from the keyboard, or from assistive technology, counts no
    // presses; one that began with the pointer down on the yes, and no key
    // since, is the pointer's whatever it counts.
    const byPointer = event.detail > 0 || (pressBegan.current !== undefined && afterAPress());
    pressBegan.current = undefined;
    if (byPointer && (!arrived.current || event.detail > 1 || began < settled.current)) {
      settled.current = now + PRESS_SETTLES_MS;
      cancel.current?.focus({ preventScroll: true });
      return;
    }
    onConfirm();
  };
  // A press on the sentence focuses the row or the group around the
  // question: the focus goes back to Cancel.
  const missed = (event: MouseEvent<HTMLDivElement>) => {
    if (event.target instanceof Element && event.target.closest('button')) return;
    cancel.current?.focus({ preventScroll: true });
  };
  return <div className="rule-confirm" role="group" aria-label={`Use on its own: ${spellingLabel(rule)}`} aria-describedby={describedBy} onClick={missed} onKeyDown={(event) => {
    if (event.key === 'Escape') { event.preventDefault(); event.stopPropagation(); onCancel(); }
  }}>
    <p id={describedBy}>{onItsOwnConsequence(rule)} Do this only if that is right for every one of them.{note ? <> {note}</> : null}</p>
    <button ref={yes} type="button" className="primary" disabled={disabled} onPointerDown={() => { pressBegan.current = performance.now(); }} onClick={answer}>Use on its own</button>
    <button ref={cancel} type="button" onClick={onCancel}>Cancel</button>
  </div>;
}

/** The code of a failure the backend reported, when it has one. */
export function failureCode(error: unknown): string | undefined {
  return typeof error === 'object' && error && 'code' in error && typeof error.code === 'string' ? error.code : undefined;
}
