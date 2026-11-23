// Run screen state (T-MIG-P5-05): start, throttled progress and the D09
// soft-budget decision. Rust owns the job lifecycle, the preflight and the
// outcome; this module only keeps the latest Rust samples of the current run
// and maps them to words. Late replies of another run or start are ignored.
//
// Polling is throttled (POLL_MS) with one request in flight at a time: one
// snapshot, one progress sample and, while undecided, one budget gate are
// kept, so memory stays bounded for runs of any length (PROGRESS.md).
import type { BudgetConfigState, BudgetGate, RunOutcome, RunProgress, RunRef, RunSnapshot } from "./ipc";
import type { ReportSelection } from "./analysis";

/** Interval between two reads of the run (at most ~4 renders per second). */
export const POLL_MS = 250;

export type RunView =
  | { status: "idle" }
  | { status: "starting"; token: number; reports: ReportSelection }
  | { status: "active"; run: RunRef; reports: ReportSelection; snapshot: RunSnapshot | null; progress: RunProgress | null;
      gate: BudgetGate | null; deciding: boolean; decisionFailed: boolean; cancel: CancelState }
  | { status: "terminal"; run: RunRef; reports: ReportSelection; outcome: RunOutcome; progress: RunProgress | null; gate: BudgetGate | null;
      acknowledged: boolean }
  | { status: "failed"; message: string };

/** Operator cancel of the active run (T-MIG-P5-06): what Rust answered. A
 * request proves nothing: only the terminal says whether it was cancelled. */
export type CancelState = "none" | "requesting" | "requested" | "too_late" | "failed";

/** `lastRun`: the last run this window attached to, so a resynchronization
 * after an uncertain start never mistakes it for the new one (P505-04).
 * `report`: the run whose report Rust holds as last published. A cancelled
 * or failed newer run never replaces it nor takes its identity (LIFECYCLE.md,
 * "errore/cancel nuovo contro report precedente"). */
export type RunState = { view: RunView; budget: BudgetConfigState | null; budgetFailed: boolean; nextToken: number; lastRun: RunRef | null;
  report: RunRef | null };

export type RunEvent =
  | { type: "budget_loaded"; budget: BudgetConfigState }
  | { type: "budget_failed" }
  | { type: "start"; reports: ReportSelection }
  | { type: "started"; token: number; run: RunRef }
  | { type: "start_failed"; token: number; message: string }
  | { type: "snapshot"; snapshot: RunSnapshot }
  | { type: "progress"; progress: RunProgress }
  | { type: "gate"; gate: BudgetGate }
  | { type: "decide" }
  | { type: "decide_failed" }
  | { type: "terminal"; run: RunRef; outcome: RunOutcome; acknowledged: boolean }
  | { type: "acknowledged"; run: RunRef }
  | { type: "cancel" }
  | { type: "cancel_answer"; run: RunRef; disposition: "requested" | "already_requested" | "too_late" | "already_terminal" }
  | { type: "cancel_failed"; run: RunRef };

export const INITIAL_RUN: RunState = { view: { status: "idle" }, budget: null, budgetFailed: false, nextToken: 1, lastRun: null, report: null };

const same = (left: RunRef, right: RunRef) => left.instance_id === right.instance_id && left.run_id === right.run_id;

/** True while the form must not change (Slint `enabled: !root.busy`). */
export function busy(state: RunState): boolean {
  return state.view.status === "starting" || state.view.status === "active";
}

/** A new start is possible once Rust has the previous terminal acknowledged
 * (LIFECYCLE.md: start is refused before the ack). The stored budget state
 * never blocks it: each start reads the configuration again (P505-02). */
export function canStart(state: RunState): boolean {
  return !busy(state) && !(state.view.status === "terminal" && !state.view.acknowledged);
}

