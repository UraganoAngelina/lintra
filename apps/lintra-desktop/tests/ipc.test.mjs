import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { afterEach, test } from "node:test";
import { mockIPC, clearMocks } from "@tauri-apps/api/mocks";
import { appInfo, clientReady, decodeReply, encodeRequest, errorText, IpcError } from "../src/ipc.ts";

globalThis.window = { crypto: globalThis.crypto };
afterEach(clearMocks);
const id = "fixture_1";
const raw = text => new TextEncoder().encode(text).buffer;
const frame = payload => ({ schemaVersion: 1, request_id: id, kind: "result", payload });
const info = { schemaVersion: 1, app_version: "0.1.1-rc.3", validation_available: false };
const reject = (run, code = "IPC_INVALID_MESSAGE") => assert.throws(run,
  error => error instanceof IpcError && error.code === code);

for (const vector of JSON.parse(readFileSync(new URL("./ipc-vectors.json", import.meta.url), "utf8"))) {
  test(`Rust/TS shared vector: ${vector.name}`, () => {
    const response = raw(JSON.stringify(vector.response));
    if (vector.response.kind === "error") {
      reject(() => decodeReply(response, id), vector.response.error.code);
    } else {
      assert.deepEqual(JSON.parse(new TextDecoder().decode(encodeRequest("app_info", id))), JSON.parse(vector.request));
      assert.deepEqual(decodeReply(response, id), vector.response.payload);
    }
  });
}

test("only raw bytes pass the UTF-8 and whole-message limits", () => {
  for (const value of ["{}", {}, [], new Uint8Array([123]), null, raw("\ufeff{}"), new Uint8Array([0xff]).buffer]) {
    reject(() => decodeReply(value, id));
  }
  const empty = JSON.stringify(frame({ schemaVersion: 1, padding: "" }));
  const atLimit = raw(JSON.stringify(frame({ schemaVersion: 1, padding: "x".repeat(1_048_576 - empty.length) })));
  assert.equal(atLimit.byteLength, 1_048_576);
  assert.equal(decodeReply(atLimit, id).padding.length, 1_048_576 - empty.length);
  reject(() => decodeReply(new Uint8Array(1_048_577).buffer, id), "IPC_PAYLOAD_TOO_LARGE");
  assert.equal(decodeReply(raw(JSON.stringify(frame({ schemaVersion: 1, text: "è😊{[\\\"" }))), id).text, "è😊{[\\\"");
});

test("depth, duplicate keys, exact integers and Unicode are checked before DTO use", () => {
  for (const [count, valid] of [[30, true], [31, false]]) {
    const text = `{"schemaVersion":1,"request_id":"${id}","kind":"result","payload":{"schemaVersion":1,"nested":${"[".repeat(count)}0${"]".repeat(count)}}}`;
    if (valid) assert.ok(decodeReply(raw(text), id));
    else reject(() => decodeReply(raw(text), id));
  }
  for (const text of [
    '{"schemaVersion":1,"schemaVersion":1}',
    '{"a":{"key":1,"\\u006bey":2}}',
    '{"s":"\\ud800"}', '{"s":"\\udc00"}', '{"s":"\\ud800X"}',
    '{"n":1.0}', '{"n":1e0}', '{"n":9007199254740992}', '{"n":-9007199254740992}',
    '{"n":NaN}', '{} trailing', '{', '[]', '{"s":"unterminated}',
  ]) reject(() => decodeReply(raw(text), id));
  for (const value of [-9007199254740991, 9007199254740991]) {
    assert.equal(decodeReply(raw(JSON.stringify(frame({ schemaVersion: 1, value }))), id).value, value);
  }
});

test("version, envelope shape, correlation and error category cannot silently fall back", () => {
  const valid = frame(info);
  for (const invalid of [
    { ...valid, extra: true }, { ...valid, request_id: "other" }, { ...valid, request_id: null },
    { ...valid, kind: "event" }, { ...valid, payload: [] }, { ...valid, schemaVersion: true },
    { ...valid, schemaVersion: 65536 }, { ...valid, payload: {} },
    { schemaVersion: 1, request_id: id, kind: "error", error: { schemaVersion: 1, code: "IPC_BUSY", category: "protocol" } },
    { schemaVersion: 1, request_id: id, kind: "error", error: { schemaVersion: 1, code: "CompletedInvalid", category: "technical" } },
  ]) reject(() => decodeReply(raw(JSON.stringify(invalid)), id));
  for (const changed of [{ ...valid, schemaVersion: 2 }, { ...valid, payload: { schemaVersion: 2 } }]) {
    reject(() => decodeReply(raw(JSON.stringify(changed)), id), "IPC_UNSUPPORTED_VERSION");
  }
  reject(() => encodeRequest("execute_sql", id));
  reject(() => encodeRequest("app_info", "x".repeat(65)));
});

test("typed client invokes only the raw dispatcher and verifies both exact DTOs", async () => {
  const operations = [];
  mockIPC((command, bytes) => {
    assert.equal(command, "lintra_ipc");
    assert.ok(bytes instanceof Uint8Array);
    const request = JSON.parse(new TextDecoder().decode(bytes));
    assert.deepEqual(request.payload, { schemaVersion: 1 });
    operations.push(request.operation);
    return raw(JSON.stringify({ schemaVersion: 1, request_id: request.request_id, kind: "result",
      payload: request.operation === "app_info" ? info : { schemaVersion: 1, ready: true } }));
  });
  assert.deepEqual(await appInfo(), info);
  await clientReady();
  assert.deepEqual(operations, ["app_info", "client_ready"]);
});

test("unknown fields and changed capability cannot enter the typed client", async () => {
  for (const payload of [
    { ...info, extra: true }, { ...info, app_version: "" }, { ...info, app_version: "<script>" },
    { ...info, validation_available: true }, { ...info, schemaVersion: 2 },
    { schemaVersion: 1, ready: true, extra: true }, { schemaVersion: 1, ready: false },
  ]) {
    mockIPC((_command, bytes) => {
      const request = JSON.parse(new TextDecoder().decode(bytes));
      return raw(JSON.stringify({ ...frame(payload), request_id: request.request_id }));
    });
    await assert.rejects(() => "ready" in payload ? clientReady() : appInfo(), IpcError);
  }
});

test("transport failures and every IPC error stay static, without technical or normative data", async () => {
  mockIPC(() => { throw new Error("SYNTHETIC secret path SQL record"); });
  await assert.rejects(appInfo, error => error.code === "IPC_TECHNICAL_FAILURE");
  for (const error of [
    new Error("SYNTHETIC secret path SQL record"),
    ...["IPC_UNSUPPORTED_VERSION", "IPC_INVALID_MESSAGE", "IPC_PAYLOAD_TOO_LARGE", "IPC_UNKNOWN_OPERATION",
      "IPC_UNAUTHORIZED", "IPC_STALE_CONTEXT", "IPC_BUSY", "IPC_INVALID_STATE", "IPC_TECHNICAL_FAILURE"].map(code => new IpcError(code)),
  ]) {
    assert.match(errorText(error), /Chiudi Lintra/);
    assert.doesNotMatch(errorText(error), /SYNTHETIC|SQL|record|CompletedValid|CompletedInvalid/);
  }
});
