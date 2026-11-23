// Analysis selection state machine (T-MIG-P5-02): flow -> explicit schema
// version -> trace -> LocalOnly plan draft. Mirrors the Slint
// AnalysisSelection contract (ui/tests/analysis_configuration.rs,
// schema_binding.rs) without any normative logic: the catalog, the captured
// schema and the trace list come from Rust, and Rust alone validates the
// selection when it creates the plan. Full mode and its per-trace inputs
// belong to T-MIG-P5-09.
//
// T-MIG-P5-03: the input file is chosen in the native dialog owned by Rust
// (plan_input_choose). The renderer never sees a path, a name or a byte: it
// receives the new plan reference and the bounded preview counters only.
//
// T-MIG-P5-04: the requested report formats, as the Slint ReportSelection.
// They are an intent only: nothing is written or converted here; the export
// after the run passes one format to Rust (export_prepare), which owns the
// canonical JSONL/CSV/XLSX writers.
import type { BundleProvenance, ConfigurationSelection, ExportFormat, InputPreviewDto, PlanDraft, SchemaDescriptor } from "./ipc";

export type TraceOption = { trace_ref: string; trace_id: string };

/** A request in flight is identified by a token; late replies are ignored. */
type Pending = { token: number };

/** Input of the plan's trace. `chosen` survives a cancelled dialog or a
 * failed new choice, exactly like the binding kept by Rust. */
export type InputState = {
  chosen: InputPreviewDto | null;
  activity: { status: "idle" } | ({ status: "choosing" } & Pending) | { status: "failed"; message: string } | { status: "cancelled" };
};

/** Report formats requested with the analysis (Slint `ReportSelection`). */
export type ReportSelection = Record<ExportFormat, boolean>;

/** Slint `ReportSelection::default()`: JSONL and XLSX, not CSV. */
export const DEFAULT_REPORTS: ReportSelection = { xlsx: true, jsonl: true, csv: false };

/** Order and labels of the Slint form (XLSX, JSONL, CSV). */
export const REPORT_FORMATS: readonly { id: ExportFormat; label: string }[] = [
  { id: "xlsx", label: "XLSX" }, { id: "jsonl", label: "JSONL" }, { id: "csv", label: "CSV" },
];

/** Format proposed when the report is saved, as Slint `preferred_format`:
 * XLSX, then CSV, then JSONL (also when nothing is requested). */
export function preferredFormat(reports: ReportSelection): ExportFormat {
  return reports.xlsx ? "xlsx" : reports.csv ? "csv" : "jsonl";
}

/** Origin of the configuration data, as shown by the form and by Informazioni. */
export function provenanceText(provenance: Pick<BundleProvenance, "bundle_source" | "bundle_version">): string {
  const source = provenance.bundle_source === "embedded" ? "schemi incorporati" : "schemi esterni";
  return provenance.bundle_version === null ? `${source}, versione non dichiarata` : `${source}, versione ${provenance.bundle_version}`;
}

export type SelectionState = {
  catalog: { status: "loading" } | { status: "ready"; entries: SchemaDescriptor[]; provenance: BundleProvenance } | { status: "failed"; message: string };
  flow: string | null;
  schemaRef: string | null;
  configuration: { status: "idle" } | ({ status: "loading" } & Pending)
    | { status: "ready"; selection: ConfigurationSelection; traces: TraceOption[] } | { status: "failed"; message: string };
  traceRef: string | null;
  plan: { status: "idle" } | ({ status: "creating" } & Pending) | { status: "ready"; draft: PlanDraft; input: InputState } | { status: "failed"; message: string };
  /** Independent of flow/schema/trace: changing them keeps it, as in Slint. */
  reports: ReportSelection;
  nextToken: number;
};

export type SelectionEvent =
  | { type: "catalog_loaded"; entries: SchemaDescriptor[]; provenance: BundleProvenance }
  | { type: "catalog_failed"; message: string }
  | { type: "select_flow"; flow: string | null }
  | { type: "select_version"; schemaRef: string | null }
  | { type: "configuration_loaded"; token: number; selection: ConfigurationSelection; traces: TraceOption[] }
  | { type: "configuration_failed"; token: number; message: string }
  | { type: "select_trace"; traceRef: string | null }
  | { type: "plan_created"; token: number; draft: PlanDraft }
  | { type: "plan_failed"; token: number; message: string }
  | { type: "choose_input" }
  | { type: "input_chosen"; token: number; planRef: string; assignedInputCount: string; preview: InputPreviewDto }
  | { type: "input_cancelled"; token: number }
  | { type: "input_failed"; token: number; message: string }
  | { type: "set_report"; format: ExportFormat; requested: boolean };

export const INITIAL_SELECTION: SelectionState = {
  catalog: { status: "loading" }, flow: null, schemaRef: null,
  configuration: { status: "idle" }, traceRef: null, plan: { status: "idle" }, reports: DEFAULT_REPORTS, nextToken: 1,
};

/** Flows in catalog order, each once. No flow is preselected. */
export function flows(state: SelectionState): string[] {
  if (state.catalog.status !== "ready") return [];
  return [...new Set(state.catalog.entries.map(entry => entry.flow_id))];
}

