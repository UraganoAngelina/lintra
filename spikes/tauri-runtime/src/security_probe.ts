import { invoke } from "@tauri-apps/api/core";

declare global {
  interface Window {
    __LINTRA_SECURITY_TEST__?: true;
    __lintraProbeExecuted?: boolean;
  }
}

// Only the isolated --security-test process enables this synthetic fixture.
// React must display the exact text without creating an active element.
export const UNTRUSTED_SENTINEL = '<img src="http://127.0.0.1:49183/text" onerror="window.__lintraProbeExecuted=true"><script>window.__lintraProbeExecuted=true</script>';
const collector = "http://127.0.0.1:49183";
const pause = (ms: number) => new Promise<void>((resolve) => setTimeout(resolve, ms));

export interface SecurityReport {
  text_only: boolean;
  inline_script_blocked: boolean;
  remote_script_blocked: boolean;
  remote_style_blocked: boolean;
  remote_font_blocked: boolean;
  image_blocked: boolean;
  frame_blocked: boolean;
  object_blocked: boolean;
  fetch_blocked: boolean;
  websocket_blocked: boolean;
  eventsource_blocked: boolean;
  beacon_blocked: boolean;
  form_blocked: boolean;
  native_navigation_blocked: boolean;
  native_popup_blocked: boolean;
  valid_invoke: boolean;
  forbidden_invokes: boolean;
  invalid_reports: boolean;
  csp_events: number;
}

/** Exercise the actual document policy, not a copy in a browser test fixture.
 * A rejected load counts only when the runtime emits the expected CSP violation;
 * a missing API, a CORS error or an unreachable collector cannot supply a PASS.
 * Native guards independently count navigation/popup denials, and the external
 * harness independently requires zero HTTP requests at the loopback collector.
 */
