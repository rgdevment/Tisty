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

  ; The template waits for the window but not for the command line beside it, and
  ; a locked tisty.exe leaves the install half done with no version left to offer
  ; the update again. Closing it costs nothing: every write is a whole line or a
  ; rename.
  !insertmacro CheckIfAppIsRunning "tisty.exe" "${PRODUCTNAME}"
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
    RMDir /r "$LOCALAPPDATA\${PRODUCTNAME}\config"
    RMDir /r "$LOCALAPPDATA\${PRODUCTNAME}\cache"
    RMDir "$INSTDIR"
  ${EndIf}
!macroend
