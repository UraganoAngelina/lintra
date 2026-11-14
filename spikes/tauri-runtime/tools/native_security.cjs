// One foreground, dependency-free harness owns and joins both the native GUI
// child and a loopback-only collector. The reachable collector distinguishes
// CSP/native rejection from a load which merely fails because Internet is off.
const fs = require("node:fs");
const path = require("node:path");
const os = require("node:os");
const http = require("node:http");
const crypto = require("node:crypto");
const { spawn, spawnSync } = require("node:child_process");

function checked(command, args, timeout = 20000) {
  const result = spawnSync(command, args, { encoding: "utf8", timeout, windowsHide: true });
  if (result.error || result.status !== 0) {
    const stages = (result.stdout || "").split(/\r?\n/).filter(line => /^LINTRA_INPUT_STAGE (init|assembly|win32|handle|activate|focus|sent)$/.test(line));
    throw new Error("Owned-window input failed: " + (result.error || result.stderr) + (stages.length ? "; " + stages.join(", ") : ""));
  }
  return result.stdout;
}

function trustedPopupInput(pid) {
  if (!Number.isSafeInteger(pid) || pid <= 0) throw new Error("Invalid native child PID");
  if (process.platform === "linux") {
    const windows = JSON.parse(checked("hyprctl", ["-j", "clients"])).filter((window) => window.pid === pid);
    if (windows.length !== 1 || !/^0x[0-9a-fA-F]+$/.test(windows[0].address)) throw new Error("Expected one owned Hyprland window");
    const target = JSON.stringify("address:" + windows[0].address);
    checked("hyprctl", ["dispatch", `hl.dsp.focus({window = ${target}})`]);
    if (JSON.parse(checked("hyprctl", ["-j", "activewindow"])).pid !== pid) throw new Error("Native child does not own focus");
    checked("hyprctl", ["dispatch", `hl.dsp.send_shortcut({mods = "", key = "Return", window = ${target}})`]);
  } else if (process.platform === "win32") {
    const script = `$ErrorActionPreference='Stop'; [Console]::Out.WriteLine('LINTRA_INPUT_STAGE init'); [Console]::Out.Flush(); Add-Type -AssemblyName System.Windows.Forms; [Console]::Out.WriteLine('LINTRA_INPUT_STAGE assembly'); [Console]::Out.Flush();
Add-Type @'
using System; using System.Runtime.InteropServices;
public static class LintraOwnedInput {
 [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
 [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
 [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h, out uint p);
}
'@
[Console]::Out.WriteLine('LINTRA_INPUT_STAGE win32'); [Console]::Out.Flush();
$p=Get-Process -Id ${pid}; $handle=$p.MainWindowHandle; if($handle -eq [IntPtr]::Zero){throw 'No owned native window'};
[Console]::Out.WriteLine('LINTRA_INPUT_STAGE handle'); [Console]::Out.Flush();
[LintraOwnedInput]::SetForegroundWindow($handle) | Out-Null; [Console]::Out.WriteLine('LINTRA_INPUT_STAGE activate'); [Console]::Out.Flush(); Start-Sleep -Milliseconds 200;
[uint32]$owner=0; [LintraOwnedInput]::GetWindowThreadProcessId([LintraOwnedInput]::GetForegroundWindow(),[ref]$owner) | Out-Null;
if($owner -ne ${pid}){throw 'Refuse input to another process'}; [Console]::Out.WriteLine('LINTRA_INPUT_STAGE focus'); [Console]::Out.Flush(); [Windows.Forms.SendKeys]::SendWait('{ENTER}'); [Console]::Out.WriteLine('LINTRA_INPUT_STAGE sent'); [Console]::Out.Flush();`;
    checked("powershell.exe", ["-NoLogo", "-NoProfile", "-NonInteractive", "-EncodedCommand", Buffer.from(script, "utf16le").toString("base64")], 60000);
  } else throw new Error("Unsupported native input platform");
}

