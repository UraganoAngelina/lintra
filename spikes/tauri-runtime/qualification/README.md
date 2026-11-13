# Windows desktop probe (T-MIG-P1-02)

`windows-desktop-probe.ps1` is a task-specific acceptance script, not a recurring
gate. It runs the unchanged, hash-verified CI probe installed by NSIS under
`C:\Program Files\Lintra Runtime Spike`.

Run it with an `AtLogOn` scheduled task whose principal is the selected standard
user, `LogonType=Interactive`, `RunLevel=Limited`. The administrator prepares
the task; the script itself rejects elevated tokens, session 0, and sessions
other than the active console. Temporary guest autologon uses the signed
[Microsoft Sysinternals Autologon](https://learn.microsoft.com/en-us/sysinternals/downloads/autologon)
tool and its encrypted LSA secret, with dedicated random VM credentials kept
outside Git and logs. No plaintext Winlogon password is used. The initial
plaintext-registry approach was rejected by automatic approval review before
execution and replaced with this approved alternative. Restore the tested pre-test snapshot to
remove those credentials, tasks and guest configuration changes after collecting
evidence. Never apply this setup to the CachyOS host or a production PC.

The small `LintraDesktopProbe` interop class queries the real console session,
checks native window visibility and brings the probe to the foreground. It
does not synthesize a successful render. The existing Rust `render_ack` command
provides that signal only after React has received the Rust reply, awaited local
fonts and completed two animation frames. The script requires that signal and
a visible native window, captures the user's desktop, and records accessible
text where exposed by WebView2. Review the capture as part of this actual run.
The window is closed normally, then `--smoke-test` must produce another render
acknowledgment and exit zero. Bounded waits fail explicitly. A `finally` block
closes a remaining probe and writes `desktop-result.json` even on failure.

`Get-InstallManifest` hashes the installed tree before and after ordinary use;
any content change fails the check. WebView profile locations and account/session
identity are recorded separately for each user. Results and PNG captures live
under that user's `%LOCALAPPDATA%\LintraQualification\P1-02`.

The script also creates a **synthetic conservation sentinel**, using the already
installed Node `node:sqlite` module, under
`%LOCALAPPDATA%\Lintra\Qualification-P1-02`. This is a real SQLite file containing
only the synthetic test account name, plus a separate synthetic configuration.
Its hash allows subsequent installer/uninstaller tests to detect data loss.
The Node helper uses an explicitly owned .NET process, concurrent pipe reads,
a timeout and a checked exit code; this avoids PowerShell 5 native stderr and
short-lived process-handle races. `process.argv[2]` identifies the database
argument after the script filename, rather than opening the script as a DB.
It is deliberately not a Lintra product database: the isolated spike opens no
clinical input or product storage. These results do not qualify the future
React/Tauri persistence integration, backup, upgrade or product release gates.
No additional dependency, service or CI workflow is introduced.

NSIS exit status, HKLM metadata, offline isolation, runtime provisioning from
an absent WebView2 installation, account ACL separation, data conservation and
snapshot restoration are checked separately by the administrator and recorded
in the existing P1-02 dossier. An SSH exit code alone proves none of them.

`windows-account-isolation.ps1` accepts only the two named standard identities.
The administrator first confirms that both accounts' sentinels and WebView
profiles exist. Each account then reads its own SQLite hash and attempts real
file reads/directory enumeration under the other profile. Only an explicit
`UnauthorizedAccessException` satisfies denial; a missing file does not pass.
A `CreateNew` attempt on a dedicated filename checks that Program Files denies
writes without touching application files. An unexpectedly permitted file is
removed and the check fails. The result is written under the caller's own
profile and a failed check exits nonzero. This filesystem check can run through
standard SSH for one user and a limited desktop task for the other; it makes
no graphical claim of its own. Script execution policy is scoped only to the
individual guest PowerShell process.
