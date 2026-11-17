/**
 * The pipeline reports review reasons and failures as stable machine codes -
 * "DATE_UNSUPPORTED", "SOURCE_LOCKED" - and until now the inspector showed
 * those codes to the person deciding what to do about them. Each sentence here
 * says what actually happened and, where one exists, what the person can do.
 * At most two short sentences: the review panel leads with a few words for
 * each code (`LABELS`), and these are its "why", one press away. Several of
 * them used to be paragraphs, and a document in review read like a manual.
 *
 * A code with no entry passes through unchanged: an unmapped code is a bug to
 * notice, not something to dress up as prose.
 */
const SENTENCES: Record<string, string> = {
  DATE_MISSING: 'No date that defines the document was found.',
  DATE_UNSUPPORTED: 'The proposed date does not appear verbatim in the document.',
  TYPE_MISSING: 'No specific document type was identified.',
  // Both are raised in more cases than their first (ReviewReason in
  // domain.rs; raised in facts.rs and validate.rs). TYPE_UNSUPPORTED: a
  // type the document does not state; and a type it does state, but only
  // where it says what it is about - a "Re:" or "Subject:" line a record
  // heads ("Subject: Annual Audit Report", the line itself shown as the
  // evidence), or a letter's text below its greeting. TYPE_INFERRED: a
  // title of the document in place of a type the model did not give or the
  // document did not support; and the kind the layout shows - "Letter" for
  // a letter typed by its matter, "Email" for a header block, "Notice" for
  // a heading that opens on it - whether or not the model gave a type.
  // Neither sentence says the type is absent or that the model gave none.
  TYPE_UNSUPPORTED: 'The document does not state this type as its own, or names it only in a subject line or the body of a letter. Check the type.',
  TYPE_INFERRED: 'Intern took the type from the document\'s title or from its layout, such as “Letter” or “Email”. Check that it names the document.',
  // Both are raised in more cases than a party missing from the text
  // (ReviewReason in domain.rs; raised in facts.rs and validate.rs, on the
  // relation compose.rs reads). PARTY_UNSUPPORTED: most often a name that
  // rests on sides only the model's reading gave, the document's wording
  // saying nothing of them (Relation::on_proposed_roles; counted before the
  // relation rules of November 2026, 99 of the 160 in the 9B's replays of
  // InternBench and the real sets, 70 of the 107 in the 4B's of InternBench
  // and real sets 1-4) - since those rules also a notice or a record
  // "from" the party it names by default, where the model called that
  // party its issuer or sender; a party no rule could join to the
  // document, named after a dash with no "from", "to" or "for" ("Purchase
  // Order - Kestrel Instruments Ltd"); then a party the model named that
  // the document does not contain, a placeholder, or one capitalized word
  // such as a first name, each left out of the name, which may then have
  // no party at all; a default the model's own reading speaks against - a
  // provider's document "for" the party the model called its issuer or
  // sender, or the first of several parties the model told no further
  // apart, kept only for coming first (Relation::doubtful); a release named
  // between two of its parties, by their order alone or as a published
  // release whose text sets out two sides; a document addressed to a
  // group of recipients such as "all staff" - named to or for the group,
  // in its parties or in its type, or, where the cast holds the party it
  // comes from, from that party in the group's place, so the group is
  // then nowhere in the name; a name whose party is longer than 70
  // characters (MAX_PARTY_NAME_CHARS in facts.rs), most often a run of
  // text read as one name, which the filename drops when it does not
  // fit; and a letter or a notice named to an organization read from one
  // coined word under the addressee ("PacifiCorp"; is_coined_name in
  // facts.rs), a shape that words the extraction ran together share.
  // PARTY_MISSING: an agreement or an amendment naming one side, an
  // invoice, a receipt or an order left naming no one, or any document left
  // naming no one because every party the model gave was set aside (copied,
  // signing for another, staff of an organization no single line names).
  // Neither sentence assumes the name has a party; each opens on what holds
  // in every case. PARTY_MISSING comes with PARTY_UNSUPPORTED (all three in
  // each model's replays), though PARTY_UNSUPPORTED mostly comes without
  // it, and the place to look, PARTY_CHECK below, follows the last of them
  // only. The sentence names the usual cause and no other: the rest are
  // the cases listed above, and a reviewer checks the parties the same way
  // for each of them.
  PARTY_UNSUPPORTED: 'The document does not confirm which party this name should give, most often because it never says which side a party is on.',
  PARTY_MISSING: 'The name lacks a party it needs, such as the other side of an agreement or who issued the document.',
  DESCRIPTION_UNSUPPORTED: 'The description asserts something the document does not contain.',
  DESCRIPTION_INVALID: 'The description was not a single usable sentence.',
  // Raised two ways: the model rated its own answer low, or - nearly always,
  // for a local model - it rated it high and Intern measured doubt in the
  // words of the date or a party's name as the model wrote them. This used to
  // read "The model reported low confidence in its own proposal", beside a
  // readout of the model's own figure at 100%. The sentence holds for both
  // and says where to look; the panel says what the figure beside it is.
  LOW_CONFIDENCE: 'The model was unsure of its reading. Check the name against the document, starting with the date and the parties.',
  MODEL_REQUESTED_REVIEW: 'The model asked for a person to look at this one.',
  PARSER_WARNING: 'Extraction reported a problem that could corrupt what was read.',
  DATE_IMPLAUSIBLE: 'The date\'s year looks wrong for a document, often a misread such as 2625 for 2025. Check it against the document.',
  DATE_IS_DEADLINE: 'The date the model chose is a due, renewal, or expiry date, not the date the document was issued. Check the date.',
  DATE_AMBIGUOUS: 'The date is written only as numbers that could be read day-first or month-first. Check which one the document means.',
  // An invoice, a statement or another issued document whose chosen date
  // stands, on every line that states it, joined to another date by "to",
  // "through", "until" or a dash: one end of a period it bills or covers,
  // not the day it was issued (ReviewReason::DateIsPeriod). The name keeps
  // the date.
  DATE_IS_PERIOD: 'The date is written only as the start or end of a period, such as a billing or service period. Check which date the document itself carries.',
  // A workbook whose chosen date stands only in cells whose formula calls
  // TODAY or NOW: an invoice template's "DATE" over =TODAY(), a quote's
  // =TODAY()+30. The cell shows what the formula worked out when the file
  // was last saved, and the text Intern read is that value, so every other
  // check passes (ReviewReason::DateIsFormula). The name keeps the date.
  DATE_IS_FORMULA: 'The date comes from a spreadsheet formula that uses today\'s date, so it can change each time the file is opened. Check which date the document should carry.',
  // The quick model read it. Its readings are never filed on their own; the
  // accurate model reads the document again when it can, unless a person has
  // acted on it first.
  FAST_MODEL_READING: 'Read by the quick model — check it, or wait for the accurate model to read it again.',
  // Approving such a document sends it back here rather than filing a name
  // that described the earlier version; reading it again is the way on, and
  // the refusal of the approval says the same (actionErrors.ts).
  FILE_CHANGED: 'The file changed after it was analyzed, so the result no longer describes it. Use Analyze again to read it as it is now.',
  // A rename refused because the file is held - a viewer the person opened
  // it in, a sync client still writing it - is rolled back and the document
  // waits in review; approving again once it is let go is the retry, and the
  // refusal of the approval itself says the same (actionErrors.ts).
  SOURCE_LOCKED: 'The document is open in another program, or a sync client is still writing it. Close it, then try again.',
  DESTINATION_UNAVAILABLE: 'The destination folder is unavailable, or already has a file with this name.',
  MOVE_VERIFICATION_FAILED: 'The rename could not be verified as intact, so it was not finalized.',
  SOURCE_DELETE_FAILED: 'The renamed copy is safe, but the original file could not be removed.',
  RECONCILIATION_REQUIRED: 'Recovery could not tell which file is the document, and both are still on disk. Compare the two names before deciding.',
  PROPOSAL_MISSING: 'Analysis finished without a usable proposal.',
  IO_ERROR: 'A file operation failed.',
  // The next eight leave a document in review, its file renamed
  // "NEEDS REVIEW - ..." where it lies (UNREADABLE_CODES in the backend):
  // Retry reads it again, a name typed for it can be approved, Keep original
  // gives the file its own name back, and Remove leaves it as it is.
  PASSWORD_PROTECTED: 'This file is password-protected, so Intern could not read it. Remove the password and retry, or name it yourself.',
  UNSUPPORTED_CONTENT: 'This file is not what its extension says it is, so Intern could not read it. Name it yourself, or keep its name.',
  DOCUMENT_TOO_LARGE: 'This document is too large for Intern to read. Name it yourself, or split it and add the parts.',
  OCR_UNAVAILABLE: 'Intern\'s text-recognition files are missing. Reinstall Intern to restore them.',
  EXTRACTION_FAILED: 'Intern could not read this file. It may be damaged: open it to check, then retry or name it yourself.',
  ANALYSIS_FAILED: 'Intern hit an internal error while reading this document. Retry, or name it yourself.',
  MODEL_FAILED: 'The model could not finish reading this document. Retry it, or name it yourself.',
  MODEL_OUTPUT_INVALID: 'The model\'s answer for this document could not be used. Retry it, or name it yourself.',
  MODEL_INPUT_TOO_LARGE: 'This document is too long for the model to read at once. Name it yourself, or split it.',
  MODEL_REPLY_TRUNCATED: 'The model\'s answer was cut off before it finished. Retry it, or name it yourself.',
  HOSTED_MODEL_UNAVAILABLE: 'The hosted model could not be used for this document. Check the Model section in Settings.',
  HOSTED_MODEL_BILLING: 'The hosted service refused the request for billing or quota reasons. Check your account\'s credit, then resume the queue.',
  STATE_CONFLICT: 'Another file operation was in progress. Try again in a moment.',
  INVALID_TRANSITION: 'That action does not apply to this document in its current state.',
  DATE_REQUIRED: 'Every rename needs a date. Start the filename with the document\'s date as YYYY-MM-DD.',
  MODEL_DECLINED: 'The hosted model declined to answer about this document. Name it yourself, or keep the original.',
  HOSTED_MODEL_MISCONFIGURED: 'The hosted model\'s address, model name, or key is not usable. Check the Model section in Settings.',
  HOSTED_MODEL_KEY_MISSING: 'No API key is stored for the hosted model. Paste one in the Model section in Settings.',
  // Linux only: the hosted model's key lives in the Secret Service, and this
  // desktop is not running one.
  SECRET_SERVICE_UNAVAILABLE: 'The hosted model\'s key could not be read because no credential store answered on this desktop. Start and unlock GNOME Keyring or KWallet, or use the local model.',
  HOSTED_MODEL_UNAUTHORIZED: 'The hosted service rejected the API key. Check the key in Settings.',
  HOSTED_MODEL_UNREACHABLE: 'The hosted service could not be reached. Check the connection and try again.',
  HOSTED_MODEL_RATE_LIMITED: 'The hosted service asked for a slower pace. Intern will retry.',
  HOSTED_MODEL_REJECTED: 'The hosted service rejected the request — often an unknown model name. Check the Model section in Settings.',
  HOSTED_MODEL_REFUSED: 'The hosted model declined to answer about this document.',
  // Raised before analysis when the content was filed before; the name it was
  // filed under is normally given instead, as "Duplicate of ...". The bare code
  // only reaches here once the record of that filing is gone.
  // With an output folder, an exact duplicate is filed there as a copy
  // instead (EXACT_COPY); this is the one left where it is - no output
  // folder, a file already in it, a copy the person undid.
  DUPLICATE: 'This document\'s content was filed once already, so Intern did not file it again. Choose Process anyway to process it all the same, keep it where it is, or remove it.',
  // An exact duplicate moved into the output folder under the original's
  // filed name with COPY in front, without being read.
  EXACT_COPY: 'This file is an exact copy of a document Intern already filed, so Intern filed it as a copy, named after that document, without reading it.',
  // Raised after analysis when the text is nearly the text of a document
  // already filed - a second scan, a re-export, a copy saved again. The
  // inspector names that filing beside this sentence.
  NEAR_DUPLICATE: 'This looks like a document that was filed already. Approve to file it as well, keep the original, or remove it.',
  // Set on a name a spelling learned from review edits changed before a
  // person checked it (LearnedRule::confirmed in pipeline.rs): an edit's
  // wording cannot always tell a spelling from a decision about one
  // document, so a rule files nothing on its own until a person chooses
  // "Always use" for it, in Settings under Learned spellings: the review
  // window does not ask. Approving the name checks nothing, so the sentence
  // says both: approve a right name, and choose Always use in Settings to
  // stop being asked. Putting the document's own word back retracts the
  // rule when nothing else in the name changed. Not "for the first time": a
  // spelling an earlier release learned has been used before, and is
  // unchecked after the update all the same.
  LEARNED_SPELLING_UNCONFIRMED: 'Intern used a spelling it learned from your edits, and a name it changes waits for you until you choose Always use for it in Settings, under Learned spellings. Approve the name if it is right; if not, put the document\'s own words back.',
  // Set when a person undoes a rename. The file is back under its original
  // name and the proposal is still here, so the document waits for a decision
  // rather than being renamed again by the next automatic pass.
  UNDONE: 'You undid this rename. The document is back under its original name and waits for your decision.',
  // Set when the document already had exactly the name Intern would give it.
  ALREADY_NAMED: 'This document already had this name, so nothing was renamed.',
  // Set on a row the folder watcher canceled itself, as opposed to a person's
  // Cancel: another computer took the document over, or who uploaded it could
  // no longer be confirmed.
  INTAKE_WITHDRAWN: 'Another computer took this document over, or who added it could no longer be confirmed, so it was set aside here. It runs again if it comes back to this computer.',
};