export async function runSecurityProbe(): Promise<SecurityReport> {
  const events: SecurityPolicyViolationEvent[] = [];
  const listener = (event: SecurityPolicyViolationEvent) => events.push(event);
  document.addEventListener("securitypolicyviolation", listener);
  const rejected = async (command: string, args: Record<string, unknown>, reason: string) => {
    try {
      await Promise.race([invoke(command, args), pause(2000).then(() => { throw new Error("probe-timeout"); })]);
      return false;
    } catch (error) {
      const message = String(error);
      return !message.includes("probe-timeout") && message.includes(reason);
    }
  };
  const blocked = async (directive: string, trigger: () => void) => {
    const start = events.length;
    try { trigger(); } catch { /* A CSP exception alone is insufficient. */ }
    await pause(100);
    return events.slice(start).some((event) => event.effectiveDirective.startsWith(directive));
  };
  const dimensions = () => ({ protocol: 1, width: Math.round(innerWidth), height: Math.round(innerHeight) });
  try {
    let report = await invoke<SecurityReport | null>("render_ack", { report: { ...dimensions(), security_resume: true } });
    if (report === null) {
      const text = document.getElementById("probe-untrusted-sentinel");
      const text_only = text?.textContent === UNTRUSTED_SENTINEL
        && text.querySelector("img,script") === null && window.__lintraProbeExecuted !== true;
      const inline_script_blocked = await blocked("script-src", () => {
        const script = document.createElement("script");
        script.textContent = "window.__lintraProbeExecuted=true";
        document.body.append(script);
      }) && window.__lintraProbeExecuted !== true;
      const remote_script_blocked = await blocked("script-src", () => {
        const script = document.createElement("script"); script.src = `${collector}/script`; document.body.append(script);
      });
      const remote_style_blocked = await blocked("style-src", () => {
        const link = document.createElement("link"); link.rel = "stylesheet"; link.href = `${collector}/style`; document.head.append(link);
      });
      const remote_font_blocked = await blocked("font-src", () => {
        void new FontFace("LintraRejectedFont", `url("${collector}/font")`).load().catch(() => {});
      });
      const image_blocked = await blocked("img-src", () => {
        const image = document.createElement("img"); image.src = `${collector}/image`; document.body.append(image);
      });
      const frame_blocked = await blocked("frame-src", () => {
        const frame = document.createElement("iframe"); frame.src = `${collector}/frame`; document.body.append(frame);
      });
      const object_blocked = await blocked("object-src", () => {
        const object = document.createElement("object"); object.data = `${collector}/object`; document.body.append(object);
      });
      const fetch_blocked = await blocked("connect-src", () => { void fetch(`${collector}/fetch`).catch(() => {}); });
      let socket: WebSocket | undefined;
      const websocket_blocked = await blocked("connect-src", () => { socket = new WebSocket("ws://127.0.0.1:49183/socket"); });
      socket?.close();
      let source: EventSource | undefined;
      const eventsource_blocked = await blocked("connect-src", () => { source = new EventSource(`${collector}/events`); });
      source?.close();
      const beacon_blocked = await blocked("connect-src", () => { navigator.sendBeacon(`${collector}/beacon`, "synthetic"); });
      const form_blocked = await blocked("form-action", () => {
        const form = document.createElement("form"); form.action = `${collector}/form`; form.method = "post";
        document.body.append(form); form.submit();
      });
      const valid_invoke = await invoke<string>("runtime_probe") === "lintra-runtime-v1";
      const denied = await Promise.all([
        // Require the ACL denial, not any failure such as a nonexistent file or
        // executable. This message is shared by pinned debug/release Tauri ACLs.
        rejected("plugin:fs|read_file", { path: "synthetic-forbidden" }, "not allowed"),
        rejected("plugin:shell|execute", { program: "synthetic-forbidden" }, "not allowed"),
        rejected("plugin:window|create", { label: "forbidden" }, "not allowed"),
        rejected("plugin:webview|create_webview_window", { options: { label: "forbidden", url: `${collector}/invoke` } }, "not allowed"),
        rejected("unknown_probe_command", {}, "not allowed"),
      ]);
      const invalid = await Promise.all([
        rejected("render_ack", { report: { protocol: 2, width: 900, height: 600 } }, "invalid runtime render acknowledgment"),
        rejected("render_ack", { report: { protocol: 1, width: 0, height: 600 } }, "invalid runtime render acknowledgment"),
        rejected("render_ack", { report: { protocol: 1, width: 32769, height: 600 } }, "invalid runtime render acknowledgment"),
        rejected("render_ack", { report: { protocol: 1, width: "900", height: 600 } }, "invalid type"),
        rejected("render_ack", { report: { protocol: 1, width: 900, height: 600, extra: true } }, "unknown field"),
        rejected("render_ack", { report: { ...dimensions(), security_resume: "yes" } }, "invalid type"),
      ]);
      report = { text_only, inline_script_blocked, remote_script_blocked, remote_style_blocked,
        remote_font_blocked, image_blocked, frame_blocked, object_blocked, fetch_blocked,
        websocket_blocked, eventsource_blocked, beacon_blocked, form_blocked,
        native_navigation_blocked: false, native_popup_blocked: false, valid_invoke,
        forbidden_invokes: denied.every(Boolean), invalid_reports: invalid.every(Boolean), csp_events: events.length };
      await invoke("render_ack", { report: { ...dimensions(), security: report } });
    }
    if (!report.native_popup_blocked) {
      const button = document.getElementById("probe-popup-attempt");
      if (button === null) throw new Error("missing synthetic popup button");
      const trustedPopup = new Promise<Window | null>((resolve) => {
        button.addEventListener("click", (event) => {
          if (event.isTrusted) resolve(window.open(`${collector}/popup`, "_blank"));
        }, { once: true });
      });
      button.focus();
      const ready = await rejected("render_ack", { report: dimensions() }, "security report required");
      if (!ready) throw new Error("popup handshake failed");
      const popup = await Promise.race([trustedPopup, pause(30000).then(() => { throw new Error("trusted popup input timeout"); })]);
      await pause(200);
      report.native_popup_blocked = popup === null || popup.closed;
    }
    if (!report.native_navigation_blocked) {
      const before = location.href;
      location.assign(`${collector}/navigation`);
      await pause(200);
      report.native_navigation_blocked = location.href === before;
    }
    // A recovered document must render the same sentinel and use the live IPC.
    // Rust accepts resume only at its exact trusted URL and supplies the two
    // separately witnessed native denial flags. No DOM persistence is used.
    const currentText = document.getElementById("probe-untrusted-sentinel");
    report.text_only &&= currentText?.textContent === UNTRUSTED_SENTINEL
      && currentText.querySelector("img,script") === null && window.__lintraProbeExecuted !== true;
    report.valid_invoke &&= await invoke<string>("runtime_probe") === "lintra-runtime-v1";
    return report;
  } finally {
    document.removeEventListener("securitypolicyviolation", listener);
  }
}
