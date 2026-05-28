; Extra cleanup when the user selects "Delete local data" during uninstall.
!macro NSIS_HOOK_POSTUNINSTALL
  ${If} $DeleteAppDataCheckboxState = 1
  ${AndIf} $UpdateMode <> 1
    SetShellVarContext current
    ; Legacy folder name used before bundle-id alignment.
    RmDir /r "$APPDATA\Friday"
    RmDir /r "$LOCALAPPDATA\Friday"
    ; Cursor API key stored in OS credential manager (Windows).
    nsExec::ExecToLog 'cmdkey /delete:Friday/cursor_api_key'
    nsExec::ExecToLog 'cmdkey /delete:Friday/friday_data_key'
    nsExec::ExecToLog 'cmdkey /delete:Friday/stt_api_key'
  ${EndIf}
!macroend
