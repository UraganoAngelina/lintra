# Lintra — preflight Slint → React + TypeScript + Tauri 2

Data: **2026-10-30**, Europe/Rome. Audit single-agent, senza migrazione.
Baseline Git: **`fe5ada864be3455479c2b46765c662875f008933`**, branch **`work/p1-foundations`**.
Oggetto osservato: HEAD **più working tree preesistente**, separati nelle evidenze. **Working tree variato anche esternamente all'audit durante la sessione**: impronte e limiti temporali in §15; i controlli conclusi non sono estesi alle revisioni successive.
Protocollo applicato: `LINTRA_PREFLIGHT_AGENT_PROMPT.md` fornito dall'autore; istruzioni operative del documento applicate entro la richiesta esplicita di solo audit.

## 1. Executive summary

**Readiness: `READY_WITH_BLOCKERS`.** Le evidenze bastano per preparare un Master Migration Plan condizionato a decisioni esplicite; non autorizzano implementazione, compatibilità Tauri, rilascio o uso di dati reali. Il report non è quel piano.

- **[CODE] Il core Rust è utilizzabile indipendentemente da Slint.** `lintra-ui` chiama direttamente la library; non lancia la CLI. Il package `lintra` contiene library e binario CLI, con moduli interni privati e re-export pubblici. Il grafo Cargo include però `clap` nel package condiviso: separazione dei moduli e separazione delle dipendenze di package non coincidono (`src/lib.rs:83`, `src/main.rs:1`, `Cargo.toml:46`, `ui/Cargo.toml:15`).
- **[CODE] La parità effettiva riguarda LocalOnly su un file/tracciato.** Configurazione esplicita, controllo iniziale, worker/cancellazione, risultati, domini paginati, export e Informazioni esistono. Full è nel servizio/CLI, non nel form Slint; storico e approvazioni non hanno schermate (`ui/src/analysis.rs::AnalysisSelection::prepare`, `src/application/service.rs::validate_full`, `ui/ui/main.slint::MainWindow`).
- **[CODE] Il percorso risultati è già scalabile per i diagnostici, ma non è un bridge pronto.** JSONL su disco, indice `Vec<u64>` O(errori), cache di 128 righe, filtri con scansione in worker; lettura della singola riga ancora sul thread UI e conversione in `ErrorRow` Slint. Nessun ordinamento interattivo. Trasferire l'intero indice o tutti gli errori in JavaScript sarebbe una regressione (`ui/src/results/mod.rs:113`, `ui/src/results/mod.rs:194`, `ui/src/results/controller.rs:210`).
- **[CODE]/[HISTORY] SQLite è implementato e ratificato**, con HMAC prima dello storage, DELETE/EXTRA, CAS e recovery esplicita. Il coordinatore di approvazione è separato dalla validazione. L'audit storico T-PERSIST-08 è lavoro locale non committato, non un prerequisito già qualificato. Privacy operativa, chiavi, cifratura/retention/backup prima dei dati reali restano aperti (`src/persistence/sqlite/mod.rs:112`, `src/application/session.rs:515`, `docs/verification/ADR-PERSIST-002-ratification.md:23`).
- **[TEST]/[HISTORY] Build check e Clippy workspace locali e 37 test mirati sono passati (§11).** Tre file WIP sono cambiati successivamente: il loro stato finale non è qualificato dai controlli precedenti. La CI registrata riguarda `aa7be4f`, antenato con sorgenti eseguibili invariati rispetto a HEAD, ma non il lavoro locale. La lettura remota è fallita: nessuna CI attuale certificata. Il dossier M8 descrive archivi Slint Linux/MSVC qualificati, non una release Tauri (`docs/verification/T-P16-08-native-ci.json:1`).

Top blocker: ratifica sostituzione N-24; nuova politica UI per WebView/runtime/offline e QG-5; contratto IPC con minimizzazione PHI e limiti di carico; ridefinizione QG-7; baseline stabile rispetto a T-PERSIST-08. Le voci sono dettagliate in §13, senza approvarle.

## 2. Ambito, metodologia e affidabilità delle evidenze

Solo letture, controlli sicuri e questo nuovo documento. Nessuna correzione dei problemi osservati, modifica a status, installazione, dipendenza, commit, push, tag, avvio CI o piano di migrazione da parte dell'audit. La ricognizione copre C1–C9 del protocollo; i controlli qualificano i file letti dal compilatore durante la loro esecuzione, non le modifiche esterne successive. Nessun subagente o processo shell lanciato in background.

Legenda vincolante:

| Etichetta | Significato e limite |
|---|---|
| `[DESIGN]` | Contratto documentale, non prova di implementazione |
| `[CODE]` | File/simbolo ispezionato nella fotografia locale |
| `[TEST]` | Comando realmente eseguito in questa sessione, con esito in §11 |
| `[CI]` | Pipeline letta direttamente; in questa sessione nessuna lettura remota riuscita |
| `[HISTORY]` | Registro, dossier o JSON CI conservato nel repository; non autenticato nuovamente contro GitHub |
| `[EXT]` | Documentazione ufficiale esterna consultata per Tauri **major 2**, non versione adottata |
| `[INFERENCE]` | Deduzione motivata, da non confondere con test |

Le tabelle di inventario implementativo sono `[CODE]` salvo diversa etichetta. La colonna test indica **test presenti**, non un esito eseguito, se non richiama un controllo V-* di §11. `NOT_VERIFIED` significa evidenza insufficiente; `NOT_RUN` significa comando non eseguito; `NOT_MEASURED` significa misura non prodotta qui. `DONE` e milestone sono riportati come dichiarati, senza promozioni autonome.

Fonti lette: AGENTS e master prompt; piano §4, §9–15, P15/P16 e persistenza; mappa dipendenze/indice/status e ultimo handoff; prompt P15/P16; ARCH §3/5/39A–D/44–45; DET §3/5/59–61/81–85/98–99/119/162/Addendum A/B/166–174; specifica persistenza e ratifiche ADR002/DEC18/release/artefatti/reference; sorgenti e test indicizzati in §15; DEVELOPMENT/CI/runbook; design corrente e reference. I dossier storici sono stati letti per scope/gate e confrontati con il codice, non trattati come nuova esecuzione.

**Assenze e organizzazione:** non esiste `SESSION_RECOVERY.md` alla root; esiste il template in `docs/governance/templates/SESSION_RECOVERY.md`. ARCH/DET/piano sono sotto `docs/`. Non esistono progetto Tauri, manifest Node o lockfile frontend nel prodotto osservato. Le vecchie cartelle `docs/ui/` e capture indicate dagli handoff non sono la fonte corrente: la riorganizzazione `ee33d64` le ha rimosse/spostate; fonti attuali in `docs/ui-design-documentation`, due PNG in `ui/tests/reference`. Prototipo escluso dal workspace in `spikes/ui-prototype`; mockup adiacente consultato come inventario storico, non come funzionalità di prodotto. Dettagli in §15.

Fonti normative locali consultate senza dati record: README/matrici di regole CAD/SIOC in `docs/normative/{cad,sioc}`, `docs/normative/DECISIONS.md`, `reference-datasets.md`, dossier G0/T-P0-08/09 e T-P14-01, mappa `docs/schema-authoring/FORMAT_COMPATIBILITY.md`, README dei profili SIOC .1/.2. I riferimenti root `SCHEMA_INPUTS/`, `CAD_REVISED_CANDIDATE/`, `CAD_NOTES_AUDIT.md`, `CAD_SAMPLE_ANALYSIS.md`, `CAD_FIXTURE_PLAN.md`, `SIOC_PERSISTENCE_AUDIT.md`, `SOURCE_CONFLICTS_AND_CHANGE_REQUEST.md` non esistono qui: rimandi allo starter kit storico, non file di prodotto omessi silenziosamente. Anche `docs/REFERENCE_DATA_SPEC.md` è assente; esistono onboarding locale e contratto membership/dossier. Non si accede ai campioni paziente del kit.

**Conflitto operativo risolto per scope:** AGENTS prescrive prosecuzione/status/commit nelle task di sviluppo (`AGENTS.md:13`, `MASTER_AGENT_PROMPT.md:48`, piano §14). La richiesta attuale impone esclusivamente il report e poi stop: nessun avanzamento di T-PERSIST-08 né aggiornamento dei registri. Non si tratta di una nuova ratifica architetturale per Tauri. Le skill del redesign UI non sono state applicate: qui non si ridisegna né modifica l'interfaccia.

Limiti: audit statico ampio, non dimostrazione esaustiva di assenza di bug; nessuna lettura di database, input paziente, `.env`, credenziali o dataset personali; nessun benchmark esteso, test Windows locale, link/release Tauri o interazione manuale desktop. Le informazioni esterne sono documentazione corrente non fissata a un tag; versione Rust/Tauri/plugin/MSRV e licenze future richiedono nuova qualifica.

## 3. Fotografia repository

Root relativa **`.`**. Remote normalizzata senza credenziali: `github.com/UraganoAngelina/lintra`, nome `origin`. HEAD del 30 ottobre 2026 alle 11:24:05 +01:00. Commit precedenti:

| Commit | Oggetto |
|---|---|
| `fe5ada8` | Chiusura documentale packaging UI/M8 |
| `aa7be4f` | Preservazione byte licenze nei checkout nativi |
| `4e44703` | Test contesa SQLite con assenza di progresso prima del retry verificato |
| `8695b72` | Policy scanner X11/Common Controls Windows |
| `eaf221b` | Remapping build UI e isolamento font probe |

Baseline iniziale `git status --short`: **8 file tracked modificati, 3 untracked, nessuna modifica staged**.

```text
 M ATOMIC_TASKS_INDEX.md
 M IMPLEMENTATION_STATUS.md
 M src/application/mod.rs
 M src/lib.rs
 M src/persistence/memory.rs
 M src/persistence/mod.rs
 M src/persistence/sqlite/mod.rs
 M src/persistence/sqlite/tests.rs
?? docs/verification/T-PERSIST-08-preflight.md
?? src/application/history_audit.rs
?? tests/history_audit.rs
```

Sono preesistenti e conservati dall'audit. Gli otto tracked hanno +248/−10 righe nel diff iniziale; gli untracked non entrano in quel conteggio. Il nuovo audit storico e re-export di occurrence sono quindi **working tree**, non codice di HEAD né API pubblica già rilasciata. Al controllo finale tre di questi file hanno hash differenti e si è aggiunto `docs/verification/T-PERSIST-08-audit.md`, per attività esterna a questo audit. Nessuno è stato sovrascritto o corretto; hash iniziali/finali in §15. Il solo file creato da questa attività è il presente report.

Ambiente `[TEST]`: Linux `7.3.0-4-cachyos`, x86_64; Rust `1.99.1 (58cf8f9c3 2026-10-13)`, Cargo `1.99.1 (5f94df478 2026-08-27)`, LLVM 23.1.1. `rust-toolchain.toml:1` pinna 1.99.1/minimal/rustfmt/clippy e target GNU Windows; nessun `rust-version` nel manifest prodotto. Host Node `v26.12.1`, npm `12.2.1`: **disponibilità host, non requisito/pin Lintra**. Fontconfig rilevato da pkg-config: 2.18.3. WebKitGTK 4.1 non trovato da pkg-config: non dimostra assenza di qualsiasi runtime WebKit, ma manca questa evidenza di sviluppo.

```text
workspace Cargo (resolver 3; default-members = ["."])
├── lintra 0.1.1-rc.1, edition 2024
│   ├── library src/lib.rs
│   ├── CLI src/main.rs → mod cli
│   ├── 84 target integration test (include history_audit locale)
│   └── bench validation + persistence, harness=false
└── lintra-ui 0.1.1-rc.1, edition 2024
    ├── library ui/src/lib.rs (include_modules Slint)
    ├── binario ui/src/main.rs, nome lintra-ui
    ├── 14 target integration test
    └── dipende da lintra, slint, serde/json, tempfile

esclusi: xtask (workspace autonomo + lock proprio), spikes/ui-prototype
xtask → lintra; lintra-ui → lintra; lintra non → lintra-ui/Slint
```

Features: core default `[]`, `embedded-bundle=[]`; UI default `embedded-bundle`, inoltrata al core. La semplice `cargo test/check --all-targets` alla root **non include UI**; serve `--workspace` o `-p lintra-ui`. Cargo.lock unico prodotto; xtask e spike hanno lock separati. Nessun React/TS/Tauri nel grafo prodotto. Direzione osservata con metadata/tree V-01/V-03 e manifest, non dedotta dal piano.

**Contenuto incorporato effettivo:** soltanto `SYNTHETIC/v1`, sviluppo/demo, più un piccolo dataset dimostrativo; non profili regionali ufficiali (`bundles/embedded/README.md:1`). Il catalogo predefinito non equivale quindi a SIOC/CAD operativi. I profili locali SIOC `structural-minimal.1/.2` e il sottoinsieme golden sono artefatti distinti; il README .2 conserva capability mancanti, inclusa persistence.audit.v1, e dichiara il rifiuto del profilo completo. Non si presume che la compilazione separata del profilo PERSIST abbia rimosso questo limite (`schemas/sioc/2026-04-20.structural-minimal.2/README.md:31`).

