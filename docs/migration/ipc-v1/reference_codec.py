"""Executable P2-02 design reference, not a product IPC handler or dispatcher."""
import json
import re

MAX_BYTES = 1_048_576
MAX_DEPTH = 32
MAX_SAFE_INTEGER = 9_007_199_254_740_991
ERROR_CODES = frozenset({
    "IPC_UNSUPPORTED_VERSION", "IPC_INVALID_MESSAGE", "IPC_PAYLOAD_TOO_LARGE",
    "IPC_UNKNOWN_OPERATION", "IPC_UNAUTHORIZED", "IPC_STALE_CONTEXT",
    "IPC_BUSY", "IPC_INVALID_STATE", "IPC_TECHNICAL_FAILURE",
})
IDENTIFIER = re.compile(r"[A-Za-z0-9_-]{1,64}", re.ASCII)
OPERATION = re.compile(r"[a-z][a-z0-9_]{0,63}", re.ASCII)


class Rejected(ValueError):
    def __init__(self, code):
        self.code = code
        super().__init__(code)


def invalid(*_):
    raise Rejected("IPC_INVALID_MESSAGE")


def unique_object(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            invalid()
        result[key] = value
    return result


def check_depth(text):
    # Count before json.loads, ignoring delimiters inside escaped JSON strings.
    depth = 0
    quoted = escaped = False
    for char in text:
        if quoted:
            if escaped:
                escaped = False
            elif char == "\\":
                escaped = True
            elif char == '"':
                quoted = False
        elif char == '"':
            quoted = True
        elif char in "[{":
            depth += 1
            if depth > MAX_DEPTH:
                invalid()
        elif char in "]}":
            depth -= 1
            if depth < 0:
                invalid()
    if depth != 0 or quoted:
        invalid()


def check_scalars(value):
    if isinstance(value, str):
        try:
            value.encode("utf-8", errors="strict")
        except UnicodeEncodeError:
            invalid()
    elif type(value) is int:
        if abs(value) > MAX_SAFE_INTEGER:
            invalid()
    elif isinstance(value, list):
        for item in value:
            check_scalars(item)
    elif isinstance(value, dict):
        for key, item in value.items():
            check_scalars(key)
            check_scalars(item)


def decode(data, direction):
    """Check bytes and envelope only. A payload DTO never gains authority here."""
    if not isinstance(data, bytes) or direction not in {"request", "response"}:
        invalid()
    if len(data) > MAX_BYTES:
        raise Rejected("IPC_PAYLOAD_TOO_LARGE")
    try:
        text = data.decode("utf-8", errors="strict")
        check_depth(text)
        packet = json.loads(text, object_pairs_hook=unique_object,
                            parse_constant=invalid, parse_float=invalid)
    except (UnicodeDecodeError, ValueError, RecursionError) as error:
        if isinstance(error, Rejected):
            raise
        invalid()
    if not isinstance(packet, dict):
        invalid()
    version = packet.get("wire_version")
    if type(version) is not int or not 1 <= version <= 65535:
        invalid()
    if version != 1:
        raise Rejected("IPC_UNSUPPORTED_VERSION")
    kind = packet.get("kind")
    if direction == "request":
        expected = {"wire_version", "request_id", "kind", "operation", "payload"}
        if kind != "request" or set(packet) != expected:
            invalid()
        if not isinstance(packet["operation"], str) or not OPERATION.fullmatch(packet["operation"]):
            invalid()
    elif kind == "result":
        expected = {"wire_version", "request_id", "kind", "payload"}
        if set(packet) != expected:
            invalid()
    elif kind == "error":
        expected = {"wire_version", "request_id", "kind", "error"}
        if set(packet) != expected:
            invalid()
        error = packet["error"]
        if not isinstance(error, dict) or set(error) != {"code"}:
            invalid()
        if not isinstance(error["code"], str) or error["code"] not in ERROR_CODES:
            invalid()
    else:
        invalid()
    identifier = packet["request_id"]
    if not (direction == "response" and kind == "error" and identifier is None):
        if not isinstance(identifier, str) or not IDENTIFIER.fullmatch(identifier):
            invalid()
    if "payload" in packet and not isinstance(packet["payload"], dict):
        invalid()
    check_scalars(packet)
    return packet
