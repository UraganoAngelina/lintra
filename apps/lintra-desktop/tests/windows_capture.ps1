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
# Usage: powershell -NoProfile -ExecutionPolicy Bypass -File windows_capture.ps1 <exe> <output-dir> [-Scale2]
#        ... windows_capture.ps1 <exe> <output-dir> -Flows <fixtures-dir>   (T-MIG-P5-08)
# Sizes are CSS pixels; the harness is per-monitor DPI aware and converts them
# with the window DPI, so the same scenarios hold at 100% and 200% scaling.
# -Scale2 runs the 200% set (requires a 192 DPI desktop, e.g. 1920x1080 at 200%).
param([Parameter(Mandatory)] [string] $Exe, [Parameter(Mandatory)] [string] $Output, [switch] $Scale2, [string] $Flows)
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
    [DllImport("user32.dll")] public static extern bool SetProcessDpiAwarenessContext(IntPtr value);
    [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr w, out uint pid);
}
'@
# Per-monitor v2 awareness: window, client and screen coordinates are physical.
if (-not [LintraShellWindow]::SetProcessDpiAwarenessContext([IntPtr]::new(-4))) { throw 'Cannot make the harness DPI aware' }

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
# 200% set on a 1920x1080 desktop: the work area is 960x492 logical, so a
# 640x480 CSS viewport plus the title bar cannot fit; every scenario uses the
# whole work area (below the 960 px breakpoint). Tab reaches every control.
$scaledScenarios = @(
    @('scale2-analisi-workarea', $null, @(), 'light'),
    @('scale2-analisi-system-dark-workarea', $null, @(), 'dark'),
    @('scale2-keyboard-skip-link-workarea', $null, @('{TAB}'), 'light'),
    @('scale2-keyboard-risultati-workarea', $null, @('{TAB}', '{TAB}', '{TAB}', '{ENTER}'), 'light'),
    @('scale2-keyboard-informazioni-workarea', $null, @('{TAB}', '{TAB}', '{TAB}', '{TAB}', '{ENTER}'), 'light'),
    @('scale2-keyboard-theme-dark-workarea', $null, @('{TAB}', '{TAB}', '{TAB}', '{TAB}', '{TAB}', '{DOWN}', '{DOWN}'), 'light'),
    @('scale2-keyboard-theme-light-on-dark-workarea', $null, @('{TAB}', '{TAB}', '{TAB}', '{TAB}', '{TAB}', '{DOWN}'), 'dark')
)
if ($Scale2) { $scenarios = $scaledScenarios }