/**
 * Where to look about the parties, after PARTY_UNSUPPORTED's sentence or
 * PARTY_MISSING's. PARTY_MISSING arrives with PARTY_UNSUPPORTED (all three
 * PARTY_MISSING readings in the 9B's replays did, in either order), and the
 * same instruction after each made one text repeat itself.
 */
const PARTY_CHECK = 'Check who the document is from, to, for or between.';

/**
 * The closing sentence a code shares with others. humanizeReason says it
 * once, after the last code in a list that ends on it; a code with none
 * reads as its sentence alone.
 */
const CLOSINGS: Record<string, string> = {
  PARTY_UNSUPPORTED: PARTY_CHECK,
  PARTY_MISSING: PARTY_CHECK,
};

/**
 * What the "queue stopped" banner says for a pause, where the sentence for the
 * same code on a single document would say the wrong thing. A document's
 * sentence tells a person to retry that document, or that Intern will retry
 * it; neither is true of a stopped queue, which waits until it is resumed.
 */
const QUEUE_STOP_SENTENCES: Record<string, string> = {
  // The local model server could not be reached, restarted, or recovered, or
  // no model has been loaded for minutes.
  MODEL_FAILED: 'The local model stopped responding. Resume the queue to try again, and restart Intern if it stops again.',
  // Several documents in a row came back with a reply that could not be used.
  MODEL_OUTPUT_INVALID: 'The model\'s answers for several documents in a row could not be used. Check the Model section in Settings, then resume the queue.',
  HOSTED_MODEL_RATE_LIMITED: 'The hosted service asked for a slower pace. Wait a minute, then resume the queue.',
  HOSTED_MODEL_UNREACHABLE: 'The hosted service could not be reached. Check the connection, then resume the queue.',
  // Intern's document reader crashed on several documents in a row, or on
  // several of one kind with none of that kind it could read; the documents
  // wait under their own names to be read again.
  READER_CRASHED: 'Intern\'s document reader crashed on several documents. They wait under their own names. Restart Intern, then resume the queue.',
  // The reader's self-test could not write its small file in Intern's own
  // data folder, which says nothing about the reader.
  READER_CHECK_UNWRITABLE: 'Intern could not write a small test file in its own data folder to check its document reader. Make sure that folder can be written to, then resume the queue.',
};

