# T-MIG-P5-08 — Gate parità configurazione/worker/UX

Data 24 novembre 2026. Stato: **IN_REVIEW** (non DONE), rework dopo la
review Codex round 1 (CHANGES, P508-01..03). Galleria UX approvata dall'autore
il 24 novembre. Mancano l'approvazione Codex e la CI remota sullo SHA
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

Approvazione UX (24 novembre 2026): l'autore approva la galleria delle
catture (`lintra-review-handoffs/T-MIG-P5-08-gallery.html`) per questo gate.
Direttiva contestuale: la UI di Lintra dovrà seguire fedelmente i riferimenti
in `/home/alberto/Desktop/LINTRA/lintra_ui_references/` (UI SaaS moderna e
responsive); l'approvazione riguarda la UI funzionale attuale, il redesign
spetta alle task UI successive.

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
| Worker: avvio, monitor, interruzione, chiusura durante il run | nativo: run attivo, monitor Full PASS1, annullamento LocalOnly e Full; **chiusura della finestra durante un run da 245 MB** con report ancora in scrittura (`errors.jsonl.part`, nessun `errors.jsonl`) verificato subito prima del close: exit 0, nessuno spool residuo (Linux 0,02 s, Windows 200% 0,08 s) |
| Keyboard/focus/a11y, responsive senza overflow | percorsi solo da tastiera su Linux (skip link, Tab/frecce, dialoghi) e Windows (SendKeys); 640×480, 1024×768, 1366×768, work area; tema chiaro/scuro; Windows 200% sul binario finale (form con scroll e focus, run completato) |
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
| scala 200% Windows con la correzione | vedi «Rework round 1» |

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

## Rework round 1 (review Codex `T-MIG-P5-08.review-1.md`)

Tutti e tre i rilievi sono fondati.

**P508-01 — budget dell'account.** `windows_capture.ps1` salva il
`resource-budget.json` preesistente al primo intervento (dentro
`Set-RunEnvironment`, quindi solo dopo il preflight della sessione console) e
nel `finally` lo ripristina byte per byte; se il file non c'era lo rimuove, e
rimuove la directory solo se l'ha creata ed è vuota. Nessuna modifica se il
preflight rifiuta. Prova con sentinella scritta da `lintra-agent`
(SHA-256 `3408ddc2…9d4091e`, script in `T-MIG-P5-08-evidence/rework-1/`):
- avvio rifiutato via SSH (sessione 0): rc 1, sentinella identica. In
  sessione 0 l'harness si ferma già alla richiesta DPI-aware, prima del
  preflight della console; il ramo del preflight stesso non ha un'esecuzione
  dedicata (nel codice non tocca il budget prima del salvataggio);
- tre esecuzioni interattive complete con `-Flows` (budget scritto e rimosso
  dagli scenari): sentinella identica dopo ciascuna.

**P508-02 — oracolo della chiusura.** Uno spool conta come run attivo solo se
contiene `errors.jsonl.part` e non `errors.jsonl`: lo scrittore atomico
(`src/report/atomic.rs`) rinomina `.part` solo alla pubblicazione, mentre un
report completato conserva lo spool con `errors.jsonl`. Il controllo è
l'ultimo passo prima del close, dopo la cattura, e fa fallire la prova se il
run è già terminato. Windows salva ora anche la cattura dello stato prima del
close (`close-during-run.png`, hash nel receipt). Il probe di Codex
(`/tmp/lintra-p508-close-probe.py`) sul nuovo `native_capture.py` è rifiutato
(AssertionError, rc 1); la variante con il solo `.part` è accettata (rc 0).
Prove native ripetute: Linux mirata (`T-MIG-P5-08-native-close/`, run attivo
a 10.657.792 record, exit 0 in 0,02 s) e Windows 200% (sotto).

**P508-03 — Windows 200%.** VM a 1920×1080 (risoluzione OVMF impostata
dall'autore, backup NVRAM a sua cura), scaling 200% per `lintra-agent`
(sessione console 2, non elevato). Sorgenti con hash verificati; MSVC
`fmt`/`test --release` (82)/`clippy -D warnings`/`build --release`: PASS / 0,
binario finale `84b0210ecc1151aeea9340680f267c2ab03fc3e6fdc0010e51fbc79d0b0476e0`
(stesso `main.rs`, MSVC non bit-riproducibile). `windows_capture.ps1 <exe>
<out> -Scale2 -Flows <fixtures>`: **PASS 9/9 + chiusura durante il run**
([receipt](T-MIG-P5-08-scale2-windows/receipt.json), SHA-256
`112281e4…89710353`). In tutti gli avvii DPI 192, finestra predefinita
esattamente sulla work area `[0,0 → 1920,984]` prima di ogni resize
dell'harness, client 1894×912; sette scenari 2× di P5-01, form di analisi da
tastiera con scroll e focus sul pulsante file, run completato, chiusura con
run attivo (exit 0 in 0,08 s).

Harness, correzioni nate da questi giri: in due giri precedenti uno scenario
per giro falliva con «Unexpected window» (receipt in
`T-MIG-P5-08-evidence/rework-1/scale2-attempts/`). L'harness ora cerca la
finestra visibile intitolata «Lintra» enumerando le finestre del processo
(`FindVisible`), invece di `Process.MainWindowHandle`, e la chiude con
`WM_CLOSE` diretto; in caso di errore elenca tutte le finestre del processo.
Il giro finale ha trovato la finestra in 10 avvii su 10. La causa indicata
(scelta della finestra principale da parte di .NET) è coerente ma non
dimostrata.

Non rieseguiti: i 17 scenari Windows 1× (receipt su harness precedente,
`6ee2910e…`, binario `8c9664a5…`): 1× e 200% non convivono nella stessa
configurazione della VM e Codex chiedeva la sola prova mirata di chiusura; il
nuovo codice dell'harness è esercitato dal giro 200%. I 43 scenari Linux non
sono cambiati; rieseguita solo la chiusura.

VM: riportata allo snapshot `LINTRA-P5-08-PRETEST-20261123` e spenta; il
ripristino della NVRAM dal backup resta all'autore.

## Rischi residui

- La causa del ripristino della geometria su Windows è un'ipotesi ben
  sostenuta dai log, non verificata nel sorgente; la correzione è provata in
  74 avvii a 96 DPI e 192 DPI (32 isolati, 32 nell'harness 1×, 10 a 200%).
- La cattura del monitor Full su Windows mostra «Analisi in corso» e
  «Interrompi analisi»; il dettaglio per tracciato è sotto la piega. Il
  dettaglio Full resta provato su Linux (`full-monitor-pass1`,
  `full-monitor-later`).
- G-MIG-5 resta aperto fino all'approvazione Codex e alla CI remota verde
  sullo SHA.
