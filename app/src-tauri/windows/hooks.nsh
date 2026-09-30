; Installer hooks (spec 15.4). The daemon runs from its own copy in the
; data folder as a scheduled task, so installing or updating the app never
; touches it; the app updates the daemon when it next opens.

; The header image (installer-header.bmp) sits on the right of each page,
; next to the title, as the Botloft mascot on the page's white.
!define MUI_HEADERIMAGE_RIGHT

; The installer's own file properties name the publisher too; Tauri's
; template leaves the company out.
VIAddVersionKey "CompanyName" "Botloft"

; Tauri keeps the install folder in the registry under
; Software\<publisher>\<product>. The publisher used to be "github", Tauri's
; default from the identifier; it is "Botloft" now (`bundle.publisher`).
; An install made under the old name is adopted under the new one, so an
; update goes to the folder Botloft is in, and "uninstall before installing"
; still finds that folder.
!ifndef BOTLOFT_FOLDER_KEY
  !define BOTLOFT_FOLDER_KEY "Software\Botloft\Botloft"
!endif
!ifndef BOTLOFT_OLD_FOLDER_KEY
  !define BOTLOFT_OLD_FOLDER_KEY "Software\github\Botloft"
!endif
!ifndef BOTLOFT_OLD_PUBLISHER_KEY
  !define BOTLOFT_OLD_PUBLISHER_KEY "Software\github"
!endif

; Copies the folder of an install made under the old publisher to the new
; key, and installs there unless /D chose another folder. It does nothing
; once the new key exists.
Function BotloftAdoptOldFolder
  Push $0
  ReadRegStr $0 SHCTX "${BOTLOFT_FOLDER_KEY}" ""
  StrCmp $0 "" 0 botloft_adopt_done
  ReadRegStr $0 SHCTX "${BOTLOFT_OLD_FOLDER_KEY}" ""
  StrCmp $0 "" botloft_adopt_done 0
  WriteRegStr SHCTX "${BOTLOFT_FOLDER_KEY}" "" $0
  ; Only when the installer is still on its default folder: /D wins.
  StrCmp $INSTDIR "$LOCALAPPDATA\Botloft" 0 botloft_adopt_done
  StrCpy $INSTDIR $0
  botloft_adopt_done:
  Pop $0
FunctionEnd

; Before the first page, so the pages and "uninstall before installing" see
; the adopted folder.
!define MUI_CUSTOMFUNCTION_GUIINIT BotloftAdoptOldFolder

!macro NSIS_HOOK_PREINSTALL
  ; The quiet installer (/S) has no pages, so nothing adopted the folder yet.
  ; The install section already set its output to the default folder.
  Call BotloftAdoptOldFolder
  SetOutPath $INSTDIR
!macroend

!macro NSIS_HOOK_POSTINSTALL
  ; The folder is under the new name now; the old key goes.
  DeleteRegKey SHCTX "${BOTLOFT_OLD_FOLDER_KEY}"
  DeleteRegKey /ifempty SHCTX "${BOTLOFT_OLD_PUBLISHER_KEY}"
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  ; A real uninstall stops the daemon and removes its scheduled task and
  ; binary. Bots and data stay in %LOCALAPPDATA%\Botloft. An uninstaller run
  ; with /UPDATE leaves the daemon running.
  ${If} $UpdateMode <> 1
    nsExec::Exec '"$INSTDIR\botloftd.exe" service uninstall'
    Pop $0
    ; Nor does the app open at sign-in any more (spec 15.2).
    DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "Botloft"
  ${EndIf}
!macroend
