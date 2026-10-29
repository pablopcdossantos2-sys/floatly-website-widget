!macro NSIS_HOOK_POSTUNINSTALL
  ; The standard Tauri NSIS checkbox sets this variable when the user opts
  ; into deleting application data. Keep data intact when it is unchecked.
  ${If} $DeleteAppDataCheckboxState == 1
    SetShellVarContext current
    RmDir /r "$APPDATA\WidgetWebsite"
    RmDir /r "$LOCALAPPDATA\WidgetWebsite"
  ${EndIf}

  ; Autostart is an installation setting, not user session data. Always remove
  ; the startup entry when the application itself is uninstalled.
  DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "Widget Website"
!macroend