/** Versions of the chosen flow in catalog order; never a "latest" default. */
export function versions(state: SelectionState): SchemaDescriptor[] {
  if (state.catalog.status !== "ready" || state.flow === null) return [];
  return state.catalog.entries.filter(entry => entry.flow_id === state.flow);
}

/** Pure transition. Choosing a parent clears every dependent choice, exactly
 * as AnalysisSelection::select_flow/select_version do in the Slint UI. */
export function reduce(state: SelectionState, event: SelectionEvent): SelectionState {
  switch (event.type) {
    case "catalog_loaded":
      return { ...INITIAL_SELECTION, catalog: { status: "ready", entries: event.entries, provenance: event.provenance }, reports: state.reports, nextToken: state.nextToken };
    case "catalog_failed":
      return { ...INITIAL_SELECTION, catalog: { status: "failed", message: event.message }, reports: state.reports, nextToken: state.nextToken };
    case "select_flow": {
      if (event.flow !== null && !flows(state).includes(event.flow)) return state;
      return { ...state, flow: event.flow, schemaRef: null, configuration: { status: "idle" }, traceRef: null, plan: { status: "idle" } };
    }
    case "select_version": {
      if (event.schemaRef !== null && !versions(state).some(entry => entry.schema_ref === event.schemaRef)) return state;
      const configuration = event.schemaRef === null ? { status: "idle" as const } : { status: "loading" as const, token: state.nextToken };
      return { ...state, schemaRef: event.schemaRef, configuration, traceRef: null, plan: { status: "idle" },
        nextToken: state.nextToken + (event.schemaRef === null ? 0 : 1) };
    }
    case "configuration_loaded":
    case "configuration_failed": {
      if (state.configuration.status !== "loading" || state.configuration.token !== event.token) return state;
      if (event.type === "configuration_failed") return { ...state, configuration: { status: "failed", message: event.message } };
      return { ...state, configuration: { status: "ready", selection: event.selection, traces: event.traces } };
    }
    case "select_trace": {
      if (state.configuration.status !== "ready") return state;
      if (event.traceRef !== null && !state.configuration.traces.some(trace => trace.trace_ref === event.traceRef)) return state;
      const plan = event.traceRef === null ? { status: "idle" as const } : { status: "creating" as const, token: state.nextToken };
      return { ...state, traceRef: event.traceRef, plan, nextToken: state.nextToken + (event.traceRef === null ? 0 : 1) };
    }
    case "plan_created":
    case "plan_failed": {
      if (state.plan.status !== "creating" || state.plan.token !== event.token) return state;
      if (event.type === "plan_failed") return { ...state, plan: { status: "failed", message: event.message } };
      return { ...state, plan: { status: "ready", draft: event.draft, input: { chosen: null, activity: { status: "idle" } } } };
    }
    case "choose_input": {
      // One dialog at a time, only for a plan Rust has accepted.
      if (state.plan.status !== "ready" || state.plan.input.activity.status === "choosing") return state;
      const input = { ...state.plan.input, activity: { status: "choosing" as const, token: state.nextToken } };
      return { ...state, plan: { ...state.plan, input }, nextToken: state.nextToken + 1 };
    }
    case "input_chosen":
    case "input_cancelled":
    case "input_failed": {
      if (state.plan.status !== "ready") return state;
      const { input } = state.plan;
      if (input.activity.status !== "choosing" || input.activity.token !== event.token) return state;
      if (event.type === "input_cancelled") return { ...state, plan: { ...state.plan, input: { ...input, activity: { status: "cancelled" } } } };
      if (event.type === "input_failed") return { ...state, plan: { ...state.plan, input: { ...input, activity: { status: "failed", message: event.message } } } };
      // Every accepted input gives the plan a new reference in Rust.
      const draft = { ...state.plan.draft, plan_ref: event.planRef, assigned_input_count: event.assignedInputCount };
      return { ...state, plan: { status: "ready", draft, input: { chosen: event.preview, activity: { status: "idle" } } } };
    }
    case "set_report":
      // Unknown formats (not from the typed form) are ignored. Every
      // combination is allowed, none included, as in Slint.
      if (!REPORT_FORMATS.some(format => format.id === event.format)) return state;
      return { ...state, reports: { ...state.reports, [event.format]: event.requested } };
  }
}

/** What the operator still has to choose, in the order of the form. */
export function missingChoice(state: SelectionState): string | null {
  if (state.catalog.status !== "ready") return null;
  if (state.flow === null) return "Scegli il flusso da analizzare.";
  if (state.schemaRef === null) return "Scegli la versione dello schema: nessuna versione viene scelta in automatico.";
  if (state.configuration.status !== "ready") return null;
  if (state.traceRef === null) return "Scegli il tracciato da analizzare.";
  if (state.plan.status === "ready" && state.plan.input.chosen === null) return "Scegli il file del tracciato.";
  return null;
}

