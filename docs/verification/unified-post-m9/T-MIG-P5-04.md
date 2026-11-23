# T-MIG-P5-04 — Selezione formati di report

Data 23 novembre 2026. Stato: **IN REVIEW**, non DONE prima della review Codex
e dell'integrazione. Prerequisito T-MIG-P5-02 DONE (`cdf8373`). Branch
`work/T-MIG-P5-04` da `integration` `9ff674d` (P5-03 integrata), worktree
dedicato `/home/alberto/Desktop/LINTRA/lintra-p5-04`.

Fonti: prompt atomico; MIGRATION_PLAN v1.1 riga P5-04 («Parità utenti e
nessun writer o conversione JS»); preflight Slint→Tauri, riga «Formati
richiesti» (`ReportSelection`/`ReportCheck`: JSONL/XLSX/CSV bool, intento sul
run, XLSX+JSONL di default, CSV no, salvataggio di un formato alla volta);
codice Slint di riferimento `ui/src/analysis.rs` (`ReportSelection`,
`Default`), `ui/src/export.rs` (`preferred_format`), `ui/src/controller.rs`
(`select_reports`), `ui/ui/analysis.slint` (etichetta «Formati del report»,
ordine XLSX/JSONL/CSV), test `ui/tests/bootstrap.rs` (attivazione con
puntatore e Spazio) e `analysis_configuration.rs` (nessun formato è una
selezione valida e non scrive report); IPC `export_prepare` (P4-10), che
riceve già un formato per volta e lascia a Rust i writer canonici.

## Cosa cambia

- `src/analysis.ts`: `ReportSelection` (`Record<ExportFormat, boolean>`),
  `DEFAULT_REPORTS` uguale a `ReportSelection::default()`, `REPORT_FORMATS`
  nell'ordine del form Slint e `preferredFormat`, copia di
  `preferred_format` (XLSX, poi CSV, poi JSONL anche senza scelte). Lo stato
  porta `reports`; l'evento `set_report` cambia un formato alla volta, ignora
  formati sconosciuti e ammette ogni combinazione, nessuna inclusa. La scelta
  non dipende da flusso, schema, tracciato o piano (come
  `AnalysisSelection.reports` in Slint) e sopravvive a quei cambi e al
  ricaricamento del catalogo.
- `src/AnalysisForm.tsx`: gruppo «Formati del report» (`fieldset`/`legend`)
  con tre checkbox nativi etichettati e una nota collegata con
  `aria-describedby`: niente viene scritto ora, il report si salva dopo
  l'analisi un formato alla volta e quello proposto segue la scelta.
- `src/style.css`: layout del gruppo con i token esistenti, come i radio del
  tema; nessun colore nuovo.
- `tests/reports.test.mjs` (nuovo, 7 test); `tests/native_capture.py`: due
  scenari da tastiera sulle caselle.
- `apps/lintra-desktop/README.md`, riga di stato, questo dossier ed evidenze
  `T-MIG-P5-04-native/`.

`preferredFormat` non ha ancora un chiamante: lo userà l'export legato al run
(come `export_controller.rs` in Slint), che arriva con le task successive.
La selezione viene congelata sul run all'avvio (P5-05), come
`requested_reports` in Slint, e come in Slint (`enabled: !root.busy`) le
caselle andranno disabilitate durante il run; qui non è anticipato perché il
run non esiste ancora nell'interfaccia.

## Cosa NON cambia

Rust/src-tauri, IPC e DTO, CSP, dipendenze e lock, core/CLI/Slint, golden e
JSONL. Nessun writer, nessuna conversione o serializzazione di report nel
frontend: l'unico percorso di scrittura resta `export_prepare`/`export_execute`
con i writer Rust. Fuori scope: avvio e progress (P5-05), risultati ed export
nell'interfaccia, Full (P5-09).

## Acceptance

| Requisito | Prova |
| --- | --- |
| Parità utenti | default XLSX+JSONL, CSV no; ordine ed etichette XLSX/JSONL/CSV; titolo «Formati del report»; ogni combinazione ammessa, nessuna inclusa; scelta indipendente da flusso/schema/tracciato; formato proposto identico a `preferred_format` sulle 8 combinazioni; attivazione da tastiera con Spazio (nativo) |
| Nessun writer o conversione JS | test statico su tutto `src/` (nessun Blob, object URL, `download=`, writeFile/writeTextFile, plugin-fs, showSaveFilePicker, librerie XLSX/CSV); dipendenze runtime invariate (`@tauri-apps/api`, `react`, `react-dom`); la scelta cambia solo lo stato, nessuna chiamata IPC |

## Test

| Verifica | Esito |
| --- | --- |
| `npm run lint` | PASS rc 0 |
| `npm test` (69 + 7 `tests/reports.test.mjs`) | PASS 76/76 |
| `npm run build` | PASS rc 0 |
| `npm run tauri -- build --no-bundle -- --locked -j 2` + `tests/native_smoke.py` | PASS, binario `d8274f5e…` |
| `python3 tests/native_capture.py <bin> <out>` | PASS 18/18, tre esecuzioni consecutive rc 0 ([receipt](T-MIG-P5-04-native/receipt.json)) |
| src-tauri fmt/test/Clippy | NOT_RUN: nessun file Rust modificato |
| `npm ci` | NOT_RUN: engine pin node 26.13.0, locale 26.13.1 |
| Windows | NOT_RUN: nessun codice nativo cambiato |

Prova nativa (CachyOS/Hyprland): selezione da tastiera, poi Tab sul pulsante
del file, Tab su XLSX e Spazio (XLSX tolto), due Tab su CSV e Spazio (CSV
scelto). Le catture a 1366×768 e 640×480 mostrano XLSX vuoto, JSONL e CSV
spuntati, anello di focus su CSV, gruppo e nota interi; a 640×480 la pagina
scorre fino al controllo con il focus. Le tre esecuzioni danno la stessa
immagine a 1366×768; a 640×480 gli hash cambiano per la barra di scorrimento
a comparsa, con lo stesso contenuto.

## Rischi residui

- Le caselle usano l'aspetto nativo di WebKitGTK/WebView2, come i radio del
  tema (P5-01); a tema scuro la casella vuota ha un bordo poco marcato,
  dipendente dalla piattaforma. La qualifica Windows resta a G-MIG-5.
- La scelta è di sessione (nessuna persistenza delle preferenze, come in
  Slint).
- CI remota NOT_RUN; G-MIG-5 aperto.

Prossima tecnicamente disponibile dopo l'integrazione: T-MIG-P5-05.
