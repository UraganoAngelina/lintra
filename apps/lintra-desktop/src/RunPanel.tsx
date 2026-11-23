import { useEffect, useLayoutEffect, useReducer, useRef } from "react";
import {
  planPrepare, resourceBudgetState, validationAcknowledge, validationBudget, validationBudgetDecide,
  validationProgress, validationStart, validationState, type BudgetGate,
} from "./ipc";
import type { SelectionState } from "./analysis";
import {
  INITIAL_RUN, POLL_MS, acknowledge, activeText, budgetText, bytes, canStart, decide, gateOpen, loadBudget, outcomeText, phaseText, pollRun,
  reduceRun, startRun, BUDGET_UNREADABLE, DECISION_FAILED, type RunApi, type RunEvent, type RunState,
} from "./run";

const api: RunApi = {
  resourceBudgetState, planPrepare, validationStart: (planRef, mode) => validationStart(planRef, mode),
  validationState: run => validationState(run), validationProgress, validationBudget, validationBudgetDecide, validationAcknowledge,
};

/** Owns the run screen state for the window lifetime and drives the
 * throttled polling of the active run (one request in flight at a time). */
export function useRun(serviceReady: boolean): [RunState, (event: RunEvent) => void, (planRef: string) => void, (decision: "proceed" | "cancel") => void] {
  const [state, dispatch] = useReducer(reduceRun, INITIAL_RUN);
  useEffect(() => { if (serviceReady) void loadBudget(api, dispatch); }, [serviceReady]);
  const startToken = state.view.status === "starting" ? state.view.token : null;
  const pending = useRef<string | null>(null);
  useEffect(() => {
    if (startToken === null || pending.current === null) return;
    void startRun(api, dispatch, startToken, pending.current, state.lastRun);
  }, [startToken]);
  const view = state.view;
  const run = view.status === "active" ? view.run : null;
  const open = view.status === "active" && gateOpen(view.gate);
  const openRef = useRef(open);
  openRef.current = open;
  useEffect(() => {
    if (run === null) return;
    let alive = true;
    let timer: ReturnType<typeof setTimeout> | undefined;
    const tick = async () => {
      let again = true;
      // A failed read is retried at the next tick; the Rust state is kept.
      try { again = await pollRun(api, event => { if (alive) dispatch(event); }, run, openRef.current); } catch { again = true; }
      if (alive && again) timer = setTimeout(() => void tick(), POLL_MS);
    };
    void tick();
    return () => { alive = false; if (timer !== undefined) clearTimeout(timer); };
  }, [run?.instance_id, run?.run_id]);
  // A failed terminal ack is retried until Rust accepts it (P505-03).
  const unacknowledged = view.status === "terminal" && !view.acknowledged ? view.run : null;
  useEffect(() => {
    if (unacknowledged === null) return;
    let alive = true;
    let timer: ReturnType<typeof setTimeout> | undefined;
    const retry = async () => {
      if (!alive || await acknowledge(api, event => { if (alive) dispatch(event); }, unacknowledged)) return;
      if (alive) timer = setTimeout(() => void retry(), POLL_MS);
    };
    timer = setTimeout(() => void retry(), POLL_MS);
    return () => { alive = false; if (timer !== undefined) clearTimeout(timer); };
  }, [unacknowledged?.instance_id, unacknowledged?.run_id]);
  const start = (planRef: string) => { pending.current = planRef; };
  const choose = (decision: "proceed" | "cancel") => {
    if (view.status === "active" && view.gate && !view.deciding) void decide(api, dispatch, view.gate, decision);
  };
  return [state, dispatch, start, choose];
}