## 4. Stato del piano originale

Fonte primaria dichiarativa: `IMPLEMENTATION_STATUS.md:2484`, `ATOMIC_TASKS_INDEX.md:8`; la frontiera più recente è a `IMPLEMENTATION_STATUS.md:5`. Il piano originario è una prescrizione storica, non una fotografia attuale. `[HISTORY]` **117/140 DONE (83,6%)**, 1 IN_PROGRESS, 22 NOT_STARTED; percentuale dichiarata di task, non percentuale di migrazione o completezza normativa.

| Fase | Stato dichiarato | Riscontro rappresentativo e limite verificabile |
|---|---|---|
| P0 | 9/9 DONE; G0 APPROVED_WITH_DEFERRALS | Contratti, schema authoring, profilo SIOC minimale; dataset/capability completi e CAD operativo rinviati (`IMPLEMENTATION_STATUS.md:2434`) |
| P1 | 3/3 DONE | Toolchain, attributi, matrice Linux/MSVC presenti; CI HEAD NOT_VERIFIED |
| P2 | 3/3 DONE | ByteRange, identità/errori, summary e re-export; doctest non forgiabilità in lib |
| P3 | 7/7 DONE | raw/compiled/repository/linter/compiler/registry reali, formato 1/2; no compilazione normativa universale implicita |
| P4 | 5/5 DONE | RecordReader byte/BOM/encoding/audit; non carica l'intero input |
| P5 | 7/7 DONE | ValidationSink/JSONL/summary/AtomicReportFile/golden; V-08 mirato |
| P6 | 8/8 DONE nello scope ratificato | logical extraction/presenza/datatype/constraint/record; niente validator fisico degli shift colonne |
| P7 | 6/6 DONE originariamente LocalOnly | Servizio ampliato da P13 a Full; clock/token/progress reali. Non include automaticamente persistenza |
| P8 | 6/6 DONE | CLI validate/validate-flow/schema tooling, segnali ed exit code; non usata come subprocess dalla UI |
| P9 | 6/6 DONE | range/date/Decimal2/regex e diagnostici; nessun float per importi |
| P10 | 6/6 DONE | AST/TruthValue/derivati/regole/presenza; capability non supportate rifiutate |
| P11 | 6/6 DONE | provider filesystem/embedded/in-memory, evidenze Verified/Degraded/Unavailable; non certifica dataset regionali assenti |
| P12 | 5/5 DONE | CSV, XLSX postanalisi, replay, workbook a memoria costante; non sink XLSX nel loop |
| P13 | 8/8 DONE, M6 scope referenziale | due passate, projection/context/join, Full e golden cross-trace reale tracciato; non Full SIOC clinico completo |
| P14 | 5/5 DONE, M7 | importer XLSX/lint/canonical/hash/diff/catalogo DEC18, comandi CLI |
| P15 | 7/7 DONE | Matrice specifica sotto; release CLI/dati distinte dalla UI |
| P16 | 9/9 DONE, M8 scope UI | Tutte 00–08 riscontrate sotto; assenza storico UI esplicita |
| P17 | 01 e 06 DONE; 02–05 NOT_STARTED | harness benchmark e robustezza presenti; soglie reali, max_errors, parallelismo non acquisiti |
| P18 | 01 e 02 DONE; 03–04 NOT_STARTED | README/runbook/smoke presenti, sincronizzazione finale non completata |
| PERSIST | 01–07 DONE; 08 IN_PROGRESS; 09–12 NOT_STARTED | profilo/tipi/port/HMAC/SQLite/metadati/approvazione implementati; 08 solo working tree, snapshot cumulativo/ulteriori lifecycle non presunti |
| CAD | 01–12 NOT_STARTED | contratto candidato e fonti disponibili, nessun runtime storico/temporale CAD completo; non deducibile da backend generico |

Per P0–P14 il riscontro è per componenti critici, non riesecuzione di ciascun acceptance gate. Le milestone dichiarate M0/M1/M3/M5/M6/M7/M8 REACHED e M2/M4 LOCAL_SCOPE_VERIFIED sono in `IMPLEMENTATION_STATUS.md:2465`; M9 NOT_REACHED. I dossier normativi del16/10 dichiaravano assente il workbook SIOC; il file ora è presente sotto `schemas/xlsx/` e T-P14-01 ne registra l’acquisizione/import reale (`docs/verification/T-P14-01-xlsx-importer.md:9`). Presenza del documento non prova eseguibilità normativa completa e non autorizza altri flussi. Alcune motivazioni di righe M2/G-PERSIST sono obsolete rispetto a M1 e PERSIST07: **CNF-MIG-04**, senza riscrittura dei registri.

| Task P15, tutte dichiarate DONE | Codice/evidenza ispezionata |
|---|---|
| 01 release profile | Cargo profile thin-LTO/1 codegen-unit/strip, config CRT; xtask build-release/remapping |
| 02 verifier | `xtask/src/artifact.rs::inspect_product`, policy per prodotto; ratifica Linux A |
| 03 embedded | `src/bundle.rs`, repository/preparer embedded, bundle_resolution; feature default UI, non default core |
| 04 data bundle | `xtask/src/bundle_data.rs`, MANIFEST/hash/ZIP; indipendenza eseguibile/dati |
| 05 executable release | `.github/workflows/release.yml`, `scripts/release.py`; dossier release 40352109303, non riletta online |
| 06 data release | `scripts/release.py`, dossier T-P15-06; stessa separazione e immutabilità |
| 07 cross-OS | `tests/packaging_probe.py`, job compare-jsonl in ci.yml; output identico richiesto |

| Task P16, tutte dichiarate DONE | Implementazione riscontrata | Limite del riscontro |
|---|---|---|
| 00 spike | `spikes/ui-prototype`, dossier T-P16-00 | renderer/risultato spike storico; non prodotto completo |
| 01 workspace | Cargo membri/default + `scripts/check_workspace.py:15` | guard esclude GUI dal core, non verifica assenza clap |
| 02 bootstrap | build.rs/fonts/main/tokens/AboutSlint e test bootstrap/font | test presenti; esecuzione UI locale NOT_RUN |
| 03 configurazione | analysis/controller/analysis.slint, `analysis_configuration.rs` | un tracciato UTF-8/LocalOnly; catalogo ≠ certificazione |
| 04 worker | worker/worker_controller, worker e worker_window test | canali limitati/cancel/shutdown, non async IPC |
| 05 risultati | results/mod/controller/diagnostic, results.slint e test | cache limitata, indice lineare, nessun sort |
| 06 export/Info | export/export_controller/export.slint, exports/export_window | writer core, export completo non filtrato, doppio output non transazione unica |
| 07 visivi/diagnostici/Signature | field_guide, dominio, signature/visual/schema_binding/domain_reference | reference chiare 1×/2×; confronto nativo temi nei dossier; non qualifica WebView |
| 08 packaging | xtask product ui, scripts/ui_package.py, ui-package.yml e probe | archivi CI interni qualificati aa7be4f; nessuna nuova release UI pubblicata |

Ultimo handoff operativo: `IMPLEMENTATION_STATUS.md:28`, T-P16-08 DONE → T-PERSIST-08; il lavoro attivo non è stato ripreso da questo audit. ADR-PERSIST-001/002 approvate, DEC18 ratificata; redesign Signature autorizzato e font/palette correnti in design docs. Restano G-SIOC-HIST OPEN_POLICY e CAD runtime non consegnato.

| Gate | Prescrizione [DESIGN] | Evidenza disponibile qui |
|---|---|---|
| QG-1 | controlli locali | fmt/check/Clippy pass, suite mirata §11; **non** tutto STD-VERIFY |
| QG-2 | Linux + Windows nativi | HISTORY aa7be4f, live NOT_VERIFIED, working tree non qualificato |
| QG-3 | layer/no hardcoding/compiled-only | moduli/grafo ispezionati, test mirati; nessun audit semantico esaustivo di tutti i rami |
| QG-4 | byte golden/ordine | suite golden mirata §11; cross-OS soltanto HISTORY |
| QG-5 | artifact/dependency policy | checker e allowlist presenti, ratifiche applicabili a Slint/CLI; nuova release locale NOT_RUN |
| QG-6 | streaming/memoria | CODE + HISTORY benchmark/test300k/1M; nessuna misura fresca end-to-end |
| QG-7 | congruenza UI | HISTORY nativa e PNG conservati; nessun risultato Tauri |
| QG-8 | documenti sincronizzati | non affermato globalmente: P18-04 aperta e incongruenze storiche §13 |

## 5. Architettura attuale e dipendenze

```text
UI Slint (proprietà/callback, Rc<RefCell>, Weak<MainWindow>)
   │ selezioni → AnalysisBundle/AnalysisSelection → PreparedAnalysis
   │                       │ SchemaRegistry + hash + Arc<CompiledFlowSchema>
   └─ WorkerHandle ────────►ValidationService ◄──────── CLI (mod cli nel binario)
       thread owned             │
       token cancel             ├─ LocalOnly: TraceValidator → RecordReader
       progresso 1 slot         │                  → RecordValidator/FieldPipeline
       esito 1 slot             └─ Full: FlowValidator, pass1 projection/index
                                    → ContextStore effimero → pass2 join/DSL
                                │
SchemaRepository → raw → compiler→compiled ← reference preparer/evidence
                                │
                     ValidationSink → JsonlSink → spool atomico owned
                                │
             summary/outcome + AnalysisReport (schema/reference/audit)
                                │
             ReportIndex (worker) → JsonlModel cache128 → ResultsView/detail
                                └─ export worker → replay/core XLSX/CSV/JSONL
                                                     + summary JSON atomici per file

PERSISTENZA SEPARATA (non collegata ai callback Slint/validate automatico)
CompiledPersistenceProfile → ApprovalDraft → ValidatedApproval → preview
    → consenso/approve_commit → report callback → repository transaction/CAS
IdentityProtector → record protetti → SQLitePersistenceAdapter / test in-memory
    → snapshot/metadati/revisioni/CommitResolution
Working tree: HistoricalAuditService → letture repository → diff streaming
```

| Ownership/modulo | API e chiamanti effettivi | Dati/comportamento e dipendenze | Estensione/test/coupling |
|---|---|---|---|
| `domain` | ByteRange, SchemaKey, ValidationError, Summary, RunStatus, PersistenceNamespace/SourceSubmission | dati neutri; valori byte/identità/provenienza, nessun widget | test dominio/summary/persistence_types; KEEP |
| `schema` | SchemaRepository/SchemaCatalog, SchemaCompiler, Registry, compiled getter | COSA validare, raw→compiled non forgiabile; parser/lint/canonical hash/encoding binding | repository/preparer iniettati; test compiler/catalog/compatibility; nessun Slint |
| `input` | InputSource::Path, RecordReader::read_next, RecordView, audit | buffer riutilizzato/slice raw/logical, newline e BOM espliciti, hash raw e conteggi | test record_reader/input_matrix/audit; nessun Slint |
| `validators` | presence/logical/datatype/domain/range/date/Decimal/regex/reference | funzioni primitive guidate da compiled, nessuna norma di flusso | test dedicati; keep byte semantics |
| `engine` | RecordValidator/TraceValidator/FlowValidator/ExpressionEvaluator/field_guide | COME validare, error_factory e ordine stage, projection e join/Unknown | chiamato da application; field_guide riusabile per UI; test golden/flow/field |
| `context` | ContextStore, InMemoryContextStore, ProjectedRecord, coverage | indice effimero per run; possiede soli campi proiettati, crescita per projection/chiavi, non memoria costante Full | estendibile per trait; context_store/memory tests; nessun SQL/Slint |
| `reference_data` | Preparer/Provider, PreparedReferenceEvidence, LookupSet/domain_page | dataset versionati/hash, stato/normalizzazione; dati immutabili catturati nel run UI | provider filesystem/in-memory/embedded; domain_reference; nessuna WebView |
| `application` | ValidationService::validate/validate_with_parameters; clock/progress/token; session approval | orchestration schema/ref/trace/full/summary; servizio di validazione non possiede repo persistente | DI reale; session service distinto da validate; future bridge richiede stato posseduto |
| `persistence` | PersistentContextRepository/PersistenceTransaction, SQLitePersistenceAdapter, IdentityProtector | SQL nell'adapter, sicurezza filesystem/protezioni/codec, CAS/revisioni/metadata | driver sostituibile; test sintetici restart/contesa; nessun Slint |
| `report` | ValidationSink, JsonlSink, AtomicReportFile, write_run_summary*, replay_canonical_report, XlsxReportWriter | errori stream e summary separato, conversione postanalisi; I/O tecnico terminale | CLI/UI condividono writer; VecSink esiste per test/piccoli casi, non default UI |
| `cli` | run/parse/validate/validate-flow/schema/reference tooling | risolve percorsi, sink/output, segnali, exit code; `src/main.rs` importa `mod cli` | codice non nel modulo library; package condivide clap/ctrlc; cli_* tests |
| `ui` | controller/worker/results/export + MainWindow | possiede intenti, report temporaneo, cache, selezione e proiezione presentazione | Slint in Model/SharedString/Rc/Weak/Timer; logica di I/O risultati non tutta separabile oggi |

