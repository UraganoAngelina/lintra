# A targeted OS/installer check, not a claim about future product persistence.
# Administrator first confirms that both synthetic files actually exist, then
# each standard account must be denied access to the other account's files.
$ErrorActionPreference = 'Stop'
$identity = [Security.Principal.WindowsIdentity]::GetCurrent()
$principal = [Security.Principal.WindowsPrincipal]::new($identity)
$outputRoot = Join-Path $env:LOCALAPPDATA 'LintraQualification\P1-02'
$result = [ordered]@{ account = $identity.Name; passed = $false }
try {
    if ($principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) { throw 'Standard token required' }
    if ($env:USERNAME -eq 'lintra-agent') { $other = 'lintra-test2' }
    elseif ($env:USERNAME -eq 'lintra-test2') { $other = 'lintra-agent' }
    else { throw 'Unexpected test identity' }
    $own = Join-Path $env:LOCALAPPDATA 'Lintra\Qualification-P1-02\synthetic-conservation.sqlite'
    $result.own_sqlite_sha256 = (Get-FileHash -LiteralPath $own -Algorithm SHA256).Hash.ToLowerInvariant()
    $otherRoot = "C:\Users\$other\AppData\Local"
    foreach ($name in @('synthetic-conservation.sqlite', 'synthetic-config.txt')) {
        $denied = $false
        try { $null = [IO.File]::ReadAllBytes("$otherRoot\Lintra\Qualification-P1-02\$name") }
        catch [UnauthorizedAccessException] { $denied = $true }
        # Missing files and other errors are deliberately not treated as denial.
        $result[$name + '_other_read_denied'] = $denied
        if (-not $denied) { throw 'Other-account sentinel was readable' }
    }
    $profileDenied = $false
    try { $null = [IO.Directory]::GetFiles("$otherRoot\it.lintra.runtime-spike") }
    catch [UnauthorizedAccessException] { $profileDenied = $true }
    $result.other_webview_profile_denied = $profileDenied
    if (-not $profileDenied) { throw 'Other-account WebView profile was enumerable' }
    $writePath = 'C:\Program Files\Lintra Runtime Spike\Lintra-qualification-must-deny.tmp'
    $writeDenied = $false
    try {
        $stream = [IO.File]::Open($writePath, [IO.FileMode]::CreateNew, [IO.FileAccess]::Write)
        $stream.Dispose()
        Remove-Item -LiteralPath $writePath
    } catch [UnauthorizedAccessException] { $writeDenied = $true }
    $result.program_files_write_denied = $writeDenied
    if (-not $writeDenied) { throw 'Standard user could write into the installed application directory' }
    $result.passed = $true
} catch { $result.error = $_.Exception.Message }
$result | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath (Join-Path $outputRoot 'isolation-result.json') -Encoding utf8
if (-not $result.passed) { exit 1 }
