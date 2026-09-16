/**
 * Plain-language, action-first explanations for guided SharePoint setup.
 *
 * Every message starts with what the person should do. The stable code stays
 * beside it for support; the backend's own message is support detail too,
 * never the headline, because it is written for engineers.
 */
export interface OnboardingProblem {
  code: string;
  /** What to do, then why. */
  action: string;
  /** The raw backend message, for support only. */
  detail?: string;
  /** Offer to disconnect and sign in with another Microsoft account. */
  switchAccount?: boolean;
  /** Offer the OneDrive download page. */
  getOneDrive?: boolean;
}

const ACTIONS: Record<string, Omit<OnboardingProblem, 'code' | 'detail'>> = {
  MICROSOFT_CONSENT_BLOCKED: { action: 'Ask your IT administrator to allow Intern to connect to Microsoft. Your organization blocked the connection, so Intern cannot safely watch the shared Inbox.' },
  MICROSOFT_SIGN_IN_DECLINED: { action: 'Connect again and finish signing in. Microsoft sign-in was declined, expired, or blocked by your organization. If your organization blocked it, ask your IT administrator to allow Intern; without this connection Intern cannot safely watch the shared Inbox.' },
  MICROSOFT_SIGN_IN_EXPIRED: { action: 'Connect again. Microsoft sign-in expired before it finished.' },
  MICROSOFT_SIGN_IN_FAILED: { action: 'Connect again. Microsoft sign-in did not finish, so Intern has not connected an account.' },
  MICROSOFT_ACCOUNT_WRONG_TENANT: { action: 'Sign in with your work account. The connected Microsoft account belongs to a different organization.', switchAccount: true },
  MICROSOFT_ACCOUNT_MISSING: { action: 'Connect your Microsoft account again. Intern could not find a connected account.', switchAccount: true },
  MICROSOFT_ACCOUNT_INVALID: { action: 'Connect your Microsoft account again. Intern could not confirm the connected account.', switchAccount: true },
  MICROSOFT_CONNECTION_UNAVAILABLE: { action: 'Check your internet connection, then try again. Intern could not reach Microsoft.' },
  MICROSOFT_BINDING_FAILED: { action: 'Check your internet connection, then try again. Intern could not confirm the Files library with Microsoft.' },
  MICROSOFT_BINDING_STATUS_FAILED: { action: 'Check your internet connection, then try again. Intern could not confirm the Files library with Microsoft.' },
  ONEDRIVE_MISSING: { action: 'Install or open OneDrive, then try again. Intern needs OneDrive to keep the Files library on this computer.', getOneDrive: true },
  ONEDRIVE_ACCOUNT_MISSING: { action: 'Open OneDrive and sign in with your work account, then try again.' },
  ONEDRIVE_OPEN_FAILED: { action: 'Open SharePoint and choose Sync on the Files library, then try again. Intern could not ask OneDrive to start syncing.' },
  SYNC_OPENER_UNAVAILABLE: { action: 'Open SharePoint and choose Sync on the Files library, then try again. This computer could not hand the sync request to OneDrive.' },
  SYNC_PROTOCOL_UNAVAILABLE: { action: 'Open SharePoint and choose Sync on the Files library, then try again. This computer could not hand the sync request to OneDrive.' },
  SHAREPOINT_SYNC_PENDING: { action: 'Keep OneDrive open and try again in a moment. OneDrive has not finished adding the Files library yet.' },
  SHAREPOINT_ROOT_UNVERIFIED: { action: 'Let OneDrive finish syncing, then try again. Intern could not confirm that the synced folder is the Contoso Files library.' },
  SHAREPOINT_ROOT_AMBIGUOUS: { action: 'Contact support. More than one synced folder looks like the Files library, and Intern will not guess which one is right.' },
  SHAREPOINT_ROOT_NESTED: { action: 'Contact support. The Files library is synced inside another synced folder, and Intern will not guess which one is right.' },
  SHAREPOINT_ROOT_UNWRITABLE: { action: 'Make sure OneDrive is running and you can edit files in the Files library, then try again.' },
  SHAREPOINT_ROOT_VERIFIER_UNAVAILABLE: { action: 'Contact support. This version of Intern cannot yet confirm the synced Files library on this computer, so it will not start filing.' },
  SHAREPOINT_DEPLOYMENT_UNAVAILABLE: { action: 'Contact support. This copy of Intern is missing its SharePoint setup, so it cannot connect to the shared Inbox.' },
  SHAREPOINT_SETUP_TASK_FAILED: { action: 'Try again. The setup check stopped before it finished.' },
  INBOX_MISSING: { action: 'Ask your SharePoint site owner to check that Files/Inbox exists, then try again. Intern could not find the Inbox folder in the synced library.' },
  FILED_UNWRITABLE: { action: 'Ask your SharePoint site owner to let you edit Files/Filed, then try again. Intern cannot save documents there.' },
  FIXED_FOLDERS_AMBIGUOUS: { action: 'Contact support. Intern found more than one possible Inbox or Filed folder and will not guess.' },
  SETTINGS_WRITE_FAILED: { action: 'Try again. Intern could not save its settings, so filing has not started.' },
  AUTOSTART_FAILED: { action: 'Try again. Intern could not set itself to start when you sign in to Windows, so setup is not finished.' },
  ACTIVATION_ROLLBACK_FAILED: { action: 'Restart Intern, then try again. Setup stopped partway and Intern could not fully undo it. Nothing will be filed until setup finishes.' },
  ONBOARDING_STATE_WRITE_FAILED: { action: 'Try again. Intern could not record that setup is finished.' },
  ONBOARDING_STATE_DURABILITY_UNCERTAIN: { action: 'Try again. Intern could not confirm that it recorded setup as finished.' },
  ONBOARDING_STATE_UNREADABLE: { action: 'Try again. Intern could not read whether setup is finished, so it has not opened yet. If this keeps happening, contact support.' },
  ONBOARDING_STATE_UNAVAILABLE: { action: 'Try again. Intern could not read whether setup is finished, so it has not opened yet. If this keeps happening, contact support.' },
  ONBOARDING_STATE_TOO_LARGE: { action: 'Contact support. Intern\'s setup record is damaged, so it has not opened yet.' },
};

const FALLBACK = 'Try again. If this keeps happening, contact support and mention the code below.';

/**
 * Microsoft device sign-in reports plain sentences rather than codes, so
 * those few are classified here to keep the same copy and support code.
 */
function classifyMessage(message: string) {
  if (/deployment configuration is unavailable/i.test(message)) return 'SHAREPOINT_DEPLOYMENT_UNAVAILABLE';
  if (/blocked by organization policy|declined/i.test(message)) return 'MICROSOFT_SIGN_IN_DECLINED';
  if (/sign-in expired/i.test(message)) return 'MICROSOFT_SIGN_IN_EXPIRED';
  if (/sign-in/i.test(message)) return 'MICROSOFT_SIGN_IN_FAILED';
  return 'UNKNOWN';
}

export function describeOnboardingProblem(error: unknown): OnboardingProblem {
  let code: string | undefined;
  let detail: string | undefined;
  if (typeof error === 'string') detail = error;
  else if (error instanceof Error) detail = error.message;
  else if (typeof error === 'object' && error) {
    if ('code' in error && typeof error.code === 'string' && error.code.trim()) code = error.code.trim();
    if ('message' in error && typeof error.message === 'string') detail = error.message;
  }
  code ??= classifyMessage(detail ?? '');
  const known = ACTIONS[code];
  return { code, detail: detail?.trim() || undefined, ...(known ?? { action: FALLBACK }) };
}