Anchors: `src/lib.rs:83`, `src/schema/registry.rs:50`, `src/input/reader.rs:14`, `src/context/store.rs:56`, `src/application/service.rs:248`, `src/report/sink.rs:56`, `src/persistence/mod.rs:242`; simboli UI in §6–8.

Percorso concreto: il main risolve il bundle da directory/ambiente/exe/embedded; la scelta chiave/versione/trace ed encoding costruisce registry e compila, poi il controllo iniziale legge massimo 65.536 byte senza trattenerli nella UI. `PreparedAnalysis` contiene richiesta, servizio Arc, schema/hash e contenitore delle reference catturate. Il worker riapre la sorgente, esegue il servizio con sink JSONL atomico, controlla hash/schema dell'esito e associa le reference verificate alla stessa esecuzione. Il report pubblicato è solo lo spool privato del run; un export utente è un'azione successiva. La richiesta corrente del form non può retargettare l'export di un run già visualizzato (`ui/src/analysis.rs:327`, `ui/src/worker.rs::execute`, `ui/src/results/controller.rs:99`, `ui/src/export_controller.rs:34`).

`CompletedInvalid` è un esito `Ok`, non eccezione; `Cancelled` produce outcome controllato senza report definitivo; `Err` tecnico invalida l'esito corrente. La completezza è una dimensione distinta: anche CompletedValid può essere parziale per reference/regole non definitive. L'ultimo report completo può restare owned dopo retry fallito/cancellato, senza essere presentato come successo del nuovo run (`src/domain/summary.rs:112`, `ui/src/worker_controller.rs:28`). `ValidationRunError::Context` riguarda contesto effimero; gli errori persistenti/sessione hanno tipi separati e non sono già un ramo integrato in validate.

## 6. Inventario UI Slint

Tutti gli asset di produzione Slint: **sei file** in `ui/ui/`: `main.slint` (shell/global props/About), `tokens.slint` (Tok, font/palette/layout), `controls.slint` (ActionButton/ThemeChoice/FieldEdit/ChoiceBox/ReportCheck), `analysis.slint` (AnalysisForm/FileBrowser), `results.slint` (ErrorRow/ResultsView), `export.slint` (ExportPanel). Build: `ui/build.rs:1` include il build script di provenienza root, verifica i cinque TrueType e compila main con `EmbedFiles`/stile fluent-light. `ui/src/lib.rs:2` include moduli generati e seleziona winit/software esplicito; main gestisce event loop e shutdown.

Font incorporati: IBM Plex Sans Regular/SemiBold/Bold e Mono Regular/SemiBold, cinque `.ttf`; licenza `ui/ui/fonts/OFL.md`; Slint `ui/licenses/SLINT-ROYALTY-FREE-2.0.md`. Nessun font di sistema richiesto per sostituire queste facce. Logo come `Path` geometrico in main, non immagine. Token: chiaro avorio `#F5F4F0`/rame `#A13E1A`, scuro grafite `#101112`/rame `#FF9C66`; Sans14 e Mono12, min640×480, breakpoint1100 e pannelli impilati/scroll, nav64px e righe36px. Questi sono valori osservati (`ui/ui/tokens.slint::Tok`, design corrente); non si propone un nuovo stile.

| Feature | File/simbolo | Input → output | Stato visibile | Dipendenza core | Stato implementazione | Test presenti | Complessità sostituzione e note |
|---|---|---|---|---|---|---|---|
| Shell/navigazione/tema | main.slint::MainWindow, tokens::Tok | proprietà run + click → analisi/risultati/Info, dark-mode | risultati disabilitati senza report; focus/tastiera | nessuna norma | presente | bootstrap, signature, visual | MEDIUM: primitive e binding da riscrivere; tema solo finestra |
| Bundle/catalogo | analysis::AnalysisBundle, controller::install_configuration | dirs/env/exe → entries/audit/label | origine, versione bundle o indisponibilità | resolve_bundle_source, SchemaCatalog/repository | presente | analysis_configuration | MEDIUM: semantica riusabile; nessun upload/install bundle GUI |
| Nuova analisi/schema | AnalysisSelection::{select_flow,select_version,prepare} | flow/version/trace/encoding → PreparedAnalysis | scelta esplicita, nessun latest, errori di selezione/compilazione | Registry/Compiler, ValidationRequest | presente LocalOnly | analysis_configuration, schema_binding | HIGH: stato dipendente e invalidazione; solo UTF-8 nel selettore |
| Percorso e Sfoglia | controller::{edit_file,confirm_file,open_browser}, FileBrowser | path testuale/indice → file regolare confermato | richiesta Invio, cartella vuota/inaccessibile, non UTF-8 non selezionabile | InputPreview/BOM | presente, browser integrato | analysis_configuration | MEDIUM: non usa file picker OS/GTK/portal; elenco intera directory in Vec |
| Controllo iniziale | InputPreview::inspect | file → dimensione/campione LF/CRLF/CR, EOF | descrive campione, BOM rifiutato, nessuna modifica input | preflight_bom Reject | presente | analysis_configuration::prefix_preflight* | MEDIUM: fino64KiB, non prova che il resto del file sia uniforme |
| Formati richiesti | ReportSelection, ReportCheck | JSONL/XLSX/CSV bool → intento sul run | preferenze, nessun file già salvato | writer usati dopo run | presente | analysis_configuration, exports | LOW: XLSX+JSONL default, CSV false; salva un formato alla volta |
| Esecuzione | WorkerHandle/WorkerController | PreparedAnalysis → progresso + WorkerResult | avviata/fase/contatori/completezza/invalidità | ValidationService/Sink/Token | presente | worker, worker_window | HIGH: niente percentuale fittizia, progresso50ms, stati terminali affidabili |
| Annulla/chiudi | worker::Drop, main shutdown | token/uscita loop → cancel + join | richiesta distinta da esito Cancelled | CancellationToken | presente | worker/worker_window | HIGH: join cooperativo, nessun limite massimo shutdown dimostrato |
| Risultati tabella | ReportIndex/JsonlModel, ResultsView ListView | JSONL+summary → offset/righe lazily | loading/vuoto/violazioni/parziale/errore | ValidationError/summary | presente virtualizzato | results, results_window | HIGH: cache128 ma indice O(n); limite i32::MAX righe visualizzabili |
| Filtri | ResultsFilter, ResultsController::apply | code/field/record/fiscal/line → nuovo indice | AND esatto case-sensitive, nessun match distinto da errore | decode diagnostici | presente | results::all_five_filters* | HIGH: scansione completa worker per applicazione, non ricerca full-text |
| Selezione/dettaglio | ResultsController::select/selected_offset, results::project | indice → ErrorRow e contesto | stabile fuori viewport, annullata se filtrata via | DiagnosticValue/field_guide | presente | results/results_window | HIGH: selezione con byte offset legato al report, non numero riga del modello |
| Dominio ammesso | show_domain_page/restart_domain, field_guide | field/constraint/cursor → pagina max64 | tipo/posizioni/width/alignment/fill, valori/range, dataset attendibile distinto | schema e PreparedReferenceEvidence del run | presente | domain_reference, schema_binding, validation_diagnostics | HIGH: range esatti oltre u128; mai enumerazione infinita regex o dominio inventato |
| Ordinamento risultati | ordine JSONL/offset | nessuna API sort → ordine canonico | ordine originale dichiarato | determinismo core | **assente** sort interattivo | results ordine originale | UNRESOLVED solo se il piano vuole aggiungerlo: non requisito di parità attuale |
| Export | ExportController/ExportJob/export_report, ExportPanel | run visualizzato+format/path/overwrite → report+summary | busy/cancel/error/overwrite/esito parziale | replay, XLSX/CSV/JSONL/atomic | presente | exports, export_window | HIGH: tutto il run, filtri ignorati; due pubblicazioni distinte, no multi-file atomicity |
| Informazioni | refresh_information, main.slint::AboutSlint | vista attiva → versione software/schema/bundle + attribuzione | provenienza della vista risultati o selezione | summary/BundleAudit | presente | bootstrap/export_window | LOW/MEDIUM: widget/licenza specifico Slint, non elenco completo dipendenze dinamico |
| Impostazioni persistenti | dark-mode/reports stato finestra | nessun contratto salvataggio → nessun file config | tema in memoria | nessuno | **assente**, soli controlli di sessione | bootstrap | non introdurre persistenza preferenze come fatto esistente |
| Storico/preview/confronto/PROCEED/ABORT | nessun callback MainWindow | nessuno | nessuna schermata | servizi session/persistence separati | **documentato soltanto per UI**, servizi Rust parziali presenti | session_approval, non test UI | fuori parità; non costruire schermate dal solo ADR |
| Namespace/CAD/sessioni/rollback/lock | nessun controller UI | nessuno | nessuno | PERSIST/CAD | **assente UI**, CAD runtime futuro | test repository/sessione separati | non simulare consenso o storico |
| Full multi-trace | nessuna opzione form | worker rifiuta scope Flow | controlli locali espliciti | Full servizio/CLI presente | **assente UI** | full_service/cli_full | non inventare disponibilità dal codice servizio |

Threading attuale: configurazione/browser/compilazione iniziale sul thread UI; validazione in thread dedicato, canali sincroni capacità1 (progresso droppable + terminale separato), `Weak::upgrade_in_event_loop`. Indice e pagine reference su worker; polling risultati50ms, export40ms. `JsonlModel::read_row` legge e decodifica una riga su cache miss nel thread UI: la scansione globale è fuori dal loop, non tutto l'I/O. Owner Arc trattiene TempDir/schema/reference; Drop cancella e joina i job. Non è presente un protocollo IPC con session ID/sequence/ack (`ui/src/worker.rs:105`, `ui/src/results/mod.rs:252`, `ui/src/results/controller.rs:20`, `ui/src/export_controller.rs:151`).

Mockup: la cartella adiacente contiene versione HTML, prototipo Rust Slint e documenti architetturali/board; `spikes/ui-prototype` contiene altri due `.slint`, build/lock separati. Sono **mockup/prototipo soltanto**, esclusi dai membri di prodotto; non si assumono feature dal loro aspetto. I riferimenti Signature correnti sono design docs + test eseguibili; non si ripristinano capture eliminate.

## 7. Matrice di parità funzionale

Questa matrice completa §6 esplicitando il rischio **della sostituzione**, non un difetto provato. Scala LOW/MEDIUM/HIGH motivata. Colonne N/P/X/M/U/D = correttezza normativa, privacy, cross-platform, prestazioni/memoria, uso, deployment. I test qui sono requisiti di preservazione da sottoporre al piano, non nuovi test scritti.

| Funzionalità attuale e stato | Codice concreto / prove disponibili | N/P/X/M/U/D | Regressione da evitare / prova richiesta |
|---|---|---|---|
| Bundle esterno prioritario e embedded, presente | AnalysisBundle + bundle_resolution; analysis_configuration, V-01 | H/M/H/M/H/H | fallback misto o latest; quattro livelli/errori identici, hash/audit corretti |
| Selezioni esplicite/schema preparato, presente | AnalysisSelection::prepare + schema_binding | H/M/M/M/H/M | stato figlio obsoleto o schema cambiato; invalidazione e confronto hash |
| File browser e BOM/campione, presente | browse_directory/InputPreview | M/H/H/M/H/M | percorsi Unicode/permessi, byte modificati o campione spacciato per intero file |
| Run LocalOnly/contatori, presente | WorkerHandle/ValidationService | H/H/H/H/H/M | blocco UI/backlog/false percentuali; eventi limitati e contatori finali |
| Cancel/errore/invalidità/parzialità, presente | WorkerController + AtomicReportFile; V-08 atomico | H/H/H/M/H/M | Cancelled o Err trasformato in valido; no report completo del run interrotto |
| Tabella lazy/virtualizzata, presente | ReportIndex/JsonlModel; results300k/1M HISTORY | M/H/H/H/H/M | milioni di DTO in JS, crescita cache, righe perse; memoria/latency su entrambi gli OS |
| 5 filtri esatti e selezione offset, presente | ResultsFilter/select/poll | M/H/M/H/H/L | trim/casefold, record diverso dopo filtro, contaminazione fra run |
| Dettaglio/diagnostica/range/reference, presente | project/field_guide/domain_page; V-08 rendering | H/H/M/H/H/L | tipo, posizioni1-based, fill/AN numerico, stato dataset alterati; schema/run snapshot coerenti |
| Export completo e overwrite, presente | export_report/core writers; exports HISTORY | H/H/H/H/H/H | dati persi/alterati, fonte sovrascritta, summary di altro run; errori e pubblicazione parziale espliciti |
| Informazioni/attribuzioni, presente | AboutSlint/refresh_information | M/L/M/L/M/M | provenienza dell'ultimo worker anziché vista; attribuzioni durante coesistenza |
| Temi/layout/tastiera, presente | Tok/controls/main/results | L/M/H/H/H/M | contrasto, DPI, scroll, focus e font diversi; nuova accettazione visiva multipiattaforma |
| Full CLI/core, presente senza UI | validate_full/cli args | H/H/H/H/M/M | preservare CLI/golden senza anticipare nuova feature GUI |
| Sessioni/protezione SQLite, presenti come API separate | ApprovalDraft/Preview/SQLite | H/H/H/H/H/H | approvazione involontaria, doppio commit, false rollback; regressioni native mirate necessarie |
| Storico UI/CAD/sort/settings durevoli, assenti | MainWindow privo di callback e ADR/PERSIST future | H/H/H/H/H/H se aggiunte | nessuna promessa di parità; eventuale ampliamento scope richiede decisione |

