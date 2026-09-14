export interface MicrosoftAccount { tenantId: string; id: string; displayName: string; email: string }
export interface MicrosoftFolderBinding { localFolder: string; driveId: string; folderId: string; webUrl: string; tenantId: string }
export interface IntakeAttribution {
  path: string; filename: string; state: 'verified' | 'other' | 'unknown' | 'processed' | 'filed';
  reason: string; uploader: MicrosoftAccount | null; processedBy: MicrosoftAccount | null;
  filedAs: string | null; checkedAt: number;
}
export interface MicrosoftIntakeStatus {
  connected: boolean; account: MicrosoftAccount | null;
  binding: MicrosoftFolderBinding | null; documents: IntakeAttribution[]; error: string | null;
}
export interface MicrosoftDevicePrompt { userCode: string; verificationUri: string; intervalSeconds: number; expiresAt: number }
export type MicrosoftSignInProgress = { state: 'pending'; intervalSeconds: number } | { state: 'connected'; account: MicrosoftAccount };
export interface MicrosoftIntakeBridge {
  microsoftIntakeStatus(): Promise<MicrosoftIntakeStatus>;
  microsoftSignInStart(): Promise<MicrosoftDevicePrompt>;
  microsoftSignInPoll(): Promise<MicrosoftSignInProgress>;
  microsoftDisconnect(): Promise<void>;
  microsoftBindIntake(): Promise<MicrosoftFolderBinding>;
  microsoftOpenSignIn(): Promise<void>;
}
