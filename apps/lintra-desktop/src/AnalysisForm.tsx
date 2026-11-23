import { useEffect, useReducer } from "react";
import { catalogPage, configurationPage, configurationSelect, planCreate, planInputChoose, planParameterSet, type InputPreviewDto } from "./ipc";
import {
  INITIAL_SELECTION, REPORT_FORMATS, chooseInput, createPlan, flows, loadCatalog, loadConfiguration, missingChoice, provenanceText, reduce, setParameter, versions,
  type InputState, type ParameterOption, type ParameterState, type ReportSelection, type SelectionApi, type SelectionEvent, type SelectionState, type TraceOption,
} from "./analysis";

const api: SelectionApi = { catalogPage, configurationSelect, configurationPage, planCreate, planInputChoose, planParameterSet };

/** Owns the selection state for the whole window lifetime (it survives
 * switching sections) and runs the Rust requests each state asks for. The
 * catalog is read once the service is ready. Late replies are discarded by
 * the reducer's tokens. */
export function useAnalysisSelection(serviceReady: boolean): [SelectionState, (event: SelectionEvent) => void] {
  const [state, dispatch] = useReducer(reduce, INITIAL_SELECTION);
  useEffect(() => { if (serviceReady) void loadCatalog(api, dispatch); }, [serviceReady]);
  const configurationToken = state.configuration.status === "loading" ? state.configuration.token : null;
  useEffect(() => {
    if (configurationToken === null) return;
    const chosen = versions(state).find(entry => entry.schema_ref === state.schemaRef);
    if (chosen) void loadConfiguration(api, dispatch, configurationToken, chosen);
    // Keyed by the token only: each token is one request, so re-running on
    // other state changes would duplicate it.
  }, [configurationToken]);
  const planToken = state.plan.status === "creating" ? state.plan.token : null;
  useEffect(() => {
    if (planToken === null || state.configuration.status !== "ready") return;
    if (state.mode === "local_only" && state.traceRef === null) return;
    void createPlan(api, dispatch, planToken, state.configuration.selection.configuration_ref, state.mode, state.mode === "full" ? null : state.traceRef);
  }, [planToken]);
  // At most one dialog or parameter request at a time (reducer `mutating`).
  const choosing = state.plan.status === "ready"
    ? Object.entries(state.plan.inputs).find(([, input]) => input.activity.status === "choosing") : undefined;
  const inputToken = choosing && choosing[1].activity.status === "choosing" ? choosing[1].activity.token : null;
  useEffect(() => {
    if (inputToken === null || state.plan.status !== "ready" || !choosing) return;
    void chooseInput(api, dispatch, inputToken, state.plan.draft.plan_ref, choosing[0]);
  }, [inputToken]);
  const setting = state.plan.status === "ready"
    ? Object.entries(state.plan.parameters).find(([, parameter]) => parameter.activity.status === "setting") : undefined;
  const parameterToken = setting && setting[1].activity.status === "setting" ? setting[1].activity.token : null;
  useEffect(() => {
    if (parameterToken === null || state.plan.status !== "ready" || state.configuration.status !== "ready" || !setting) return;
    const option = state.configuration.parameters.find(item => item.parameter_ref === setting[0]);
    const activity = setting[1].activity;
    if (option && activity.status === "setting") void setParameter(api, dispatch, parameterToken, state.plan.draft.plan_ref, option, activity.text);
  }, [parameterToken]);
  return [state, dispatch];
}

