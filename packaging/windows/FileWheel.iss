; Inno Setup script for the FileWheel Windows installer.
; Build the portable folder first (packaging/windows/package.sh), then:
;   iscc /DAppVersion=0.1.0 packaging\windows\FileWheel.iss
; Produces dist\FileWheel-Setup-x86_64.exe.
;
; The installer is per-user (no administrator prompt) and installs into
; %LOCALAPPDATA%\Programs\FileWheel. Settings in %APPDATA%\FileWheel are kept
; on uninstall so a reinstall restores them.

#ifndef AppVersion
  #define AppVersion "0.0.0"
#endif

[Setup]
AppId={{6F1C2B8E-4D3A-4E7B-9A51-F11E3EE1A0C1}
AppName=FileWheel
AppVersion={#AppVersion}
AppVerName=FileWheel {#AppVersion}
AppPublisher=Danny Rudd
AppPublisherURL=https://github.com/danny19862025/FileWheel
AppSupportURL=https://github.com/danny19862025/FileWheel/issues
AppUpdatesURL=https://github.com/danny19862025/FileWheel/releases
DefaultDirName={autopf}\FileWheel
DefaultGroupName=FileWheel
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
MinVersion=10.0.17763
LicenseFile=..\..\LICENSE
SetupIconFile=..\..\data\filewheel.ico
UninstallDisplayIcon={app}\FileWheel.exe
UninstallDisplayName=FileWheel
OutputDir=..\..\dist
OutputBaseFilename=FileWheel-Setup-x86_64
Compression=lzma2/max
SolidCompression=yes
WizardStyle=modern
CloseApplications=yes
RestartApplications=no

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked
Name: "startup"; Description: "Start the FileWheel drop target when I sign in"; GroupDescription: "Startup:"; Flags: unchecked

[Files]
Source: "..\..\dist\FileWheel\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs

[Icons]
Name: "{autoprograms}\FileWheel"; Filename: "{app}\FileWheel.exe"
Name: "{autodesktop}\FileWheel"; Filename: "{app}\FileWheel.exe"; Tasks: desktopicon
Name: "{userstartup}\FileWheel"; Filename: "{app}\FileWheel.exe"; Parameters: "--background"; Tasks: startup

[Run]
Filename: "{app}\FileWheel.exe"; Description: "{cm:LaunchProgram,FileWheel}"; Flags: nowait postinstall skipifsilent

[UninstallRun]
Filename: "{sys}\taskkill.exe"; Parameters: "/F /IM FileWheel.exe"; Flags: runhidden; RunOnceId: "StopFileWheel"
Filename: "{sys}\WindowsPowerShell\v1.0\powershell.exe"; Parameters: "-NoProfile -Command ""Get-Process gdbus -ErrorAction SilentlyContinue | Where-Object {{ $_.Path -like '{app}\*' } | Stop-Process -Force"""; Flags: runhidden; RunOnceId: "StopGDBus"
