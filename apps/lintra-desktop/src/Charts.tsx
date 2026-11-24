// Results charts (T-MIG-P6-01, references lintra_ui_references: Dioxus
// "Errori per tracciato" / "Tipologia errori", Tauri+Leptos "Distribuzione
// errori" / "Tasso di validità"). Plain SVG, no chart library. Every number
// comes from results_overview (verified Rust summary and index); the renderer
// only scales them. Values are written next to the marks and each chart has a
// text alternative, so colour never carries the information alone.
import { AlertTriangle } from "lucide-react";
import type { ResultsOverview } from "./ipc";
import { previewText } from "./information";
import { Card, cn } from "./ui";

/** Codes drawn one by one; the rest is one honest "Altri codici" entry. */
export const MAX_SERIES = 7;
const grouped = (value: bigint) => value.toString().replace(/\B(?=(\d{3})+(?!\d))/g, ".");

/** Share in tenths of a percent, exact on u64 (floor, never rounded up to 100). */
export function permille(part: bigint, whole: bigint): bigint | null {
  return whole === 0n ? null : (part * 1000n) / whole;
}
/** "98,2%" from tenths of a percent. */
export function percentText(tenths: bigint): string {
  return `${tenths / 10n},${tenths % 10n}%`;
}

/**
 * Codes ordered by count (ties by code, as Rust sorts them), the first
 * MAX_SERIES kept and the remainder summed into one entry.
 */
export function codeSeries(overview: ResultsOverview): { label: string; count: bigint; other: boolean }[] {
  const items = overview.by_code.map(entry => ({ label: previewText(entry.code), count: BigInt(entry.count), other: false }))
    .filter(item => item.count > 0n)
    .sort((a, b) => (a.count === b.count ? 0 : a.count > b.count ? -1 : 1));
  if (items.length <= MAX_SERIES) return items;
  const rest = items.slice(MAX_SERIES - 1).reduce((sum, item) => sum + item.count, 0n);
  return [...items.slice(0, MAX_SERIES - 1), { label: `Altri codici (${items.length - MAX_SERIES + 1})`, count: rest, other: true }];
}

/** Ratio for drawing only (geometry), never shown as a number. */
const ratio = (part: bigint, whole: bigint) => (whole === 0n ? 0 : Number((part * 10_000n) / whole) / 10_000);

/** Vertical bars: diagnostics per trace, every scope trace (zero included). */
function TraceBars({ overview }: { overview: ResultsOverview }) {
  const traces = overview.by_trace.map(entry => ({ label: previewText(entry.trace), count: BigInt(entry.count) }));
  const max = traces.reduce((top, item) => (item.count > top ? item.count : top), 0n);
  const width = 320, height = 170, base = 140, slot = width / Math.max(traces.length, 1), bar = Math.min(40, slot * 0.5);
  const summary = traces.map(item => `${item.label}: ${grouped(item.count)}`).join("; ");
  return <Card className="chart-card">
    <h2>Errori per tracciato</h2>
    <svg className="chart" viewBox={`0 0 ${width} ${height}`} role="img" aria-label={`Diagnostiche per tracciato. ${summary}`}>
      <line className="chart-axis" x1={0} x2={width} y1={base} y2={base} />
      {traces.map((item, index) => {
        const h = Math.round(ratio(item.count, max) * 110);
        const x = index * slot + (slot - bar) / 2;
        return <g key={item.label + index}>
          <rect className="chart-error" x={x} y={base - h} width={bar} height={Math.max(h, item.count > 0n ? 2 : 0)} rx={3} />
          <text className="chart-value" x={x + bar / 2} y={base - h - 6} textAnchor="middle">{grouped(item.count)}</text>
          <text className="chart-label" x={x + bar / 2} y={base + 18} textAnchor="middle">{item.label}</text>
        </g>;
      })}
    </svg>
  </Card>;
}

