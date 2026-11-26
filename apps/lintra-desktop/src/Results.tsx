// Risultati section (T-MIG-P6-01): summary cards, charts and the diagnostics
// grid. Rust builds the query, orders it and serves bounded pages (PAGING.md);
// the grid shows one logical page of at most 128 rows and keeps in the DOM
// only the rows in view (TanStack Virtual), so DOM and memory stay bounded for
// any report size. Columns hold only what a row DTO carries: line, trace
// (named through results_overview), field, code and the reason as previews
// (author decision of 24 Nov 2026); raw bytes, context and identity stay Rust.
import { useCallback, useEffect, useMemo, useRef, useState, type FormEvent, type KeyboardEvent } from "react";
import { createColumnHelper, tableFeatures, useTable } from "@tanstack/react-table";
import { useVirtualizer } from "@tanstack/react-virtual";
import { ChevronLeft, ChevronRight } from "lucide-react";
import { resultsContext, resultsDomain, resultsDomainState, resultsOverview, resultsPage, resultsQuery, resultsQueryState, resultsSelect, resultsSources, resultsText, type ContextPage, type DetailComponent, type DomainPage, type DomainSource, type Preview, type ResultRow, type ResultsOverview, type ResultsPage, type RunRef, type SelectedDiagnostic, type SourcesPage, type TextChunk } from "./ipc";
import { Charts, type ChartScope } from "./Charts";
import { previewText, reportStatusText, type AboutReport } from "./information";
import { NO_FILTERS, PAGE_ROWS, acceptPage, cached, filtering, pageCount, pageItems, pageOffset, rangeText, remember, sameRunRef, validLine, type ResultsFilters, type ResultsView } from "./pages";
import { Badge, Button, Card, cn } from "./ui";

/** Interval between query state reads while Rust builds the index. */
const POLL_MS = 200;
/** Fixed row height (px): two text lines at most (message, expected/found),
 * each cut with an ellipsis, so no measuring is needed. */
export const ROW_HEIGHT = 56;
/** Rows rendered beyond the viewport on each side. */
export const OVERSCAN = 6;

/**
 * Opens the Rust query of `filters` on `report`, polls it while Rust builds
 * the index and reads its first page. Every reply is used only while `live()`
 * holds: the caller turns it off when the report, the filters or the
 * visibility change, so a late state or page of the query left behind never
 * reaches the view (stale-safe, T-MIG-P6-02). Returns the timer cleanup.
 */
export function openQuery(report: RunRef, filters: ResultsFilters, live: () => boolean,
  set: (update: ResultsView | ((view: ResultsView) => ResultsView)) => void, keep: (page: ResultsPage) => void): () => void {
  let timer: ReturnType<typeof setTimeout> | undefined;
  set({ status: "building", run: report });
  const ready = async (queryRef: string, matches: string, total: string) => {
    set({ status: "ready", run: report, query_ref: queryRef, total_matches: matches, total_diagnostics: total,
      page: 0n, loading: true, pageFailed: false, current: null });
    const page = await resultsPage(queryRef, "0", PAGE_ROWS);
    if (!live()) return;
    keep(page);
    set(current => acceptPage(current, page));
  };
  const fail = () => { if (live()) set({ status: "failed", run: report }); };
  const poll = (queryRef: string) => {
    timer = setTimeout(() => {
      resultsQueryState(queryRef).then(status => {
        if (!live()) return;
        if (status.state === "building") poll(queryRef);
        else if (status.state === "ready" && status.total_matches !== null && status.total_diagnostics !== null) {
          return ready(queryRef, status.total_matches, status.total_diagnostics);
        } else fail();
      }).catch(fail);
    }, POLL_MS);
  };
  resultsQuery(report, { schemaVersion: 1, ...filters }).then(status => {
    if (!live()) return;
    if (status.state === "building") poll(status.query_ref);
    else if (status.total_matches !== null && status.total_diagnostics !== null) {
      return ready(status.query_ref, status.total_matches, status.total_diagnostics);
    }
  }).catch(fail);
  return () => { if (timer !== undefined) clearTimeout(timer); };
}

/**
 * Query of the report Rust holds as last published, built while the section
 * is visible, with the filters last applied to that report (T-MIG-P6-02).
 * Applying filters opens a new Rust query; every reply of the query left
 * behind (state, first page, a page asked for later) is dropped, so rows of
 * an old query or report never appear under the filters just applied. A new
 * report starts unfiltered, as in Slint.
 */
export function useResults(report: RunRef | null, visible: boolean):
  [ResultsView, (page: bigint) => void, () => void, ResultsFilters, (filters: ResultsFilters) => void] {
  const [view, setView] = useState<ResultsView>({ status: "none" });
  const [applied, setApplied] = useState<{ run: RunRef | null; filters: ResultsFilters }>({ run: null, filters: NO_FILTERS });
  const filters = report !== null && applied.run !== null && sameRunRef(applied.run, report) ? applied.filters : NO_FILTERS;
  const latest = useRef(view);
  latest.current = view;
  const cache = useRef<ResultsPage[]>([]);
  const [attempt, setAttempt] = useState(0);
  useEffect(() => {
    cache.current = [];
    if (report === null || !visible) { setView({ status: "none" }); return; }
    let active = true;
    const stop = openQuery(report, filters, () => active, setView, page => { cache.current = remember(cache.current, page); });
    return () => { active = false; stop(); };
  }, [report, visible, attempt, filters]);
  const goTo = useCallback((page: bigint) => {
    const current = latest.current;
    if (current.status !== "ready" || page < 0n || page >= pageCount(current.total_matches)) return;
    const offset = pageOffset(page);
    const hit = cached(cache.current, current.query_ref, offset);
    if (hit) { setView(acceptPage({ ...current, page }, hit)); return; }
    setView({ ...current, page, loading: true, pageFailed: false });
    resultsPage(current.query_ref, offset, PAGE_ROWS).then(loaded => {
      // A page of a query already replaced must not evict the new query's pages.
      const shown = latest.current;
      if (shown.status === "ready" && shown.query_ref === loaded.query_ref) cache.current = remember(cache.current, loaded);
      setView(now => acceptPage(now, loaded));
    }, () => setView(now => now.status === "ready" && now.query_ref === current.query_ref && now.page === page
      ? { ...now, loading: false, pageFailed: true } : now));
  }, []);
  const retry = useCallback(() => setAttempt(value => value + 1), []);
  // A new object even for the same text: applying again opens a new query, as in Slint.
  const apply = useCallback((next: ResultsFilters) => { if (report !== null) setApplied({ run: report, filters: { ...next } }); }, [report]);
  return [view, goTo, retry, filters, apply];
}