/** Pure transition; every Rust value is taken as is, never recomputed. */
export function reduceRun(state: RunState, event: RunEvent): RunState {
  const view = state.view;
  switch (event.type) {
    case "budget_loaded": return { ...state, budget: event.budget, budgetFailed: false };
    case "budget_failed": return { ...state, budget: null, budgetFailed: true };
    case "start":
      if (busy(state)) return state;
      return { ...state, view: { status: "starting", token: state.nextToken, reports: event.reports }, nextToken: state.nextToken + 1 };
    case "started":
      if (view.status !== "starting" || view.token !== event.token) return state;
      return { ...state, lastRun: event.run, view: { status: "active", run: event.run, reports: view.reports, snapshot: null, progress: null,
        gate: null, deciding: false, decisionFailed: false, cancel: "none" } };
    case "start_failed":
      if (view.status !== "starting" || view.token !== event.token) return state;
      return { ...state, view: { status: "failed", message: event.message } };
    case "snapshot": {
      if (view.status !== "active" || event.snapshot.run === null || !same(view.run, event.snapshot.run)) return state;
      // A lost or undecodable cancel reply proves nothing (LIFECYCLE.md):
      // once Rust shows the request recorded, the uncertainty is resolved.
      const confirmed = event.snapshot.cancel_requested || event.snapshot.state === "cancelling";
      return { ...state, view: { ...view, snapshot: event.snapshot, cancel: view.cancel === "failed" && confirmed ? "requested" : view.cancel } };
    }
    case "progress": {
      if (view.status !== "active" || !same(view.run, event.progress.run)) return state;
      // A repeated or older sequence never moves the display backwards.
      if (view.progress !== null && BigInt(event.progress.sequence) <= BigInt(view.progress.sequence)) return state;
      return { ...state, view: { ...view, progress: event.progress } };
    }
    case "gate":
      if (view.status !== "active" || !same(view.run, event.gate.run)) return state;
      return { ...state, view: { ...view, gate: event.gate, deciding: view.deciding && event.gate.state === "confirmation_required" } };
    case "decide":
      if (view.status !== "active" || view.deciding || view.gate?.state !== "confirmation_required") return state;
      return { ...state, view: { ...view, deciding: true, decisionFailed: false } };
    case "decide_failed":
      if (view.status !== "active") return state;
      return { ...state, view: { ...view, deciding: false, decisionFailed: true } };
    case "terminal":
      // A terminal of this run is final: late samples cannot reopen it.
      if (view.status !== "active" || !same(view.run, event.run)) return state;
      return { ...state, report: event.outcome.kind === "validation" && event.outcome.report_available ? view.run : state.report,
        view: { status: "terminal", run: view.run, reports: view.reports, outcome: event.outcome,
        progress: view.progress, gate: view.gate, acknowledged: event.acknowledged } };
    case "acknowledged":
      if (view.status !== "terminal" || !same(view.run, event.run)) return state;
      return { ...state, view: { ...view, acknowledged: true } };
    case "cancel":
      if (view.status !== "active" || !canCancel(view)) return state;
      return { ...state, view: { ...view, cancel: "requesting" } };
    case "cancel_answer": {
      if (view.status !== "active" || !same(view.run, event.run)) return state;
      // already_terminal: the next state read shows the real outcome.
      const cancel = event.disposition === "too_late" ? "too_late" : "requested";
      return { ...state, view: { ...view, cancel } };
    }
    case "cancel_failed":
      if (view.status !== "active" || !same(view.run, event.run)) return state;
      return { ...state, view: { ...view, cancel: "failed" } };
  }
}

/** "Interrompi analisi" is offered while the run is active and Rust may still
 * accept it (no cancel requested yet, no D09 decision pending: there Annulla
 * is the cancel). Finalizing is included: before the service boundary Rust
 * still answers requested, after it too_late (LIFECYCLE.md); only Rust knows
 * which side of the boundary the run is on. */
export function canCancel(view: Extract<RunView, { status: "active" }>): boolean {
  const snapshot = view.snapshot?.state === "idle" ? null : view.snapshot;
  const state = snapshot?.state ?? "preparing";
  return (view.cancel === "none" || view.cancel === "failed") && !snapshot?.cancel_requested
    && (state === "preparing" || state === "running" || state === "finalizing") && view.gate?.state !== "confirmation_required";
}

export const CANCEL_TOO_LATE = "L'analisi è già in conclusione: non è più possibile interromperla. Attendi l'esito.";
/** Neutral: a transport error does not prove Rust ignored the request. */
export const CANCEL_FAILED = "Non è stato possibile confermare la richiesta di interruzione. Se l'analisi prosegue, riprova.";

/** Words of the interface (MIGRATION_TAURI_BRIEF.md, "Stati e parole"). */
export const START_FAILED = "Non è stato possibile avviare l'analisi. Controlla la selezione e il file, poi riprova; se il problema resta, chiudi e riapri Lintra.";
export const BUDGET_INVALID = "Il file di configurazione dei budget di risorse non è valido: correggilo o rimuovilo, poi avvia di nuovo l'analisi.";
export const BUDGET_UNREADABLE = "Non è stato possibile leggere lo stato dei budget di risorse. Chiudi e riapri Lintra.";
export const DECISION_FAILED = "La scelta non è stata registrata: la richiesta non è più valida. Attendi lo stato aggiornato.";

