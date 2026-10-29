!macro NSIS_HOOK_POSTUNINSTALL
  ; Tauri uses Cargo package name "widget-website" for the default autostart entry.
  ; Keep the old display-name value removal as a migration safeguard.
  ; Never remove startup settings or data when the uninstaller is running in update mode.
  ${If} $UpdateMode <> 1
    ${If} $DeleteAppDataCheckboxState == 1
      SetShellVarContext current
      RmDir /r "$APPDATA\WidgetWebsite"
      RmDir /r "$LOCALAPPDATA\WidgetWebsite"
    ${EndIf}

    DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "widget-website"
    DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "Widget Website"
  ${EndIf}
!macroend
