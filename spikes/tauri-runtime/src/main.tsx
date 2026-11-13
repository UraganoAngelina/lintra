import { useEffect, useState } from "react";
import { createRoot } from "react-dom/client";
import { invoke } from "@tauri-apps/api/core";
import "./style.css";
import { runSecurityProbe, UNTRUSTED_SENTINEL } from "./security_probe";
import { measureProfile } from "./profile_probe";

declare global { interface Window { __LINTRA_PROFILE_TEST__?: true; } }

type ProbeState = "pending" | "ready" | "failed";

// This isolated probe has no validation data. The constant reply demonstrates
// an actual WebView → Rust → React round trip, rather than a simulated service.
function RuntimeProbe() {
  const [state, setState] = useState<ProbeState>("pending");
  const [dark, setDark] = useState(false);

  useEffect(() => {
    let active = true;
    void invoke<string>("runtime_probe").then(
      (reply) => {
        if (active) setState(reply === "lintra-runtime-v1" ? "ready" : "failed");
      },
      () => { if (active) setState("failed"); },
    );
    return () => { active = false; };
  }, []);

  useEffect(() => {
    if (state !== "ready") return;
    // Wait for two animation frames and local fonts before acknowledging the
    // render. This is a runtime smoke signal, not a pixel-parity assertion.
    let active = true;
    void document.fonts.ready.then(() => {
      requestAnimationFrame(() => requestAnimationFrame(() => {
        if (!active) return;
        void (async () => {
          const security = window.__LINTRA_SECURITY_TEST__ === true ? await runSecurityProbe() : undefined;
          if (!active) return;
          await invoke("render_ack", { report: {
            protocol: 1,
            width: Math.round(window.innerWidth),
            height: Math.round(window.innerHeight),
            ...(security === undefined ? {} : { security }),
            ...(window.__LINTRA_PROFILE_TEST__ === true ? { profile: measureProfile() } : {}),
          } });
        })().catch(() => { if (active) setState("failed"); });
      }));
    });
    return () => { active = false; };
  }, [state]);

  return (
    <main data-theme={dark ? "dark" : "light"}>
      <header>
        <strong>Lintra</strong>
        <button type="button" aria-pressed={dark} onClick={() => setDark(!dark)}>
          Tema scuro
        </button>
      </header>
      <section aria-labelledby="title">
        <h1 id="title">Verifica runtime desktop</h1>
        <p>Prova tecnica React e Tauri 2. Nessun file di lavoro viene aperto.</p>
        {window.__LINTRA_SECURITY_TEST__ === true && <p id="probe-untrusted-sentinel">{UNTRUSTED_SENTINEL}</p>}
        {window.__LINTRA_SECURITY_TEST__ === true && <button type="button" id="probe-popup-attempt">Verifica popup sintetico</button>}
        <dl>
          <div><dt>Interfaccia</dt><dd>React · TypeScript strict</dd></div>
          <div><dt>Contenuti</dt><dd>Inclusi nell’applicazione</dd></div>
          <div><dt>Servizio</dt><dd role="status" aria-live="polite">
            {state === "pending" && "Verifica del collegamento Rust…"}
            {state === "ready" && "Collegamento Rust verificato"}
            {state === "failed" && "Verifica non riuscita. Chiudi e riapri la prova."}
          </dd></div>
        </dl>
        <p className="note">Questa finestra verifica il runtime. La validazione dei tracciati sarà collegata nelle fasi successive.</p>
      </section>
      <footer>T-MIG-P1-01 · Prova locale senza dati clinici</footer>
    </main>
  );
}

const root = document.getElementById("root");
if (root === null) throw new Error("Contenitore dell’interfaccia assente");
createRoot(root).render(<RuntimeProbe />);