export function budgetText(budget: BudgetConfigState): string {
  if (budget.state === "unconfigured") return "Budget di risorse non configurato: nessun avviso sulle dimensioni dei file.";
  if (budget.state === "invalid") return BUDGET_INVALID;
  const limits = [budget.input_bytes && `${bytes(budget.input_bytes)} di input`, budget.reference_bytes && `${bytes(budget.reference_bytes)} di reference`]
    .filter(Boolean).join(" e ");
  return limits ? `Budget di risorse configurato: avviso oltre ${limits}.` : "Budget di risorse configurato senza soglie: nessun avviso sulle dimensioni dei file.";
}

export function bytes(value: string): string {
  return `${BigInt(value).toLocaleString("it-IT")} byte`;
}

const PHASES: Record<NonNullable<RunProgress["phase"]>, string> = {
  preflight: "Controlli preliminari",
  local_validation: "Validazione dei record",
  cross_record_validation: "Controlli tra record",
  finalizing_report: "Scrittura del report",
};

/** Headline of an active run, from the Rust state and gate only. */
export function activeText(view: Extract<RunView, { status: "active" }>): string {
  const snapshot = view.snapshot?.state === "idle" ? null : view.snapshot;
  const state = snapshot?.state ?? "preparing";
  if (state === "cancelling" || snapshot?.cancel_requested || view.cancel === "requested") return "Interruzione richiesta: attendi la conclusione comunicata dal servizio.";
  if (view.gate?.state === "checking") return "Controllo dei budget di risorse: lettura dei file selezionati…";
  if (view.gate?.state === "confirmation_required") return "I file superano la soglia di avviso configurata: scegli come procedere.";
  if (state === "preparing") return "Il servizio prepara l'analisi…";
  if (state === "finalizing") return "Scrittura del report…";
  return "Analisi in corso.";
}

export function phaseText(progress: RunProgress | null): string | null {
  return progress?.phase ? PHASES[progress.phase] : null;
}

/** Outcome words; NotApplicable/Unknown/Partial are never merged into success.
 * `previousReport`: Rust still holds the report of an earlier run; it is
 * named as such and never presented as this run's result (T-MIG-P5-06). */
export function outcomeText(outcome: RunOutcome, previousReport = false): { title: string; detail: string } {
  const previous = previousReport ? " Il report dell'analisi precedente resta in Risultati: riguarda quell'analisi, non questa." : "";
  if (outcome.kind === "technical_failure") return { title: "Errore tecnico",
    detail: `L'analisi non è stata completata e non ha prodotto un report. Controlla che i file siano ancora disponibili e riprova; se il problema resta, chiudi e riapri Lintra.${previous}` };
  if (outcome.kind === "not_started") return { title: "Analisi annullata", detail: `L'analisi non è iniziata: nessun report è stato prodotto.${previous}` };
  const report = outcome.report_available ? "Il report è disponibile in Risultati." : `Nessun report completo pubblicato per questa analisi.${previous}`;
  if (outcome.status === "cancelled") return { title: "Analisi annullata", detail: `I controlli sono stati interrotti. ${report}` };
  // Truncated (operational limit): never a conformity, whatever the status.
  if (outcome.truncated) return { title: "Analisi parziale", detail: `Raggiunto il limite operativo. ${report}` };
  const partial = outcome.completeness === "partial";
  if (outcome.status === "completed_invalid") return { title: "Violazioni rilevate",
    detail: `${partial ? "Alcuni controlli sono incompleti. " : ""}${report}` };
  if (partial) return { title: "Controlli incompleti", detail: `Non sono state rilevate violazioni, ma non tutti i controlli sono stati completati. ${report}` };
  return { title: "Nessuna violazione nei controlli eseguiti", detail: report };
}

/** IPC operations the run screen needs; injected so tests can replace them. */
export type RunApi = {
  resourceBudgetState: () => Promise<BudgetConfigState>;
  planPrepare: (planRef: string) => Promise<{ plan_ref: string; mode: "local_only" | "full" }>;
  validationStart: (planRef: string, mode: "local_only") => Promise<RunRef>;
  validationState: (run: RunRef | null) => Promise<RunSnapshot>;
  validationProgress: (run: RunRef) => Promise<RunProgress>;
  validationBudget: (run: RunRef) => Promise<BudgetGate>;
  validationBudgetDecide: (run: RunRef, confirmationRef: string, decision: "proceed" | "cancel") => Promise<"proceeding" | "cancelled">;
  validationAcknowledge: (run: RunRef) => Promise<void>;
  validationCancel: (run: RunRef) => Promise<"requested" | "already_requested" | "too_late" | "already_terminal">;
};
type Dispatch = (event: RunEvent) => void;

