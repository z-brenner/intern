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

!macro NSIS_HOOK_PREINSTALL
  ; Releases up to alpha.14 also carried a llama.cpp build at the install
  ; root, beside Intern.exe: the PrismML fork's (alpha.12 to alpha.14), and
  ; upstream b10361 before it (alpha.11). Nothing has started it since
  ; alpha.13, and this build ships none there: its one llama.cpp build is in
  ; llama-upstream\. An update installs over the old files without
  ; uninstalling them first (the updater passes /UPDATE, and the template
  ; then skips its uninstall), so the old root build - 22 files, 43 MB -
  ; would stay, and, missing from this build's uninstall list, outlive an
  ; uninstall too. This runs before any file of this build is written.
  ;
  ; Each name is one that those releases' own runtime inventories
  ; (runtime-assets.json, published with each release) list at the install
  ; root as llama.cpp's, or as the OpenMP runtime alpha.11 shipped with it.
  ; No pattern, so nothing else at the root and nothing in llama-upstream\
  ; can match; and only a folder that already holds Intern is touched, so a
  ; first install into a folder of someone's own leaves what is there.
  ${If} ${FileExists} "$INSTDIR\${MAINBINARYNAME}.exe"
    Delete "$INSTDIR\llama-server.exe"
    Delete "$INSTDIR\llama-server-impl.dll"
    Delete "$INSTDIR\llama-common.dll"
    Delete "$INSTDIR\llama.dll"
    Delete "$INSTDIR\mtmd.dll"
    Delete "$INSTDIR\ggml.dll"
    Delete "$INSTDIR\ggml-base.dll"
    Delete "$INSTDIR\ggml-rpc.dll"
    Delete "$INSTDIR\libomp140.x86_64.dll"
    Delete "$INSTDIR\ggml-cpu-x64.dll"
    Delete "$INSTDIR\ggml-cpu-sse42.dll"
    Delete "$INSTDIR\ggml-cpu-ssse3.dll"
    Delete "$INSTDIR\ggml-cpu-sandybridge.dll"
    Delete "$INSTDIR\ggml-cpu-ivybridge.dll"
    Delete "$INSTDIR\ggml-cpu-piledriver.dll"
    Delete "$INSTDIR\ggml-cpu-haswell.dll"
    Delete "$INSTDIR\ggml-cpu-alderlake.dll"
    Delete "$INSTDIR\ggml-cpu-skylakex.dll"
    Delete "$INSTDIR\ggml-cpu-cannonlake.dll"
    Delete "$INSTDIR\ggml-cpu-cascadelake.dll"
    Delete "$INSTDIR\ggml-cpu-cooperlake.dll"
    Delete "$INSTDIR\ggml-cpu-icelake.dll"
    Delete "$INSTDIR\ggml-cpu-zen4.dll"
    Delete "$INSTDIR\ggml-cpu-sapphirerapids.dll"
  ${EndIf}
!macroend

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
