import { useEffect, useReducer } from "react";
import { catalogPage, configurationPage, configurationSelect, planCreate, planInputChoose, type InputPreviewDto } from "./ipc";
import {
  INITIAL_SELECTION, chooseInput, createPlan, flows, loadCatalog, loadConfiguration, missingChoice, reduce, versions,
  type InputState, type SelectionApi, type SelectionEvent, type SelectionState,
} from "./analysis";

const api: SelectionApi = { catalogPage, configurationSelect, configurationPage, planCreate, planInputChoose };

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
    if (planToken === null || state.configuration.status !== "ready" || state.traceRef === null) return;
    void createPlan(api, dispatch, planToken, state.configuration.selection.configuration_ref, state.traceRef);
  }, [planToken]);
  const inputToken = state.plan.status === "ready" && state.plan.input.activity.status === "choosing" ? state.plan.input.activity.token : null;
  useEffect(() => {
    if (inputToken === null || state.plan.status !== "ready" || state.traceRef === null) return;
    void chooseInput(api, dispatch, inputToken, state.plan.draft.plan_ref, state.traceRef);
  }, [inputToken]);
  return [state, dispatch];
}

/** Presentational form: every choice and status comes from the state. */
export function AnalysisForm({ state, onEvent }: { state: SelectionState; onEvent: (event: SelectionEvent) => void }) {
  const catalogReady = state.catalog.status === "ready";
  const configuration = state.configuration.status === "ready" ? state.configuration : null;
  const trace = configuration?.traces.find(item => item.trace_ref === state.traceRef) ?? null;
  return <section className="selection" aria-labelledby="selection-title">
    <h2 id="selection-title">Schema e tracciato</h2>
    <div className="fields">
      <div className="field">
        <label htmlFor="selection-flow">Flusso</label>
        <select id="selection-flow" value={state.flow ?? ""} disabled={!catalogReady}
          onChange={event => onEvent({ type: "select_flow", flow: event.target.value || null })}>
          <option value="">Scegli il flusso</option>
          {flows(state).map(flow => <option key={flow} value={flow}>{flow}</option>)}
        </select>
      </div>
      <div className="field">
        <label htmlFor="selection-version">Versione dello schema</label>
        <select id="selection-version" value={state.schemaRef ?? ""} disabled={state.flow === null}
          onChange={event => onEvent({ type: "select_version", schemaRef: event.target.value || null })}>
          <option value="">Scegli la versione</option>
          {versions(state).map(entry => <option key={entry.schema_ref} value={entry.schema_ref}>{entry.schema_version}</option>)}
        </select>
      </div>
      <div className="field">
        <label htmlFor="selection-trace">Tracciato</label>
        <select id="selection-trace" value={state.traceRef ?? ""} disabled={configuration === null}
          onChange={event => onEvent({ type: "select_trace", traceRef: event.target.value || null })}>
          <option value="">Scegli il tracciato</option>
          {configuration?.traces.map(item => <option key={item.trace_ref} value={item.trace_ref}>{item.trace_id}</option>)}
        </select>
      </div>
    </div>
    <SelectionStatus state={state} />
    {state.plan.status === "ready" && <InputChoice input={state.plan.input} onChoose={() => onEvent({ type: "choose_input" })} />}
    {configuration && <dl className="selection-summary" aria-label="Schema acquisito dal servizio">
      <dt>Schema</dt><dd>{configuration.selection.flow_id} · {configuration.selection.schema_version}</dd>
      <dt>Codifica</dt><dd>UTF-8</dd>
      <dt>Impronta</dt><dd>{configuration.selection.schema_hash.slice(0, 16)}</dd>
      <dt>Origine</dt><dd>{provenanceText(configuration.selection.provenance)}</dd>
      {trace && <><dt>Tracciato</dt><dd>{trace.trace_id}</dd></>}
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

function provenanceText(provenance: { bundle_source: "embedded" | "external"; bundle_version: string | null }): string {
  const source = provenance.bundle_source === "embedded" ? "schemi incorporati" : "schemi esterni";
  return provenance.bundle_version === null ? `${source}, versione non dichiarata` : `${source}, versione ${provenance.bundle_version}`;
}

/** Input file of the trace. The dialog is native and owned by Rust; only the
 * bounded preview counters come back, and they describe a sample. */
function InputChoice({ input, onChoose }: { input: InputState; onChoose: () => void }) {
  const choosing = input.activity.status === "choosing";
  return <div className="input-choice">
    <h3>File del tracciato</h3>
    <button type="button" onClick={onChoose} disabled={choosing}>
      {input.chosen ? "Scegli un altro file" : "Scegli il file"}
    </button>
    {input.activity.status === "failed" ? <p className="selection-status" role="alert">{input.activity.message}</p>
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
