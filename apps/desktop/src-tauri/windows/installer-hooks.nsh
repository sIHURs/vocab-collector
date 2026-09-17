; The pinned template passes /UPDATE for installer-driven replacement.
!macro NSIS_HOOK_PREUNINSTALL
  ${If} $UpdateMode <> 1
    DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "Vocab Collector Release"
    DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run" "Vocab Collector Release"
  ${EndIf}
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  ${If} $UpdateMode <> 1
    ; Fixed per-user release directories; external files and development roots stay.
    SetShellVarContext current
    ClearErrors
    RMDir /r "$APPDATA\app.vocabcollector.desktop.release"
    RMDir /r "$LOCALAPPDATA\app.vocabcollector.desktop.release"
    ${If} ${FileExists} "$APPDATA\app.vocabcollector.desktop.release"
    ${OrIf} ${FileExists} "$LOCALAPPDATA\app.vocabcollector.desktop.release"
      SetErrorLevel 2
      IfSilent vocab_cleanup_done
      MessageBox MB_OK|MB_ICONEXCLAMATION "Some local data could not be removed. Close Vocab Collector and remove the app.vocabcollector.desktop.release folders in your AppData Roaming and Local directories. Other files were not targeted."
    ${EndIf}
    vocab_cleanup_done:
  ${EndIf}
!macroend