/**
 * Row chosen in the grid (T-MIG-P6-03): the query it belongs to, its ordinal
 * in that query (the selection offset, stable across pages) and the token of
 * the page it was read from.
 */
export type Selection = { query_ref: string; ordinal: string; diagnostic_ref: string };
/** Detail of the selection, as Rust pinned it (results_select). */
export type DetailState =
  | { status: "none" }
  | { status: "loading"; selection: Selection }
  | { status: "failed"; selection: Selection }
  | { status: "ready"; selection: Selection; detail: SelectedDiagnostic };
const NO_DETAIL: DetailState = Object.freeze({ status: "none" });

/**
 * Asks Rust to pin `selection` and shows its detail. Like openQuery, every
 * reply is used only while `live()` holds, so the detail of a row left
 * behind never replaces the one just chosen.
 */
export function openDetail(selection: Selection, live: () => boolean, set: (state: DetailState) => void): void {
  set({ status: "loading", selection });
  resultsSelect(selection.query_ref, selection.diagnostic_ref).then(
    detail => { if (live()) set({ status: "ready", selection, detail }); },
    () => { if (live()) set({ status: "failed", selection }); });
}

/**
 * Selection and detail of the query shown. Rust keeps a pinned selection
 * across page changes but drops it with a new query (PAGING/DETAIL v1), so a
 * new query or report, or none, clears it and drops pending replies.
 */
export function useDetail(queryRef: string | null): [DetailState, (selection: Selection) => void, () => void] {
  const [state, setState] = useState<DetailState>(NO_DETAIL);
  const request = useRef(0);
  useEffect(() => { request.current += 1; setState(NO_DETAIL); }, [queryRef]);
  const select = useCallback((selection: Selection) => {
    const mine = ++request.current;
    openDetail(selection, () => request.current === mine, setState);
  }, []);
  const latest = useRef(state);
  latest.current = state;
  const retry = useCallback(() => { const now = latest.current; if (now.status === "failed") select(now.selection); }, [select]);
  // Until the effect runs, a selection of another query is already hidden.
  const shown = state.status !== "none" && state.selection.query_ref !== queryRef ? NO_DETAIL : state;
  return [shown, select, retry];
}

/** Overview of the published report (charts, counts, trace names). */
export type OverviewState = { status: "none" } | { status: "loading" } | { status: "failed" } | { status: "ready"; overview: ResultsOverview };

/** Reads results_overview for the report shown; a late reply is dropped. */
export function useOverview(report: RunRef | null, visible: boolean): OverviewState {
  const [state, setState] = useState<OverviewState>({ status: "none" });
  useEffect(() => {
    if (report === null || !visible) { setState({ status: "none" }); return; }
    let active = true;
    setState({ status: "loading" });
    resultsOverview(report).then(overview => { if (active) setState({ status: "ready", overview }); },
      () => { if (active) setState({ status: "failed" }); });
    return () => { active = false; };
  }, [report, visible]);
  return state;
}

/** Decimal u64 with Italian thousands grouping (no Number cast). */
export function grouped(value: string): string {
  return value.replace(/\B(?=(\d{3})+(?!\d))/g, ".");
}

/** A preview is shown as Rust sent it; an incomplete one says so. */
function PreviewText({ preview }: { preview: Preview }) {
  return <>{previewText(preview)}{!preview.complete && <span className="sr-only"> (anteprima incompleta)</span>}</>;
}

const features = tableFeatures({});
const column = createColumnHelper<typeof features, ResultRow>();
/** Trace names by handle, as results_overview lists them (never decoded). */
export type TraceNames = ReadonlyMap<string, Preview>;

/** Reason of a row: message, then what was expected and what was found. */
function Reason({ row }: { row: ResultRow }) {
  return <span className="reason">
    <span className="reason-message"><PreviewText preview={row.message} /></span>
    {(row.expected || row.actual) && <span className="reason-detail">
      {row.expected && <><span className="reason-key">Atteso</span> <PreviewText preview={row.expected} /></>}
      {row.expected && row.actual && <span aria-hidden="true"> · </span>}
      {row.actual && <><span className="reason-key">Trovato</span> <span className="font-mono"><PreviewText preview={row.actual} /></span></>}
    </span>}
  </span>;
}

/** Columns of the reference table that a row DTO can fill honestly. */
function makeColumns(traces: TraceNames) {
  return column.columns([
    column.display({ id: "ordinal", header: "N.", cell: ({ row }) => grouped((BigInt(row.original.ordinal) + 1n).toString()) }),
    column.display({ id: "line", header: "Riga", cell: ({ row }) => grouped(row.original.line) }),
    column.display({ id: "trace", header: "Tracciato", cell: ({ row }) => {
      const name = traces.get(row.original.trace_ref);
      return name ? <PreviewText preview={name} /> : <span className="text-muted">—</span>;
    } }),
    column.display({ id: "field", header: "Campo", cell: ({ row }) => row.original.field
      ? <PreviewText preview={row.original.field} /> : <span className="text-muted">—</span> }),
    column.display({ id: "code", header: "Controllo", cell: ({ row }) =>
      <Badge tone="error" className="font-mono font-normal"><PreviewText preview={row.original.code} /></Badge> }),
    column.display({ id: "reason", header: "Motivo", cell: ({ row }) => <Reason row={row.original} /> }),
  ]);
}
const NO_ROWS: ResultRow[] = [];
const NO_TRACES: TraceNames = new Map();