/**
 * The reason the queue stopped itself, in words that fit a stopped queue:
 * its own sentence where a document's would mislead, and otherwise the same
 * sentence a document gets.
 */
export function describeQueueStop(code: string): string {
  return QUEUE_STOP_SENTENCES[code.trim().toUpperCase()] ?? humanizeReason(code);
}

/**
 * Translates a reason string - a single code, or the pipeline's comma-joined
 * list of codes - into sentences. Anything that is not a known code is kept
 * verbatim, including free text like "Duplicate of X".
 */
export function humanizeReason(reason: string): string {
  const parts = reason.split(',').map((part) => part.trim()).filter((part) => part.length > 0);
  // Only a list in which every entry is a known code is a code list; free text
  // can legitimately contain commas ("Duplicate of ... Worldwide, Inc ...")
  // and must come through with them intact.
  if (parts.length === 0 || !parts.every((part) => SENTENCES[part.toUpperCase()] !== undefined)) {
    return reason;
  }
  return sentences(parts.map((part) => part.toUpperCase()), {});
}

/**
 * The sentences for a list of codes, each with its closing sentence unless
 * a later code says the same one. `instead` has the sentence to say for a
 * code in place of its own, which then stands alone, with no closing. A
 * code with no sentence is said as it came.
 */
function sentences(codes: readonly string[], instead: Record<string, string>): string {
  return eachSaid(codes, instead).join(' ');
}

