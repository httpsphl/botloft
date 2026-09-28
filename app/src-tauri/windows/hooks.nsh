; Installer hooks (spec 15.4). The daemon runs from its own copy in the
; data folder as a scheduled task, so installing or updating the app never
; touches it; the app updates the daemon when it next opens.

!macro NSIS_HOOK_PREUNINSTALL
  ; A real uninstall stops the daemon and removes its scheduled task and
  ; binary. Bots and data stay in %LOCALAPPDATA%\Botloft. An update runs
  ; this uninstaller too, with /UPDATE: then the daemon keeps running.
  ${If} $UpdateMode <> 1
    nsExec::Exec '"$INSTDIR\botloftd.exe" service uninstall'
    Pop $0
  ${EndIf}
!macroend
