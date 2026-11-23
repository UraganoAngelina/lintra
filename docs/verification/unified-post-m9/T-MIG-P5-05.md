# T-MIG-P5-05 — Run screen e progress throttled (con D09 completo)

Data 23 novembre 2026. Stato: **IN REVIEW**, non DONE prima della review Codex
e dell'integrazione. Prerequisiti T-MIG-P5-03 e T-MIG-P5-04 DONE (`integration`
`81b4006`). Branch `work/T-MIG-P5-05` da `81b4006`, worktree
`/home/alberto/Desktop/LINTRA/lintra-p5-05`.

## Scope esteso autorizzato

Il prompt chiede, oltre a run screen e progress, il dialogo D09 («budget soft:
Avviso e Procedi comunque; hard non aggirabile; non troncare input»). Il
backend Tauri e IPC v1 non avevano alcuna operazione di budget e nessun
documento fissava da dove la GUI leggesse il budget. L'autore ha deciso
(23/11) l'opzione «file versionato in posizione fissa» e ha autorizzato
esplicitamente l'estensione a Rust/Tauri, contratto IPC v1 (additivo), React,
test e documentazione, chiedendo D09 completo in questa task. Decisioni
applicate:

1. riuso di `ResourceBudget`/`resource_preflight`/`PreparedResourcePreflight`
   di U-FIX-03, nessun motore parallelo;
2. file `resource-budget.json` nella directory di configurazione per-utente
   risolta da Tauri (`app_config_dir`), nessun path OS scritto nel codice;
3. stesso formato e stesso parser stretto della CLI (`ResourceBudget::parse`,
   aggiunto al core e usato anche da `resource_preflight`);
4. nessuna soglia inventata; 5. file assente = «non configurato», mostrato;
6. file non valido = stato `invalid`, avvio rifiutato; 7. estensione IPC v1
   additiva documentata; 8. preflight e decisione in Rust con handle opaco
   monouso; 9. dialogo React completo; 10. configurazione documentata, stato
   sempre visibile, nessun picker aggiuntivo.

Fonti: prompt atomico; MIGRATION_PLAN v1.1 riga P5-05; Master Plan §1 (D09) e
§4; `docs/RESOURCE_BUDGETS.md`; dossier U-FIX-03; IPC v1 `CONTRACT.md`,
`LIFECYCLE.md`, `PROGRESS.md`; `MIGRATION_TAURI_BRIEF.md` (stati e parole,
testo del budget soft, layout affiancato); codice Slint `ui/ui/analysis.slint`
(`enabled: !root.busy`); preflight Slint→Tauri (righe 232, 281–283).

## Cosa cambia

Core (`lintra`, additivo):
- `ResourceBudget::parse` e `check` (stesse regole già in `resource_preflight`).
- `AnalysisBundle::resource_references()`: la sorgente reference (directory
  esterna o bundle incorporato) che il preparer del bundle legge davvero.

Backend Tauri:
- `budget.rs`: lettura limitata (64 KiB) del file, stati
  unconfigured/invalid/configured, gate del run e DTO.
- `jobs.rs`: `start_budgeted` e `soft_budget_gate` sul thread del worker
  prima di aprire output; attesa della decisione con controllo del token ogni
  50 ms; Annulla = cancel del run (`not_started`, `cancel_requested`);
  `verify_unchanged` subito prima della validazione; operazioni
  `resource_budget_state`, `validation_budget`, `validation_budget_decide`;
  riga aggregata su stderr come la CLI.
- `configuration.rs`: bundle condiviso via `Arc`, directory di budget, lettura
  del file a ogni start; `state.rs`/`main.rs`: routing e directory Tauri.

Frontend:
- `ipc.ts`: tre operazioni tipizzate con validazione esatta (flag coerenti con
  i contatori, handle solo in attesa di decisione, soglie u64 positive).
- `run.ts`: stato del run, polling ogni 250 ms con una richiesta per volta
  (stato prima, poi progress e, finché aperto, gate), un solo campione tenuto,
  sequence monotona, terminale definitivo e acknowledged, conteggio finale
  riletto al terminale (best effort), parole del brief.
- `RunPanel.tsx`: «Avvia analisi», stato del budget, avanzamento
  indeterminato (nessuna percentuale o ETA), contatori Rust, formati congelati
  sul run (P5-04), esito; dialogo nativo `<dialog>` modale con i byte misurati
  e le soglie, Annulla con focus iniziale, Esc = Annulla, Procedi comunque;
  traccia della scelta.
- `AnalysisForm.tsx`: form bloccato durante il run (selezioni, file, formati),
  come `enabled: !root.busy` in Slint. `AppShell.tsx`: «Analisi in corso».
  `main.tsx`: layout affiancato configurazione/esecuzione ≥1280 px (brief),
  colonna sotto; disponibilità del report da `report_available` Rust.
- `style.css`: solo token esistenti.

Documentazione: `docs/migration/ipc-v1/BUDGET.md` (nuovo contratto),
`CONTRACT.md` e `LIFECYCLE.md` (rimandi), `docs/RESOURCE_BUDGETS.md` (sezione
desktop con posizione del file per OS), README desktop.

## Cosa NON cambia

Operazioni, DTO e stati IPC esistenti; `schemaVersion` 1; RunSnapshot e
outcome; CLI e suo comportamento (`resource_budget.rs` della CLI invariato);
golden/JSONL; nessuna dipendenza nuova. Cancel come azione dell'operatore,
stato terminale con report precedente e risultati restano P5-06/P5-07; Full
P5-09.

