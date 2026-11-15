import { createRoot } from "react-dom/client";
import "./style.css";

function App() {
  return <main>
    <header><span className="wordmark">Lintra</span></header>
    <section aria-labelledby="welcome">
      <h1 id="welcome">Il tuo spazio di validazione</h1>
      <p>Nessuna analisi in corso.</p>
    </section>
  </main>;
}

const root = document.getElementById("root");
if (!root) throw new Error("Application root unavailable");
createRoot(root).render(<App />);