# T-MIG-P5-08: P5-02..P5-10 flows on WebView2 with the real IFileOpenDialog.
# Fifth item: options. full = external synthetic Full bundle (S, T, unit,
# year) through LINTRA_SCHEMA_DIR; live = capture while a run is active (the
# indeterminate bar never gives two equal frames); budget = resource-budget.json
# written in %APPDATA%\it.lintra.desktop (Tauri app_config_dir) before launch.
# Steps: WAIT (a Rust round trip), DIALOG:<file> (typed in the owned dialog's
# file name box), TYPE:<text>, otherwise SendKeys. Tab order as on Linux:
# skip link, Analisi, Risultati, Informazioni, theme, flow, version, mode,
# trace (LocalOnly) or the per-trace buttons and parameters (Full).
$T = '{TAB}'
$select = @($T, $T, $T, $T, $T, $T, '{DOWN}', 'WAIT', $T, '{DOWN}', 'WAIT', $T, $T, '{DOWN}', 'WAIT')
$file = $select + @($T, '{ENTER}', 'DIALOG:partial.txt', 'WAIT')
$fileBig = $select + @($T, '{ENTER}', 'DIALOG:big.txt', 'WAIT')
$start = @($T, $T, $T, $T, '{ENTER}', 'WAIT', 'WAIT')
$full = @($T, $T, $T, $T, $T, $T, '{DOWN}', 'WAIT', $T, '{DOWN}', 'WAIT', $T, '{DOWN}', 'WAIT')
$fullFiles = $full + @($T, '{ENTER}', 'DIALOG:fulls.txt', 'WAIT', $T, '{ENTER}', 'DIALOG:fullt.txt', 'WAIT')
$fullBig = $full + @($T, '{ENTER}', 'DIALOG:big.txt', 'WAIT', $T, '{ENTER}', 'DIALOG:onet.txt', 'WAIT')
$params = @($T, 'TYPE:a', $T, 'WAIT', 'TYPE:2026', $T, 'WAIT', $T, $T, $T, '{ENTER}')
$budget = '{"version":1,"measurement_source":"misura sintetica","input_bytes":65536,"reference_bytes":null}'
$flowScenarios = @(
    @('analisi-workarea', $null, @(), 'light', @{}),
    @('analisi-dark-workarea', $null, @(), 'dark', @{}),
    @('analisi-640x480', @(640, 480), @(), 'light', @{}),
    @('selection-workarea', $null, $select, 'light', @{}),
    @('input-partial-workarea', $null, $file, 'light', @{}),
    @('input-partial-640x480', @(640, 480), $file, 'light', @{}),
    @('run-completed-workarea', $null, ($file + $start + @('WAIT')), 'light', @{}),
    @('budget-warning-workarea', $null, ($file + $start), 'light', @{ budget = $budget }),
    @('run-active-workarea', $null, ($fileBig + @($T, $T, $T, $T, '{ENTER}', 'WAIT', 'WAIT')), 'light', @{ live = $true }),
    @('run-cancelled-workarea', $null, ($fileBig + @($T, $T, $T, $T, '{ENTER}', 'WAIT', $T, '{ENTER}', 'WAIT', 'WAIT', 'WAIT')), 'light', @{}),
    @('informazioni-report-workarea', $null, ($file + $start + @('WAIT') + @('+{TAB}') * 10 + @('{ENTER}', 'WAIT')), 'light', @{}),
    @('run-completed-dark-workarea', $null, ($file + $start + @('WAIT')), 'dark', @{}),
    @('full-selection-workarea', $null, $full, 'light', @{ full = $true }),
    @('full-run-completed-workarea', $null, ($fullFiles + $params + @('WAIT', 'WAIT', 'WAIT')), 'light', @{ full = $true }),
    @('full-monitor-pass1-workarea', $null, ($fullBig + $params + @('WAIT')), 'light', @{ full = $true; live = $true }),
    @('full-run-cancelled-workarea', $null, ($fullBig + $params + @('WAIT', $T, '{ENTER}', 'WAIT', 'WAIT', 'WAIT')), 'light', @{ full = $true }),
    @('full-run-completed-640x480', @(640, 480), ($fullFiles + $params + @('WAIT', 'WAIT', 'WAIT')), 'light', @{ full = $true })
)
if ($Flows) {
    $Flows = (Resolve-Path -LiteralPath $Flows).Path
    $scenarios = $flowScenarios
}
$configDirectory = Join-Path $env:APPDATA 'it.lintra.desktop'
# One fresh TEMP per launch: WebView2 helpers of the previous instance can
# still hold files in theirs, so a shared folder cannot be emptied in between.
$runTempBase = Join-Path ([IO.Path]::GetTempPath()) 'lintra-p508-run-temp'
$runTempIndex = 0
$runTemp = $null

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

function Get-ClientFrame([IntPtr] $window) {
    # Physical client pixels: SHA-256 of the raw frame and whether it is uniform.
    $client = [LintraShellWindow+Rect]::new(); [LintraShellWindow]::GetClientRect($window, [ref]$client) | Out-Null
    $origin = [LintraShellWindow+Point]::new(); [LintraShellWindow]::ClientToScreen($window, [ref]$origin) | Out-Null
    $bitmap = [Drawing.Bitmap]::new([Math]::Max(1, $client.Right), [Math]::Max(1, $client.Bottom))
    $graphics = [Drawing.Graphics]::FromImage($bitmap)
    try {
        $graphics.CopyFromScreen([Drawing.Point]::new($origin.X, $origin.Y), [Drawing.Point]::Empty, $bitmap.Size)
        $stream = [IO.MemoryStream]::new()
        $bitmap.Save($stream, [Drawing.Imaging.ImageFormat]::Bmp)
        $hash = [BitConverter]::ToString([Security.Cryptography.SHA256]::Create().ComputeHash($stream.ToArray()))
        $first = $bitmap.GetPixel(0, 0); $uniform = $true
        for ($y = 0; $y -lt $bitmap.Height -and $uniform; $y += [Math]::Max(1, [int]($bitmap.Height / 24))) {
            for ($x = 0; $x -lt $bitmap.Width; $x += [Math]::Max(1, [int]($bitmap.Width / 24))) {
                if ($bitmap.GetPixel($x, $y) -ne $first) { $uniform = $false; break }
            }
        }
        return @{ hash = $hash; uniform = $uniform }
    } finally { $graphics.Dispose(); $bitmap.Dispose() }
}

