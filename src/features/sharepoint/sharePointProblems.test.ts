import { describe, expect, it } from 'vitest';
import { describeSharePointProblem } from './sharePointProblems';

/** Every code the backend's SharePoint setup, verifier, and onboarding state can produce. */
const BACKEND_CODES = [
  'ACTIVATION_ROLLBACK_FAILED', 'AUTOSTART_FAILED', 'FILED_UNWRITABLE', 'FIXED_FOLDERS_AMBIGUOUS', 'INBOX_MISSING',
  'MICROSOFT_ACCOUNT_INVALID', 'MICROSOFT_ACCOUNT_MISSING', 'MICROSOFT_ACCOUNT_WRONG_TENANT', 'MICROSOFT_BINDING_FAILED',
  'MICROSOFT_BINDING_STATUS_FAILED', 'MICROSOFT_CONNECTION_UNAVAILABLE', 'ONEDRIVE_ACCOUNT_MISSING', 'ONEDRIVE_MISSING',
  'ONEDRIVE_OPEN_FAILED', 'SETTINGS_WRITE_FAILED', 'SHAREPOINT_DEPLOYMENT_UNAVAILABLE', 'SHAREPOINT_ROOT_AMBIGUOUS',
  'SHAREPOINT_ROOT_NESTED', 'SHAREPOINT_ROOT_UNWRITABLE',
  'SHAREPOINT_SETUP_TASK_FAILED', 'SHAREPOINT_SYNC_PENDING', 'SYNC_OPENER_UNAVAILABLE', 'SYNC_PROTOCOL_UNAVAILABLE',
  'SHAREPOINT_MANAGED_SETTINGS_UNAVAILABLE', 'SHAREPOINT_ROOT_RECORD_MALFORMED', 'SHAREPOINT_ROOT_RECORD_CONFLICT',
  'SHAREPOINT_ROOT_RECORD_UNAVAILABLE', 'ONBOARDING_STATE_WRITE_FAILED', 'ONBOARDING_STATE_UNREADABLE',
  'ONEDRIVE_ACCOUNT_MISMATCH', 'MICROSOFT_MANUAL_PAIRING_DISABLED', 'ONBOARDING_SETUP_INCOMPLETE', 'SHAREPOINT_ACTIVATION_IN_PROGRESS',
];

