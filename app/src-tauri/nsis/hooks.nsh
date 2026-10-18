; The program goes under Programs, not into the folder the store lives in:
; $LOCALAPPDATA\Tisty and $LOCALAPPDATA\tisty are one folder on Windows.
;
; PATH is left alone. NSIS reads at most 1024 characters and writes back what
; it managed to read, which cost a 1724-character PATH its twenty-two entries.
; The app offers the command line instead, where the value is read whole.
;
; The data directory is left alone too: wanting the program gone is not wanting
; the history gone.

!macro NSIS_HOOK_PREINSTALL
  ; Only when nothing is installed yet: an existing install keeps its place,
  ; or the old copy would be orphaned where the store lives.
  ReadRegStr $0 SHCTX "Software\${MANUFACTURER}\${PRODUCTNAME}" ""
  ${If} $0 == ""
  ${AndIf} $INSTDIR == "$LOCALAPPDATA\${PRODUCTNAME}"
    StrCpy $INSTDIR "$LOCALAPPDATA\Programs\${PRODUCTNAME}"
    SetOutPath $INSTDIR
  ${EndIf}

  ; Moved aside rather than closed, so no assistant's door is cut; the door hands over by itself.
  Delete "$INSTDIR\ns*.tmp.old"
  ${If} ${FileExists} "$INSTDIR\tisty.exe"
    Push $R9
    GetTempFileName $R9 "$INSTDIR"
    Delete $R9
    ClearErrors
    Rename "$INSTDIR\tisty.exe" "$R9.old"
    ${If} ${Errors}
      !insertmacro CheckIfAppIsRunning "tisty.exe" "${PRODUCTNAME}"
    ${EndIf}
    Pop $R9
  ${EndIf}
!macroend

; Tisty only joins "Open with" for Markdown and calendars: the default program stays the person's choice.
!macro NSIS_HOOK_POSTINSTALL
  WriteRegStr SHCTX "Software\Classes\tisty" "" "URL:Tisty"
  WriteRegStr SHCTX "Software\Classes\tisty" "URL Protocol" ""
  WriteRegStr SHCTX "Software\Classes\tisty\DefaultIcon" "" "$INSTDIR\${MAINBINARYNAME}.exe,0"
  WriteRegStr SHCTX "Software\Classes\tisty\shell\open\command" "" '"$INSTDIR\${MAINBINARYNAME}.exe" -- "%1"'
  WriteRegStr SHCTX "Software\Classes\Tisty.Markdown" "" "Markdown"
  WriteRegStr SHCTX "Software\Classes\Tisty.Markdown\DefaultIcon" "" "$INSTDIR\${MAINBINARYNAME}.exe,0"
  WriteRegStr SHCTX "Software\Classes\Tisty.Markdown\shell\open\command" "" '"$INSTDIR\${MAINBINARYNAME}.exe" -- "%1"'
  WriteRegStr SHCTX "Software\Classes\.md\OpenWithProgids" "Tisty.Markdown" ""
  WriteRegStr SHCTX "Software\Classes\.markdown\OpenWithProgids" "Tisty.Markdown" ""
  WriteRegStr SHCTX "Software\Classes\Tisty.Calendar" "" "Calendar"
  WriteRegStr SHCTX "Software\Classes\Tisty.Calendar\DefaultIcon" "" "$INSTDIR\${MAINBINARYNAME}.exe,0"
  WriteRegStr SHCTX "Software\Classes\Tisty.Calendar\shell\open\command" "" '"$INSTDIR\${MAINBINARYNAME}.exe" -- "%1"'
  WriteRegStr SHCTX "Software\Classes\.ics\OpenWithProgids" "Tisty.Calendar" ""
  WriteRegStr SHCTX "Software\Classes\.ical\OpenWithProgids" "Tisty.Calendar" ""
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  ; The window is not the only thing running: an assistant keeps `tisty.exe mcp`
  ; alive for as long as its client lives, and a locked binary leaves the folder
  ; behind. Both are closed before anything is taken away.
  !insertmacro CheckIfAppIsRunning "${MAINBINARYNAME}.exe" "${PRODUCTNAME}"
  !insertmacro CheckIfAppIsRunning "tisty.exe" "${PRODUCTNAME}"

  ; The app put the PATH entry and the startup entry there, and is the only
  ; thing that can read the PATH value whole to take it back out. It also sweeps
  ; its own settings and cache on the way, which are the program's, not the
  ; person's.
  ;
  ; Not while updating: the generated installer.nsi guards its own shortcuts and
  ; startup key the same way, and an update has no business undoing either.
  ${If} $UpdateMode <> 1
  ${AndIf} ${FileExists} "$INSTDIR\${MAINBINARYNAME}.exe"
    ExecWait '"$INSTDIR\${MAINBINARYNAME}.exe" --unreach'
  ${EndIf}
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  ; What the binary could not sweep because it was already gone. The store is
  ; left where it is on purpose: wanting the program gone is not wanting the
  ; documents gone, and they are the person's.
  ${If} $UpdateMode <> 1
    DeleteRegKey SHCTX "Software\Classes\tisty"
    DeleteRegKey SHCTX "Software\Classes\Tisty.Markdown"
    DeleteRegValue SHCTX "Software\Classes\.md\OpenWithProgids" "Tisty.Markdown"
    DeleteRegValue SHCTX "Software\Classes\.markdown\OpenWithProgids" "Tisty.Markdown"
    DeleteRegKey SHCTX "Software\Classes\Tisty.Calendar"
    DeleteRegValue SHCTX "Software\Classes\.ics\OpenWithProgids" "Tisty.Calendar"
    DeleteRegValue SHCTX "Software\Classes\.ical\OpenWithProgids" "Tisty.Calendar"
    RMDir /r "$LOCALAPPDATA\${PRODUCTNAME}\config"
    RMDir /r "$LOCALAPPDATA\${PRODUCTNAME}\cache"
    RMDir /r "$LOCALAPPDATA\${PRODUCTNAME}\secrets"
    Delete "$INSTDIR\ns*.tmp.old"
    RMDir "$INSTDIR"
  ${EndIf}
!macroend
