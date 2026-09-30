; Installer hooks (spec 15.4). The daemon runs from its own copy in the
; data folder as a scheduled task, so installing or updating the app never
; touches it; the app updates the daemon when it next opens.

; The header image (installer-header.bmp) sits on the right of each page,
; next to the title, as the Botloft mascot on the page's white.
!define MUI_HEADERIMAGE_RIGHT

!macro NSIS_HOOK_POSTINSTALL
  ; Installed apps shows Botloft as the publisher. `bundle.publisher` stays
  ; unset on purpose: Tauri also names the key that keeps the install folder
  ; after it (Software\<publisher>\Botloft), so a new name would make the
  ; first update of an existing install run the old uninstaller without its
  ; folder.
  WriteRegStr SHCTX "${UNINSTKEY}" "Publisher" "Botloft"
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  ; A real uninstall stops the daemon and removes its scheduled task and
  ; binary. Bots and data stay in %LOCALAPPDATA%\Botloft. An update runs
  ; this uninstaller too, with /UPDATE: then the daemon keeps running.
  ${If} $UpdateMode <> 1
    nsExec::Exec '"$INSTDIR\botloftd.exe" service uninstall'
    Pop $0
    ; Nor does the app open at sign-in any more (spec 15.2).
    DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "Botloft"
  ${EndIf}
!macroend