describe('describeSharePointProblem', () => {
  const generic = describeSharePointProblem({ code: 'SOMETHING_NEW', message: 'raw backend detail' });

  it('gives every backend code its own action-first sentence, never the fallback', () => {
    for (const code of BACKEND_CODES) {
      const problem = describeSharePointProblem({ code, message: 'backend words' });
      expect(problem.code, code).toBe(code);
      expect(problem.action, code).not.toBe(generic.action);
      expect(problem.action, code).not.toContain(code);
      expect(problem.detail, code).toBe('backend words');
    }
  });

  it('falls back to plain language for an unknown code and keeps the code and detail for support', () => {
    expect(generic.code).toBe('SOMETHING_NEW');
    expect(generic.action).toMatch(/^Try again/);
    expect(generic.action).not.toContain('SOMETHING_NEW');
    expect(generic.action).not.toContain('raw backend detail');
    expect(generic.detail).toBe('raw backend detail');
  });

  it('treats a bare code string as a code rather than as a message', () => {
    expect(describeSharePointProblem('ONEDRIVE_MISSING')).toMatchObject({ code: 'ONEDRIVE_MISSING', getOneDrive: true });
  });

  it('offers a different account only for account problems, and the OneDrive download only when it is missing', () => {
    expect(describeSharePointProblem({ code: 'MICROSOFT_ACCOUNT_WRONG_TENANT' })).toMatchObject({ switchAccount: true });
    expect(describeSharePointProblem({ code: 'MICROSOFT_ACCOUNT_WRONG_TENANT' }).action).toMatch(/work account/);
    expect(describeSharePointProblem({ code: 'ONEDRIVE_MISSING' }).action).toMatch(/^Install or open OneDrive/);
    expect(describeSharePointProblem({ code: 'SHAREPOINT_SYNC_PENDING' }).switchAccount).toBeUndefined();
    expect(describeSharePointProblem({ code: 'SHAREPOINT_SYNC_PENDING' }).getOneDrive).toBeUndefined();
  });

  it('asks for OneDrive and Intern to use the same work account, offering a different one only as a secondary choice', () => {
    const problem = describeSharePointProblem({ code: 'ONEDRIVE_ACCOUNT_MISMATCH' });
    expect(problem.action).toMatch(/^Sign in to OneDrive with the same work account/);
    expect(problem.switchAccount).toBeUndefined();
    expect(problem.otherAccount).toBe(true);
    expect(problem.getOneDrive).toBeUndefined();
    expect(problem.offerSync).toBeUndefined();
  });

  it('offers the sync request only for problems another sync request can help with', () => {
    const helps = ['SHAREPOINT_SYNC_PENDING', 'SHAREPOINT_ROOT_RECORD_UNAVAILABLE', 'SHAREPOINT_ROOT_RECORD_MALFORMED', 'SHAREPOINT_ROOT_RECORD_CONFLICT', 'ONEDRIVE_OPEN_FAILED', 'SYNC_OPENER_UNAVAILABLE'];
    for (const code of BACKEND_CODES) {
      expect(Boolean(describeSharePointProblem({ code }).offerSync), code).toBe(helps.includes(code));
    }
    expect(describeSharePointProblem({ code: 'SOMETHING_NEW' }).offerSync).toBeUndefined();
  });

  it('no longer carries copy for verifier codes the backend stopped producing', () => {
    for (const code of ['SHAREPOINT_ROOT_UNVERIFIED', 'SHAREPOINT_ROOT_VERIFIER_UNAVAILABLE']) {
      const problem = describeSharePointProblem({ code, message: 'old backend' });
      expect(problem.action, code).toBe(generic.action);
      expect(problem.code, code).toBe(code);
    }
  });

  it('explains the refusals that protect managed setup in terms of what to do next', () => {
    expect(describeSharePointProblem({ code: 'SHAREPOINT_ACTIVATION_IN_PROGRESS' }).action).toMatch(/^Try again in a moment/);
    expect(describeSharePointProblem({ code: 'ONBOARDING_SETUP_INCOMPLETE' }).action).toMatch(/^Turn on filing/);
    expect(describeSharePointProblem({ code: 'MICROSOFT_MANUAL_PAIRING_DISABLED' }).action).toMatch(/SharePoint connection/);
  });

  it('recognizes the backend consent-blocked token in a sign-in message', () => {
    const message = 'Your Microsoft organization has blocked Intern from connecting. Ask your IT administrator to allow it. (MICROSOFT_CONSENT_BLOCKED)';
    for (const error of [message, new Error(message), { message }]) {
      const problem = describeSharePointProblem(error);
      expect(problem.code).toBe('MICROSOFT_CONSENT_BLOCKED');
      expect(problem.action).toMatch(/organization blocked/i);
      expect(problem.action).toMatch(/cannot safely watch the shared Inbox/i);
      expect(problem.detail).toBe(message);
    }
  });

  it('lets the consent token win over a less specific code on the same failure', () => {
    const problem = describeSharePointProblem({ code: 'MICROSOFT_ACCOUNT_INVALID', message: 'Your Microsoft organization has blocked Intern from connecting. (MICROSOFT_CONSENT_BLOCKED)' });
    expect(problem.code).toBe('MICROSOFT_CONSENT_BLOCKED');
  });

  it('keeps the older sign-in phrases as a fallback classification', () => {
    expect(describeSharePointProblem('Microsoft sign-in was declined, expired, or blocked by organization policy. Files remain held.').code).toBe('MICROSOFT_SIGN_IN_DECLINED');
    expect(describeSharePointProblem('Microsoft sign-in expired. Start again; files remain held.').code).toBe('MICROSOFT_SIGN_IN_EXPIRED');
    expect(describeSharePointProblem(new Error('SharePoint deployment configuration is unavailable: missing')).code).toBe('SHAREPOINT_DEPLOYMENT_UNAVAILABLE');
    expect(describeSharePointProblem(undefined)).toMatchObject({ code: 'UNKNOWN', detail: undefined });
  });
});
