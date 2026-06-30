[Setup]
; Dastur nomlari va versiyasi
AppName=UltraNet AI
AppVersion=1.0.2
AppPublisher=Nodirbek
DefaultDirName={autopf}\UltraNet AI
DefaultGroupName=UltraNet AI
; O'rnatuvchi fayl (Setup.exe) qayerga tushishi
OutputDir=build\windows\installer
OutputBaseFilename=UltraNet_Setup
Compression=lzma
SolidCompression=yes
; O'rnatish paytida Admin huquqini so'rash
PrivilegesRequired=admin

[Tasks]
Name: "desktopicon"; Description: "Ekranga (Desktop) belgisini chiqarish"; GroupDescription: "Qo'shimcha sozlamalar:"

[Files]
; Dasturning asosiy .exe fayli va qolgan barcha DLL/Data fayllari
Source: "build\windows\x64\runner\Release\ultranet.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "build\windows\x64\runner\Release\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs

[Icons]
; Pusk va Ekranga ikonka yaratish
Name: "{group}\UltraNet AI"; Filename: "{app}\ultranet.exe"
Name: "{autodesktop}\UltraNet AI"; Filename: "{app}\ultranet.exe"; Tasks: desktopicon

[Run]
; O'rnatib bo'lgach dasturni ishga tushirish
Filename: "{app}\ultranet.exe"; Description: "UltraNet AI dasturini ishga tushirish"; Flags: nowait postinstall skipifsilent
