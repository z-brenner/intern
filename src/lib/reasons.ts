/**
 * The pipeline reports review reasons and failures as stable machine codes -
 * "DATE_UNSUPPORTED", "SOURCE_LOCKED" - and until now the inspector showed
 * those codes to the person deciding what to do about them. Each sentence here
 * says what actually happened and, where one exists, what the person can do.
 *
 * A code with no entry passes through unchanged: an unmapped code is a bug to
 * notice, not something to dress up as prose.
 */
const SENTENCES: Record<string, string> = {
  DATE_MISSING: 'No date that defines the document was found.',
  DATE_UNSUPPORTED: 'The proposed date does not appear verbatim in the document.',
  TYPE_MISSING: 'No specific document type was identified.',
  TYPE_UNSUPPORTED: 'The proposed document type does not appear verbatim in the document.',
  TYPE_INFERRED: 'The model gave no document type, so the document\'s own title was used; check that it names the document.',
  PARTY_UNSUPPORTED: 'A proposed party could not be found in the document.',
  DESCRIPTION_UNSUPPORTED: 'The description asserts something the document does not contain.',
  DESCRIPTION_INVALID: 'The description was not a single usable sentence.',
  LOW_CONFIDENCE: 'The model reported low confidence in its own proposal.',
  MODEL_REQUESTED_REVIEW: 'The model asked for a person to look at this one.',
  PARSER_WARNING: 'Extraction reported a problem that could corrupt what was read.',
  DATE_IMPLAUSIBLE: 'The date\'s year looks wrong for a document, often a misread such as 2625 for 2025. Check it against the document.',
  DATE_IS_DEADLINE: 'The date the model chose is a due, renewal, or expiry date, not the date the document was issued. Check the date.',
  DATE_AMBIGUOUS: 'The date is written only as numbers that could be read day-first or month-first. Check which one the document means.',
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
  PASSWORD_PROTECTED: 'This file is password-protected. Remove the password and add it again.',
  UNSUPPORTED_CONTENT: 'This file is not what its extension says it is, so Intern could not read it.',
  DOCUMENT_TOO_LARGE: 'This document is too large for Intern to read. Split it, or name it yourself.',
  OCR_UNAVAILABLE: 'Intern\'s text-recognition files are missing. Reinstall Intern to restore them.',
  EXTRACTION_FAILED: 'Intern could not read this file. It may be damaged; open it to check, then retry or remove it.',
  ANALYSIS_FAILED: 'Intern hit an internal error while reading this document. Retry, or name it yourself.',
  MODEL_FAILED: 'The model could not finish reading this document. Retry it.',
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
  HOSTED_MODEL_UNAUTHORIZED: 'The hosted service rejected the API key. Check the key in Settings.',
  HOSTED_MODEL_UNREACHABLE: 'The hosted service could not be reached. Check the connection and try again.',
  HOSTED_MODEL_RATE_LIMITED: 'The hosted service asked for a slower pace. Intern will retry.',
  HOSTED_MODEL_REJECTED: 'The hosted service rejected the request — often an unknown model name. Check the Model section in Settings.',
  HOSTED_MODEL_REFUSED: 'The hosted model declined to answer about this document.',
  // Raised before analysis when the content was filed before; the name it was
  // filed under is normally given instead, as "Duplicate of ...". The bare code
  // only reaches here once the record of that filing is gone.
  DUPLICATE: 'This document\'s content was filed once already. Choose Process anyway to process it all the same, or remove it.',
  // Raised after analysis when the text is nearly the text of a document
  // already filed - a second scan, a re-export, a copy saved again. The
  // inspector names that filing beside this sentence.
  NEAR_DUPLICATE: 'This looks like a document that was filed already. Approve to file it as well, keep the original, or remove it.',
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
  return parts.map((part) => SENTENCES[part.toUpperCase()]).join(' ');
}
