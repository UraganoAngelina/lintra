# T-MIG-P5-07 — Informazioni/provenienza e licenze

Data 23 novembre 2026. Stato: **IN_REVIEW** (READY_FOR_CODEX_REVIEW), non DONE
prima della review e dell'integrazione. Prerequisiti T-MIG-P5-01 e T-MIG-P5-06
DONE (`integration` `db3cd7c`). Branch `work/T-MIG-P5-07` da `db3cd7c`,
worktree `/home/alberto/Desktop/LINTRA/lintra-p5-07`.

Fonti: prompt atomico («Dati di provenienza corretti; Slint notice rimane dove
richiesto per legacy, Tauri notices corrette»); MIGRATION_PLAN v1.1 riga
P5-07; Master Plan §6; preflight Slint→Tauri righe 222, 240 e 290 (Informazioni
= `refresh_information` + `AboutSlint`; «provenienza dello stesso run,
testo/licenze frontend»); Slint `ui/ui/main.slint` (vista Informazioni),
`ui/src/export_controller.rs::refresh_information`,
`src/bundle_acquisition.rs::origin_label`; `scripts/release.py::ui_licenses`;
IPC v1 `EXPORT.md` (`report_about`); dossier T-MIG-P3-02 (origine di
`public/THIRD_PARTY_NOTICES`).

## Cosa cambia

- `src/About.tsx` (nuovo): sezione Informazioni con versione (`app_info`),
  «Analisi selezionata» (schema, impronta completa, origine e manifest del
  bundle dalla selezione Rust; senza schema, l'origine del catalogo),
  «Ultimo report» (da `report_about`, letto a ogni apertura della sezione e a
  ogni nuovo report pubblicato; risposte tardive scartate) e «Licenze di terze
  parti» (un `<details>` chiuso per testo, `<pre>` con `tabIndex=0` per lo
  scroll da tastiera). Il report e la selezione restano separati e il report
  dichiara che riguarda l'analisi che lo ha prodotto: è la stessa distinzione
  di Slint (vista risultati contro selezione), mostrata insieme.
- `src/about.ts` (nuovo): `splitNotices` (taglio esatto dei blocchi, testo
  verbatim), `previewText` (anteprima Rust troncata marcata con «…»),
  `reportStatusText` (stato, completezza e troncamento come li dichiara Rust;
  annullato o troncato non diventano mai una conformità).
- `src/analysis.ts`: `provenanceText` spostato qui da `AnalysisForm.tsx`, ora
  condiviso dalle due viste (stesso testo).
- `src/AppShell.tsx`: prop `information`; senza di essa resta il corpo P5-01.
- `src/main.tsx`: importa `public/fonts/OFL.md` e
  `public/THIRD_PARTY_NOTICES` con `?raw` (nel bundle; la CSP non permette
  `fetch`).
- `public/THIRD_PARTY_NOTICES`: i blocchi P3-02 restano identici byte per
  byte; aggiunte le licenze originali delle crate runtime Tauri collegate
  nell'eseguibile, alle versioni di `Cargo.lock`: tauri 2.12.5,
  tauri-runtime 2.12.2, tauri-runtime-wry 2.12.3, tauri-utils 2.10.2 (stessi
  file, un blocco per licenza), wry 0.57.2 (Apache-2.0 e MIT), tao 0.37.2
  (Apache-2.0). Testi copiati dal registry Cargo locale, verificati per hash.
- `assets-provenance.json`: sezione `notices` con pacchetti, versione, file e
  SHA-256 di ogni blocco.
- `.gitattributes`: `THIRD_PARTY_NOTICES` con `eol=lf`, come `OFL.md`, perché
  gli hash valgano anche sul checkout Windows.
- `src/style.css`: stili della sezione con i token esistenti.
- Test: `tests/about.test.mjs` (8), tre scenari in `tests/native_capture.py`.
- README desktop.

## Cosa NON cambia

Backend Rust, IPC v1 e core: `report_about` esisteva già da P4-10. Nessuna
dipendenza nuova. La UI Slint legacy e la sua attribuzione: `AboutSlint` in
`ui/ui/main.slint` e la licenza Slint nell'archivio UI di `release.py`
restano dove sono (verificato dal test). Risultati (P6) e Full (P5-09/P5-10)
fuori scope.

## Acceptance

| Requisito | Prova |
| --- | --- |
| Dati di provenienza corretti | tutti i valori vengono da Rust (`app_info`, `configuration_select`, `report_about`); test di render: selezione v2 e report v1 mostrati ciascuno col proprio schema/impronta/bundle; anteprima troncata marcata; esiti parziali/annullati mai come conformità. Nativo: dopo un run reale, report e selezione con schema, impronta, validatore, bundle `synthetic-embedded-v1`, manifest presente, 1 tracciato |
| Slint notice dove richiesto per legacy | test: `AboutSlint` presente in `main.slint`, `ui_licenses()` include ancora `SLINT-ROYALTY-FREE-2.0.md`; nessuna crate `slint` nel `Cargo.lock` desktop, nessuna notice Slint nell'app React |
| Tauri notices corrette | test: il file è esattamente la concatenazione dei blocchi dichiarati (hash per blocco); i blocchi npm coprono l'intera chiusura runtime del `package-lock` e coincidono coi file in `node_modules`; le crate Tauri runtime hanno la versione di `Cargo.lock`; tauri e wry con entrambe le licenze. Mutazione: il file di `integration` e un file con un byte alterato fanno fallire 2 test |

## Test

| Verifica | Esito |
| --- | --- |
| `npm run lint` / `npm test` / `npm run build` | PASS / 103 PASS (95 + 8) / PASS |
| desktop `cargo fmt … -- --check` | PASS |
| desktop `cargo test … --release --target x86_64-unknown-linux-gnu --features tauri/custom-protocol -j 2` | PASS 82, 0 fail, 1 ignored |
| desktop `cargo clippy … --release --all-targets … -- -D warnings` | PASS |
| `npm run tauri -- build --no-bundle -- --locked -j 2` + `native_smoke.py` | PASS, binario `efb43ac6…a198008` |
| `python3 tests/native_capture.py <bin> <out>` | PASS 34/34, rc 0 ([receipt](T-MIG-P5-07-native/receipt.json)) |
| workspace core | NOT_RUN: nessun file core modificato |
| `npm ci` | NOT_RUN: engine pin node 26.13.0, locale 26.13.1 |
| CI remota / Windows / WebView2 | NOT_RUN |

Prova nativa (Hyprland, controllata a vista): Informazioni da tastiera, Tab
sulla prima licenza, Invio la apre, Tab porta il focus sul testo scorrevole,
a 1366×768 e a 640×480; dopo un run completo, «Ultimo report» mostra la
provenienza Rust e la barra «Report disponibile».

## Rischi residui

- L'inventario completo delle licenze Rust (centinaia di crate transitive:
  serde, gtk, webkit2gtk, windows…) non è in questa task: spetta a
  T-MIG-P8-02 (audit licenze/SBOM). Qui sono coperti i pacchetti JS del
  bundle e le crate runtime Tauri.
- Il hook `useReportAbout` non ha un test unitario (servirebbe un DOM in
  Node); lo copre la prova nativa.
- CI remota e Windows NOT_RUN; G-MIG-5 aperto.

Prossima tecnicamente disponibile dopo l'integrazione: T-MIG-P5-09.
