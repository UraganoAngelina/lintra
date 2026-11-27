# T-MIG-P6-10 — React Full: risultati cross-trace, detail ed export canonico

Data 27 novembre 2026. Stato: **DONE**. Review Codex round 1 APPROVED su
`3a99726` senza BLOCKER o MAJOR (`lintra-review-handoffs/T-MIG-P6-10-CODEX-REVIEW.md`).
Integrazione autorizzata dall'autore per fast-forward di `integration`,
senza push iniziale. G-MIG-6 e G-REL NOT_VERIFIED fino al checkpoint del
blocco P6 (CI remota e Windows/WebView2). Prerequisiti
T-MIG-P6-09 DONE (`integration` `436aeca`) e T-MIG-P5-10 DONE. Branch
`work/T-MIG-P6-10` da `436aeca`, worktree
`/home/alberto/Desktop/LINTRA/lintra-p6-10`.

Fonti: prompt atomico `MIGRAZIONE_89/T-MIG-P6-10.md`; MIGRATION_PLAN v1.1
riga P6-10 («Parità Full e LocalOnly dell'oracolo Rust provata; nessuna
logica cross-record ricreata in React») e G-MIG-6; `MIGRATION_SCOPE_PARITY.md`
(Full obbligatorio: P6-10); IPC v1 `DETAIL` (contesto per voce con tracciato)
ed `EXPORT.md`; dossier P5-09, P5-10, P6-01..P6-09. Il
`TASK_PROMPTS/T-MIG-P6-10.md` originale non è nel kit.

Richiesta dell'autore (27 novembre 2026): nei test di questa task controllare
il **contenuto** dei file XLSX, non solo la firma ZIP.

## Preflight

