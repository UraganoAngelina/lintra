import { useEffect, useState } from "react";
import { reportAbout, type BundleProvenance, type RunRef } from "./ipc";
import { provenanceText, type SelectionState } from "./analysis";
import { previewText, reportStatusText, type AboutReport, type Notice } from "./information";

/** Reads the provenance of the report Rust holds as last published, each
 * time the section is shown and whenever that report changes. A late reply
 * for a report no longer shown is dropped. */
export function useReportAbout(report: RunRef | null, visible: boolean): AboutReport {
  const [state, setState] = useState<AboutReport>({ status: "none" });
  useEffect(() => {
    if (report === null || !visible) { setState({ status: "none" }); return; }
    let active = true;
    setState({ status: "loading" });
    reportAbout(report).then(about => { if (active) setState({ status: "ready", about }); },
      () => { if (active) setState({ status: "failed" }); });
    return () => { active = false; };
  }, [report, visible]);
  return state;
}

/**
 * Informazioni section, the Slint About view (ui/ui/main.slint) without its
 * Slint-only widget: version, provenance of the current selection, provenance
 * of the published report, and the license texts of the shipped components.
 * The Slint attribution belongs to the legacy executable only; this one
 * contains no Slint code.
 */
export function About({ version, selection, report, notices }:
  { version: string | null; selection: SelectionState; report: AboutReport; notices: Notice[] }) {
  return <>
    <section aria-labelledby="about-version">
      <h2 id="about-version">Lintra</h2>
      <p className="lead">Validazione di flussi regionali</p>
      <dl>
        <dt>Versione</dt><dd>{version ?? "Non ancora disponibile"}</dd>
      </dl>
    </section>
    <SelectionProvenance selection={selection} />
    <ReportProvenance report={report} />
    <section className="about-block" aria-labelledby="about-licenses">
      <h2 id="about-licenses">Licenze di terze parti</h2>
      <p>Componenti distribuiti con questa applicazione e testo integrale delle loro licenze.</p>
      <ul className="notices">
        {notices.map(notice => <li key={notice.title}>
          <details>
            <summary>{notice.title}</summary>
            {/* Focusable so the long text can be scrolled from the keyboard. */}
            <pre tabIndex={0} aria-label={`Testo della licenza: ${notice.title}`}>{notice.body}</pre>
          </details>
        </li>)}
      </ul>
    </section>
  </>;
}

function Bundle({ bundle }: { bundle: BundleProvenance }) {
  return <>
    <dt>Origine</dt><dd>{provenanceText(bundle)}</dd>
    <dt>Manifest del bundle</dt><dd>{bundle.bundle_manifest_status === "present" ? "presente" : "assente"}</dd>
  </>;
}

function SelectionProvenance({ selection }: { selection: SelectionState }) {
  const configuration = selection.configuration.status === "ready" ? selection.configuration.selection : null;
  const catalog = selection.catalog.status === "ready" ? selection.catalog.provenance : null;
  return <section className="about-block" aria-labelledby="about-selection">
    <h2 id="about-selection">Analisi selezionata</h2>
    {configuration ? <dl>
      <dt>Schema</dt><dd>{configuration.flow_id} · {configuration.schema_version}</dd>
      <dt>Impronta dello schema</dt><dd>{configuration.schema_hash}</dd>
      <Bundle bundle={configuration.provenance} />
    </dl> : <>
      <p>Nessuno schema selezionato.</p>
      {catalog && <dl><Bundle bundle={catalog} /></dl>}
    </>}
  </section>;
}

function ReportProvenance({ report }: { report: AboutReport }) {
  return <section className="about-block" aria-labelledby="about-report">
    <h2 id="about-report">Ultimo report</h2>
    {report.status === "none" && <p>Nessun report pubblicato in questa sessione.</p>}
    {report.status === "loading" && <p role="status">Lettura della provenienza del report…</p>}
    {report.status === "failed" && <p role="alert">Non è stato possibile leggere la provenienza del report. Apri di nuovo Informazioni per riprovare.</p>}
    {report.status === "ready" && <>
      <p>Questi dati riguardano l’analisi che ha prodotto il report, non la selezione corrente.</p>
      <dl>
        <dt>Esito</dt><dd>{reportStatusText(report.about)}</dd>
        <dt>Modalità</dt><dd>{report.about.mode === "local_only" ? "Controlli locali sul tracciato" : "Completa, tutti i tracciati"}</dd>
        <dt>Schema</dt><dd>{previewText(report.about.flow_id)} · {previewText(report.about.schema_version)}</dd>
        <dt>Impronta dello schema</dt><dd>{previewText(report.about.schema_hash)}</dd>
        <dt>Versione del validatore</dt><dd>{previewText(report.about.validator_version)}</dd>
        <dt>Versione dell’applicazione</dt><dd>{report.about.app_version}</dd>
        <Bundle bundle={report.about.bundle} />
        <dt>Tracciati</dt><dd>{report.about.trace_count}</dd>
        <dt>Dataset di riferimento</dt><dd>{report.about.reference_dataset_count}</dd>
      </dl>
    </>}
  </section>;
}
