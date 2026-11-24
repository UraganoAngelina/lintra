import { invoke } from "@tauri-apps/api/core";

const MAX_BYTES = 1_048_576;
const categories = {
  IPC_UNSUPPORTED_VERSION: "protocol",
  IPC_INVALID_MESSAGE: "protocol",
  IPC_PAYLOAD_TOO_LARGE: "protocol",
  IPC_UNKNOWN_OPERATION: "protocol",
  IPC_UNAUTHORIZED: "technical",
  IPC_STALE_CONTEXT: "technical",
  IPC_BUSY: "technical",
  IPC_INVALID_STATE: "technical",
  IPC_TECHNICAL_FAILURE: "technical",
} as const;
type ErrorCode = keyof typeof categories;
type RequestPayloads = {
  app_info: { schemaVersion: 1 };
  client_ready: { schemaVersion: 1 };
  catalog_page: { schemaVersion: 1; offset: string };
  configuration_select: { schemaVersion: 1; schema_ref: string; encoding: "utf8" };
  configuration_page: { schemaVersion: 1; configuration_ref: string; offset: string };
  plan_create: { schemaVersion: 1; configuration_ref: string; mode: "local_only" | "full"; trace_ref: string | null };
  plan_input_choose: { schemaVersion: 1; plan_ref: string; trace_ref: string };
  plan_parameter_set: { schemaVersion: 1; plan_ref: string; parameter_ref: string; value: { schemaVersion: 1; kind: "text" | "integer"; text: string } };
  plan_prepare: { schemaVersion: 1; plan_ref: string };
  validation_start: { schemaVersion: 1; plan_ref: string; mode: "local_only" | "full" };
  validation_state: { schemaVersion: 1; run: RunRef | null };
  validation_acknowledge: { schemaVersion: 1; run: RunRef };
  validation_cancel: { schemaVersion: 1; run: RunRef };
  validation_progress: { schemaVersion: 1; run: RunRef };
  resource_budget_state: { schemaVersion: 1 };
  validation_budget: { schemaVersion: 1; run: RunRef };
  validation_budget_decide: { schemaVersion: 1; run: RunRef; confirmation_ref: string; decision: "proceed" | "cancel" };
  results_query: { schemaVersion: 1; run: RunRef; filters: ResultsFilter };
  results_query_state: { schemaVersion: 1; query_ref: string };
  results_page: { schemaVersion: 1; query_ref: string; offset: string; limit: number };
  results_overview: { schemaVersion: 1; run: RunRef };
  results_select: { schemaVersion: 1; query_ref: string; diagnostic_ref: string };
  results_text: { schemaVersion: 1; selection_ref: string; component_ref: string; offset: string; limit: number };
  results_context: { schemaVersion: 1; selection_ref: string; offset: string; limit: number };
  results_sources: { schemaVersion: 1; selection_ref: string; offset: string; limit: number };
  results_domain: { schemaVersion: 1; selection_ref: string; source_ref: string; cursor_ref: string | null };
  results_domain_state: { schemaVersion: 1; selection_ref: string; read_ref: string };
  report_about: { schemaVersion: 1; run: RunRef };
  export_prepare: { schemaVersion: 1; report_ref: string; format: ExportFormat };
  export_execute: { schemaVersion: 1; export_ref: string };
  export_state: { schemaVersion: 1; export_ref: string };
  export_cancel: { schemaVersion: 1; export_ref: string };
};
type Operation = keyof RequestPayloads;
type ObjectDto = Record<string, unknown>;
export type AppInfo = { schemaVersion: 1; app_version: string; validation_available: false };

export class IpcError extends Error {
  readonly code: ErrorCode;
  constructor(code: ErrorCode) {
    super(code);
    this.code = code;
  }
}

function invalid(): never { throw new IpcError("IPC_INVALID_MESSAGE"); }

function object(value: unknown): ObjectDto {
  if (typeof value !== "object" || value === null || Array.isArray(value)) invalid();
  return value as ObjectDto;
}

function exact(dto: ObjectDto, keys: string[]): void {
  if (Object.keys(dto).length !== keys.length || !keys.every(key => Object.hasOwn(dto, key))) invalid();
}

function version(dto: ObjectDto): void {
  const value = dto.schemaVersion;
  if (typeof value !== "number" || !Number.isInteger(value) || value < 1 || value > 65535) invalid();
  if (value !== 1) throw new IpcError("IPC_UNSUPPORTED_VERSION");
}