/**
 * One page of diagnostics. The table header stays in place; only the rows in
 * the viewport (plus OVERSCAN) are in the DOM, between two spacer rows that
 * keep the scroll height of the whole page.
 */
export function ResultsGrid({ page, traces = NO_TRACES, selected = null, onSelect }:
  { page: ResultsPage | null; traces?: TraceNames; selected?: string | null; onSelect?: ((row: ResultRow) => void) | undefined }) {
  const data = page?.rows ?? NO_ROWS;
  const columns = useMemo(() => makeColumns(traces), [traces]);
  const table = useTable({ features, columns, data, getRowId: row => row.diagnostic_ref });
  const rows = table.getRowModel().rows;
  const viewport = useRef<HTMLDivElement>(null);
  const virtualizer = useVirtualizer({
    count: rows.length, getScrollElement: () => viewport.current, estimateSize: () => ROW_HEIGHT,
    overscan: OVERSCAN, initialRect: { width: 960, height: 10 * ROW_HEIGHT },
  });
  useEffect(() => { viewport.current?.scrollTo({ top: 0 }); }, [page?.offset, page?.query_ref]);
  const items = virtualizer.getVirtualItems();
  const before = items[0]?.start ?? 0;
  const after = virtualizer.getTotalSize() - (items.at(-1)?.end ?? 0);
  const total = page ? BigInt(page.total_matches) + 1n : 1n;
  // Keys of the Slint table: arrows, Home/End and PageUp/PageDown move the
  // selection, here within the page shown (pages are changed by the pager).
  const keyDown = (event: KeyboardEvent<HTMLDivElement>) => {
    if (!onSelect || rows.length === 0) return;
    const current = rows.findIndex(row => row.original.ordinal === selected);
    const step = Math.max(1, Math.floor((viewport.current?.clientHeight ?? ROW_HEIGHT) / ROW_HEIGHT));
    const target = { ArrowDown: current < 0 ? 0 : current + 1, ArrowUp: current < 0 ? 0 : current - 1, Home: 0, End: rows.length - 1,
      PageDown: current + step, PageUp: current < 0 ? 0 : current - step }[event.key];
    if (target === undefined) return;
    event.preventDefault();
    const index = Math.min(rows.length - 1, Math.max(0, target));
    virtualizer.scrollToIndex(index);
    const row = rows[index];
    if (row && row.original.ordinal !== selected) onSelect(row.original);
  };
  return <div ref={viewport} className="grid-viewport" tabIndex={0} role="region" onKeyDown={keyDown}
    aria-label={page ? `Diagnostiche, ${rangeText(page).toLowerCase()}${onSelect ? "; frecce per scegliere una riga" : ""}` : "Diagnostiche"}>
    <table className="grid-table" role={onSelect ? "grid" : undefined} aria-readonly={onSelect ? true : undefined}
      aria-rowcount={Number(total) <= Number.MAX_SAFE_INTEGER ? Number(total) : -1}>
      <thead>
        {table.getHeaderGroups().map(group => <tr key={group.id} aria-rowindex={1}>
          {group.headers.map(header => <th key={header.id} scope="col" className={`col-${header.column.id}`}>
            {header.isPlaceholder ? null : <table.FlexRender header={header} />}
          </th>)}
        </tr>)}
      </thead>
      <tbody>
        {before > 0 && <tr aria-hidden="true"><td colSpan={columns.length} ref={cell => { if (cell) cell.style.height = `${before}px`; }} /></tr>}
        {items.map(item => {
          const row = rows[item.index];
          if (!row) return null;
          const chosen = row.original.ordinal === selected;
          return <tr key={row.id} aria-rowindex={Number(BigInt(row.original.ordinal) + 2n)}
            aria-selected={onSelect ? chosen : undefined} className={cn(onSelect && "selectable", chosen && "is-selected") || undefined}
            onClick={onSelect ? () => { if (!chosen) onSelect(row.original); } : undefined}>
            {row.getAllCells().map(cell => <td key={cell.id} className={`col-${cell.column.id}`}>
              <table.FlexRender cell={cell} />
            </td>)}
          </tr>;
        })}
        {after > 0 && <tr aria-hidden="true"><td colSpan={columns.length} ref={cell => { if (cell) cell.style.height = `${after}px`; }} /></tr>}
      </tbody>
    </table>
  </div>;
}

/** Footer: range of the page shown and the page buttons of the reference. */
function Pager({ view, onPage }: { view: Extract<ResultsView, { status: "ready" }>; onPage: (page: bigint) => void }) {
  const count = pageCount(view.total_matches);
  if (count <= 1n) return null;
  const go = (page: bigint) => () => { if (!view.loading) onPage(page); };
  return <nav className="pager" aria-label="Pagine delle diagnostiche">
    <Button variant="ghost" size="icon" aria-label="Pagina precedente" aria-disabled={view.page === 0n || view.loading}
      onClick={view.page === 0n ? undefined : go(view.page - 1n)}><ChevronLeft aria-hidden="true" /></Button>
    {pageItems(view.page, count).map((item, index) => item === "gap"
      ? <span key={`gap-${index}`} className="pager-gap" aria-hidden="true">…</span>
      : <Button key={item.toString()} variant={item === view.page ? "primary" : "ghost"} size="sm"
          aria-current={item === view.page ? "page" : undefined} aria-label={`Pagina ${grouped((item + 1n).toString())}`}
          onClick={item === view.page ? undefined : go(item)}>{grouped((item + 1n).toString())}</Button>)}
    <Button variant="ghost" size="icon" aria-label="Pagina successiva" aria-disabled={view.page >= count - 1n || view.loading}
      onClick={view.page >= count - 1n ? undefined : go(view.page + 1n)}><ChevronRight aria-hidden="true" /></Button>
  </nav>;
}

