# Native Windows shell captures and keyboard interaction (T-MIG-P5-01).
#
# Counterpart of native_capture.py for WebView2. Task evidence, not a CI gate.
# Run in the real console session of a standard user (Interactive/Limited
# scheduled task): the script rejects elevated tokens, session 0 and sessions
# other than the active console. Every scenario starts its own process, acts
# only on that process's verified main window (size, foreground, keys via
# SendKeys only while it owns the foreground), captures its client area, then
# closes it normally and requires exit code 0.
#
# Usage: powershell -NoProfile -ExecutionPolicy Bypass -File windows_capture.ps1 <exe> <output-dir>
param([Parameter(Mandatory)] [string] $Exe, [Parameter(Mandatory)] [string] $Output)
$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'
if (Test-Path -LiteralPath $Output) { throw 'Refusing to overwrite capture evidence' }
New-Item -ItemType Directory -Path $Output | Out-Null
Add-Type -AssemblyName System.Drawing, System.Windows.Forms
Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class LintraShellWindow {
    [StructLayout(LayoutKind.Sequential)] public struct Rect { public int Left, Top, Right, Bottom; }
    [StructLayout(LayoutKind.Sequential)] public struct Point { public int X, Y; }
    [DllImport("kernel32.dll")] public static extern uint WTSGetActiveConsoleSessionId();
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr w, out Rect r);
    [DllImport("user32.dll")] public static extern bool GetClientRect(IntPtr w, out Rect r);
    [DllImport("user32.dll")] public static extern bool ClientToScreen(IntPtr w, ref Point p);
    [DllImport("user32.dll")] public static extern bool SetWindowPos(IntPtr w, IntPtr after, int x, int y, int cx, int cy, uint flags);
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr w);
    [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
    [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr w);
    [DllImport("user32.dll")] public static extern uint GetDpiForWindow(IntPtr w);
}
'@

# name, client (CSS) size or $null for the whole work area, SendKeys sequence,
# system theme (light/dark). Tab order: skip link, Analisi, Risultati,
# Informazioni, theme radio group. 640x480 exercises the narrow bar; the work
# area (wider than the 960 px breakpoint) exercises the side column. Larger
# fixed sizes are refused explicitly when the monitor cannot hold them.
$scenarios = @(
    @('analisi-640x480', @(640, 480), @(), 'light'),
    @('analisi-workarea', $null, @(), 'light'),
    @('analisi-system-dark-640x480', @(640, 480), @(), 'dark'),
    @('analisi-system-dark-workarea', $null, @(), 'dark'),
    @('keyboard-skip-link-workarea', $null, @('{TAB}'), 'light'),
    @('keyboard-risultati-workarea', $null, @('{TAB}', '{TAB}', '{TAB}', '{ENTER}'), 'light'),
    @('keyboard-risultati-640x480', @(640, 480), @('{TAB}', '{TAB}', '{TAB}', '{ENTER}'), 'light'),
    @('keyboard-informazioni-640x480', @(640, 480), @('{TAB}', '{TAB}', '{TAB}', '{TAB}', '{ENTER}'), 'light'),
    @('keyboard-theme-light-on-dark-workarea', $null, @('{TAB}', '{TAB}', '{TAB}', '{TAB}', '{TAB}', '{DOWN}'), 'dark'),
    @('keyboard-theme-dark-workarea', $null, @('{TAB}', '{TAB}', '{TAB}', '{TAB}', '{TAB}', '{DOWN}', '{DOWN}'), 'light'),
    @('keyboard-theme-dark-640x480', @(640, 480), @('{TAB}', '{TAB}', '{TAB}', '{TAB}', '{TAB}', '{DOWN}', '{DOWN}'), 'light')
)

$identity = [Security.Principal.WindowsIdentity]::GetCurrent()
$principal = [Security.Principal.WindowsPrincipal]::new($identity)
$session = [Diagnostics.Process]::GetCurrentProcess().SessionId
$result = [ordered]@{
    passed = $false; account = $identity.Name; os_version = [Environment]::OSVersion.Version.ToString()
    elevated = $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
    session = $session; console_session = [LintraShellWindow]::WTSGetActiveConsoleSessionId()
    exe_sha256 = (Get-FileHash -LiteralPath $Exe -Algorithm SHA256).Hash.ToLowerInvariant()
    scenarios = @()
}
$themeKey = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize'
$originalTheme = (Get-ItemProperty -Path $themeKey -Name AppsUseLightTheme -ErrorAction SilentlyContinue).AppsUseLightTheme

function Assert-Foreground([IntPtr] $window) {
    if ([LintraShellWindow]::GetForegroundWindow() -ne $window) {
        [LintraShellWindow]::SetForegroundWindow($window) | Out-Null
        Start-Sleep -Milliseconds 300
        if ([LintraShellWindow]::GetForegroundWindow() -ne $window) { throw 'Owned window lost the foreground' }
    }
}

