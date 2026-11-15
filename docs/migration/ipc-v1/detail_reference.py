"""P2-06 selected diagnostic/text and captured-domain design adapters."""
from copy import deepcopy
import json
import secrets

from paging_reference import decimal, preview
from reference_codec import Rejected, decode

MAX_TEXT_BYTES = 16_384
MAX_DOMAIN_VALUES = 64


def checked_payload(payload):
    frame = {"schemaVersion": 1, "request_id": "x" * 64, "kind": "result", "payload": payload}
    decode(json.dumps(frame, ensure_ascii=False, separators=(",", ":")).encode(), "response")


def text_chunk(text, offset, limit):
    start = decimal(offset)
    if type(limit) is not int or not 1 <= limit <= MAX_TEXT_BYTES:
        raise Rejected("IPC_INVALID_MESSAGE")
    try:
        raw = text.encode("utf-8", errors="strict")
    except UnicodeError as error:
        raise Rejected("IPC_TECHNICAL_FAILURE") from error
    if start > len(raw) or (start < len(raw) and raw[start] & 0xC0 == 0x80):
        raise Rejected("IPC_INVALID_MESSAGE")
    value = raw[start:start + limit].decode("utf-8", errors="ignore")
    if not value and start < len(raw):
        raise Rejected("IPC_INVALID_MESSAGE")
    end = start + len(value.encode())
    result = {"schemaVersion": 1, "offset": offset, "text": value,
              "next_offset": str(end), "has_more": end < len(raw)}
    checked_payload(result)
    return result


class Detail:
    def __init__(self, view, proper_field_subsets, guides):
        # Coverage/guide are computed from the captured compiled schema in Rust.
        self.view = view
        self.subsets = dict(proper_field_subsets)
        self.guides = dict(guides)
        self.selection = None

    def select(self, owner, query_ref, diagnostic_ref):
        row = deepcopy(self.view.selected(owner, query_ref, diagnostic_ref))
        try:
            selection_ref = "s_" + secrets.token_hex(16)
            if self.selection is not None and selection_ref == self.selection["ref"]:
                raise ValueError("selection collision")
            components = {}
            def add(kind, text):
                if text is None:
                    return
                token = "x_" + secrets.token_hex(16)
                if token in components:
                    raise ValueError("component collision")
                components[token] = {"kind": kind, "text": text}
            add("code", row["code"])
            add("message", row["message"])
            add("expected", row["expected"])
            field = row["field"]
            key = (row["trace"], field["id"]) if field else None
            if field:
                add("field_name", field["name"])
                add("guide", self.guides.get(key))
            actual = row["actual"]
            allowed = field is not None and self.subsets.get(key) == (field["start"], field["end"])
            # Generic record-length diagnostics contain a byte count, not input.
            count_metadata = (not field and row["code"] == "E_RECORD_LENGTH" and actual is not None
                              and actual["raw_hex"] is None and actual["display"].isascii()
                              and actual["display"].isdigit())
            if actual is not None and (allowed or count_metadata):
                add("actual", actual["display"])
            # Individual identity components for the selected diagnostic only;
            # no wildcard technical_ids map or raw record is exposed.
            for name in ["sioc_record_id", "fiscal_code", "episode_id"]:
                add(name, row["record_identity"][name])
            dto = {"schemaVersion": 1, "selection_ref": selection_ref, "query_ref": query_ref,
                   "diagnostic_ref": diagnostic_ref, "run": deepcopy(self.view.run), "line": str(row["line"]),
                   "field_position": {"schemaVersion": 1, "start": str(field["start"]), "end": str(field["end"]),
                                      "units": "one_based_inclusive_bytes"} if field else None,
                   "actual_visibility": "available" if actual is not None and (allowed or count_metadata)
                                        else "not_present" if actual is None else "local_report_only_record_scope",
                   "components": [{"schemaVersion": 1, "component_ref": token, "kind": value["kind"],
                                   "preview": preview(value["text"])} for token, value in components.items()]}
            checked_payload(dto)
        except (OSError, UnicodeError, ValueError, KeyError, TypeError) as error:
            raise Rejected("IPC_TECHNICAL_FAILURE") from error
        self.selection = {"ref": selection_ref, "query_ref": query_ref, "row": row,
                          "components": components, "domain_components": {}, "context_components": {}}
        return dto

    def matching(self, owner, selection_ref):
        self.view.authorize(owner)
        selected = self.selection
        if (selected is None or selection_ref != selected["ref"] or self.view.query is None
                or selected["query_ref"] != self.view.query["ref"]):
            raise Rejected("IPC_STALE_CONTEXT")
        return selected

    def read_text(self, owner, selection_ref, component_ref, offset, limit=MAX_TEXT_BYTES):
        selected = self.matching(owner, selection_ref)
        if not isinstance(component_ref, str):
            raise Rejected("IPC_INVALID_MESSAGE")
        component = (selected["components"].get(component_ref) or selected["domain_components"].get(component_ref)
                     or selected["context_components"].get(component_ref))
        if component is None:
            raise Rejected("IPC_STALE_CONTEXT")
        return text_chunk(component["text"], offset, limit)

    def context_page(self, owner, selection_ref, offset, limit=MAX_DOMAIN_VALUES):
        selected = self.matching(owner, selection_ref)
        start = decimal(offset)
        context = selected["row"]["context"]
        if type(limit) is not int or not 1 <= limit <= MAX_DOMAIN_VALUES or start > len(context):
            raise Rejected("IPC_INVALID_MESSAGE")
        components, entries = {}, []
        end = min(start + limit, len(context))
        try:
            for item in context[start:end]:
                token = None
                # A context value also needs captured field coverage; never a full record.
                allowed = (item["trace"], item["field_id"]) in self.subsets
                if allowed:
                    token = "k_" + secrets.token_hex(16)
                    if token in components or token in selected["context_components"]:
                        raise ValueError("context handle collision")
                    components[token] = {"kind": "context_value", "text": item["value"]["display"]}
                entries.append({"schemaVersion": 1, "trace": preview(item["trace"]),
                                "field": preview(item["field_id"]), "component_ref": token,
                                "value": preview(item["value"]["display"]) if allowed else None,
                                "visibility": "available" if allowed else "local_report_only_record_scope"})
            result = {"schemaVersion": 1, "selection_ref": selection_ref, "offset": offset,
                      "entries": entries, "next_offset": str(end), "has_more": end < len(context)}
            checked_payload(result)
        except (OSError, UnicodeError, ValueError, KeyError, TypeError) as error:
            raise Rejected("IPC_TECHNICAL_FAILURE") from error
        selected["context_components"] = components
        return result