/** Donut: diagnostics by code, with the total in the centre and a legend. */
function CodeDonut({ overview }: { overview: ResultsOverview }) {
  const series = codeSeries(overview);
  const total = BigInt(overview.total_errors);
  const radius = 52, stroke = 18, circumference = 2 * Math.PI * radius;
  let start = 0;
  return <Card className="chart-card">
    <h2>Tipologia errori</h2>
    <div className="donut">
      <svg className="chart donut-chart" viewBox="0 0 140 140" role="img"
        aria-label={`Diagnostiche per codice: ${grouped(total)} in totale.`}>
        <circle className="chart-track" cx={70} cy={70} r={radius} strokeWidth={stroke} fill="none" />
        {series.map((item, index) => {
          const length = ratio(item.count, total) * circumference;
          const arc = <circle key={item.label} className={`chart-s${index + 1}`} cx={70} cy={70} r={radius} fill="none" strokeWidth={stroke}
            strokeDasharray={`${length} ${circumference - length}`} strokeDashoffset={-start} transform="rotate(-90 70 70)" />;
          start += length;
          return arc;
        })}
        <text className="donut-total" x={70} y={75} textAnchor="middle">{grouped(total)}</text>
      </svg>
      <ul className="legend">
        {series.map((item, index) => <li key={item.label}>
          <span className={cn("legend-swatch", `swatch-s${index + 1}`)} aria-hidden="true" />
          <span className={cn("legend-label", !item.other && "font-mono")}>{item.label}</span>
          <span className="legend-value">{grouped(item.count)}</span>
        </li>)}
      </ul>
    </div>
  </Card>;
}

/** Horizontal bars: distribution by code, value at the end of each bar. */
function CodeBars({ overview }: { overview: ResultsOverview }) {
  const series = codeSeries(overview);
  const max = series.reduce((top, item) => (item.count > top ? item.count : top), 0n);
  return <Card className="chart-card">
    <h2>Distribuzione errori</h2>
    <ul className="hbars" aria-label="Diagnostiche per codice">
      {series.map(item => <li key={item.label}>
        <span className={cn("hbar-label", !item.other && "font-mono")}>{item.label}</span>
        <svg className="hbar" viewBox="0 0 100 10" preserveAspectRatio="none" aria-hidden="true">
          <rect className="chart-error" x={0} y={0} width={Math.max(ratio(item.count, max) * 100, 1)} height={10} rx={1} />
        </svg>
        <span className="hbar-value">{grouped(item.count)}</span>
      </li>)}
    </ul>
  </Card>;
}

/** Ring: records without violations among the records processed. */
function ValidityRing({ overview }: { overview: ResultsOverview }) {
  const processed = BigInt(overview.records_processed), valid = BigInt(overview.valid_records);
  const tenths = permille(valid, processed);
  const radius = 52, stroke = 12, circumference = 2 * Math.PI * radius;
  const length = ratio(valid, processed) * circumference;
  const text = tenths === null ? "—" : percentText(tenths);
  return <Card className="chart-card">
    <h2>Record senza violazioni</h2>
    <div className="validity">
      <svg className="chart validity-chart" viewBox="0 0 140 140" role="img"
        aria-label={`${text} dei record analizzati senza violazioni nei controlli eseguiti.`}>
        <circle className="chart-track" cx={70} cy={70} r={radius} strokeWidth={stroke} fill="none" />
        {valid > 0n && <circle className="chart-ok" cx={70} cy={70} r={radius} strokeWidth={stroke} fill="none"
          strokeDasharray={`${length} ${circumference - length}`} transform="rotate(-90 70 70)" strokeLinecap="round" />}
        <text className="validity-value" x={70} y={77} textAnchor="middle">{text}</text>
      </svg>
      <p className="validity-note"><strong>{grouped(valid)}</strong> senza violazioni su {grouped(processed)} analizzati</p>
      <p className="validity-note tone-error">{grouped(BigInt(overview.invalid_records))} con violazioni</p>
    </div>
  </Card>;
}

/**
 * The four charts. When the controls were not all performed (partial,
 * truncated or cancelled report) they stay visible, under a warning that
 * says so (author decision, 24 Nov 2026).
 */
export function Charts({ overview, incomplete }: { overview: ResultsOverview; incomplete: boolean }) {
  return <section className="charts" aria-label="Grafici del report">
    {incomplete && <p className="notice-warn" role="note">
      <AlertTriangle aria-hidden="true" />
      <span><strong>Risultati incompleti.</strong> Non tutti i controlli sono stati eseguiti: grafici e conteggi
        riguardano solo i controlli completati e non descrivono l’intero file.</span>
    </p>}
    <div className="charts-grid">
      <TraceBars overview={overview} />
      <CodeDonut overview={overview} />
      <CodeBars overview={overview} />
      <ValidityRing overview={overview} />
    </div>
  </section>;
}
