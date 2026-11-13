// Foreground native document/font/DPI measurement. It owns and joins the GUI
// child; Windows desktop/token checks belong to the task's desktop launcher.
const fs = require("node:fs");
const path = require("node:path");
const os = require("node:os");
const crypto = require("node:crypto");
const { spawn } = require("node:child_process");

async function main() {
  const exe = fs.realpathSync(process.argv[2]);
  const output = path.resolve(process.argv[3]);
  if (fs.existsSync(output)) throw new Error("Refusing to overwrite evidence");
  const temp = fs.mkdtempSync(path.join(os.tmpdir(), "lintra-profile-"));
  const env = { ...process.env };
  if (process.platform === "linux") {
    for (const [variable, dirname] of [["XDG_DATA_HOME", "data"], ["XDG_CONFIG_HOME", "config"], ["XDG_CACHE_HOME", "cache"]]) {
      const dir = path.join(temp, dirname); fs.mkdirSync(dir, { mode: 0o700 }); env[variable] = dir;
    }
    env.GDK_BACKEND = "wayland";
  }
  const result = { passed: false, started_utc: new Date().toISOString(),
    executable_sha256: crypto.createHash("sha256").update(fs.readFileSync(exe)).digest("hex"),
    platform: process.platform, architecture: os.arch(), os_release: os.release(), node: process.version };
  let child;
  try {
    child = spawn(exe, ["--smoke-test", "--profile-test"], { env, stdio: ["ignore", "pipe", "pipe"] });
    let stdout = "", stderr = "";
    child.stdout.on("data", (chunk) => { stdout += chunk.toString(); });
    child.stderr.on("data", (chunk) => { stderr += chunk.toString(); });
    const outcome = await new Promise((resolve, reject) => {
      const timer = setTimeout(() => child.kill(), 90000);
      child.once("error", (error) => { clearTimeout(timer); reject(error); });
      child.once("close", (code, signal) => { clearTimeout(timer); resolve({ code, signal }); });
    });
    Object.assign(result, { exit_code: outcome.code, signal: outcome.signal, stdout, stderr });
    const decode = (prefix) => stdout.split(/\r?\n/).filter((line) => line.startsWith(prefix)).map((line) => JSON.parse(line.slice(prefix.length)));
    const native = decode("LINTRA_RUNTIME_PROFILE_NATIVE ");
    const render = decode("LINTRA_RUNTIME_RENDER_ACK ");
    result.native = native[0]; result.render = render[0];
    // Tauri's pinned Linux/Windows manager defaults the WebView data directory
    // to app_local_data_dir. Check that a real profile was created there;
    // merely printing the resolver's path would not prove this.
    if (native.length === 1) {
      const root = native[0].app_local_data_root;
      const expected = path.join(process.platform === "linux" ? env.XDG_DATA_HOME : env.LOCALAPPDATA, "it.lintra.runtime-spike");
      result.profile_directory_matches = path.resolve(root) === path.resolve(expected);
      result.profile_directory_exists = fs.existsSync(root) && fs.statSync(root).isDirectory();
      if (result.profile_directory_matches && result.profile_directory_exists) {
        result.profile_top_level = fs.readdirSync(root, { withFileTypes: true })
          .map((entry) => ({ name: entry.name, directory: entry.isDirectory() }));
      }
    }
    result.passed = outcome.code === 0 && outcome.signal === null && native.length === 1 && render.length === 1
      && result.profile_directory_matches === true && result.profile_directory_exists === true
      && render[0].protocol === 1 && render[0].width >= 640 && render[0].height >= 480
      && render[0].profile && Math.abs(render[0].profile.scale_milli - native[0].scale_milli) <= 2
      && Object.entries(render[0].profile).filter(([key]) => !key.endsWith("_milli")).every(([, value]) => value === true);
  } catch (error) { result.error = String(error); }
  finally {
    if (child && child.exitCode === null && child.signalCode === null) child.kill();
    fs.rmSync(temp, { recursive: true, force: true });
    fs.writeFileSync(output, JSON.stringify(result, null, 2) + "\n", { mode: 0o600 });
  }
  process.stdout.write(JSON.stringify(result, null, 2) + "\n");
  process.exitCode = result.passed ? 0 : 1;
}
main().catch((error) => { console.error(String(error)); process.exitCode = 1; });
