// Results grid model (T-MIG-P6-01). Rust owns the query, its order and its
// pages (PAGING.md); this module only does the arithmetic of logical pages on
// the decimal u64 counters Rust returns, and bounds what the renderer keeps:
// at most CACHE_PAGES pages of at most PAGE_ROWS rows, never the dataset.
import type { ResultsPage, RunRef } from "./ipc";

/** Rows per logical page: the contract maximum (MAX_PAGE_ROWS of ipc.ts,
 * checked by tests/grid.test.mjs), one IPC page each. */
export const PAGE_ROWS = 128;
/** Pages kept in the renderer: the shown one and the last one left. */
export const CACHE_PAGES = 2;

/** Number of pages for a filtered total (0 when there are no matches). */
export function pageCount(totalMatches: string): bigint {
  const total = BigInt(totalMatches);
  return (total + BigInt(PAGE_ROWS) - 1n) / BigInt(PAGE_ROWS);
}

/** Canonical decimal offset of a 0-based page. */
export function pageOffset(page: bigint): string {
  if (page < 0n) throw new RangeError("negative page");
  return (page * BigInt(PAGE_ROWS)).toString();
}

/** 0-based page holding a 0-based ordinal. */
export function pageOf(ordinal: string): bigint {
  return BigInt(ordinal) / BigInt(PAGE_ROWS);
}

/**
 * Page buttons as in the reference footer: first, last, the current page and
 * its neighbours, with "gap" where pages are skipped. At most 7 items, so the
 * footer stays bounded for millions of pages.
 */
export function pageItems(current: bigint, count: bigint): (bigint | "gap")[] {
  if (count <= 0n) return [];
  if (count <= 7n) return Array.from({ length: Number(count) }, (_, index) => BigInt(index));
  const last = count - 1n;
  const start = current <= 3n ? 1n : current >= last - 3n ? last - 4n : current - 1n;
  const end = current <= 3n ? 4n : current >= last - 3n ? last - 1n : current + 1n;
  const items: (bigint | "gap")[] = [0n];
  if (start > 1n) items.push("gap");
  for (let page = start; page <= end; page++) items.push(page);
  if (end < last - 1n) items.push("gap");
  items.push(last);
  return items;
}

/** "Righe 129–256 di 1.000" with Italian grouping, computed in BigInt. */
export function rangeText(page: ResultsPage): string {
  const group = (value: bigint) => value.toString().replace(/\B(?=(\d{3})+(?!\d))/g, ".");
  if (page.rows.length === 0) return `Nessuna riga su ${group(BigInt(page.total_matches))}`;
  const first = BigInt(page.offset) + 1n;
  return `Righe ${group(first)}–${group(BigInt(page.next_offset))} di ${group(BigInt(page.total_matches))}`;
}

/**
 * Keeps the newest page first and at most CACHE_PAGES pages, all of the same
 * query: a page of another query_ref (a superseded query) is dropped.
 */
export function remember(cache: readonly ResultsPage[], page: ResultsPage): ResultsPage[] {
  const kept = cache.filter(entry => entry.query_ref === page.query_ref && entry.offset !== page.offset);
  return [page, ...kept].slice(0, CACHE_PAGES);
}

/** Cached page of this query at this offset, if still held. */
export function cached(cache: readonly ResultsPage[], queryRef: string, offset: string): ResultsPage | undefined {
  return cache.find(entry => entry.query_ref === queryRef && entry.offset === offset);
}

/** Same run identity (a page of another run is stale, whatever it says). */
export function sameRunRef(a: RunRef, b: RunRef): boolean {
  return a.instance_id === b.instance_id && a.run_id === b.run_id;
}

/**
 * State of the results view. `ready.page` is the requested page; `current`
 * is the page shown, which may still be the previous one while a new page is
 * loading, never a page of another query or run.
 */
export type ResultsView =
  | { status: "none" }
  | { status: "building"; run: RunRef }
  | { status: "failed"; run: RunRef }
  | { status: "ready"; run: RunRef; query_ref: string; total_matches: string; total_diagnostics: string;
      page: bigint; loading: boolean; pageFailed: boolean; current: ResultsPage | null };

/**
 * Accepts a loaded page only for the query and page still requested; a late
 * answer for a page or query left behind changes nothing.
 */
export function acceptPage(view: ResultsView, page: ResultsPage): ResultsView {
  if (view.status !== "ready" || page.query_ref !== view.query_ref || !sameRunRef(page.run, view.run)
      || page.offset !== pageOffset(view.page)) return view;
  return { ...view, loading: false, pageFailed: false, current: page,
    total_matches: page.total_matches, total_diagnostics: page.total_diagnostics };
}
