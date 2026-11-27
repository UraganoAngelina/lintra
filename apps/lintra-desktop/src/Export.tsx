// T-MIG-P6-08: «Salva il report» in Risultati. The renderer sends only the
// opaque report_ref and a format: Rust opens the native save dialog, asks the
// native overwrite and destination confirmations, writes the whole captured
// report with the canonical writers and publishes report and summary as two
// separate files (EXPORT.md). Here: the format choice, the job lifecycle as
// Rust reports it and its receipt, never a path or report content.
import { useEffect, useRef, useState, useSyncExternalStore } from "react";
import { exportCancel, exportExecute, exportPrepare, exportState, type ExportFormat, type ExportResult, type ReportAbout } from "./ipc";
import { REPORT_FORMATS } from "./analysis";
import { Button, Card } from "./ui";

/** Interval between two reads of a running export. */
export const EXPORT_POLL_MS = 250;

type Terminal = Extract<ExportResult, { state: "succeeded" | "failed" | "cancelled" }>;
export type ExportView =
  | { status: "idle" }
  | { status: "choosing" }
  | { status: "closed"; reason: "native_selection_cancelled" | "native_confirmation_declined" }
  | { status: "rejected" }
  | { status: "prepare_failed" }
  | { status: "writing"; exportRef: string; cancel: "none" | "requested" | "failed" }
  | { status: "done"; result: Terminal }
  // The reply was lost: files may already be published, so no blind retry;
  // Rust keeps the receipt, read again on request (EXPORT.md).
  | { status: "unknown"; exportRef: string };

const ROLE = { report: "il report", summary: "il riepilogo" } as const;

/** What a terminal export left on disk, as Rust's receipt says. Two
 * publications are never presented as one atomic write. */
export function receiptText(result: Terminal): { tone: "status" | "alert"; lines: string[] } {
  const format = result.format.toUpperCase();
  const confirmed = result.destination.warning_acknowledged
    ? ["Destinazione remota o non determinata confermata nella finestra di sistema."] : [];
  if (result.state === "succeeded") return { tone: "status", lines: [
    `Report e riepilogo salvati in formato ${format}.`,
    "Sono due file pubblicati uno dopo l'altro, non un'unica scrittura.", ...confirmed] };
  const roles = result.published_roles;
  const [only] = roles;
  const lines = [result.state === "cancelled"
    ? roles.length === 0 ? "Salvataggio annullato prima della pubblicazione dei file." : "Salvataggio annullato."
    : "Salvataggio non riuscito."];
  if (roles.length === 0) lines.push("Nessun file è stato pubblicato.");
  else if (roles.length === 1 && only) lines.push(`È stato salvato solo ${ROLE[only]}: l'esportazione non è completa. Controlla la destinazione.`);
  else lines.push("Report e riepilogo sono stati pubblicati, ma l'esportazione non è completa. Controlla la destinazione.");
  if (roles.length > 0 && !result.durability_confirmed) lines.push("La scrittura su disco dei file pubblicati non è confermata.");
  if (result.cleanup_failed) lines.push("Non è stato possibile eliminare tutti i file temporanei creati da questo salvataggio.");
  return { tone: "alert", lines: [...lines, ...confirmed] };
}

/** Why Rust does not allow saving this report (EXPORT.md: cancelled and
 * truncated reports are not exportable; Partial alone is). */
function notExportable(about: ReportAbout): string {
  if (about.status === "cancelled") return "Il report di un'analisi interrotta non si può salvare.";
  if (about.truncated) return "Il report di un'analisi che ha raggiunto il limite operativo non si può salvare.";
  return "Il servizio non consente di salvare questo report.";
}

export type ExportApi = { exportPrepare: typeof exportPrepare; exportExecute: typeof exportExecute; exportState: typeof exportState; exportCancel: typeof exportCancel };
const IPC: ExportApi = { exportPrepare, exportExecute, exportState, exportCancel };

type Set = (view: ExportView) => void;
type Update = (change: (view: ExportView) => ExportView) => void;

/** Reads a running export until Rust reports its terminal state. A lost
 * reply leaves the outcome unknown, never "nothing written". */