/** Presentational form: every choice and status comes from the state. */
export function AnalysisForm({ state, onEvent, busy = false }: { state: SelectionState; onEvent: (event: SelectionEvent) => void; busy?: boolean }) {
  const catalogReady = state.catalog.status === "ready";
  const configuration = state.configuration.status === "ready" ? state.configuration : null;
  const trace = configuration?.traces.find(item => item.trace_ref === state.traceRef) ?? null;
  const full = state.mode === "full";
  const plan = state.plan.status === "ready" ? state.plan : null;
  return <section className="selection" aria-labelledby="selection-title">
    <h2 id="selection-title">Schema e tracciato</h2>
    <div className="fields">
      <div className="field">
        <label htmlFor="selection-flow">Flusso</label>
        <select id="selection-flow" value={state.flow ?? ""} disabled={!catalogReady || busy}
          onChange={event => onEvent({ type: "select_flow", flow: event.target.value || null })}>
          <option value="">Scegli il flusso</option>
          {flows(state).map(flow => <option key={flow} value={flow}>{flow}</option>)}
        </select>
      </div>
      <div className="field">
        <label htmlFor="selection-version">Versione dello schema</label>
        <select id="selection-version" value={state.schemaRef ?? ""} disabled={state.flow === null || busy}
          onChange={event => onEvent({ type: "select_version", schemaRef: event.target.value || null })}>
          <option value="">Scegli la versione</option>
          {versions(state).map(entry => <option key={entry.schema_ref} value={entry.schema_ref}>{entry.schema_version}</option>)}
        </select>
      </div>
    </div>
    {/* Explicit mode (T-MIG-P5-09): native radios, one Tab stop, arrows move. */}
    <fieldset className="analysis-mode" disabled={!catalogReady || busy}>
      <legend>Tipo di analisi</legend>
      <label><input type="radio" name="analysis-mode" value="local_only" checked={!full}
        onChange={() => onEvent({ type: "select_mode", mode: "local_only" })} />Singolo tracciato</label>
      <label><input type="radio" name="analysis-mode" value="full" checked={full}
        onChange={() => onEvent({ type: "select_mode", mode: "full" })} />Flusso completo (Full)</label>
    </fieldset>
    {!full && <div className="fields">
      <div className="field">
        <label htmlFor="selection-trace">Tracciato</label>
        <select id="selection-trace" value={state.traceRef ?? ""} disabled={configuration === null || busy}
          onChange={event => onEvent({ type: "select_trace", traceRef: event.target.value || null })}>
          <option value="">Scegli il tracciato</option>
          {configuration?.traces.map(item => <option key={item.trace_ref} value={item.trace_ref}>{item.trace_id}</option>)}
        </select>
      </div>
    </div>}
    <SelectionStatus state={state} />
    {/* While a run is active the form is frozen, as Slint `enabled: !root.busy`. */}
    {plan && !full && Object.values(plan.inputs).map(input => <InputChoice key="local" input={input} busy={busy}
      onChoose={() => { if (!busy) onEvent({ type: "choose_input" }); }} />)}
    {plan && full && configuration && <FullInputs traces={configuration.traces} plan={plan} busy={busy} onEvent={onEvent} />}
    {plan && full && configuration && configuration.parameters.length > 0 &&
      <FullParameters parameters={configuration.parameters} values={plan.parameters} busy={busy} onEvent={onEvent} />}
    <ReportFormats reports={state.reports} busy={busy} onEvent={onEvent} />
    {configuration && <dl className="selection-summary" aria-label="Schema acquisito dal servizio">
      <dt>Schema</dt><dd>{configuration.selection.flow_id} · {configuration.selection.schema_version}</dd>
      <dt>Codifica</dt><dd>UTF-8</dd>
      <dt>Impronta</dt><dd>{configuration.selection.schema_hash.slice(0, 16)}</dd>
      <dt>Origine</dt><dd>{provenanceText(configuration.selection.provenance)}</dd>
      <dt>Analisi</dt><dd>{full ? "Flusso completo (Full)" : "Singolo tracciato"}</dd>
      {!full && trace && <><dt>Tracciato</dt><dd>{trace.trace_id}</dd></>}
      {full && plan && <><dt>File assegnati</dt><dd>{plan.draft.assigned_input_count} di {configuration.traces.length}</dd></>}
    </dl>}
  </section>;
}

