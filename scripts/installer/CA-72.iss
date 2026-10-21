; The CA-72's installer for Windows (Inno Setup 6), made by scripts/package.sh from the
; bundles it stages: CA-72.vst3 into Common Files\VST3, CA-72.clap into Common Files\CLAP,
; and the standalone, licence, notices and uninstaller into a folder the user chooses
; (decisions.md R17). Defines: Version, Stage (the staged files), Output (where it goes).

#ifndef Version
  #error Run from scripts/package.sh, which defines Version, Stage and Output
#endif

[Setup]
; Never change the AppId: it is how a later installer finds this one's installation.
AppId={{9FF1B860-34D5-4A52-8E33-91FCAF8CF5B2}
AppName=CA-72
AppVersion={#Version}
AppVerName=CA-72 {#Version}
AppPublisher=Idle Foundry Ltd.
AppPublisherURL=https://github.com/idlefoundry/ca-72
AppSupportURL=https://github.com/idlefoundry/ca-72/issues
AppUpdatesURL=https://github.com/idlefoundry/ca-72/releases
AppCopyright=Copyright (C) 2026 Idle Foundry Ltd. GNU GPL version 3 or later.
DefaultDirName={commonpf64}\Idle Foundry\CA-72
DisableDirPage=no
UsePreviousAppDir=yes
DisableProgramGroupPage=yes
DisableWelcomePage=no
PrivilegesRequired=admin
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
MinVersion=10.0
OutputDir={#Output}
OutputBaseFilename=CA-72-{#Version}-Windows-setup
Compression=lzma2/max
SolidCompression=yes
WizardStyle=modern
UninstallDisplayName=CA-72 {#Version}
VersionInfoVersion={#Version}
VersionInfoCompany=Idle Foundry Ltd.
VersionInfoDescription=CA-72 installer

[Types]
Name: "full"; Description: "Plug-ins and standalone application"
Name: "custom"; Description: "Choose what to install"; Flags: iscustom

[Components]
Name: "vst3"; Description: "VST3, in C:\Program Files\Common Files\VST3"; Types: full custom
Name: "clap"; Description: "CLAP, in C:\Program Files\Common Files\CLAP"; Types: full custom
Name: "standalone"; Description: "Standalone application, with Start menu shortcut"; Types: full custom

[InstallDelete]
; A bundle from an earlier version goes first, so that none of its files outlive it.
Type: filesandordirs; Name: "{commoncf64}\VST3\CA-72.vst3"; Components: vst3

[Files]
Source: "{#Stage}\CA-72.vst3\*"; DestDir: "{commoncf64}\VST3\CA-72.vst3"; Components: vst3; Flags: ignoreversion recursesubdirs createallsubdirs
Source: "{#Stage}\CA-72.clap"; DestDir: "{commoncf64}\CLAP"; Components: clap; Flags: ignoreversion
Source: "{#Stage}\CA-72.exe"; DestDir: "{app}"; Components: standalone; Flags: ignoreversion
Source: "{#Stage}\ca72-mcp.exe"; DestDir: "{app}"; Components: standalone; Flags: ignoreversion
Source: "{#Stage}\CA-72.ico"; DestDir: "{app}"; Components: standalone; Flags: ignoreversion
Source: "{#Stage}\LICENSE.txt"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#Stage}\THIRD-PARTY-NOTICES.txt"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{commonprograms}\Idle Foundry\CA-72"; Filename: "{app}\CA-72.exe"; IconFilename: "{app}\CA-72.ico"; Components: standalone

[Messages]
SelectDirDesc=Where should the standalone application and its support files go?
SelectDirLabel3=The standalone application (if selected), licenses and uninstaller go in this folder.

[UninstallDelete]
Type: filesandordirs; Name: "{commoncf64}\VST3\CA-72.vst3"; Components: vst3

[Code]
var
  PreviousAppDir: String;

procedure InitializeWizard;
var
  Locations: TNewStaticText;
begin
  RegQueryStringValue(HKLM64,
    ExpandConstant('Software\Microsoft\Windows\CurrentVersion\Uninstall\{#SetupSetting("AppId")}_is1'),
    'InstallLocation', PreviousAppDir);
  Locations := TNewStaticText.Create(WizardForm);
  Locations.Parent := WizardForm.SelectDirPage;
  Locations.Left := WizardForm.DirEdit.Left;
  Locations.Top := WizardForm.DirEdit.Top + WizardForm.DirEdit.Height + ScaleY(16);
  Locations.Width := WizardForm.SelectDirLabel.Width;
  Locations.Height := ScaleY(110);
  Locations.WordWrap := True;
  Locations.Caption := 'Plug-ins stay in the standard host folders:' + #13#10 +
    ExpandConstant('VST3: {commoncf64}\VST3') + #13#10 +
    ExpandConstant('CLAP: {commoncf64}\CLAP') + #13#10#13#10 +
    'An external drive must be connected to run, update or uninstall the app. ' +
    'Updates keep this folder. To move an existing installation, uninstall it first; ' +
    'your presets and settings are kept.';
end;

function DestinationError: String;
begin
  Result := '';
  if not DirExists(AddBackslash(ExtractFileDrive(WizardDirValue))) then
    Result := 'The destination drive is unavailable. Reconnect it with the same drive letter, then try again.'
  else if (PreviousAppDir <> '') and
    (CompareText(RemoveBackslashUnlessRoot(ExpandFileName(WizardDirValue)),
      RemoveBackslashUnlessRoot(ExpandFileName(PreviousAppDir))) <> 0) then
    Result := 'Update CA-72 in its existing folder: ' + PreviousAppDir + #13#10 +
      'To choose another folder, cancel and uninstall the existing copy first. Your presets and settings are kept.';
end;

function NextButtonClick(CurPageID: Integer): Boolean;
var
  Error: String;
begin
  Result := True;
  if CurPageID = wpSelectDir then begin
    Error := DestinationError;
    Result := Error = '';
    if not Result then
      SuppressibleMsgBox(Error, mbError, MB_OK, IDOK);
  end;
end;

// Check again before any plug-in is replaced, including on a silent upgrade.
function PrepareToInstall(var NeedsRestart: Boolean): String;
begin
  Result := DestinationError;
end;

// The maker's folder goes with the last of its programs (RemoveDir leaves it if not empty).
procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
begin
  if CurUninstallStep = usPostUninstall then
    RemoveDir(ExpandConstant('{commonpf64}\Idle Foundry'));
end;