function Wait-StableRender([IntPtr] $window, [int] $seconds) {
    # Software-rendered WebView2 at high DPI can take seconds to paint and to
    # follow a resize: wait for two equal, non-uniform frames one second apart.
    $deadline = [DateTime]::UtcNow.AddSeconds($seconds)
    $previous = $null
    do {
        Start-Sleep -Seconds 1
        $frame = Get-ClientFrame $window
        if (-not $frame.uniform -and $null -ne $previous -and $frame.hash -eq $previous) { return }
        $previous = $frame.hash
    } until ([DateTime]::UtcNow -ge $deadline)
    throw 'Rendering did not become stable'
}

function Invoke-Dialog($app, [IntPtr] $main, [string] $file) {
    # The IFileOpenDialog of the owned process takes the foreground with the
    # focus in its file name box; the full path is typed and confirmed.
    $deadline = [DateTime]::UtcNow.AddSeconds(15)
    do {
        Start-Sleep -Milliseconds 300
        $foreground = [LintraShellWindow]::GetForegroundWindow()
        $owner = [uint32]0; [LintraShellWindow]::GetWindowThreadProcessId($foreground, [ref]$owner) | Out-Null
    } until (($foreground -ne $main -and $owner -eq $app.Id) -or [DateTime]::UtcNow -ge $deadline)
    if ($foreground -eq $main -or $owner -ne $app.Id) { throw 'Native file dialog not in the foreground' }
    Start-Sleep -Milliseconds 800
    [Windows.Forms.SendKeys]::SendWait((Join-Path $Flows $file))
    Start-Sleep -Milliseconds 300
    [Windows.Forms.SendKeys]::SendWait('{ENTER}')
    $deadline = [DateTime]::UtcNow.AddSeconds(15)
    do { Start-Sleep -Milliseconds 300 } until ([LintraShellWindow]::GetForegroundWindow() -eq $main -or [DateTime]::UtcNow -ge $deadline)
    if ([LintraShellWindow]::GetForegroundWindow() -ne $main) { throw 'Native file dialog did not accept the file' }
}

function Set-RunEnvironment($options) {
    # Inherited by the next Start-Process only: Full bundle, private TEMP.
    if ($options.full) {
        $env:LINTRA_SCHEMA_DIR = Join-Path $Flows 'bundle\schemas'
        $env:LINTRA_REFERENCE_DATA_DIR = Join-Path $Flows 'bundle\reference-data'
    } else {
        Remove-Item Env:LINTRA_SCHEMA_DIR, Env:LINTRA_REFERENCE_DATA_DIR -ErrorAction SilentlyContinue
    }
    if ($Flows) {
        $script:runTempIndex++
        $script:runTemp = Join-Path $runTempBase $script:runTempIndex
        New-Item -ItemType Directory -Path $script:runTemp -Force | Out-Null
        $env:TEMP = $runTemp; $env:TMP = $runTemp
    }
    if (Test-Path -LiteralPath (Join-Path $configDirectory 'resource-budget.json')) { Remove-Item -LiteralPath (Join-Path $configDirectory 'resource-budget.json') }
    if ($options.budget) {
        New-Item -ItemType Directory -Path $configDirectory -Force | Out-Null
        [IO.File]::WriteAllText((Join-Path $configDirectory 'resource-budget.json'), $options.budget)
    }
}

