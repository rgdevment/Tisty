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
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  ; The app put the PATH entry and the startup entry there, and is the only
  ; thing that can read the PATH value whole to take it back out.
  ${If} ${FileExists} "$INSTDIR\${MAINBINARYNAME}.exe"
    ExecWait '"$INSTDIR\${MAINBINARYNAME}.exe" --unreach'
  ${EndIf}
!macroend
