# T-MIG-P5-08 — Gate parità configurazione/worker/UX

Data 24 novembre 2026. Stato: **IN_REVIEW** (non DONE). Mancano la review
Codex, l'approvazione UX dell'autore sulla galleria e la CI remota sullo SHA
consegnato. Prerequisiti T-MIG-P5-01..07, 09 e 10 DONE (`integration`
`5e7f078`). Branch `work/T-MIG-P5-08` da `5e7f078`, worktree
`/home/alberto/Desktop/LINTRA/lintra-p5-08`.

Decisioni dell'autore (23 novembre 2026): push e CI remota solo dopo la review
Codex, sullo stesso SHA; prova Windows 11 nella VM come in P5-01; la parte
«keyboard/UX responsive approvata» è approvata dall'autore su una galleria di
catture. Scope esteso autorizzato dall'autore il 24 novembre: correzione del
difetto di posizionamento della finestra Windows trovato da questo gate
(codice P5-01, `main.rs`).

Fonti: prompt atomico; MIGRATION_PLAN v1.1 riga T-MIG-P5-08 («Parità
configurazione/worker LocalOnly+Full, keyboard/UX responsive approvata») e
G-MIG-5 (QG-1/QG-7: redesign responsive, config/preflight/run LocalOnly+Full,
esiti, Informazioni, accessibilità); `MIGRATION_SCOPE_PARITY.md` (matrice di
scope e criteri vincolanti di parità); `.github/workflows/desktop.yml` e
`ci.yml`; dossier P5-01..P5-10. Il `TASK_PROMPTS/T-MIG-P5-08.md` originale non
è nel kit.

## Preflight

- P5-10 integrata (`98bdf80`, chiusura `5e7f078`); nessuna implementazione
  precedente di P5-08 in Git.
- I dossier P5-02..P5-10 rinviavano a questo checkpoint CI remota,
  Windows/WebView2 e `npm ci` con i pin esatti.
- L'harness Windows di P5-01 copriva solo la shell (navigazione e tema); i
  flussi P5-02..P5-10 su WebView2 non erano mai stati provati.

## Matrice di parità

| Criterio (`MIGRATION_SCOPE_PARITY.md`) | Prova |
| --- | --- |
| LocalOnly: adapter e invocazione diretta Rust su input identici (esito, completezza, JSONL canonico) | `jobs::tests::actual_local_worker_retains_terminal_report_identity_and_matches_core_jsonl`, PASS su Linux e MSVC |
| Full: idem, più riepilogo | `jobs::tests::full_worker_matches_direct_service_outcome_summary_and_canonical_jsonl` (valido, cross-field, record correlato mancante), PASS su Linux e MSVC |
| Esito e completezza su due assi; annullato, troncato e parziale mai successo | test `jobs.rs` (troncato → Partial senza report, cancel per passata, cancel dopo checkpoint); matrice esiti P5-10 (24 casi) nei test Node; nativo Linux e Windows: annullato senza report |
| Errore tecnico distinto da violazione | test `jobs.rs` (progress avvelenato, input failure Full); nativo: `full-run-invalid` mostra «Violazioni rilevate» con report |
| Run/report ID: niente eventi tardivi, un solo run attivo | `cancel_of_a_newer_run_keeps_the_previous_report_and_publishes_nothing`, `superseded_closed_and_failed_full_runs_release_every_spool`, secondo `start` → `InvalidState` |
| Configurazione: selezione esplicita, file per tracciato, parametri tipizzati, budget | nativo Linux 43 scenari e Windows 17 (selezione, file parziale/intero, annullo dialogo, formati, budget avviso/procedi/annulla/Esc/invalido, Full con parametri bloccati e piano rifiutato) |
| Worker: avvio, monitor, interruzione, chiusura durante il run | nativo: run attivo, monitor Full PASS1, annullamento LocalOnly e Full; **chiusura della finestra durante un run da 245 MB**: exit 0, nessuno spool residuo (Linux 0,07 s, Windows 0,16 s) |
| Keyboard/focus/a11y, responsive senza overflow | percorsi solo da tastiera su Linux (skip link, Tab/frecce, dialoghi) e Windows (SendKeys); 640×480, 1024×768, 1366×768, work area; tema chiaro/scuro; scala 2× da P5-01 |
| Informazioni: versioni, provenienza, licenze | `keyboard-informazioni-*` (Linux), `informazioni-report-workarea` (Windows) |

Sorting interattivo, installazione di bundle, risultati/dettaglio/export Full
(P6-10) restano fuori da questo gate.

## Cosa cambia

Rust (adapter desktop, scope esteso):
- `src-tauri/src/main.rs`: su Windows la finestra principale è creata
  nascosta (`.visible(false)`), adattata alla work area da `fit_window` e poi
  mostrata (`built.show()?`). `window_fit.rs` invariato.

Harness (solo test):
- `tests/windows_capture.ps1`: parametro `-Flows <fixtures>` con 17 scenari
  dei flussi P5-02..P5-10 su WebView2 (dialogo nativo `IFileOpenDialog`,
  bundle Full esterno via `LINTRA_SCHEMA_DIR`, `resource-budget.json` in
  `%APPDATA%\it.lintra.desktop`, TEMP privato per ogni avvio) e
  `Invoke-CloseDuringRun`. Correzioni nate dalla prova: righe di scenario
  con le sequenze `+` tra parentesi (in PowerShell la virgola lega più del
  `+`), controllo che rifiuta righe malformate, baseline non toccate per il
  tema scuro e per 640×480, nuova lettura dell'handle quando la finestra
  «Lintra» è visibile.
- `tests/native_capture.py`: `close_during_run` (Hyprland, 1366×768).

## Difetto trovato e corretto: finestra fuori dalla work area su Windows