export const SELECTION_FAILED = "Non è stato possibile caricare lo schema scelto. Scegli di nuovo la versione; se il problema resta, chiudi e riapri Lintra.";
export const PLAN_FAILED = "Il servizio non ha accettato la selezione. Scegli di nuovo il tracciato; se il problema resta, chiudi e riapri Lintra.";
export const CATALOG_FAILED = "Non è stato possibile leggere il catalogo degli schemi. Chiudi e riapri Lintra.";
// The form completes this with what is still assigned (P503-01).
export const INPUT_FAILED = "Non è stato possibile usare il file scelto: potrebbe essere illeggibile o iniziare con un BOM non ammesso.";

/** IPC operations the controller needs; injected so tests can replace them. */
export type SelectionApi = {
  catalogPage: (offset?: string) => Promise<{ entries: SchemaDescriptor[]; next_offset: string | null; provenance: BundleProvenance }>;
  configurationSelect: (schemaRef: string, encoding: "utf8") => Promise<ConfigurationSelection>;
  configurationPage: (configurationRef: string, offset?: string) => Promise<{ descriptors: ({ kind: "trace"; trace_ref: string; trace_id: string } | { kind: "parameter" })[]; next_offset: string | null }>;
  planCreate: (configurationRef: string, mode: "local_only", traceRef: string) => Promise<PlanDraft>;
  planInputChoose: (planRef: string, traceRef: string) => Promise<{ plan_ref: string; trace_ref: string; disposition: "selected" | "cancelled"; assigned_input_count: string; preview: InputPreviewDto | null }>;
};
type Dispatch = (event: SelectionEvent) => void;

/** Reads every catalog page. Pages are bounded by Rust (64 rows); the
 * provenance must stay the same across pages of one catalog. */
export async function loadCatalog(api: SelectionApi, dispatch: Dispatch): Promise<void> {
  try {
    const entries: SchemaDescriptor[] = [];
    let offset: string | null = "0";
    let provenance: BundleProvenance | null = null;
    while (offset !== null) {
      const page = await api.catalogPage(offset);
      if (provenance !== null && JSON.stringify(provenance) !== JSON.stringify(page.provenance)) throw new Error("catalog changed while paging");
      provenance = page.provenance;
      entries.push(...page.entries);
      offset = page.next_offset;
    }
    if (provenance === null) throw new Error("empty catalog reply");
    dispatch({ type: "catalog_loaded", entries, provenance });
  } catch {
    dispatch({ type: "catalog_failed", message: CATALOG_FAILED });
  }
}

/** Captures the chosen schema in Rust and lists its traces. The schema shown
 * afterwards is the one Rust captured (schema_binding parity): a reply for a
 * different flow/version, or a trace list that disagrees with the declared
 * count, is a technical failure rather than a silent substitution. */
export async function loadConfiguration(api: SelectionApi, dispatch: Dispatch, token: number, chosen: SchemaDescriptor): Promise<void> {
  try {
    const selection = await api.configurationSelect(chosen.schema_ref, "utf8");
    if (selection.flow_id !== chosen.flow_id || selection.schema_version !== chosen.schema_version) throw new Error("captured schema differs from the choice");
    const traces: TraceOption[] = [];
    let offset: string | null = "0";
    while (offset !== null) {
      const page = await api.configurationPage(selection.configuration_ref, offset);
      for (const row of page.descriptors) if (row.kind === "trace") traces.push({ trace_ref: row.trace_ref, trace_id: row.trace_id });
      offset = page.next_offset;
    }
    if (BigInt(traces.length) !== BigInt(selection.trace_count)) throw new Error("trace count mismatch");
    dispatch({ type: "configuration_loaded", token, selection, traces });
  } catch {
    dispatch({ type: "configuration_failed", token, message: SELECTION_FAILED });
  }
}

/** Asks Rust for a LocalOnly plan bound to the captured configuration and
 * trace; Rust rejects unknown or stale references. */
export async function createPlan(api: SelectionApi, dispatch: Dispatch, token: number, configurationRef: string, traceRef: string): Promise<void> {
  try {
    const draft = await api.planCreate(configurationRef, "local_only", traceRef);
    if (draft.configuration_ref !== configurationRef || draft.mode !== "local_only") throw new Error("plan for another selection");
    dispatch({ type: "plan_created", token, draft });
  } catch {
    dispatch({ type: "plan_failed", token, message: PLAN_FAILED });
  }
}

/** Opens the native dialog through Rust for the plan's trace. A cancelled
 * dialog keeps the plan reference; an accepted file returns a new one. */
export async function chooseInput(api: SelectionApi, dispatch: Dispatch, token: number, planRef: string, traceRef: string): Promise<void> {
  try {
    const choice = await api.planInputChoose(planRef, traceRef);
    if (choice.trace_ref !== traceRef) throw new Error("choice for another trace");
    if (choice.disposition === "cancelled") dispatch({ type: "input_cancelled", token });
    else if (choice.preview !== null && choice.plan_ref !== planRef) dispatch({ type: "input_chosen", token, planRef: choice.plan_ref, assignedInputCount: choice.assigned_input_count, preview: choice.preview });
    else throw new Error("selected input without a new plan or preview");
  } catch {
    dispatch({ type: "input_failed", token, message: INPUT_FAILED });
  }
}
