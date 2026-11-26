# T-MIG-P6-08 — Export UI con receipt e overwrite

Data 26 novembre 2026. Stato: **IN_REVIEW** (non DONE). Prerequisito
T-MIG-P6-07 DONE (`integration` `a2d9f75`, che comprende anche la mini-task
FIX-RUN-PROGRESS). Branch `work/T-MIG-P6-08` da `a2d9f75`, worktree
`/home/alberto/Desktop/LINTRA/lintra-p6-08`.

Fonti: prompt atomico `MIGRAZIONE_89/T-MIG-P6-08.md`; MIGRATION_PLAN v1.1
riga P6-08 («Export sempre intero e byte-identico al core; due
pubblicazioni non dette atomic pair»); IPC v1 `EXPORT.md` (prepare con solo
report_ref e formato, dialogo e conferme native, execute/state/cancel, receipt
con `published_roles`, `cleanup_failed`, `durability_confirmed`, conteggi di
destinazione; pubblicazione per file con `AtomicReportFile`, nessuna
transazione multi-file; errore di risposta che non attesta assenza di
pubblicazione: nessun retry cieco); `docs/EXPORT_DESTINATIONS.md`; adapter
`src-tauri/src/exports.rs` e `native_picker.rs` (T-MIG-P4-10); client
`ipc.ts` (`exportPrepare`, `exportExecute`, `exportState`, `exportCancel`);
Slint `ui/src/export_controller.rs` e `src/report/export.rs`
(`ExportFailure` Display: testi di annullamento, pubblicazione parziale,
pulizia). Il `TASK_PROMPTS/T-MIG-P6-08.md` originale non è nel kit.

## Preflight

Backend e client erano già completi dalla T-MIG-P4-10: dialogo di
salvataggio nativo, conferma di sovrascrittura distinta anche per il solo
sidecar, avviso di destinazione remota/indeterminata, protezione di tutti
gli input (anche Full) e dello spool, `verify_unchanged` prima dello staging
e della pubblicazione, writer canonici JSONL/CSV/XLSX. Il test Rust
`local_only_and_full_reports_export_canonical_files_through_the_bridge`
confronta l'export con il writer canonico per LocalOnly e Full nei tre
formati; `full_inputs_are_protected_and_overwrite_is_a_separate_native_choice`
copre la sovrascrittura. Nessun componente React usava l'export: la task è
lato renderer, senza modifiche Rust.

## Cosa cambia

**`src/Export.tsx`** (nuovo):
- `startExport`: `export_prepare` (solo `report_ref` e formato: dialogo e
  conferme sono di Rust), poi `export_execute` e `followExport`, che legge
  `export_state` ogni 250 ms fino all'esito. Ogni risposta passa da `live()`.
- `cancelExport`: la richiesta di annullamento non decide l'esito, lo decide
  la receipt; «già concluso» lascia parlare la lettura successiva.
- Risposta persa durante l'esecuzione: stato «Esito del salvataggio non
  disponibile: alcuni file potrebbero essere già stati salvati…» con
  «Controlla l'esito», che rilegge la receipt conservata da Rust. Nessun
  nuovo salvataggio automatico.
- `receiptText`: successo come «Report e riepilogo salvati in formato X.
  Sono due file pubblicati uno dopo l'altro, non un'unica scrittura.»;
  fallimento o annullamento con i ruoli pubblicati (nessuno, solo report,
  solo riepilogo, entrambi ma incompleto), durabilità non confermata,
  pulizia non riuscita (testi del core) e destinazione remota confermata.
- `useExport`: legato all'identità del report pubblicato (RunRef), non alla
  sezione visibile: uscire da Risultati durante la scrittura non perde la
  receipt; un nuovo report invalida le risposte tardive.
