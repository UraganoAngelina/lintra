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
  `canCancel` (in preparing/running/finalizing, nessun cancel già richiesto,
  nessun avviso D09 aperto, dove Annulla è il cancel; vedi rework P506-01); `cancelRun` con una sola
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
- Test: `tests/cancel.test.mjs` (7), test Rust
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
| `npm run lint` / `npm test` / `npm run build` | PASS / 95 PASS / PASS (round 2) |
| `npm run tauri -- build --no-bundle -- --locked` + `native_smoke.py` | PASS, binario `d28d99d8…` (round 2; round 1 `d0552f78…`) |
| `python3 tests/native_capture.py <bin> <out>` | PASS 31/31 (round 2, una esecuzione; round 1 tre esecuzioni consecutive rc 0) ([receipt](T-MIG-P5-06-native/receipt.json)) |

Prova nativa: input di 245 MB; «Interrompi analisi» offerto durante il run;
cancel da tastiera → «Analisi annullata», «Nessun report completo pubblicato
per questa analisi», «Nessun report» nella barra, focus su «Avvia analisi»;
dopo un run completo, il run successivo annullato dichiara «Il report
dell'analisi precedente resta in Risultati: riguarda quell'analisi, non
questa» e la barra mantiene «Report disponibile».

| workspace core | NOT_RUN: nessun file core modificato |
| `npm ci` | NOT_RUN: engine pin node 26.13.0, locale 26.13.1 |
| CI remota / Windows | NOT_RUN |

Harness: la gestione di Shift+Tab aveva riusato la variabile `name` dello
scenario, e una prima esecuzione ha salvato le immagini col nome del tasto;
corretto e aggiunto il controllo «un'immagine per scenario» (assente in
precedenza). Le evidenze committate sono successive alla correzione.

## Rework dopo review Codex round 1

Review round 1 APPROVED su `6b6a961` con due MINOR non bloccanti; chiusi su
richiesta dell'autore prima dell'integrazione. Solo frontend: Rust, IPC e core
invariati.

- **P506-01 — finalizing anticipava la disabilitazione del cancel.**
  `LIFECYCLE.md` distingue finalizing prima del confine del servizio (cancel
  ancora `requested`) e dopo (`too_late`); Rust passa a finalizing già a
  `PhaseStarted(FinalizingReport)` e sigilla solo a `RunCompleted`. `canCancel`
  ora offre «Interrompi analisi» anche in finalizing; la disposizione Rust
  decide e `too_late` era già gestito (`CANCEL_TOO_LATE`, nessuna seconda
  richiesta). Commento corretto: la UI non dichiara più un'impossibilità che
  solo Rust conosce. Test: i due confini sono distinti (`requested` →
  «Interruzione richiesta», `too_late` → messaggio dedicato e pulsante
  inattivo).
- **P506-02 — risposta persa mostrata come richiesta non registrata.** Un
  errore di trasporto non prova che Rust abbia ignorato il comando. Testo
  neutro: «Non è stato possibile confermare la richiesta di interruzione. Se
  l'analisi prosegue, riprova.» (`role="status"`, non più `alert`). Il reducer
  risincronizza: uno snapshot dello stesso run con `cancel_requested` o
  `cancelling` porta `failed` → `requested`, l'avviso sparisce e il retry non
  è più offerto; uno snapshot senza conferma lascia il retry (idempotente in
  Rust: `already_requested`). Test nuovo: risposta persa → snapshot positivo
  (running e cancelling), snapshot negativo, snapshot di un altro run
  ignorato, pannello senza messaggi contraddittori.

Verifica di regressione: i test nuovi falliscono (3) sul sorgente di
`6b6a961` e passano sul rework. Gli scenari nativi annullano durante
`running` e non toccano i due stati modificati; le evidenze sono state
rigenerate col binario nuovo (31 hash verificati contro il receipt), le
capture di cancel sono state controllate visivamente.

## Rischi residui

- Il cancel interrompe al prossimo checkpoint del servizio: su input molto
  grandi la conclusione richiede qualche istante, mostrato come
  «Interruzione richiesta».
- La sezione Risultati non mostra ancora il contenuto del report (task
  successive): oggi indica solo che un report esiste.
- CI remota NOT_RUN; Windows NOT_RUN; G-MIG-5 aperto.

Prossima tecnicamente disponibile dopo l'integrazione: T-MIG-P5-07.
