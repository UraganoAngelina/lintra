"""Synthetic contract vectors; no Tauri, filesystem broker or product backend."""
import json
import sys

sys.dont_write_bytecode = True
import reference_codec as codec


def wire(packet):
    return json.dumps(packet, ensure_ascii=False, separators=(",", ":")).encode("utf-8")


def main():
    results = []

    def case(name, data, expected=None, direction="request", before_parse=False):
        original = codec.json.loads
        if before_parse:
            def forbidden(*args, **kwargs):
                raise AssertionError("full parser called before byte/UTF-8/depth rejection")
            codec.json.loads = forbidden
        try:
            try:
                decoded = codec.decode(data, direction)
                actual = None
            except codec.Rejected as error:
                decoded = None
                actual = error.code
            assert actual == expected, (name, expected, actual)
        finally:
            codec.json.loads = original
        results.append({"case": name, "expected": expected or "ACCEPT_ENVELOPE_ONLY", "passed": True})
        return decoded

    base = {"wire_version": 1, "request_id": "fixture_1", "kind": "request",
            "operation": "synthetic_probe", "payload": {}}
    result = {"wire_version": 1, "request_id": "fixture_1", "kind": "result", "payload": {}}
    bad = "IPC_INVALID_MESSAGE"
    case("minimal request", wire(base))
    case("minimal result", wire(result), direction="response")
    case("UTF-8 text and escaped delimiters", wire({**base, "payload": {"text": 'è😀 ]} \\" [<script>'}}))
    case("operation registry is a separate check", wire({**base, "operation": "not_registered"}))
    for version, expected in [(0, bad), (2, "IPC_UNSUPPORTED_VERSION"),
                              (65535, "IPC_UNSUPPORTED_VERSION"), (65536, bad),
                              (True, bad), ("1", bad), (1.0, bad)]:
        case("version " + repr(version), wire({**base, "wire_version": version}), expected)
    for key in base:
        case("missing " + key, wire({k: v for k, v in base.items() if k != key}), bad)
    case("unknown envelope field", wire({**base, "unexpected": True}), bad)
    case("wrong direction", wire(result), bad)
    case("request decoded as response", wire(base), bad, direction="response")
    case("null payload", wire({**base, "payload": None}), bad)
    case("array payload", wire({**base, "payload": []}), bad)
    for identifier in ["", "x" * 65, "è", "a\nb"]:
        case("invalid request id " + repr(identifier[:4]), wire({**base, "request_id": identifier}), bad)
    case("64 character request id", wire({**base, "request_id": "x" * 64}))
    case("64 character operation", wire({**base, "operation": "a" * 64}))
    for operation in ["", "a" * 65, "../read", "plugin:shell|execute", "Uppercase"]:
        case("invalid operation " + repr(operation[:12]), wire({**base, "operation": operation}), bad)
    for raw in [b"", b"null", b"[]", b"{", b"{}{}", b"\xef\xbb\xbf" + wire(base)]:
        case("invalid JSON " + repr(raw[:6]), raw, bad)
    case("invalid UTF-8 rejected before parse", b"\xff", bad, before_parse=True)
    duplicate = wire(base).replace(b'"wire_version":1', b'"wire_version":1,"wire_version":1')
    case("duplicate envelope key", duplicate, bad)
    duplicate = wire({**base, "payload": {"x": 1}}).replace(b'"x":1', b'"x":1,"x":2')
    case("duplicate payload key", duplicate, bad)
    for number, expected in [(9_007_199_254_740_991, None), (-9_007_199_254_740_991, None),
                             (9_007_199_254_740_992, bad), (1.25, bad), (float("nan"), bad),
                             (float("inf"), bad)]:
        case("JSON number " + repr(number), wire({**base, "payload": {"n": number}}), expected)
    preserved = case("u64 preserved as text", wire({**base, "payload": {"n": "18446744073709551615"}}))
    assert preserved["payload"]["n"] == "18446744073709551615"
    case("lone surrogate", wire({**base, "payload": {"text": "sentinel"}}).replace(b'sentinel', b'\\ud800'), bad)
    # Root envelope and payload account for two container levels.
    for arrays, expected in [(30, None), (31, bad)]:
        nested = b"[" * arrays + b"0" + b"]" * arrays
        raw = wire({**base, "payload": {"x": "nested"}}).replace(b'"nested"', nested)
        case("container depth " + str(arrays + 2), raw, expected, before_parse=expected is not None)
    for direction, packet in [("request", base), ("response", result)]:
        empty = wire({**packet, "payload": {"text": ""}})
        for length, expected in [(1_048_575, None), (1_048_576, None),
                                 (1_048_577, "IPC_PAYLOAD_TOO_LARGE")]:
            raw = wire({**packet, "payload": {"text": "a" * (length - len(empty))}})
            assert len(raw) == length
            case(direction + " byte length " + str(length), raw, expected,
                 direction=direction, before_parse=expected is not None)
    # Characters and bytes must not be confused for non-ASCII messages.
    raw = wire({**base, "payload": {"text": "è" * 524_288}})
    assert len(raw.decode("utf-8")) < 1_048_576 < len(raw)
    case("multi-byte payload limit", raw, "IPC_PAYLOAD_TOO_LARGE", before_parse=True)
    codes = ["IPC_UNSUPPORTED_VERSION", "IPC_INVALID_MESSAGE", "IPC_PAYLOAD_TOO_LARGE",
             "IPC_UNKNOWN_OPERATION", "IPC_UNAUTHORIZED", "IPC_STALE_CONTEXT",
             "IPC_BUSY", "IPC_INVALID_STATE", "IPC_TECHNICAL_FAILURE"]
    for code in codes:
        packet = {"wire_version": 1, "request_id": None, "kind": "error", "error": {"code": code}}
        case("bounded error " + code, wire(packet), direction="response")
    packet = {"wire_version": 1, "request_id": "fixture_1", "kind": "error",
              "error": {"code": "IPC_TECHNICAL_FAILURE"}}
    case("correlated error", wire(packet), direction="response")
    case("no arbitrary diagnostic text", wire({**packet, "error": {**packet["error"], "message": "synthetic path"}}), bad, direction="response")
    case("normative outcome is not a transport error", wire({**packet, "error": {"code": "CompletedInvalid"}}), bad, direction="response")
    case("result requires correlation", wire({**result, "request_id": None}), bad, direction="response")
    print(json.dumps({"passed": True, "vectors": len(results), "scope": "design reference only; no product/native gate", "results": results}, indent=2))


if __name__ == "__main__":
    main()
