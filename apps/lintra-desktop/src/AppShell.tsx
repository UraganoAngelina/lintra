import { useEffect, useRef, type ReactNode } from "react";
import { THEMES, VIEWS, isThemePreference, type ThemePreference, type ViewId } from "./shell";

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
  /** Analysis selection shown in the Analisi section (T-MIG-P5-02). */
  analysis?: ReactNode;
  /** Informazioni section: provenance and licenses (T-MIG-P5-07). */
  information?: ReactNode;
};

/**
 * Responsive application shell: banner with the ratified bar (sections,
 * theme, service state) and the main region of the selected section.
 * Purely presentational; every state shown comes from props.
 */
export function AppShell({ view, onView, theme, onTheme, service, reportAvailable, running = false, analysis, information }: ShellProps) {
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
      <p className="app-name">Lintra</p>
      <nav aria-label="Sezioni">
        <ul>
          {VIEWS.map(item => <li key={item.id}>
            <button type="button" aria-current={item.id === view ? "page" : undefined}
              onClick={() => onView(item.id)}>
              <span>{item.label}</span>
              {item.id === "risultati" && <span className="nav-note">
                {reportAvailable ? "Report disponibile" : "Nessun report"}
              </span>}
            </button>
          </li>)}
        </ul>
      </nav>
      <fieldset className="theme">
        <legend>Tema</legend>
        {THEMES.map(item => <label key={item.id}>
          <input type="radio" name="theme" value={item.id} checked={item.id === theme}
            onChange={event => { if (isThemePreference(event.target.value)) onTheme(event.target.value); }} />
          {item.label}
        </label>)}
      </fieldset>
      <ServiceStatus service={service} />
    </header>
    <main id="content" tabIndex={-1}>
      <h1 ref={heading} tabIndex={-1}>{title}</h1>
      <ViewBody view={view} service={service} reportAvailable={reportAvailable} running={running} analysis={analysis} information={information} />
    </main>
  </div>;
}

function ServiceStatus({ service }: { service: ServiceState }) {
  if (service.kind === "error") return <p className="service" role="alert">{service.message}</p>;
  return <p className="service" role="status">
    {service.kind === "ready" ? "Servizio locale pronto" : "Avvio del servizio…"}
  </p>;
}

/** Section bodies show only real states; forms and results arrive in P5/P6. */
function ViewBody({ view, service, reportAvailable, running, analysis, information }:
  { view: ViewId; service: ServiceState; reportAvailable: boolean; running: boolean; analysis?: ReactNode; information?: ReactNode }) {
  switch (view) {
    case "analisi":
      return <section aria-label="Stato dell'analisi">
        <p className="lead">Prepara una nuova analisi.</p>
        <p>{running ? "Analisi in corso." : "Nessuna analisi in corso."}</p>
        {analysis}
      </section>;
    case "risultati":
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