/** "Avvia analisi", budget configuration status, progress and outcome. */
export function RunPanel({ state, selection, onEvent, onStart, onDecide }: {
  state: RunState; selection: SelectionState; onEvent: (event: RunEvent) => void;
  onStart: (planRef: string) => void; onDecide: (decision: "proceed" | "cancel") => void;
}) {
  const view = state.view;
  const plan = selection.plan.status === "ready" && selection.plan.input.chosen !== null ? selection.plan.draft : null;
  const busy = view.status === "starting" || view.status === "active";
  // The stored budget state is information only: "Avvia" reads the file
  // again, so a corrected or removed file is honoured (P505-02).
  const ready = plan !== null && canStart(state);
  // In the single-column layout the outcome may be below the fold: bring it
  // into view once, when Rust reports the terminal (focus is not moved).
  const terminal = view.status === "terminal" ? `${view.run.instance_id}:${view.run.run_id}` : null;
  const outcome = useRef<HTMLDivElement>(null);
  useEffect(() => { if (terminal !== null) outcome.current?.scrollIntoView?.({ block: "nearest" }); }, [terminal]);
  return <section className="run" aria-labelledby="run-title">
    <h2 id="run-title">Esecuzione</h2>
    {state.budgetFailed ? <p className="selection-status" role="alert">{BUDGET_UNREADABLE}</p>
      : state.budget && <p className="selection-status" role={state.budget.state === "invalid" ? "alert" : "status"}>{budgetText(state.budget)}</p>}
    <button type="button" className="primary" aria-disabled={!ready}
      onClick={() => { if (ready && plan) { onStart(plan.plan_ref); onEvent({ type: "start", reports: selection.reports }); } }}>
      Avvia analisi
    </button>
    {!busy && plan === null && <p className="selection-status">Completa la selezione e scegli il file per avviare l'analisi.</p>}
    <div ref={outcome}><RunStatus state={state} /></div>
    {view.status === "active" && view.gate?.state === "confirmation_required" &&
      <BudgetDialog gate={view.gate} deciding={view.deciding} onDecide={onDecide} />}
  </section>;
}

function RunStatus({ state }: { state: RunState }) {
  const view = state.view;
  if (view.status === "idle") return null;
  if (view.status === "failed") return <p className="selection-status" role="alert">{view.message}</p>;
  if (view.status === "starting") return <p className="selection-status" role="status">Il servizio prepara l'analisi…</p>;
  const formats = Object.entries(view.reports).filter(([, on]) => on).map(([id]) => id.toUpperCase()).join(", ") || "nessuno";
  if (view.status === "terminal") {
    const { title, detail } = outcomeText(view.outcome);
    return <div className="run-state" role="status">
      <h3>{title}</h3>
      <p>{detail}</p>
      <GateNote gate={view.gate} />
      <RunCounters progress={view.progress} formats={formats} />
    </div>;
  }
  const phase = phaseText(view.progress);
  return <div className="run-state">
    <p className="selection-status" role="status">{activeText(view)}</p>
    {/* Indeterminate: Rust gives no total, so no percentage or ETA is shown. */}
    <progress aria-label="Avanzamento dell'analisi" />
    {view.decisionFailed && <p className="selection-status" role="alert">{DECISION_FAILED}</p>}
    {phase && <p>{phase}</p>}
    <GateNote gate={view.gate} />
    <RunCounters progress={view.progress} formats={formats} />
  </div>;
}

/** Trace of the D09 choice for this run, as Rust recorded it. */
function GateNote({ gate }: { gate: BudgetGate | null }) {
  if (gate?.state === "proceeded") return <p>Soglia di avviso superata: hai scelto Procedi comunque, l’input viene analizzato per intero.</p>;
  if (gate?.state === "declined") return <p>Soglia di avviso superata: hai scelto Annulla.</p>;
  if (gate?.state === "within_budget") return <p>File entro le soglie di avviso configurate.</p>;
  return null;
}

function RunCounters({ progress, formats }: { progress: { trace_records: string; cross_records: string; trace_ref: string | null } | null; formats: string }) {
  return <dl className="selection-summary" aria-label="Contatori del servizio">
    <dt>Record del tracciato</dt><dd>{progress ? BigInt(progress.trace_records).toLocaleString("it-IT") : "—"}</dd>
    {progress && progress.cross_records !== "0" && <><dt>Record nei controlli tra record</dt><dd>{BigInt(progress.cross_records).toLocaleString("it-IT")}</dd></>}
    <dt>Formati del report</dt><dd>{formats}</dd>
  </dl>;
}

/** D09 warning with the counters Rust measured. Modal native dialog: focus
 * stays inside, Esc means Annulla, Annulla has the initial focus. */
export function BudgetDialog({ gate, deciding, onDecide }: { gate: BudgetGate; deciding: boolean; onDecide: (decision: "proceed" | "cancel") => void }) {
  const dialog = useRef<HTMLDialogElement>(null);
  // Layout effect: close() runs while the element is still attached, so the
  // browser returns the focus to "Avvia analisi" when the decision is taken.
  useLayoutEffect(() => {
    const element = dialog.current;
    if (element && typeof element.showModal === "function" && !element.open) element.showModal();
    return () => { if (element?.open) element.close(); };
  }, []);
  return <dialog ref={dialog} className="budget-dialog" aria-labelledby="budget-title" aria-describedby="budget-text"
    onCancel={event => { event.preventDefault(); if (!deciding) onDecide("cancel"); }}>
    <BudgetChoice gate={gate} deciding={deciding} onDecide={onDecide} />
  </dialog>;
}

/** Content and actions of the dialog (no hooks: rendered and clicked in tests). */
export function BudgetChoice({ gate, deciding, onDecide }: { gate: BudgetGate; deciding: boolean; onDecide: (decision: "proceed" | "cancel") => void }) {
  const audit = gate.audit;
  if (audit === null) return null;
  return <>
    <h3 id="budget-title">Soglia di avviso superata</h3>
    <p id="budget-text">I file superano la soglia di avviso configurata. L’analisi non ridurrà l’input per rientrare nella soglia.</p>
    <dl className="selection-summary" aria-label="Misure del servizio">
      {audit.input_budget !== null && <><dt>Input</dt><dd>{bytes(audit.input_bytes)} (soglia {bytes(audit.input_budget)}){audit.input_budget_exceeded ? ", superata" : ""}</dd></>}
      {audit.reference_budget !== null && <><dt>Reference</dt><dd>{bytes(audit.reference_bytes)} (soglia {bytes(audit.reference_budget)}){audit.reference_budget_exceeded ? ", superata" : ""}</dd></>}
      {audit.missing_reference_files !== "0" && <><dt>Reference mancanti</dt><dd>{audit.missing_reference_files} file (non sono dataset vuoti)</dd></>}
    </dl>
    <div className="dialog-actions">
      <button type="button" autoFocus aria-disabled={deciding} onClick={() => { if (!deciding) onDecide("cancel"); }}>Annulla</button>
      <button type="button" className="primary" aria-disabled={deciding} onClick={() => { if (!deciding) onDecide("proceed"); }}>Procedi comunque</button>
    </div>
  </>;
}