- `ExportCard` «Salva il report»: formato (ordine Slint XLSX, JSONL, CSV;
  proposto il primo richiesto con l'analisi, `preferredFormat`), «Salva
  report…», stati con `role` `status`/`alert`/`note`; report annullato o
  troncato non esportabile con il motivo, Partial esportabile. A ogni nuovo
  stato il messaggio viene portato in vista (`scrollIntoView`, `nearest`):
  la prima campagna lo lasciava sotto il bordo della finestra.

**`Results.tsx`**: la scheda in fondo alla pagina, così l'ordine di
tabulazione di riepilogo, filtri e griglia resta quello di prima.
**`main.tsx`**: collegamento al report pubblicato e al formato proposto.
**`style.css`**: scheda.

**Test** `tests/export.test.mjs` (nuovo, 7 test) con il client IPC reale
sotto `mockIPC`: richieste esatte (`export_prepare` con solo
`schemaVersion`, `report_ref`, `format`; poi `export_execute` e
`export_state` con il solo `export_ref`), nessun path, nome file, overwrite
o consenso dal renderer; dialogo chiuso, conferma rifiutata, destinazione
rifiutata, prepare fallito senza job; risposta persa come esito ignoto e
rilettura; polling che si ferma con un nuovo report; annullamento richiesto,
tardivo o fallito; receipt per ogni combinazione di ruoli, durabilità,
pulizia e conferma remota; scheda e ruoli accessibili; nessun Blob, download
o writer nel sorgente. Prove di mutazione: successo detto «atomico»,
polling senza `live()`, risposta persa detta «nessun file scritto»,
durabilità nascosta: ciascuna fallisce (2, 1, 1, 1 FAIL); codice reale 7/7.

**Banco nativo** (`native_capture.py`): `results-export-1366x768` salva il
report di `fields.txt` in JSONL tramite il vero dialogo GTK;
`results-export-cancel-1366x768` chiude il dialogo; `results-export-640x480`.
Il receipt registra dimensione e SHA-256 dei due file scritti da Rust.

## Cosa NON cambia

Core, writer, adapter export Rust, dialoghi e conferme native, contratto IPC
v1, CLI, golden, SQLite, dipendenze.

## Acceptance

| Requisito | Prova |
| --- | --- |
| Export sempre intero | il renderer non invia filtri né contenuto: solo `report_ref` e formato (test); nota «I filtri della tabella non limitano l'esportazione» |
| Byte-identico al core | test Rust del bridge contro il writer canonico (3 formati, LocalOnly e Full); JSONL salvato in nativo identico all'output della CLI dello stesso albero (SHA-256 `00b012d5…1d1d`) |
| Due pubblicazioni non dette atomic pair | testo di successo «due file pubblicati uno dopo l'altro, non un'unica scrittura»; parziali espliciti per ruolo (test, mutazione); cattura `results-export` |
| Overwrite | conferma nativa distinta di Rust (test Rust esistente); il renderer non ha alcun flag |

## Test

| Verifica | Esito |
| --- | --- |
| `npm run lint` / `npm test` / `npm run build` | PASS / 182 PASS (7 nuovi) / PASS |
| Prove di mutazione su `Export.tsx` | 4 mutazioni, ciascuna con FAIL; codice reale 7/7 |
| comandi Linux di `desktop.yml`: `cargo fmt -- --check` / `cargo test --locked --release --target x86_64-unknown-linux-gnu --features tauri/custom-protocol -j 2` / `cargo clippy … --all-targets … -- -D warnings` / `cargo build …` | PASS (0) / 86 passed, 0 failed, 1 ignored (0) / PASS (0) / PASS (0) |
| `native_smoke.py` sul binario del target CI (sessione Hyprland reale: `xvfb-run` non installato) | PASS / 0, `f1bebcd2007b3dd2e3989d5a6d6cae7945a532a812204833fa79aab411d0ff1f` |
| `npm run tauri -- build --no-bundle` + `native_smoke.py` | PASS / 0, binario `e0656b709d041a54e2b51c33b79579104e247944825326e750b1f6a89035f7da` |
| `python3 tests/native_capture.py <bin> <out>` | PASS 74/74 + chiusura durante il run (exit 0, 0,02 s, nessuno spool residuo) su `e0656b70…f7da` ([receipt](T-MIG-P6-08-native/receipt.json)); 71 scenari precedenti + 3 export. Una campagna precedente (binario prima del `scrollIntoView`) è stata scartata e ripetuta per intero |
| Confronto JSONL esportato / CLI `lintra validate` | PASS: `r.jsonl` salvato dal dialogo nativo (receipt `export`) = output `--errors` della CLI costruita dallo stesso albero (`cargo build --release --locked --bin lintra`, `ad68628f…5739`) su `fields.txt` con `--schema-dir bundles/embedded/schemas`: entrambi 2672 byte, SHA-256 `00b012d5c46436f4f0baa5f800f22e2f808d2b7898d331d56f104533a2e41d1d` (CLI exit 2 = INVALID, 3 diagnostiche). Il riepilogo non è confrontabile byte per byte: contiene `started_at` ed `elapsed` |
| `npm ci --ignore-scripts` | NOT_RUN in locale: EBADENGINE (node 26.13.1/npm 12.2.3 contro i pin) |
| workspace `ci.yml` | NOT_RUN: core non toccato |
| Windows/WebView2 e CI remota | NOT_RUN (al checkpoint del blocco P6) |

## Rischi residui

- Sovrascrittura e avviso di destinazione remota sono conferme native di
  Rust: provate dai test Rust esistenti, non dal banco nativo (che non
  automatizza le finestre di conferma).
- In nativo si prova solo JSONL; CSV e XLSX restano coperti dal test Rust del
  bridge.
- Il dialogo GTK seleziona solo il nome base del nome proposto
  (`report.<ext>`): un percorso digitato con estensione diventa
  `nome.<ext>.<ext>`. È un comportamento del dialogo, non toccato qui.

Prossima task tecnicamente disponibile dopo l'integrazione: T-MIG-P6-09.