async function main() {
  const exe = fs.realpathSync(process.argv[2]);
  const output = path.resolve(process.argv[3]);
  if (fs.existsSync(output)) throw new Error("Refusing to overwrite existing security evidence");
  const temp = fs.mkdtempSync(path.join(os.tmpdir(), "lintra-security-"));
  let measuring = false, connections = 0, requests = 0;
  const measuredSockets = [];
  const connectionDiagnostics = [];
  const requestCases = [];
  const server = http.createServer((request, response) => {
    if (measuring) {
      requests++;
      const known = ["/text", "/script", "/style", "/font", "/image", "/frame", "/object", "/fetch", "/socket", "/events", "/beacon", "/form", "/popup", "/navigation", "/invoke"];
      requestCases.push({ method: request.method, test_case: known.includes(request.url) ? request.url : "<unexpected>" });
    }
    response.writeHead(204, { "Connection": "close" }); response.end();
  });
  server.on("connection", (socket) => {
    if (measuring) {
      connections++;
      measuredSockets.push(socket);
      const entry = { accepted_utc: new Date().toISOString(), peer_port: socket.remotePort };
      if (process.platform === "win32") {
        const rows = spawnSync("netstat.exe", ["-ano", "-p", "tcp"], { encoding: "utf8", timeout: 3000 });
        entry.loopback_owner_rows = (rows.stdout || "").split(/\r?\n/).filter((line) => line.includes("127.0.0.1:49183") && line.includes(":" + socket.remotePort));
      }
      connectionDiagnostics.push(entry);
    }
  });
  let child;
  const result = { passed: false, executable_sha256: crypto.createHash("sha256").update(fs.readFileSync(exe)).digest("hex"),
    platform: process.platform, node: process.version, started_utc: new Date().toISOString() };
  const stopChild = () => {
    if (!child) return;
    if (Number.isSafeInteger(child.pid) && child.exitCode === null && child.signalCode === null) {
      if (process.platform === "win32") {
        // Kill only this owned process tree; orphan WebView pipes otherwise
        // keep the foreground harness alive after a timed-out Rust child.
        const stopped = spawnSync("taskkill.exe", ["/PID", String(child.pid), "/T", "/F"], { encoding: "utf8", timeout: 10000 });
        if (stopped.error || stopped.status !== 0) result.termination_error = String(stopped.error || stopped.stderr);
      }
      child.kill();
    }
    child.stdout?.destroy(); child.stderr?.destroy();
  };
  try {
    await new Promise((resolve, reject) => {
      server.once("error", reject); server.listen(49183, "127.0.0.1", resolve);
    });
    await new Promise((resolve, reject) => {
      http.get("http://127.0.0.1:49183/health", { agent: false }, (response) => {
        response.resume(); response.once("end", resolve);
      }).once("error", reject);
    });
    result.collector_health = "reachable before native launch";
    const env = { ...process.env };
    // Linux test data are private and ephemeral. Windows uses the standard
    // test account's actual profile, which is checked by the desktop launcher.
    for (const [variable, dirname] of [["XDG_DATA_HOME", "data"], ["XDG_CONFIG_HOME", "config"], ["XDG_CACHE_HOME", "cache"]]) {
      const folder = path.join(temp, dirname); fs.mkdirSync(folder, { mode: 0o700 }); env[variable] = folder;
    }
    if (process.platform === "linux") env.GDK_BACKEND = "wayland";
    measuring = true;
    child = spawn(exe, ["--security-test"], { env, stdio: ["ignore", "pipe", "pipe"] });
    result.native_pid = child.pid;
    let stdout = "", stderr = "";
    let trustedInput = false;
    child.stdout.on("data", (chunk) => {
      stdout += chunk.toString();
      if (!trustedInput && stdout.includes("LINTRA_RUNTIME_SECURITY_AWAIT_POPUP")) {
        trustedInput = true;
        try { trustedPopupInput(child.pid); result.trusted_popup_input = "sent only to the focused owned child"; }
        catch (error) { result.input_error = String(error); stopChild(); }
      }
    });
    child.stderr.on("data", (chunk) => { stderr += chunk.toString(); });
    const outcome = await new Promise((resolve, reject) => {
      const timer = setTimeout(() => { result.timed_out = true; stopChild(); }, 150000);
      child.once("error", (error) => { clearTimeout(timer); reject(error); });
      child.once("close", (code, signal) => { clearTimeout(timer); resolve({ code, signal }); });
    });
    // Allow queued collector callbacks to run before closing measurement.
    await new Promise((resolve) => setTimeout(resolve, 200));
    measuring = false;
    Object.assign(result, { exit_code: outcome.code, signal: outcome.signal, stdout, stderr,
      collector_connections: connections, collector_requests: requests,
      collector_socket_bytes: measuredSockets.map((socket) => ({ read: socket.bytesRead, written: socket.bytesWritten })), connection_diagnostics: connectionDiagnostics, request_cases: requestCases });
    const lines = stdout.split(/\r?\n/);
    const decode = (prefix) => lines.filter((line) => line.startsWith(prefix)).map((line) => JSON.parse(line.slice(prefix.length)));
    const reports = decode("LINTRA_RUNTIME_SECURITY_REPORT ");
    const guards = decode("LINTRA_RUNTIME_SECURITY_ACK ");
    const renders = decode("LINTRA_RUNTIME_RENDER_ACK ");
    result.report = reports[0]; result.native_guards = guards[0];
    result.passed = !result.timed_out && !result.termination_error && outcome.code === 0 && outcome.signal === null && requests === 0 && connections === 0 && trustedInput && !result.input_error
      && reports.length === 1 && guards.length === 1 && renders.length === 1
      && Object.entries(reports[0]).every(([name, value]) => name === "csp_events" ? value >= 10 && value <= 128 : value === true)
      && guards[0].navigation_denied >= 1 && guards[0].new_window_denied >= 1
      && renders[0].protocol === 1 && renders[0].width >= 640 && renders[0].height >= 480;
  } catch (error) { result.error = String(error); }
  finally {
    stopChild();
    // closeAllConnections also bounds cleanup if an unexpected forbidden
    // connection was opened and kept alive. Such a connection already fails.
    server.closeAllConnections();
    await new Promise((resolve) => server.close(resolve));
    fs.rmSync(temp, { recursive: true, force: true });
    fs.writeFileSync(output, JSON.stringify(result, null, 2) + "\n", { mode: 0o600 });
  }
  process.stdout.write(JSON.stringify(result, null, 2) + "\n");
  process.exitCode = result.passed ? 0 : 1;
}
main().catch((error) => { console.error(String(error)); process.exitCode = 1; });
