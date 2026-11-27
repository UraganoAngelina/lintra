import { createRoot } from "react-dom/client";
import { useEffect, useState } from "react";
import { appInfo, clientReady, errorText, type AppInfo } from "./ipc";
import { AppShell, type ServiceState } from "./AppShell";
import { AnalysisForm, useAnalysisSelection } from "./AnalysisForm";
import { RunPanel, useRun } from "./RunPanel";
import { About, useReportAbout } from "./About";
import { Results, useDetail, useOverview, useResults } from "./Results";
import { splitNotices } from "./information";
import { ExportCard, useExport } from "./Export";
import { DEFAULT_REPORTS, preferredFormat } from "./analysis";
import { sameRunRef } from "./pages";
import fontLicense from "../public/fonts/OFL.md?raw";
import thirdPartyNotices from "../public/THIRD_PARTY_NOTICES?raw";
import { busy } from "./run";
import { INITIAL_THEME, INITIAL_VIEW, applyTheme, type ThemePreference, type ViewId } from "./shell";
import "./style.css";

// Shipped license texts, read once from the same files packaged in public/.
const NOTICES = [{ title: "IBM Plex Sans, IBM Plex Mono / OFL.md", body: fontLicense }, ...splitNotices(thirdPartyNotices)];

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
  const [selection, onSelection] = useAnalysisSelection(service.kind === "ready");
  const [run, onRun, onStart, onDecide, onCancel] = useRun(service.kind === "ready");
  const running = busy(run);
  // Report availability as Rust declared it in the outcome of the last run.
  // Rust keeps the last published report across cancelled or failed runs;
  // the run screen says whose report it is (T-MIG-P5-06).
  const reportAvailable = run.report !== null;
  const report = service.kind === "ready" ? run.report : null;
  const about = useReportAbout(report, view === "informazioni" || view === "risultati");
  const [results, onPage, onRetry, filters, onFilter] = useResults(report, view === "risultati");
  const [detail, onSelect, onDetailRetry] = useDetail(results.status === "ready" ? results.query_ref : null);
  const overview = useOverview(report, view === "risultati");
  // Export (T-MIG-P6-08): one per instance, kept across new reports until
  // Rust reports its outcome; the proposed format is the first one requested
  // with the shown run.
  const described = about.status === "ready" && report !== null && sameRunRef(about.about.run, report) ? about.about : null;
  const exporting = useExport(report === null ? null : `${report.instance_id}:${report.run_id}`);
  const runView = run.view.status === "active" || run.view.status === "terminal" ? run.view : null;
  const requested = runView && report !== null && sameRunRef(runView.run, report) ? runView.reports : DEFAULT_REPORTS;
  const exportCard = described ? <ExportCard about={described} initial={preferredFormat(requested)} view={exporting.view} previous={exporting.previous}
    onSave={exporting.save} onCancel={exporting.cancel} onCheck={exporting.check} /> : null;
  // Flow and version of the configuration Rust accepted for the current form.
  const configured = selection.configuration.status === "ready" ? selection.configuration.selection : null;
  const activeFlow = configured ? `${configured.flow_id} · ${configured.schema_version}` : null;
  return <AppShell view={view} onView={setView} theme={theme} onTheme={setTheme}
    service={service} reportAvailable={reportAvailable} running={running} activeFlow={activeFlow}
    analysis={service.kind === "ready" ? <div className="analysis-layout">
      <AnalysisForm state={selection} onEvent={onSelection} busy={running} />
      <RunPanel state={run} selection={selection} onEvent={onRun} onStart={onStart} onDecide={onDecide} onCancel={onCancel} />
    </div> : null}
    results={service.kind === "ready" ? <Results view={results} about={about} overview={overview} reportAvailable={reportAvailable} filters={filters} onFilter={onFilter} onPage={onPage} onRetry={onRetry} detail={detail} onSelect={onSelect} onDetailRetry={onDetailRetry} exportCard={exportCard} /> : null}
    information={<About version={service.kind === "ready" ? service.version : null} selection={selection} report={about} notices={NOTICES} />} />;
}

const root = document.getElementById("root");
if (!root) throw new Error("Application root unavailable");
createRoot(root).render(<App />);
