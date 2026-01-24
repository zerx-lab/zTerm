; Axon Term - Windows Installer Script
; Built with Inno Setup 6
; https://jrsoftware.org/isinfo.php

#define AppName "Axon Term"
#define AppVersion GetEnv("AXON_VERSION")
#if AppVersion == ""
#define AppVersion "0.1.0"
#endif
#define AppPublisher "Axon Team"
#define AppURL "https://github.com/axon-term/axon_term"
#define AppExeName "axon_term.exe"
#define AppId "{{A8E2F9C1-4B6D-4E8F-9A2C-3D5E7F1A2B4C}"

; 从环境变量获取路径 (由打包脚本设置)
#define TargetDir GetEnv("AXON_TARGET_DIR")
#if TargetDir == ""
#define TargetDir "..\..\target"
#endif
#define ProjectRoot GetEnv("AXON_PROJECT_ROOT")
#if ProjectRoot == ""
#define ProjectRoot "..\..\"
#endif

[Setup]
AppId={#AppId}
AppName={#AppName}
AppVersion={#AppVersion}
AppVerName={#AppName} {#AppVersion}
AppPublisher={#AppPublisher}
AppPublisherURL={#AppURL}
AppSupportURL={#AppURL}
AppUpdatesURL={#AppURL}
DefaultDirName={autopf}\{#AppName}
DefaultGroupName={#AppName}
AllowNoIcons=yes
; 输出目录和文件名
OutputDir={#TargetDir}\installer
OutputBaseFilename=AxonTerm-{#AppVersion}-x64-setup
; 压缩设置
Compression=lzma2/ultra64
SolidCompression=yes
; 架构设置
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
; 权限设置 - 不需要管理员权限
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
; 最低 Windows 版本 (Windows 10 1709+)
MinVersion=10.0.16299
; 安装程序图标
SetupIconFile=app-icon.ico
UninstallDisplayIcon={app}\{#AppExeName}
; 向导样式
WizardStyle=modern
WizardSizePercent=120
; 禁用不必要的页面
DisableProgramGroupPage=yes
; 许可证
LicenseFile={#ProjectRoot}\LICENSE

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"
Name: "chinesesimplified"; MessagesFile: "messages\ChineseSimplified.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked
Name: "addtopath"; Description: "Add to PATH environment variable"; GroupDescription: "System Integration:"; Flags: unchecked
Name: "addcontextmenu"; Description: "Add ""Open with Axon Term"" to context menu"; GroupDescription: "System Integration:"; Flags: unchecked

[Files]
; 主程序
Source: "{#TargetDir}\release\axon_term.exe"; DestDir: "{app}"; Flags: ignoreversion
; CLI 工具 (如果存在)
Source: "{#TargetDir}\release\axon.exe"; DestDir: "{app}\bin"; Flags: ignoreversion skipifsourcedoesntexist
; ConPTY 支持
Source: "bin\conpty.dll"; DestDir: "{app}"; Flags: ignoreversion skipifsourcedoesntexist
Source: "bin\OpenConsole.exe"; DestDir: "{app}"; Flags: ignoreversion skipifsourcedoesntexist
; 资源文件
Source: "{#ProjectRoot}\assets\*"; DestDir: "{app}\assets"; Flags: ignoreversion recursesubdirs createallsubdirs

[Icons]
Name: "{group}\{#AppName}"; Filename: "{app}\{#AppExeName}"
Name: "{group}\{cm:UninstallProgram,{#AppName}}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\{#AppName}"; Filename: "{app}\{#AppExeName}"; Tasks: desktopicon

[Registry]
; 添加到 PATH
Root: HKCU; Subkey: "Environment"; ValueType: expandsz; ValueName: "Path"; ValueData: "{olddata};{app}\bin"; Tasks: addtopath; Check: NeedsAddPath('{app}\bin')
; 右键菜单 - 文件夹背景
Root: HKCU; Subkey: "Software\Classes\Directory\Background\shell\AxonTerm"; ValueType: string; ValueName: ""; ValueData: "Open with Axon Term"; Tasks: addcontextmenu
Root: HKCU; Subkey: "Software\Classes\Directory\Background\shell\AxonTerm"; ValueType: string; ValueName: "Icon"; ValueData: """{app}\{#AppExeName}"""; Tasks: addcontextmenu
Root: HKCU; Subkey: "Software\Classes\Directory\Background\shell\AxonTerm\command"; ValueType: string; ValueName: ""; ValueData: """{app}\{#AppExeName}"" ""%V"""; Tasks: addcontextmenu
; 右键菜单 - 文件夹
Root: HKCU; Subkey: "Software\Classes\Directory\shell\AxonTerm"; ValueType: string; ValueName: ""; ValueData: "Open with Axon Term"; Tasks: addcontextmenu
Root: HKCU; Subkey: "Software\Classes\Directory\shell\AxonTerm"; ValueType: string; ValueName: "Icon"; ValueData: """{app}\{#AppExeName}"""; Tasks: addcontextmenu
Root: HKCU; Subkey: "Software\Classes\Directory\shell\AxonTerm\command"; ValueType: string; ValueName: ""; ValueData: """{app}\{#AppExeName}"" ""%1"""; Tasks: addcontextmenu

[Run]
Filename: "{app}\{#AppExeName}"; Description: "{cm:LaunchProgram,{#StringChange(AppName, '&', '&&')}}"; Flags: nowait postinstall skipifsilent

[UninstallDelete]
Type: filesandordirs; Name: "{app}"

[Code]
// 检查是否需要添加到 PATH
function NeedsAddPath(Param: string): Boolean;
var
  OrigPath: string;
begin
  if not RegQueryStringValue(HKEY_CURRENT_USER, 'Environment', 'Path', OrigPath) then
  begin
    Result := True;
    exit;
  end;
  Result := Pos(';' + Param + ';', ';' + OrigPath + ';') = 0;
end;

// 卸载时清理 PATH
procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
var
  Path: string;
  AppPath: string;
  P: Integer;
begin
  if CurUninstallStep = usPostUninstall then
  begin
    if RegQueryStringValue(HKEY_CURRENT_USER, 'Environment', 'Path', Path) then
    begin
      AppPath := ExpandConstant('{app}\bin');
      P := Pos(';' + AppPath, Path);
      if P > 0 then
      begin
        Delete(Path, P, Length(';' + AppPath));
        RegWriteStringValue(HKEY_CURRENT_USER, 'Environment', 'Path', Path);
      end;
    end;
  end;
end;
