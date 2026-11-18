# T-MIG-P4-07 — Riutilizzo indice JSONL, senza bulk transfer

18 novembre 2026. Autore Codex, Lane 3. Baseline:
`45abfdcbcca4d2f099469295ba578e615fd3b933`, branch
`agent/codex/T-MIG-P4-07`. Prima consegna: `565623b`, review Claude **CHANGES**.
Stato finale: **DONE**; commit pubblicati `a68d17d` e `5e9cacd` (vedi
"Chiusura"). Nessun gate aggregato o compatibilità Windows attestati.

## Perimetro e comportamento

Prerequisito P4-06: dossier e stato unificato registrano conclusione sul commit
`5e3acdb`, con CI nativa Linux/Windows. La baseline include quel commit.
La specifica autorizzata è
`docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P4-07.md`.

- `src/report/index.rs` estrae l'indice Slint nella libreria Rust. La scansione
  verifica LF, diagnostica, ambito, conteggio e istogramma, conservando soltanto
  gli offset delle corrispondenze. Le diagnostiche invalide fuori filtro fanno fallire la scansione.
- Il decoder è quello già impiegato da XLSX e replay CSV/JSONL. Nessuna modifica
  ai nomi wire, ai controlli di dominio o alla serializzazione canonica.
- La lettura casuale conserva al massimo 128 `ValidationError` in una FIFO,
  identificati dalla posizione nell'indice. Il buffer contiene una sola riga.
  I contatori di cache miss escludono la scansione iniziale di verifica.
- Slint conserva le sue API tramite un adapter che trattiene il proprietario
  del report. Il modello proietta soltanto le righe richieste e usa la cache
  della libreria; non possiede una seconda cache di righe presentate.
- Il worker Tauri prepara l'indice dopo la pubblicazione privata dello spool,
  prima di esporre il report terminale. Un errore di scansione produce errore
  tecnico e non sostituisce il precedente report. Il reader viene distrutto
  prima del proprietario della directory, anche quando il report è condiviso.

## Contratti preservati e limiti

Nessun `Vec<ValidationError>` globale, conversione preventiva di tutte le righe
UI o trasferimento dell'indice/delle diagnostiche in JavaScript. Stato IPC,
report availability e identità delle run mantengono le forme precedenti.
Query/paging IPC, DTO di dettaglio, export e Full adapter appartengono alle
successive task P4-08..11 e non sono implementati qui.

L'indice resta O(corrispondenze), 8 byte di payload per offset: 8.000.000 byte
per 1.000.000 righe. Il precedente limite `i32::MAX` è conservato per Slint.
Questo valore misura il payload degli offset, non capacità allocata o RSS.
La cache è limitata per numero di righe; dimensioni di diagnostiche e buffer
dipendono dalla singola riga canonica. Non viene dichiarato un limite RSS.
La scansione verifica una diagnostica alla volta e scarta il valore dopo
aver aggiornato offset/contatori; la cache nasce vuota. La libreria supporta
cancellazione cooperativa tra righe tramite callback. Tauri completa l'indice
del report pubblicato senza osservare il token della run, il cui esito è già
fissato dal checkpoint finale del servizio. Nessun thread o sincronizzazione nuovo.

Motore, DSL, storage, writer JSONL, golden e test protetti restano invariati.
Solo fixture sintetiche. Nessun cambiamento di dipendenze o configurazioni.

## Verifiche

Prove della sessione precedente su Linux: 121 test superati e 2 esclusioni preesistenti,
oltre ai 12 test di baseline. Tutti i comandi di verifica terminano con exit 0.
Sorgenti del worktree sopra la baseline indicata.
Le prove native Tauri usano `--release --features tauri/custom-protocol -j 2`.
Le dipendenze/cache già presenti sono state copiate nel worktree, senza modificare
le copie originali né installare nuovi pacchetti.