class DomainPages:
    def __init__(self, detail, owner, selection_ref, source, reader):
        selected = detail.matching(owner, selection_ref)
        if (source["kind"] not in {"allowed_values", "range", "fallback", "reference", "regex", "non_enumerable"}
                or type(source.get("conditional", False)) is not bool
                or (source["kind"] == "reference" and source.get("availability") not in
                    {"verified", "degraded", "unavailable", "not_captured"})):
            raise Rejected("IPC_TECHNICAL_FAILURE")
        self.detail, self.owner, self.selection_ref = detail, owner, selection_ref
        self.source, self.reader = deepcopy(source), reader
        self.cursor_ref = None
        self.position = None
        selected["domain_source"] = self
        selected["domain_components"] = {}

    def page(self, owner, cursor_ref=None, cancelled=False):
        selected = self.detail.matching(owner, self.selection_ref)
        if owner != self.owner:
            raise Rejected("IPC_UNAUTHORIZED")
        if selected.get("domain_source") is not self:
            raise Rejected("IPC_STALE_CONTEXT")
        if cursor_ref is not None and (self.cursor_ref is None or cursor_ref != self.cursor_ref):
            raise Rejected("IPC_STALE_CONTEXT")
        source = self.source
        metadata = {"schemaVersion": 1, "selection_ref": self.selection_ref,
                    "kind": source["kind"], "conditional": source.get("conditional", False),
                    "expected_sha256": source.get("expected_sha256"), "actual_sha256": source.get("actual_sha256")}
        status = None
        if selected["row"]["field"] is None:
            status = "record_scope"
        elif cancelled:
            status = "cancelled"
        elif source["kind"] in {"regex", "non_enumerable"}:
            status = "not_enumerable"
        elif source["kind"] == "reference" and source["availability"] != "verified":
            status = source["availability"]
        if status is not None:
            result = {**metadata, "status": status, "entries": None, "next_cursor": None}
            checked_payload(result)
            selected["domain_components"] = {}
            self.cursor_ref, self.position = None, None
            return result
        if source["kind"] == "reference" and (
                source.get("actual_sha256") is None or source["actual_sha256"] != source.get("expected_sha256")):
            raise Rejected("IPC_TECHNICAL_FAILURE")
        try:
            page = self.reader(self.position if cursor_ref is not None else None, MAX_DOMAIN_VALUES)
            if page is None:
                result = {**metadata, "status": "unsupported", "entries": None, "next_cursor": None}
                checked_payload(result)
                selected["domain_components"] = {}
                self.cursor_ref, self.position = None, None
                return result
            values, position = page
            if len(values) > MAX_DOMAIN_VALUES or (not values and position is not None):
                raise ValueError("provider exceeded its native page contract")
            components, entries = {}, []
            for value in values:
                token = "v_" + secrets.token_hex(16)
                if token in components or token in selected["domain_components"]:
                    raise ValueError("member handle collision")
                components[token] = {"kind": "domain_value", "text": value}
                entries.append({"schemaVersion": 1, "component_ref": token, "preview": preview(value)})
            cursor = "c_" + secrets.token_hex(16) if position is not None else None
            if cursor is not None and cursor == self.cursor_ref:
                raise ValueError("cursor collision")
            result = {**metadata, "status": "available" if values else "available_empty",
                      "entries": entries, "next_cursor": cursor}
            checked_payload(result)
        except (OSError, UnicodeError, ValueError, KeyError, TypeError) as error:
            raise Rejected("IPC_TECHNICAL_FAILURE") from error
        # One selected domain page and one opaque continuation, never all members.
        selected["domain_components"] = components
        self.cursor_ref, self.position = cursor, position
        return result
