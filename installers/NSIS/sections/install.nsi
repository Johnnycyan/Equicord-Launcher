Section /o "Discord Stable" InstallStable

	SetOutPath "$INSTDIR\Stable"
	File "/oname=Equicord.exe" "${BINARIES_ROOT}\equicord-stable.exe"
	File "${BINARIES_ROOT}\equicord_launcher.dll"

	WriteRegStr HKCU "Software\Equicord Launcher\Stable" "" "$INSTDIR\Stable"

	WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Equicord" "DisplayName" "Equicord"
	WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Equicord" "HelpLink" "https://equicord.dev"
	WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Equicord" "InstallLocation" "$INSTDIR\Stable"
	WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Equicord" "InstallSource" "https://equicord.dev"
	WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Equicord" "UninstallString" "$\"$INSTDIR\Uninstall Equicord.exe$\" /Stable"
	WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Equicord" "QuietUninstallString" "$\"$INSTDIR\Uninstall Equicord.exe$\" /Stable /S"
	WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Equicord" "DisplayIcon" "$INSTDIR\Stable\Equicord.exe"

	CreateDirectory "$SMPROGRAMS\Equicord"
	CreateShortCut "$SMPROGRAMS\Equicord\Equicord.lnk" "$INSTDIR\Stable\Equicord.exe" "" "$INSTDIR\Stable\Equicord.exe"

SectionEnd


Section /o "Discord PTB" InstallPTB

	SetOutPath "$INSTDIR\PTB"
	File "/oname=Equicord PTB.exe" "${BINARIES_ROOT}\equicord-ptb.exe"
	File "${BINARIES_ROOT}\equicord_launcher.dll"

	WriteRegStr HKCU "Software\Equicord Launcher\PTB" "" "$INSTDIR\PTB"

	WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Equicord PTB" "DisplayName" "Equicord PTB"
	WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Equicord PTB" "HelpLink" "https://equicord.dev"
	WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Equicord PTB" "InstallSource" "https://equicord.dev"
	WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Equicord PTB" "InstallLocation" "$INSTDIR\PTB"
	WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Equicord PTB" "UninstallString" "$\"$INSTDIR\Uninstall Equicord.exe$\" /PTB"
	WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Equicord PTB" "QuietUninstallString" "$\"$INSTDIR\Uninstall Equicord.exe$\" /PTB /S"
	WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Equicord PTB" "DisplayIcon" "$INSTDIR\PTB\Equicord PTB.exe"

	CreateDirectory "$SMPROGRAMS\Equicord"
	CreateShortCut "$SMPROGRAMS\Equicord\Equicord PTB.lnk" "$INSTDIR\PTB\Equicord PTB.exe" "" "$INSTDIR\PTB\Equicord PTB.exe"

SectionEnd


Section /o "Discord Canary" InstallCanary

	SetOutPath "$INSTDIR\Canary"
	File "/oname=Equicord Canary.exe" "${BINARIES_ROOT}\equicord-canary.exe"
	File "${BINARIES_ROOT}\equicord_launcher.dll"

	WriteRegStr HKCU "Software\Equicord Launcher\Canary" "" "$INSTDIR\Canary"

	WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Equicord Canary" "DisplayName" "Equicord Canary"
	WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Equicord Canary" "HelpLink" "https://equicord.dev"
	WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Equicord Canary" "InstallSource" "https://equicord.dev"
	WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Equicord Canary" "InstallLocation" "$INSTDIR\Canary"
	WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Equicord Canary" "UninstallString" "$\"$INSTDIR\Uninstall Equicord.exe$\" /Canary"
	WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Equicord Canary" "QuietUninstallString" "$\"$INSTDIR\Uninstall Equicord.exe$\" /Canary /S"
	WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Equicord Canary" "DisplayIcon" "$INSTDIR\Canary\Equicord Canary.exe"

	CreateDirectory "$SMPROGRAMS\Equicord"
	CreateShortCut "$SMPROGRAMS\Equicord\Equicord Canary.lnk" "$INSTDIR\Canary\Equicord Canary.exe" "" "$INSTDIR\Canary\Equicord Canary.exe"

SectionEnd

Function .onInstSuccess

	WriteUninstaller "$INSTDIR\Uninstall Equicord.exe"

	CreateDirectory "$SMPROGRAMS\Equicord"

FunctionEnd