## 8. Inventario dei contratti UI ↔ Rust

La library esiste, il contratto IPC **no**. `ValidationRequest` è un tipo Rust Clone/PartialEq senza derive serde; `ValidationSummary` include SystemTime/Duration e mappe, non è direttamente il JSON del report. RunStatus/ValidationMode e SchemaKey hanno serde; il formato canonico è scritto dai writer dedicati. Arc, trait object, riferimenti borrowed, `ApprovalPreview<'repo>`, cancellation atomici e Model Slint non sono payload TS. Nessuna promessa di serializzazione degli errori Rust o di compatibilità semver dell'IPC (`src/application/request.rs:4`, `src/domain/summary.rs:11`, `src/report/summary.rs`, `src/application/session.rs:701`).

| Operazione | Implementazione Rust / caller Slint | Tipi input → output e ownership | Side effect / errore | Durata/frequenza e dati sensibili | Implicazione futuro bridge (gap, non API approvata) |
|---|---|---|---|---|---|
| Catalogo/bundle | AnalysisBundle::resolve; main/install_configuration | BundleDirectories + EnvProvider + Path → owned entries/repository/preparer/audit | letture filesystem/manifest, Failure/BundleError | startup; percorsi locali, no record | esporre metadati minimi/versioni esplicite; risoluzione rimane Rust |
| Flow/version/trace | controller/select + AnalysisBundle::{flows,versions,traces} | String/indice → Vec<String>, SchemaKey | solo stato e invalidazione | interazione breve; ID schema non PHI | indici di UI non devono diventare identità persistenti IPC |
| Conferma file | InputPreview::inspect / confirm_file | &Path → InputPreview contatori ≤64KiB | legge, BOM/Input/io typed | su conferma; path personale, byte non esposti | picker/path scope/handle e metadati; nessun invio contenuto input al frontend |
| Preparazione run | AnalysisSelection::prepare / refresh | selezioni → PreparedAnalysis (Arc service/schema, request/hash/ref-capture) | compile+read iniziale, ConfigurationError | per refresh confermato; input path | compilazione non sul thread JS; DTO separati da handle backend |
| Avvio | WorkerHandle::start / WorkerController::start | PreparedAnalysis owned + CancellationToken + Notify → handle | thread + spool TempDir; Spawn/Validation/Report/SchemaChanged | una run attiva nel controller; input sanitario solo Rust | identificazione run, stato/race, error mapping: contratti ancora da stabilire |
| Core validate | ValidationService::validate / worker::execute | request owned; &mut sink/progress, &token → Result<Outcome,RunError> | legge input, emette sink; nessun commit persistente | sincrono lungo, progress count di trace | comando asincrono non deve bloccare runtime; worker e backpressure richiesti |
| Progress | ChannelObserver/upgrade_in_event_loop / worker_poll | ProgressEvent → WorkerProgress (phase/trace/count) | try_send, notify; terminale separato | contatori limitati50ms; niente record/ValidationError | eventi/canali da valutare; no milioni errori live; ordering e fine affidabile |
| Cancel/shutdown | CancellationToken/WorkerHandle Drop / cancel-analysis e main | clone token/handle → richiesta, poi esito | cooperative cancel e join; panic/disconnect distinti | run e close; nessun payload clinico | token Rust non serializzabile, invalidazione e lifecycle finestre da censire |
| Apri risultati/filtri | IndexJob::start/ReportIndex::scan / open/apply | Arc report + summary + ResultsFilter → indice owned | lettura scan, cancellazione AtomicBool, malformed/scope/count mismatch | O(tutte righe) per filtro; codice/CF filtro sensibile | query/pagine in Rust, limiti payload e cache; no indice globale JS |
| Riga/dettaglio | JsonlModel::read_row/project / Model e select | numero modello → offset → Option<ErrorRow> | seek/read/decode; ResultsError | per cache miss, cache128; CF/actual/context PHI | estrarre proiezione neutra; ID stabile legato al run/offset; bounded response |
| Dominio statico | declared_domain_page/show_domain_page | compiled constraint + DomainPosition → DomainPage { values,next } | read-only schema | max64 valori; dati normativi, integer arbitrari | cursore opaco/versionato e ordinamento da contrattualizzare |
| Dominio reference | PreparedReferenceEvidence::domain_page / ReferencePageJob | dataset key/after/token → pagina + stato | usa snapshot catturato, non file riletto; stato degradato/assente distinto | max64; valori dataset da minimizzare | solo Verified come dominio ammesso; nessun reload con file diverso |
| Export | ExportJob::start/export_report / ExportController | Arc report + summary + format/path/overwrite → Receipt o Failure | file principale + summary; errori/partial files/cleanup | una job, poll40ms; diagnostici sensibili su disco | filesystem/export in Rust, autorizzazione path, errori redatti; no blob integrale JS |
| Informazioni | refresh_information / toggle-information | vista → version/schema/bundle | read-only proprietà | apertura, non ultimo form in risultati | provenienza dello stesso run, testo/licenze frontend |
| Persistenza preview/approva | ApprovalDraft → ValidatedApproval::preview → ApprovalPreview::approve_commit | compiled profile/namespace/source/protected rows, repo borrowed → receipt/error | report callback prima transazione; CAS/idempotenza/ambiguità | nessun caller UI; metadati pseudonimi ancora personali | **gap di scope UI**; preview non clone/deserializzabile non può diventare consenso frontend forgiabile |
| Audit storico | HistoricalAuditService::{prepare}, PreparedHistoricalAudit::write_diffs | profile/repo/protector/candidate → diff via callback + summary | read-only, generazione fissata, errori tecnici | working tree, nessun caller Slint; identities protette | non API stabilizzata; non vincolare il piano a WIP non qualificato |

Dettagli da chiarire nel successivo piano, **nessuna scelta di libreria DataGrid/React o signature definitiva qui**:

- Comandi tipizzati, DTO/errori/versioni, eventi versus canali, schema di correlazione run/richiesta, numeri oltre precisione JS, timestamp/Duration e cursori; limiti pagine/byte, annullamento query, terminale affidabile e backpressure. I due canali Rust attuali dimostrano un requisito di boundedness, non la sua soluzione IPC.
- Backend mantiene input, spool, indice e reference; frontend riceve solo vista/pagina e dettaglio richiesti. Virtualizzazione DOM e selezione stabile separata dalla paginazione; eventuale sort non è già implementato e non deve modificare l'ordine canonico dell'export.
- Contenuto di record e messaggi è non attendibile; testo da rendere senza HTML eseguibile, nessuna interpolazione raw in script/URL. File picker, filesystem scope, capability per finestra, CSP, protocollo asset e URL esterni richiedono threat model. Nessuna shell arbitraria necessaria all'attuale percorso.
- PHI in IPC/DOM/devtools/console/crash/cache/storage WebView/telemetria/export richiede minimizzazione e decisioni esplicite. La chiave HMAC non attraversa il frontend. Una CSP non sostituisce l'autorizzazione dei comandi applicativi.
- Worker può continuare quando una finestra si chiude: ownership e stop devono preservare drop/join/spool. Cancel tardivo, errore di filtro e nuovo run non devono far riapparire risultati vecchi come correnti. Durata massima di shutdown e I/O bloccante: `NOT_MEASURED`.

## 9. Dati, sicurezza, privacy e invarianti normativi

| Vincolo da preservare | Riscontro e responsabilità |
|---|---|
| Schema sa COSA, motore COME; solo compiled | ARCH §3/5, `src/schema/compiled.rs:220`, compiler/registry; compilazione e hash da medesimo raw, nessuna norma React |
| Byte/raw/logical/MissingValuePolicy | RecordReader/RecordView/validators::logical_value; no trim/casefold globale; AN non vieta implicitamente numeri |
| Posizioni e larghezze | ByteRange zero-based/end-exclusive interno, diagnostici one-based; tipo/fill/allineamento da schema effettivo, nessun nuovo E_ALIGNMENT fisico |
| Ordine/tre valori | FlowValidator e FieldPipeline/DSL; Unknown e NotApplicable non false/valido; errori deterministici trace/riga/campo/stage/regola |
| Audit | RunMetadata/summary/input_audit e BundleAudit; versione/hash schema, reference con digest atteso/osservato, nome/dimensione/hash raw input e UTC; path canonicalizzati solo gestione privata, non audit di input |
| Stream/output | ValidationSink terminale su errore; JSONL UTF-8/LF/no BOM; summary separato, XLSX da JSONL con testo/rollover; file atomico, non transazione report+DB o coppia export |
| Context effimero ≠ archivio | `src/context/store.rs:56` vs `src/persistence/mod.rs:242`; Full conserva proiezioni, non storico organizzativo |
| Preview/approvazione | `ValidatedApproval::preview` è read-only; candidate/base/repo legati al preview, non forgiabile dal chiamante; report callback prima commit; stato ambiguo esplicito |
| Sessione immutabile/namespace | SourceSubmission/SessionBinding/approve_commit; ULSS/periodi dichiarati da parametri, non nome flusso; consenso trace assenti durevole, file presente vuoto distinto da assente |
| SIOC cross-run | ADR001 §3 e DEC-SIOC-HIST-01: audit non autorizza un errore automatico su PK ripetuta in invio diverso; baseline FROZEN non modificata |
| CAD | policy/documenti candidati e futuro runtime; snapshot cumulativo, progressive/evento, CAD17 calendario ecc. non inferiti dall'implementazione SQLite/sessione generica |

**Backend effettivo:** `rusqlite =0.41.0`, default-features false, `bundled/cache/hooks`, engine SQLite C incorporato tramite libsqlite3-sys; eccezione ADR-PERSIST-002 a N-20 circoscritta, non autorizzazione generale a nuove dipendenze native. SQL parametrizzato/codec/versioni solo nell'adapter. `SQLitePersistenceAdapter::{create,open,settings,upgrade_storage_v1_to_v2}`: formato corrente2, lettura1/2 con upgrade esplicito non distruttivo, nessuna migrazione automatica in open. DELETE letto dopo impostazione, synchronous EXTRA verificato (=3), foreign_keys ON/trusted_schema OFF, cache2MiB/mmap0/temp_store FILE e cache8 statement (`src/persistence/sqlite/mod.rs:112`, `:182`, `:270`, `:287`). Non è stato aperto alcun archivio dell'utente.

`IdentityProtector` usa HMAC e provider chiavi esterno; `PersistenceKey` zeroizing, dominio di protezione/namespace/trace/schema e proof verificati. Non è cifratura del DB né anonimizzazione garantita dei digest. Sicurezza root/permessi/ACL/traversal/symlink è nel modulo security, Windows isolato in `security/windows.rs`. Archivio solo locale per installazione; nessuna promessa SMB/NFS/merge/backup o completezza dell'organizzazione. `load_snapshot` materializza O(n), mentre metadati sono paginati e lookup indicizzati; il WIP08 non rende il DB interamente streaming (`src/persistence/protection.rs:64`, `:153`, ADR002 §quattro prescrizioni).

**Privacy UI attuale:** lo storage protetto non elimina i valori sensibili nei report: `ValidationError` può contenere fiscal_code, record ID, actual e contesto; `results::project` li presenta legittimamente all'analista. Lo spool temporaneo e l'export possono quindi contenere PHI in chiaro. TempDir posseduta e cancellazione controllata limitano la durata ordinaria, non dimostrano secure erase, cifratura del temp, assenza in swap/crash o pulizia dopo kill. Il browser/path di export espone percorsi nella finestra; non sono copiati nel presente report. Qualificazione su dati reali e politiche operative restano `NOT_VERIFIED`, non risolte da questo audit.

[DESIGN] Cifratura a riposo, provisioning/rotazione/recovery chiavi, retention e backup sicuro richiedono decisioni prima dei dati reali (ADR002:23–39). [INFERENCE] Aggiungere WebView amplia la superficie verso DOM/IPC/devtools/cache: il piano deve censire quei dati separatamente dal DB; non è una vulnerabilità Tauri già osservata perché il prodotto Tauri non esiste.

## 10. Toolchain, dipendenze, build, CI, packaging e compatibilità Tauri 2

