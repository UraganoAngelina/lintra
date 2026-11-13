# Runs only inside a standard user's real interactive desktop. The installed,
# unmodified CI probe supplies its own React -> Rust render acknowledgment.
# No clinical inputs, product database, password or key is read by this script.
$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'
$resultRoot = Join-Path $env:LOCALAPPDATA 'LintraQualification\P1-02'
New-Item -ItemType Directory -Path $resultRoot -Force | Out-Null
$result = [ordered]@{ passed = $false; account = [Security.Principal.WindowsIdentity]::GetCurrent().Name }
$app = $null
try {
    Add-Type -AssemblyName System.Windows.Forms
    Add-Type -AssemblyName System.Drawing
    Add-Type -AssemblyName UIAutomationClient
    Add-Type -AssemblyName UIAutomationTypes
    Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class LintraDesktopProbe {
    [DllImport("kernel32.dll")] public static extern uint WTSGetActiveConsoleSessionId();
    [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr hWnd);
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr hWnd);
}
'@
    $principal = [Security.Principal.WindowsPrincipal]::new([Security.Principal.WindowsIdentity]::GetCurrent())
    $result.elevated = $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
    $result.session = [Diagnostics.Process]::GetCurrentProcess().SessionId
    $result.console_session = [LintraDesktopProbe]::WTSGetActiveConsoleSessionId()
    $result.local_app_data = $env:LOCALAPPDATA
    $result.user_profile = $env:USERPROFILE
    if ($result.elevated -or $result.session -eq 0 -or $result.session -ne $result.console_session) {
        throw 'A non-elevated active console desktop is required; SSH session 0 is insufficient'
    }
    # Allow first-logon Explorer/profile initialization to finish before capture.
    Start-Sleep -Seconds 20
    # These explicitly synthetic files exercise installer conservation of the
    # future per-account product directory. They are not a product DB or proof
    # that the isolated runtime spike implements persistence.
    $sentinelRoot = Join-Path $env:LOCALAPPDATA 'Lintra\Qualification-P1-02'
    New-Item -ItemType Directory -Path $sentinelRoot -Force | Out-Null
    $sqlite = Join-Path $sentinelRoot 'synthetic-conservation.sqlite'
    $nodeCode = 'const {DatabaseSync}=require("node:sqlite");const db=new DatabaseSync(process.argv[2]);db.exec("CREATE TABLE IF NOT EXISTS qualification_sentinel(account TEXT PRIMARY KEY)");db.prepare("INSERT OR IGNORE INTO qualification_sentinel VALUES (?)").run(process.env.USERNAME);console.log(JSON.stringify(db.prepare("SELECT account FROM qualification_sentinel").all()));db.close();'
    $nodeScript = Join-Path $resultRoot 'synthetic-sentinel.cjs'
    Set-Content -LiteralPath $nodeScript -Value $nodeCode -Encoding utf8
    # Own the Process handle before starting: PowerShell 5 Start-Process can
    # otherwise lose ExitCode for a short-lived native child. Read both pipes
    # asynchronously so neither pipe can block a child during the bounded wait.
    $node = [Diagnostics.Process]::new()
    $node.StartInfo.FileName = 'C:\Program Files\nodejs\node.exe'
    $node.StartInfo.Arguments = '--no-warnings "' + $nodeScript + '" "' + $sqlite + '"'
    $node.StartInfo.UseShellExecute = $false
    $node.StartInfo.CreateNoWindow = $true
    $node.StartInfo.RedirectStandardOutput = $true
    $node.StartInfo.RedirectStandardError = $true
    if (-not $node.Start()) { throw 'Cannot start SQLite sentinel helper' }
    $nodeOut = $node.StandardOutput.ReadToEndAsync()
    $nodeErr = $node.StandardError.ReadToEndAsync()
    if (-not $node.WaitForExit(30000)) { $node.Kill(); throw 'SQLite sentinel helper timeout' }
    $result.sqlite_helper_exit = $node.ExitCode
    $result.sqlite_sentinel_rows = $nodeOut.Result.Trim()
    Set-Content -LiteralPath (Join-Path $resultRoot 'sqlite-stderr.txt') -Value $nodeErr.Result -Encoding utf8
    if ($node.ExitCode -ne 0) { throw 'Cannot create/read synthetic SQLite conservation sentinel' }
    $node.Dispose()
    Set-Content -LiteralPath (Join-Path $sentinelRoot 'synthetic-config.txt') -Value $env:USERNAME -Encoding ascii
    $result.synthetic_sqlite_path = $sqlite
    $result.synthetic_sqlite_sha256 = (Get-FileHash -LiteralPath $sqlite -Algorithm SHA256).Hash.ToLowerInvariant()
    $exe = 'C:\Program Files\Lintra Runtime Spike\lintra-runtime-spike.exe'
    if (-not (Test-Path -LiteralPath $exe)) { throw 'Machine-installed probe missing' }
    $result.executable_sha256 = (Get-FileHash -LiteralPath $exe -Algorithm SHA256).Hash.ToLowerInvariant()
    $installRoot = Split-Path -Parent $exe
    function Get-InstallManifest {
        @(Get-ChildItem -LiteralPath $installRoot -File -Recurse | Sort-Object FullName | ForEach-Object {
            $_.FullName.Substring($installRoot.Length) + ':' + (Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash
        })
    }
    $before = Get-InstallManifest
    $stdout = Join-Path $resultRoot 'interactive-stdout.txt'
    $stderr = Join-Path $resultRoot 'interactive-stderr.txt'
    $app = Start-Process -FilePath $exe -PassThru -RedirectStandardOutput $stdout -RedirectStandardError $stderr
    $null = $app.Handle
    $deadline = [DateTime]::UtcNow.AddSeconds(90)
    do {
        Start-Sleep -Seconds 1
        $app.Refresh()
        $ack = if (Test-Path -LiteralPath $stdout) { Get-Content -LiteralPath $stdout -Raw } else { '' }
        if ($app.HasExited) { throw "Probe exited before capture: $($app.ExitCode)" }
    } until (($app.MainWindowHandle -ne [IntPtr]::Zero -and $ack -match 'LINTRA_RUNTIME_RENDER_ACK') -or [DateTime]::UtcNow -ge $deadline)
    if ($ack -notmatch 'LINTRA_RUNTIME_RENDER_ACK') { throw 'No real React/Rust render acknowledgment' }
    $result.render_ack = $ack.Trim()
    $result.window_title = $app.MainWindowTitle
    $result.app_session = $app.SessionId
    $result.window_visible = [LintraDesktopProbe]::IsWindowVisible($app.MainWindowHandle)
    if (-not $result.window_visible -or $result.app_session -ne $result.session) { throw 'Probe window not visible in this desktop' }
    [LintraDesktopProbe]::SetForegroundWindow($app.MainWindowHandle) | Out-Null
    Start-Sleep -Seconds 2
    # UI Automation complements the IPC acknowledgment with actual rendered text.
    # Some WebView builds omit the accessibility subtree until it is requested;
    # capture all exposed names, while retaining the independent native screenshot.
    try {
        $root = [Windows.Automation.AutomationElement]::FromHandle($app.MainWindowHandle)
        $nodes = $root.FindAll([Windows.Automation.TreeScope]::Descendants, [Windows.Automation.Condition]::TrueCondition)
        $result.accessible_names = @($nodes | ForEach-Object { $_.Current.Name } | Where-Object { $_ } | Select-Object -Unique)
    } catch { $result.accessibility_error = $_.Exception.Message }
    $bounds = [Windows.Forms.Screen]::PrimaryScreen.Bounds
    $bitmap = [Drawing.Bitmap]::new($bounds.Width, $bounds.Height)
    $graphics = [Drawing.Graphics]::FromImage($bitmap)
    try {
        $graphics.CopyFromScreen($bounds.Location, [Drawing.Point]::Empty, $bounds.Size)
        $bitmap.Save((Join-Path $resultRoot 'interactive-desktop.png'), [Drawing.Imaging.ImageFormat]::Png)
    } finally { $graphics.Dispose(); $bitmap.Dispose() }
    $result.screen = @{ width = $bounds.Width; height = $bounds.Height }
    if (-not $app.CloseMainWindow()) { throw 'Normal window-close request failed' }
    if (-not $app.WaitForExit(30000)) { throw 'Probe did not close normally' }
    $result.interactive_exit = $app.ExitCode
    if ($app.ExitCode -ne 0) { throw 'Interactive probe exited with an error' }
    # A second execution proves smoke-mode termination only after real render.
    $smokeOut = Join-Path $resultRoot 'smoke-stdout.txt'
    $app = Start-Process -FilePath $exe -ArgumentList '--smoke-test' -PassThru -RedirectStandardOutput $smokeOut -RedirectStandardError (Join-Path $resultRoot 'smoke-stderr.txt')
    $null = $app.Handle
    if (-not $app.WaitForExit(90000)) { throw 'Desktop smoke timed out' }
    $result.smoke_exit = $app.ExitCode
    $result.smoke_ack = (Get-Content -LiteralPath $smokeOut -Raw).Trim()
    if ($app.ExitCode -ne 0 -or $result.smoke_ack -notmatch 'LINTRA_RUNTIME_RENDER_ACK') { throw 'Smoke lacks successful render and exit' }
    $after = Get-InstallManifest
    $result.program_files_unchanged = ($before -join "`n") -ceq ($after -join "`n")
    if (-not $result.program_files_unchanged) { throw 'Ordinary use changed Program Files' }
    $dataRoot = Join-Path $env:LOCALAPPDATA 'it.lintra.runtime-spike'
    $result.webview_profile_root = $dataRoot
    $result.webview_profile_created = Test-Path -LiteralPath $dataRoot
    $result.passed = $true
} catch {
    $result.error = $_.Exception.Message
} finally {
    if ($null -ne $app -and -not $app.HasExited) { Stop-Process -Id $app.Id -Force -ErrorAction SilentlyContinue }
    $result | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath (Join-Path $resultRoot 'desktop-result.json') -Encoding utf8
}
if (-not $result.passed) { exit 1 }