## Acceptance

| Requisito | Prova |
| --- | --- |
| UI reattiva su un run sintetico di 60 s | `tests/native_run.py` sul binario reale: 250.000.000 record (1,75 GB), run di 82,5 s; 16 frame distinti in 16 campioni (la schermata si aggiorna); Shift+Tab a 20,5 s visibile entro 0,1 s; conteggio finale 250.000.000 |
| Memoria bounded | RSS del processo app 161.740–161.904 KiB (≈158 MiB) per tutto il run; renderer WebKit 167.248–201.528 KiB (≈163–197 MiB), crescita massima 34.280 KiB (≈33 MiB, limite del test 64 MiB), stabile dopo ~30 s; nessun campione/record accumulato in JS (un solo campione nello stato) |
| Messaggi coerenti con Rust | esiti mappati dal solo outcome Rust (valido, violazioni, incompleto, annullato, non iniziato, errore tecnico); contatori e fasi dal DTO progress; nessuna percentuale; test `run.test.mjs` |
| D09: avviso + Procedi comunque | nativo: soglia 65.536 byte, file 140.000 byte → dialogo con misure reali; Procedi → run completo («Nessuna violazione…», report disponibile); Rust: `over_budget_waits_for_the_operator_then_proceed_runs_the_whole_input_once` (tutti i record) |
| Annulla / Esc | nativo: «Analisi annullata», nessun report; Rust: `annulla_cancels_the_pending_run_before_any_output` |
| Hard non aggirabile | `hard_failures_are_technical_and_no_decision_can_bypass_them`; `verify_unchanged` prima della validazione |
| Non troncare input | Procedi esegue l'intero input (2/2 record nel test Rust; 250 M nativo senza budget) |
| Handle monouso, stale, duplicati, concorrenza | test Rust: handle falso/riuso/dopo cancel → `IPC_STALE_CONTEXT`; secondo start → `IPC_BUSY`; chiusura app durante l'attesa → join |
| Configurazione | test `budget.rs` (assente/valido/non valido/oversize/directory) e `configuration.rs` (file non valido blocca lo start, file corretto applicato senza riavvio, reference su bundle esterno); nativo: messaggio di file non valido e avvio disabilitato |

## Test

| Verifica | Esito |
| --- | --- |
| `cargo fmt --all -- --check` | PASS |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | PASS |
| `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` | PASS |
| `cargo test --workspace --locked` | PASS 1116, 0 fail, 9 ignored |
| `cargo test --workspace --all-features --locked` | PASS 1116, 0 fail, 9 ignored |
| `cargo test -p lintra --no-default-features --locked` | PASS 1071, 0 fail, 8 ignored |
| desktop `cargo fmt … -- --check` | PASS |
| desktop `cargo test … --release --target x86_64-unknown-linux-gnu --features tauri/custom-protocol` | PASS 80, 0 fail, 1 ignored (12 nuovi test D09) |
| desktop `cargo clippy … --release --all-targets … -- -D warnings` | PASS |
| `npm run lint` / `npm test` / `npm run build` | PASS / 86 test PASS / PASS |
| `npm run tauri -- build --no-bundle -- --locked` + `tests/native_smoke.py` | PASS, binario `6ea92f6d…` |
| `python3 tests/native_capture.py <bin> <out>` | PASS 26/26, tre esecuzioni consecutive rc 0 ([receipt](T-MIG-P5-05-native/receipt.json)) |
| `python3 tests/native_run.py <bin> <out> 250000000` | PASS ([run.json](T-MIG-P5-05-native/run/run.json)) |
| `npm ci` | NOT_RUN: engine pin node 26.13.0, locale 26.13.1 |
| CI remota | NOT_RUN (nessun push) |
| Windows (MSVC, WebView2, `%APPDATA%`) | NOT_RUN |

Note di sviluppo verificate: il primo client TS rifiutava le nuove operazioni
(allowlist di `encodeRequest`, aggiornata); Annulla senza cancel del token
avrebbe prodotto `not_started` senza `cancel_requested`, che il client
rifiuta (corretto: Annulla richiede il cancel); al terminale la UI mostrava
l'ultimo campione intermedio (3.630.080 su 4.000.000), corretto rileggendo il
progress al terminale; il conteggio esatto è verificato nel run nativo.

## Rischi residui

- Qualifica Windows assente: posizione `%APPDATA%\it.lintra.desktop`, dialogo
  modale in WebView2, build MSVC; da fare al checkpoint G-MIG-5.
- La scelta Procedi comunque è tracciata nel gate del run e su stderr (come la
  CLI); non è persistita nel report (il summary del core non ha un campo
  budget) né in un log desktop, che non esiste.
- Il preflight legge interamente i file (SHA, come la CLI): con budget
  configurato l'avvio di input grandi richiede una lettura in più, mostrata
  come «Controllo dei budget di risorse».
- `verify_unchanged` non è un confinamento ostile (D10): la tree locale deve
  restare controllata, come documentato.
- Le catture includono a sinistra la dock del compositore della sessione
  (fuori dall'app, non è un difetto del prodotto).
- CI remota NOT_RUN; G-MIG-5 aperto.

Prossima tecnicamente disponibile dopo l'integrazione: T-MIG-P5-06.