/** The sentences of `sentences`, one a code, each with its closing unless a later code says the same one. */
function eachSaid(codes: readonly string[], instead: Record<string, string>): string[] {
  const closingOf = (code: string) => instead[code] === undefined ? CLOSINGS[code] : undefined;
  return codes.map((code, index) => {
    const closing = closingOf(code);
    const saidLater = closing !== undefined && codes.slice(index + 1).some((later) => closingOf(later) === closing);
    const sentence = instead[code] ?? SENTENCES[code] ?? code;
    return closing === undefined || saidLater ? sentence : `${sentence} ${closing}`;
  });
}

/**
 * The same codes on a document that is not waiting. At the Balanced review
 * level a reading can be ready while it carries some of them, and a document
 * Intern then renames by itself is filed with them still on its record. There
 * they are notes: nothing holds the document, and it can be renamed with
 * nobody looking at it. A sentence above that ends on an instruction -
 * "Check the type." - reads on such a document as if it were still waiting
 * for that check, so each of those has its note here: what Intern found,
 * and for the model's doubt where to look if the person does look.
 *
 * Only the codes the level lets a ready document carry need one (the level's
 * rule is the engine's). A code with no entry keeps its sentence above, as
 * TYPE_MISSING and DESCRIPTION_UNSUPPORTED do: each says what was found and
 * asks for nothing. On such a document TYPE_MISSING never means the name
 * has no type: the level makes a reading ready only when its name has one,
 * and the engine raises the code for a type that is a general word alone
 * ("Letter", "Agreement", "Notice") as well. Its sentence, "No specific
 * document type was identified", holds of that. Its few words, "No
 * document type", do not, so a note has its own (SHORT_NOTES).
 */
