"""P2-05 disk-backed design reference, not the production diagnostic decoder."""
from collections import Counter, OrderedDict
from copy import deepcopy
import json
import secrets

from lifecycle_reference import MAX_U64, validate_ref
from reference_codec import IDENTIFIER, Rejected, decode, require_version, unique_object

MAX_PAGE_ROWS = 128  # Matches the existing Slint JsonlModel cache bound.
DEFAULT_PAGE_ROWS = 64
MAX_CACHED_PAGES = 2
PREVIEW_BYTES = 256
FILTER_FIELDS = {"schemaVersion", "code", "field", "record", "fiscal_code", "line", "trace_ref"}


def decimal(value):
    if (not isinstance(value, str) or not value.isascii() or not value.isdigit()
            or (len(value) > 1 and value[0] == "0") or len(value) > 20 or int(value) > MAX_U64):
        raise Rejected("IPC_INVALID_MESSAGE")
    return int(value)


def preview(text, limit=PREVIEW_BYTES):
    if not isinstance(text, str):
        raise ValueError("invalid captured metadata")
    raw = text.encode("utf-8", errors="strict")
    return {"schemaVersion": 1, "text": raw[:limit].decode("utf-8", errors="ignore"),
            "complete": len(raw) <= limit}


# Amendment of 24 Nov 2026 (T-MIG-P6-01, author decision): the row carries
# the reason (message, expected, displayed value found) as shorter previews,
# so a page of 128 rows at worst-case escaping still fits 1 MiB.
ROW_TEXT_BYTES = 128


