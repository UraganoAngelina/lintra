# Task P1-04 only. An Interactive/Limited scheduled task runs this in the
# standard account's real console; SSH merely prepares and reads the evidence.
$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'
$root = Join-Path $env:USERPROFILE 'LintraQualification\P1-04\source\spikes\tauri-runtime'
$output = Join-Path $env:LOCALAPPDATA 'LintraQualification\P1-04'
New-Item -ItemType Directory -Path $output -Force | Out-Null
$result = [ordered]@{ passed = $false; account = [Security.Principal.WindowsIdentity]::GetCurrent().Name }
try {
    Add-Type @'
using System.Runtime.InteropServices;
public static class LintraSecurityConsole {
    [DllImport("kernel32.dll")] public static extern uint WTSGetActiveConsoleSessionId();
}
'@
    $principal = [Security.Principal.WindowsPrincipal]::new([Security.Principal.WindowsIdentity]::GetCurrent())
    $result.elevated = $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
    $result.session = [Diagnostics.Process]::GetCurrentProcess().SessionId
    $result.console_session = [LintraSecurityConsole]::WTSGetActiveConsoleSessionId()
    if ($result.elevated -or $result.session -eq 0 -or $result.session -ne $result.console_session) { throw 'Real standard console desktop required' }
    Start-Sleep -Seconds 15
    $exe = Join-Path $root 'src-tauri\target\x86_64-pc-windows-msvc\release\lintra-runtime-spike.exe'
    $normalOut = Join-Path $output 'ordinary-stdout.txt'
    $ordinary = Start-Process -FilePath $exe -ArgumentList '--smoke-test' -PassThru -RedirectStandardOutput $normalOut -RedirectStandardError (Join-Path $output 'ordinary-stderr.txt')
    $null = $ordinary.Handle
    if (-not $ordinary.WaitForExit(90000)) { Stop-Process -Id $ordinary.Id -Force; throw 'Ordinary Windows smoke timeout' }
    $result.ordinary_exit = $ordinary.ExitCode
    $result.ordinary_ack = [string](Get-Content -LiteralPath $normalOut -Raw)
    if ($ordinary.ExitCode -ne 0 -or $result.ordinary_ack -notmatch 'LINTRA_RUNTIME_RENDER_ACK') { throw 'Ordinary smoke lacks render/exit0' }
    # Keep the .NET process handle from before Start, and drain both pipes
    # asynchronously. No PowerShell short-lived ExitCode race or pipe deadlock.
    $node = [Diagnostics.Process]::new()
    $node.StartInfo.FileName = 'C:\Program Files\nodejs\node.exe'
    $harness = Join-Path $root 'tools\native_security.cjs'
    $evidence = Join-Path $output 'native-security.json'
    $node.StartInfo.Arguments = '"' + $harness + '" "' + $exe + '" "' + $evidence + '"'
    $node.StartInfo.UseShellExecute = $false
    $node.StartInfo.CreateNoWindow = $true
    $node.StartInfo.RedirectStandardOutput = $true
    $node.StartInfo.RedirectStandardError = $true
    if (-not $node.Start()) { throw 'Cannot start owned native security harness' }
    $stdout = $node.StandardOutput.ReadToEndAsync()
    $stderr = $node.StandardError.ReadToEndAsync()
    if (-not $node.WaitForExit(180000)) { $node.Kill(); throw 'Native security harness timeout' }
    $result.harness_exit = $node.ExitCode
    Set-Content -LiteralPath (Join-Path $output 'harness-stdout.txt') -Value $stdout.Result -Encoding utf8
    Set-Content -LiteralPath (Join-Path $output 'harness-stderr.txt') -Value $stderr.Result -Encoding utf8
    $native = Get-Content -LiteralPath $evidence -Raw | ConvertFrom-Json
    $result.native = $native
    if ($node.ExitCode -ne 0 -or -not $native.passed) { throw 'Native Windows CSP/guard/ACL checks failed' }
    $node.Dispose()
    $result.passed = $true
} catch { $result.error = $_.Exception.Message }
$result | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $output 'launcher-result.json') -Encoding utf8
if (-not $result.passed) { exit 1 }
