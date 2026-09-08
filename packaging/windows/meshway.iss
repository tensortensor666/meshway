#define MyAppName "Meshway"
#define MyAppVersion "0.1.0"
#define MyAppPublisher "Meshway contributors"
#define MyAppExeName "meshway.exe"

[Setup]
AppId={{A6C9B831-7554-4ADB-9A12-0F7AA1D68BE3}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
SetupIconFile=meshway.ico
DefaultDirName={localappdata}\Programs\Meshway
DefaultGroupName=Meshway
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
OutputDir=..\..\installer-output
OutputBaseFilename=Meshway-Setup
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
UninstallDisplayIcon={app}\{#MyAppExeName}

[Files]
Source: "..\..\backend\target\release\meshway.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\frontend\dist\*"; DestDir: "{app}\frontend\dist"; Flags: ignoreversion recursesubdirs createallsubdirs
Source: "meshway.env.example"; DestDir: "{app}"; DestName: ".env.example"; Flags: ignoreversion
Source: "..\..\LICENSE"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\Meshway"; Filename: "{app}\{#MyAppExeName}"; IconFilename: "{app}\{#MyAppExeName}"; WorkingDir: "{app}"
Name: "{userdesktop}\Meshway"; Filename: "{app}\{#MyAppExeName}"; IconFilename: "{app}\{#MyAppExeName}"; WorkingDir: "{app}"; Tasks: desktopicon

[Tasks]
Name: "desktopicon"; Description: "创建桌面快捷方式"; GroupDescription: "附加选项："; Flags: unchecked

[Run]
Filename: "{app}\{#MyAppExeName}"; WorkingDir: "{app}"; Description: "启动 Meshway"; Flags: nowait postinstall skipifsilent
