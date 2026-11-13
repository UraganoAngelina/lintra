# P1-05 only: run through an Interactive/Limited task in the real console.
# Own every GUI process; capture only the synthetic probe's verified window.
$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'
$root = Join-Path $env:USERPROFILE 'LintraQualification\P1-05'
$output = Join-Path $env:LOCALAPPDATA 'LintraQualification\P1-05'
if (Test-Path -LiteralPath $output) { throw 'Refusing to overwrite profile evidence' }
New-Item -ItemType Directory -Path $output | Out-Null
$result = [ordered]@{ passed = $false; account = [Security.Principal.WindowsIdentity]::GetCurrent().Name }
$app = $null
$node = $null
try {
    Add-Type -AssemblyName System.Drawing
    Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class LintraProfileConsole {
    [StructLayout(LayoutKind.Sequential)] public struct Rect { public int Left, Top, Right, Bottom; }
    [DllImport("kernel32.dll")] public static extern uint WTSGetActiveConsoleSessionId();
    [DllImport("user32.dll")] public static extern uint GetDpiForWindow(IntPtr window);
    [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr window);
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr window);
    [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr window, out Rect rect);
}
'@
    $principal = [Security.Principal.WindowsPrincipal]::new([Security.Principal.WindowsIdentity]::GetCurrent())
    $result.elevated = $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
    $result.session = [Diagnostics.Process]::GetCurrentProcess().SessionId
    $result.console_session = [LintraProfileConsole]::WTSGetActiveConsoleSessionId()
    $result.os_version = [Environment]::OSVersion.Version.ToString()
    if ($result.elevated -or $result.session -eq 0 -or $result.session -ne $result.console_session) {
        throw 'A real non-elevated console desktop is required'
    }
    $exe = Join-Path $root 'profile-probe.exe'
    $node = [Diagnostics.Process]::new()
    $node.StartInfo.FileName = 'C:\Program Files\nodejs\node.exe'
    $harness = Join-Path $root 'source\spikes\tauri-runtime\tools\native_profile.cjs'
    $evidence = Join-Path $output 'native-profile.json'
    $node.StartInfo.Arguments = '"' + $harness + '" "' + $exe + '" "' + $evidence + '"'
    $node.StartInfo.UseShellExecute = $false
    $node.StartInfo.CreateNoWindow = $true
    $node.StartInfo.RedirectStandardOutput = $true
    $node.StartInfo.RedirectStandardError = $true
    if (-not $node.Start()) { throw 'Cannot start profile harness' }
    $stdout = $node.StandardOutput.ReadToEndAsync()
    $stderr = $node.StandardError.ReadToEndAsync()
    if (-not $node.WaitForExit(120000)) { throw 'Profile harness timed out' }
    $result.harness_exit = $node.ExitCode
    Set-Content (Join-Path $output 'harness-stdout.txt') -Value $stdout.Result -Encoding utf8
    Set-Content (Join-Path $output 'harness-stderr.txt') -Value $stderr.Result -Encoding utf8
    $native = Get-Content -LiteralPath $evidence -Raw | ConvertFrom-Json
    $result.native = $native
    if ($node.ExitCode -ne 0 -or -not $native.passed) { throw 'Native profile measurements failed' }
    $app = Start-Process -FilePath $exe -ArgumentList '--profile-test' -PassThru -RedirectStandardOutput (Join-Path $output 'capture-stdout.txt') -RedirectStandardError (Join-Path $output 'capture-stderr.txt')
    $null = $app.Handle
    $deadline = [DateTime]::UtcNow.AddSeconds(60)
    do {
        Start-Sleep -Milliseconds 500
        $app.Refresh()
        if ($app.HasExited) { throw 'Probe exited before capture' }
        $ack = Get-Content -LiteralPath (Join-Path $output 'capture-stdout.txt') -Raw
    } until (($app.MainWindowHandle -ne [IntPtr]::Zero -and $ack -match 'LINTRA_RUNTIME_RENDER_ACK') -or [DateTime]::UtcNow -ge $deadline)
    if ($ack -notmatch 'LINTRA_RUNTIME_RENDER_ACK' -or $app.SessionId -ne $result.session) { throw 'Capture requires native acknowledgment in this desktop' }
    $window = $app.MainWindowHandle
    if (-not [LintraProfileConsole]::IsWindowVisible($window) -or $app.MainWindowTitle -ne 'Lintra · Verifica runtime') { throw 'Unexpected capture window' }
    [LintraProfileConsole]::SetForegroundWindow($window) | Out-Null
    Start-Sleep -Seconds 1
    if ([LintraProfileConsole]::GetForegroundWindow() -ne $window) { throw 'Probe does not own foreground' }
    $result.window_dpi = [LintraProfileConsole]::GetDpiForWindow($window)
    if ([Math]::Abs($result.window_dpi * 1000.0 / 96 - $native.native.scale_milli) -gt 2) { throw 'Win32 DPI and WebView scale differ' }
    $rect = [LintraProfileConsole+Rect]::new()
    if (-not [LintraProfileConsole]::GetWindowRect($window, [ref]$rect)) { throw 'Cannot read capture bounds' }
    $bitmap = [Drawing.Bitmap]::new($rect.Right - $rect.Left, $rect.Bottom - $rect.Top)
    $graphics = [Drawing.Graphics]::FromImage($bitmap)
    try {
        $graphics.CopyFromScreen([Drawing.Point]::new($rect.Left, $rect.Top), [Drawing.Point]::Empty, $bitmap.Size)
        $capture = Join-Path $output 'profile-window.png'
        $bitmap.Save($capture, [Drawing.Imaging.ImageFormat]::Png)
        $result.capture_sha256 = (Get-FileHash $capture -Algorithm SHA256).Hash.ToLowerInvariant()
    } finally { $graphics.Dispose(); $bitmap.Dispose() }
    if (-not $app.CloseMainWindow() -or -not $app.WaitForExit(15000) -or $app.ExitCode -ne 0) { throw 'Normal probe close failed' }
    $result.capture_exit = $app.ExitCode
    $result.passed = $true
} catch { $result.error = $_.Exception.Message }
finally {
    if ($null -ne $app -and -not $app.HasExited) { Stop-Process -Id $app.Id -Force -ErrorAction SilentlyContinue }
    if ($null -ne $node) {
        if (-not $node.HasExited) { $node.Kill() }
        $node.Dispose()
    }
    $result | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $output 'launcher-result.json') -Encoding utf8
}
if (-not $result.passed) { exit 1 }