export function followExport(api: ExportApi, exportRef: string, live: () => boolean, set: Set): void {
  const read = () => api.exportState(exportRef).then(result => {
    if (!live()) return;
    if (result.state === "writing") { setTimeout(() => { if (live()) read(); }, EXPORT_POLL_MS); return; }
    set({ status: "done", result });
  }, () => { if (live()) set({ status: "unknown", exportRef }); });
  read();
}

/** Native save of the report (export_prepare), then the owned job
 * (export_execute), followed to its receipt. Every reply is used only while
 * `live()` holds. */
export function startExport(api: ExportApi, reportRef: string, format: ExportFormat, live: () => boolean, set: Set): void {
  set({ status: "choosing" });
  api.exportPrepare(reportRef, format).then(plan => {
    if (!live()) return;
    if (plan.state === "cancelled") { set({ status: "closed", reason: plan.reason }); return; }
    if (plan.state === "rejected") { set({ status: "rejected" }); return; }
    api.exportExecute(plan.export_ref).then(() => {
      if (!live()) return;
      set({ status: "writing", exportRef: plan.export_ref, cancel: "none" });
      followExport(api, plan.export_ref, live, set);
    }, () => { if (live()) set({ status: "unknown", exportRef: plan.export_ref }); });
  }, () => { if (live()) set({ status: "prepare_failed" }); });
}

/** Asks Rust to cancel; only the receipt says what was published. */
export function cancelExport(api: ExportApi, exportRef: string, live: () => boolean, update: Update): void {
  const mark = (cancel: "requested" | "failed") => update(view => view.status === "writing" && view.exportRef === exportRef ? { ...view, cancel } : view);
  api.exportCancel(exportRef).then(disposition => {
    // already_terminal: the next read publishes the outcome.
    if (live() && disposition !== "already_terminal") mark("requested");
  }, () => { if (live()) mark("failed"); });
}

/** An export Rust may still be carrying out, or whose outcome is not known:
 * its export_ref must stay reachable until the receipt is read. */
export function exportPending(view: ExportView): boolean {
  return view.status === "choosing" || view.status === "writing" || view.status === "unknown";
}

/** The export and the report (RunRef key) that started it. */
export type ExportJob = { owner: string | null; view: ExportView };
const IDLE: ExportView = { status: "idle" };

/**
 * The instance's export, independent of the report on screen (P608-01):
 * Rust keeps writing the captured report after a new run, so a new report
 * clears only a settled outcome, never a pending one. While one is pending
 * no new save starts (P608-02: Rust would answer Busy and the uncertain
 * export_ref would be lost).
 */
export function exportStore(api: ExportApi = IPC) {
  let job: ExportJob = { owner: null, view: IDLE };
  let shown: string | null = null;
  const listeners = new Set<() => void>();
  const update = (change: (current: ExportJob) => ExportJob) => { job = change(job); listeners.forEach(listener => listener()); };
  // No stale replies to drop: a job starts only once the previous one settled,
  // and a late cancel reply only marks the writing job it names.
  const live = () => true;
  const set: Set = view => update(current => ({ ...current, view }));
  return {
    get: () => job,
    subscribe(listener: () => void) { listeners.add(listener); return () => { listeners.delete(listener); }; },
    /** The report now shown: a change clears a settled outcome only. */
    report(owner: string | null) {
      if (owner === shown) return;
      shown = owner;
      update(current => exportPending(current.view) ? current : { owner, view: IDLE });
    },
    save(owner: string, reportRef: string, format: ExportFormat) {
      if (exportPending(job.view)) return;
      update(() => ({ owner, view: { status: "choosing" } }));
      startExport(api, reportRef, format, live, set);
    },
    cancel() {
      const view = job.view;
      if (view.status === "writing" && view.cancel !== "requested") cancelExport(api, view.exportRef, live, change => update(current => ({ ...current, view: change(current.view) })));
    },
    check() {
      const view = job.view;
      if (view.status !== "unknown") return;
      set({ status: "writing", exportRef: view.exportRef, cancel: "none" });
      followExport(api, view.exportRef, live, set);
    },
  };
}

/** The export store for the published report `report` (RunRef key):
 * `previous` when the shown export was started from an earlier report. */
