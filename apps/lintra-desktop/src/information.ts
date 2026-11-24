// Informazioni model (T-MIG-P5-07): provenance of the selection and of the
// published report, and the third-party notices shipped with the bundle.
// Every value comes from Rust (app_info, configuration_select, report_about)
// or from public/THIRD_PARTY_NOTICES; nothing is inferred here.
import type { Preview, ReportAbout } from "./ipc";

/** One license text of public/THIRD_PARTY_NOTICES, under its
 * "<packages> / <license file>" header. */
export type Notice = { title: string; body: string };

// The header form written by the notices generator (assets-provenance.json).
// The file name is the package's own: clsx ships a lowercase "license".
const HEADER = /^(.+ \/ (?:LICENSE|license)[A-Za-z0-9.-]*)\n/gm;

/** Splits the notices file into its blocks, keeping each text verbatim. */
export function splitNotices(text: string): Notice[] {
  const heads = [...text.matchAll(HEADER)];
  return heads.map((head, index) => {
    const start = head.index + head[0].length;
    // Blocks are joined by "\n\n" and the file ends with "\n"; a license
    // text may itself end with or without a newline, so cut exactly that.
    const next = heads[index + 1]?.index;
    return { title: head[1] ?? "", body: text.slice(start, next === undefined ? text.length - 1 : next - 2) };
  });
}

/** A bounded Rust preview; a cut value is marked, never presented whole. */
export function previewText(preview: Preview): string {
  return preview.complete ? preview.text : `${preview.text}…`;
}

/** Outcome of the published report in the words of the run screen. Rust
 * declares status, completeness and truncation separately; none implies
 * conformity of data that was not checked. */
export function reportStatusText(about: Pick<ReportAbout, "status" | "completeness" | "truncated">): string {
  if (about.status === "cancelled") return "Analisi interrotta: i controlli non sono stati completati";
  if (about.truncated) return "Analisi parziale: raggiunto il limite operativo";
  const partial = about.completeness === "partial" ? ", controlli incompleti" : "";
  return about.status === "completed_invalid" ? `Violazioni rilevate${partial}` : `Nessuna violazione nei controlli eseguiti${partial}`;
}

export type AboutReport =
  | { status: "none" }
  | { status: "loading" }
  | { status: "ready"; about: ReportAbout }
  | { status: "failed" };
