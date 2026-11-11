import { describe, expect, it } from 'vitest';
import { describeActionError } from './actionErrors';
import { describeQueueStop, humanizeReason } from './reasons';

describe('humanizeReason', () => {
  it('translates a single code into its sentence', () => {
    expect(humanizeReason('SOURCE_LOCKED')).toBe('The document is open in another program, or a sync client is still writing it. Close it, then try again.');
  });

  // A rename refused by a held file is rolled back and the document waits in
  // review under SOURCE_LOCKED: the refusal shown at the moment of approving
  // and the reason shown on the item afterwards are the same sentence.
  it('says one thing about a held file, as a reason and as a refusal', () => {
    expect(describeActionError({ code: 'SOURCE_LOCKED', message: 'atomic no-replace rename failed (os error 32)' })).toBe(humanizeReason('SOURCE_LOCKED'));
  });

  // Pipeline::apply_if_unchanged refuses an approval of a file that changed
  // since it was read with "Re-analyze it.", and the panel calls that action
  // Analyze again. The refusal and the item's reason afterwards both name the
  // control a person can find.
  it('points a refused approval of a changed file at Analyze again, the action the panel offers', () => {
    const refusal = { code: 'FILE_CHANGED', message: 'The file changed after it was analyzed. Re-analyze it.' };
    for (const sentence of [describeActionError(refusal, { approve: true }), humanizeReason('FILE_CHANGED')]) {
      expect(sentence).toContain('Use Analyze again');
      expect(sentence).not.toMatch(/Re-analyze/);
    }
    // An undo refused with the same code is about a filed copy that was
    // moved; the backend names where it was, and that is kept.
    const undo = { code: 'FILE_CHANGED', message: 'The filed document is no longer at C:\\Filed\\Lease.pdf; it was moved or deleted.' };
    expect(describeActionError(undo)).toBe(undo.message);
  });

  // Check again runs reconciliation, which refuses with the same code the
  // item was parked under when the files are still ambiguous. Sending the
  // person back to Check again from its own failure was a loop.
  it('does not send a failed Check again back to Check again', () => {
    const refusal = { code: 'RECONCILIATION_REQUIRED', message: 'an incomplete operation left a file at both of its paths' };
    const afterCheck = describeActionError(refusal, { checkedFiles: true });
    expect(afterCheck).not.toMatch(/Check again/);
    expect(afterCheck).toContain('\u201cI have resolved the files myself\u201d');
    // Keep or remove of an item that was parked after the panel last showed
    // it: checking is the way on, and the panel now offers it.
    expect(describeActionError(refusal)).toContain('Use Check again.');
  });

  it('translates the pipeline comma-joined list into sentences', () => {
    const result = humanizeReason('TYPE_UNSUPPORTED, LOW_CONFIDENCE, MODEL_REQUESTED_REVIEW');
    expect(result).toBe(
      'The proposed document type does not appear verbatim in the document. '
      + 'The model was unsure of its reading. Check the name against the document, starting with the date and the parties. '
      + 'The model asked for a person to look at this one.',
    );
  });

  // LOW_CONFIDENCE is raised when the model rates itself low, and also when
  // it rates itself at 100% and Intern measures doubt in the words of the
  // date or a party. The sentence must be true of both, so it does not say
  // the model "reported" anything, and it says where to look.
  it('says a doubted reading without claiming the model reported its doubt', () => {
    const sentence = humanizeReason('LOW_CONFIDENCE');
    expect(sentence).toBe('The model was unsure of its reading. Check the name against the document, starting with the date and the parties.');
    expect(sentence).not.toMatch(/reported|low confidence/i);
  });

  // PARTY_MISSING arrives with PARTY_UNSUPPORTED, in either order, and the
  // two end on the same instruction. It is said once, after the last of
  // them, and each still ends on it alone.
  it('says where to look about the parties once, after the last party sentence', () => {
    const check = 'Check who the document is from, to, for or between.';
    for (const code of ['PARTY_UNSUPPORTED', 'PARTY_MISSING']) {
      const sentence = humanizeReason(code);
      expect(sentence.endsWith(` ${check}`), sentence).toBe(true);
      expect(sentence.split(check), code).toHaveLength(2);
    }
    const missing = 'The name lacks a party it needs, such as the other side of an agreement or who issued the document.';
    for (const reason of ['DATE_AMBIGUOUS, PARTY_UNSUPPORTED, PARTY_MISSING, LOW_CONFIDENCE', 'PARTY_MISSING, PARTY_UNSUPPORTED, LOW_CONFIDENCE']) {
      const text = humanizeReason(reason);
      expect(text.split(check), reason).toHaveLength(2);
      expect(text, reason).toContain(`${check} The model was unsure of its reading.`);
      expect(text, reason).toContain(missing);
    }
    expect(humanizeReason('PARTY_UNSUPPORTED, PARTY_MISSING')).toBe(`${humanizeReason('PARTY_UNSUPPORTED').slice(0, -check.length)}${missing} ${check}`);
  });

  // PARTY_UNSUPPORTED is raised most often for a side only the model's
  // reading gave (99 of 160 in the 9B's replays), not a party the document
  // lacks, and also for a name with no party left in it; the sentence says
  // so and assumes no party.
  it('names the usual cause of an uncertain party, and does not assume the name has one', () => {
    const sentence = humanizeReason('PARTY_UNSUPPORTED');
    expect(sentence).toMatch(/^Intern could not confirm from the document which party this name should give\. Most often the document does not say which side a party is on/);
    expect(sentence).not.toMatch(/the party in this name|the proposed party/);
  });

  it('tolerates lowercase codes from other serializations', () => {
    expect(humanizeReason('date_unsupported')).toContain('verbatim');
  });

  it('tells a near-duplicate apart from an exact one', () => {
    expect(humanizeReason('NEAR_DUPLICATE')).toBe('This looks like a document that was filed already. Approve to file it as well, keep the original, or remove it.');
    expect(humanizeReason('LOW_CONFIDENCE, NEAR_DUPLICATE')).toContain('filed already');
  });

  it('explains a bare duplicate flag once the filed name it referred to is gone', () => {
    // Named as the review menu names it: a review item has no Retry.
    expect(humanizeReason('DUPLICATE')).toBe('This document\'s content was filed once already, so Intern did not file it again. Choose Process anyway to process it all the same, keep it where it is, or remove it.');
  });

  it('names the date gate and the hosted-model failures in plain words', () => {
    expect(humanizeReason('DATE_REQUIRED')).toContain('YYYY-MM-DD');
    expect(humanizeReason('HOSTED_MODEL_UNAUTHORIZED')).toContain('rejected the API key');
    expect(humanizeReason('MODEL_DECLINED')).toContain('declined');
  });

  it('keeps unknown codes and free text verbatim', () => {
    expect(humanizeReason('SOMETHING_NEW')).toBe('SOMETHING_NEW');
    expect(humanizeReason('Duplicate of 2026-01-05 Invoice.pdf')).toBe('Duplicate of 2026-01-05 Invoice.pdf');
  });

  it('never rewrites a list containing an unknown entry, so free-text commas survive', () => {
    const name = 'Duplicate of 2026-04-01 SOW between Ridgeline, LLC and Contoso Worldwide, Inc.pdf';
    expect(humanizeReason(name)).toBe(name);
    expect(humanizeReason('TYPE_UNSUPPORTED, SOMETHING_NEW')).toBe('TYPE_UNSUPPORTED, SOMETHING_NEW');
  });

  it('has a plain sentence for every failure and review code the backend can store', () => {
    const codes = [
      'DATE_IMPLAUSIBLE', 'DATE_IS_DEADLINE', 'DATE_AMBIGUOUS', 'FAST_MODEL_READING', 'PARTY_MISSING',
      'PASSWORD_PROTECTED', 'UNSUPPORTED_CONTENT', 'DOCUMENT_TOO_LARGE', 'OCR_UNAVAILABLE',
      'EXTRACTION_FAILED', 'ANALYSIS_FAILED', 'MODEL_FAILED', 'MODEL_OUTPUT_INVALID',
      'MODEL_INPUT_TOO_LARGE', 'MODEL_REPLY_TRUNCATED', 'HOSTED_MODEL_UNAVAILABLE',
      'HOSTED_MODEL_BILLING', 'STATE_CONFLICT', 'INVALID_TRANSITION', 'ALREADY_NAMED',
      'INTAKE_WITHDRAWN', 'SECRET_SERVICE_UNAVAILABLE', 'EXACT_COPY',
    ];
    for (const code of codes) {
      const sentence = humanizeReason(code);
      expect(sentence, code).not.toBe(code);
      expect(sentence, code).toMatch(/[.]$/);
    }
  });
});

