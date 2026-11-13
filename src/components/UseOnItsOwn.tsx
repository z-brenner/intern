import { useEffect, useLayoutEffect, useRef, useState } from 'react';
import type { MouseEvent, ReactNode } from 'react';
import { afterAPress, lastPointers, pressBeganAt, watchPointer } from '../lib/lastInput';
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
 * came before it: about the time a system gives a double-click. Opened by a
 * press, the time runs from when that press began, as a double-click is
 * timed from its first press to its second.
 */
export const PRESS_SETTLES_MS = 500;

type At = { x: number; y: number };
const inside = (at: At, box: DOMRect) => at.x >= box.left && at.x < box.right && at.y >= box.top && at.y < box.bottom;
const sameBox = (a: DOMRect, b: DOMRect) => a.left === b.left && a.top === b.top && a.width === b.width && a.height === b.height;

/**
 * The question "Use on its own" asks before it checks a spelling, where the
 * action was: the consequence, then Use on its own and Cancel, Cancel
 * holding the focus so that Enter pressed on is not a yes. Escape takes
 * back the question and nothing else - not the dialog or the drawer it is
 * in, which take it back on Escape wherever the focus is in them. `note` is
 * said after the consequence.
 *
 * The yes takes a press of the pointer only from a pointer that came onto
 * it since the question appeared and is on it still. A yes that came under
 * a pointer that did not move to it is no yes: the question opening under
 * the pointer that pressed its action, the row below a closed question
 * moving up, a scroll, a change of layout, a menu or a dialog over it
 * going away. Each time the review panel or Settings was made to stop
 * moving the yes under the pointer, another way was found; this does not
 * depend on how the yes got there. A pointer came onto the yes when:
 *
 * - it is seen on the yes itself - the element the browser found under
 *   it, not only inside the yes's box, so a pointer on a menu or a bar
 *   drawn over the yes is not on it;
 * - the same kind of pointer was last seen somewhere else, off the yes as
 *   it is now: a mouse, a pen and a finger are each their own, and a
 *   finger's tap says nothing of where the mouse is;
 * - and for a mouse or a pen, which hover, it moved there, with the yes
 *   standing where it was when that pointer was last seen: a pointer that
 *   moved onto a yes that had moved since came there by the yes moving. A
 *   finger has no hover, nor has a pen that does not report it: they come
 *   onto the yes by touching down on it, and a pen that hovers does so
 *   only with the yes standing where it was when it last hovered.
 *
 * Where each pointer was last seen is lastInput's (`lastPointers`,
 * `watchPointer`). A report of a pointer where it already was is no move:
 * it says what is under it now, and when that is not the yes, the pointer
 * is no longer on it. A pointer that came onto the yes stops being on it
 * when it is seen anywhere else, or when the yes moves - it is watched
 * every frame while a pointer is on it - away and back included. A press
 * of the pointer is one that began on the yes, by a pointer that had come
 * onto it then and still has. A press refused because its pointer had not
 * says, beside the buttons, to move the pointer off the yes and back - or
 * for a finger, to tap the sentence first - so that a person who means it
 * is not left pressing a yes that does nothing; nothing moves under the
 * pointer for it.
 *
 * A press never gives the yes the focus: a press with another button, or
 * one let go off the yes, clicks nothing, and the focus it would have left
 * on the yes made the next Enter or Space a yes from the keyboard. The
 * focus comes to the yes only from a key, or from assistive technology,
 * and a yes from there is taken at once.
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
 * Cancel.
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
  /** The press now on the yes: the kind of pointer, when it began, and whether that pointer had come onto the yes. */
  const press = useRef<{ type: string; began: number; came: boolean } | undefined>(undefined);
  /** The kind of pointer whose press was refused because it had not come onto the yes. */
  const [notOnIt, setNotOnIt] = useState<string>();
  /** Whether a pointer of this kind came onto the yes and is on it still. */
  const onIt = useRef<(type: string) => boolean>(() => false);
  // Before the question is painted, and before its Cancel takes the focus
  // and may scroll it: where the yes is when the question appears.
  useLayoutEffect(() => {
    const drawn = () => yes.current?.getBoundingClientRect() ?? new DOMRect();
    // Where each kind of pointer was last seen, and where the yes was then.
    const seen = new Map<string, At & { hover: boolean; box: DOMRect }>();
    const appeared = drawn();
    for (const at of lastPointers()) seen.set(at.type, { x: at.x, y: at.y, hover: at.hover, box: appeared });
    // The pointers on the yes, each with where the yes was when it came.
    const came = new Map<string, DOMRect>();
    const stood = () => {
      const box = drawn();
      for (const [type, at] of came) if (!sameBox(at, box)) came.delete(type);
    };
    let frame = 0;
    const watch = () => { frame = 0; stood(); if (came.size > 0) frame = requestAnimationFrame(watch); };
    onIt.current = (type) => { stood(); return came.has(type); };
    const stop = watchPointer((at, under) => {
      const box = drawn();
      const before = seen.get(at.type);
      const onTheYes = inside(at, box) && under instanceof Node && yes.current?.contains(under) === true;
      if (!onTheYes) came.delete(at.type);
      if (before !== undefined && before.x === at.x && before.y === at.y) return;
      seen.set(at.type, { x: at.x, y: at.y, hover: at.hover, box });
      if (!onTheYes || before === undefined || inside(before, box)) return;
      // A mouse hovers, and comes onto the yes by moving, from where it was
      // seen with the yes where it is: a press where it was not seen to
      // move to is not that. A finger has no hover, nor has a pen that
      // does not report it: they come onto it by touching down, from where
      // they last touched off it. A pen that hovers is held to what a mouse
      // is: touching down from where it hovered counts only when the yes
      // stood still since.
      if (at.press ? at.type === 'mouse' || (before.hover && !sameBox(before.box, box)) : !sameBox(before.box, box)) return;
      came.set(at.type, box);
      if (frame === 0) frame = requestAnimationFrame(watch);
    });
    return () => { stop(); cancelAnimationFrame(frame); onIt.current = () => false; };
  }, []);
  useEffect(() => {
    const now = performance.now();
    const opening = pressBeganAt();
    settled.current = (opening !== undefined && now - opening < PRESS_SETTLES_MS ? opening : now) + PRESS_SETTLES_MS;
    cancel.current?.focus({ preventScroll: afterAPress() });
  }, []);
  const answer = (event: MouseEvent<HTMLButtonElement>) => {
    const now = performance.now();
    const pressed = press.current;
    press.current = undefined;
    // A click from the keyboard, or from assistive technology, counts no
    // presses; one that began with the pointer down on the yes, and no key
    // since, is the pointer's whatever it counts.
    const byPointer = event.detail > 0 || (pressed !== undefined && afterAPress());
    const came = pressed !== undefined && pressed.came && onIt.current(pressed.type);
    if (byPointer && (!came || event.detail > 1 || pressed.began < settled.current)) {
      if (!came) setNotOnIt(pressed?.type ?? 'mouse');
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
    <span className="rule-confirm-buttons"><button ref={yes} type="button" className="primary" disabled={disabled}
      onPointerDown={(event) => { press.current = { type: event.pointerType, began: performance.now(), came: onIt.current(event.pointerType) }; }}
      onMouseDown={(event) => event.preventDefault()}
      onClick={answer}>Use on its own</button>
    <button ref={cancel} type="button" onClick={onCancel}>Cancel</button></span>
    <div className="rule-confirm-again" role="status">{notOnIt === undefined ? null : notOnIt === 'touch'
      ? 'Tap the sentence, then Use on its own.'
      : 'Move off Use on its own, then back.'}</div>
  </div>;
}

/** The code of a failure the backend reported, when it has one. */
export function failureCode(error: unknown): string | undefined {
  return typeof error === 'object' && error && 'code' in error && typeof error.code === 'string' ? error.code : undefined;
}