const NOTE_SENTENCES: Record<string, string> = {
  TYPE_UNSUPPORTED: 'The document does not state the type in this name as its own, or names it only in a subject line or the body of a letter.',
  TYPE_INFERRED: 'Intern took the type from the document\'s title or from its layout, such as “Letter” or “Email”.',
  LOW_CONFIDENCE: 'The model was unsure of its reading. If you look at this name, start with the date and the parties.',
};

/**
 * Codes that say what became of a document's file, not anything about how
 * it was read: an exact copy filed as one, a name the file already had. On
 * a ready or a filed document they are not notes about its reading.
 */
const OUTCOME_CODES = new Set(['EXACT_COPY', 'ALREADY_NAMED']);

/** The codes of a list that are notes about a reading: all but those that say what became of the file. */
export function noteCodes(codes: readonly string[]): string[] {
  return codes.filter((code) => !OUTCOME_CODES.has(code));
}

/**
 * A list of codes with what became of the file first. On a filed document
 * that is the news - nothing was renamed - and what Intern had noted about
 * its reading reads after it.
 */
export function outcomeFirst(codes: readonly string[]): string[] {
  return [...codes.filter((code) => OUTCOME_CODES.has(code)), ...noteCodes(codes)];
}

/**
 * What Intern noted, for a list of codes on a document that is not waiting:
 * each code's note, or its sentence where that already reads as one.
 */