- P6-09 e P5-10 integrate; nessuna implementazione precedente di P6-10.
- Il renderer non esclude il Full da Risultati: griglia, nomi di tracciato
  (da `results_overview`), dettaglio e contesto sono già generici. Il banco
  nativo però non aveva nessuna cattura di risultati Full, e nessun test
  provava il dettaglio cross-trace (guida e posizione del campo del
  tracciato giusto, contesto con il valore dell'altro tracciato) né il
  contenuto dei file XLSX.
- Sonda sul backend (non committata): per una diagnostica `E_CROSS_FIELD` di
  S il dettaglio porta posizione 5–6 e guida del campo DATA di S, e il
  contesto le voci `S · DATA = NO` e `T · DATA = OK`, ognuna col suo
  tracciato. Il core salta la PASS2 se un tracciato ha un record corto
  (copertura parziale): comportamento del servizio, non del desktop.

## Cosa cambia

**Rust (solo test), `src-tauri/src/exports/golden_parity.rs`:**
- **XLSX, contenuto.** Per i 15 golden di P6-09 (12 LocalOnly, 3 Full SIOC)
  l'export XLSX tramite il bridge è confrontato byte per byte con il writer
  canonico (deterministico: la data di creazione è `started_at` del
  summary), poi riaperto come ZIP: fogli `Riepilogo` ed `Errori` in
  quest'ordine; in `Errori` l'intestazione e una riga per diagnostica del
  golden, in ordine, con le 14 colonne prese dal JSONL atteso (codice,
  flusso, versione, tracciato, linea, ID record, codice fiscale, ID e nome
  campo, inizio, fine, valore letto, atteso, messaggio); in `Riepilogo` il
  summary del report appiattito in percorsi JSON Pointer.
- **Dettaglio Full cross-trace** (`full_cross_trace_detail_matches_the_spool_and_the_captured_schema`):
  sul bundle a due tracciati S/T, per ogni riga (record corto, confronto
  cross-trace, record correlato assente) codice, messaggio, atteso, nome
  campo, guida `field_format_guide` del campo del tracciato della
  diagnostica, posizione in byte, valore letto e ogni voce di contesto con
  il suo tracciato (anche T); filtro per tracciato su S e T.
- `Cargo.toml`: dev-dependency `zip = "=9.0.5"` (stesso pin del workspace),
  per leggere il workbook. Era già nel lockfile tramite `rust_xlsxwriter`:
  `Cargo.lock` aggiunge solo il collegamento, nessun pacchetto nuovo.

**Renderer, `src/Results.tsx`:** la lista delle voci di contesto è estratta
da `ContextList` in `ContextEntries` (stesso markup), per provarla con voci di
due tracciati. Nessun cambio di comportamento.

**Test Node** `tests/full-results.test.mjs` (5): righe Full con il nome di
tracciato dato da Rust per il loro handle (un handle sconosciuto non viene
decodificato); dettaglio cross-trace con guida, posizione e tracciato;
contesto con voci di S e T e una non esposta; export di un report Full
intero; nessun sorgente del renderer contiene concetti della PASS2 (join,
chiavi, cardinalità, tracciato correlato).

**Banco nativo** `tests/native_capture.py`:
- campione `fullsr.txt` (S con tre diagnostiche) contro `badt.txt`;
- scenari `results-full-1366x768` (Risultati Full),
  `results-full-detail-context-workarea` (dettaglio cross-trace con il
  contesto di S e T; area di lavoro perché a 1366×768 la seconda voce resta
  sotto il bordo del pannello) e `results-full-export-1366x768` (salvataggio
  nel formato proposto, XLSX, come `rf.xlsx`);
- le celle di `rf.xlsx` sono rilette con `zipfile` (libreria standard) e
  registrate nel receipt;
- `LINTRA_CAPTURE_ONLY=<prefisso>[,…]`: esegue solo gli scenari scelti più le
  baseline delle loro dimensioni, senza ripetere l'intera campagna.

**Oracolo** `tests/full_xlsx_oracle.py`: ricostruisce bundle e input della
campagna, esegue `lintra validate-flow` (stessi parametri) e confronta le
celle di `rf.xlsx` con il JSONL e il summary della CLI (esclusi orario,
durata e provenienza `bundle_*`, che il workbook non porta).

## Cosa NON cambia

Core, CLI, golden, writer, adapter Rust di produzione, contratto IPC v1,
client `ipc.ts`, comportamento del renderer, workflow. Nessuna logica
cross-record in React: il renderer mostra ciò che Rust manda.

## Acceptance

| Requisito | Prova |
| --- | --- |
| Parità Full con l'oracolo Rust | 3 golden SIOC Full (P6-09) più XLSX; dettaglio cross-trace contro spool e schema catturato; nativo: griglia, dettaglio con contesto S/T, export XLSX = CLI `validate-flow` |
| Parità LocalOnly con l'oracolo Rust | 12 golden (P6-09) più contenuto XLSX |
| Export canonico | JSONL/CSV/XLSX contro writer canonico e golden; XLSX cella per cella |
| Nessuna logica cross-record in React | test sui sorgenti; il contesto arriva da Rust con il tracciato di ogni voce |

## Test

| Verifica | Esito |
| --- | --- |
| `cargo fmt -- --check` (desktop) | PASS (0) |
| `cargo test --locked --release --target x86_64-unknown-linux-gnu --features tauri/custom-protocol -j 2` | PASS (0): 89 passed, 1 ignored (1 test nuovo, 2 estesi) |
| `cargo clippy … --all-targets … -- -D warnings` / `cargo build …` | PASS / PASS |
| Mutazioni del writer XLSX del core (solo `golden_parity`) | messaggio vuoto nel foglio Errori: 2 FAIL; percorsi del Riepilogo senza `/`: 2 FAIL; un terzo tentativo (trim dei valori) era un mutante equivalente, sostituito; core ripristinato e verificato con `cmp` |
| `npm run lint` / `npm test` / `npm run build` | PASS / 190 PASS (5 nuovi) / PASS |
| `native_smoke.py` (sessione reale, `xvfb-run` assente) | PASS (0), binario `463631205bfd9dd8d7473c72bb529310c13681574242f580ea2d3a4b09beaa19` |
| `LINTRA_CAPTURE_ONLY=results-full python3 tests/native_capture.py <bin> <out>` | PASS (0): 3 scenari nuovi + 2 baseline sullo stesso binario ([receipt](T-MIG-P6-10-native/receipt.json)); `rf.xlsx` 7844 byte e sidecar 1739 byte |
| `cargo build --release --locked --bin lintra` + `python3 tests/full_xlsx_oracle.py <cli> <receipt>` | PASS: CLI `8bfe2641b2b9e9e81d91e791dc308b7c31096ef84ceacb6de9a241c572e4d728`, exit 2 (INVALID); 3 diagnostiche e 68 righe di riepilogo uguali |
| Campagna completa (74 scenari) | NOT_RUN di proposito: nessuno stato già catturato cambia (renderer invariato nel comportamento) |
| `npm ci` con i pin; `ci.yml` completo | NOT_RUN: node/npm locali diversi dai pin; core non toccato |
| CI remota, Windows/WebView2 | NOT_RUN: checkpoint del blocco P6, dopo questa task (decisione dell'autore del 27 novembre) |

## Rischi residui

- A 1366×768 il pannello di dettaglio mostra solo la prima voce di contesto
  senza scorrere; la cattura usa l'area di lavoro. Dopo «Mostra il contesto»
  il pulsante sparisce e il focus torna al documento: comportamento di
  P6-03, non modificato qui.
- G-MIG-6 non è dichiarato: mancano Windows/WebView2 e CI remota sullo
  stesso SHA (checkpoint del blocco P6, da autorizzare: richiede push).

Prossimo passo: checkpoint del blocco P6 (CI remota e Windows/WebView2) per
G-MIG-6; poi T-MIG-P7-01.