function SelectionStatus({ state }: { state: SelectionState }) {
  const failure = state.catalog.status === "failed" ? state.catalog.message
    : state.configuration.status === "failed" ? state.configuration.message
      : state.plan.status === "failed" ? state.plan.message : null;
  if (failure) return <p className="selection-status" role="alert">{failure}</p>;
  const text = state.catalog.status === "loading" ? "Lettura del catalogo degli schemi…"
    : state.configuration.status === "loading" ? "Il servizio acquisisce lo schema scelto…"
      : state.plan.status === "creating" ? "Il servizio registra la selezione…"
        : missingChoice(state) ?? (state.plan.status === "ready" ? "Selezione accettata dal servizio." : null);
  return <p className="selection-status" role="status">{text}</p>;
}

/** Full: one native file choice per trace of the captured schema, in schema
 * order. No trace is marked required or optional here: Rust decides it when
 * the plan is prepared. One dialog at a time (reducer `mutating`). */
function FullInputs({ traces, plan, busy, onEvent }: { traces: TraceOption[]; plan: Extract<SelectionState["plan"], { status: "ready" }>; busy: boolean; onEvent: (event: SelectionEvent) => void }) {
  return <section className="full-inputs" aria-labelledby="full-inputs-title">
    <h3 id="full-inputs-title">File dei tracciati</h3>
    <p className="sample-note">Assegna un file a ogni tracciato da analizzare. Il servizio verifica all’avvio che ci siano tutti i tracciati richiesti dallo schema.</p>
    {traces.map(trace => {
      const input = plan.inputs[trace.trace_ref];
      return input && <InputChoice key={trace.trace_ref} input={input} busy={busy} trace={trace.trace_id}
        onChoose={() => { if (!busy) onEvent({ type: "choose_input", traceRef: trace.trace_ref }); }} />;
    })}
  </section>;
}

/** Full: the parameters the schema declares, typed as declared. A value is
 * sent when the field is left or Invio is pressed; Rust says if it is
 * accepted. The message is tied to the field (aria-describedby). */
function FullParameters({ parameters, values, busy, onEvent }: { parameters: ParameterOption[]; values: Record<string, ParameterState>; busy: boolean; onEvent: (event: SelectionEvent) => void }) {
  return <fieldset className="full-parameters" disabled={busy}>
    <legend>Parametri del flusso</legend>
    {parameters.map(option => {
      const value = values[option.parameter_ref];
      if (!value) return null;
      const id = `parameter-${option.parameter_ref}`;
      const commit = () => onEvent({ type: "commit_parameter", parameterRef: option.parameter_ref });
      return <div className="field" key={option.parameter_ref}>
        <label htmlFor={id}>{option.parameter_id} ({option.value_kind === "integer" ? "numero intero" : "testo"})</label>
        <input id={id} type="text" value={value.text} spellCheck={false} autoComplete="off"
          inputMode={option.value_kind === "integer" ? "numeric" : "text"}
          aria-invalid={value.activity.status === "failed"} aria-describedby={`${id}-status`}
          readOnly={value.activity.status === "setting"}
          onChange={event => onEvent({ type: "edit_parameter", parameterRef: option.parameter_ref, text: event.target.value })}
          onBlur={commit} onKeyDown={event => { if (event.key === "Enter") commit(); }} />
        <ParameterStatus id={`${id}-status`} value={value} />
      </div>;
    })}
  </fieldset>;
}

function ParameterStatus({ id, value }: { id: string; value: ParameterState }) {
  const held = value.applied === null ? "nessun valore applicato." : `resta applicato «${value.applied}».`;
  if (value.activity.status === "failed") return <p id={id} className="selection-status" role="alert">{value.activity.message} Per il servizio {held}</p>;
  const text = value.activity.status === "setting" ? "Il servizio controlla il valore…"
    : value.applied === null && value.text === "" ? "Nessun valore applicato."
      : value.text === (value.applied ?? "") ? "Valore applicato dal servizio."
        : `Valore non ancora applicato: premi Invio o esci dal campo. Per il servizio ${held}`;
  return <p id={id} className="selection-status" role="status">{text}</p>;
}

