import { useState } from 'react';
import { SUPPORT_LINKS } from '../../lib/bridge';
import type { DesktopBridge, SupportLinkTarget } from '../../lib/bridge';

const SUPPORT_LINK_NAMES: Record<SupportLinkTarget, string> = { 'sharepoint-site': 'SharePoint', 'onedrive-download': 'The OneDrive download page' };

/**
 * Opens a fixed support link through the bridge; a link element would do
 * nothing inside the desktop app. If the shell refuses, the address is shown
 * instead, beside any setup problem rather than replacing it.
 */
export function useSupportLink(bridge: DesktopBridge) {
  const [error, setError] = useState('');
  const open = (target: SupportLinkTarget) => {
    setError('');
    void bridge.openSupportLink(target).catch(() => {
      setError(`${SUPPORT_LINK_NAMES[target]} could not be opened. You can reach it at ${SUPPORT_LINKS[target]}.`);
    });
  };
  return { error, open };
}
