!macro NSIS_HOOK_PREUNINSTALL
  DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "Vocab Collector"
  DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run" "Vocab Collector"
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  MessageBox MB_YESNO|MB_ICONQUESTION|MB_DEFBUTTON2 \
    "Keep your Vocab Collector vocabulary and settings for a future installation?$\r$\n$\r$\nChoose No to permanently delete the local database and settings." \
    IDYES keep_vocab_collector_data

  RMDir /r "$APPDATA\app.vocabcollector.desktop"

  keep_vocab_collector_data:
!macroend
