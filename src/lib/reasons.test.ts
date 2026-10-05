import { describe, expect, it } from 'vitest';
import { describeQueueStop, humanizeReason } from './reasons';

describe('humanizeReason', () => {
  it('translates a single code into its sentence', () => {
    expect(humanizeReason('SOURCE_LOCKED')).toContain('sync client mid-upload');
  });

  it('translates the pipeline comma-joined list into sentences', () => {
    const result = humanizeReason('TYPE_UNSUPPORTED, LOW_CONFIDENCE, MODEL_REQUESTED_REVIEW');
    expect(result).toBe(
      'The proposed document type does not appear verbatim in the document. '
      + 'The model reported low confidence in its own proposal. '
      + 'The model asked for a person to look at this one.',
    );
  });

  it('tolerates lowercase codes from other serializations', () => {
    expect(humanizeReason('date_unsupported')).toContain('verbatim');
  });

  it('tells a near-duplicate apart from an exact one', () => {
    expect(humanizeReason('NEAR_DUPLICATE')).toBe('This looks like a document that was filed already. Approve to file it as well, keep the original, or remove it.');
    expect(humanizeReason('LOW_CONFIDENCE, NEAR_DUPLICATE')).toContain('filed already');
  });

  it('explains a bare duplicate flag once the filed name it referred to is gone', () => {
    expect(humanizeReason('DUPLICATE')).toBe('This document\'s content was filed once already. Retry to process it anyway, or remove it.');
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
      'DATE_IMPLAUSIBLE', 'DATE_IS_DEADLINE', 'DATE_AMBIGUOUS',
      'PASSWORD_PROTECTED', 'UNSUPPORTED_CONTENT', 'DOCUMENT_TOO_LARGE', 'OCR_UNAVAILABLE',
      'EXTRACTION_FAILED', 'ANALYSIS_FAILED', 'MODEL_FAILED', 'MODEL_OUTPUT_INVALID',
      'MODEL_INPUT_TOO_LARGE', 'MODEL_REPLY_TRUNCATED', 'HOSTED_MODEL_UNAVAILABLE',
      'HOSTED_MODEL_BILLING', 'STATE_CONFLICT', 'INVALID_TRANSITION', 'ALREADY_NAMED',
      'INTAKE_WITHDRAWN',
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
    for (const code of ['MODEL_FAILED', 'MODEL_OUTPUT_INVALID', 'HOSTED_MODEL_RATE_LIMITED', 'HOSTED_MODEL_UNREACHABLE']) {
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
