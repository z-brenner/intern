/**
 * SharePoint setup failures, as sentences that start with what to do.
 *
 * The backend (`src-tauri/src/sharepoint_setup.rs`) rejects with a stable
 * `{ code, message }`. Its message is written for support, so the person sees
 * the sentence here and the code stays in Support details. A code this build
 * does not know still gets a usable sentence rather than the raw code.
 */
const MESSAGES: Record<string, string> = {
  ACTIVATION_ROLLBACK_FAILED: 'Restart Intern, then check again. A setup step failed and could not be fully undone.',
  AUTOSTART_FAILED: 'Allow Intern to start when you sign in, then check again. Windows refused the change.',
  FILED_UNWRITABLE: 'Ask your SharePoint administrator for permission to add files to Filed. Intern cannot write there.',
  FIXED_FOLDERS_AMBIGUOUS: 'Contact support. More than one Inbox or Filed folder was found in the Files library.',
  INBOX_MISSING: 'Ask your SharePoint administrator to restore the Inbox folder in the Files library.',
  MICROSOFT_ACCOUNT_INVALID: 'Reconnect Microsoft. The saved account could not be confirmed.',
  MICROSOFT_ACCOUNT_MISSING: 'Reconnect Microsoft with your work account. Documents stay untouched until you do.',
  MICROSOFT_ACCOUNT_WRONG_TENANT: 'Reconnect with your work account. The account you signed in with belongs to a different organization.',
  MICROSOFT_BINDING_FAILED: 'Check again in a moment. Intern could not confirm the Inbox folder with Microsoft.',
  MICROSOFT_BINDING_STATUS_FAILED: 'Check again in a moment. Intern could not read its Microsoft connection.',
  MICROSOFT_CONNECTION_UNAVAILABLE: 'Check your internet connection, then check again. Microsoft could not be reached.',
  ONEDRIVE_ACCOUNT_MISSING: 'Sign in to OneDrive with your work account, then check again.',
  ONEDRIVE_MISSING: 'Install or open OneDrive, then check again. Intern needs OneDrive to reach the Files library.',
  ONEDRIVE_OPEN_FAILED: 'Open OneDrive yourself, then check again. Intern could not start it.',
  SETTINGS_WRITE_FAILED: 'Check again. Intern could not save its settings on this computer.',
  SHAREPOINT_DEPLOYMENT_UNAVAILABLE: 'Install Intern from your organization\'s installer. This copy does not include the SharePoint connection.',
  SHAREPOINT_ROOT_AMBIGUOUS: 'Remove the extra synced copy of the Files library from OneDrive, then check again.',
  SHAREPOINT_ROOT_NESTED: 'Sync the Files library on its own, not inside another synced folder, then check again.',
  SHAREPOINT_ROOT_UNVERIFIED: 'Check again once OneDrive finishes syncing. Intern could not confirm the synced Files library.',
  SHAREPOINT_ROOT_UNWRITABLE: 'Ask your SharePoint administrator for edit access to the Files library. Intern cannot write to it.',
  SHAREPOINT_ROOT_VERIFIER_UNAVAILABLE: 'Check again in a moment. The synced Files library could not be checked.',
  SHAREPOINT_SETUP_TASK_FAILED: 'Check again. The connection check stopped before it finished.',
  SHAREPOINT_SYNC_PENDING: 'Keep OneDrive running, then check again. The Files library is still syncing to this computer.',
  SYNC_OPENER_UNAVAILABLE: 'Install or update OneDrive, then check again. Intern could not ask OneDrive to sync the library.',
  SYNC_PROTOCOL_UNAVAILABLE: 'Install or update OneDrive, then check again. Intern could not ask OneDrive to sync the library.',
};

const GENERIC = 'Check again in a moment. If this keeps happening, contact support with the details below.';

export function describeSharePointError(code: string | undefined): string {
  return (code && MESSAGES[code]) || GENERIC;
}

export interface SharePointFailure {
  /** The stable code, when the failure carried one. */
  code?: string;
  /** The backend's own words, kept for Support details. */
  detail: string;
  /** What the person is told. */
  message: string;
}

/** Normalizes a rejected setup call: `{ code, message }`, a bare code, or anything else. */
export function sharePointFailure(error: unknown): SharePointFailure {
  let code: string | undefined;
  let detail = '';
  if (typeof error === 'object' && error !== null) {
    if ('code' in error && typeof error.code === 'string') code = error.code;
    if ('message' in error && typeof error.message === 'string') detail = error.message;
  } else if (typeof error === 'string') {
    if (/^[A-Z][A-Z0-9_]+$/.test(error.trim())) code = error.trim();
    else detail = error;
  }
  return { code, detail, message: describeSharePointError(code) };
}