class Paging:
    def __init__(self, owner, file, run, flow, version, traces, total, by_code):
        # File/summary come from an owned completed report in Rust, never IPC.
        validate_ref(run)
        self.owner, self.file, self.run = owner, file, deepcopy(run)
        self.flow, self.version, self.traces = flow, version, tuple(traces)
        self.trace_handles = {"t" + str(i): trace for i, trace in enumerate(self.traces)}
        self.total, self.by_code = total, dict(by_code)
        self.generation = 0
        self.query = None
        self.pages = OrderedDict()

    def authorize(self, owner):
        if owner != self.owner:
            raise Rejected("IPC_UNAUTHORIZED")

    def record(self):
        raw = self.file.readline()
        if not raw:
            return None
        if not raw.endswith(b"\n") or b"\r" in raw:
            raise ValueError("uncommitted JSONL line")
        row = json.loads(raw.decode("utf-8"), object_pairs_hook=unique_object)
        # Canonical ValidationError::validate remains Rust's responsibility.
        if (row["flow"] != self.flow or row["schema_version"] != self.version
                or row["trace"] not in self.traces or type(row["line"]) is not int
                or not 1 <= row["line"] <= MAX_U64 or not isinstance(row["code"], str)):
            raise ValueError("scope/metadata mismatch")
        return row

    def filters(self, dto):
        if not isinstance(dto, dict) or not set(dto) <= FILTER_FIELDS:
            raise Rejected("IPC_INVALID_MESSAGE")
        require_version(dto)
        for key in ["code", "field", "record", "fiscal_code"]:
            if key in dto and not isinstance(dto[key], str):
                raise Rejected("IPC_INVALID_MESSAGE")
        line = dto.get("line")
        if line not in (None, ""):
            # Reuse the legacy input grammar: ASCII digits, positive u64;
            # leading zeros are harmless here, unlike canonical wire counters.
            if (not isinstance(line, str) or not line.isascii() or not line.isdigit()
                    or not line.lstrip("0") or len(line.lstrip("0")) > 20
                    or int(line.lstrip("0")) > MAX_U64):
                raise Rejected("IPC_INVALID_MESSAGE")
        trace = dto.get("trace_ref")
        if trace is not None and (not isinstance(trace, str) or not IDENTIFIER.fullmatch(trace)):
            raise Rejected("IPC_INVALID_MESSAGE")
        if trace is not None and trace not in self.trace_handles:
            raise Rejected("IPC_STALE_CONTEXT")
        return deepcopy(dto)

    def matches(self, row, filters):
        identity = row["record_identity"]
        selected = {"code": row["code"], "field": row["field"]["id"] if row["field"] else None,
                    "record": identity["sioc_record_id"], "fiscal_code": identity["fiscal_code"]}
        for key, actual in selected.items():
            if filters.get(key, "") != "" and filters[key] != actual:
                return False
        if filters.get("line") not in (None, "") and int(filters["line"].lstrip("0")) != row["line"]:
            return False
        trace = filters.get("trace_ref")
        return trace is None or self.trace_handles[trace] == row["trace"]

    def open_query(self, owner, filters):
        self.authorize(owner)
        filters = self.filters(filters)
        if self.generation == MAX_U64:
            raise Rejected("IPC_TECHNICAL_FAILURE")
        offsets, codes, count = [], Counter(), 0
        try:
            self.file.seek(0)
            while True:
                offset = self.file.tell()
                row = self.record()
                if row is None:
                    break
                count += 1
                codes[row["code"]] += 1
                if self.matches(row, filters):
                    offsets.append(offset)
            if count != self.total or codes != self.by_code:
                raise ValueError("summary/count mismatch")
        except (OSError, ValueError, KeyError, TypeError, Rejected) as error:
            raise Rejected("IPC_TECHNICAL_FAILURE") from error
        # A failed candidate never replaces the previous immutable query.
        try:
            query_ref = "q_" + secrets.token_hex(16)
        except OSError as error:
            raise Rejected("IPC_TECHNICAL_FAILURE") from error
        if self.query is not None and query_ref == self.query["ref"]:
            raise Rejected("IPC_TECHNICAL_FAILURE")
        self.generation += 1
        self.query = {"ref": query_ref, "filters": filters,
                      "offsets": offsets, "page_generation": 0}
        self.pages.clear()
        return {"schemaVersion": 1, "query_ref": self.query["ref"], "run": deepcopy(self.run),
                "total_matches": str(len(offsets)), "total_diagnostics": str(count)}

    def matching_query(self, owner, query_ref):
        self.authorize(owner)
        if not isinstance(query_ref, str) or not IDENTIFIER.fullmatch(query_ref):
            raise Rejected("IPC_INVALID_MESSAGE")
        if self.query is None or query_ref != self.query["ref"]:
            raise Rejected("IPC_STALE_CONTEXT")
        return self.query

    def page(self, owner, query_ref, offset, limit=DEFAULT_PAGE_ROWS):
        query = self.matching_query(owner, query_ref)
        start = decimal(offset)
        if type(limit) is not int or not 1 <= limit <= MAX_PAGE_ROWS or start > len(query["offsets"]):
            raise Rejected("IPC_INVALID_MESSAGE")
        if query["page_generation"] == MAX_U64:
            raise Rejected("IPC_TECHNICAL_FAILURE")
        generation = query["page_generation"] + 1
        end = min(start + limit, len(query["offsets"]))
        rows, handles = [], {}
        try:
            for position in range(start, end):
                self.file.seek(query["offsets"][position])
                row = self.record()
                if row is None or not self.matches(row, query["filters"]):
                    raise ValueError("immutable report changed")
                token = "d_" + secrets.token_hex(16)
                if token in handles or any(token in page for page in self.pages.values()):
                    raise ValueError("native handle collision")
                trace_ref = next(key for key, value in self.trace_handles.items() if value == row["trace"])
                field = row["field"]
                rows.append({"schemaVersion": 1, "diagnostic_ref": token, "ordinal": str(position),
                             "line": str(row["line"]), "trace_ref": trace_ref,
                             "code": preview(row["code"]),
                             "field": preview(field["name"]) if field is not None else None,
                             "message": preview(row["message"], ROW_TEXT_BYTES),
                             "expected": preview(row["expected"], ROW_TEXT_BYTES) if row["expected"] is not None else None,
                             "actual": preview(row["actual"]["display"], ROW_TEXT_BYTES) if row["actual"] is not None else None})
                handles[token] = position
        except (OSError, ValueError, KeyError, TypeError, StopIteration, Rejected) as error:
            raise Rejected("IPC_TECHNICAL_FAILURE") from error
        payload = {"schemaVersion": 1, "query_ref": query_ref, "run": deepcopy(self.run),
                   "page_generation": str(generation), "offset": offset, "rows": rows,
                   "next_offset": str(end), "has_more": end < len(query["offsets"]),
                   "total_matches": str(len(query["offsets"])), "total_diagnostics": str(self.total)}
        # Check the entire encoded envelope before publishing handles/state.
        frame = {"schemaVersion": 1, "request_id": "x" * 64, "kind": "result", "payload": payload}
        decode(json.dumps(frame, ensure_ascii=False, separators=(",", ":")).encode(), "response")
        query["page_generation"] = generation
        self.pages[(start, limit)] = handles
        self.pages.move_to_end((start, limit))
        if len(self.pages) > MAX_CACHED_PAGES:
            self.pages.popitem(last=False)
        return payload

    def selected(self, owner, query_ref, diagnostic_ref):
        query = self.matching_query(owner, query_ref)
        if not isinstance(diagnostic_ref, str) or not IDENTIFIER.fullmatch(diagnostic_ref):
            raise Rejected("IPC_INVALID_MESSAGE")
        for handles in self.pages.values():
            if diagnostic_ref in handles:
                position = handles[diagnostic_ref]
                try:
                    self.file.seek(query["offsets"][position])
                    row = self.record()
                    if row is None or not self.matches(row, query["filters"]):
                        raise ValueError("report changed")
                    # Private validated row for P2-06, never a serializable DTO.
                    return row
                except (OSError, ValueError, KeyError, TypeError, Rejected) as error:
                    raise Rejected("IPC_TECHNICAL_FAILURE") from error
        raise Rejected("IPC_STALE_CONTEXT")