export function describeNotes(codes: readonly string[]): string {
  return sentences(codes.map((code) => code.trim().toUpperCase()), NOTE_SENTENCES);
}

/**
 * The codes in a reason string, when it is a list of codes; empty for free
 * text ("Duplicate of ..."), which names no code even when it has commas.
 */
export function reasonCodes(reason: string): string[] {
  const parts = reason.split(',').map((part) => part.trim().toUpperCase()).filter((part) => part.length > 0);
  return parts.length > 0 && parts.every((part) => SENTENCES[part] !== undefined) ? parts : [];
}

/**
 * Each code in a few words: the review panel's chips, the queue's Check
 * column, the history's notes - what a person scanning forty rows, or one
 * panel, needs to see to know what to look at. Two to five words, no
 * sentence: the sentence above is one press or one hover away (the chip's
 * "why", the cell's tooltip).
 *
 * A code with a sentence and no entry here - one added since - is said in
 * its own words (`codeWords`), so it is never shown as a bare code; a code
 * with no sentence either is shown as it came (`reasonLabel`).
 */
const LABELS: Record<string, string> = {
  DATE_MISSING: 'No date found',
  DATE_UNSUPPORTED: 'Date not in document',
  DATE_AMBIGUOUS: 'Date order unclear',
  DATE_IS_DEADLINE: 'Date is a deadline',
  DATE_IS_PERIOD: 'Date is a period end',
  DATE_IS_FORMULA: 'Date from a formula',
  DATE_IMPLAUSIBLE: 'Year looks wrong',
  DATE_REQUIRED: 'Needs a date',
  TYPE_MISSING: 'No document type',
  TYPE_UNSUPPORTED: 'Type not confirmed',
  TYPE_INFERRED: 'Type guessed',
  PARTY_UNSUPPORTED: 'Party unclear',
  PARTY_MISSING: 'Party missing',
  DESCRIPTION_UNSUPPORTED: 'Description unsupported',
  DESCRIPTION_INVALID: 'Description unusable',
  LOW_CONFIDENCE: 'Model unsure',
  MODEL_REQUESTED_REVIEW: 'Model asked for a check',
  PARSER_WARNING: 'Text may be misread',
  FAST_MODEL_READING: 'Quick model\'s reading',
  FILE_CHANGED: 'File changed since read',
  SOURCE_LOCKED: 'File in use',
  DESTINATION_UNAVAILABLE: 'Destination unavailable',
  MOVE_VERIFICATION_FAILED: 'Rename not verified',
  SOURCE_DELETE_FAILED: 'Original not removed',
  RECONCILIATION_REQUIRED: 'Compare two files',
  PROPOSAL_MISSING: 'No proposal',
  IO_ERROR: 'File error',
  PASSWORD_PROTECTED: 'Password-protected',
  UNSUPPORTED_CONTENT: 'Not the format it claims',
  DOCUMENT_TOO_LARGE: 'Too large to read',
  OCR_UNAVAILABLE: 'Text recognition missing',
  EXTRACTION_FAILED: 'Could not be read',
  ANALYSIS_FAILED: 'Internal error',
  MODEL_FAILED: 'Model failed',
  MODEL_OUTPUT_INVALID: 'Model answer unusable',
  MODEL_INPUT_TOO_LARGE: 'Too long for the model',
  MODEL_REPLY_TRUNCATED: 'Model answer cut off',
  MODEL_DECLINED: 'Model declined',
  HOSTED_MODEL_UNAVAILABLE: 'Hosted model unavailable',
  HOSTED_MODEL_BILLING: 'Hosted billing problem',
  HOSTED_MODEL_MISCONFIGURED: 'Hosted model misconfigured',
  HOSTED_MODEL_KEY_MISSING: 'No API key',
  HOSTED_MODEL_UNAUTHORIZED: 'API key rejected',
  HOSTED_MODEL_UNREACHABLE: 'Hosted service unreachable',
  HOSTED_MODEL_RATE_LIMITED: 'Rate limited',
  HOSTED_MODEL_REJECTED: 'Request rejected',
  HOSTED_MODEL_REFUSED: 'Model declined',
  SECRET_SERVICE_UNAVAILABLE: 'Key store unavailable',
  STATE_CONFLICT: 'Busy, try again',
  INVALID_TRANSITION: 'Not possible now',
  DUPLICATE: 'Already filed',
  NEAR_DUPLICATE: 'Possible duplicate',
  EXACT_COPY: 'Exact copy',
  LEARNED_SPELLING_UNCONFIRMED: 'Uses a learned spelling',
  UNDONE: 'Rename undone',
  ALREADY_NAMED: 'Already had this name',
  INTAKE_WITHDRAWN: 'Taken over elsewhere',
};