// JSON.parse loses duplicate keys and integer spelling; inspect bounded text first.
function strictJson(raw: unknown): ObjectDto {
  if (!(raw instanceof ArrayBuffer)) invalid();
  if (raw.byteLength > MAX_BYTES) throw new IpcError("IPC_PAYLOAD_TOO_LARGE");
  let text: string;
  try { text = new TextDecoder("utf-8", { fatal: true, ignoreBOM: true }).decode(raw); }
  catch { return invalid(); }
  const stack: (Set<string> | null)[] = [];
  let index = 0;
  while (index < text.length) {
    const char = text[index];
    if (char === "{" || char === "[") {
      stack.push(char === "{" ? new Set() : null);
      if (stack.length > 32) invalid();
      index++;
    } else if (char === "}" || char === "]") {
      if (stack.length === 0) invalid();
      stack.pop();
      index++;
    } else if (char === '"') {
      const start = index++;
      let escaped = false;
      while (index < text.length) {
        const current = text[index++];
        if (escaped) escaped = false;
        else if (current === "\\") escaped = true;
        else if (current === '"') break;
      }
      let value: unknown;
      try { value = JSON.parse(text.slice(start, index)); } catch { return invalid(); }
      if (typeof value !== "string") invalid();
      for (let position = 0; position < value.length; position++) {
        const unit = value.charCodeAt(position);
        if (unit >= 0xd800 && unit <= 0xdbff) {
          const next = value.charCodeAt(++position);
          if (!(next >= 0xdc00 && next <= 0xdfff)) invalid();
        } else if (unit >= 0xdc00 && unit <= 0xdfff) invalid();
      }
      let next = index;
      while (/^[\t\r\n ]$/.test(text[next] ?? "")) next++;
      if (text[next] === ":") {
        const keys = stack[stack.length - 1];
        if (!keys || keys.has(value)) invalid();
        keys.add(value);
      }
    } else if (char !== undefined && /[-0-9]/.test(char)) {
      const start = index++;
      while (index < text.length && !/[\t\r\n ,\]}:{\["]/.test(text[index] ?? "")) index++;
      const token = text.slice(start, index);
      if (!/^-?(0|[1-9][0-9]*)$/.test(token) || !Number.isSafeInteger(Number(token))) invalid();
    } else index++;
  }
  if (stack.length !== 0) invalid();
  try { return object(JSON.parse(text)); } catch { return invalid(); }
}

export function decodeReply(raw: unknown, requestId: string): ObjectDto {
  const frame = strictJson(raw);
  version(frame);
  if (frame.kind === "error") {
    exact(frame, ["schemaVersion", "request_id", "kind", "error"]);
    if (frame.request_id !== null && frame.request_id !== requestId) invalid();
    const error = object(frame.error);
    version(error);
    exact(error, ["schemaVersion", "category", "code"]);
    if (typeof error.code !== "string" || !Object.hasOwn(categories, error.code)) invalid();
    const code = error.code as ErrorCode;
    if (error.category !== categories[code]) invalid();
    throw new IpcError(code);
  }
  exact(frame, ["schemaVersion", "request_id", "kind", "payload"]);
  if (frame.kind !== "result" || frame.request_id !== requestId) invalid();
  const dto = object(frame.payload);
  version(dto);
  return dto;
}

export function encodeRequest<K extends Operation>(operation: K, requestId: string, payload: RequestPayloads[K]): Uint8Array {
  if (!/^[A-Za-z0-9_-]{1,64}$/.test(requestId)
      || !["app_info", "client_ready", "catalog_page", "configuration_select", "configuration_page", "plan_create", "plan_input_choose", "plan_parameter_set", "plan_prepare", "validation_start", "validation_state", "validation_acknowledge", "validation_cancel", "validation_progress", "resource_budget_state", "validation_budget", "validation_budget_decide", "results_query", "results_query_state", "results_page", "results_overview", "results_select", "results_text", "results_context", "results_sources", "results_domain", "results_domain_state", "report_about", "export_prepare", "export_execute", "export_state", "export_cancel"].includes(operation)) invalid();
  const bytes = new TextEncoder().encode(JSON.stringify({
    schemaVersion: 1, request_id: requestId, kind: "request", operation,
    payload,
  }));
  if (bytes.byteLength > MAX_BYTES) throw new IpcError("IPC_PAYLOAD_TOO_LARGE");
  return bytes;
}

async function exchange<K extends Operation>(operation: K, payload: RequestPayloads[K]): Promise<ObjectDto> {
  const requestId = crypto.randomUUID();
  const bytes = encodeRequest(operation, requestId, payload);
  let raw: unknown;
  try { raw = await invoke<unknown>("lintra_ipc", bytes); }
  catch { throw new IpcError("IPC_TECHNICAL_FAILURE"); }
  return decodeReply(raw, requestId);
}

export async function appInfo(): Promise<AppInfo> {
  const dto = await exchange("app_info", { schemaVersion: 1 });
  exact(dto, ["schemaVersion", "app_version", "validation_available"]);
  if (typeof dto.app_version !== "string" || dto.app_version.length < 1
      || dto.app_version.length > 64 || !/^[A-Za-z0-9.+-]+$/.test(dto.app_version)
      || dto.validation_available !== false) invalid();
  return { schemaVersion: 1, app_version: dto.app_version, validation_available: false };
}

export async function clientReady(): Promise<void> {
  const dto = await exchange("client_ready", { schemaVersion: 1 });
  exact(dto, ["schemaVersion", "ready"]);
  if (dto.ready !== true) invalid();
}

export type BundleProvenance = {
  schemaVersion: 1; bundle_source: "embedded" | "external";
  bundle_version: string | null; bundle_manifest_status: "present" | "absent";
};
export type SchemaDescriptor = { schemaVersion: 1; schema_ref: string; flow_id: string; schema_version: string; trace_count: string };
export type CatalogPage = { schemaVersion: 1; offset: string; entries: SchemaDescriptor[]; next_offset: string | null; provenance: BundleProvenance };
export type ConfigurationSelection = {
  schemaVersion: 1; configuration_ref: string; flow_id: string; schema_version: string;
  schema_hash: string; encoding: "utf8"; trace_count: string; parameter_count: string; provenance: BundleProvenance;
};
export type ConfigurationDescriptor =
  | { schemaVersion: 1; kind: "trace"; trace_ref: string; trace_id: string }
  | { schemaVersion: 1; kind: "parameter"; parameter_ref: string; parameter_id: string; value_kind: "text" | "integer" };
export type ConfigurationPage = { schemaVersion: 1; configuration_ref: string; offset: string; descriptors: ConfigurationDescriptor[]; next_offset: string | null };

function text(value: unknown): string {
  if (typeof value !== "string" || value.length === 0) invalid();
  return value;
}
function opaque(value: unknown): string {
  const result = text(value);
  if (!/^[A-Za-z0-9_-]{1,128}$/.test(result)) invalid();
  return result;
}
function unsigned(value: unknown): string {
  const result = text(value);
  if (!/^(0|[1-9][0-9]{0,19})$/.test(result) || BigInt(result) > 18446744073709551615n) invalid();
  return result;
}
function provenance(value: unknown): BundleProvenance {
  const dto = object(value);
  version(dto);
  exact(dto, ["schemaVersion", "bundle_source", "bundle_version", "bundle_manifest_status"]);
  if (dto.bundle_source !== "embedded" && dto.bundle_source !== "external") invalid();
  if (dto.bundle_manifest_status !== "present" && dto.bundle_manifest_status !== "absent") invalid();
  const declared = dto.bundle_manifest_status === "present";
  if (declared ? typeof dto.bundle_version !== "string" || dto.bundle_version.length === 0
      : dto.bundle_version !== null || dto.bundle_source === "embedded") invalid();
  return { schemaVersion: 1, bundle_source: dto.bundle_source,
    bundle_version: declared ? text(dto.bundle_version) : null, bundle_manifest_status: dto.bundle_manifest_status };
}
function page(dto: ObjectDto, name: "entries" | "descriptors", offset: string): unknown[] {
  if (unsigned(dto.offset) !== offset || !Array.isArray(dto[name])) invalid();
  const rows = dto[name];
  if (rows.length > 64) invalid();
  if (dto.next_offset !== null && (rows.length !== 64
      || BigInt(unsigned(dto.next_offset)) !== BigInt(offset) + BigInt(rows.length))) invalid();
  return rows;
}

export async function catalogPage(offset = "0"): Promise<CatalogPage> {
  unsigned(offset);
  const dto = await exchange("catalog_page", { schemaVersion: 1, offset });
  exact(dto, ["schemaVersion", "offset", "entries", "next_offset", "provenance"]);
  const entries = page(dto, "entries", offset).map(value => {
    const entry = object(value);
    version(entry);
    exact(entry, ["schemaVersion", "schema_ref", "flow_id", "schema_version", "trace_count"]);
    return { schemaVersion: 1 as const, schema_ref: opaque(entry.schema_ref), flow_id: text(entry.flow_id),
      schema_version: text(entry.schema_version), trace_count: unsigned(entry.trace_count) };
  });
  return { schemaVersion: 1, offset, entries, next_offset: dto.next_offset === null ? null : unsigned(dto.next_offset), provenance: provenance(dto.provenance) };
}

export async function configurationSelect(schemaRef: string, encoding: "utf8"): Promise<ConfigurationSelection> {
  opaque(schemaRef);
  if (encoding !== "utf8") invalid();
  const dto = await exchange("configuration_select", { schemaVersion: 1, schema_ref: schemaRef, encoding });
  exact(dto, ["schemaVersion", "configuration_ref", "flow_id", "schema_version", "schema_hash", "encoding", "trace_count", "parameter_count", "provenance"]);
  const hash = text(dto.schema_hash);
  if (!/^[0-9a-f]{64}$/.test(hash) || dto.encoding !== "utf8") invalid();
  return { schemaVersion: 1, configuration_ref: opaque(dto.configuration_ref), flow_id: text(dto.flow_id),
    schema_version: text(dto.schema_version), schema_hash: hash, encoding: "utf8", trace_count: unsigned(dto.trace_count),
    parameter_count: unsigned(dto.parameter_count), provenance: provenance(dto.provenance) };
}

export async function configurationPage(configurationRef: string, offset = "0"): Promise<ConfigurationPage> {
  opaque(configurationRef); unsigned(offset);
  const dto = await exchange("configuration_page", { schemaVersion: 1, configuration_ref: configurationRef, offset });
  exact(dto, ["schemaVersion", "configuration_ref", "offset", "descriptors", "next_offset"]);
  if (dto.configuration_ref !== configurationRef) invalid();
  const descriptors: ConfigurationDescriptor[] = page(dto, "descriptors", offset).map(value => {
    const row = object(value);
    version(row);
    if (row.kind === "trace") {
      exact(row, ["schemaVersion", "kind", "trace_ref", "trace_id"]);
      return { schemaVersion: 1, kind: "trace", trace_ref: opaque(row.trace_ref), trace_id: text(row.trace_id) };
    }
    exact(row, ["schemaVersion", "kind", "parameter_ref", "parameter_id", "value_kind"]);
    if (row.kind !== "parameter" || (row.value_kind !== "text" && row.value_kind !== "integer")) invalid();
    return { schemaVersion: 1, kind: "parameter", parameter_ref: opaque(row.parameter_ref), parameter_id: text(row.parameter_id), value_kind: row.value_kind };
  });
  return { schemaVersion: 1, configuration_ref: configurationRef, offset, descriptors, next_offset: dto.next_offset === null ? null : unsigned(dto.next_offset) };
}

export type PlanDraft = { schemaVersion: 1; plan_ref: string; configuration_ref: string; mode: "local_only" | "full"; assigned_input_count: string };
export type InputPreviewDto = { schemaVersion: 1; file_size: string; sampled_bytes: string; lf: string; crlf: string; lone_cr: string; reached_eof: boolean };
export type InputChoice = { schemaVersion: 1; plan_ref: string; trace_ref: string; disposition: "selected" | "cancelled"; assigned_input_count: string; preview: InputPreviewDto | null };

function readPlanDraft(dto: ObjectDto): PlanDraft {
  exact(dto, ["schemaVersion", "plan_ref", "configuration_ref", "mode", "assigned_input_count"]);
  if (dto.mode !== "local_only" && dto.mode !== "full") invalid();
  return { schemaVersion: 1, plan_ref: opaque(dto.plan_ref), configuration_ref: opaque(dto.configuration_ref),
    mode: dto.mode, assigned_input_count: unsigned(dto.assigned_input_count) };
}

export async function planCreate(configurationRef: string, mode: "local_only" | "full", traceRef: string | null): Promise<PlanDraft> {
  opaque(configurationRef);
  if (mode === "local_only") opaque(traceRef);
  else if (mode !== "full" || traceRef !== null) invalid();
  const dto = await exchange("plan_create", { schemaVersion: 1, configuration_ref: configurationRef, mode, trace_ref: traceRef });
  const draft = readPlanDraft(dto);
  if (draft.configuration_ref !== configurationRef || draft.mode !== mode) invalid();
  return draft;
}

export async function planInputChoose(planRef: string, traceRef: string): Promise<InputChoice> {
  opaque(planRef); opaque(traceRef);
  const dto = await exchange("plan_input_choose", { schemaVersion: 1, plan_ref: planRef, trace_ref: traceRef });
  exact(dto, ["schemaVersion", "plan_ref", "trace_ref", "disposition", "assigned_input_count", "preview"]);
  if (dto.trace_ref !== traceRef || (dto.disposition !== "selected" && dto.disposition !== "cancelled")) invalid();
  let preview: InputPreviewDto | null = null;
  if (dto.disposition === "selected") {
    const raw = object(dto.preview); version(raw);
    exact(raw, ["schemaVersion", "file_size", "sampled_bytes", "lf", "crlf", "lone_cr", "reached_eof"]);
    if (typeof raw.reached_eof !== "boolean" || dto.plan_ref === planRef) invalid();
    preview = { schemaVersion: 1, file_size: unsigned(raw.file_size), sampled_bytes: unsigned(raw.sampled_bytes),
      lf: unsigned(raw.lf), crlf: unsigned(raw.crlf), lone_cr: unsigned(raw.lone_cr), reached_eof: raw.reached_eof };
  } else if (dto.preview !== null || dto.plan_ref !== planRef) invalid();
  return { schemaVersion: 1, plan_ref: opaque(dto.plan_ref), trace_ref: traceRef, disposition: dto.disposition,
    assigned_input_count: unsigned(dto.assigned_input_count), preview };
}

/** Wire syntax of an integer parameter (FULL.md: exact text, parsed by Rust
 * as IntegerLiteral): optional minus and decimal digits. Range and allowed
 * values are checked by Rust only. */
export function integerText(text: string): boolean {
  return /^-?[0-9]+$/.test(text);
}

export async function planParameterSet(planRef: string, parameterRef: string, value: { kind: "text" | "integer"; text: string }): Promise<PlanDraft> {
  opaque(planRef); opaque(parameterRef);
  exact(object(value), ["kind", "text"]);
  if ((value.kind !== "text" && value.kind !== "integer") || typeof value.text !== "string") invalid();
  if (value.kind === "integer" && !integerText(value.text)) invalid();
  const dto = await exchange("plan_parameter_set", { schemaVersion: 1, plan_ref: planRef, parameter_ref: parameterRef,
    value: { schemaVersion: 1, kind: value.kind, text: value.text } });
  const draft = readPlanDraft(dto);
  if (draft.plan_ref === planRef) invalid();
  return draft;
}

export type PreparedPlan = { schemaVersion: 1; plan_ref: string; mode: "local_only" | "full"; ready: true; schema_hash: string; input_count: string; parameter_count: string; provenance: BundleProvenance };
export async function planPrepare(planRef: string): Promise<PreparedPlan> {
  opaque(planRef);
  const dto = await exchange("plan_prepare", { schemaVersion: 1, plan_ref: planRef });
  exact(dto, ["schemaVersion", "plan_ref", "mode", "ready", "schema_hash", "input_count", "parameter_count", "provenance"]);
  if (dto.plan_ref !== planRef || dto.ready !== true || (dto.mode !== "local_only" && dto.mode !== "full")) invalid();
  const hash = text(dto.schema_hash);
  if (!/^[0-9a-f]{64}$/.test(hash)) invalid();
  return { schemaVersion: 1, plan_ref: planRef, mode: dto.mode, ready: true, schema_hash: hash,
    input_count: unsigned(dto.input_count), parameter_count: unsigned(dto.parameter_count), provenance: provenance(dto.provenance) };
}

export type RunRef = { schemaVersion: 1; instance_id: string; run_id: string };
export type RunOutcome =
  | { schemaVersion: 1; kind: "validation"; status: "completed_valid" | "completed_invalid" | "cancelled"; completeness: "complete" | "partial"; truncated: boolean; report_available: boolean }
  | { schemaVersion: 1; kind: "technical_failure"; error: { schemaVersion: 1; category: "technical"; code: "IPC_TECHNICAL_FAILURE" } }
  | { schemaVersion: 1; kind: "not_started"; reason: "cancelled" };
export type RunSnapshot =
  | { schemaVersion: 1; state: "idle"; run: null; outcome: null }
  | { schemaVersion: 1; state: "preparing" | "running" | "cancelling" | "finalizing" | "terminal"; run: RunRef; plan_ref: string; mode: "local_only" | "full"; cancel_requested: boolean; outcome: RunOutcome | null };

function readRun(value: unknown): RunRef {
  const dto = object(value);
  version(dto); exact(dto, ["schemaVersion", "instance_id", "run_id"]);
  if (typeof dto.instance_id !== "string" || !/^[0-9a-f]{32}$/.test(dto.instance_id) || unsigned(dto.run_id) === "0") invalid();
  return { schemaVersion: 1, instance_id: dto.instance_id, run_id: unsigned(dto.run_id) };
}
function sameRun(left: RunRef, right: RunRef): boolean {
  return left.instance_id === right.instance_id && left.run_id === right.run_id;
}
function readOutcome(value: unknown): RunOutcome {
  const dto = object(value);
  version(dto);
  if (dto.kind === "not_started") {
    exact(dto, ["schemaVersion", "kind", "reason"]);
    if (dto.reason !== "cancelled") invalid();
    return { schemaVersion: 1, kind: "not_started", reason: "cancelled" };
  }
  if (dto.kind === "technical_failure") {
    exact(dto, ["schemaVersion", "kind", "error"]);
    const error = object(dto.error); version(error); exact(error, ["schemaVersion", "category", "code"]);
    if (error.category !== "technical" || error.code !== "IPC_TECHNICAL_FAILURE") invalid();
    return { schemaVersion: 1, kind: "technical_failure", error: { schemaVersion: 1, category: "technical", code: "IPC_TECHNICAL_FAILURE" } };
  }
  exact(dto, ["schemaVersion", "kind", "status", "completeness", "truncated", "report_available"]);
  if (dto.kind !== "validation" || !["completed_valid", "completed_invalid", "cancelled"].includes(String(dto.status))
      || (dto.completeness !== "complete" && dto.completeness !== "partial") || typeof dto.truncated !== "boolean" || typeof dto.report_available !== "boolean") invalid();
  const partial = dto.status === "cancelled" || dto.truncated;
  if ((partial && dto.completeness !== "partial") || dto.report_available === partial) invalid();
  return { schemaVersion: 1, kind: "validation", status: dto.status as "completed_valid" | "completed_invalid" | "cancelled", completeness: dto.completeness, truncated: dto.truncated, report_available: dto.report_available };
}
export async function validationStart(planRef: string, mode: "local_only" | "full"): Promise<RunRef> {
  opaque(planRef);
  if (mode !== "local_only" && mode !== "full") invalid();
  return readRun(await exchange("validation_start", { schemaVersion: 1, plan_ref: planRef, mode }));
}
export async function validationState(run: RunRef | null = null): Promise<RunSnapshot> {
  if (run !== null) readRun(run);
  const dto = await exchange("validation_state", { schemaVersion: 1, run });
  if (dto.state === "idle") {
    exact(dto, ["schemaVersion", "state", "run", "outcome"]);
    if (run !== null || dto.run !== null || dto.outcome !== null) invalid();
    return { schemaVersion: 1, state: "idle", run: null, outcome: null };
  }
  exact(dto, ["schemaVersion", "state", "run", "plan_ref", "mode", "cancel_requested", "outcome"]);
  const current = readRun(dto.run);
  if (run !== null && !sameRun(run, current)) invalid();
  if (!(["preparing", "running", "cancelling", "finalizing", "terminal"] as unknown[]).includes(dto.state)
      || (dto.mode !== "local_only" && dto.mode !== "full") || typeof dto.cancel_requested !== "boolean") invalid();
  if ((dto.state === "terminal") !== (dto.outcome !== null)) invalid();
  const outcome = dto.outcome === null ? null : readOutcome(dto.outcome);
  if (outcome?.kind === "not_started" && !dto.cancel_requested) invalid();
  return { schemaVersion: 1, state: dto.state as Exclude<RunSnapshot["state"], "idle">, run: current,
    plan_ref: opaque(dto.plan_ref), mode: dto.mode, cancel_requested: dto.cancel_requested, outcome };
}
export async function validationAcknowledge(run: RunRef): Promise<void> {
  readRun(run);
  const dto = await exchange("validation_acknowledge", { schemaVersion: 1, run });
  exact(dto, ["schemaVersion", "run", "acknowledged"]);
  if (!sameRun(run, readRun(dto.run)) || dto.acknowledged !== true) invalid();
}

export type RunProgress = {
  schemaVersion: 1; run: RunRef; sequence: string;
  phase: "preflight" | "local_validation" | "cross_record_validation" | "finalizing_report" | null;
  trace_ref: string | null; trace_records: string; cross_records: string;
};
export async function validationProgress(run: RunRef): Promise<RunProgress> {
  readRun(run);
  const dto = await exchange("validation_progress", { schemaVersion: 1, run });
  exact(dto, ["schemaVersion", "run", "sequence", "phase", "trace_ref", "trace_records", "cross_records"]);
  if (!sameRun(run, readRun(dto.run)) || !([null, "preflight", "local_validation", "cross_record_validation", "finalizing_report"] as unknown[]).includes(dto.phase)) invalid();
  const trace = dto.trace_ref === null ? null : opaque(dto.trace_ref);
  const traceRecords = unsigned(dto.trace_records);
  if ((trace !== null && trace.length > 64) || (trace === null && traceRecords !== "0")
      || (dto.phase === "cross_record_validation" && trace !== null)) invalid();
  return { schemaVersion: 1, run, sequence: unsigned(dto.sequence), phase: dto.phase as RunProgress["phase"],
    trace_ref: trace, trace_records: traceRecords, cross_records: unsigned(dto.cross_records) };
}
// Pure consumer rule: a terminal run cannot reopen after a late progress reply.
export function acceptRunProgress(sample: RunProgress, run: RunRef, previousSequence: string | null, terminalSeen: boolean): boolean {
  return !terminalSeen && sameRun(sample.run, run)
    && (previousSequence === null || BigInt(sample.sequence) > BigInt(previousSequence));
}
// D09 soft resource budget (T-MIG-P5-05, additive IPC v1 extension). Rust
// reads the configuration, runs the preflight and owns the decision; the
// renderer only shows the counters and returns the one-time handle.
export type BudgetConfigState = { schemaVersion: 1; state: "unconfigured" | "invalid" | "configured"; input_bytes: string | null; reference_bytes: string | null };
export type BudgetAudit = {
  schemaVersion: 1; input_bytes: string; reference_bytes: string; missing_reference_files: string;
  input_budget: string | null; reference_budget: string | null; input_budget_exceeded: boolean; reference_budget_exceeded: boolean;
};
export type BudgetGate = {
  schemaVersion: 1; run: RunRef;
  state: "unconfigured" | "checking" | "within_budget" | "confirmation_required" | "proceeded" | "declined" | "not_completed";
  audit: BudgetAudit | null; confirmation_ref: string | null;
};
function threshold(value: unknown): string | null {
  return nullable(value, item => { const result = unsigned(item); if (result === "0") invalid(); return result; });
}
export async function resourceBudgetState(): Promise<BudgetConfigState> {
  const dto = await exchange("resource_budget_state", { schemaVersion: 1 });
  exact(dto, ["schemaVersion", "state", "input_bytes", "reference_bytes"]);
  const state = oneOf(dto.state, ["unconfigured", "invalid", "configured"] as const);
  const input = threshold(dto.input_bytes), reference = threshold(dto.reference_bytes);
  if (state !== "configured" && (input !== null || reference !== null)) invalid();
  return { schemaVersion: 1, state, input_bytes: input, reference_bytes: reference };
}
function readAudit(value: unknown): BudgetAudit {
  const dto = versioned(value, ["schemaVersion", "input_bytes", "reference_bytes", "missing_reference_files", "input_budget", "reference_budget", "input_budget_exceeded", "reference_budget_exceeded"]);
  const audit = { schemaVersion: 1 as const, input_bytes: unsigned(dto.input_bytes), reference_bytes: unsigned(dto.reference_bytes),
    missing_reference_files: unsigned(dto.missing_reference_files), input_budget: threshold(dto.input_budget), reference_budget: threshold(dto.reference_budget),
    input_budget_exceeded: dto.input_budget_exceeded, reference_budget_exceeded: dto.reference_budget_exceeded };
  // Flags must agree with the counters Rust sent: never a warning without a crossed threshold.
  const over = (bytes: string, limit: string | null) => limit !== null && BigInt(bytes) > BigInt(limit);
  if (audit.input_budget_exceeded !== over(audit.input_bytes, audit.input_budget)
      || audit.reference_budget_exceeded !== over(audit.reference_bytes, audit.reference_budget)) invalid();
  return audit as BudgetAudit;
}
export async function validationBudget(run: RunRef): Promise<BudgetGate> {
  readRun(run);
  const dto = await exchange("validation_budget", { schemaVersion: 1, run });
  exact(dto, ["schemaVersion", "run", "state", "audit", "confirmation_ref"]);
  if (!sameRun(run, readRun(dto.run))) invalid();
  const state = oneOf(dto.state, ["unconfigured", "checking", "within_budget", "confirmation_required", "proceeded", "declined", "not_completed"] as const);
  const audit = nullable(dto.audit, readAudit);
  const withAudit = state === "within_budget" || state === "confirmation_required" || state === "proceeded" || state === "declined";
  if (withAudit !== (audit !== null)) invalid();
  const exceeded = audit !== null && (audit.input_budget_exceeded || audit.reference_budget_exceeded);
  if ((state === "within_budget" && exceeded) || ((state === "confirmation_required" || state === "proceeded") && !exceeded)) invalid();
  const handle = nullable(dto.confirmation_ref, opaque);
  if ((state === "confirmation_required") !== (handle !== null)) invalid();
  return { schemaVersion: 1, run, state, audit, confirmation_ref: handle };
}
export async function validationBudgetDecide(run: RunRef, confirmationRef: string, decision: "proceed" | "cancel"): Promise<"proceeding" | "cancelled"> {
  readRun(run); opaque(confirmationRef); oneOf(decision, ["proceed", "cancel"] as const);
  const dto = await exchange("validation_budget_decide", { schemaVersion: 1, run, confirmation_ref: confirmationRef, decision });
  exact(dto, ["schemaVersion", "run", "disposition"]);
  if (!sameRun(run, readRun(dto.run))) invalid();
  const disposition = oneOf(dto.disposition, ["proceeding", "cancelled"] as const);
  if ((decision === "proceed") !== (disposition === "proceeding")) invalid();
  return disposition;
}
export async function validationCancel(run: RunRef): Promise<"requested" | "already_requested" | "too_late" | "already_terminal"> {
  readRun(run);
  const dto = await exchange("validation_cancel", { schemaVersion: 1, run });
  exact(dto, ["schemaVersion", "run", "disposition"]);
  if (!sameRun(run, readRun(dto.run)) || !["requested", "already_requested", "too_late", "already_terminal"].includes(String(dto.disposition))) invalid();
  return dto.disposition as "requested" | "already_requested" | "too_late" | "already_terminal";
}

// Results browsing (T-MIG-P4-08). Filtering, ordering and paging run in Rust;
// this client only sends exact DTOs and checks that each bounded page is
// internally consistent. It never receives or rebuilds an index or dataset.
export const DEFAULT_PAGE_ROWS = 64;
export const MAX_PAGE_ROWS = 128;
const PREVIEW_BYTES = 256;
export type ResultsFilter = {
  schemaVersion: 1; code?: string; field?: string; record?: string; fiscal_code?: string;
  line?: string | null; trace_ref?: string | null;
};
export type QueryStatus = {
  schemaVersion: 1; run: RunRef; query_ref: string; state: "building" | "ready" | "failed";
  total_matches: string | null; total_diagnostics: string | null;
};
export type Preview = { schemaVersion: 1; text: string; complete: boolean };
/** Row DTO. Since T-MIG-P6-01 (author decision, PAGING.md amendment of 24
 * Nov 2026) it also carries the reason: message, expected text and the
 * displayed value found, each a preview of at most ROW_TEXT_BYTES. */
export type ResultRow = {
  schemaVersion: 1; diagnostic_ref: string; ordinal: string; line: string; trace_ref: string;
  code: Preview; field: Preview | null; message: Preview; expected: Preview | null; actual: Preview | null;
};
export const ROW_TEXT_BYTES = 128;
export type ResultsPage = {
  schemaVersion: 1; query_ref: string; run: RunRef; page_generation: string; offset: string;
  rows: ResultRow[]; next_offset: string; has_more: boolean; total_matches: string; total_diagnostics: string;
};

function identifier(value: unknown): string {
  const result = opaque(value);
  if (result.length > 64) invalid();
  return result;
}
function readFilter(filters: ResultsFilter): ResultsFilter {
  const dto = object(filters);
  version(dto);
  if (!Object.keys(dto).every(key => ["schemaVersion", "code", "field", "record", "fiscal_code", "line", "trace_ref"].includes(key))) invalid();
  for (const key of ["code", "field", "record", "fiscal_code"]) if (Object.hasOwn(dto, key) && typeof dto[key] !== "string") invalid();
  if (Object.hasOwn(dto, "line") && dto.line !== null && (typeof dto.line !== "string" || !/^[0-9]*$/.test(dto.line))) invalid();
  if (Object.hasOwn(dto, "trace_ref") && dto.trace_ref !== null) identifier(dto.trace_ref);
  return filters;
}
function readStatus(dto: ObjectDto, queryRef: string | null, run: RunRef | null): QueryStatus {
  exact(dto, ["schemaVersion", "run", "query_ref", "state", "total_matches", "total_diagnostics"]);
  const current = readRun(dto.run);
  const reference = identifier(dto.query_ref);
  if ((run !== null && !sameRun(run, current)) || (queryRef !== null && queryRef !== reference)
      || !["building", "ready", "failed"].includes(String(dto.state))) invalid();
  if (dto.state !== "ready") {
    if (dto.total_matches !== null || dto.total_diagnostics !== null) invalid();
    return { schemaVersion: 1, run: current, query_ref: reference, state: dto.state as "building" | "failed", total_matches: null, total_diagnostics: null };
  }
  const matches = unsigned(dto.total_matches);
  const total = unsigned(dto.total_diagnostics);
  if (BigInt(matches) > BigInt(total)) invalid();
  return { schemaVersion: 1, run: current, query_ref: reference, state: "ready", total_matches: matches, total_diagnostics: total };
}
function readPreview(value: unknown, limit = PREVIEW_BYTES): Preview {
  const dto = object(value);
  version(dto); exact(dto, ["schemaVersion", "text", "complete"]);
  if (typeof dto.text !== "string" || typeof dto.complete !== "boolean"
      || new TextEncoder().encode(dto.text).byteLength > limit) invalid();
  return { schemaVersion: 1, text: dto.text, complete: dto.complete };
}
export async function resultsQuery(run: RunRef, filters: ResultsFilter): Promise<QueryStatus> {
  readRun(run); readFilter(filters);
  const status = readStatus(await exchange("results_query", { schemaVersion: 1, run, filters }), null, run);
  if (status.state === "failed") invalid();
  return status;
}
export async function resultsQueryState(queryRef: string): Promise<QueryStatus> {
  identifier(queryRef);
  return readStatus(await exchange("results_query_state", { schemaVersion: 1, query_ref: queryRef }), queryRef, null);
}
export async function resultsPage(queryRef: string, offset = "0", limit = DEFAULT_PAGE_ROWS): Promise<ResultsPage> {
  identifier(queryRef); unsigned(offset);
  if (!Number.isInteger(limit) || limit < 1 || limit > MAX_PAGE_ROWS) invalid();
  const dto = await exchange("results_page", { schemaVersion: 1, query_ref: queryRef, offset, limit });
  exact(dto, ["schemaVersion", "query_ref", "run", "page_generation", "offset", "rows", "next_offset", "has_more", "total_matches", "total_diagnostics"]);
  if (dto.query_ref !== queryRef || dto.offset !== offset || !Array.isArray(dto.rows) || dto.rows.length > limit || typeof dto.has_more !== "boolean") invalid();
  const start = BigInt(offset);
  const rows = dto.rows.map((value, index): ResultRow => {
    const row = object(value);
    version(row); exact(row, ["schemaVersion", "diagnostic_ref", "ordinal", "line", "trace_ref", "code", "field", "message", "expected", "actual"]);
    const ordinal = unsigned(row.ordinal);
    const line = unsigned(row.line);
    if (BigInt(ordinal) !== start + BigInt(index) || line === "0") invalid();
    return { schemaVersion: 1, diagnostic_ref: identifier(row.diagnostic_ref), ordinal, line,
      trace_ref: identifier(row.trace_ref), code: readPreview(row.code), field: row.field === null ? null : readPreview(row.field),
      message: readPreview(row.message, ROW_TEXT_BYTES),
      expected: row.expected === null ? null : readPreview(row.expected, ROW_TEXT_BYTES),
      actual: row.actual === null ? null : readPreview(row.actual, ROW_TEXT_BYTES) };
  });
  const next = unsigned(dto.next_offset);
  const matches = unsigned(dto.total_matches);
  const total = unsigned(dto.total_diagnostics);
  if (BigInt(next) !== start + BigInt(rows.length) || BigInt(next) > BigInt(matches)
      || BigInt(matches) > BigInt(total) || dto.has_more !== BigInt(next) < BigInt(matches)) invalid();
  return { schemaVersion: 1, query_ref: queryRef, run: readRun(dto.run), page_generation: unsigned(dto.page_generation),
    offset, rows, next_offset: next, has_more: dto.has_more, total_matches: matches, total_diagnostics: total };
}

// Report overview for the results charts (T-MIG-P6-01). Counts come from the
// verified Rust summary and publication index; this client only checks that
// they are canonical and consistent and never derives counts from rows.
export type CodeCount = { schemaVersion: 1; code: Preview; count: string };
export type TraceCount = { schemaVersion: 1; trace_ref: string; trace: Preview; count: string };
export type ResultsOverview = {
  schemaVersion: 1; run: RunRef; records_processed: string; valid_records: string; invalid_records: string;
  total_errors: string; by_code: CodeCount[]; by_trace: TraceCount[];
};
export async function resultsOverview(run: RunRef): Promise<ResultsOverview> {
  readRun(run);
  const dto = await exchange("results_overview", { schemaVersion: 1, run });
  exact(dto, ["schemaVersion", "run", "records_processed", "valid_records", "invalid_records", "total_errors", "by_code", "by_trace"]);
  const current = readRun(dto.run);
  if (!sameRun(run, current) || !Array.isArray(dto.by_code) || !Array.isArray(dto.by_trace)) invalid();
  const processed = unsigned(dto.records_processed), valid = unsigned(dto.valid_records);
  const invalidRecords = unsigned(dto.invalid_records), total = unsigned(dto.total_errors);
  if (BigInt(valid) + BigInt(invalidRecords) > BigInt(processed)) invalid();
  const byCode = dto.by_code.map((value): CodeCount => {
    const entry = object(value);
    version(entry); exact(entry, ["schemaVersion", "code", "count"]);
    return { schemaVersion: 1, code: readPreview(entry.code), count: unsigned(entry.count) };
  });
  const byTrace = dto.by_trace.map((value): TraceCount => {
    const entry = object(value);
    version(entry); exact(entry, ["schemaVersion", "trace_ref", "trace", "count"]);
    return { schemaVersion: 1, trace_ref: identifier(entry.trace_ref), trace: readPreview(entry.trace), count: unsigned(entry.count) };
  });
  const sum = (items: { count: string }[]) => items.reduce((acc, item) => acc + BigInt(item.count), 0n);
  if (sum(byCode) !== BigInt(total) || sum(byTrace) !== BigInt(total)
      || new Set(byTrace.map(item => item.trace_ref)).size !== byTrace.length) invalid();
  return { schemaVersion: 1, run: current, records_processed: processed, valid_records: valid, invalid_records: invalidRecords,
    total_errors: total, by_code: byCode, by_trace: byTrace };
}

// Selected diagnostic detail (T-MIG-P4-09). Rust resolves the reference, pins
// one diagnostic and exposes texts only as previews or bounded chunks; this
// client keeps at most the current chunk and never rebuilds a report, schema
// or domain. Counters and offsets stay decimal strings (no Number casts).
export const MAX_TEXT_BYTES = 16_384;
export const MAX_DETAIL_ENTRIES = 64;
export type DetailComponent = {
  schemaVersion: 1; component_ref: string;
  kind: "code" | "message" | "expected" | "field_name" | "guide" | "actual" | "sioc_record_id" | "fiscal_code" | "episode_id";
  preview: Preview;
};
export type DomainSourceKind = "allowed_values" | "range" | "fallback" | "reference" | "regex" | "non_enumerable";
// `condition` is the native guard of a conditional source (readable in full
// through resultsText); null for unconditional sources, or when the guard has
// no textual rendering, in which case `conditional` still says it applies
// only under that rule.
export type DomainSource = {
  schemaVersion: 1; source_ref: string; kind: DomainSourceKind; conditional: boolean; rule: Preview | null;
  condition: { schemaVersion: 1; component_ref: string; preview: Preview } | null;
  dataset: { schemaVersion: 1; id: Preview; version: Preview } | null; comparison: Preview;
};
export type SourcesPage = { schemaVersion: 1; selection_ref: string; offset: string; sources: DomainSource[]; next_offset: string; has_more: boolean };
export type SelectedDiagnostic = {
  schemaVersion: 1; selection_ref: string; query_ref: string; diagnostic_ref: string; run: RunRef;
  line: string; trace_ref: string;
  field_position: { schemaVersion: 1; start: string; end: string; units: "one_based_inclusive_bytes" } | null;
  actual_visibility: "available" | "not_present" | "local_report_only_record_scope";
  components: DetailComponent[]; source_count: string;
};
export type TextChunk = { schemaVersion: 1; offset: string; text: string; next_offset: string; has_more: boolean };
export type ContextEntry = {
  schemaVersion: 1; trace: Preview; field: Preview; component_ref: string | null; value: Preview | null;
  visibility: "available" | "local_report_only_record_scope";
};
export type ContextPage = { schemaVersion: 1; selection_ref: string; offset: string; entries: ContextEntry[]; next_offset: string; has_more: boolean };
export type DomainStatus = "available" | "available_empty" | "degraded" | "unavailable" | "not_captured" | "unsupported" | "cancelled" | "not_enumerable";
export type DomainPage = {
  schemaVersion: 1; kind: DomainSourceKind; conditional: boolean; expected_sha256: string | null; actual_sha256: string | null;
  status: DomainStatus; entries: { schemaVersion: 1; component_ref: string; preview: Preview }[] | null; next_cursor: string | null;
};
export type DomainAnswer =
  | { schemaVersion: 1; selection_ref: string; source_ref: string; state: "loading"; read_ref: string; domain: null }
  | { schemaVersion: 1; selection_ref: string; source_ref: string; state: "ready"; read_ref: null; domain: DomainPage };

function oneOf<T extends string>(value: unknown, values: readonly T[]): T {
  if (typeof value !== "string" || !(values as readonly string[]).includes(value)) invalid();
  return value as T;
}
function nullable<T>(value: unknown, read: (value: unknown) => T): T | null {
  return value === null ? null : read(value);
}
function sha256(value: unknown): string {
  if (typeof value !== "string" || !/^[0-9a-f]{64}$/.test(value)) invalid();
  return value;
}
function versioned(value: unknown, keys: string[]): ObjectDto {
  const dto = object(value);
  version(dto); exact(dto, keys);
  return dto;
}
function readSource(value: unknown): DomainSource {
  const dto = versioned(value, ["schemaVersion", "source_ref", "kind", "conditional", "rule", "condition", "dataset", "comparison"]);
  if (typeof dto.conditional !== "boolean" || (dto.conditional !== (dto.rule !== null))
      || (!dto.conditional && dto.condition !== null)) invalid();
  const condition = nullable(dto.condition, item => {
    const guard = versioned(item, ["schemaVersion", "component_ref", "preview"]);
    return { schemaVersion: 1 as const, component_ref: identifier(guard.component_ref), preview: readPreview(guard.preview) };
  });
  const dataset = nullable(dto.dataset, item => {
    const key = versioned(item, ["schemaVersion", "id", "version"]);
    return { schemaVersion: 1 as const, id: readPreview(key.id), version: readPreview(key.version) };
  });
  const kind = oneOf(dto.kind, ["allowed_values", "range", "fallback", "reference", "regex", "non_enumerable"] as const);
  if ((kind === "reference") !== (dataset !== null)) invalid();
  return { schemaVersion: 1, source_ref: identifier(dto.source_ref), kind, conditional: dto.conditional,
    rule: nullable(dto.rule, readPreview), condition, dataset, comparison: readPreview(dto.comparison) };
}
export async function resultsSelect(queryRef: string, diagnosticRef: string): Promise<SelectedDiagnostic> {
  identifier(queryRef); identifier(diagnosticRef);
  const dto = await exchange("results_select", { schemaVersion: 1, query_ref: queryRef, diagnostic_ref: diagnosticRef });
  exact(dto, ["schemaVersion", "selection_ref", "query_ref", "diagnostic_ref", "run", "line", "trace_ref", "field_position", "actual_visibility", "components", "source_count"]);
  if (dto.query_ref !== queryRef || dto.diagnostic_ref !== diagnosticRef || !Array.isArray(dto.components)) invalid();
  const sourceCount = unsigned(dto.source_count);
  const line = unsigned(dto.line);
  if (line === "0") invalid();
  const position = nullable(dto.field_position, value => {
    const range = versioned(value, ["schemaVersion", "start", "end", "units"]);
    const start = unsigned(range.start), end = unsigned(range.end);
    if (start === "0" || BigInt(end) < BigInt(start) || range.units !== "one_based_inclusive_bytes") invalid();
    return { schemaVersion: 1 as const, start, end, units: "one_based_inclusive_bytes" as const };
  });
  const seen = new Set<string>();
  const components = dto.components.map((value): DetailComponent => {
    const item = versioned(value, ["schemaVersion", "component_ref", "kind", "preview"]);
    const kind = oneOf(item.kind, ["code", "message", "expected", "field_name", "guide", "actual", "sioc_record_id", "fiscal_code", "episode_id"] as const);
    if (seen.has(kind)) invalid();
    seen.add(kind);
    return { schemaVersion: 1, component_ref: identifier(item.component_ref), kind, preview: readPreview(item.preview) };
  });
  const visibility = oneOf(dto.actual_visibility, ["available", "not_present", "local_report_only_record_scope"] as const);
  if (!seen.has("code") || !seen.has("message") || seen.has("actual") !== (visibility === "available")) invalid();
  return { schemaVersion: 1, selection_ref: identifier(dto.selection_ref), query_ref: queryRef, diagnostic_ref: diagnosticRef,
    run: readRun(dto.run), line, trace_ref: identifier(dto.trace_ref), field_position: position, actual_visibility: visibility,
    components, source_count: sourceCount };
}
// Domain sources are listed in pages of at most 64: their number never
// limits opening a diagnostic, and no complete list is kept by the client.
export async function resultsSources(selectionRef: string, offset = "0", limit = MAX_DETAIL_ENTRIES): Promise<SourcesPage> {
  identifier(selectionRef); unsigned(offset);
  if (!Number.isInteger(limit) || limit < 1 || limit > MAX_DETAIL_ENTRIES) invalid();
  const dto = await exchange("results_sources", { schemaVersion: 1, selection_ref: selectionRef, offset, limit });
  exact(dto, ["schemaVersion", "selection_ref", "offset", "sources", "next_offset", "has_more"]);
  if (dto.selection_ref !== selectionRef || dto.offset !== offset || !Array.isArray(dto.sources)
      || dto.sources.length > limit || typeof dto.has_more !== "boolean") invalid();
  const sources = dto.sources.map(readSource);
  const next = unsigned(dto.next_offset);
  if (BigInt(next) !== BigInt(offset) + BigInt(sources.length) || (sources.length === 0 && dto.has_more)) invalid();
  return { schemaVersion: 1, selection_ref: selectionRef, offset, sources, next_offset: next, has_more: dto.has_more };
}
export async function resultsText(selectionRef: string, componentRef: string, offset = "0", limit = MAX_TEXT_BYTES): Promise<TextChunk> {
  identifier(selectionRef); identifier(componentRef); unsigned(offset);
  if (!Number.isInteger(limit) || limit < 1 || limit > MAX_TEXT_BYTES) invalid();
  const dto = await exchange("results_text", { schemaVersion: 1, selection_ref: selectionRef, component_ref: componentRef, offset, limit });
  exact(dto, ["schemaVersion", "offset", "text", "next_offset", "has_more"]);
  if (dto.offset !== offset || typeof dto.text !== "string" || typeof dto.has_more !== "boolean") invalid();
  const size = new TextEncoder().encode(dto.text).byteLength;
  const next = unsigned(dto.next_offset);
  if (size > limit || BigInt(next) !== BigInt(offset) + BigInt(size) || (size === 0 && dto.has_more)) invalid();
  return { schemaVersion: 1, offset, text: dto.text, next_offset: next, has_more: dto.has_more };
}
export async function resultsContext(selectionRef: string, offset = "0", limit = MAX_DETAIL_ENTRIES): Promise<ContextPage> {
  identifier(selectionRef); unsigned(offset);
  if (!Number.isInteger(limit) || limit < 1 || limit > MAX_DETAIL_ENTRIES) invalid();
  const dto = await exchange("results_context", { schemaVersion: 1, selection_ref: selectionRef, offset, limit });
  exact(dto, ["schemaVersion", "selection_ref", "offset", "entries", "next_offset", "has_more"]);
  if (dto.selection_ref !== selectionRef || dto.offset !== offset || !Array.isArray(dto.entries)
      || dto.entries.length > limit || typeof dto.has_more !== "boolean") invalid();
  const entries = dto.entries.map((value): ContextEntry => {
    const item = versioned(value, ["schemaVersion", "trace", "field", "component_ref", "value", "visibility"]);
    const visibility = oneOf(item.visibility, ["available", "local_report_only_record_scope"] as const);
    const available = visibility === "available";
    if ((item.component_ref !== null) !== available || (item.value !== null) !== available) invalid();
    return { schemaVersion: 1, trace: readPreview(item.trace), field: readPreview(item.field),
      component_ref: nullable(item.component_ref, identifier), value: nullable(item.value, readPreview), visibility };
  });
  const next = unsigned(dto.next_offset);
  if (BigInt(next) !== BigInt(offset) + BigInt(entries.length) || (entries.length === 0 && dto.has_more)) invalid();
  return { schemaVersion: 1, selection_ref: selectionRef, offset, entries, next_offset: next, has_more: dto.has_more };
}
function readDomain(value: unknown): DomainPage {
  const dto = versioned(value, ["schemaVersion", "kind", "conditional", "expected_sha256", "actual_sha256", "status", "entries", "next_cursor"]);
  const status = oneOf(dto.status, ["available", "available_empty", "degraded", "unavailable", "not_captured", "unsupported", "cancelled", "not_enumerable"] as const);
  if (typeof dto.conditional !== "boolean") invalid();
  const listed = status === "available" || status === "available_empty";
  if ((dto.entries !== null) !== listed || (!listed && dto.next_cursor !== null)) invalid();
  const entries = dto.entries === null ? null : (() => {
    if (!Array.isArray(dto.entries) || dto.entries.length > MAX_DETAIL_ENTRIES
        || (dto.entries.length === 0) !== (status === "available_empty")) invalid();
    return dto.entries.map(item => {
      const entry = versioned(item, ["schemaVersion", "component_ref", "preview"]);
      return { schemaVersion: 1 as const, component_ref: identifier(entry.component_ref), preview: readPreview(entry.preview) };
    });
  })();
  const cursor = nullable(dto.next_cursor, identifier);
  if (cursor !== null && entries !== null && entries.length === 0) invalid();
  return { schemaVersion: 1, kind: oneOf(dto.kind, ["allowed_values", "range", "fallback", "reference", "regex", "non_enumerable"] as const),
    conditional: dto.conditional, expected_sha256: nullable(dto.expected_sha256, sha256), actual_sha256: nullable(dto.actual_sha256, sha256),
    status, entries, next_cursor: cursor };
}
function readDomainAnswer(dto: ObjectDto, selectionRef: string, sourceRef: string | null): DomainAnswer {
  exact(dto, ["schemaVersion", "selection_ref", "source_ref", "state", "read_ref", "domain"]);
  const source = identifier(dto.source_ref);
  if (dto.selection_ref !== selectionRef || (sourceRef !== null && source !== sourceRef)) invalid();
  if (dto.state === "loading") {
    if (dto.domain !== null) invalid();
    return { schemaVersion: 1, selection_ref: selectionRef, source_ref: source, state: "loading", read_ref: identifier(dto.read_ref), domain: null };
  }
  if (dto.state !== "ready" || dto.read_ref !== null) invalid();
  return { schemaVersion: 1, selection_ref: selectionRef, source_ref: source, state: "ready", read_ref: null, domain: readDomain(dto.domain) };
}
export async function resultsDomain(selectionRef: string, sourceRef: string, cursorRef: string | null = null): Promise<DomainAnswer> {
  identifier(selectionRef); identifier(sourceRef);
  if (cursorRef !== null) identifier(cursorRef);
  const dto = await exchange("results_domain", { schemaVersion: 1, selection_ref: selectionRef, source_ref: sourceRef, cursor_ref: cursorRef });
  return readDomainAnswer(dto, selectionRef, sourceRef);
}
export async function resultsDomainState(selectionRef: string, readRef: string): Promise<DomainAnswer> {
  identifier(selectionRef); identifier(readRef);
  const dto = await exchange("results_domain_state", { schemaVersion: 1, selection_ref: selectionRef, read_ref: readRef });
  const answer = readDomainAnswer(dto, selectionRef, null);
  if (answer.state === "loading" && answer.read_ref !== readRef) invalid();
  return answer;
}

// Report provenance and native export (T-MIG-P4-10). The renderer sends only
// opaque handles and a format: the save dialog, the destination warning and
// the overwrite confirmation are native, and no path, file name or report
// content ever reaches this client.
export type ExportFormat = "jsonl" | "csv" | "xlsx";
export type ReportAbout = {
  schemaVersion: 1; report_ref: string; run: RunRef; app_version: string; validator_version: Preview;
  flow_id: Preview; schema_version: Preview; schema_hash: Preview; mode: "local_only" | "full";
  status: "completed_valid" | "completed_invalid" | "cancelled"; completeness: "complete" | "partial";
  truncated: boolean; exportable: boolean; trace_count: string; reference_dataset_count: string;
  bundle: BundleProvenance;
};
export type DestinationCounts = { schemaVersion: 1; local_files: string; remote_files: string; undetermined_files: string; warning_acknowledged: boolean };
export type ExportPlan =
  | { schemaVersion: 1; state: "ready"; export_ref: string; report_ref: string; run: RunRef; format: ExportFormat; destination: DestinationCounts }
  | { schemaVersion: 1; state: "cancelled"; reason: "native_selection_cancelled" | "native_confirmation_declined" }
  | { schemaVersion: 1; state: "rejected"; reason: "destination_not_allowed" };
export type ExportResult =
  | { schemaVersion: 1; export_ref: string; state: "writing" }
  | { schemaVersion: 1; export_ref: string; report_ref: string; run: RunRef; format: ExportFormat;
      state: "succeeded" | "failed" | "cancelled"; published_roles: ("report" | "summary")[];
      cleanup_failed: boolean; durability_confirmed: boolean; destination: DestinationCounts;
      error: { schemaVersion: 1; category: "technical"; code: "IPC_TECHNICAL_FAILURE" } | null };

function readCounts(value: unknown): DestinationCounts {
  const dto = versioned(value, ["schemaVersion", "local_files", "remote_files", "undetermined_files", "warning_acknowledged"]);
  const local = unsigned(dto.local_files), remote = unsigned(dto.remote_files), undetermined = unsigned(dto.undetermined_files);
  // Exactly the report file and its summary sidecar are classified.
  if (BigInt(local) + BigInt(remote) + BigInt(undetermined) !== 2n || typeof dto.warning_acknowledged !== "boolean"
      || (dto.warning_acknowledged && remote === "0" && undetermined === "0")) invalid();
  return { schemaVersion: 1, local_files: local, remote_files: remote, undetermined_files: undetermined, warning_acknowledged: dto.warning_acknowledged };
}
function readProvenance(value: unknown): BundleProvenance {
  const dto = versioned(value, ["schemaVersion", "bundle_source", "bundle_version", "bundle_manifest_status"]);
  const source = oneOf(dto.bundle_source, ["embedded", "external"] as const);
  const manifest = oneOf(dto.bundle_manifest_status, ["present", "absent"] as const);
  if (dto.bundle_version !== null && typeof dto.bundle_version !== "string") invalid();
  return { schemaVersion: 1, bundle_source: source, bundle_version: dto.bundle_version as string | null, bundle_manifest_status: manifest };
}
export async function reportAbout(run: RunRef): Promise<ReportAbout> {
  readRun(run);
  const dto = await exchange("report_about", { schemaVersion: 1, run });
  exact(dto, ["schemaVersion", "report_ref", "run", "app_version", "validator_version", "flow_id", "schema_version", "schema_hash", "mode", "status", "completeness", "truncated", "exportable", "trace_count", "reference_dataset_count", "bundle"]);
  const current = readRun(dto.run);
  if (!sameRun(run, current) || typeof dto.app_version !== "string" || !/^[A-Za-z0-9.+-]{1,64}$/.test(dto.app_version)
      || typeof dto.truncated !== "boolean" || typeof dto.exportable !== "boolean") invalid();
  const status = oneOf(dto.status, ["completed_valid", "completed_invalid", "cancelled"] as const);
  const traces = unsigned(dto.trace_count);
  if (traces === "0" || (dto.exportable && (status === "cancelled" || dto.truncated))) invalid();
  return { schemaVersion: 1, report_ref: identifier(dto.report_ref), run: current, app_version: dto.app_version,
    validator_version: readPreview(dto.validator_version), flow_id: readPreview(dto.flow_id), schema_version: readPreview(dto.schema_version),
    schema_hash: readPreview(dto.schema_hash), mode: oneOf(dto.mode, ["local_only", "full"] as const), status,
    completeness: oneOf(dto.completeness, ["complete", "partial"] as const), truncated: dto.truncated, exportable: dto.exportable,
    trace_count: traces, reference_dataset_count: unsigned(dto.reference_dataset_count), bundle: readProvenance(dto.bundle) };
}
export async function exportPrepare(reportRef: string, format: ExportFormat): Promise<ExportPlan> {
  identifier(reportRef); oneOf(format, ["jsonl", "csv", "xlsx"] as const);
  const dto = await exchange("export_prepare", { schemaVersion: 1, report_ref: reportRef, format });
  if (dto.state === "cancelled") {
    exact(dto, ["schemaVersion", "state", "reason"]);
    return { schemaVersion: 1, state: "cancelled", reason: oneOf(dto.reason, ["native_selection_cancelled", "native_confirmation_declined"] as const) };
  }
  if (dto.state === "rejected") {
    exact(dto, ["schemaVersion", "state", "reason"]);
    return { schemaVersion: 1, state: "rejected", reason: oneOf(dto.reason, ["destination_not_allowed"] as const) };
  }
  exact(dto, ["schemaVersion", "export_ref", "report_ref", "run", "format", "state", "destination"]);
  if (dto.state !== "ready" || dto.report_ref !== reportRef || dto.format !== format) invalid();
  return { schemaVersion: 1, state: "ready", export_ref: identifier(dto.export_ref), report_ref: reportRef, run: readRun(dto.run),
    format, destination: readCounts(dto.destination) };
}
function readExportResult(dto: ObjectDto, exportRef: string): ExportResult {
  if (dto.export_ref !== exportRef) invalid();
  if (dto.state === "writing") {
    exact(dto, ["schemaVersion", "export_ref", "state"]);
    return { schemaVersion: 1, export_ref: exportRef, state: "writing" };
  }
  exact(dto, ["schemaVersion", "export_ref", "report_ref", "run", "format", "state", "published_roles", "cleanup_failed", "durability_confirmed", "destination", "error"]);
  const state = oneOf(dto.state, ["succeeded", "failed", "cancelled"] as const);
  if (!Array.isArray(dto.published_roles) || typeof dto.cleanup_failed !== "boolean" || typeof dto.durability_confirmed !== "boolean") invalid();
  const roles = dto.published_roles.map(role => oneOf(role, ["report", "summary"] as const));
  if (new Set(roles).size !== roles.length) invalid();
  // Success means both files published and durable, with nothing left over.
  if (state === "succeeded" && (roles.length !== 2 || dto.cleanup_failed || !dto.durability_confirmed || dto.error !== null)) invalid();
  let error: { schemaVersion: 1; category: "technical"; code: "IPC_TECHNICAL_FAILURE" } | null = null;
  if (state === "failed") {
    const item = versioned(dto.error, ["schemaVersion", "category", "code"]);
    if (item.category !== "technical" || item.code !== "IPC_TECHNICAL_FAILURE") invalid();
    error = { schemaVersion: 1, category: "technical", code: "IPC_TECHNICAL_FAILURE" };
  } else if (dto.error !== null) invalid();
  return { schemaVersion: 1, export_ref: exportRef, report_ref: identifier(dto.report_ref), run: readRun(dto.run),
    format: oneOf(dto.format, ["jsonl", "csv", "xlsx"] as const), state, published_roles: roles,
    cleanup_failed: dto.cleanup_failed, durability_confirmed: dto.durability_confirmed, destination: readCounts(dto.destination), error };
}
export async function exportExecute(exportRef: string): Promise<ExportResult> {
  identifier(exportRef);
  const dto = await exchange("export_execute", { schemaVersion: 1, export_ref: exportRef });
  const result = readExportResult(dto, exportRef);
  if (result.state !== "writing") invalid();
  return result;
}
export async function exportState(exportRef: string): Promise<ExportResult> {
  identifier(exportRef);
  return readExportResult(await exchange("export_state", { schemaVersion: 1, export_ref: exportRef }), exportRef);
}
export async function exportCancel(exportRef: string): Promise<"requested" | "already_requested" | "already_terminal"> {
  identifier(exportRef);
  const dto = await exchange("export_cancel", { schemaVersion: 1, export_ref: exportRef });
  exact(dto, ["schemaVersion", "export_ref", "disposition"]);
  if (dto.export_ref !== exportRef) invalid();
  return oneOf(dto.disposition, ["requested", "already_requested", "already_terminal"] as const);
}

export function errorText(error: unknown): string {
  if (error instanceof IpcError) {
    if (error.code === "IPC_UNSUPPORTED_VERSION") return "Le versioni dell’interfaccia e del servizio non coincidono. Chiudi Lintra e riapri l’applicazione installata.";
    if (error.code === "IPC_INVALID_MESSAGE" || error.code === "IPC_PAYLOAD_TOO_LARGE") return "La risposta del servizio non è compatibile. Chiudi Lintra e riapri l’applicazione installata.";
    if (error.code === "IPC_BUSY") return "Il servizio è occupato. Chiudi Lintra e riapri l’applicazione.";
  }
  return "Non è stato possibile avviare il servizio. Chiudi Lintra e riapri l’applicazione.";
}
