#ifndef AppVersion
  #error AppVersion must be defined: /DAppVersion=x.y.z
#endif
#ifndef BinDir
  #error BinDir must be defined: /DBinDir=path\to\release
#endif

[Setup]
AppName=HashCalc
AppVersion={#AppVersion}
AppPublisher=Antidote1911
AppPublisherURL=https://github.com/Antidote1911/hashcalc
AppSupportURL=https://github.com/Antidote1911/hashcalc/issues
AppUpdatesURL=https://github.com/Antidote1911/hashcalc/releases
DefaultDirName={autopf}\HashCalc
DefaultGroupName=HashCalc
AllowNoIcons=yes
OutputDir=.
OutputBaseFilename=hashcalc-v{#AppVersion}-win-setup
Compression=lzma
SolidCompression=yes
WizardStyle=modern
ArchitecturesInstallIn64BitMode=x64compatible
ChangesEnvironment=yes
MinVersion=10.0

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked
Name: "addtopath";   Description: "Add HashCalc to PATH (CLI)"; GroupDescription: "Additional options:"

[Files]
Source: "{#BinDir}\hashcalc.exe";     DestDir: "{app}"; Flags: ignoreversion
Source: "{#BinDir}\hashcalc-gui.exe"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\HashCalc GUI";                     Filename: "{app}\hashcalc-gui.exe"
Name: "{group}\{cm:UninstallProgram,HashCalc}";   Filename: "{uninstallexe}"
Name: "{autodesktop}\HashCalc GUI";               Filename: "{app}\hashcalc-gui.exe"; Tasks: desktopicon

[Registry]
Root: HKLM; Subkey: "SYSTEM\CurrentControlSet\Control\Session Manager\Environment"; \
  ValueType: expandsz; ValueName: "Path"; \
  ValueData: "{olddata};{app}"; \
  Check: NeedsAddPath('{app}'); Tasks: addtopath

[Code]
function NeedsAddPath(Param: string): boolean;
var
  OrigPath: string;
begin
  if not RegQueryStringValue(HKLM,
    'SYSTEM\CurrentControlSet\Control\Session Manager\Environment',
    'Path', OrigPath)
  then begin
    Result := True;
    exit;
  end;
  Result := Pos(';' + Param + ';', ';' + OrigPath + ';') = 0;
end;

[Run]
Filename: "{app}\hashcalc-gui.exe"; \
  Description: "{cm:LaunchProgram,HashCalc GUI}"; \
  Flags: nowait postinstall skipifsilent