/**
 * A code's own words, for a code with a sentence and no label yet:
 * DATE_FROM_FILENAME reads "Date from filename".
 */
function codeWords(code: string): string {
  const words = code.toLowerCase().replace(/_+/g, ' ').trim();
  return words.charAt(0).toUpperCase() + words.slice(1);
}

/** Whether a code is one this build has words for. */
const isKnown = (code: string) => SENTENCES[code] !== undefined || LABELS[code] !== undefined;

/**
 * The few words for one code: its label, its own words when it has a
 * sentence but no label yet, and otherwise the code as it came - an
 * unmapped code is a bug to notice.
 */
export function reasonLabel(code: string): string {
  const key = code.trim().toUpperCase();
  return LABELS[key] ?? (SENTENCES[key] !== undefined ? codeWords(key) : code.trim());
}

/** The few words for a code on a document that is not waiting, where a review's would be false of it (SHORT_NOTES). */
export function noteLabel(code: string): string {
  return SHORT_NOTES[code.trim().toUpperCase()] ?? reasonLabel(code);
}

/**
 * Why, for one code: its sentence, with the closing it shares with others
 * (where to look about the parties) after it, as the code stands alone
 * beside its few words. Empty for a code with no sentence.
 */
export function reasonWhy(code: string): string {
  const key = code.trim().toUpperCase();
  return SENTENCES[key] === undefined ? '' : sentences([key], {});
}

/** The same for a code on a document that is not waiting: its note, where it has one. */
export function noteWhy(code: string): string {
  const key = code.trim().toUpperCase();
  return SENTENCES[key] === undefined && NOTE_SENTENCES[key] === undefined ? '' : sentences([key], NOTE_SENTENCES);
}

