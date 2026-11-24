// Risultati section (T-MIG-P6-01): summary cards, charts and the diagnostics
// grid. Rust builds the query, orders it and serves bounded pages (PAGING.md);
// the grid shows one logical page of at most 128 rows and keeps in the DOM
// only the rows in view (TanStack Virtual), so DOM and memory stay bounded for
// any report size. Columns hold only what a row DTO carries: line, trace
// (named through results_overview), field, code and the reason as previews
// (author decision of 24 Nov 2026); raw bytes, context and identity stay Rust.
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { createColumnHelper, tableFeatures, useTable } from "@tanstack/react-table";
import { useVirtualizer } from "@tanstack/react-virtual";
import { ChevronLeft, ChevronRight } from "lucide-react";
import { resultsOverview, resultsPage, resultsQuery, resultsQueryState, type Preview, type ResultRow, type ResultsOverview, type ResultsPage, type RunRef } from "./ipc";
import { Charts } from "./Charts";
import { previewText, reportStatusText, type AboutReport } from "./information";
import { PAGE_ROWS, acceptPage, cached, pageCount, pageItems, pageOffset, rangeText, remember, type ResultsView } from "./pages";
import { Badge, Button, Card, cn } from "./ui";

/** Interval between query state reads while Rust builds the index. */
const POLL_MS = 200;
/** Fixed row height (px): two text lines at most (message, expected/found),
 * each cut with an ellipsis, so no measuring is needed. */
export const ROW_HEIGHT = 56;
/** Rows rendered beyond the viewport on each side. */
export const OVERSCAN = 6;

/**
 * Query of the report Rust holds as last published, built while the section
 * is visible. Late replies for another report, query or page are dropped, so
 * a previous query never appears under a new report.
 */
export function useResults(report: RunRef | null, visible: boolean):
  [ResultsView, (page: bigint) => void, () => void] {
  const [view, setView] = useState<ResultsView>({ status: "none" });
  const latest = useRef(view);
  latest.current = view;
  const cache = useRef<ResultsPage[]>([]);
  const [attempt, setAttempt] = useState(0);
  useEffect(() => {
    cache.current = [];
    if (report === null || !visible) { setView({ status: "none" }); return; }
    let active = true;
    let timer: ReturnType<typeof setTimeout> | undefined;
    setView({ status: "building", run: report });
    const ready = async (queryRef: string, matches: string, total: string) => {
      setView({ status: "ready", run: report, query_ref: queryRef, total_matches: matches, total_diagnostics: total,
        page: 0n, loading: true, pageFailed: false, current: null });
      const page = await resultsPage(queryRef, "0", PAGE_ROWS);
      if (!active) return;
      cache.current = remember(cache.current, page);
      setView(current => acceptPage(current, page));
    };
    const poll = (queryRef: string) => {
      timer = setTimeout(() => {
        resultsQueryState(queryRef).then(status => {
          if (!active) return;
          if (status.state === "building") poll(queryRef);
          else if (status.state === "ready" && status.total_matches !== null && status.total_diagnostics !== null) {
            return ready(queryRef, status.total_matches, status.total_diagnostics);
          } else setView({ status: "failed", run: report });
        }).catch(() => { if (active) setView({ status: "failed", run: report }); });
      }, POLL_MS);
    };
    resultsQuery(report, { schemaVersion: 1 }).then(status => {
      if (!active) return;
      if (status.state === "building") poll(status.query_ref);
      else if (status.total_matches !== null && status.total_diagnostics !== null) {
        return ready(status.query_ref, status.total_matches, status.total_diagnostics);
      }
    }).catch(() => { if (active) setView({ status: "failed", run: report }); });
    return () => { active = false; if (timer !== undefined) clearTimeout(timer); };
  }, [report, visible, attempt]);
  const goTo = useCallback((page: bigint) => {
    const current = latest.current;
    if (current.status !== "ready" || page < 0n || page >= pageCount(current.total_matches)) return;
    const offset = pageOffset(page);
    const hit = cached(cache.current, current.query_ref, offset);
    if (hit) { setView(acceptPage({ ...current, page }, hit)); return; }
    setView({ ...current, page, loading: true, pageFailed: false });
    resultsPage(current.query_ref, offset, PAGE_ROWS).then(loaded => {
      cache.current = remember(cache.current, loaded);
      setView(now => acceptPage(now, loaded));
    }, () => setView(now => now.status === "ready" && now.query_ref === current.query_ref && now.page === page
      ? { ...now, loading: false, pageFailed: true } : now));
  }, []);
  const retry = useCallback(() => setAttempt(value => value + 1), []);
  return [view, goTo, retry];
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
export function Results({ view, about, overview, reportAvailable, onPage, onRetry }:
  { view: ResultsView; about: AboutReport; overview: OverviewState; reportAvailable: boolean; onPage: (page: bigint) => void; onRetry: () => void }) {
  const traces = useMemo<TraceNames>(() => overview.status === "ready"
    ? new Map(overview.overview.by_trace.map(entry => [entry.trace_ref, entry.trace])) : NO_TRACES, [overview]);
  const incomplete = about.status === "ready"
    && (about.about.completeness === "partial" || about.about.truncated || about.about.status === "cancelled");
  if (!reportAvailable) return <Card className="empty-card">
    <p className="lead">Nessun report disponibile.</p>
    <p>I risultati di un'analisi conclusa compaiono in questa sezione.</p>
  </Card>;
  return <>
    <Summary about={about} overview={overview} />
    {overview.status === "ready" && <Charts overview={overview.overview} incomplete={incomplete} />}
    {overview.status === "failed" && <p className="grid-message" role="alert">Non è stato possibile leggere i conteggi del report per i grafici.</p>}
    <Card className="grid-card" aria-labelledby="diagnostics-title">
      <header className="grid-head">
        <h2 id="diagnostics-title">Diagnostiche <span>/ ordine del report</span></h2>
        {view.status === "ready" && view.current && <p className="grid-range">{rangeText(view.current)}</p>}
      </header>
      {view.status === "building" && <p className="grid-message" role="status">Preparazione dei risultati…</p>}
      {view.status === "failed" && <div className="grid-message" role="alert">
        <p>Non è stato possibile preparare i risultati del report.</p>
        <Button onClick={onRetry}>Riprova</Button>
      </div>}
      {view.status === "ready" && view.total_matches === "0" && <p className="grid-message">Il report non contiene diagnostiche.</p>}
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