export async function loadBudget(api: RunApi, dispatch: Dispatch): Promise<void> {
  try { dispatch({ type: "budget_loaded", budget: await api.resourceBudgetState() }); }
  catch { dispatch({ type: "budget_failed" }); }
}

/** Freezes the plan in Rust and starts it. The budget is read again first,
 * so a file corrected, removed or broken meanwhile decides this start. */
export async function startRun(api: RunApi, dispatch: Dispatch, token: number, planRef: string, previous: RunRef | null): Promise<void> {
  try {
    const budget = await api.resourceBudgetState();
    dispatch({ type: "budget_loaded", budget });
    if (budget.state === "invalid") return dispatch({ type: "start_failed", token, message: BUDGET_INVALID });
    const prepared = await api.planPrepare(planRef);
    if (prepared.plan_ref !== planRef || prepared.mode !== "local_only") throw new Error("prepared another plan");
    dispatch({ type: "started", token, run: await api.validationStart(planRef, "local_only") });
  } catch {
    // The reply may be lost after Rust accepted the start (LIFECYCLE.md):
    // read the current run instead of repeating a non-idempotent start, and
    // attach only to a run of this plan that is not the previous one.
    try {
      const current = await api.validationState(null);
      if (current.state !== "idle" && current.plan_ref === planRef && current.mode === "local_only"
          && (previous === null || !same(previous, current.run))) {
        return dispatch({ type: "started", token, run: current.run });
      }
    } catch { /* no authoritative state: report the failure */ }
    dispatch({ type: "start_failed", token, message: START_FAILED });
    void loadBudget(api, dispatch);
  }
}

/** One throttled read of the run: state first (the terminal is authoritative
 * and never waits for progress), then progress and, while undecided, the
 * budget gate. Returns false once the terminal has been shown. */
export async function pollRun(api: RunApi, dispatch: Dispatch, run: RunRef, gateOpen: boolean): Promise<boolean> {
  const snapshot = await api.validationState(run);
  if (snapshot.state === "terminal" && snapshot.outcome !== null) {
    // The terminal is shown whatever happens to the accessory reads: the last
    // progress sample (final count from TraceCompleted) and the gate are best
    // effort, because a sealed or poisoned channel must not hide it (P505-03).
    try { dispatch({ type: "progress", progress: await api.validationProgress(run) }); } catch { /* terminal still shown */ }
    if (gateOpen) try { dispatch({ type: "gate", gate: await api.validationBudget(run) }); } catch { /* terminal still shown */ }
    let acknowledged = true;
    try { await api.validationAcknowledge(run); } catch { acknowledged = false; }
    dispatch({ type: "terminal", run, outcome: snapshot.outcome, acknowledged });
    return false;
  }
  dispatch({ type: "snapshot", snapshot });
  dispatch({ type: "progress", progress: await api.validationProgress(run) });
  if (gateOpen) dispatch({ type: "gate", gate: await api.validationBudget(run) });
  return true;
}

/** Whether the gate can still change (read it again on the next poll). */
export function gateOpen(gate: BudgetGate | null): boolean {
  return gate === null || gate.state === "checking" || gate.state === "confirmation_required";
}

/** Annulla / Procedi comunque for the exact pending preflight. A refused
 * (stale or duplicate) handle changes nothing: the next poll shows the truth. */
export async function decide(api: RunApi, dispatch: Dispatch, gate: BudgetGate, decision: "proceed" | "cancel"): Promise<void> {
  if (gate.state !== "confirmation_required" || gate.confirmation_ref === null) return;
  dispatch({ type: "decide" });
  try {
    await api.validationBudgetDecide(gate.run, gate.confirmation_ref, decision);
    dispatch({ type: "gate", gate: await api.validationBudget(gate.run) });
  } catch {
    dispatch({ type: "decide_failed" });
  }
}

/** Retry of a failed terminal ack (idempotent in Rust); the outcome shown
 * does not change, only a new start waits for it. */
export async function acknowledge(api: RunApi, dispatch: Dispatch, run: RunRef): Promise<boolean> {
  try { await api.validationAcknowledge(run); dispatch({ type: "acknowledged", run }); return true; } catch { return false; }
}

/** Operator cancel: one request; Rust's disposition is shown, the terminal
 * read later is the only proof of what happened (LIFECYCLE.md). */
export async function cancelRun(api: RunApi, dispatch: Dispatch, run: RunRef): Promise<void> {
  dispatch({ type: "cancel" });
  try { dispatch({ type: "cancel_answer", run, disposition: await api.validationCancel(run) }); }
  catch { dispatch({ type: "cancel_failed", run }); }
}