function Invoke-Scenario($name, $size, $keys, $theme, $options = @{}) {
    $record = [ordered]@{ name = $name; keys = $keys; system_theme = $theme; options = $options; passed = $false }
    # A malformed scenario row (PowerShell array precedence) must not pass silently.
    if ($theme -isnot [string] -or $theme -notin @('light', 'dark') -or $options -isnot [hashtable]) { throw "Malformed scenario $name" }
    Set-ItemProperty -Path $themeKey -Name AppsUseLightTheme -Value ([int]($theme -eq 'light')) -Type DWord
    Set-RunEnvironment $options
    $app = Start-Process -FilePath $Exe -PassThru
    try {
        $deadline = [DateTime]::UtcNow.AddSeconds(30)
        do { Start-Sleep -Milliseconds 300; $app.Refresh() } until ($app.HasExited -or $app.MainWindowHandle -ne [IntPtr]::Zero -or [DateTime]::UtcNow -ge $deadline)
        if ($app.HasExited -or $app.MainWindowHandle -eq [IntPtr]::Zero) { throw 'Owned window not created' }
        $window = $app.MainWindowHandle
        do { Start-Sleep -Milliseconds 300; $app.Refresh() } until ($app.HasExited -or ([LintraShellWindow]::IsWindowVisible($window) -and $app.MainWindowTitle -eq 'Lintra') -or [DateTime]::UtcNow -ge $deadline)
        if (-not [LintraShellWindow]::IsWindowVisible($window) -or $app.MainWindowTitle -ne 'Lintra') { throw 'Unexpected window' }
        # The app window is created hidden and shown after its work-area fit:
        # the first handle seen can be another window of the process.
        $window = $app.MainWindowHandle
        if ($app.SessionId -ne $session) { throw 'Window outside this console session' }
        $area = [Windows.Forms.Screen]::PrimaryScreen.WorkingArea
        # Default placement after the adapter's startup fit, before any resize
        # by this harness: it must lie inside the work area (Tauri brief).
        Wait-StableRender $window 60
        $outer = [LintraShellWindow+Rect]::new(); $client = [LintraShellWindow+Rect]::new()
        [LintraShellWindow]::GetWindowRect($window, [ref]$outer) | Out-Null
        [LintraShellWindow]::GetClientRect($window, [ref]$client) | Out-Null
        # Placement chosen by Windows/Tauri before any resize by this harness.
        $record.default_window = @($outer.Left, $outer.Top, $outer.Right, $outer.Bottom)
        $record.default_window_inside_work_area = ($outer.Left -ge $area.Left -and $outer.Top -ge $area.Top -and $outer.Right -le $area.Right -and $outer.Bottom -le $area.Bottom)
        if (-not $record.default_window_inside_work_area) { throw 'Default window exceeds the work area' }
        $frameW = ($outer.Right - $outer.Left) - $client.Right
        $frameH = ($outer.Bottom - $outer.Top) - $client.Bottom
        $record.dpi = [LintraShellWindow]::GetDpiForWindow($window)
        $scale = $record.dpi / 96.0
        if ($Scale2 -and $record.dpi -ne 192) { throw "Expected 192 DPI, window has $($record.dpi)" }
        if ($null -eq $size) { $size = @([int][Math]::Floor(($area.Width - $frameW) / $scale), [int][Math]::Floor(($area.Height - $frameH) / $scale)) }
        $physical = @([int][Math]::Round($size[0] * $scale), [int][Math]::Round($size[1] * $scale))
        if ($physical[0] + $frameW -gt $area.Width -or $physical[1] + $frameH -gt $area.Height) { throw "Size $size does not fit the work area" }
        $record.work_area_physical = @($area.Width, $area.Height)
        # SWP_NOZORDER: resize/move only, keep activation to the process.
        [LintraShellWindow]::SetWindowPos($window, [IntPtr]::Zero, $area.Left, $area.Top, $physical[0] + $frameW, $physical[1] + $frameH, 0x0004) | Out-Null
        Wait-StableRender $window 60  # bootstrap IPC, local fonts, paint at the new size
        Assert-Foreground $window
        foreach ($key in $keys) {
            if ($key -eq 'WAIT') { Start-Sleep -Milliseconds 1500; continue }
            if ($key.StartsWith('DIALOG:')) { Invoke-Dialog $app $window $key.Substring(7); continue }
            Assert-Foreground $window
            [Windows.Forms.SendKeys]::SendWait($(if ($key.StartsWith('TYPE:')) { $key.Substring(5) } else { $key }))
            Start-Sleep -Milliseconds 400
        }
        if ($options.live) { Start-Sleep -Seconds 1 } else { Wait-StableRender $window 30 }
        Assert-Foreground $window
        [LintraShellWindow]::GetClientRect($window, [ref]$client) | Out-Null
        $origin = [LintraShellWindow+Point]::new()
        [LintraShellWindow]::ClientToScreen($window, [ref]$origin) | Out-Null
        $record.requested = $size
        $record.captured = @($client.Right, $client.Bottom)
        if ($record.captured[0] -ne $physical[0] -or $record.captured[1] -ne $physical[1]) { throw 'Client size differs from the request' }
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

function Invoke-CloseDuringRun {
    # Slint parity "Annulla/chiudi": a normal close while a 245 MB run is
    # active must cancel, join and exit 0, leaving no run spool in TEMP.
    $record = [ordered]@{ name = 'close-during-run'; passed = $false }
    Set-ItemProperty -Path $themeKey -Name AppsUseLightTheme -Value 1 -Type DWord
    Set-RunEnvironment @{}
    $app = Start-Process -FilePath $Exe -PassThru
    try {
        $deadline = [DateTime]::UtcNow.AddSeconds(30)
        do { Start-Sleep -Milliseconds 300; $app.Refresh() } until ($app.HasExited -or ($app.MainWindowHandle -ne [IntPtr]::Zero -and $app.MainWindowTitle -eq 'Lintra') -or [DateTime]::UtcNow -ge $deadline)
        $window = $app.MainWindowHandle
        Wait-StableRender $window 60
        foreach ($key in $fileBig + @($T, $T, $T, $T, '{ENTER}', 'WAIT', 'WAIT')) {
            if ($key -eq 'WAIT') { Start-Sleep -Milliseconds 1500; continue }
            if ($key.StartsWith('DIALOG:')) { Invoke-Dialog $app $window $key.Substring(7); continue }
            Assert-Foreground $window
            [Windows.Forms.SendKeys]::SendWait($key); Start-Sleep -Milliseconds 400
        }
        $record.spools_while_running = @(Get-ChildItem -LiteralPath $runTemp -Filter 'lintra-desktop-run-*' -Name)
        if ($record.spools_while_running.Count -eq 0) { throw 'The run was not active when closed' }
        $closed = [DateTime]::UtcNow
        if (-not $app.CloseMainWindow() -or -not $app.WaitForExit(60000)) { throw 'Close during the run did not end the process' }
        $record.seconds_to_exit = [Math]::Round(([DateTime]::UtcNow - $closed).TotalSeconds, 2)
        $record.exit_code = $app.ExitCode
        $record.spools_after_exit = @(Get-ChildItem -LiteralPath $runTemp -Filter 'lintra-desktop-run-*' -Name)
        if ($app.ExitCode -ne 0 -or $record.spools_after_exit.Count -ne 0) { throw 'Exit code or spool cleanup failed' }
        $record.passed = $true
    } catch { $record.error = $_.Exception.Message }
    finally { if (-not $app.HasExited) { Stop-Process -Id $app.Id -Force -ErrorAction SilentlyContinue } }
    return $record
}

try {
    if ($result.elevated -or $session -eq 0 -or $session -ne $result.console_session) {
        throw 'A real non-elevated console desktop is required'
    }
    foreach ($item in $scenarios) { $result.scenarios += Invoke-Scenario $item[0] $item[1] $item[2] $item[3] $(if ($item.Count -gt 4) { $item[4] } else { @{} }) }
    if ($Flows) { $result.close_during_run = Invoke-CloseDuringRun }
    # A key sequence must change the pixels of the untouched capture with the
    # same size and system theme.
    foreach ($record in $result.scenarios | Where-Object { $_.passed -and $_.keys.Count -gt 0 }) {
        $base = $result.scenarios | Where-Object { $_.passed -and $_.keys.Count -eq 0 -and $_.system_theme -eq $record.system_theme -and "$($_.captured)" -eq "$($record.captured)" } | Select-Object -First 1
        if ($null -eq $base -or $base.image_sha256 -eq $record.image_sha256) { $record.passed = $false; $record.error = 'No pixel change against the baseline' }
    }
    $result.passed = -not ($result.scenarios | Where-Object { -not $_.passed }) -and (-not $Flows -or $result.close_during_run.passed)
} catch { $result.error = $_.Exception.Message }
finally {
    Remove-Item Env:LINTRA_SCHEMA_DIR, Env:LINTRA_REFERENCE_DATA_DIR -ErrorAction SilentlyContinue
    Remove-Item -LiteralPath (Join-Path $configDirectory 'resource-budget.json') -ErrorAction SilentlyContinue
    Remove-Item -LiteralPath $runTempBase -Recurse -Force -ErrorAction SilentlyContinue
    if ($null -eq $originalTheme) { Remove-ItemProperty -Path $themeKey -Name AppsUseLightTheme -ErrorAction SilentlyContinue }
    else { Set-ItemProperty -Path $themeKey -Name AppsUseLightTheme -Value $originalTheme -Type DWord }
    $result | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath (Join-Path $Output 'receipt.json') -Encoding utf8
}
if (-not $result.passed) { exit 1 }
