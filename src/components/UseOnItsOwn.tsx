import { useEffect, useRef } from 'react';
import type { ReactNode } from 'react';
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
 * The question "Use on its own" asks before it checks a spelling, where the
 * action was: the consequence, then Use on its own and Cancel, Cancel
 * holding the focus so that Enter pressed on is not a yes. Escape takes
 * back the question and nothing else - not the dialog or the drawer it is
 * in. `note` is said after the consequence.
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
  useEffect(() => { cancel.current?.focus(); }, []);
  return <div className="rule-confirm" role="group" aria-label={`Use on its own: ${spellingLabel(rule)}`} aria-describedby={describedBy} onKeyDown={(event) => {
    if (event.key === 'Escape') { event.preventDefault(); event.stopPropagation(); onCancel(); }
  }}>
    <p id={describedBy}>{onItsOwnConsequence(rule)} Do this only if that is right for every one of them.{note ? <> {note}</> : null}</p>
    <button type="button" className="primary" disabled={disabled} onClick={onConfirm}>Use on its own</button>
    <button ref={cancel} type="button" onClick={onCancel}>Cancel</button>
  </div>;
}

/** The code of a failure the backend reported, when it has one. */
export function failureCode(error: unknown): string | undefined {
  return typeof error === 'object' && error && 'code' in error && typeof error.code === 'string' ? error.code : undefined;
}
