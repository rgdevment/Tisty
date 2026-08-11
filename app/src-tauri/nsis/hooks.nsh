; Where the program goes, and what the installer deliberately leaves alone.
;
; It does not touch PATH. NSIS is built with 1024-character strings unless it
; is rebuilt for large ones, and Tauri ships the ordinary build: reading a PATH
; longer than that returns nothing, and writing it back wipes every entry the
; person had. Measured here on a 1724-character PATH — twenty-two entries gone,
; on install and again on uninstall. The command line is offered from inside
; the app instead, where the registry can be read without a length limit.
;
; It does not remove the data directory either. Uninstalling the program is not
; a request to throw away everything the person ever wrote.

!macro NSIS_HOOK_PREINSTALL
  ; Tauri's per-user default is `$LOCALAPPDATA\<product>`, which on Windows is
  ; the very directory the store lives in — `tisty` and `Tisty` are one folder.
  ; Programs go under Programs, and the history stays where it was.
  ;
  ; Only the untouched default is redirected: a path the person chose, or one
  ; restored from an earlier install, is theirs and is left alone.
  ${If} $INSTDIR == "$LOCALAPPDATA\${PRODUCTNAME}"
    StrCpy $INSTDIR "$LOCALAPPDATA\Programs\${PRODUCTNAME}"
    SetOutPath $INSTDIR
  ${EndIf}
!macroend

!macro NSIS_HOOK_PREUNINSTALL
!macroend