| Comando | Esito / exit code |
|---|---|
| Baseline `cargo test --locked -p lintra-ui --test results` | 6 PASS / 0; prima di collegare il nuovo modulo |
| Baseline `cargo test --manifest-path apps/lintra-desktop/src-tauri/Cargo.toml --locked --offline --release --features tauri/custom-protocol -j 2 jobs::tests -- --nocapture` | 6 PASS / 0 |
| `cargo test --locked --offline -p lintra-ui --test results --test worker --test results_window -j 2` | 14 PASS / 0; results 8, results_window 1, worker 5; incluso 300k/1M |
| `cargo test --locked --offline --test golden --test atomic_report --test xlsx --test xlsx_writer --test cli_xlsx --test report_export_policy -j 2` | 46 PASS, 1 ignored / 0 |
| `cargo test --manifest-path apps/lintra-desktop/src-tauri/Cargo.toml --locked --offline --release --features tauri/custom-protocol -j 2` | 33 PASS, 1 ignored / 0 |
| `cargo fmt --all -- --check` e fmt del manifest Tauri | PASS / 0 |
| `cargo clippy --locked --offline -p lintra --lib -p lintra-ui --all-targets -j 2 -- -D warnings` | PASS / 0 |
| `cargo clippy --manifest-path apps/lintra-desktop/src-tauri/Cargo.toml --locked --offline --release --all-targets --features tauri/custom-protocol -j 2 -- -D warnings` | PASS / 0 |
| `npm --prefix apps/lintra-desktop test` | 28 PASS / 0 |
| `npm --prefix apps/lintra-desktop run build` | TypeScript strict e Vite PASS / 0 |
| `git diff --check`, confronto modifiche con write_set della card | PASS / 0; 8 file autorizzati, nessun test protetto modificato |

Ignored preesistenti: probe manuale di memoria XLSX e picker GTK che richiede
un display interattivo. I messaggi di panic e fallimento ZIP nei test di fault
injection sono attesi; i test terminano con successo.

Test aggiunti: accesso diretto alla libreria con FIFO di 129 righe, cache hit,
espulsione e riaccesso, posizione inesistente senza lettura; cancellazione dopo
una riga; diagnostica invalida esclusa dal filtro che invalida comunque l'indice.
Il nuovo test Tauri prova output valido senza errori e invalido, cache
inizialmente vuota, lettura su richiesta e lifetime fino all'ultimo Arc.
Le regressioni esistenti coprono contesto raw, tutti i filtri, corruzione,
conteggi/istogramma/ambito, file troncato, golden byte-identici e report precedente
preservato dopo una nuova analisi fallita.

### Gate completo della prima consegna (`565623b`)

`python3 .ai_runtime/bin/aiproto.py gate`: **GATE VERDE, exit 0**.
Tutti i nove comandi di `protocol.toml` sono stati eseguiti senza modifiche.
Il log completo è `.ai_runtime/tasks/T-MIG-P4-07/gate-1795009436.log`, SHA-256
`5da53bbc03fd09ce49ea1551eac4e91f5c94d9c06cc4747a60a86dfea19c04cf`.

| Controllo configurato | Esito / exit code |
|---|---|
| `cargo fmt --all -- --check` | PASS / 0 |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | PASS / 0 |
| `cargo test --workspace --locked` | 1.112 PASS, 9 ignored preesistenti / 0 |
| `npm --prefix apps/lintra-desktop run lint` | PASS / 0 |
| `npm --prefix apps/lintra-desktop test` | 28 PASS / 0 |
| `npm --prefix apps/lintra-desktop run build` | TypeScript strict e Vite PASS / 0 |
| `cargo fmt --manifest-path apps/lintra-desktop/src-tauri/Cargo.toml -- --check` | PASS / 0 |
| `cargo test --manifest-path apps/lintra-desktop/src-tauri/Cargo.toml --locked --release --features tauri/custom-protocol -j 2` | 33 PASS, 1 ignored GTK preesistente / 0 |
| `cargo clippy --manifest-path apps/lintra-desktop/src-tauri/Cargo.toml --locked --release --all-targets --features tauri/custom-protocol -j 2 -- -D warnings` | PASS / 0 |

Totale del gate: **1.173 PASS, zero fallimenti, 10 ignored preesistenti**.
Le nove esclusioni workspace sono tre helper di subprocess invocati dai test
parent, due fixture di scambio fra runner nativi CI, la misura CAD83, il test
manuale sul workbook normativo dell'autore, il probe memoria XLSX e il test
di directory sync che richiede un account Linux non privilegiato. Il picker
GTK nativo richiede un display interattivo. Nessuna esclusione è stata aggiunta.