/** The five filters of the Slint view, with its labels. */
const FILTER_FIELDS: readonly (readonly [keyof ResultsFilters, string])[] = [
  ["code", "Codice"], ["field", "ID campo"], ["record", "ID record"], ["fiscal_code", "Codice fiscale"], ["line", "Riga"],
];

/**
 * Filter bar of the grid (T-MIG-P6-02), in place of the reference search box:
 * Lintra has no free-text search, only the five exact filters Rust applies.
 * Values are sent as typed (no trim, no case folding). Enter or "Applica
 * filtri" opens a new query; a line that Rust would reject is flagged next
 * to the field and nothing is sent.
 */
export function FilterBar({ applied, onApply }: { applied: ResultsFilters; onApply: (filters: ResultsFilters) => void }) {
  const [draft, setDraft] = useState(applied);
  const [lineInvalid, setLineInvalid] = useState(false);
  useEffect(() => { setDraft(applied); setLineInvalid(false); }, [applied]);
  const clearable = filtering(draft) || filtering(applied);
  const submit = (event: FormEvent) => {
    event.preventDefault();
    if (!validLine(draft.line)) { setLineInvalid(true); return; }
    onApply(draft);
  };
  const clear = () => {
    if (!clearable) return;
    setDraft(NO_FILTERS); setLineInvalid(false);
    if (filtering(applied)) onApply(NO_FILTERS);
  };
  return <form className="grid-filters" onSubmit={submit} aria-label="Filtri delle diagnostiche" noValidate>
    {FILTER_FIELDS.map(([key, label]) => {
      const invalid = key === "line" && lineInvalid;
      return <div className={cn("field", key === "line" && "field-line")} key={key}>
        <label htmlFor={`filter-${key}`}>{label}</label>
        <input id={`filter-${key}`} type="text" value={draft[key]} spellCheck={false} autoComplete="off"
          className={key === "code" || key === "field" ? "font-mono" : undefined}
          inputMode={key === "line" ? "numeric" : undefined} aria-invalid={invalid}
          aria-describedby={invalid ? "filter-line-error" : "filter-hint"}
          onChange={event => { const text = event.target.value; setDraft(now => ({ ...now, [key]: text })); if (key === "line") setLineInvalid(false); }} />
        {invalid && <p id="filter-line-error" className="field-error" role="alert">Numero di riga positivo, solo cifre.</p>}
      </div>;
    })}
    <div className="filter-footer">
      <p id="filter-hint" className="filter-hint">Corrispondenza esatta su tutti i filtri compilati: maiuscole e spazi contano. Invio per applicare.</p>
      <div className="filter-actions">
        <Button type="button" variant="ghost" aria-disabled={!clearable} onClick={clear}>Azzera</Button>
        <Button type="submit" variant="primary">Applica filtri</Button>
      </div>
    </div>
  </form>;
}

/** Turns the pending replies of a component off when it goes away. */
function useRequests() {
  const request = useRef(0);
  useEffect(() => () => { request.current += 1; }, []);
  return request;
}

/**
 * A text of the selected diagnostic: the Rust preview, and when that is not
 * complete the exact text read through results_text in windows of at most
 * 16 KiB, one window at a time (DETAIL v1: no hidden accumulation). Always a
 * React text node: no HTML is ever interpreted.
 */