Nella VM la finestra si apriva a volte a `[x,1 → x+1116,800]`, alta 799 px su
una work area di 752: dimensione configurata, non adattata. Visto in 2 avvii
su 32 nei giri completi dell'harness, poi riprodotto in modo isolato con un
binario strumentato (solo nella VM, non committato; sorgenti in
`lintra-review-handoffs/T-MIG-P5-08-evidence/window-fit-diag/`):

| Giro (avvii) | Finestra fuori work area |
| --- | --- |
| binario base, solo log del fit (32) | 5 |
| con polling della geometria ogni 20 ms (24) | 22 |
| creata nascosta, fit, poi show (32) | 0 |

Il log mostra che il fit riesce sempre (subito dopo: `y=0`, 752 px), poi in
parte degli avvii la geometria torna a quella precedente al fit senza eventi
Moved/Resized osservabili, mentre parte il ciclo degli eventi. Ipotesi, non
dimostrata nel sorgente Tauri/tao: il primo show applica il placement
registrato alla creazione. Con la finestra nascosta il fit avviene prima di
qualunque show e il primo frame visibile ha già la dimensione corretta.
Sul binario corretto l'harness completo ha 17 avvii su 17 dentro la work area.

## Cosa NON cambia

Core, CLI, servizio, IPC v1, DTO, frontend React, lockfile, dipendenze,
workflow. Linux: il fit non esiste (il compositor possiede il placement) e la
finestra resta visibile alla creazione.

## Test

Linux (CachyOS/Hyprland), binario `18bd5ab7b98589dc228aa7add4742a27a180327ae562151861ae83b30eb49e8a`:

| Verifica | Esito |
| --- | --- |
| `npm run lint` / `npm test` / `npm run build` | PASS / 124 PASS / PASS |
| desktop `cargo fmt … -- --check` | PASS / 0 |
| desktop `cargo test … --locked --release --target x86_64-unknown-linux-gnu --features tauri/custom-protocol -j 2` | PASS / 0: 83, 0 fail, 1 ignored |
| desktop `cargo clippy … --release --all-targets … -- -D warnings` | PASS / 0 |
| desktop `cargo build … --release …` + `native_smoke.py` | PASS / 0 (`NATIVE_DESKTOP_SMOKE_PASS`), su Hyprland: `xvfb-run` non installato |
| `python3 tests/native_capture.py <bin> <out>` | PASS 43/43 + chiusura durante il run, rc 0 ([receipt](T-MIG-P5-08-native/receipt.json), SHA-256 `3528e2ff…51cb3c0`) |
| workspace `ci.yml` (fmt; clippy e test default e `--all-features`; xtask fmt/clippy/test `--include-ignored` con `LINTRA_TEST_BIN`), target `x86_64-unknown-linux-gnu` | PASS / 0 ×8: test 1116 pass, 0 fail, 9 ignored (130 suite) sia default sia all-features; xtask 18/18. Workspace non toccato dal diff; log in `lintra-review-handoffs/T-MIG-P5-08-evidence/workspace/` |
| `npm ci` | NOT_RUN: engine pin node 26.13.0, locale 26.13.1 |

Windows 11 VM (`Windows11-Lintra-Test`, 1280×800, DPI 96, `lintra-agent`
non elevato, sessione console 1), sorgenti con hash verificati sul worktree:

| Verifica | Esito |
| --- | --- |
| MSVC `cargo fmt` / `test --release` / `clippy -D warnings` / `build --release` (comandi di `desktop.yml`) | PASS / 0 ×4: 82 test, 0 fail; binario `8c9664a5e2b529c02e11b11025b2f6a65f440a805bc28dd35ee86e4c72522b32` |
| `windows_capture.ps1 <exe> <out> -Flows <fixtures>` | PASS 17/17 + chiusura durante il run ([receipt](T-MIG-P5-08-windows/receipt.json), SHA-256 `edcc0648…d242b71c`) |
| `native_smoke.py` su Windows | NOT_RUN: nella VM c'è solo l'alias Store di Python; resta alla CI remota |
| scala 200% Windows con la correzione | NOT_RUN: richiede di cambiare la risoluzione del firmware OVMF (procedura P5-01) |

Tentativi Windows precedenti, tutti per cause dell'harness tranne il difetto
corretto sopra (receipt in `lintra-review-handoffs/T-MIG-P5-08-evidence/windows-attempts/`):
run 1 TEMP condiviso bloccato da WebView2; run 2 precedenza della virgola;
run 3 e 4 baseline mancanti più il difetto della finestra; run 5, sul binario
corretto, handle della finestra letto troppo presto. Un primo giro Linux si è
fermato al dialogo GTK per un cambio di focus sulla sessione dell'autore;
rieseguito senza modifiche.

La VM è stata riportata allo snapshot `LINTRA-P5-08-PRETEST-20261123` e
lasciata spenta: nessuna task pianificata, file di prova o binario residuo.

CI remota: NOT_RUN, dopo la review Codex come deciso dall'autore.

## Rischi residui

- La causa del ripristino della geometria su Windows è un'ipotesi ben
  sostenuta dai log, non verificata nel sorgente; la correzione è provata in
  64 avvii (32 isolati e 32 nell'harness, compresi i 17 dell'ultimo giro).
- Scala 200% su Windows non riprovata dopo la correzione.
- La cattura del monitor Full su Windows mostra «Analisi in corso» e
  «Interrompi analisi»; il dettaglio per tracciato è sotto la piega. Il
  dettaglio Full resta provato su Linux (`full-monitor-pass1`,
  `full-monitor-later`).
- G-MIG-5 resta aperto fino a review Codex, approvazione UX dell'autore e CI
  remota verde sullo SHA.
