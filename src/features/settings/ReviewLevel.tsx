import { useId } from 'react';
import type { ReviewLevel } from '../../types';

/**
 * The two levels, each with what it does in a few sentences. Every sentence
 * has to be true of the engine's rule for the level, which is the only place
 * the rule is written (the review level's function in intern-engine). What
 * Balanced adds is said as what "can" be ready, since the rule holds some of
 * it back: a title standing in for a type the document did not confirm, a
 * description that is no sentence, a low rating the model gave itself. What
 * it says still waits, always does. "Could not confirm from the document"
 * is every reason about the date or a party, and a name with no date; the
 * model being unsure is not one of those, though Intern measures it in the
 * words of the date and the parties, which is why it is named apart. No
 * figures: how many more names are wrong depends on the documents.
 */
const CHOICES: { value: ReviewLevel; label: string; hint: string; cost?: string }[] = [
  {
    value: 'strict',
    label: 'Strict (default)',
    hint: 'Ready only when every check passes. Everything else waits for you.',
  },
  {
    value: 'balanced',
    label: 'Balanced',
    hint: 'Also ready when the only doubts are its type, its description or the model’s own confidence. Still waits when the date or a party cannot be confirmed, the name would have no document type, or ATLAS could not read it.',
    // On a line of its own, in the body's color: it is what the choice
    // costs, and at the end of the paragraph above it was easy to miss.
    cost: 'More documents are renamed without you looking, and more of those names will be wrong.',
  },
];

interface Props {
  level: ReviewLevel;
  onChange(level: ReviewLevel): void;
}

/**
 * Which documents are ready. It stands above the automatic-renaming checkbox
 * and says so in its first line, because it is not a part of that checkbox:
 * the level also decides what Rename all ready renames with automatic
 * renaming off.
 */
export function ReviewLevelChoice({ level, onChange }: Props) {
  const id = useId();
  return <fieldset className="review-level" aria-describedby={`${id}-lead ${id}-quick`}>
    <legend>Which documents are ready</legend>
    <p className="check-hint" id={`${id}-lead`}>Ready documents are the ones Rename all ready renames, or that ATLAS renames by itself if automatic renaming is on.</p>
    {CHOICES.map((choice) => <label key={choice.value} className="check-label">
      <input type="radio" name="review-level" value={choice.value} checked={level === choice.value} onChange={() => onChange(choice.value)} />
      <span>{choice.label}<span className="check-hint">{choice.hint}</span>{choice.cost && <span className="check-hint review-level-cost">{choice.cost}</span>}</span>
    </label>)}
    <p className="check-hint" id={`${id}-quick`}>Either way, a document the quick model read always waits for you.</p>
  </fieldset>;
}
