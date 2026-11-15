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
type Operation = "app_info" | "client_ready";
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

export function encodeRequest(operation: Operation, requestId: string): Uint8Array {
  if (!/^[A-Za-z0-9_-]{1,64}$/.test(requestId)
      || (operation !== "app_info" && operation !== "client_ready")) invalid();
  const bytes = new TextEncoder().encode(JSON.stringify({
    schemaVersion: 1, request_id: requestId, kind: "request", operation,
    payload: { schemaVersion: 1 },
  }));
  if (bytes.byteLength > MAX_BYTES) throw new IpcError("IPC_PAYLOAD_TOO_LARGE");
  return bytes;
}

async function exchange(operation: Operation): Promise<ObjectDto> {
  const requestId = crypto.randomUUID();
  const bytes = encodeRequest(operation, requestId);
  let raw: unknown;
  try { raw = await invoke<unknown>("lintra_ipc", bytes); }
  catch { throw new IpcError("IPC_TECHNICAL_FAILURE"); }
  return decodeReply(raw, requestId);
}

export async function appInfo(): Promise<AppInfo> {
  const dto = await exchange("app_info");
  exact(dto, ["schemaVersion", "app_version", "validation_available"]);
  if (typeof dto.app_version !== "string" || dto.app_version.length < 1
      || dto.app_version.length > 64 || !/^[A-Za-z0-9.+-]+$/.test(dto.app_version)
      || dto.validation_available !== false) invalid();
  return { schemaVersion: 1, app_version: dto.app_version, validation_available: false };
}

export async function clientReady(): Promise<void> {
  const dto = await exchange("client_ready");
  exact(dto, ["schemaVersion", "ready"]);
  if (dto.ready !== true) invalid();
}

export function errorText(error: unknown): string {
  if (error instanceof IpcError) {
    if (error.code === "IPC_UNSUPPORTED_VERSION") return "Le versioni dell’interfaccia e del servizio non coincidono. Chiudi Lintra e riapri l’applicazione installata.";
    if (error.code === "IPC_INVALID_MESSAGE" || error.code === "IPC_PAYLOAD_TOO_LARGE") return "La risposta del servizio non è compatibile. Chiudi Lintra e riapri l’applicazione installata.";
    if (error.code === "IPC_BUSY") return "Il servizio è occupato. Chiudi Lintra e riapri l’applicazione.";
  }
  return "Non è stato possibile avviare il servizio. Chiudi Lintra e riapri l’applicazione.";
}
