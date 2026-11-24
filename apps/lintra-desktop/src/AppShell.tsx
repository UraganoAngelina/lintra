import { useEffect, useRef, type ReactNode } from "react";
import { ClipboardCheck, Info, Plus, Table2, type LucideIcon } from "lucide-react";
import { THEMES, VIEWS, isThemePreference, type ThemePreference, type ViewId } from "./shell";
import { Button } from "./ui";

/** Local service state as reported by the Rust bootstrap (app_info + ack). */
export type ServiceState =
  | { kind: "starting" }
  | { kind: "ready"; version: string }
  | { kind: "error"; message: string };

export type ShellProps = {
  view: ViewId;
  onView: (view: ViewId) => void;
  theme: ThemePreference;
  onTheme: (theme: ThemePreference) => void;
  service: ServiceState;
  /** Availability of a real report, owned by the Rust job lifecycle. */
  reportAvailable: boolean;
  /** A run is starting or active in Rust (T-MIG-P5-05). */
  running?: boolean;
  /** Flow and version of the current selection as Rust accepted it, if any. */
  activeFlow?: string | null;
  /** Analysis selection shown in the Analisi section (T-MIG-P5-02). */
  analysis?: ReactNode;
  /** Risultati section: report summary and diagnostics grid (T-MIG-P6-01). */
  results?: ReactNode;
  /** Informazioni section: provenance and licenses (T-MIG-P5-07). */
  information?: ReactNode;
};

const ICONS: Record<ViewId, LucideIcon> = { analisi: ClipboardCheck, risultati: Table2, informazioni: Info };
const SUBTITLES: Record<ViewId, string> = {
  analisi: "Prepara una nuova analisi.",
  risultati: "Esito, ambito e diagnostiche dell’ultimo report.",
  informazioni: "Versioni, provenienza e licenze.",
};

/**
 * Application shell in the layout of lintra_tauri_ui.png: side bar with the
 * official logo, the ratified sections, the selected flow, the theme and the
 * service state; main region with breadcrumb, title and the section body.
 * From 960 px the bar is a fixed column, below it stacks above the content.
 * Purely presentational; every state shown comes from props.
 */
export function AppShell({ view, onView, theme, onTheme, service, reportAvailable, running = false, activeFlow = null, analysis, results, information }: ShellProps) {
  const heading = useRef<HTMLHeadingElement>(null);
  const firstRender = useRef(true);
  useEffect(() => {
    // Move focus to the new section title so keyboard and screen-reader users
    // land on the content they asked for; never steal focus on first paint.
    if (firstRender.current) { firstRender.current = false; return; }
    heading.current?.focus();
  }, [view]);
  const title = VIEWS.find(item => item.id === view)?.label ?? "";
  return <div className="shell">
    <a className="skip-link" href="#content">Vai al contenuto</a>
    <header className="bar">
      <p className="brand">
        <img className="logo logo-light" src="/brand/lintra-logo-light.png" alt="Lintra" width={387} height={128} />
        <img className="logo logo-dark" src="/brand/lintra-logo-dark.png" alt="Lintra" width={351} height={121} />
      </p>
      <p className="bar-label" aria-hidden="true">Workspace</p>
      <nav aria-label="Sezioni">
        <ul>
          {VIEWS.map(item => {
            const Icon = ICONS[item.id];
            return <li key={item.id}>
              <button type="button" aria-current={item.id === view ? "page" : undefined}
                onClick={() => onView(item.id)}>
                <Icon className="nav-icon" aria-hidden="true" />
                <span className="nav-text"><span>{item.label}</span>
                  {item.id === "risultati" && <span className="nav-note">
                    {reportAvailable ? "Report disponibile" : "Nessun report"}
                  </span>}</span>
              </button>
            </li>;
          })}
        </ul>
      </nav>
      <div className="bar-flow">
        <p className="bar-label">Flusso selezionato</p>
        <p className="flow-chip">{activeFlow ?? "Nessuno"}</p>
      </div>
      <fieldset className="theme">
        <legend className="bar-label">Tema</legend>
        <div className="theme-options">
          {THEMES.map(item => <label key={item.id}>
            <input type="radio" name="theme" value={item.id} checked={item.id === theme}
              onChange={event => { if (isThemePreference(event.target.value)) onTheme(event.target.value); }} />
            <span>{item.label}</span>
          </label>)}
        </div>
      </fieldset>
      <ServiceStatus service={service} />
    </header>
    <main id="content" tabIndex={-1}>
      <div className="page-head">
        <div>
          <p className="crumbs">Lintra / {title}</p>
          <h1 ref={heading} tabIndex={-1}>{title}</h1>
          <p className="page-subtitle">{SUBTITLES[view]}</p>
        </div>
        {view === "risultati" && <Button variant="primary" onClick={() => onView("analisi")}>
          <Plus aria-hidden="true" />Nuova analisi
        </Button>}
      </div>
      <ViewBody view={view} service={service} reportAvailable={reportAvailable} running={running} analysis={analysis} results={results} information={information} />
    </main>
  </div>;
}

function ServiceStatus({ service }: { service: ServiceState }) {
  if (service.kind === "error") return <p className="service" role="alert">{service.message}</p>;
  return <div className="service">
    <p role="status" className={service.kind === "ready" ? "service-ready" : undefined}>
      {service.kind === "ready" ? "Servizio locale pronto" : "Avvio del servizio…"}
    </p>
    {service.kind === "ready" && <p className="service-version">v{service.version} · ambiente locale</p>}
  </div>;
}

/** Section bodies show only real states. */
function ViewBody({ view, service, reportAvailable, running, analysis, results, information }:
  { view: ViewId; service: ServiceState; reportAvailable: boolean; running: boolean; analysis?: ReactNode; results?: ReactNode; information?: ReactNode }) {
  switch (view) {
    case "analisi":
      return <section aria-label="Stato dell'analisi">
        <p className="run-note">{running ? "Analisi in corso." : "Nessuna analisi in corso."}</p>
        {analysis}
      </section>;
    case "risultati":
      if (results) return <section aria-label="Risultati del report">{results}</section>;
      return <section aria-label="Disponibilità del report">
        <p className="lead">{reportAvailable ? "Report disponibile." : "Nessun report disponibile."}</p>
        <p>I risultati di un'analisi conclusa compaiono in questa sezione.</p>
      </section>;
    case "informazioni":
      if (information) return information;
      return <section aria-label="Versione">
        <dl>
          <dt>Versione</dt>
          <dd>{service.kind === "ready" ? service.version : "Non ancora disponibile"}</dd>
        </dl>
      </section>;
  }
}