Le prove hanno eseguito il delta non ancora committato sulla baseline indicata,
con i sei digest codice/test della prima consegna riportati sotto; solo il dossier è aggiornato dopo
il gate. Lo SHA di consegna è registrato nell'handoff runtime dopo il commit.
I risultati delle prove precedenti e del gate si sovrappongono e non si sommano.
Nessuna CI, prova Windows o VM eseguita per questo delta; G-MIG-4 resta aperto.

## Acceptance della task

| Vincolo | Evidenza locale |
|---|---|
| Nessun `Vec<ValidationError>` globale | L'indice conserva `Vec<u64>`, un buffer e la cache FIFO. I test 300k/1M verificano `count * 8` byte di offset. |
| Nessuna conversione preventiva di tutte le righe UI/DTO | La scansione valida una diagnostica alla volta e non popola la cache. Le letture richieste sono contate separatamente. Lo scrolling 300k/1M richiede meno di 2.000 decodifiche, verificato dal test esistente. Nessuna modifica ai DTO/IPC. |
| Cache bounded | Il test di 129 righe verifica hit, espulsione FIFO e riaccesso; le prove 300k/1M controllano il massimo di 128 righe. Slint usa la stessa cache della libreria. |

## Consegna e prerequisiti residui

Nella sessione precedente commit e handoff erano bloccati dal filesystem in
sola lettura sui metadati Git e sul runtime esterno. La ripresa del 18 novembre
ha confermato il vincolo nel sandbox ordinario e ottenuto accesso tramite
l'escalation nativa consentita: staging dei sette file di codice/test concluso
con exit 0, gate completo PASS e handoff scritto nel runtime configurato.
Nessun hook, configurazione
o permesso modificato; nessun runtime alternativo creato.

I sei digest della prima consegna sotto coincidono con quelli delle prove precedenti. Manifest e
lockfile Rust/frontend sono identici alla baseline; il delta applicativo è
invariato. I log esistenti confermano 121 PASS e 2 ignored. Queste evidenze
sono riutilizzate; il nuovo lint TypeScript strict è passato con exit 0.
Review Claude separata, checkpoint umano Lane 3, gate di integrazione previsto
da AIProto e approvazione/merge dell'utente restano necessari.
G-MIG-4 aggregato resta aperto; P4-08 dipende dall'integrazione
approvata di questa task.

## Identificazione dei sorgenti della prima consegna (`565623b`)

SHA-256 del codice e dei test del delta; il decoder Slint duplicato
`ui/src/results/diagnostic.rs` è eliminato. Gli altri file nel write_set
(`src/report/xlsx.rs`, `ui/src/results/controller.rs`) non richiedono modifiche.

| File | SHA-256 |
|---|---|
| `src/lib.rs` | `b1cabe9cdc470574474bb034b1b5c25fe29b77d30b594daafdad0c65d1fe237c` |
| `src/report/mod.rs` | `361bcbf25d7b3b1b6bbd1dc32901d8953d398d7c2383c814a4b9609ff1d07abd` |
| `src/report/index.rs` | `211c48d114da812d63b3c52fb35593d129898009741cee8d122fac5b5683b315` |
| `ui/src/results/mod.rs` | `be4e1a1e54a1eaeea8788f9f7ff6d8471570f0091405727c6f43a0e1ff49c55d` |
| `apps/lintra-desktop/src-tauri/src/jobs.rs` | `116d51feb45096b290f855dbe244c94573e7a76f78ccfbd7891c0666c0be184a` |
| `ui/tests/results.rs` | `34f5ae6179bb81e59026adaab1b0f2a58675cacb34dae084a4fa62a89cd605d7` |

Log locali conservati in `/tmp`: `T-MIG-P4-07-results.log`,
`T-MIG-P4-07-reports.log`, `T-MIG-P4-07-native.log`,
`T-MIG-P4-07-native-baseline.log`, `T-MIG-P4-07-clippy.log`,
`T-MIG-P4-07-native-clippy.log`, `T-MIG-P4-07-node.log` e
`T-MIG-P4-07-frontend-build.log`. Questi log non sono artefatti CI.

## Correzione della review Claude

