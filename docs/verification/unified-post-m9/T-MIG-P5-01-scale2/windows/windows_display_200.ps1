# T-MIG-P5-01 VM setup only: 1920x1080 primary display and 200% (192 DPI)
# scaling for the current user, effective at the next logon. Reverted by the
# pre-test snapshot. Writes display-setup.json next to the script.
$ErrorActionPreference = 'Stop'
Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class LintraDisplay {
    [StructLayout(LayoutKind.Sequential, CharSet = CharSet.Ansi)]
    public struct DEVMODE {
        [MarshalAs(UnmanagedType.ByValTStr, SizeConst = 32)] public string dmDeviceName;
        public short dmSpecVersion, dmDriverVersion, dmSize, dmDriverExtra;
        public int dmFields, dmPositionX, dmPositionY, dmDisplayOrientation, dmDisplayFixedOutput;
        public short dmColor, dmDuplex, dmYResolution, dmTTOption, dmCollate;
        [MarshalAs(UnmanagedType.ByValTStr, SizeConst = 32)] public string dmFormName;
        public short dmLogPixels;
        public int dmBitsPerPel, dmPelsWidth, dmPelsHeight, dmDisplayFlags, dmDisplayFrequency;
        public int dmICMMethod, dmICMIntent, dmMediaType, dmDitherType, dmReserved1, dmReserved2, dmPanningWidth, dmPanningHeight;
    }
    [DllImport("user32.dll", CharSet = CharSet.Ansi)] public static extern bool EnumDisplaySettings(string device, int mode, ref DEVMODE dm);
    [DllImport("user32.dll", CharSet = CharSet.Ansi)] public static extern int ChangeDisplaySettings(ref DEVMODE dm, int flags);
}
'@
$result = [ordered]@{}
$dm = New-Object LintraDisplay+DEVMODE
$dm.dmSize = [Runtime.InteropServices.Marshal]::SizeOf($dm)
[LintraDisplay]::EnumDisplaySettings([NullString]::Value, -1, [ref]$dm) | Out-Null
$result.before = "$($dm.dmPelsWidth)x$($dm.dmPelsHeight)"
$dm.dmPelsWidth = 1920; $dm.dmPelsHeight = 1080; $dm.dmFields = 0x80000 -bor 0x100000
$result.change = [LintraDisplay]::ChangeDisplaySettings([ref]$dm, 1)  # CDS_UPDATEREGISTRY; 0 = DISP_CHANGE_SUCCESSFUL
[LintraDisplay]::EnumDisplaySettings([NullString]::Value, -1, [ref]$dm) | Out-Null
$result.after = "$($dm.dmPelsWidth)x$($dm.dmPelsHeight)"
Set-ItemProperty 'HKCU:\Control Panel\Desktop' -Name LogPixels -Value 192 -Type DWord
Set-ItemProperty 'HKCU:\Control Panel\Desktop' -Name Win8DpiScaling -Value 1 -Type DWord
$result.log_pixels = (Get-ItemProperty 'HKCU:\Control Panel\Desktop').LogPixels
$result | ConvertTo-Json | Set-Content (Join-Path $PSScriptRoot 'display-setup.json') -Encoding utf8
if ($result.change -ne 0 -or $result.after -ne '1920x1080') { exit 1 }