export function FullText({ selectionRef, part, className }: { selectionRef: string; part: { component_ref: string; preview: Preview }; className?: string }) {
  const [chunk, setChunk] = useState<{ status: "idle" } | { status: "loading"; offset: string } | { status: "failed"; offset: string } | { status: "ready"; chunk: TextChunk }>({ status: "idle" });
  const request = useRequests();
  const read = (offset: string) => {
    const mine = ++request.current;
    setChunk({ status: "loading", offset });
    resultsText(selectionRef, part.component_ref, offset).then(
      value => { if (request.current === mine) setChunk({ status: "ready", chunk: value }); },
      () => { if (request.current === mine) setChunk({ status: "failed", offset }); });
  };
  if (part.preview.complete) return <span className={cn("detail-text", className)}>{part.preview.text}</span>;
  return <span className="detail-full">
    {chunk.status === "ready"
      ? <span className={cn("detail-text", className)}>{chunk.chunk.text}</span>
      : <span className={cn("detail-text", className)}>{part.preview.text}<span className="text-muted">…</span></span>}
    <span className="detail-more">
      {chunk.status === "idle" && <Button size="sm" variant="ghost" onClick={() => read("0")}>Leggi il testo completo</Button>}
      {chunk.status === "loading" && <span role="status">Lettura del testo…</span>}
      {chunk.status === "failed" && <><span role="alert">Lettura del testo non riuscita.</span>
        <Button size="sm" variant="ghost" onClick={() => read(chunk.offset)}>Riprova</Button></>}
      {chunk.status === "ready" && chunk.chunk.offset !== "0" && <Button size="sm" variant="ghost" onClick={() => read("0")}>Dall'inizio</Button>}
      {chunk.status === "ready" && chunk.chunk.has_more && <Button size="sm" variant="ghost" onClick={() => read(chunk.chunk.next_offset)}>Parte successiva</Button>}
      {chunk.status === "ready" && !chunk.chunk.has_more && <span className="text-muted">Fine del testo.</span>}
    </span>
  </span>;
}

/** The value read, or why Rust does not expose one. */
function Actual({ detail }: { detail: SelectedDiagnostic }) {
  const actual = detail.components.find(part => part.kind === "actual");
  if (detail.actual_visibility === "local_report_only_record_scope") return <span className="text-muted">Il valore copre l'intero record: si consulta solo nel report locale.</span>;
  if (!actual) return <span className="text-muted">Nessun valore letto.</span>;
  return <FullText selectionRef={detail.selection_ref} part={actual} className="detail-value" />;
}

/**
 * Context of the selected record, on request (DETAIL v1: separate pages of
 * at most 64 entries, one page kept). Values covering the whole record stay
 * in the local report.
 */
function ContextList({ selectionRef }: { selectionRef: string }) {
  const [state, setState] = useState<{ status: "idle" } | { status: "loading"; offset: string } | { status: "failed"; offset: string } | { status: "ready"; page: ContextPage }>({ status: "idle" });
  const request = useRequests();
  const read = (offset: string) => {
    const mine = ++request.current;
    setState({ status: "loading", offset });
    resultsContext(selectionRef, offset).then(
      page => { if (request.current === mine) setState({ status: "ready", page }); },
      () => { if (request.current === mine) setState({ status: "failed", offset }); });
  };
  return <section className="detail-context" aria-labelledby="detail-context-title">
    <h3 id="detail-context-title">Contesto del record</h3>
    {state.status === "idle" && <Button size="sm" variant="ghost" onClick={() => read("0")}>Mostra il contesto</Button>}
    {state.status === "loading" && <p role="status">Lettura del contesto…</p>}
    {state.status === "failed" && <div role="alert"><p>Non è stato possibile leggere il contesto.</p>
      <Button size="sm" variant="ghost" onClick={() => read(state.offset)}>Riprova</Button></div>}
    {state.status === "ready" && <>
      {state.page.entries.length === 0 ? <p className="text-muted">Nessun elemento di contesto.</p>
        : <dl className="detail-pairs">{state.page.entries.map((entry, index) => <div key={`${state.page.offset}-${index}`}>
            <dt><PreviewText preview={entry.trace} /> · <PreviewText preview={entry.field} /></dt>
            <dd>{entry.visibility === "local_report_only_record_scope"
              ? <span className="text-muted">Solo nel report locale.</span>
              : entry.value === null ? <span className="text-muted">—</span>
              : entry.component_ref === null ? <span className="detail-text detail-value">{previewText(entry.value)}</span>
              : <FullText selectionRef={selectionRef} part={{ component_ref: entry.component_ref, preview: entry.value }} className="detail-value" />}</dd>
          </div>)}</dl>}
      <div className="detail-more">
        {state.page.offset !== "0" && <Button size="sm" variant="ghost" onClick={() => read("0")}>Primi elementi</Button>}
        {state.page.has_more && <Button size="sm" variant="ghost" onClick={() => read(state.page.next_offset)}>Elementi successivi</Button>}
      </div>
    </>}
  </section>;
}

/** Names of the declaration kinds Rust sends, in the words of the Slint view. */
const SOURCE_KIND: Record<DomainSource["kind"], string> = {
  allowed_values: "Valori dichiarati", range: "Intervallo dichiarato", fallback: "Fallback autorizzati",
  reference: "Dataset di riferimento", regex: "Espressione regolare", non_enumerable: "Vincolo non enumerabile",
};
/**
 * Why a reference dataset has no list (T-MIG-P6-06), in the words of the
 * Slint view: missing or unverified evidence is never an empty valid domain.
 */
const REFERENCE_STATUS: Partial<Record<DomainPage["status"], string>> = {
  not_captured: "Dataset non presente nell'evidenza di questa analisi: dominio non ricostruibile.",
  degraded: "Integrità non verificata: i byte osservati non costituiscono un dominio ammesso. L'elenco non viene sostituito con un dominio vuoto.",
  unavailable: "Sorgente non disponibile durante l'analisi: dominio non ricostruibile.",
  unsupported: "Il provider del dataset non espone l'enumerazione: nessun elenco vuoto è dichiarato valido.",
  cancelled: "Lettura annullata: nessun elenco è dichiarato.",
};

/** The digests Rust took from the run's evidence, never from the renderer. */
function Evidence({ domain }: { domain: DomainPage }) {
  const listed = domain.status === "available" || domain.status === "available_empty";
  if (listed && domain.actual_sha256 !== null) return <p className="domain-evidence">Dataset verificato usato nell'analisi · SHA-256 <span className="font-mono">{domain.actual_sha256}</span></p>;
  if (domain.expected_sha256 === null && domain.actual_sha256 === null) return null;
  return <p className="domain-evidence text-muted">SHA-256 atteso <span className="font-mono">{domain.expected_sha256 ?? "—"}</span>
    {" · "}osservato <span className="font-mono">{domain.actual_sha256 ?? "non disponibile"}</span></p>;
}

/** One page of values of one source, as Rust read it, or why there is none. */
export function DomainValues({ selectionRef, domain, first = true }: { selectionRef: string; domain: DomainPage; first?: boolean }) {
  const reference = domain.kind === "reference";
  const reason = REFERENCE_STATUS[domain.status];
  return <>
    {reference && <Evidence domain={domain} />}
    {domain.status === "not_enumerable" ? <p className="text-muted">Nessun elenco finito: il vincolo non è enumerabile. Non viene inventato un elenco di valori.</p>
      : reason ? <p className="domain-warning" role="note">{reason}</p>
      : domain.status === "available_empty" ? <p className="text-muted">{reference && first ? "Il dataset verificato non contiene codici." : "Nessun valore in questa pagina del dominio."}</p>
      : domain.entries === null ? <p className="text-muted">Valori non disponibili per questa dichiarazione.</p>
      : <ul className="domain-values" aria-label="Valori del dominio">
        {domain.entries.map(entry => <li key={entry.component_ref}><FullText selectionRef={selectionRef} part={entry} className="detail-value" /></li>)}
      </ul>}
  </>;
}

/**
 * Reads one page of `source` (results_domain). A reference dataset is read by
 * a Rust worker: the answer is "loading" with a read handle, polled with
 * results_domain_state until Rust publishes the page or its status. Every
 * reply is used only while `live()` holds, as in openQuery.
 */
export function readDomain(selectionRef: string, source: string, cursor: string | null, live: () => boolean,
  done: (domain: DomainPage) => void, failed: () => void): void {
  const answer = (value: Awaited<ReturnType<typeof resultsDomain>>) => {
    if (!live()) return;
    if (value.state === "ready") { done(value.domain); return; }
    const readRef = value.read_ref;
    setTimeout(() => { if (live()) resultsDomainState(selectionRef, readRef).then(answer, fail); }, POLL_MS);
  };
  const fail = () => { if (live()) failed(); };
  resultsDomain(selectionRef, source, cursor).then(answer, fail);
}

/** Values of one source, as DomainSection holds them. */
export type DomainValuesState = { status: "loading"; source: string } | { status: "failed"; source: string; cursor: string | null }
  | { status: "ready"; source: string; domain: DomainPage; first: boolean };

/** One declaration of the field, and its values when they were read. */
export function DomainSourceItem({ selectionRef, source, number, shown, onRead: read }: { selectionRef: string; source: DomainSource; number: string;
  shown: DomainValuesState | null; onRead: (source: string, cursor: string | null) => void }) {
  // The button pressed goes away with the answer: the keyboard focus moves to the values read.
  const valuesRef = useRef<HTMLDivElement>(null);
  const page = shown?.status === "ready" ? shown.domain : null;
  useEffect(() => { if (page) valuesRef.current?.focus(); }, [page]);
  return <li className="domain-source">
    <p className="domain-kind">{SOURCE_KIND[source.kind]} <span className="text-muted">· dichiarazione {number} · {source.conditional
      ? <>regola condizionale {source.rule ? <PreviewText preview={source.rule} /> : "senza nome"}</> : "campo"}</span></p>
    {source.condition && <p className="domain-condition"><span className="text-muted">Si applica quando: </span>
      <FullText selectionRef={selectionRef} part={source.condition} className="detail-value" /></p>}
    {source.dataset && <p>Dataset <PreviewText preview={source.dataset.id} /> · versione <PreviewText preview={source.dataset.version} /></p>}
    {/* The DTO has no handle for the comparison text: its preview is all there is. */}
    <p className="domain-comparison detail-text"><PreviewText preview={source.comparison} /></p>
    <div className="detail-more">
      {!shown && <Button size="sm" variant="ghost" onClick={() => read(source.source_ref, null)}>Consulta i valori</Button>}
      {shown?.status === "loading" && <span role="status">{source.kind === "reference" ? "Lettura della pagina dal dataset usato nell'analisi…" : "Lettura dei valori…"}</span>}
      {shown?.status === "failed" && <><span role="alert">Lettura dei valori non riuscita.</span>
        <Button size="sm" variant="ghost" onClick={() => read(source.source_ref, shown.cursor)}>Riprova</Button></>}
    </div>
    {shown?.status === "ready" && <>
      <div ref={valuesRef} tabIndex={-1} className="domain-page" aria-label={`Valori di ${SOURCE_KIND[source.kind].toLowerCase()}, dichiarazione ${number}`}>
        <DomainValues selectionRef={selectionRef} domain={shown.domain} first={shown.first} />
      </div>
      <div className="detail-more">
        {!shown.first && <Button size="sm" variant="ghost" onClick={() => read(source.source_ref, null)}>Primi valori</Button>}
        {shown.domain.next_cursor !== null && <Button size="sm" variant="ghost" onClick={() => read(source.source_ref, shown.domain.next_cursor)}>Valori successivi</Button>}
      </div>
    </>}
  </li>;
}

/**
 * «Dominio ammesso» of the selected field (T-MIG-P6-05): the declarations
 * Rust lists for it in the captured schema (results_sources, pages of 64)
 * and, one source at a time, the values Rust pages from it (results_domain,
 * at most 64 per page, cursor kept by Rust). Literals and integers are Rust
 * text, never parsed or computed here; a conditional declaration keeps its
 * rule. A reference dataset is read by a Rust worker from the evidence
 * captured with the run (T-MIG-P6-06): only verified evidence lists values.
 */
function DomainSection({ selectionRef }: { selectionRef: string }) {
  type Sources = { status: "idle" } | { status: "loading"; offset: string } | { status: "failed"; offset: string } | { status: "ready"; page: SourcesPage };
  type Values = { status: "none" } | DomainValuesState;
  const [sources, setSources] = useState<Sources>({ status: "idle" });
  const [values, setValues] = useState<Values>({ status: "none" });
  const request = useRequests();
  const valueRequest = useRequests();
  // As for the values: the focus moves to the declarations once they are read.
  const heading = useRef<HTMLHeadingElement>(null);
  const listed = sources.status === "ready" ? sources.page : null;
  useEffect(() => { if (listed) heading.current?.focus(); }, [listed]);
  const list = (offset: string) => {
    const mine = ++request.current;
    valueRequest.current += 1;
    setValues({ status: "none" });
    setSources({ status: "loading", offset });
    resultsSources(selectionRef, offset).then(
      page => { if (request.current === mine) setSources({ status: "ready", page }); },
      () => { if (request.current === mine) setSources({ status: "failed", offset }); });
  };
  const read = (source: string, cursor: string | null) => {
    const mine = ++valueRequest.current;
    setValues({ status: "loading", source });
    readDomain(selectionRef, source, cursor, () => valueRequest.current === mine,
      domain => setValues({ status: "ready", source, domain, first: cursor === null }),
      () => setValues({ status: "failed", source, cursor }));
  };
  return <section className="detail-domain" aria-labelledby="detail-domain-title">
    <h3 id="detail-domain-title" ref={heading} tabIndex={-1}>Dominio ammesso</h3>
    {sources.status === "idle" && <Button size="sm" variant="ghost" onClick={() => list("0")}>Mostra il dominio</Button>}
    {sources.status === "loading" && <p role="status">Lettura delle dichiarazioni…</p>}
    {sources.status === "failed" && <div role="alert"><p>Non è stato possibile leggere le dichiarazioni del campo.</p>
      <Button size="sm" variant="ghost" onClick={() => list(sources.offset)}>Riprova</Button></div>}
    {sources.status === "ready" && <>
      {sources.page.sources.length === 0 ? <p className="text-muted">Nessun dominio dichiarato per il campo.</p>
        : <ol className="domain-sources">{sources.page.sources.map((source, index) => {
          const shown = values.status !== "none" && values.source === source.source_ref ? values : null;
          return <DomainSourceItem key={source.source_ref} selectionRef={selectionRef} source={source}
            number={grouped((BigInt(sources.page.offset) + BigInt(index) + 1n).toString())} shown={shown} onRead={read} />;
        })}</ol>}
      <div className="detail-more">
        {sources.page.offset !== "0" && <Button size="sm" variant="ghost" onClick={() => list("0")}>Prime dichiarazioni</Button>}
        {sources.page.has_more && <Button size="sm" variant="ghost" onClick={() => list(sources.page.next_offset)}>Dichiarazioni successive</Button>}
      </div>
    </>}
  </section>;
}

/**
 * «Dettaglio controllo» of the reference (T-MIG-P6-03): code, message and the
 * values of the selected diagnostic exactly as Rust captured them, with the
 * byte position and the field guide of the captured schema (T-MIG-P6-04:
 * written by Rust, never rebuilt here) and the declared domain of the field
 * (P6-05; reference datasets: P6-06); fiscal
 * code and episode are not shown (data minimisation).
 */
export function DetailPanel({ state, traces, onRetry }: { state: DetailState; traces: TraceNames; onRetry: () => void }) {
  const part = (detail: SelectedDiagnostic, kind: DetailComponent["kind"]) => detail.components.find(item => item.kind === kind);
  const value = (detail: SelectedDiagnostic, kind: DetailComponent["kind"]) => {
    const found = part(detail, kind);
    return found ? <FullText selectionRef={detail.selection_ref} part={found} /> : <span className="text-muted">—</span>;
  };
  const row = state.status === "none" ? "" : grouped((BigInt(state.selection.ordinal) + 1n).toString());
  return <Card className="detail-card" aria-labelledby="detail-title">
    <header className="detail-head"><h2 id="detail-title">Dettaglio controllo</h2></header>
    <div className="detail-body">
      {state.status === "none" && <p className="text-muted">Seleziona una riga per consultare causa e valori.</p>}
      {state.status === "loading" && <p role="status">Lettura del dettaglio della diagnostica {row}…</p>}
      {state.status === "failed" && <div role="alert">
        <p>Non è stato possibile leggere il dettaglio della diagnostica {row}.</p>
        <Button size="sm" onClick={onRetry}>Riprova</Button>
      </div>}
      {state.status === "ready" && <div key={state.detail.selection_ref} className="detail-content" aria-label={`Diagnostica ${row}`}>
        <p className="detail-kicker">Codice anomalia</p>
        <p className="detail-code">{value(state.detail, "code")}</p>
        <p className="detail-message">{value(state.detail, "message")}</p>
        <dl className="detail-pairs detail-grid">
          <div><dt>Riga</dt><dd>{grouped(state.detail.line)}</dd></div>
          <div><dt>Tracciato</dt><dd>{traces.has(state.detail.trace_ref)
            ? <PreviewText preview={traces.get(state.detail.trace_ref) as Preview} /> : <span className="text-muted">—</span>}</dd></div>
          <div><dt>Campo</dt><dd>{value(state.detail, "field_name")}</dd></div>
          <div><dt>Posizione in byte</dt><dd>{state.detail.field_position
            ? `${grouped(state.detail.field_position.start)}–${grouped(state.detail.field_position.end)}`
            : <span className="text-muted">—</span>}</dd></div>
          <div><dt>ID record</dt><dd>{value(state.detail, "sioc_record_id")}</dd></div>
        </dl>
        <dl className="detail-pairs">
          <div><dt>Valore letto <span className="text-muted">· spazi conservati</span></dt><dd><Actual detail={state.detail} /></dd></div>
          {part(state.detail, "expected") && <div><dt>Valore atteso</dt><dd>{value(state.detail, "expected")}</dd></div>}
        </dl>
        {part(state.detail, "guide") && <section className="detail-guide" aria-labelledby="detail-guide-title">
          <h3 id="detail-guide-title">Guida al campo</h3>
          {value(state.detail, "guide")}
        </section>}
        {state.detail.field_position && <DomainSection selectionRef={state.detail.selection_ref} />}
        <ContextList selectionRef={state.detail.selection_ref} />
      </div>}
    </div>
  </Card>;
}

/** Summary cards of the reference: records, diagnostics, records with
 * violations and outcome, all as Rust reports them. */
function Summary({ about, overview }: { about: AboutReport; overview: OverviewState }) {
  const ready = about.status === "ready" ? about.about : null;
  const counts = overview.status === "ready" ? overview.overview : null;
  const tone = !ready ? "neutral" : ready.status === "completed_invalid" ? "error"
    : ready.completeness === "partial" || ready.truncated ? "warn" : "success";
  const number = (value: string | undefined) => (value === undefined ? "—" : grouped(value));
  return <div className="summary" aria-label="Riepilogo del report">
    <Card className="kpi"><h2>Record analizzati</h2>
      <p className="kpi-value">{number(counts?.records_processed)}</p>
      <p className="kpi-note">{ready ? (ready.mode === "local_only" ? "Controlli locali sul tracciato" : "Flusso completo (Full)") : "In lettura…"}</p></Card>
    <Card className="kpi"><h2>Diagnostiche nel report</h2>
      <p className={cn("kpi-value", counts && counts.total_errors !== "0" && "tone-error")}>{number(counts?.total_errors)}</p>
      <p className="kpi-note">{counts ? `${grouped(String(counts.by_code.length))} ${counts.by_code.length === 1 ? "codice" : "codici"} di controllo` : "—"}</p></Card>
    <Card className="kpi"><h2>Record con violazioni</h2>
      <p className={cn("kpi-value", counts && counts.invalid_records !== "0" && "tone-error")}>{number(counts?.invalid_records)}</p>
      <p className="kpi-note">{counts ? `Senza violazioni: ${grouped(counts.valid_records)}` : "—"}</p></Card>
    <Card className="kpi"><h2>Esito</h2>
      <p className={cn("kpi-value kpi-text", `tone-${tone}`)}>{ready ? reportStatusText(ready) : "—"}</p>
      <p className="kpi-note">{ready ? `${previewText(ready.flow_id)} · ${previewText(ready.schema_version)}` : "—"}</p></Card>
  </div>;
}

/**
 * Risultati body. Without a report it says so; otherwise the summary, then
 * the diagnostics card with its own states (building, failed, empty, page).
 */
export function Results({ view, about, overview, reportAvailable, filters, onFilter, onPage, onRetry,
  detail = NO_DETAIL, onSelect, onDetailRetry = noop }:
  { view: ResultsView; about: AboutReport; overview: OverviewState; reportAvailable: boolean; filters: ResultsFilters;
    onFilter: (filters: ResultsFilters) => void; onPage: (page: bigint) => void; onRetry: () => void;
    detail?: DetailState; onSelect?: (selection: Selection) => void; onDetailRetry?: () => void }) {
  const filtered = filtering(filters);
  const traces = useMemo<TraceNames>(() => overview.status === "ready"
    ? new Map(overview.overview.by_trace.map(entry => [entry.trace_ref, entry.trace])) : NO_TRACES, [overview]);
  // Only a read `report_about` can clear the qualifier; "none" means it was not requested yet.
  const scope: ChartScope = about.status === "failed" ? "unavailable"
    : about.status !== "ready" ? "pending"
    : about.about.completeness === "partial" || about.about.truncated || about.about.status === "cancelled" ? "incomplete" : "complete";
  if (!reportAvailable) return <Card className="empty-card">
    <p className="lead">Nessun report disponibile.</p>
    <p>I risultati di un'analisi conclusa compaiono in questa sezione.</p>
  </Card>;
  return <>
    <Summary about={about} overview={overview} />
    {overview.status === "ready" && <Charts overview={overview.overview} scope={scope} />}
    {overview.status === "failed" && <p className="grid-message" role="alert">Non è stato possibile leggere i conteggi del report per i grafici.</p>}
    <div className={cn("results-split", view.status === "ready" && view.total_matches !== "0" && "with-detail")}>
    <Card className="grid-card" aria-labelledby="diagnostics-title">
      <header className="grid-head">
        <h2 id="diagnostics-title">Diagnostiche <span>/ ordine del report</span></h2>
        {view.status === "ready" && filtered && <p className="grid-range">
          {grouped(view.total_matches)} {view.total_matches === "1" ? "corrispondenza" : "corrispondenze"} su {grouped(view.total_diagnostics)} diagnostiche
        </p>}
        {view.status === "ready" && !filtered && view.current && <p className="grid-range">{rangeText(view.current)}</p>}
      </header>
      {view.status !== "none" && <FilterBar applied={filters} onApply={onFilter} />}
      {view.status === "building" && <p className="grid-message" role="status">{filtered ? "Applicazione dei filtri…" : "Preparazione dei risultati…"}</p>}
      {view.status === "failed" && <div className="grid-message" role="alert">
        <p>{filtered ? "Non è stato possibile applicare i filtri: nessuna riga è mostrata." : "Non è stato possibile preparare i risultati del report."}</p>
        <Button onClick={onRetry}>Riprova</Button>
      </div>}
      {view.status === "ready" && view.total_matches === "0" && <p className="grid-message">
        {filtered ? "Nessuna diagnostica corrisponde ai filtri." : "Il report non contiene diagnostiche."}</p>}
      {view.status === "ready" && view.total_matches !== "0" && <>
        {view.pageFailed && <div className="grid-message" role="alert">
          <p>Non è stato possibile leggere questa pagina dei risultati.</p>
          <Button onClick={() => onPage(view.page)}>Riprova</Button>
        </div>}
        <ResultsGrid page={view.current} traces={traces} selected={detail.status === "none" ? null : detail.selection.ordinal}
          onSelect={onSelect ? row => onSelect({ query_ref: view.query_ref, ordinal: row.ordinal, diagnostic_ref: row.diagnostic_ref }) : undefined} />
        <footer className="grid-foot">
          <p role="status">{view.loading ? "Lettura della pagina…" : view.current ? rangeText(view.current) : ""}</p>
          <Pager view={view} onPage={onPage} />
        </footer>
      </>}
    </Card>
    {view.status === "ready" && view.total_matches !== "0" && <DetailPanel state={detail} traces={traces} onRetry={onDetailRetry} />}
    </div>
  </>;
}
function noop() { /* no detail callbacks in a static rendering */ }
