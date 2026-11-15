import { describe, expect, it } from 'vitest';
import { describeActionError } from './actionErrors';
import { describeNotes, describeQueueStop, humanizeReason, noteCodes, outcomeFirst, shortNote, shortOutcome, shortReason } from './reasons';

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
      'The document does not confirm the proposed type as its own. The type does not appear verbatim in the document, or appears only where the document says what it is about, such as a subject line or the text of a letter. Check the type. '
      + 'The model was unsure of its reading. Check the name against the document, starting with the date and the parties. '
      + 'The model asked for a person to look at this one.',
    );
  });

  // TYPE_UNSUPPORTED is raised for a type the document does not state, and
  // also for one it states only where it says what it is about: a subject
  // line a record heads, a letter's text below its greeting. TYPE_INFERRED
  // is raised when a title stands in for the model's type and when the
  // layout's kind does ("Letter", "Email", "Notice"), whether or not the
  // model gave a type. Each sentence must be true of every case, so the
  // first does not say the type is absent and the second does not say the
  // model gave none or that a title was used.
  it('says the type reasons without claiming the type is absent or that the model gave none', () => {
    const unsupported = humanizeReason('TYPE_UNSUPPORTED');
    expect(unsupported).toMatch(/^The document does not confirm the proposed type as its own\./);
    expect(unsupported).toMatch(/does not appear verbatim in the document, or appears only where/);
    expect(unsupported.endsWith(' Check the type.'), unsupported).toBe(true);
    const inferred = humanizeReason('TYPE_INFERRED');
    expect(inferred).toMatch(/own title, or the kind its layout shows/);
    expect(inferred).not.toMatch(/gave no|title was used/);
    expect(shortReason(['TYPE_UNSUPPORTED'])).toEqual({ text: 'Type uncertain', more: 0 });
    expect(shortReason(['TYPE_INFERRED'])).toEqual({ text: 'Type supplied by Intern', more: 0 });
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

  // A name a learned spelling changed waits for a person until they choose
  // "Use on its own" for the spelling: the sentence says why, and what to do
  // either way - approve a right name, and choose Use on its own in
  // Settings to stop being asked, or put the document's word back. It does not say an
  // approval checks the spelling: none does. Nor does it promise a put-back
  // forgets the spelling whatever else changed: one made with another part
  // of the name changed teaches nothing (tests/pipeline.rs
  // the_documents_own_word_put_back_is_no_check_of_the_rule).
  it('says a name a learned spelling changed waits until the spelling is used on its own, and what to do if the name is right or not', () => {
    const sentence = humanizeReason('LEARNED_SPELLING_UNCONFIRMED');
    expect(sentence).toBe('Intern used a spelling it learned from your edits. Until you tell it to use that spelling on its own, a name it changes waits for you. If the name is right, approve it; to stop being asked, choose Use on its own for the spelling in Settings, under Learned spellings. If not, put the document\'s own words back - when that is the only change, Intern stops using that spelling.');
    // It is chosen in Settings, not beside this sentence in the review window.
    expect(sentence).toMatch(/Use on its own for the spelling in Settings, under Learned spellings/);
    expect(sentence).not.toMatch(/first time/);
    expect(sentence).not.toMatch(/approv\w* (it )?checks|checked by approv/i);
    expect(shortReason(['LEARNED_SPELLING_UNCONFIRMED'])).toEqual({ text: 'Learned spelling', more: 0 });
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

  // An invoice dated with the first day of the week it bills. The date is in
  // the document, in numbers that read one way, and is no deadline: neither
  // DATE_AMBIGUOUS's sentence nor DATE_IS_DEADLINE's is true of it.
  it('says a date that only bounds a period is the period\'s, and to check for the document\'s own', () => {
    const sentence = humanizeReason('DATE_IS_PERIOD');
    expect(sentence).toBe('The date is written only as the start or end of a period, such as a billing or service period. Check which date the document itself carries.');
    expect(sentence).not.toMatch(/day-first|month-first|due|expiry/);
    expect(humanizeReason('DATE_IS_PERIOD, LOW_CONFIDENCE')).toBe(`${sentence} ${humanizeReason('LOW_CONFIDENCE')}`);
  });

  // An invoice template whose date cell holds =TODAY(). The date is in the
  // text, written one way, labeled as the invoice's own and joined to no
  // period: none of the other date sentences is true of it.
  it('says a date only a TODAY formula shows comes from the formula, and to check which date the document should carry', () => {
    const sentence = humanizeReason('DATE_IS_FORMULA');
    expect(sentence).toBe('The date comes from a spreadsheet formula that uses today\'s date, so it can change each time the file is opened. Check which date the document should carry.');
    expect(sentence).not.toMatch(/day-first|month-first|due|expiry|period|verbatim|year/);
    expect(humanizeReason('DATE_IS_FORMULA, LOW_CONFIDENCE')).toBe(`${sentence} ${humanizeReason('LOW_CONFIDENCE')}`);
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
      'DATE_IMPLAUSIBLE', 'DATE_IS_DEADLINE', 'DATE_AMBIGUOUS', 'DATE_IS_PERIOD', 'DATE_IS_FORMULA', 'FAST_MODEL_READING', 'PARTY_MISSING',
      'PASSWORD_PROTECTED', 'UNSUPPORTED_CONTENT', 'DOCUMENT_TOO_LARGE', 'OCR_UNAVAILABLE',
      'EXTRACTION_FAILED', 'ANALYSIS_FAILED', 'MODEL_FAILED', 'MODEL_OUTPUT_INVALID',
      'MODEL_INPUT_TOO_LARGE', 'MODEL_REPLY_TRUNCATED', 'HOSTED_MODEL_UNAVAILABLE',
      'HOSTED_MODEL_BILLING', 'STATE_CONFLICT', 'INVALID_TRANSITION', 'ALREADY_NAMED',
      'INTAKE_WITHDRAWN', 'SECRET_SERVICE_UNAVAILABLE', 'EXACT_COPY', 'LEARNED_SPELLING_UNCONFIRMED',
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

/*
  At the Balanced review level a ready document can carry review codes, and
  one Intern then renames by itself is filed with them. There they are notes.
  A review's sentence that tells the person to check something, or that the
  document waits, would be false of a document nothing holds.
*/
describe('a note on a document that is not waiting', () => {
  // The codes the Balanced level lets a ready document carry (the level's rule is the engine's).
  const carried = ['TYPE_MISSING', 'TYPE_UNSUPPORTED', 'TYPE_INFERRED', 'DESCRIPTION_UNSUPPORTED', 'LOW_CONFIDENCE'];

  it('says what Intern found, tells nobody to check, and says of nothing that it waits', () => {
    for (const code of carried) {
      const note = describeNotes([code]);
      expect(note, code).not.toBe(code);
      expect(note, code).toMatch(/[.]$/);
      // No instruction, and nothing about waiting or approving: the document is ready.
      expect(note, code).not.toMatch(/(^|\. )Check\b|\bwaits?\b|\bapprove\b/i);
    }
    expect(describeNotes(['TYPE_UNSUPPORTED'])).toBe('The document does not confirm the type in this name as its own. The type does not appear verbatim in the document, or appears only where the document says what it is about, such as a subject line or the text of a letter.');
    expect(describeNotes(['TYPE_INFERRED'])).toBe('Intern supplied the type in this name: the document\'s own title, or the kind its layout shows, such as “Letter” or “Email”.');
    // Where to look is still said, for someone who does look.
    expect(describeNotes(['LOW_CONFIDENCE'])).toBe('The model was unsure of its reading. If you look at this name, start with the date and the parties.');
    // These two already say what was found and ask for nothing.
    expect(describeNotes(['TYPE_MISSING'])).toBe(humanizeReason('TYPE_MISSING'));
    expect(describeNotes(['DESCRIPTION_UNSUPPORTED'])).toBe(humanizeReason('DESCRIPTION_UNSUPPORTED'));
  });

  it('differs from the review sentence exactly where that one ends on an instruction', () => {
    for (const code of ['TYPE_UNSUPPORTED', 'TYPE_INFERRED', 'LOW_CONFIDENCE']) {
      expect(humanizeReason(code), code).toMatch(/\. Check [^.]+\.$/);
      expect(describeNotes([code]), code).not.toBe(humanizeReason(code));
    }
    // The note keeps what the review sentence says was found.
    expect(humanizeReason('TYPE_UNSUPPORTED')).toContain(describeNotes(['TYPE_UNSUPPORTED']).replace('the type in this name', 'the proposed type'));
    expect(humanizeReason('TYPE_INFERRED').startsWith(describeNotes(['TYPE_INFERRED']))).toBe(true);
    expect(humanizeReason('LOW_CONFIDENCE').startsWith('The model was unsure of its reading.')).toBe(true);
  });

  it('says several notes in the order they came', () => {
    expect(describeNotes(['TYPE_INFERRED', 'LOW_CONFIDENCE'])).toBe(`${describeNotes(['TYPE_INFERRED'])} ${describeNotes(['LOW_CONFIDENCE'])}`);
    expect(describeNotes(['type_inferred', ' LOW_CONFIDENCE '])).toBe(describeNotes(['TYPE_INFERRED', 'LOW_CONFIDENCE']));
    expect(describeNotes([])).toBe('');
  });

  it('keeps the sentence of a code with no note of its own, and an unknown code as it came', () => {
    expect(describeNotes(['DATE_AMBIGUOUS'])).toBe(humanizeReason('DATE_AMBIGUOUS'));
    expect(describeNotes(['PARTY_UNSUPPORTED', 'PARTY_MISSING'])).toBe(humanizeReason('PARTY_UNSUPPORTED, PARTY_MISSING'));
    expect(describeNotes(['SOMETHING_NEW'])).toBe('SOMETHING_NEW');
    expect(describeNotes(['TYPE_INFERRED', 'SOMETHING_NEW'])).toBe(`${describeNotes(['TYPE_INFERRED'])} SOMETHING_NEW`);
  });

  it('does not count what became of the file as a note about its reading', () => {
    expect(noteCodes(['TYPE_INFERRED', 'ALREADY_NAMED', 'LOW_CONFIDENCE', 'EXACT_COPY'])).toEqual(['TYPE_INFERRED', 'LOW_CONFIDENCE']);
    expect(noteCodes(['ALREADY_NAMED'])).toEqual([]);
    expect(noteCodes(['EXACT_COPY'])).toEqual([]);
    // Anything else is a note, a code nobody has heard of included: it is shown, not dropped.
    expect(noteCodes(['SOMETHING_NEW'])).toEqual(['SOMETHING_NEW']);
  });

  /*
    The few words for a note are the review's, except where those would be
    false of a document that is ready or renamed. TYPE_MISSING is raised for
    a type that is a general word alone ("Letter"), and the level lets such a
    reading be ready only when its name has a type: "No document type" then
    contradicts the name beside it.
  */
  it('has a few words for every note, none of which says the name has no type', () => {
    for (const code of carried) {
      const short = shortNote([code]);
      expect(short, code).toBeDefined();
      expect(short?.text, code).not.toMatch(/^No document type$/);
      expect(short?.more, code).toBe(0);
    }
    expect(shortNote(['TYPE_MISSING'])).toEqual({ text: 'No specific type', more: 0 });
    // The others are the review's own few words.
    for (const code of carried.filter((code) => code !== 'TYPE_MISSING')) expect(shortNote([code]), code).toEqual(shortReason([code]));
    // Ranked as a review's are: a note about a part of the name leads, the rest are counted.
    expect(shortNote(['LOW_CONFIDENCE', 'TYPE_MISSING'])).toEqual({ text: 'No specific type', more: 1 });
    expect(shortNote(['TYPE_INFERRED', 'TYPE_MISSING', 'LOW_CONFIDENCE'])).toEqual({ text: 'Type supplied by Intern', more: 2 });
    expect(shortNote(['NOT_A_CODE'])).toBeUndefined();
    expect(shortNote([])).toBeUndefined();
    // The review's few words for the same code are as they were.
    expect(shortReason(['TYPE_MISSING'])).toEqual({ text: 'No document type', more: 0 });
    expect(shortReason(['LOW_CONFIDENCE', 'TYPE_MISSING'])).toEqual({ text: 'No document type', more: 1 });
  });

  it('leads with what became of the file on a filed document, and says it first', () => {
    expect(shortOutcome(['TYPE_INFERRED', 'LOW_CONFIDENCE', 'ALREADY_NAMED'])).toEqual({ text: 'Already had this name', more: 2 });
    expect(shortOutcome(['ALREADY_NAMED'])).toEqual({ text: 'Already had this name', more: 0 });
    expect(shortOutcome(['EXACT_COPY'])).toEqual({ text: 'Exact copy', more: 0 });
    // With nothing said of the file, exactly the review's few words.
    for (const codes of [['TYPE_INFERRED', 'LOW_CONFIDENCE'], ['LOW_CONFIDENCE', 'DATE_AMBIGUOUS'], ['TYPE_MISSING'], ['NOT_A_CODE'], []]) {
      expect(shortOutcome(codes), codes.join()).toEqual(shortReason(codes));
    }
    expect(outcomeFirst(['TYPE_INFERRED', 'LOW_CONFIDENCE', 'ALREADY_NAMED'])).toEqual(['ALREADY_NAMED', 'TYPE_INFERRED', 'LOW_CONFIDENCE']);
    expect(outcomeFirst(['TYPE_INFERRED', 'LOW_CONFIDENCE'])).toEqual(['TYPE_INFERRED', 'LOW_CONFIDENCE']);
    expect(describeNotes(outcomeFirst(['TYPE_INFERRED', 'ALREADY_NAMED']))).toBe(`${humanizeReason('ALREADY_NAMED')} ${describeNotes(['TYPE_INFERRED'])}`);
  });

  it('leaves every review sentence as it was', () => {
    expect(humanizeReason('TYPE_UNSUPPORTED, TYPE_INFERRED, LOW_CONFIDENCE')).toBe(
      'The document does not confirm the proposed type as its own. The type does not appear verbatim in the document, or appears only where the document says what it is about, such as a subject line or the text of a letter. Check the type. '
      + 'Intern supplied the type in this name: the document\'s own title, or the kind its layout shows, such as “Letter” or “Email”. Check that it names the document. '
      + 'The model was unsure of its reading. Check the name against the document, starting with the date and the parties.',
    );
  });
});
