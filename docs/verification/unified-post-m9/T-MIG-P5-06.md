# T-MIG-P5-06 — Cancel, stato terminale e old report

Data 23 novembre 2026. Stato: **IN REVIEW**, non DONE prima della review Codex
e dell'integrazione. Prerequisito T-MIG-P5-05 DONE (`integration` `3a5d685`).
Branch `work/T-MIG-P5-06` da `3a5d685`, worktree
`/home/alberto/Desktop/LINTRA/lintra-p5-06`.

Fonti: prompt atomico («Nessuna falsa conformità o report nuovo dopo cancel,
test state machine»); MIGRATION_PLAN v1.1 riga P5-06; Master Plan §6;
`MIGRATION_TAURI_BRIEF.md` (Cancellazione richiesta: «Interruzione
richiesta», attendere il terminale Rust; Cancelled: «Analisi annullata», dire
quali report sono davvero disponibili; cancel solo quando Rust consente
l'interruzione); IPC v1 `LIFECYCLE.md` (disposizioni cancel, race «errore o
cancel nuovo contro report precedente»); preflight Slint→Tauri riga 216 e
RSK-MIG-02; Slint `ui/src/worker_controller.rs` (testi annullata/parziale,
`last_completed` conservato senza diventare l'esito corrente).

## Cosa cambia

- `src/run.ts`: stato di cancel del run attivo (`none`, `requesting`,
  `requested`, `too_late`, `failed`) mappato dalle quattro disposizioni Rust;
  `canCancel` (solo in preparing/running, nessun cancel già richiesto, nessun
  avviso D09 aperto, dove Annulla è il cancel); `cancelRun` con una sola
  richiesta; `report` = run di cui Rust conserva l'ultimo report pubblicato,
  aggiornato solo da un terminale con `report_available`; testi: «Analisi
  annullata» con «Nessun report completo pubblicato per questa analisi», e se
  esiste un report precedente «Il report dell'analisi precedente resta in
  Risultati: riguarda quell'analisi, non questa»; troncamento esplicito come
  «Analisi parziale» (mai una conformità), come in Slint.
- `src/RunPanel.tsx`: pulsante secondario «Interrompi analisi» dopo «Avvia»,
  lasciato in posizione (disabilitato, «Interruzione richiesta») dopo la
  richiesta per non perdere il focus; al terminale, se il focus era su di lui,
  passa ad «Avvia analisi»; messaggi per too_late e richiesta fallita; testo
  dell'esito con l'identità del report precedente.
- `src/main.tsx`: «Report disponibile» segue il report che Rust conserva,
  non l'esito dell'ultimo run.
- `src/style.css`: pulsante secondario con i token esistenti.
- Test: `tests/cancel.test.mjs` (6), test Rust
  `cancel_of_a_newer_run_keeps_the_previous_report_and_publishes_nothing`,
  tre scenari nativi in `tests/native_capture.py`.

## Cosa NON cambia

Backend Rust e IPC (le operazioni di cancel e il report precedente erano già
nel contratto e nel job manager P4): il test Rust nuovo verifica il
comportamento esistente. Nessuna dipendenza nuova. Risultati e Informazioni
(P5-07), Full (P5-09/P5-10) fuori scope.

## Acceptance

| Requisito | Prova |
| --- | --- |
| Nessun report nuovo dopo cancel | Rust: run tenuto in PASS1, cancel `requested` → `cancelled`, `partial`, `report_available=false`, nessuno spool nuovo; cancel tardivo `already_terminal`; frontend: un terminale annullato non aggiorna `report` |
| Nessuna falsa conformità | testi: annullato, non iniziato, errore tecnico, troncato non producono mai «Nessuna violazione» o «Violazioni rilevate»; la richiesta di cancel non è un esito (il run resta attivo finché Rust non manda il terminale, che può anche essere completo) |
| Old report con la propria identità | Rust: il report del run precedente resta interrogabile, il run annullato è `IPC_STALE_CONTEXT` per i risultati; UI: report precedente dichiarato tale |
| Test state machine | `tests/cancel.test.mjs`: offerta del cancel per stato, una sola richiesta, disposizioni, retry dopo errore, risposte di altri run ignorate, outcome che decide, sostituzione del report solo con un nuovo report pubblicato |

## Test

| Verifica | Esito |
| --- | --- |
| desktop `cargo fmt … -- --check` / `clippy … -D warnings` (release, all-targets) | PASS / PASS |
| desktop `cargo test … --release --target x86_64-unknown-linux-gnu --features tauri/custom-protocol` | PASS 82, 0 fail, 1 ignored |
| `npm run lint` / `npm test` / `npm run build` | PASS / 94 PASS / PASS |
| `npm run tauri -- build --no-bundle -- --locked` + `native_smoke.py` | PASS, binario `d0552f78…` |
| `python3 tests/native_capture.py <bin> <out>` | PASS 31/31, tre esecuzioni consecutive rc 0 ([receipt](T-MIG-P5-06-native/receipt.json)) |

Prova nativa: input di 245 MB; «Interrompi analisi» offerto durante il run;
cancel da tastiera → «Analisi annullata», «Nessun report completo pubblicato
per questa analisi», «Nessun report» nella barra, focus su «Avvia analisi»;
dopo un run completo, il run successivo annullato dichiara «Il report
dell'analisi precedente resta in Risultati: riguarda quell'analisi, non
questa» e la barra mantiene «Report disponibile».

Harness: la gestione di Shift+Tab aveva riusato la variabile `name` dello
scenario, e una prima esecuzione ha salvato le immagini col nome del tasto;
corretto e aggiunto il controllo «un'immagine per scenario» (assente in
precedenza). Le evidenze committate sono successive alla correzione.
| workspace core | NOT_RUN: nessun file core modificato |
| `npm ci` | NOT_RUN: engine pin node 26.13.0, locale 26.13.1 |
| CI remota / Windows | NOT_RUN |

## Rischi residui

- Il cancel interrompe al prossimo checkpoint del servizio: su input molto
  grandi la conclusione richiede qualche istante, mostrato come
  «Interruzione richiesta».
- La sezione Risultati non mostra ancora il contenuto del report (task
  successive): oggi indica solo che un report esiste.
- CI remota NOT_RUN; Windows NOT_RUN; G-MIG-5 aperto.

Prossima tecnicamente disponibile dopo l'integrazione: T-MIG-P5-07.
