// Risultati section (T-MIG-P6-01): summary cards, charts and the diagnostics
// grid. Rust builds the query, orders it and serves bounded pages (PAGING.md);
// the grid shows one logical page of at most 128 rows and keeps in the DOM
// only the rows in view (TanStack Virtual), so DOM and memory stay bounded for
// any report size. Columns hold only what a row DTO carries: line, trace
// (named through results_overview), field, code and the reason as previews
// (author decision of 24 Nov 2026); raw bytes, context and identity stay Rust.
import { useCallback, useEffect, useMemo, useRef, useState, type FormEvent } from "react";
import { createColumnHelper, tableFeatures, useTable } from "@tanstack/react-table";
import { useVirtualizer } from "@tanstack/react-virtual";
import { ChevronLeft, ChevronRight } from "lucide-react";
import { resultsOverview, resultsPage, resultsQuery, resultsQueryState, type Preview, type ResultRow, type ResultsOverview, type ResultsPage, type RunRef } from "./ipc";
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
export function ResultsGrid({ page, traces = NO_TRACES }: { page: ResultsPage | null; traces?: TraceNames }) {
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
  return <div ref={viewport} className="grid-viewport" tabIndex={0} role="region"
    aria-label={page ? `Diagnostiche, ${rangeText(page).toLowerCase()}` : "Diagnostiche"}>
    <table className="grid-table" aria-rowcount={Number(total) <= Number.MAX_SAFE_INTEGER ? Number(total) : -1}>
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
          return <tr key={row.id} aria-rowindex={Number(BigInt(row.original.ordinal) + 2n)}>
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
export function Results({ view, about, overview, reportAvailable, filters, onFilter, onPage, onRetry }:
  { view: ResultsView; about: AboutReport; overview: OverviewState; reportAvailable: boolean; filters: ResultsFilters;
    onFilter: (filters: ResultsFilters) => void; onPage: (page: bigint) => void; onRetry: () => void }) {
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
        <ResultsGrid page={view.current} traces={traces} />
        <footer className="grid-foot">
          <p role="status">{view.loading ? "Lettura della pagina…" : view.current ? rangeText(view.current) : ""}</p>
          <Pager view={view} onPage={onPage} />
        </footer>
      </>}
    </Card>
  </>;
}
