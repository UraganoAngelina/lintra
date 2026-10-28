"""Check the v1 fixture using Python's independent standard-library HMAC.

Only public synthetic test material is used. This does not import the Rust
implementation or rewrite the fixture: a framing regression must fail review.
Run from any directory with Python 3; successful output contains no key bytes.
"""
import hashlib
import hmac
import json
from pathlib import Path
import struct


def component(value):
    """Length-prefix original bytes, independent of host endian/pointer width."""
    return struct.pack(">Q", len(value)) + value


def frame(purpose):
    """Protocol header binds the application domain, purpose and key label."""
    return b"".join(component(v) for v in (
        b"LINTRA-PERSISTENCE-PROTECTION-v1", purpose, b"test-key",
    ))


def tag(message):
    """Public RFC-style 131-byte test key; never application configuration."""
    return hmac.new(bytes([0xAA]) * 131, message, hashlib.sha256).hexdigest()


def fingerprint(scheme, message):
    return {"protection_scheme": scheme, "key_id": "test-key", "value_hex": tag(message)}


def expected():
    """Encode the declared SYNTHETIC/v1/T1 scope and logical 001A record.

    The schema strips F1 leading zeroes and F2 trailing spaces. JoinKey itself
    length-prefixes each logical component before protection frames that key.
    """
    scope = struct.pack(">Q", 1) + component(b"unit") + b"S" + component(b"UNIT-A")
    scope += component(b"SYNTHETIC") + b"V" + component(b"v1")
    scope += component(b"T1") + component(b"record_key")
    pk = component(b"1")
    projection = component(b"A")
    definition = hashlib.sha256(b"SYNTHETIC/v1/T1:F2:logical:utf8").digest()
    return {
        "projection_definition": definition.hex(),
        "audit": {
            "policy_id": "lintra.hmac-sha256.v1",
            "key_confirmation": fingerprint(
                "lintra.hmac-sha256.confirm.v1", frame(b"key-confirmation"),
            ),
        },
        "key_fingerprint": fingerprint(
            "lintra.hmac-sha256.key.v1", frame(b"key") + scope + component(pk),
        ),
        "record_fingerprint": fingerprint(
            "lintra.hmac-sha256.record.v1:" + definition.hex(),
            frame(b"record") + scope + component(pk) + component(definition) + component(projection),
        ),
    }


if __name__ == "__main__":
    root = Path(__file__).resolve().parents[2]
    fixture = json.loads((root / "tests/fixtures/persistence/protected-projection.json").read_text())
    if fixture != expected():
        raise SystemExit("FAIL: protected projection fixture differs from independent protocol oracle")
    print("PASS: protected projection fixture matches independent Python HMAC framing")
