Section /o "un.Stable" UninstallStable

  Delete "$INSTDIR\Stable\Equicord.exe"
  Delete "$INSTDIR\Stable\equicord_launcher.dll"
  RMDir "$INSTDIR\Stable"

  Delete "$SMPROGRAMS\Equicord\Equicord.lnk"

  DeleteRegKey HKCU "Software\Equicord Launcher\Stable"

	DeleteRegKey HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Equicord"

SectionEnd


Section /o "un.PTB" UninstallPTB

  Delete "$INSTDIR\PTB\Equicord PTB.exe"
  Delete "$INSTDIR\PTB\equicord_launcher.dll"
  RMDir "$INSTDIR\PTB"

  Delete "$SMPROGRAMS\Equicord\Equicord PTB.lnk"

  DeleteRegKey HKCU "Software\Equicord Launcher\PTB"

	DeleteRegKey HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Equicord PTB"

SectionEnd


Section /o "un.Canary" UninstallCanary

  Delete "$INSTDIR\Canary\Equicord Canary.exe"
  Delete "$INSTDIR\Canary\equicord_launcher.dll"
  RMDir "$INSTDIR\Canary"

  Delete "$SMPROGRAMS\Equicord\Equicord Canary.lnk"

  DeleteRegKey HKCU "Software\Equicord Launcher\Canary"
  
	DeleteRegKey HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Equicord Canary"

SectionEnd

# If Canary, PTB, and Stable are all uninstalled, remove the cache folder and uninstaller
Function un.onUninstSuccess
  IfFileExists "$INSTDIR\Canary" EndFunc 0
  IfFileExists "$INSTDIR\PTB" EndFunc 0
  IfFileExists "$INSTDIR\Stable" EndFunc 0

  RMDir /r "$INSTDIR\cache"
  Delete "$INSTDIR\Uninstall Equicord.exe"
  RMDir "$INSTDIR"

  DeleteRegKey HKCU "Software\Equicord Launcher"

  IfFileExists "$SMPROGRAMS\Equicord\Equicord.lnk" EndFunc 0
  IfFileExists "$SMPROGRAMS\Equicord\Equicord PTB.lnk" EndFunc 0
  IfFileExists "$SMPROGRAMS\Equicord\Equicord Canary.lnk" EndFunc 0

  RMDir "$SMPROGRAMS\Equicord"

  EndFunc:
FunctionEnd