function Invoke-Scenario($name, $size, $keys, $theme) {
    $record = [ordered]@{ name = $name; keys = $keys; system_theme = $theme; passed = $false }
    Set-ItemProperty -Path $themeKey -Name AppsUseLightTheme -Value ([int]($theme -eq 'light')) -Type DWord
    $app = Start-Process -FilePath $Exe -PassThru
    try {
        $deadline = [DateTime]::UtcNow.AddSeconds(30)
        do { Start-Sleep -Milliseconds 300; $app.Refresh() } until ($app.HasExited -or $app.MainWindowHandle -ne [IntPtr]::Zero -or [DateTime]::UtcNow -ge $deadline)
        if ($app.HasExited -or $app.MainWindowHandle -eq [IntPtr]::Zero) { throw 'Owned window not created' }
        $window = $app.MainWindowHandle
        if (-not [LintraShellWindow]::IsWindowVisible($window) -or $app.MainWindowTitle -ne 'Lintra') { throw 'Unexpected window' }
        if ($app.SessionId -ne $session) { throw 'Window outside this console session' }
        $area = [Windows.Forms.Screen]::PrimaryScreen.WorkingArea
        $outer = [LintraShellWindow+Rect]::new(); $client = [LintraShellWindow+Rect]::new()
        [LintraShellWindow]::GetWindowRect($window, [ref]$outer) | Out-Null
        [LintraShellWindow]::GetClientRect($window, [ref]$client) | Out-Null
        $frameW = ($outer.Right - $outer.Left) - $client.Right
        $frameH = ($outer.Bottom - $outer.Top) - $client.Bottom
        if ($null -eq $size) { $size = @(($area.Width - $frameW), ($area.Height - $frameH)) }
        if ($size[0] + $frameW -gt $area.Width -or $size[1] + $frameH -gt $area.Height) { throw "Size $size does not fit the work area" }
        # SWP_NOZORDER: resize/move only, keep activation to the process.
        [LintraShellWindow]::SetWindowPos($window, [IntPtr]::Zero, $area.Left, $area.Top, $size[0] + $frameW, $size[1] + $frameH, 0x0004) | Out-Null
        Start-Sleep -Seconds 4  # bootstrap IPC, local fonts and two animation frames
        Assert-Foreground $window
        foreach ($key in $keys) {
            Assert-Foreground $window
            [Windows.Forms.SendKeys]::SendWait($key)
            Start-Sleep -Milliseconds 400
        }
        Start-Sleep -Milliseconds 600
        Assert-Foreground $window
        [LintraShellWindow]::GetClientRect($window, [ref]$client) | Out-Null
        $origin = [LintraShellWindow+Point]::new()
        [LintraShellWindow]::ClientToScreen($window, [ref]$origin) | Out-Null
        $record.requested = $size
        $record.captured = @($client.Right, $client.Bottom)
        $record.dpi = [LintraShellWindow]::GetDpiForWindow($window)
        if ($record.captured[0] -ne $size[0] -or $record.captured[1] -ne $size[1]) { throw 'Client size differs from the request' }
        $bitmap = [Drawing.Bitmap]::new($client.Right, $client.Bottom)
        $graphics = [Drawing.Graphics]::FromImage($bitmap)
        try {
            $graphics.CopyFromScreen([Drawing.Point]::new($origin.X, $origin.Y), [Drawing.Point]::Empty, $bitmap.Size)
            $image = Join-Path $Output "$name.png"
            $bitmap.Save($image, [Drawing.Imaging.ImageFormat]::Png)
            $record.image_sha256 = (Get-FileHash -LiteralPath $image -Algorithm SHA256).Hash.ToLowerInvariant()
        } finally { $graphics.Dispose(); $bitmap.Dispose() }
        if (-not $app.CloseMainWindow() -or -not $app.WaitForExit(15000)) { throw 'Normal close failed' }
        $record.exit_code = $app.ExitCode
        if ($app.ExitCode -ne 0) { throw 'Non-zero exit code' }
        $record.passed = $true
    } catch { $record.error = $_.Exception.Message }
    finally { if (-not $app.HasExited) { Stop-Process -Id $app.Id -Force -ErrorAction SilentlyContinue } }
    return $record
}

try {
    if ($result.elevated -or $session -eq 0 -or $session -ne $result.console_session) {
        throw 'A real non-elevated console desktop is required'
    }
    foreach ($item in $scenarios) { $result.scenarios += Invoke-Scenario $item[0] $item[1] $item[2] $item[3] }
    # A key sequence must change the pixels of the untouched capture with the
    # same size and system theme.
    foreach ($record in $result.scenarios | Where-Object { $_.passed -and $_.keys.Count -gt 0 }) {
        $base = $result.scenarios | Where-Object { $_.passed -and $_.keys.Count -eq 0 -and $_.system_theme -eq $record.system_theme -and "$($_.captured)" -eq "$($record.captured)" } | Select-Object -First 1
        if ($null -eq $base -or $base.image_sha256 -eq $record.image_sha256) { $record.passed = $false; $record.error = 'No pixel change against the baseline' }
    }
    $result.passed = -not ($result.scenarios | Where-Object { -not $_.passed })
} catch { $result.error = $_.Exception.Message }
finally {
    if ($null -eq $originalTheme) { Remove-ItemProperty -Path $themeKey -Name AppsUseLightTheme -ErrorAction SilentlyContinue }
    else { Set-ItemProperty -Path $themeKey -Name AppsUseLightTheme -Value $originalTheme -Type DWord }
    $result | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath (Join-Path $Output 'receipt.json') -Encoding utf8
}
if (-not $result.passed) { exit 1 }