Versioni effettive rilevanti: Rust/Cargo1.99.1; lintra/UI0.1.1-rc.1; Slint/slint-build/i-slint1.18.1; rusqlite0.41.0; rust_xlsxwriter0.100.0 constant_memory; calamine0.36.1; sha2 0.11.1, hmac0.13.0, zeroize1.9.1; tempfile3.27.0; clap4.6.7; ctrlc3.5.2; regex1.13.1; serde1.0.229; serde_json1.0.151 arbitrary_precision. Le versioni risolte provengono da Cargo.lock/tree, non tutti i requirement sono `=` (serde/json/regex usano range). xtask goblin0.10.7/zip9.0.5, UI dev png0.17.16. Registro qualificazioni in `docs/DEPENDENCIES.md:1`, SQLite in `:204`.

Release profile thin-LTO/codegen1/strip, config MSVC +crt-static; build.rs usa Git solo per provenienza a build-time, oppure `LINTRA_BUILD_COMMIT` esplicito; nessuna dipendenza runtime Git. Il divieto documentale generale di invocazioni da build.rs ha quindi questo caso osservato da riconciliare, non da correggere (§13 CNF-MIG-05). Una build dirty può stampare HEAD senza qualificare i byte WIP: il report distingue sempre HEAD da working tree.

Workflow configurati: `ci.yml`, `ui-spike.yml`, `ui-results.yml`, `ui-package.yml`, `sqlite.yml`, `sqlite-probe.yml`, `release.yml`. CI generale su ubuntu24.04/LinuxGNU e windows2025/MSVC, fmt/Clippy/test workspace default/allfeatures/core-no-bundle, guard grafo, missing-font negativo, benchmark smoke, xtask/release riproducibile/verify-artifact, README smoke, confronto JSONL/pixel nativi. UI packaging opzionale chiamato dalla CI produce archivi/provenienza/licenze e prova avvio estratto; non pubblica release. `release.yml` pubblica il prodotto eseguibile originario via script e preflight gate stesso source SHA; bundle dati ha ciclo separato. Nessuna CI è stata avviata nell'audit.

`verify-artifact` distingue `Product::Cli` e `Product::Ui`: CLI Linux GNU solo libc/libm/loader/libgcc_s ratificati; UI aggiunge dipendenza diretta fontconfig e controlla librerie caricate contro `ui/linux-runtime-libraries.list`, X11/input/font/relative transitive. Windows solo DLL di sistema con estensioni UI esplicite, CRT statica. Scanner path host e openssl-sys restano vincolanti. **Non ampliare automaticamente la policy CLI per far passare Tauri** (`xtask/src/artifact.rs:6`, `:79`, `:228`; ratifica CNF-ARTIFACT-LINUX-01).

| Area | Stato attuale verificabile | Compatibilità/cambiamento contrattuale da chiarire |
|---|---|---|
| Linux x86_64 | HISTORY artifact ubuntu24.04/glibc2.39, runtime X11/software/Fontconfig; Linux interno/personale ratificato | Tauri2 usa WebKitGTK; requisiti build4.1 documentati [EXT E1]. Nuovi SONAME/runtime e baseline non inclusi nella qualifica Slint; host .pc4.1 assente |
| Windows x86_64 | DESIGN Windows10/Server2016+; HISTORY runner windows2025/MSVC/CRT statico | WebView2 [EXT E2], disponibilità sui PC/offline/versioni da provare; runner2025 non prova installazione Windows10 pulita |
| N-20/QG-5 | CLI policy ristretta, UI policy separata; SQLite statico approvato | mantenere gate CLI; nuove librerie/plugin/processi WebView UI richiedono politica e scanner/avvio nativi propri |
| Portable vs installer | CLI/UI archivi tar.gz/ZIP; UI qualificata come artifact CI, non nuova release pubblicata | archivio non installa automaticamente WebView/prerequisiti. Installer online/offline o runtime ridistribuito sono opzioni da ratificare, nessuna scelta implicita |
| WebView2 offline | nessun runtime WebView qualificato da Lintra | [EXT E6] Tauri documenta offlineInstaller e fixedRuntime; costi di aggiornamento/licenza/dimensione/installazione da valutare, nessuna installazione ora |
| N-23/release | solo CI, hash/provenienza/versione, release immutabili, WindowsMSVC; no firma Authenticode ratificata | nuovi asset frontend/runtime e licenze/checksum; separare dati normativi dal frontend; ridefinire prova di riproducibilità su pipeline scelta |
| Node/TS/React | non presenti nel prodotto, solo Node/npm host | pin/versioni/lock/CI/audit licenze/supply-chain e build offline da definire; nessuna dipendenza selezionata qui |
| Rust/Tauri/plugin | Rust pinned, Tauri assente | major2 come destinazione, patch/MSRV/plugin/features/lock NON scelte; nessuna affermazione di compatibilità compilatore/linker |
| N-24/QG-7 | Slint vincolante/software/font/token/AboutSlint, confronto nativo RGB identico | sostituzione richiede ratifica; WebKitGTK e Chromium non promettono identità pixel-per-pixel. Criteri funzionali/visivi/DPI/font/OS da ridefinire |
| Sicurezza WebView | nessuna capability/CSP/IPC nel prodotto | threat model, autorizzazione comandi, CSP/percorso/URL/devtools/cache da approvare e verificare con dati sintetici |

Fonti `[EXT]` consultate il30/10/2026, documentazione Tauri major2 (non snapshot di release adottata):

