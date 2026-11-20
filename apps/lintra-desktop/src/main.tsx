import { createRoot } from "react-dom/client";
import { useEffect, useState } from "react";
import { appInfo, clientReady, errorText, type AppInfo } from "./ipc";
import { AppShell, type ServiceState } from "./AppShell";
import { INITIAL_THEME, INITIAL_VIEW, applyTheme, type ThemePreference, type ViewId } from "./shell";
import "./style.css";

function App() {
  const [info, setInfo] = useState<AppInfo | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [ready, setReady] = useState(false);
  const [view, setView] = useState<ViewId>(INITIAL_VIEW);
  const [theme, setTheme] = useState<ThemePreference>(INITIAL_THEME);
  useEffect(() => { applyTheme(document.documentElement, theme); }, [theme]);
  useEffect(() => {
    let active = true;
    void appInfo().then(value => { if (active) setInfo(value); })
      .catch(reason => { if (active) setError(errorText(reason)); });
    return () => { active = false; };
  }, []);
  useEffect(() => {
    if (!info) return;
    let active = true;
    // Acknowledge only after React commits and the local document has painted.
    void document.fonts.ready.then(() => new Promise<void>(resolve => {
      requestAnimationFrame(() => requestAnimationFrame(() => resolve()));
    })).then(async () => {
      if (!active) return;
      await clientReady();
      if (active) setReady(true);
    }).catch(reason => { if (active) setError(errorText(reason)); });
    return () => { active = false; };
  }, [info]);
  const service: ServiceState = error ? { kind: "error", message: error }
    : ready && info ? { kind: "ready", version: info.app_version } : { kind: "starting" };
  // No report exists until the P5 run views connect the Rust job lifecycle.
  return <AppShell view={view} onView={setView} theme={theme} onTheme={setTheme}
    service={service} reportAvailable={false} />;
}

const root = document.getElementById("root");
if (!root) throw new Error("Application root unavailable");
createRoot(root).render(<App />);