/** Input file of the trace. The dialog is native and owned by Rust; only the
 * bounded preview counters come back, and they describe a sample. */
function InputChoice({ input, busy, onChoose, trace }: { input: InputState; busy: boolean; onChoose: () => void; trace?: string }) {
  const choosing = input.activity.status === "choosing";
  return <div className="input-choice">
    <h3>{trace === undefined ? "File del tracciato" : `Tracciato ${trace}`}</h3>
    {/* aria-disabled, not disabled: a disabled button loses keyboard focus
        while the native dialog is open, so after Esc the operator would have
        to navigate back to it (P503-02). Double opening is refused by the
        reducer while a dialog is open. */}
    <button type="button" onClick={onChoose} aria-disabled={choosing || busy}
      aria-label={trace === undefined ? undefined : `${input.chosen ? "Scegli un altro file" : "Scegli il file"} per il tracciato ${trace}`}>
      {input.chosen ? "Scegli un altro file" : "Scegli il file"}
    </button>
    {input.activity.status === "failed" ? <p className="selection-status" role="alert">{input.activity.message} {input.chosen
      ? "Resta assegnato il file scelto prima; puoi sceglierne un altro."
      : "Nessun file è assegnato al tracciato; scegline un altro."}</p>
      : <p className="selection-status" role="status">{choosing ? "Finestra di scelta del file aperta…"
        : input.activity.status === "cancelled" ? (input.chosen ? "Scelta annullata: resta il file scelto prima." : "Scelta annullata: nessun file assegnato.")
          : input.chosen ? "File assegnato al tracciato." : "Nessun file assegnato."}</p>}
    {input.chosen && <SamplePreview preview={input.chosen} />}
  </div>;
}

/** Counters of the bounded sample read by Rust (at most 64 KiB). They are
 * never presented as counts of the whole file unless the sample reached EOF. */
function SamplePreview({ preview }: { preview: InputPreviewDto }) {
  const bytes = (value: string) => `${BigInt(value).toLocaleString("it-IT")} byte`;
  return <>
    <dl className="selection-summary" aria-label="Anteprima del campione">
      <dt>Dimensione del file</dt><dd>{bytes(preview.file_size)}</dd>
      <dt>Campione letto</dt><dd>{bytes(preview.sampled_bytes)}</dd>
      <dt>Fine riga nel campione</dt><dd>LF {preview.lf} · CRLF {preview.crlf} · CR isolati {preview.lone_cr}</dd>
    </dl>
    <p className="sample-note">{preview.reached_eof
      ? "Il campione comprende l’intero file."
      : `Il campione comprende solo i primi ${bytes(preview.sampled_bytes)}: i conteggi non descrivono l’intero file.`}</p>
  </>;
}

/** Requested report formats (Slint "Formati del report"): native checkboxes,
 * keyboard and screen readers handled by the platform. An intent only. */
function ReportFormats({ reports, busy, onEvent }: { reports: ReportSelection; busy: boolean; onEvent: (event: SelectionEvent) => void }) {
  return <fieldset className="report-formats" aria-describedby="report-formats-note" disabled={busy}>
    <legend>Formati del report</legend>
    <div className="report-options">
      {REPORT_FORMATS.map(format => <label key={format.id}>
        <input type="checkbox" checked={reports[format.id]}
          onChange={event => onEvent({ type: "set_report", format: format.id, requested: event.target.checked })} />
        {format.label}
      </label>)}
    </div>
    <p id="report-formats-note">Nessun file viene scritto ora. Dopo l’analisi il report si salva un formato alla volta: viene proposto il primo scelto tra XLSX, CSV e JSONL, oppure JSONL se non ne scegli nessuno.</p>
  </fieldset>;
}