Riferimento: `review-1795010898045000002.log`, verdetto CHANGES su `565623b`.
Il servizio fissa l'esito al checkpoint finale: se il token viene cancellato
dopo quel punto, la preparazione dell'indice non deve riclassificare la run
completata come errore tecnico né eliminare il report pubblicato.
`execute` passa quindi `|| false` alla scansione dopo `AtomicOutputOutcome::Published`.
Gli errori reali di I/O o integrità restano errori tecnici. La cancellazione
prima del checkpoint mantiene esito Cancelled e report non pubblicato.

Il test `cancellation_after_final_checkpoint_preserves_completed_report`
cancella il token su `RunCompleted`, dopo il checkpoint e prima della scansione,
senza temporizzazioni o thread aggiuntivi. Verifica esito CompletedInvalid,
completezza Complete, presenza dello spool e lettura della diagnostica indicizzata.
Prima della correzione il test fallisce con `TechnicalFailure`, exit 101:
`/tmp/T-MIG-P4-07-late-cancel-before.log`.

Il delta della correzione interessa soltanto `jobs.rs` e questo dossier.
SHA-256 corrente di `jobs.rs`:
`c615de17163d0a4675189ec76555938fb04c598323d65df2d5a450e9da09cd6f`.
Indice, cache, decoder, IPC, manifest e test protetti sono invariati.
Verifiche successive alla correzione: **PASS**, exit 0. Il gate completo
`python3 .ai_runtime/bin/aiproto.py gate` ha eseguito nuovamente tutti i nove
comandi configurati, senza modifiche: 1.112 test workspace, 28 test Node e
34 test nativi Tauri superati, zero fallimenti, 10 ignored preesistenti.
Totale: **1.174 PASS**. Log:
`.ai_runtime/tasks/T-MIG-P4-07/gate-1795012494.log`, SHA-256
`0df0836e4e86586ad581e706ea2c47ed52f2feae61f94765a315c48ad2a0b32a`.
Il ledger runtime registra `ok: true` il 18 novembre alle 15:44:49 CET.
Il digest di `jobs.rs` sopra coincide con il sorgente corrente verificato.
Le altre cinque impronte codice/test della prima consegna sono invariate;
solo questo dossier viene aggiornato dopo il gate.

La prova mirata `jobs::tests` passa con 8 test, zero esclusioni e exit 0;
log `/tmp/T-MIG-P4-07-late-cancel-after.log`. Il test di regressione è presente
anche nel gate nativo completo. Nuova review indipendente, checkpoint umano
Lane 3 e gate sull'integrazione restano necessari. Nessuna CI o prova Windows
aggiunta; G-MIG-4 e G-REL restano aperti. P4-08 non viene iniziata.

## Chiusura (18 novembre 2026)

- Review Claude round 2 su `3b15f0a`: **approve** (evidenza runtime
  `review-1795013666944511928.log`, SHA-256
  `f59c8a0367a47c2682dfe1e7580e04e9e9c1862c04c8054b56d5906ba8b39575`).
- Checkpoint umano Lane 3 confermato su `3b15f0a` (15:59 CET).
- Gate di merge locale sul risultato dell'integrazione: 9 comandi configurati,
  1.146 test Rust e 28 Node superati, 0 falliti, 10 ignored preesistenti.
  Log `gate-merge-1795013997.log`, SHA-256
  `a54a1324cbd9ef901ae1ff245d1edeca86911f82adc57c30c1c7b90f393b22ad`.
- Merge locale in `integration`: `f358db6`. Nessun file di `tests/acceptance` modificato.
- Pubblicazione: `integration` ricostruita linearmente sul remoto `9171501`,
  escludendo i commit di governance del workflow AIProto dismesso. `a68d17d` e
  `5e9cacd` riapplicano `565623b` e `3b15f0a`; `git diff f358db6` sul codice è
  vuoto, cambiano solo i file di istruzioni degli agenti. I log runtime citati
  (`.ai_runtime/...`, gate di merge) sono conservati nell'archivio locale
  del runtime, fuori dal repository.
- **Non eseguito:** CI remota e prove Windows su `5e9cacd`. La qualifica
  nativa Linux/Windows è rinviata al checkpoint del blocco P4, come per
  P4-01..03; G-MIG-4 e G-REL restano aperti.
- Prossime task READY: T-MIG-P4-08 (query/paging in Rust) e T-MIG-P4-11.
