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
      || !["app_info", "client_ready", "catalog_page", "configuration_select", "configuration_page", "plan_create", "plan_input_choose"].includes(operation)) invalid();
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

export async function planCreate(configurationRef: string, mode: "local_only" | "full", traceRef: string | null): Promise<PlanDraft> {
  opaque(configurationRef);
  if (mode === "local_only") opaque(traceRef);
  else if (mode !== "full" || traceRef !== null) invalid();
  const dto = await exchange("plan_create", { schemaVersion: 1, configuration_ref: configurationRef, mode, trace_ref: traceRef });
  exact(dto, ["schemaVersion", "plan_ref", "configuration_ref", "mode", "assigned_input_count"]);
  if (dto.configuration_ref !== configurationRef || dto.mode !== mode) invalid();
  return { schemaVersion: 1, plan_ref: opaque(dto.plan_ref), configuration_ref: configurationRef, mode, assigned_input_count: unsigned(dto.assigned_input_count) };
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

export function errorText(error: unknown): string {
  if (error instanceof IpcError) {
    if (error.code === "IPC_UNSUPPORTED_VERSION") return "Le versioni dell’interfaccia e del servizio non coincidono. Chiudi Lintra e riapri l’applicazione installata.";
    if (error.code === "IPC_INVALID_MESSAGE" || error.code === "IPC_PAYLOAD_TOO_LARGE") return "La risposta del servizio non è compatibile. Chiudi Lintra e riapri l’applicazione installata.";
    if (error.code === "IPC_BUSY") return "Il servizio è occupato. Chiudi Lintra e riapri l’applicazione.";
  }
  return "Non è stato possibile avviare il servizio. Chiudi Lintra e riapri l’applicazione.";
}
