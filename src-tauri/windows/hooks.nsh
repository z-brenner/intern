; Intern's additions to the NSIS installer Tauri generates
; (tauri.conf.json > bundle > windows > nsis > installerHooks).
;
; "Send to > Intern" in Explorer's right-click menu is how people who live in
; Explorer and Outlook expect to hand documents over: select the attachments
; saved from an email, right-click, Send to. The shortcut starts Intern.exe
; with one path per selected file; a copy already running receives them
; through the single-instance plugin (commands.rs, launch_documents).
;
; The template's own uninstall already removes the autostart value from the
; HKCU Run key, so nothing here touches it.

!macro NSIS_HOOK_POSTINSTALL
  ; Written on every install, updates included: a person updating from a
  ; release without the shortcut gets it, at the cost of restoring one a
  ; person deleted by hand.
  CreateShortcut "$SENDTO\${PRODUCTNAME}.lnk" "$INSTDIR\${MAINBINARYNAME}.exe"
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  ; An uninstall that is part of an update - the template records /UPDATE in
  ; $UpdateMode, and keeps the Start menu shortcut the same way - leaves the
  ; shortcut where it is.
  ${If} $UpdateMode <> 1
    Delete "$SENDTO\${PRODUCTNAME}.lnk"
  ${EndIf}
!macroend