export function useExport(report: string | null, api: ExportApi = IPC) {
  const [store] = useState(() => exportStore(api));
  const job = useSyncExternalStore(store.subscribe, store.get);
  useEffect(() => { store.report(report); }, [store, report]);
  return {
    view: job.view, previous: job.owner !== null && job.owner !== report,
    save: (reportRef: string, format: ExportFormat) => { if (report !== null) store.save(report, reportRef, format); },
    cancel: store.cancel, check: store.check,
  };
}

/** «Salva il report»: format, the native save, then Rust's receipt. */
export function ExportCard({ about, initial, view, previous = false, onSave, onCancel, onCheck }:
  { about: ReportAbout; initial: ExportFormat; view: ExportView; previous?: boolean; onSave: (reportRef: string, format: ExportFormat) => void; onCancel: () => void; onCheck: () => void }) {
  const [format, setFormat] = useState<ExportFormat>(initial);
  useEffect(() => { setFormat(initial); }, [about.report_ref, initial]);
  // The card is last in Risultati: bring each new state into view, not only
  // to assistive technologies (role status/alert).
  const status = useRef<HTMLDivElement>(null);
  useEffect(() => { if (view.status !== "idle") status.current?.scrollIntoView({ block: "nearest" }); }, [view]);
  const busy = exportPending(view);
  return <Card className="export-card" aria-labelledby="export-title">
    <div className="export-head">
      <h2 id="export-title">Salva il report</h2>
      <p className="text-muted">Il salvataggio include report e riepilogo JSON con versioni e provenienza. I filtri della tabella non limitano l'esportazione.</p>
    </div>
    {previous && view.status !== "idle" && <p className="export-message" role="note">Questo salvataggio è stato avviato dal report di un'analisi precedente, non da quello mostrato.</p>}
    {!about.exportable && !busy ? <p className="export-message" role="note">{notExportable(about)}</p> : <>
      <div className="export-actions">
        <fieldset className="export-formats" disabled={busy}>
          <legend>Formato</legend>
          {REPORT_FORMATS.map(item => <label key={item.id}>
            <input type="radio" name="export-format" value={item.id} checked={format === item.id} onChange={() => setFormat(item.id)} />
            {item.label}
          </label>)}
        </fieldset>
        {view.status === "writing"
          ? <Button variant="ghost" aria-disabled={view.cancel === "requested"} onClick={() => { if (view.cancel !== "requested") onCancel(); }}>Annulla salvataggio</Button>
          : <Button variant="primary" aria-disabled={busy} onClick={() => { if (!busy) onSave(about.report_ref, format); }}>Salva report…</Button>}
      </div>
    </>}
    <div ref={status} className="export-status"><ExportStatus view={view} onCheck={onCheck} /></div>
  </Card>;
}

function ExportStatus({ view, onCheck }: { view: ExportView; onCheck: () => void }) {
  switch (view.status) {
    case "idle": return null;
    case "choosing": return <p className="export-message" role="status">Scegli la destinazione nella finestra di salvataggio…</p>;
    case "closed": return <p className="export-message" role="status">{view.reason === "native_selection_cancelled"
      ? "Salvataggio annullato: nessun file scritto." : "Conferma non data: nessun file scritto."}</p>;
    case "rejected": return <p className="export-message" role="alert">Destinazione non consentita: nessun file scritto. Scegli un'altra cartella o un altro nome.</p>;
    case "prepare_failed": return <p className="export-message" role="alert">Non è stato possibile preparare il salvataggio: nessun file scritto.</p>;
    case "writing": return <p className="export-message" role="status">{view.cancel === "requested"
      ? "Annullamento richiesto. Attendi l'esito del salvataggio."
      : view.cancel === "failed" ? "Annullamento non riuscito. Il salvataggio continua: attendi l'esito."
      : "Salvataggio in corso. Attendi la conferma prima di usare i file."}</p>;
    case "unknown": return <div className="export-message" role="alert">
      <p>Esito del salvataggio non disponibile: alcuni file potrebbero essere già stati salvati. Controlla l'esito prima di avviare un altro salvataggio.</p>
      <Button size="sm" variant="ghost" onClick={onCheck}>Controlla l'esito</Button>
    </div>;
    case "done": {
      const text = receiptText(view.result);
      return <div className="export-message" role={text.tone}>{text.lines.map(line => <p key={line}>{line}</p>)}</div>;
    }
  }
}