/**
 * A list of codes as a tooltip says it: each code's few words and its why,
 * a line each - for a cell or a status that shows one reason in words and
 * counts the rest, so the rest are named too. `note`: on a document that is
 * not waiting, each note's words and sentence. Where to look about the
 * parties is said once, after the last code that shares it, as
 * humanizeReason says it; a code with no sentence is its words alone.
 */
export function reasonLines(codes: readonly string[], { note = false }: { note?: boolean } = {}): string {
  const keys = codes.map((code) => code.trim().toUpperCase());
  const whys = eachSaid(keys, note ? NOTE_SENTENCES : {});
  return keys.map((code, index) => {
    const label = note ? noteLabel(code) : reasonLabel(code);
    return isKnown(code) || NOTE_SENTENCES[code] !== undefined ? `${label}: ${whys[index]}` : label;
  }).join('\n');
}

/** Which part of a proposal a code is about, for marking that part where it is shown. */
export type ReasonField = 'date' | 'type' | 'parties' | 'description';

export function reasonField(code: string): ReasonField | undefined {
  if (code.startsWith('DATE_')) return 'date';
  if (code.startsWith('TYPE_')) return 'type';
  if (code.startsWith('PARTY_')) return 'parties';
  if (code.startsWith('DESCRIPTION_')) return 'description';
  return undefined;
}

/**
 * The parts of a proposal the codes put in doubt, in the order a name is
 * read: date, type, parties, description.
 */
export function flaggedFields(codes: readonly string[]): ReasonField[] {
  const order: ReasonField[] = ['date', 'type', 'parties', 'description'];
  const fields = new Set(codes.map(reasonField).filter((field): field is ReasonField => field !== undefined));
  return order.filter((field) => fields.has(field));
}

/**
 * The few words for a note, where a review's would be false of a document
 * that is not waiting. TYPE_MISSING is raised for a name with no type and
 * for one whose type is a general word alone ("Letter", "Agreement"). Only
 * the second can be ready (the Balanced level holds a name with no type),
 * so on a ready or a renamed document "No document type" stood beside a
 * name that had one. These words hold in both cases.
 */
const SHORT_NOTES: Record<string, string> = {
  TYPE_MISSING: 'No specific type',
};

/** A code about one part of the name says more than the model's general doubt. */
const aboutAPart = (code: string) => Number(reasonField(code) !== undefined);

/**
 * The few words, from `words`, for the code of a list that `rank` puts
 * first (the higher, the sooner; codes of one rank in the order they came),
 * with the rest counted rather than listed.
 */
function fewWords(codes: readonly string[], words: (code: string) => string, rank: (code: string) => number): { text: string; more: number } | undefined {
  const known = codes.filter(isKnown);
  if (!known.length) return undefined;
  const ranked = [...known].sort((left, right) => rank(right) - rank(left));
  return { text: words(ranked[0]), more: ranked.length - 1 };
}

/**
 * The few words for a list of codes, most specific first: a code about one
 * part of the name says more than the model's general doubt, so it leads,
 * and the rest are counted rather than listed.
 */
export function shortReason(codes: readonly string[]): { text: string; more: number } | undefined {
  return fewWords(codes, reasonLabel, aboutAPart);
}

/**
 * The same for the notes on a document that is not waiting: a review's few
 * words, except where those would be false of it (SHORT_NOTES).
 */
export function shortNote(codes: readonly string[]): { text: string; more: number } | undefined {
  return fewWords(codes, noteLabel, aboutAPart);
}

/**
 * The same for a filed document that was not renamed with a note: what
 * became of the file leads ("Already had this name"), and what was noted
 * about its reading is counted behind it. With nothing said of the file
 * these are a review's few words, as a document a person kept under its
 * own name has always shown them.
 */
export function shortOutcome(codes: readonly string[]): { text: string; more: number } | undefined {
  return fewWords(codes, reasonLabel, (code) => OUTCOME_CODES.has(code) ? 2 : aboutAPart(code));
}
