import { createRoot } from "react-dom/client";
import { useEffect, useState } from "react";
import { appInfo, clientReady, errorText, type AppInfo } from "./ipc";
import "./style.css";

function App() {
  const [info, setInfo] = useState<AppInfo | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [ready, setReady] = useState(false);
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
  return <main>
    <header><span className="app-name">Lintra</span></header>
    <section aria-labelledby="welcome">
      <h1 id="welcome">Il tuo spazio di validazione</h1>
      {error ? <p role="alert">{error}</p> : <p role="status">
        {ready ? "Nessuna analisi in corso." : "Avvio del servizio…"}
      </p>}
    </section>
  </main>;
}

const root = document.getElementById("root");
if (!root) throw new Error("Application root unavailable");
createRoot(root).render(<App />);