- **E1** [Prerequisites](https://v2.tauri.app/start/prerequisites/): build Linux richiede WebKitGTK4.1 e altri pacchetti nativi; l'elenco dipende dall'OS. Non si traduce la lista build in una policy runtime automaticamente.
- **E2** [Webview Versions](https://v2.tauri.app/reference/webview-versions/): Windows usa WebView2 basato Chromium, Linux WebKit tramite webkit2gtk; la versione del motore varia con l'ambiente. Il minimo OS di Lintra resta una qualifica propria.
- **E3** [Capabilities](https://v2.tauri.app/security/capabilities/): capability per window/platform; i comandi applicativi registrati sono disponibili per default alle finestre/WebView, salvo configurazione specifica tramite AppManifest. Necessario censire anche i propri comandi, non solo plugin.
- **E4** [CSP](https://v2.tauri.app/security/csp/): protezione attiva solo se impostata nella configurazione; policy da limitare agli asset/origini necessari. Non è una configurazione Lintra già esistente.
- **E5** [Calling Rust](https://v2.tauri.app/develop/calling-rust/): comandi serializzati e canali sono meccanismi disponibili; non garantiscono da soli i limiti di memoria/backpressure Lintra e non serializzano automaticamente i trait/handle attuali.
- **E6** [Windows Installer](https://v2.tauri.app/distribute/windows-installer/): contempla provisioning WebView2 offline e fixed runtime. Offline, distribuzione portable e aggiornamenti devono essere verificati nel contesto scelto, non assunti dall'esistenza dell'opzione.

## 11. Verifiche eseguite

Ambiente Linux di §3, fotografia HEAD+WIP; nessuna prova locale Windows. Gli artefatti Cargo sono confinati con `CARGO_TARGET_DIR=/tmp/lintra-preflight/target`; i log sono temporanei sotto `/tmp/lintra-preflight`. Questi percorsi non identificano utenti. `--offline` evita installazioni/download; `--locked` impedisce modifiche del lock. Esiti e hash log in appendice; i log temporanei non sono il deliverable necessario al lettore.

| ID | Comando esatto (dalla root) | Esito / evidenza condensata |
|---|---|---|
| V-01 | `cargo metadata --locked --offline --no-deps --format-version 1` | PASS exit0: membri lintra/lintra-ui e target di §3, default solo root |
| V-02 | `cargo fmt --all -- --check` | PASS exit0, nessuna riscrittura |
| V-03 | `cargo tree --locked --offline -p lintra --depth 1` | PASS exit0, nessun Slint diretto, clap nel package condiviso |
| V-04 | `CARGO_TARGET_DIR=/tmp/lintra-preflight/target cargo check --all-targets --locked --offline` | PASS exit0, default package; non link/test/GUI |
| V-05 | `CARGO_TARGET_DIR=/tmp/lintra-preflight/target cargo clippy --all-targets --locked --offline -- -D warnings` | PASS exit0, default package e WIP compilato; non chiude T-PERSIST08 |
| V-06 | `CARGO_TARGET_DIR=/tmp/lintra-preflight/target CARGO_BUILD_JOBS=2 cargo check --workspace --all-targets --locked --offline` | PASS exit0, include UI e target test/bench, nessun link/release |
| V-07 | `CARGO_TARGET_DIR=/tmp/lintra-preflight/target CARGO_BUILD_JOBS=2 cargo clippy --workspace --all-targets --locked --offline -- -D warnings` | PASS exit0, include adapter Slint |
| V-08 | `env -u LINTRA_UPDATE_GOLDEN CARGO_TARGET_DIR=/tmp/lintra-preflight/target CARGO_BUILD_JOBS=2 TMPDIR=/tmp/lintra-preflight cargo test -p lintra --locked --offline --test golden --test atomic_report --test request_preflight --test diagnostic_rendering -- --test-threads=1` | PASS exit0: atomic_report12 + diagnostic_rendering6 + golden13 + request_preflight6 = **37 pass, 0 fail, 0 ignored**. Solo queste quattro suite, non suite globale |
| V-09 | `gh run view 41036453232 --repo UraganoAngelina/lintra --json headSha,status,conclusion,jobs,url` | FAIL exit1: error connecting to api.github.com. Nessun avvio CI, nessun risultato remoto inventato |
| V-10 | `node --version`; `npm --version`; `pkg-config --modversion fontconfig`; `pkg-config --modversion webkit2gtk-4.1` | PASS/exit0 per primi tre con versioni §3; FAIL/exit1 per .pcWebKitGTK assente |
| V-11 | Git baseline/finale + hash file preesistenti e lock/font/reference | PASS controllo: sole variazioni attribuibili all’audit nel report; drift esterno documentato in §15 |
| V-12 | `CARGO_TARGET_DIR=/tmp/lintra-preflight/target cargo metadata --locked --offline --all-features --format-version 1 --filter-platform x86_64-unknown-linux-gnu` + `scripts/check_workspace.py::check` sul JSON in memoria | PASS exit0: grafo nativo risolto, closure core89 package senza Slint/openssl-sys, clap presente; nessuna esecuzione del main script o download |

Prevalutazione sicurezza test: `tests/atomic_report.rs:20` crea directory sintetiche nel temp e le rimuove; golden usa Check con `LINTRA_UPDATE_GOLDEN` rimosso, fixture synthetic/semantic/DSL/derived e helper temporanei (`tests/support/golden.rs:9`, `tests/support/golden_tests.rs:16`); request_preflight e diagnostic_rendering usano valori sintetici/in memoria. Nessun DB utente.

| Verifica ulteriore | Stato e motivo |
|---|---|
| `cargo test --locked` completo/workspace/allfeatures | NOT_RUN: test session_approval/history_audit/SQLite e capture UI scrivono in target tramite CARGO_MANIFEST_DIR fisso, contro il perimetro di sole scritture temp esterne. Nessuna modifica ai test per aggirarlo |
| Test visivo/Signature/bootstrap/results_window/export_window | NOT_RUN: capture nel repository; `ui/tests/visual.rs:186`, `ui/README.md` e fixture analoghe. Reference hash verificate, non rigenerate |
| Runtime worker/export UI e font negativo | NOT_RUN: check/Clippy non eseguono quelle suite; HISTORY qualificata nei dossier, non promossa a TEST corrente |
| SQLite crash/restart/contesa/sessioni/WIP08 | NOT_RUN: harness con root target fissa; test presenti, nessun archivio aperto. Passati storici non provano assenza race |
| Build/link release/verify-artifact nuovi, WindowsGNU/Wine/MSVC | NOT_RUN: nessuna release/gate nativo richiesto qui; Windows reale non disponibile. check Linux non sostituisce linking o Windows |
| Benchmark estesi/scorrimento/memoria | NOT_RUN, misure correnti NOT_MEASURED; si conservano solo le evidenze storiche qualificate sotto |
| Tauri/React build/test/picker/security/installer | NOT_RUN: progetto e dipendenze assenti; crearli violerebbe lo scope |

**CI:** il JSON conservato T-P16-08-native-ci dichiara sette job success su `aa7be4f2fc76cf78d0f03fdcaf6d2b653c130829`, run [41036453232](https://github.com/UraganoAngelina/lintra/actions/runs/41036453232): Linux, MSVC, due packaging, confronto JSONL, due pixel. `[TEST] git diff aa7be4f HEAD --stat` mostra solo documenti/indice/status/UI README (11 file); sorgenti/config/workflow eseguibili invariati. È una relazione precisa con HEAD, **non una CI letta su HEAD**, né copertura delle modifiche WIP. Manifest archivi conserva glibc2.39/Windows2025, SHA e startup/PATH vuoto; artifact non riscaricati in questo audit.

**Flakiness nota:** contesa SQLite con timeout zero può produrre zero commit in un tentativo senza violare CAS. Il test corretto in `4e44703` forza SHARED e ritenta soltanto dopo assenza verificata; tentativi falliti precedenti documentati in acceptance UI08/status. Non si interpreta il success storico come assenza di race su ogni filesystem/power-loss. Nessuna misura hardware power-loss effettuata.

**Prestazioni storiche, non confrontabili automaticamente con Tauri:** T-P17-01 misura sintetici10k/100k/1M, due trace e scenari valid/error-heavy/projection, CountSink; timer include service/preflight/compile/due passate, non JSONL/XLSX. Metrica heap Rust richiesto, non RSS (`docs/verification/T-P17-01-benchmark.md:14`). Dossier PERSIST05 contiene misure SQLite release10k/100k/300k, lookup/commit/restart e picco processo C+Rust, cache2MiB (`T-PERSIST-05-local-benchmark.json`); non aprono dati reali. UI results/signature misura300k/1M e cache128 nei dossier, ma raw capture/log Signature eliminati dal refactoring non sono ricreati qui. Nessun numero nuovo di throughput/RSS/p95 né soglia regionale dedotta; confronto futuro richiede stesso hardware, build, dati/metodo e metriche separati.

## 12. Matrice KEEP/ADAPT/REPLACE/REMOVE_LATER/UNRESOLVED

Natura proposta dell'impatto, **non autorizzazione a modificare**. Il percorso futuro deve mantenere le regressioni del core/CLI.

| Classe / modulo | File/simbolo | Natura del cambiamento possibile | Rischio | Test da preservare/richiedere | Prerequisito |
|---|---|---|---|---|---|
| KEEP dominio/input/schema/validator/engine/context | src/domain, input, schema, validators, engine, context; CompiledFlowSchema/FieldPipeline/FlowValidator | conservare algoritmo/byte/ordine e contratti, nessuna norma nel frontend | alto normativo se alterati | unit/golden/Full/memory, compiled non forgiabile | scope core invariato |
| KEEP writer/report/CLI | src/report, src/cli, src/main.rs | stessi formati/sink/exit/mode/bundle | dati/false successi | byte golden/cross-OS/XLSX/CLI processo/atomic | N-01–22/25 |
| KEEP backend persistente/protezione | src/persistence + application/session | preservare trait/HMAC/CAS/approvazione distinta | alto privacy/durabilità | SQLite nativo/sessioni/ambiguità/ACL/restart | ADR001/002, nessuna UI storica implicita |
| ADAPT configurazione | ui/src/analysis.rs, controller.rs | isolare intenti/metadata dal binding Slint, nuovi DTO da definire | schema/path/race | analysis_configuration/schema_binding, filesystem scope | decisione IPC/versioni |
| ADAPT worker/lifecycle | ui/src/worker.rs, worker_controller.rs | semantica bounded/terminale/cancel/ownership riutilizzabile, dispatcher da sostituire | memoria/chiusura/falsi successi | worker terminal/cancel/panic/disconnect/shutdown | run ownership e threading approvati |
| ADAPT risultati | ReportIndex/ResultsFilter/JsonlModel/project, results/controller | scansione/query restano Rust; togliere Model/SharedString/Timer; paging pubblico manca | payload/PHI/offset/memoria | malformed/scope/count/filters/selection300k/1M | contratto query e limiti |
| ADAPT domini | field_guide + ResultsController/reference_job | funzioni core riusabili, presentazione/cursori bridge da precisare | dominio errato/dataset mutato | schema_binding/domain_reference/range oltre-u128 | snapshot run e DTO |
| ADAPT export/provenienza | export_report/ExportJob/export_controller | writer invariati, stato/dialog bridge nuovo | overwrite/partial publication | exports/export_window/errore spazio/cleanup | permessi path/error mapping |
| REPLACE renderer/binding | sei ui/ui/*.slint, ui/src/lib.rs include_modules, shell/controllers UI-specific | React/TS markup/stato/stili da progettare, non trasposizione automatica | focus/DPI/semantica e licenze | nuova parità funzionale/visiva multipiattaforma | N-24/QG-7 ratificati |
| ADAPT asset/design | fonts/OFL, tokens e docs/ui-design-documentation | font riusabili con licenza; token convertibili, logo codice da ridisegnare in nuovo renderer | font fallback/contrasto | hash/font loading/layout due temi/scaling | strategia asset statici |
| ADAPT CI/tooling | Cargo workspace, ci/ui-package/release, scripts/check_workspace/ui_package, xtask artifact | nuova toolchain e product policy UI, CLI invariata | supply-chain/runtime/release | build/link/installer/offline/native/artifact; stesso-SHA gate | policy runtime e distribuzione |
| REMOVE_LATER legacy Slint | slint/slint-build lock subtree, ui/build.rs, Slint-only tests/widget/license | eliminare solo dopo parità/gate e assenza runtime legacy | rimozione anticipata audit/attribuzioni | grafo/artifact/licenze/baseline preservate | ratifica, rollback e parity evidence |
| KEEP storico documentale | spikes/ui-prototype, mockup/reference storiche/Git | non scambiare prototipo per prodotto; rimozione non necessaria all'audit | perdita evidenze | provenienza/hash reference | organizzazione ee33d64 preservata |
| UNRESOLVED WIP08 | history_audit.rs, occurrence APIs, tests e status dirty | API locale non qualificata, base piano da fissare | drift concorrenziale/gate | riesame su baseline scelta + CI nativa | DEC-MIG-06 |
| UNRESOLVED future feature | UI storico/CAD/Full/sort/settings durevoli | nessun equivalente Slint da sostituire | scope creep/norma/sicurezza | acceptance future distinta dalla parità | decisione scope esplicita |

## 13. Registro CNF-MIG-*, DEC-MIG-*, RSK-MIG-*

Registro **preliminare nel solo report**. Nessuna voce modifica N-24, ratifica eccezioni o apre task ufficiali. “Blocca il piano?” distingue un piano definitivo da una bozza condizionata: l'audit rimane utilizzabile anche prima delle risposte. Confidence HIGH indica forza dell'evidenza del fatto, non probabilità numerica del rischio.

| ID | Categoria | Fatto osservato | Fonte/evidence anchor | Impatto | Opzioni senza scelta implicita | Decision maker | Blocca il piano? | Confidenza |
|---|---|---|---|---|---|---|---|---|
| CNF-MIG-01 | architettura | N-24/ARCH39D/DET162 prescrivono Slint, destinazione richiesta Tauri | piano:113; ARCH:1725; DET:4086 | framework/renderer/attribuzioni e layer | ratifica sostituzione delimitata; mantenere baseline finché deciso | autore/architetto | sì, ratifica del target implementabile; non ricognizione | HIGH |
| CNF-MIG-02 | packaging | allowlist UI Slint e autosufficienza attuali non qualificano WebView | xtask/artifact:79; ui runtime list; E1/E2/E6 | N-20/N-19/QG-5/offline | prerequisiti gestiti; installer; runtime ridistribuito dove consentito, con politica esplicita | autore/release/security | sì, distribuzione definitiva | HIGH |
| CNF-MIG-03 | qualità visiva | renderer software/confronti RGB nativi correnti vs motori WebView diversi | ARCH:1740; ci.yml:151; E2 | N-24/QG-7 e test | nuovi criteri per-OS e funzionali; target visivo qualificato con limiti dichiarati | autore/design/QA | sì, acceptance definitiva | HIGH |
| CNF-MIG-04 | evidenze documentali | tabella G-PERSIST “solo modulo vuoto” e sezioni CI/DEVELOPMENT storiche divergono da SQLite/PERSIST07/release attuali | status:2428 e :2597; CI:16; ADR002; sqlite | QG-8, falso riavvio/duplicazione | usare frontiera/dossier più recenti con relazione Git; riconciliazione futura registri | autore/maintainer | no alla bozza; sì se assunti gate globali | HIGH |
| CNF-MIG-05 | confine dipendenze/build | N-02 esclude clap dal core; package library+CLI lo dipende; build.rs invoca Git contro divieto generale ARCH build.rs | Cargo:46; main:1; build.rs:2; ARCH:1648 | ambiguità grafo vs modulo, QG-3/5 | chiarire perimetro contrattuale; eventuale separazione package da approvare; preservare eccezioni documentate | autore/architetto | no inventario; sì a dichiarazione “core privo di clap” | HIGH |
| CNF-MIG-06 | baseline concorrente | tre file WIP e un dossier variano esternamente durante l'audit, dopo i controlli registrati | impronte/mtime §15 | non esiste un unico hash finale coperto dai controlli dell'intera sessione | fissare successivamente revisione qualificata; mantenere qui doppia fotografia, senza inseguire o correggere lo sviluppo | autore/maintainer | sì alla baseline esecutiva; no alla ricognizione | HIGH |
| DEC-MIG-01 | decisione target | nessuna ratifica migrazione trovata oltre incarico di audit | richiesta + CNF-MIG-01 | scope ADR/N-24 | ratifica React/TS/Tauri2 con confini; rinvio | autore | sì prima piano eseguibile | HIGH |
| DEC-MIG-02 | runtime/deployment | minimo OS provato solo runner attuali, nessun WebView install probe | acceptance UI08:1; E1/E2/E6 | offline/portable/prerequisiti N-19/20/23 | dichiarare OS e prerequisiti; installer/offline/fixed runtime valutati | autore/release | sì per baseline e gate | HIGH |
| DEC-MIG-03 | sicurezza | report e dettaglio contengono dati sensibili; nessun IPC/CSP oggi | results::project; errors:59; E3/E4 | N-31/threat model/permissions | DTO minimizzati; accesso backend controllato; policy log/cache/devtools esplicita | autore/security/privacy | sì al contratto bridge | HIGH |
| DEC-MIG-04 | contratto runtime | API Rust sync/Arc/borrowed, nessun DTO IPC/query paging neutra | request:34; results:206; session:692 | run lifecycle/errors/stream/query | adapter con contratti versionati da formalizzare; riuso logica neutra, non widget | architetto con autore per variazioni | sì alle firme definitive, no analisi | HIGH |
| DEC-MIG-05 | acceptance | criterio QG-7 Slint non trasferibile automaticamente | CNF-MIG-03; visual:31 | parità UX e non regressione | separare criteri per piattaforma/scala e semantica; soglie motivate | autore/QA/design | sì a gate di completamento | HIGH |
| DEC-MIG-06 | baseline | WIP T-PERSIST08 cambia API export senza commit/CI | git baseline §3; history_audit::HistoricalAuditService | stabilità riferimenti/dipendenze piano | pianificare da HEAD con WIP escluso; fissare futura revisione qualificata08 | autore/maintainer | sì a baseline esecutiva, non alla bozza | HIGH |
| DEC-MIG-07 | scope/parità | Slint priva di Full/storico/CAD/sort/settings durevoli | MainWindow; PRODUCT.md:15 | scope creep e false capacità | parità UI corrente; estensioni separate con prerequisiti reali | autore/prodotto | sì solo se il piano intende includerle | HIGH |
| DEC-MIG-08 | supply-chain | Node host non pinnato nel repo, Tauri/React/plugin assenti | V-01/10, Cargo lock | N-26/build/lock/licenze/security | qualificare versioni/features/toolchain nuove; non scegliere in audit | autore/architetto/release | sì prima dipendenze e CI definitive | HIGH |
| DEC-MIG-09 | dati reali | cifratura/retention/chiavi/backup operativi restano aperti | ADR002:23; DET:4545 | privacy dati reali, non renderer | decisioni operative e prove; restare sintetico fino ad allora | autore/security/privacy | non blocca piano su sintetici; sì uso reale | HIGH |
| RSK-MIG-01 | streaming/prestazioni | indice O(n), scansione filtri e cache128, sync row I/O | results/mod:113/:194/:270 | duplicazione RAM Rust+JS/UI stall | pagine/limiti/virtualizzazione/benchmark qualificati nel piano; nessuna DataGrid scelta | architetto/QA | gate QG-6 da definire; no audit | HIGH |
| RSK-MIG-02 | correttezza/race | ultimo report conservato e stato corrente separato; cancel cooperativo | worker_controller:28; worker Drop; results open | risultato sbagliato/false successi dopo eventi tardivi | binding stabile run/vista/offset; prove cancel/new-run/close/disconnect | architetto/QA | sì contratto lifecycle | HIGH |
| RSK-MIG-03 | privacy/injection | actual/context/path entrerebbero nel frontend; contenuti input non affidabili | results::project; export_controller::poll; E3/E4 | XSS/esfiltrazione/log/devtools/cache | rendering testo, origini/permissions/CSP/scope/log policy da validare | security/autore | sì contratto sicurezza | HIGH |
| RSK-MIG-04 | dati/file | export due file e DB/report senza atomicità comune | export.rs::export_report; session::approve_commit | salvataggio parziale/falso rollback | mantenere esiti parziali e commit ambiguo; non promettere transazione distribuita | architetto/QA | no bozza, sì semantica finale | HIGH |
| RSK-MIG-05 | CI/cross-OS | live CI fallita, snapshot verde solo antenato | V-09; native-ci JSON; diff aa7be4f HEAD | baseline WIP non qualificata, minimi OS non testati | nuove prove su SHA fissato nella fase autorizzata; non riusare vecchio verde | maintainer/release | sì all'avvio qualificato, non al censimento | HIGH |
| RSK-MIG-06 | normativa | G0 con rinvii, SIOC audit ≠ divieto cross-run, CAD runtime assente | status:2434; PERSIST spec:29; CAD tasks | conformità falsamente certificata | scope capability esplicito, errori unsupported, decisione normativa separata | autore/referente normativo | solo per feature normative aggiuntive | HIGH |
| RSK-MIG-07 | storage/concorrenza | timeout0 e commit ambiguo; load_snapshot owned O(n) | sqlite; acceptance UI08; ADR002:42 | retry distruttivo, grandi snapshot, race | conservare CAS/resolve_commit e limiti; misure/ripetizioni native | architetto/QA | gate persistenza, non parity sola UI | HIGH |
| RSK-MIG-08 | cleanup/recovery | temp/export possono contenere PHI; Drop non prova pulizia dopo kill | worker::AnalysisReport; export.rs; ADR002 | dati residui, file export/log/cache | policy temp/retention/recovery dati reali e WebView da approvare | autore/security | sì uso reale | HIGH |

Gate interessati dal registro: CNF-MIG-01/DEC-MIG-01 → QG-3/7/8; CNF-MIG-02/DEC-MIG-02/08 → QG-2/5/8; CNF-MIG-03/DEC-MIG-05 → QG-2/7; CNF-MIG-04/05 → QG-3/5/8; CNF-MIG-06/DEC-MIG-06/RSK-MIG-05 → QG-1/2/8; DEC-MIG-03/09 e RSK-MIG-03/08 → N-31 e gate privacy; DEC-MIG-04/RSK-MIG-01/02/04 → QG-3/4/6; DEC-MIG-07/RSK-MIG-06/07 → G0/G-PERSIST/G-CAD-RUNTIME e QG-2/3/4/6 secondo scope. Nessuno di questi gate è chiuso dal registro.

Censimento dei registri preesistenti (nessuna modifica/riapertura automatica):

- **N-01…31**, tutti presenti nel piano: 01 motore/schema; 02 library/adapters; 03 versioni esterne; 04 raw/compiled; 05 byte; 06 streaming; 07 condizioni non mutabili; 08 ordine Vec/lookup; 09 AST; 10 Unknown; 11 esiti/tecnici; 12 determinismo; 13 niente note automatiche/LLM; 14 boundary dati mancanti; 15 due passate/projection; 16 log separato; 17 coordinate; 18 missing per campo; 19 Linux/Windows; 20 autosufficienza; 21 bundle ibrido; 22 formati byte/testo/UTC; 23 release CI; 24 Slint; 25 audit; 26 dipendenze autorizzate; 27 G0; 28 persistenza core; 29 preview/approvazione/immutabilità; 30 SIOC intra-run≠inter-run; 31 privacy. Stato: prescrizioni vigenti, eccezioni solo ratificate, impatti §5/9/10 (`piano:90–120`).
- **DEC-01…18** del piano: 01 anticipo P1/P2, 02 codici, 03 nome, 04 MVP sono registrati risolti; 05 note normative ha risoluzioni puntuali e rinvii, non approvazione indiscriminata; 06 XLSX qualificata da P12; 07 font/Slint da P16; 08/09/11 release da DEC-RELEASE-SETUP-01 e ratifica Linux; 10 spike Slint chiuso nello scope00; 12 BOM/riga ratificati; 13 encoding runtime esplicito (UI solo UTF-8); **14 soglie reali**, **16 max_errors**, **17 parallelismo** ancora aperti/rinviati; 15 registro voce per voce, non autorizzazione globale; 18 catalogo ratificato. Il piano §13.1 conserva vecchie diciture “Aperta”: fonti successive in status/ratifiche prevalgono nel merito senza cancellare la storia.
- Decisioni ulteriori censite in status: `DEC-AUTHORING-FORMAT-01`, `DEC-SIOC-MINIMAL-01`, `DEC-05-SIOC-ID-2026`, `DEC-05-SIOC-OPTIONAL-PARENT`, `DEC-05-CAD-ICPC-000`, `DEC-05-CAD-VALUTAZIONE-INIZIALE`: contratti e risoluzioni storiche di authoring/G0; **DEC-SIOC-HIST-01 resta aperta**. Non diventano capacità UI o runtime CAD per il solo nome.
- **CNF-01…11**, `CNF-LINTRA-SOURCE-01`, `CNF-PERSIST-01` sono DOCUMENTED_RESOLVED nella matrice: MVP, ordine DSL/reference, Slint, workspace/DI, codici, naming, request, XLSX postanalisi, CI, RequiredUnless e JSONL UI. Ratifiche successive: `CNF-DOMAIN-01`, `CNF-COMPILER-01`, `CNF-FORMAT-01`, `CNF-AUDIT-01`, `CNF-ALIGNMENT-01` (chiusa come estensione fisica non implementata), `CNF-RECORD-01`, `CNF-RUN-SCOPE-01`, `CNF-SERVICE-DI-01`, `CNF-LINTRA-DSL-01`, `CNF-REFERENCE-FORMAT-01`, `CNF-SIOC-REFERENTIAL-G0-01`, `CNF-ARTIFACT-LINUX-01`, `CNF-BUNDLE-AUDIT-01`: scope e limiti ratificati in AGENTS/status/dossier. Non confondere vecchi paragrafi BLOCKED con frontiera attuale. Nuove incompatibilità della migrazione restano solo CNF-MIG sopra.
- **RSK-01…14** originali (`piano:894–907`): fonti mancanti, spike/API GUI, XLSX instabile, font cross-OS, costi CI, CRLF fixture, Linux ABI, deriva contesto, memoria contesto/storage, dipendenza da singolo autore, minimo Windows, unsigned executable, ridistribuibilità fonti, supply-chain. Tutti censiti: 01/04/06/07/09/11/13/14 rimangono materialmente rilevanti al futuro piano; 02 si ripresenta come rischio nuovo stack; 03 qualificata per versione attuale; 05/08/10/12 gestionali/operativi non diventano difetti implementativi. Nessuna stima di costo o probabilità inventata.
- **QG-1…8** tutti censiti in §4. Cambiare renderer non chiude QG-7, cambiare allowlist non dimostra QG-5, una nuova richiesta non certifica QG-2.

## 14. Readiness e prerequisiti del futuro Master Migration Plan

**Già sufficiente:** mappa del prodotto e del grafo Cargo; pipeline core/CLI/UI; schermate e assenze; modalità realmente eseguibile dalla UI; semantica di outcome/completezza/cancellazione/errori; ownership spool/indice/schema/reference; backend persistente reale e separazione approvazione; inventario dei punti di contatto e asset; policy build/release e prove storiche con SHA. Questo permette a un architetto esterno di motivare ADR e definire successivamente identificativi, dipendenze, gate/rollback senza immaginare un nuovo motore.

**Informazioni mancanti:** baseline esecutiva definitiva post-WIP, CI riletta su quella revisione; versione patch Tauri/React/TS/Node/package manager/plugin e relativa qualifica; OS minimi di consegna e provisioning runtime; DTO/permissions/CSP/threat model; acceptance visiva WebView e limiti quantitativi IPC/memoria; decisioni operative su dati reali. Nessuna di queste lacune è riempita con una dipendenza o una nuova norma.

Ordine logico dei chiarimenti, **non roadmap implementativa**:

1. Autore conferma perimetro di sostituzione N-24 e parità UI attuale, separando estensioni future (DEC-MIG-01/07).
2. Fissare revisione di riferimento e trattamento del WIP08, con gate coerenti; non usare “117DONE” come prova tecnica di ogni feature (06).
3. Precisare piattaforme, portable/installer/offline e responsabilità runtime UI, mantenendo contratto CLI e dati indipendenti (02, CNF-MIG-02).
4. Chiarire threat model e confine dati/autorizzazioni/lifecycle/query; quindi formalizzare DTO e semantiche IPC con ownership backend, senza serializzare direttamente gli handle attuali (03/04).
5. Concordare acceptance multipiattaforma/font/visiva/performance e qualificazione nuove toolchain/dipendenze/licenze (05/08). Dati reali restano subordinati alle decisioni privacy/storage già aperte (09).

Il successivo piano dovrà poter esprimere rollback/coabitazione temporanea e prove di parità prima della rimozione legacy, senza modificare golden normative o diminuire i gate per ottenere un verde. Qui non si fissano milestone di migrazione, task atomiche, tempi/effort o schema definitivo del bridge. **Fine audit: nessuna task successiva.**

## 15. Appendice — Evidence index

Riferimenti relativi alla root osservata; le righe sono quelle del working tree del30/10/2026. Dove è indicato un simbolo, l'anchor include il file e rimane leggibile senza repository grazie alle descrizioni nel corpo. Le prove di comandi sono riassunte in §11; questa appendice consente verifica successiva senza dati personali.

| Gruppo | Evidence anchor / contenuto verificabile |
|---|---|
| Governance | `AGENTS.md:1` GOV; mandati successivi nello stesso file; `MASTER_AGENT_PROMPT.md:1`; `docs/MASTER_IMPLEMENTATION_PLAN.md:924` protocollo e `:1014` handoff |
| Contratti | `docs/ARCHITECTURE.md:71` motore/schema, `:135` NFR, `:1508` piattaforme, `:1621` packaging, `:1725` Slint, `:1904` recovery, `:1922` persistenza |
| Runtime design | `docs/DETAILED_DESIGN.md:69` library/raw/stream, `:279` dipendenze, `:1862` esiti, `:2369` progress/cancel/service, `:2798` UI, `:3221` cancel, `:4086` boundary, `:4319`/`:4458` addenda, `:4501`–`:4553` persistenza |
| Piano/stato | `docs/MASTER_IMPLEMENTATION_PLAN.md:90`, `:420`, `:698`, `:714`, `:786`, `:844`, `:890`; `IMPLEMENTATION_STATUS.md:5`, `:28`, `:2417`, `:2465`, `:2484`, `:2538`, `:2597`, `:2881`, `:3221`; `ATOMIC_TASKS_INDEX.md:8`; `TASK_DEPENDENCY_MAP.md:1` |
| Prompt letti | `TASK_PROMPTS/T-P15-01.md`…`T-P15-07.md`, `TASK_PROMPTS/T-P16-00.md`…`T-P16-08.md`: obiettivi originali e limiti, non avviati; `docs/verification/T-PERSIST-08-preflight.md:1` stato locale |
| Cargo/entry | `Cargo.toml:1`, `ui/Cargo.toml:1`, `xtask/Cargo.toml:1`, `.cargo/config.toml:1`, `rust-toolchain.toml:1`, `build.rs:2`, `src/main.rs:1`, `src/lib.rs:83`, `ui/src/main.rs:4`, `ui/src/lib.rs:2` |
| Schema/input | `src/schema/registry.rs:50`/`:80`; `src/schema/compiler.rs::SchemaCompiler::compile`; `src/schema/compiled.rs:23`/`:55`/`:220`; `src/schema/repository.rs::SchemaCatalog`; `src/schema/persistence_profile.rs:23`/`:380`; `src/input/reader.rs:14`/`:33`; `src/input/record.rs::RecordView`; `src/input/audit.rs::InputAuditSession` |
| Core/contesto | `src/engine/field_pipeline.rs::FieldPipeline`; `src/engine/record_validator.rs::RecordValidator`; `src/engine/flow_execution.rs:138`/`:176`; `src/engine/expression_evaluator.rs::TruthValue`; `src/context/store.rs:56`; `src/context/projected_record.rs::project_record` |
| Service/report | `src/application/request.rs:34`; `src/application/service.rs:248` e `ValidationService::{validate,validate_with_parameters,validate_full}`; `src/application/progress.rs:5`/`:14`/`:25`; `src/application/cancellation.rs:10`; `src/domain/summary.rs:11`/`:37`/`:61`/`:112`; `src/domain/errors.rs:59`; `src/report/sink.rs::ValidationSink`; `src/report/atomic.rs:94`/`:169`; `src/report/replay.rs::replay_canonical_report`; `src/report/xlsx.rs::XlsxReportWriter`; `src/report/summary.rs` |
| Persistenza | `docs/PERSISTENCE_CORE_SPEC.md:1`; `docs/verification/ADR-PERSIST-002-ratification.md:1`; `src/persistence/mod.rs:242`/`:327`/`:338`; `src/persistence/protection.rs:64`/`:153`; `src/persistence/sqlite/mod.rs:112`/`:182`/`:270`/`:287`; `src/persistence/sqlite/security.rs`, `security/windows.rs`; `src/application/session.rs:227`/`:515`/`:701` e `ApprovalPreview::approve_commit`; working tree `src/application/history_audit.rs:132`/`:378` |
| Configurazione UI | `ui/src/analysis.rs:16`/`:327` e `AnalysisBundle`, `InputPreview::inspect`, `AnalysisSelection::prepare`; `ui/src/controller.rs::ConfigurationController`; `ui/ui/analysis.slint:7`/`:167` |
| Worker UI | `ui/src/worker.rs::ChannelObserver`, `::execute`, `::WorkerHandle`; `ui/src/worker_controller.rs:28` e `WorkerController::{start,poll,shutdown}`; `ui/src/main.rs` ordine shutdown |
| Risultati/domini | `ui/src/results/mod.rs:65`/`:113`/`:194`/`:206` e `JsonlModel::read_row`, `project`; `ui/src/results/diagnostic.rs:8`; `ui/src/results/controller.rs:20`/`:99`/`:122`/`:210`/`:244`/`:274`/`:428`/`:508`; `src/engine/field_guide.rs:111`/`:233`/`:243`; `ui/ui/results.slint:19`/`:160` |
| Export/Info | `ui/src/export.rs::export_report`, `::ExportFailure`, `::ExportJob`; `ui/src/export_controller.rs:34`/`:151`/`:233`; `ui/ui/export.slint:4`; `ui/ui/main.slint::AboutSlint` |
| Fonti normative locali | `docs/normative/{cad,sioc}/{README,rules-and-joins}.md`, `docs/normative/DECISIONS.md:1`, `reference-datasets.md:1`, `INPUT_INVENTORY.md:1`; `docs/schema-authoring/FORMAT_COMPATIBILITY.md:1`; `schemas/sioc/2026-04-20.structural-minimal.2/README.md:1`; `bundles/embedded/README.md:1`; `docs/verification/T-P14-01-xlsx-importer.md:9`; `docs/ui-design-documentation/DIAGNOSTIC_REQUIREMENTS.md:1` |
| Asset e design | `ui/build.rs:1`, `ui/build_support/fonts.rs:5`, `ui/ui/tokens.slint::Tok`, `ui/ui/controls.slint`, `ui/ui/main.slint:10`/`:113`; `docs/ui-design-documentation/{DESIGN,PRODUCT,LINTRA_DESIGN_DIRECTION}.md:1`; `ui/tests/reference/README.md:1`; `ui/licenses/SLINT-ROYALTY-FREE-2.0.md`, `ui/ui/fonts/OFL.md` |
| Ratifiche | `docs/CONFLICT_RESOLUTION_MATRIX.md:7`/`:32`; `docs/verification/{DEC-18,CNF-REFERENCE-FORMAT-01,CNF-SIOC-REFERENTIAL-G0-01,CNF-ARTIFACT-LINUX-01,CNF-BUNDLE-AUDIT-01,DEC-RELEASE-SETUP-01}-ratification.md:1`; `docs/DEPENDENCIES.md:1`/`:204` |
| Build/CI/release | `docs/DEVELOPMENT.md:1`; `docs/CI.md:1`; `docs/RUNBOOK_SCHEMA_UPDATE.md:1`; `.github/workflows/ci.yml:1`, `ui-package.yml:1`, `release.yml:1`; `scripts/check_workspace.py:15`; `scripts/release.py`; `scripts/ui_package.py`; `xtask/src/artifact.rs:6`/`:79`/`:228`; `ui/linux-runtime-libraries.list:1` |
| Test di core | `tests/{golden,atomic_report,request_preflight,diagnostic_rendering}.rs`; `tests/support/golden.rs:9`; `tests/support/golden_tests.rs:16`; `tests/full_service.rs`, `tests/sioc_referential.rs`, `tests/session_approval.rs:450`, `tests/history_audit.rs:358` (WIP), `src/persistence/sqlite/tests.rs` |
| Test UI | `ui/tests/{analysis_configuration,bootstrap,font_contract,worker,worker_window,results,results_window,domain_reference,schema_binding,exports,export_window,signature,visual,packaging}.rs`; `visual.rs:186` scritture fisse, `results.rs::three_hundred_thousand_and_one_million_rows_keep_only_offsets_and_a_bounded_cache` |
| Evidenza storica UI/P15 | `docs/verification/T-P16-08-{acceptance,packaging}.md:1`, `T-P16-08-{native-ci,qualified-archives,style-preservation}.json:1`; dossier T-P16-00…07 e T-P15-01…07; link run in §11 |
| Benchmark | `benches/{validation,persistence}.rs`; `docs/verification/T-P17-01-benchmark.md:14`; `T-P17-01-benchmark-run{1,2}.json`; `T-PERSIST-05-{local,native-linux,native-windows}-benchmark.json`; misure non rieseguite |

File assenti o storici, senza inferirne disponibilità runtime: le fonti del kit e `docs/REFERENCE_DATA_SPEC.md` assenti sono elencate in §2; il requisito diagnostici esiste in `docs/ui-design-documentation/DIAGNOSTIC_REQUIREMENTS.md`, non più in docs/ui. La vecchia `docs/ui/signature/REQUEST.md` richiamata dai mandati non è disponibile al percorso originale né in una sottocartella signature della directory corrente (mandato storico in AGENTS e documenti correnti letti). `SESSION_RECOVERY.md` root assente (template presente); `ARCHITECTURE.md`/`DETAILED_DESIGN.md` root assenti (presenti sotto docs); `docs/ui/` e vecchie capture/log rimossi in ee33d64 (Git storico); manifest Node/TS/Tauri e capability/CSP assenti dal prodotto; UI storica/CAD/Full/sort/settings durevoli assenti. Mockup adiacente identificato solo relativamente come `../mockup_slint_UI/{architecture,lintra_ui_html,lintra-ui-Rust_slint}`, due `.slint` nel prototipo esterno e due nello spike escluso. Nessun percorso personale completo trascritto.

Glossario: **raw** modello authoring non eseguibile; **compiled** schema validato/lowered, readonly; **LocalOnly** controlli locali del trace selezionato, non conformità globale; **Full** due passate/cross-trace nello scope supportato; **Unknown** evidenza non conclusiva; **NotApplicable** regola non applicabile; **spool** JSONL temporaneo owned; **offset** posizione byte nel singolo report; **CAS** verifica generazione prima del commit; **PHI/PII** dati sanitari/personali; **HMAC** pseudonimizzazione keyed, non cifratura; **M8** milestone UI nello scope ratificato, non prodotto clinico completo; **readiness** disponibilità a pianificare con condizioni, non autorizzazione a migrare.

### Verifica finale e impronte

V-08 conclusa con exit0 e **37 test positivi**: atomic_report12, diagnostic_rendering6, golden13 (include helper del harness), request_preflight6. Nessuna suite SQLite/sessione/UI runtime eseguita. V-01…08 e V-12 non sono attestazioni della CI o dell'intero prodotto finale.

Tempi di chiusura dei log (UTC, da mtime, non benchmark): V-06 13:38:39, V-07 13:40:15, V-08 13:41:44. Rilevata successivamente modifica esterna di `src/persistence/mod.rs` alle13:46:49, `src/application/history_audit.rs` alle13:47:29 e `tests/history_audit.rs` alle13:47:30. Gli mtime aiutano a collocare la variazione; **gli hash differenti sono la prova del drift**, non una ricostruzione completa dei processi concorrenti. Queste revisioni finali sono `NOT_VERIFIED` dai controlli precedenti. La ricognizione delle API WIP è indicativa e il piano deve scegliere una baseline stabile.

Impronte SHA-256 dei file inizialmente dirty; “=” indica hash finale identico al valore iniziale. File non modificati dall'audit.

| File | SHA-256 iniziale | SHA-256 al riesame finale |
|---|---|---|
| ATOMIC_TASKS_INDEX.md | `b05d2ca625e72528370ac2ed350804a1986352fd3cfee2de8db6d9532af7c230` | = |
| IMPLEMENTATION_STATUS.md | `ebe68bf0bf14365c1609784a9d5daf8cca5d2ac824629a45ad43dad0a6c3b468` | = |
| src/application/mod.rs | `8c3160c5e8d10385b1f3ac572e436c230c5ec7ef3ce32cfcedeef392c30b5de4` | = |
| src/lib.rs | `edfbf6d9d8f71b61759f1df9d71ba2cd042ba26c1fac16d3b6f4977285998766` | = |
| src/persistence/memory.rs | `975e1f60c4c1f256cdc9c83e1c44976743dba170affc203c4692d3e84f48a752` | = |
| src/persistence/mod.rs | `b2377428aeb807d85eb47fdfce1280ff25fc92a38d0b67bfb3cde76646e739b7` | `b895f6c78f19304eb6b20b019e606a7acda89a9964674dac78c3d56ff1e27e52` |
| src/persistence/sqlite/mod.rs | `9c51b4a5398f27c5211ff4b120c321ecc9760aab23cb6b64cf880bfed5dcc795` | = |
| src/persistence/sqlite/tests.rs | `2d0f214144fcad69f85173fa93fb45a0938d8d80e2be835fad453d8ba1012f5e` | = |
| docs/verification/T-PERSIST-08-preflight.md | `e6e56fc3883707a2f68495de7c6f0f09c838379ebe786c133aae34d6cc17b256` | = |
| src/application/history_audit.rs | `2c649d3585c4d3912e632596916be755b098bb9dd7a4abefabf6742b39a4fc10` | `defa6da97b637df55c7f9ac863a1f4244ba5800d3f27be8c288f60dd1a252113` |
| tests/history_audit.rs | `2c27efd043fa04213187003ab6983167c0bcae196e1535950579aad0fd283bf6` | `6d0fdcef01c84face64c9a8fbe1d4188c3c263201eaad5b93906d9083cd89073` |

Ulteriore file comparso esternamente: `docs/verification/T-PERSIST-08-audit.md`, SHA-256 `21f33830b1f55ba7a1a6605a86e655f1cac81d850916eda0b312cb2be7f1d382`; descrive implementazione **in qualificazione**, non ratifica/CI nuova. HEAD resta `fe5ada864be3455479c2b46765c662875f008933`, branch invariato, index senza modifiche staged. Differenza iniziale/finale: il presente report creato dall'audit, più le variazioni esterne esplicitate sopra; nessun effetto collaterale dei controlli trovato nei file tracked restanti. Output di build/test/log confinati in `/tmp/lintra-preflight`.

Impronte dei lock e delle due reference lette, senza rigenerazione. `light-1x.png` anche ispezionata visivamente: fixture dichiarata sintetica, tabella/dettaglio/dominio e palette coerenti con l’inventario; ciò non costituisce esecuzione di test UI:

| File | SHA-256 |
|---|---|
| Cargo.lock | `016c337544ea30b25e673b8f9f637be11caa8af4789155c18e21d683d08fdfac` |
| xtask/Cargo.lock | `fedec5cf14e2390447969557c35d563f1d89025da009e6fcd4a69b3fad58cfa4` |
| ui/tests/reference/light-1x.png | `05563bb32ac5d647563e775fc7ce46b85dc7c37b38dd5ee8da0988febd70859f` |
| ui/tests/reference/light-2x.png | `06f4a1efb32d96826daee4c72d9bc542017943bff2b42e2fb59100b28efa1f51` |

Impronte dei cinque font letti: MonoRegular `fe11304a5fe956d5744e9b6a246cc83d90425245e75a62230044966ca96a7f50`; MonoSemiBold `c9417148ce13f8fa7d2d5c9180bbc141f72aa0d814ffeb280f6904dc2b1bbd7a`; SansBold `9e6c74a889a700d707613d24548fe4ffa6bc59559a0689d2cf9e133bdcdafb2f`; SansRegular `975dcda37d80f038dcd143c22e33ca2d97a0cc5a929aace1c749153b0fe1afa5`; SansSemiBold `a20caf8286023a6a7a85e40b1d2a4ae9fc3e3b1f9eda8f4c542dd4986af67bb1`.

Log temporanei condensati in §11, con digest per verifica eventuale (non indispensabili alla lettura autosufficiente):

| Log | SHA-256 |
|---|---|
| metadata.log | `b223a94858f9dc1f52a1403aea30d9be7d3de70000204e64f6e0c55b1dfac3d9` |
| fmt.log (vuoto, exit0) | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| tree.log | `e2f702e87a59cfdaa207828d45eed26b2381c23c24f3787439bb617e04958b94` |
| check.log | `56c4df822782d568cc393fe9e3d35c564139b284ee2d74dd5cd4f517f8b0e620` |
| clippy.log | `49961e6a5ef0d9a52f20b09dbdf7c8112da6599d170a94105a0bb899927656fc` |
| check-workspace.log | `05f62023bd911e1ed7018ba3d0de9548a3c0483a02a040aa548a8735be325ce1` |
| clippy-workspace.log | `8c7d5e95f3dc37e4d63352a6d40d7dda9d2f47d8422ff05d4bc6be7c1233c5c4` |
| tests-core.log | `761785ffd32ab0117051153c0ba73f4af3f7fd93094091525f90427358c9a8b3` |
| ci-read.log | `eed65211f0fcee1b82c2661764250c1adaebcb36f0cb78a702d59632f056c439` |
| graph.log (V-12, riepilogo guard) | `958660e35ba6d697271dd2cfd5990d4a3913422f225515faf3d35b2410d17197` |

Controllo finale documento: indice richiesto in15sezioni, registri con tutti i campi, riferimenti file/riga esistenti e entro lunghezza, nessun percorso personale o contenuto di record reale, nessuna modifica implementativa/commit/push dell'audit. Le incongruenze trovate sono documentate e lasciate inalterate.