describe('describeQueueStop', () => {
  // A stopped queue waits for Resume. The sentence a single document gets for
  // the same code says to retry that document, or that Intern will retry it,
  // and neither is what a person reading the banner has to do.
  it('says to resume the queue, not to retry a document, for the pauses a document sentence would get wrong', () => {
    for (const code of ['MODEL_FAILED', 'MODEL_OUTPUT_INVALID', 'HOSTED_MODEL_RATE_LIMITED', 'HOSTED_MODEL_UNREACHABLE', 'READER_CRASHED', 'READER_CHECK_UNWRITABLE']) {
      const sentence = describeQueueStop(code);
      expect(sentence, code).toMatch(/resume the queue/i);
      expect(sentence, code).not.toMatch(/Retry it|Intern will retry/);
      expect(sentence, code).not.toBe(humanizeReason(code));
    }
    expect(describeQueueStop('HOSTED_MODEL_RATE_LIMITED')).toBe('The hosted service asked for a slower pace. Wait a minute, then resume the queue.');
  });

  it('falls back to the document sentence where that one already fits, and keeps unknown codes', () => {
    expect(describeQueueStop('HOSTED_MODEL_BILLING')).toBe(humanizeReason('HOSTED_MODEL_BILLING'));
    expect(describeQueueStop('OCR_UNAVAILABLE')).toBe(humanizeReason('OCR_UNAVAILABLE'));
    expect(describeQueueStop('LEASE_LOST')).toBe('LEASE_LOST');
  });
});

describe('a quick-model reading', () => {
  it('says who read it and what the person can do', () => {
    expect(humanizeReason('FAST_MODEL_READING')).toBe('Read by the quick model — check it, or wait for the accurate model to read it again.');
  });
});
