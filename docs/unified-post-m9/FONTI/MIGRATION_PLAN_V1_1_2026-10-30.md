# LINTRA — MASTER MIGRATION IMPLEMENTATION PLAN
## Slint → React + TypeScript + Tauri 2 — piano operativo per agenti AI

| Campo | Valore |
|---|---|
| Versione | **1.1 — KIT CONSOLIDATO; decisioni prodotto approvate dall’autore, gate tecnici aperti** |
| Data | 2026-10-30 (Europe/Rome) |
| Baseline di analisi | Git `fe5ada864be3455479c2b46765c662875f008933` + working tree variato durante audit (non baseline esecutiva) |
| Fonte primaria audit | `SOURCES/SLINT_TO_TAURI2_PREFLIGHT_REPORT.md` (§§1–15) |
| Fonte architetturale | `docs/ARCHITECTURE.md`, `docs/DETAILED_DESIGN.md` del repository e ADR-PERSIST-001/002 |
| Governance | `AGENTS.md`, `MASTER_AGENT_PROMPT.md`, piano originale §§4,14–15, secondo ratifiche |
| Stato del piano | **STRATEGY_USER_APPROVED / G-MIG-0 NOT_VERIFIED**; autorizzazione al solo P0 e ratifiche, non a implementazione Tauri prima di G0 |
| Consuntivo | `MIGRATION_IMPLEMENTATION_STATUS.md` separato dal consuntivo originario |

> **Decisioni consolidate dall’autore (2026-10-30):** `DEC-MIG-01/02/03/04/05/07/08/09` `USER_APPROVED`; `DEC-MIG-06` richiede baseline Git e CI. **Windows GUI Windows 11 x86_64 solamente**: installer NSIS offline + WebView2 `offlineInstaller`, `perMachine`, manual update, uninstall conservativo, dati separati per account. **Linux GUI**: build e smoke personali soltanto sul PC CachyOS/Hyprland/Wayland. React + TS strict + Vite + Tailwind + shadcn/ui + TanStack, redesign responsive e **Full multi-tracciato obbligatorio** prima del cutover. Local-only, nessuna telemetria/rete/updater, test con dati sintetici e real-data gate separato. Backup SQLite pre-upgrade è workstream PERSIST separato. CORE/CLI/policy normative invariati. **Le ADR nel kit sono da recepire/ratificare formalmente nel repository; nessun gate si considera già superato.**

> **Il motore sa COME validare, lo schema sa COSA è valido.** La migrazione modifica solo la UI desktop e l'adapter di presentazione, **esponendo anche Full già presente nel core**; non modifica regole normative, core, schema, CLI, persistenza approvativa o dati. Questo piano non ratifica N-24 e non dichiara pronti i dati clinici reali.

## 1. Scopo e confini

**In scope:** redesign moderno e responsive durante la migrazione; parità LocalOnly con Slint **più UI Full multi-tracciato del servizio Rust esistente**, qualificata con fixture sintetiche e gate dedicati; sostituzione controllata del prodotto `lintra-ui` Slint con un nuovo frontend React/TypeScript su Tauri major 2, coesistenza temporanea, interfaccia tipizzata Rust↔JS, parità osservata, **installer NSIS Windows 11 x86_64 offline per-machine** e **UI Linux soltanto per build personale sul PC CachyOS/Hyprland dell’autore**, sicurezza WebView, osservabilità e rollback. La CI core/CLI Linux+Windows rimane invariata.

**Parità osservata nel preflight:** una analisi **LocalOnly** su un file/tracciato con scelta flow/version/trace/encoding supportato (selettore UI corrente UTF-8), catalogo/bundle e audit, conferma percorso e preflight campione, worker/progresso/annullamento, `CompletedValid`/`CompletedInvalid`/`Cancelled`/`Err` separati da completezza, tabella JSONL lazy con 5 filtri esatti AND, dettaglio/field guide/domìni paginati, export integrale e Informazioni.

**Esplicitamente out of scope:** UI di storico SQLite/preview/approve/commit, GUI CAD temporale non qualificata, sorting interattivo, settings permanenti, installazione di nuovi bundle, nuova logica di compliance, correzione normativa SIOC/CAD, cifratura database, API REST, server database, sincronizzazione remota, backup SQLite come implementazione implicita. **Full multi-tracciato è invece IN SCOPE ed è obbligatorio per il cutover** (DEC-MIG-07 C): il servizio core esistente non prova automaticamente disponibilità/completezza dei profili regionali. Altre feature core possono essere proposte solo se qualificate, con task esplicite e gate, mai aggiunte automaticamente.

**Attenzione catalogo:** snapshot embedded corrente `SYNTHETIC/v1` dimostrativo; non è validatore regionale completo. Profilo SIOC structural-minimal e CAD candidate non devono diventare “certificati” per effetto della nuova GUI. **Non usare dati paziente reali** in benchmark, screenshot o UI smoke.

## 2. Fotografia del prodotto e razionale di migrazione

Dal preflight (evidenza statica + test limitati):
- Package `lintra` v0.1.1-rc.1 con library e CLI; `lintra-ui` separato, ma in workspace `default-members = ["."]`: i test senza `--workspace` **non** coprono UI. Slint v1.18.1 e software renderer, font IBM Plex embedded.
- `lintra-ui → lintra`, mai `lintra → slint`. `clap` è comunque nel package core+CLI; non affermare un grafo package senza clap (CNF-MIG-05).
- Risultati JSONL privati, indice `Vec<u64>` O(errori), cache 128, filtri scansiti da worker; riga su cache miss letta dal thread UI. Il backend Rust del bridge deve mantenere dati/indice/paging, con sola virtualizzazione DOM in React.
- SQLite approvato con storage HMAC, CAS/commit ambiguity/explicit recovery, **separato dalla validazione LocalOnly e senza UI storica**; T-PERSIST-08 WIP non qualificato dalla sessione di audit, non assumerne API stabili.
- 37 test mirati V-08 PASS nella fotografia osservata; `cargo check/clippy --workspace` PASS; suite completa e release Tauri **NOT_RUN**; CI remota live **NOT_VERIFIED**. Il piano originale riporta 117/140 `DONE` dichiarati, non 117 componenti tecnici verificati su HEAD.

Conseguenza: fare **strangler migration** dell'adapter desktop, preservando Slint sino ai gate di parità. Non ricostruire motore e servizi né invocare la CLI come subprocess.

## 3. Gerarchia delle fonti e convenzioni

1. ARCH/DET ratificati, decisioni normative e ADR di persistenza: **vincoli**, non sostituiti dalla presente proposta.
2. GOV corrente (`AGENTS.md`, `MASTER_AGENT_PROMPT.md`) + piano originale §§4,14–15 per flusso single-agent e status. Il presente piano è **approvato strategicamente dall’autore** ma abilita l’implementazione solo dopo ADR formalmente recepite nel repository e G-MIG-0 APPROVED con evidenze. Le scelte utente nei file ADR-MIG-001..004 devono essere recepite formalmente nel repository durante P0, non sostituiscono i gate.
3. Codice sul commit esecutivo scelto e test effettivi: **evidenza reale**. Le prescrizioni non si spacciano per implementazione e viceversa.
4. Preflight del 2026-10-30: **[AUDIT]**, non HEAD odierno. Valori tecnici qui attribuiti all'audit possono cambiare.
5. Documentazione ufficiale esterna: **[EXT]**, utile a verificare Tauri/React/WebView alle versioni realmente selezionate, non ratifica.

ID **non riutilizzabili**: `T-MIG-Px-nn` task; `P0..P9` fasi di questo documento (sempre prefisso `MIG-` se ambiguo); `MM0..MM9` milestone migrazione; `G-MIG-0..9` gate; `DEC-MIG-*`, `CNF-MIG-*`, `RSK-MIG-*`, `QG-MIG-*` registri. **Non modificare** gli ID `T-P0..P18`, `T-PERSIST-*`, `T-CAD-*` originali.

Stati: `NOT_STARTED`, `IN_PROGRESS`, `BLOCKED`, `DONE_UNVERIFIED`, `DONE`. `DONE` richiede prove tutte le acceptance e gate richiesti. Consuntivo separato in `MIGRATION_IMPLEMENTATION_STATUS.md`; cambi al piano soltanto ratificati e registrati nel Plan Change Log.

## 4. Vincoli non negoziabili e disciplina del delta

Vigono **N-01..N-31** del piano originale, con modifiche esplicite limitate al **contratto UI N-24**, eccezioni **UI-only a N-19/N-20/QG-5**, chiarimento release GUI personale rispetto a N-23 e revisione QG-7 approvate come scelte di prodotto (ADR da recepire). Fino alla ratifica Slint rimane la scelta vigente. N-19/20/23, QG-5/7 e ARCH §39D non si dichiarano superati perché Tauri compila.

| Ambito | Invariante conservata | Nuovo obbligo UI / decisione |
|---|---|---|
| N-01..04, N-07..10, N-13..18 | engine generic/schema external/compiled only; `Unknown` non `false`; no runtime AI | React non interpreta DSL, non calcola obbligatorietà, non sceglie latest |
| N-05..06, N-12, N-22, N-25 | byte-oriented, JSONL canonico streaming, ordine/hash/audit, CSV/XLSX equivalenti | nessuna modifica writer o golden per rendere verde frontend; backend paging |
| N-02 | UI e CLI adapters della library | solo adapter Tauri importa core; niente `tauri` in core, niente CLI subprocess; chiarire `clap` package CNF-05 |
| N-11 | completed invalid != technical failure; Cancelled controllato | wire outcome + completezza separata e old-report context |
| N-19..20 | Linux/Windows CI per core e CLI; nuovo scope UI mirato | Windows UI installer NSIS offline con WebView2; Linux UI soltanto CachyOS personale, WebKitGTK installabile sul target e smoke locale. N-19/20 richiedono ADR per questa eccezione **UI-only**; nessun cambiamento alle regole CLI |
| N-21 | embedded default vs external prioritized, no fallback misto | bundle resolution/audit esclusivamente Rust, no nuovo bundle loader JS |
| N-23 | release CI immutabili, hash, dati separati | nuova catena JS+Rust/installer/WebView ed evidence sulla stessa SHA |
| N-24 | oggi Slint/software/font/test RGB | React+TS strict+Vite+Tailwind/shadcn/TanStack + Tauri2, redesign, design token/font locali, acceptance WebView per-OS; slint deprecata solo dopo parità |
| N-26 | no dependencies speculative | qualifica licenze/pinning/plugin; nessuna libreria UI aggiunta automaticamente dal piano |
| N-28..31 | storage generic, HMAC, preview/approval, CAS/snapshot, PHI minimized | nuova GUI NON ottiene approva_commit o raw DB; bridge limita accesso report e DOM |

## 5. Decisioni, contraddizioni e policy di ratifica

Il registro completo è `MIGRATION_DECISION_REGISTER.md` (preservati gli ID del preflight). Qui le **scelte di prodotto approvate dall’autore** DEC-MIG-01/02/03/04/05/07/08/09 non sono dichiarazioni di completamento dei gate: P0 le recepisce negli ADR e riconcilia ARCH/DET/PLAN. DEC-MIG-06 rimane `OPEN_TECHNICAL` sino a fotografia Git/CI. Parametri tecnici non ancora determinati (DTO, limiti, versioni, performance budgets e API) si qualificano e ratificano in G1/G2/G3 prima del codice interessato.

- **CNF-MIG-01:** sostituzione N-24 richiede ADR e revisione ARCH/DET.
- **CNF-MIG-02:** la nuova UI adotta Windows11 NSIS offline perMachine e Linux limitato al CachyOS personale del proprietario. È una scelta di prodotto approvata dall’utente, ma la deviazione da N-19/N-20 richiede ADR e prove native; CLI policy resta intatta.
- **CNF-MIG-03:** quality gate pixel RGB Slint non è trasferibile 1:1 tra WebKitGTK e WebView2; definire screenshot funzionali e divergenze accettabili, non annullare il gate.
- **CNF-MIG-04/05:** status storici e clap/build provenance divergono da alcune frasi del piano; documentare e ratificare l'interpretazione senza refactor fortuito.
- **CNF-MIG-06:** il working tree del preflight ha drift T-PERSIST-08; è **bloccante** per una baseline esecutiva, non per il presente piano.

## 6. Architettura target (proposta, non API definitiva)

```text
apps/lintra-desktop/ (React+TypeScript strict, Vite, Tailwind, shadcn/ui, TanStack Table/Virtual)
  ├─ stato presentazione, navigazione, local style/IBM Plex
  └─ typed client / invoke + bounded channel (versioned)
               │
               ▼
apps/lintra-desktop/src-tauri/ (Rust adapter; allowlist, authz, DTO)
  ├─ AppState: runId, reportId, queryId, worker cancellation, TempDir ownership
  ├─ Commands: catalog / prepare / input / run (LocalOnly+Full) / results / domains / export
  └─ DB e filesystem access SOLO tramite API Rust e scope autorizzati
               │
               ▼
lintra library: ValidationService ← CompiledFlowSchema / ReferenceEvidence
  ├─ streaming ValidationSink → canonical JSONL spool + summary
  ├─ Rust ReportIndex, filtered query, lazy row, domain page
  └─ Writer existing (XLSX, CSV, JSONL), audit

CLI unchanged ────────────────────────┘
PERSIST/SQLite Session/Approval API: **NON ESPOSTE** dalla nuova UI in questa migrazione; backend Full **è** esposto tramite adapter con gate propri
```

Principi: nessun record fixed-width entra nel JS; nessun file intero o tutti i diagnostici inviati attraverso IPC; solo pagina/dettaglio richiesto. Le query filtro rimangono Rust. DTO con opacità stabile per run e cursor, precisione numerica conservata (numeri oltre safe JS rappresentati come testo). `CompiledFlowSchema` non diventa JSON indiscriminato e `PreparedReferenceEvidence` non viene riletto dal disco dopo il run.

### 6.1 Eventi, errori e security
- Progress bounded/coalescibile; terminal status recuperabile, cancellazione cooperativa con ownership Rust.
- `CompletedValid`, `CompletedInvalid`, `Cancelled`, errore tecnico e completezza evidenziale distinti, senza percentuali finte.
- Capability/plugin policy e autorizzazioni backend **anche dei custom commands**: i comandi `invoke_handler` Tauri sono accessibili per default alle WebView dell’app, salvo configurazione restrittiva apposita (`AppManifest::commands` e controlli Rust); CSP offline, rendering come **testo**, nessun URL/asset/HTML raw da valori record; devtools/log/cache policy di produzione definita.
- È vietata l'approvazione persistente dal frontend: eventuale futura UI storica richiede ADR e token/preview server-side antifalsificazione.
- Forma iniziale non vincolante in `MIGRATION_IPC_ARCHITECTURE_DRAFT.md`, **da approvare G-MIG-2**.

### 6.2 Layout del repository proposto
```text
lintra/
├── Cargo.toml                      # workspace originale conservato
├── src/                            # core + CLI; non importare Tauri
├── ui/                             # Slint legacy fino a parità/rollback
├── apps/lintra-desktop/
│   ├── package.json + lockfile
│   ├── src/                         # React, features, typed IPC, styles
│   ├── public/fonts/                # OFL + font asset previo permesso
│   └── src-tauri/                  # Tauri2, Rust adapter, app state/commands
├── xtask/                           # verifier policy CLI vs UI separata
├── tests/                            # golden/invarianti esistenti
├── docs/migration/                  # ADR, gate, evidence, plan/status
└── .github/workflows/               # CI per OS senza regressioni core
```

## 7. Strategia per fase e milestone

| Fase | Descrizione | Milestone | Gate |
|---|---|---|---|
| MIG-P0 | Governance, baseline e ratifiche | MM0 | G-MIG-0 |
| MIG-P1 | Fattibilità nativa Tauri/WebView | MM1 | G-MIG-1 |
| MIG-P2 | Contratti di architettura, IPC e sicurezza | MM2 | G-MIG-2 |
| MIG-P3 | Scaffold affiancato e toolchain | MM3 | G-MIG-3 |
| MIG-P4 | Adapter Rust e lifecycle backend | MM4 | G-MIG-4 |
| MIG-P5 | React: configurazione ed esecuzione | MM5 | G-MIG-5 |
| MIG-P6 | React: risultati, dominio, export | MM6 | G-MIG-6 |
| MIG-P7 | Sicurezza, accessibilità e performance | MM7 | G-MIG-7 |
| MIG-P8 | CI, dipendenze, packaging e installazione | MM8 | G-MIG-8 |
| MIG-P9 | Cutover, rollback, documentazione e consegna | MM9 | G-MIG-9 |

**Milestone non equivalgono a release**: `MM0` significa governance ratificata, `MM8` artefatti nativi qualificati, `MM9` cutover/rollback/documenti. Il vecchio M8 Slint resta storicamente `DONE` nello scope originale.

### 7.1 Critical path
`P0 ratifica+baseline → P1 runtime+installer NSIS offline GO → P2 security/IPC **Full** approval → P3 scaffold affiancato → P4 Rust Job Manager **LocalOnly + Full** → P5 redesign React con Full → P6 DataGrid/diagnostici/export **anche Full** → P7 qualifica Full/PHI/performance → P8 CI Windows11 + packaging e smoke CachyOS → P9 scorecard Full+LocalOnly, cutover/rollback, decommission Slint`.

Le attività indipendenti di sola documentazione possono essere preparate prima dei gate, ma **non** aggirano prerequisiti di una task implementativa. Se Windows11 clean-room non è disponibile, non equiparare un runner CI alla prova di installer NSIS offline e provisioning WebView2 in VM pulita.

## 8. Schede di fase (scope, accettazione, anti-regressioni)

### MIG-P0 — Governance, baseline e ratifiche
**Obiettivo.** Nessun intervento alla UI o all’engine prima della ratifica delle scelte e della fissazione della baseline.

**Prerequisito di ingresso:** baseline/audit e GOV senza modificare codice.
**Output:** dossier task `T-MIG-P0-*`, stato/evidenze nel consuntivo, gate `G-MIG-0`, milestone `MM0`.
**Accettazione:** tutte le task in scope `DONE` con test reali pertinenti e decisioni necessarie ratificate; nessun blocco `CNF-MIG-*` legato alla fase. Si emette stato `BLOCKED`/`DONE_UNVERIFIED` quando mancano prove, non `DONE`.
**Stop:** nessuna riduzione di QG-1..8 o alterazione di core/CLI/SQLite per far passare GUI, nessuna feature normativa nuova.

### MIG-P1 — Fattibilità nativa Tauri/WebView
**Obiettivo.** Provare le assunzioni su CachyOS e Windows11, **incluso un installer NSIS offline anticipato**, prima dello sviluppo applicativo.

**Prerequisito di ingresso:** G-MIG-0 APPROVED e decisioni della fase ratificate.
**Output:** dossier task `T-MIG-P1-*`, stato/evidenze nel consuntivo, gate `G-MIG-1`, milestone `MM1`.
**Accettazione:** tutte le task in scope `DONE` con test reali pertinenti e decisioni necessarie ratificate; nessun blocco `CNF-MIG-*` legato alla fase. Si emette stato `BLOCKED`/`DONE_UNVERIFIED` quando mancano prove, non `DONE`.
**Stop:** nessuna riduzione di QG-1..8 o alterazione di core/CLI/SQLite per far passare GUI, nessuna feature normativa nuova.

### MIG-P2 — Contratti di architettura, IPC e sicurezza
**Obiettivo.** Congelare i contratti versionati di interfaccia e privacy prima di implementarli.

**Prerequisito di ingresso:** G-MIG-1 APPROVED e decisioni della fase ratificate.
**Output:** dossier task `T-MIG-P2-*`, stato/evidenze nel consuntivo, gate `G-MIG-2`, milestone `MM2`.
**Accettazione:** tutte le task in scope `DONE` con test reali pertinenti e decisioni necessarie ratificate; nessun blocco `CNF-MIG-*` legato alla fase. Si emette stato `BLOCKED`/`DONE_UNVERIFIED` quando mancano prove, non `DONE`.
**Stop:** nessuna riduzione di QG-1..8 o alterazione di core/CLI/SQLite per far passare GUI, nessuna feature normativa nuova.

### MIG-P3 — Scaffold affiancato e toolchain
**Obiettivo.** Aggiungere una seconda UI senza alterare il prodotto Slint o i contratti del core.

**Prerequisito di ingresso:** G-MIG-2 APPROVED e decisioni della fase ratificate.
**Output:** dossier task `T-MIG-P3-*`, stato/evidenze nel consuntivo, gate `G-MIG-3`, milestone `MM3`.
**Accettazione:** tutte le task in scope `DONE` con test reali pertinenti e decisioni necessarie ratificate; nessun blocco `CNF-MIG-*` legato alla fase. Si emette stato `BLOCKED`/`DONE_UNVERIFIED` quando mancano prove, non `DONE`.
**Stop:** nessuna riduzione di QG-1..8 o alterazione di core/CLI/SQLite per far passare GUI, nessuna feature normativa nuova.

### MIG-P4 — Adapter Rust e lifecycle backend
**Obiettivo.** Rendere query, worker e report **LocalOnly+Full** accessibili da un bridge bounded, con ownership Rust.

**Prerequisito di ingresso:** G-MIG-3 APPROVED e decisioni della fase ratificate.
**Output:** dossier task `T-MIG-P4-*`, stato/evidenze nel consuntivo, gate `G-MIG-4`, milestone `MM4`.
**Accettazione:** tutte le task in scope `DONE` con test reali pertinenti e decisioni necessarie ratificate; nessun blocco `CNF-MIG-*` legato alla fase. Si emette stato `BLOCKED`/`DONE_UNVERIFIED` quando mancano prove, non `DONE`.
**Stop:** nessuna riduzione di QG-1..8 o alterazione di core/CLI/SQLite per far passare GUI, nessuna feature normativa nuova.

### MIG-P5 — React: configurazione ed esecuzione
**Obiettivo.** Realizzare UI LocalOnly esistente **e nuova UI Full multi-tracciato già nel core**, senza introdurre regole normative nuove; redesign moderno responsive.

**Prerequisito di ingresso:** G-MIG-4 APPROVED e decisioni della fase ratificate.
**Output:** dossier task `T-MIG-P5-*`, stato/evidenze nel consuntivo, gate `G-MIG-5`, milestone `MM5`.
**Accettazione:** tutte le task in scope `DONE` con test reali pertinenti e decisioni necessarie ratificate; nessun blocco `CNF-MIG-*` legato alla fase. Si emette stato `BLOCKED`/`DONE_UNVERIFIED` quando mancano prove, non `DONE`.
**Stop:** nessuna riduzione di QG-1..8 o alterazione di core/CLI/SQLite per far passare GUI, nessuna feature normativa nuova.

### MIG-P6 — React: risultati, dominio, export
**Obiettivo.** Parità funzionale sul percorso diagnostico LocalOnly **e Full**, con grandi dataset virtualizzati e zero bulk IPC.

**Prerequisito di ingresso:** G-MIG-5 APPROVED e decisioni della fase ratificate.
**Output:** dossier task `T-MIG-P6-*`, stato/evidenze nel consuntivo, gate `G-MIG-6`, milestone `MM6`.
**Accettazione:** tutte le task in scope `DONE` con test reali pertinenti e decisioni necessarie ratificate; nessun blocco `CNF-MIG-*` legato alla fase. Si emette stato `BLOCKED`/`DONE_UNVERIFIED` quando mancano prove, non `DONE`.
**Stop:** nessuna riduzione di QG-1..8 o alterazione di core/CLI/SQLite per far passare GUI, nessuna feature normativa nuova.

### MIG-P7 — Sicurezza, accessibilità e performance
**Obiettivo.** Dimostrare assenza di regressioni di correttezza, contenimento PHI e controllo memoria.

**Prerequisito di ingresso:** G-MIG-6 APPROVED e decisioni della fase ratificate.
**Output:** dossier task `T-MIG-P7-*`, stato/evidenze nel consuntivo, gate `G-MIG-7`, milestone `MM7`.
**Accettazione:** tutte le task in scope `DONE` con test reali pertinenti e decisioni necessarie ratificate; nessun blocco `CNF-MIG-*` legato alla fase. Si emette stato `BLOCKED`/`DONE_UNVERIFIED` quando mancano prove, non `DONE`.
**Stop:** nessuna riduzione di QG-1..8 o alterazione di core/CLI/SQLite per far passare GUI, nessuna feature normativa nuova.

### MIG-P8 — CI, dipendenze, packaging e installazione
**Obiettivo.** Qualificare l’artefatto Tauri nativo senza allargare le policy CLI e senza fingere portability.

**Prerequisito di ingresso:** G-MIG-7 APPROVED e decisioni della fase ratificate.
**Output:** dossier task `T-MIG-P8-*`, stato/evidenze nel consuntivo, gate `G-MIG-8`, milestone `MM8`.
**Accettazione:** tutte le task in scope `DONE` con test reali pertinenti e decisioni necessarie ratificate; nessun blocco `CNF-MIG-*` legato alla fase. Si emette stato `BLOCKED`/`DONE_UNVERIFIED` quando mancano prove, non `DONE`.
**Stop:** nessuna riduzione di QG-1..8 o alterazione di core/CLI/SQLite per far passare GUI, nessuna feature normativa nuova.

### MIG-P9 — Cutover, rollback, documentazione e consegna
**Obiettivo.** Approvare il passaggio definitivo soltanto dopo parità e release qualificate, conservando rollback.

**Prerequisito di ingresso:** G-MIG-8 APPROVED e decisioni della fase ratificate.
**Output:** dossier task `T-MIG-P9-*`, stato/evidenze nel consuntivo, gate `G-MIG-9`, milestone `MM9`.
**Accettazione:** tutte le task in scope `DONE` con test reali pertinenti e decisioni necessarie ratificate; nessun blocco `CNF-MIG-*` legato alla fase. Si emette stato `BLOCKED`/`DONE_UNVERIFIED` quando mancano prove, non `DONE`.
**Stop:** nessuna riduzione di QG-1..8 o alterazione di core/CLI/SQLite per far passare GUI, nessuna feature normativa nuova.

## 9. Task atomiche operative

**89 task atomiche in 10 fasi, nove aggiunte senza rinumerare le 80 precedenti.** I prerequisiti vincolanti sono mantenuti in un'unica matrice in `MIGRATION_ATOMIC_TASKS_INDEX.md` e `MIGRATION_DEPENDENCY_MAP.md`. I dettagli implementativi sono in `TASK_PROMPTS/`. L'agente esegue una task per volta, con dossier e handoff; le ratifiche non si desumono dal nome del task.

| ID | Task | Prerequisiti | Acceptance |
|---|---|---|---|
| [T-MIG-P0-01](TASK_PROMPTS/T-MIG-P0-01.md) | Fotografia Git e baseline post-WIP08 | — | Baseline identificata con SHA e matrice test OK/NOT_RUN; se WIP non qualificato bloccare task che dipendono dalle nuove API. |
| [T-MIG-P0-02](TASK_PROMPTS/T-MIG-P0-02.md) | Riconciliazione dello stato e dei riferimenti storici | T-MIG-P0-01 | Matrice di discrepanze con outcome: da aggiornare/accettare/bloccato; no cambiamento silenzioso. |
| [T-MIG-P0-03](TASK_PROMPTS/T-MIG-P0-03.md) | ADR di sostituzione mirata N-24 | T-MIG-P0-02 | ADR firmato/approvato da autore, oppure BLOCKED; non basta questo piano come ratifica. |
| [T-MIG-P0-04](TASK_PROMPTS/T-MIG-P0-04.md) | Perimetro LocalOnly+Full, parità e capability future | T-MIG-P0-03 | Matrice approvata LocalOnly + Full (obbligatoria) e capability ancora non qualificate; niente Full fittizio né UI approval. |
| [T-MIG-P0-05](TASK_PROMPTS/T-MIG-P0-05.md) | Decisione OS minimi e distribuzione offline | T-MIG-P0-03 | Windows11 x86_64 NSIS perMachine offline + CachyOS personale ratificati; CLI original invariata. |
| [T-MIG-P0-06](TASK_PROMPTS/T-MIG-P0-06.md) | Policy PHI e threat model da ratificare | T-MIG-P0-03 | Decisione di sicurezza documentata, uso real data bloccato finché DEC-MIG-09 e ADR002 aperte. |
| [T-MIG-P0-07](TASK_PROMPTS/T-MIG-P0-07.md) | Criteri di parità UX/visiva e budget prestazionale | T-MIG-P0-04, T-MIG-P0-05 | Design brief responsive/redesign, acceptance UX/visiva per OS, budget prestazioni misurabili senza soglie inventate. |
| [T-MIG-P0-08](TASK_PROMPTS/T-MIG-P0-08.md) | Gate governance e autorizzazione all’avvio | T-MIG-P0-01, T-MIG-P0-02, T-MIG-P0-03, T-MIG-P0-04, T-MIG-P0-05, T-MIG-P0-06, T-MIG-P0-07 | G-MIG-0 APPROVED solo con ratifiche; altrimenti BLOCKED e stop implementazione. |
| [T-MIG-P1-01](TASK_PROMPTS/T-MIG-P1-01.md) | Spike build minimale Tauri su Linux | T-MIG-P0-08 | Dossier con versioni, librerie e limitazioni; no sostituzione N-20 via slogan. |
| [T-MIG-P1-02](TASK_PROMPTS/T-MIG-P1-02.md) | Spike Windows clean install e WebView2 | T-MIG-P0-08 | Spike Windows11 x86_64: WebView2/NSIS offline su VM clean-room, senza confondere check con installazione. |
| [T-MIG-P1-03](TASK_PROMPTS/T-MIG-P1-03.md) | Qualifica runtime CachyOS e dipendenze | T-MIG-P1-01 | Lista chiusa di runtime UI e distribuzione testabile, oppure no-go motivato. |
| [T-MIG-P1-04](TASK_PROMPTS/T-MIG-P1-04.md) | POC zero-network, capability e CSP | T-MIG-P1-01, T-MIG-P1-02 | Configurazione minima riproducibile e threat findings registrati; non basta build verde. |
| [T-MIG-P1-05](TASK_PROMPTS/T-MIG-P1-05.md) | Profilo WebView, font e comportamento DPI | T-MIG-P1-01, T-MIG-P1-02 | Prove riproducibili con configurazione OS+WebView, non dichiarare pixel parity universale. |
| [T-MIG-P1-06](TASK_PROMPTS/T-MIG-P1-06.md) | Decisione GO/NO-GO piattaforme | T-MIG-P1-02, T-MIG-P1-03, T-MIG-P1-04, T-MIG-P1-05, T-MIG-P1-07 | G-MIG-1 GO con prove native sulla matrice, oppure NO-GO/BLOCKED, senza avviare P3. |
| [T-MIG-P1-07](TASK_PROMPTS/T-MIG-P1-07.md) | Spike anticipato installer NSIS offline per-machine su Windows 11 | T-MIG-P1-02, T-MIG-P1-04 | Dossier riproducibile con log e SHA installazione/disinstallazione offline; errore o impossibilità di produrre la prova è NO-GO tecnico, non PASS presunto. |
| [T-MIG-P2-01](TASK_PROMPTS/T-MIG-P2-01.md) | Contratto architetturale e ownership processo | T-MIG-P1-06 | Diagrammi e boundary ratificati; niente dipendenze Tauri in domain/schema/engine. |
| [T-MIG-P2-02](TASK_PROMPTS/T-MIG-P2-02.md) | Envelope JSON DTO versionati e error taxonomy | T-MIG-P2-01 | Wire schema version, compatibilità, limiti byte e casi valid/invalid/boundary approvati. |
| [T-MIG-P2-03](TASK_PROMPTS/T-MIG-P2-03.md) | Run identity, ownership e macchina stati | T-MIG-P2-02 | Transizioni complete e matrice races; mai falso CompletedValid su errore tecnico. |
| [T-MIG-P2-04](TASK_PROMPTS/T-MIG-P2-04.md) | Progresso bounded e terminale affidabile | T-MIG-P2-03 | Invarianti no per-error push, <= bounded buffers, terminale osservabile anche con progresso scartato. |
| [T-MIG-P2-05](TASK_PROMPTS/T-MIG-P2-05.md) | Contratto risultati: paging, offset e filtri | T-MIG-P2-02, T-MIG-P2-03 | Max page e payload fissati con test; nessun indice o dataset in JS; ordinamento canonico preservato. |
| [T-MIG-P2-06](TASK_PROMPTS/T-MIG-P2-06.md) | Contratto dettaglio e domini catturati | T-MIG-P2-05 | Result DTO fedele e stato reference esplicito; nessuna enumerazione regex infinita. |
| [T-MIG-P2-07](TASK_PROMPTS/T-MIG-P2-07.md) | Contratto export e conferma path/overwrite | T-MIG-P2-03 | No transazione multi-file promessa; path non attraversa renderer se non inevitabile e autorizzato. |
| [T-MIG-P2-08](TASK_PROMPTS/T-MIG-P2-08.md) | Capability, CSP, path e log policy | T-MIG-P2-02, T-MIG-P2-07 | Threat model approvato con test negativi obbligatori e audit delle permission effettive. |
| [T-MIG-P2-09](TASK_PROMPTS/T-MIG-P2-09.md) | Contratto di parity test/fixture sintetica | T-MIG-P2-03, T-MIG-P2-04, T-MIG-P2-05, T-MIG-P2-06, T-MIG-P2-07, T-MIG-P2-08 | Harness definito con expected indipendenti e hash fixture, senza usare dati reali. |
| [T-MIG-P2-10](TASK_PROMPTS/T-MIG-P2-10.md) | Ratifica cross-contract e gate di design | T-MIG-P2-01, T-MIG-P2-02, T-MIG-P2-03, T-MIG-P2-04, T-MIG-P2-05, T-MIG-P2-06, T-MIG-P2-07, T-MIG-P2-08, T-MIG-P2-09, T-MIG-P2-11 | G-MIG-2 APPROVED prima del codice bridge; firme e primitive non finali sino ad allora. |
| [T-MIG-P2-11](TASK_PROMPTS/T-MIG-P2-11.md) | Contratto Full multi-tracciato: input, run e report coerenti | T-MIG-P2-03, T-MIG-P2-05, T-MIG-P2-08 | Schema DTO/protocollo + matrice casi Full e modello di ownership pubblicati e approvati nel gate G-MIG-2; nessuna API PERSIST/approve esposta. |
| [T-MIG-P3-01](TASK_PROMPTS/T-MIG-P3-01.md) | Scaffold Tauri affiancato non predefinito | T-MIG-P2-10 | Build scaffold Linux e Windows CI condizionato, Slint/CLI unchanged; no rimozione legacy. |
| [T-MIG-P3-02](TASK_PROMPTS/T-MIG-P3-02.md) | Pin frontend e lockfile/supply-chain | T-MIG-P3-01 | Pin versioni e hash/lock verificati; motivazione dipendenza; nessun aggiornamento silenzioso. |
| [T-MIG-P3-03](TASK_PROMPTS/T-MIG-P3-03.md) | Copia autorizzata token/font/asset | T-MIG-P3-01 | Nessuna risorsa remota, hash fonte/attribuzioni preservate; visual baseline differenziata. |
| [T-MIG-P3-04](TASK_PROMPTS/T-MIG-P3-04.md) | Rust app state e dispatcher senza business logic | T-MIG-P3-01, T-MIG-P3-02 | Core non dipende Tauri; shell avvio/chiusura smoke; nessuna validazione ancora collegata. |
| [T-MIG-P3-05](TASK_PROMPTS/T-MIG-P3-05.md) | Client TS tipizzato e verifica schema bridge | T-MIG-P3-02, T-MIG-P3-04 | Test schema TS↔Rust incl. unknown fields/version mismatch e fallback; nessuna duplicazione normativa. |
| [T-MIG-P3-06](TASK_PROMPTS/T-MIG-P3-06.md) | CI scaffold e verifica regressione legacy | T-MIG-P3-01, T-MIG-P3-02, T-MIG-P3-03, T-MIG-P3-04, T-MIG-P3-05 | G-MIG-3 con cargo test core+Slint e frontend lint/typecheck/build + smoke nativo; gate mancante NON DONE. |
| [T-MIG-P4-01](TASK_PROMPTS/T-MIG-P4-01.md) | API catalogo bundle e selezione | T-MIG-P3-06 | Parità analysis_configuration e audit hash/provenienza senza PHI. |
| [T-MIG-P4-02](TASK_PROMPTS/T-MIG-P4-02.md) | API conferma path e preflight input | T-MIG-P4-01 | Parità comportamento Slint e messaggi redatti, nessun apertura arbitraria da JS. |
| [T-MIG-P4-03](TASK_PROMPTS/T-MIG-P4-03.md) | Preparazione run e capture immutabile | T-MIG-P4-01, T-MIG-P4-02 | No race con selezione e nessun cambio schema dopo start; test request invarianti. |
| [T-MIG-P4-04](TASK_PROMPTS/T-MIG-P4-04.md) | Worker nativo e start LocalOnly | T-MIG-P4-03 | Esiti finali coerenti e no blocco UI; golden invariati. |
| [T-MIG-P4-05](TASK_PROMPTS/T-MIG-P4-05.md) | Canale progresso e annullamento bounded | T-MIG-P4-04 | No error per-record over IPC, terminal reliable; race tests PASS. |
| [T-MIG-P4-06](TASK_PROMPTS/T-MIG-P4-06.md) | Gestione esito, report handle e cleanup | T-MIG-P4-04, T-MIG-P4-05 | State machine parity and negative tests; TempDir ownership verificata. |
| [T-MIG-P4-07](TASK_PROMPTS/T-MIG-P4-07.md) | Riutilizzo indice JSONL, senza bulk transfer | T-MIG-P4-06 | No Vec<ValidationError> globale, no conversione tutte righe, cache bounded. |
| [T-MIG-P4-08](TASK_PROMPTS/T-MIG-P4-08.md) | Query filtrate e pagina risultati in Rust | T-MIG-P4-07 | Parità filtri Slint e nessun query da JS su 1M errori; limite payload rispettato. |
| [T-MIG-P4-09](TASK_PROMPTS/T-MIG-P4-09.md) | Dettagli, field guide e reference snapshot | T-MIG-P4-08 | Valori e diagnostici corrispondono al report originale e alla fonte Rust; niente cast number JS unsafe. |
| [T-MIG-P4-10](TASK_PROMPTS/T-MIG-P4-10.md) | Export, About e test bridge integrati | T-MIG-P4-06, T-MIG-P4-08, T-MIG-P4-09, T-MIG-P4-12 | Bridge LocalOnly+Full completo, export e provenance; test integrazione su synthetic. |
| [T-MIG-P4-11](TASK_PROMPTS/T-MIG-P4-11.md) | Rust adapter Full multi-tracciato su ValidationService esistente | T-MIG-P4-06, T-MIG-P2-11 | Test end-to-end bridge Full con fixture sintetica e confronto outcome/summary/JSONL canonici rispetto a invocazione diretta al servizio. |
| [T-MIG-P4-12](TASK_PROMPTS/T-MIG-P4-12.md) | Progresso, cancel e cleanup affidabili per Full | T-MIG-P4-11, T-MIG-P4-05 | Dimostrare con test di integrazione nessun report falso, nessun run zombie, zero trasferimento bulk e stato finale coerente. |
| [T-MIG-P5-01](TASK_PROMPTS/T-MIG-P5-01.md) | Shell React responsive e tema | T-MIG-P4-10 | Screenshot e test interazione su Linux/Windows; attributi HTML semantic. |
| [T-MIG-P5-02](TASK_PROMPTS/T-MIG-P5-02.md) | Flow/schema/trace state machine UI | T-MIG-P5-01 | Parità dei test configuration schema_binding; no norm logic TS. |
| [T-MIG-P5-03](TASK_PROMPTS/T-MIG-P5-03.md) | Conferma file e picker sicuro | T-MIG-P5-02 | Il frontend non legge byte fixed-width e non presenta sample come file completo. |
| [T-MIG-P5-04](TASK_PROMPTS/T-MIG-P5-04.md) | Selezione formati di report | T-MIG-P5-02 | Parità utenti e nessun writer o conversione JS. |
| [T-MIG-P5-05](TASK_PROMPTS/T-MIG-P5-05.md) | Run screen e progress throttled | T-MIG-P5-03, T-MIG-P5-04 | UI reattiva 60s synthetic run, bounded memory, messaggi coerenti con Rust. |
| [T-MIG-P5-06](TASK_PROMPTS/T-MIG-P5-06.md) | Cancel, stato terminale e old report | T-MIG-P5-05 | Nessuna falsa conformità o report nuovo dopo cancel, test state machine. |
| [T-MIG-P5-07](TASK_PROMPTS/T-MIG-P5-07.md) | Informazioni/provenienza e licenze | T-MIG-P5-01, T-MIG-P5-06 | Dati di provenienza corretti; Slint notice rimane dove richiesto per legacy, Tauri notices corrette. |
| [T-MIG-P5-08](TASK_PROMPTS/T-MIG-P5-08.md) | Gate parità configurazione/worker/UX | T-MIG-P5-01, T-MIG-P5-02, T-MIG-P5-03, T-MIG-P5-04, T-MIG-P5-05, T-MIG-P5-06, T-MIG-P5-07, T-MIG-P5-10 | Parità configurazione/worker LocalOnly+Full, keyboard/UX responsive approvata. |
| [T-MIG-P5-09](TASK_PROMPTS/T-MIG-P5-09.md) | React: selezione Full con input esplicito per tracciato | T-MIG-P5-03, T-MIG-P4-12 | La UI invia solo richieste legittime e tipizzate; blocca input incoerente; nessun latest implicito, nessuna inferenza normativa lato React. |
| [T-MIG-P5-10](TASK_PROMPTS/T-MIG-P5-10.md) | React: monitor Full e stati terminali multi-pass | T-MIG-P5-09, T-MIG-P5-06 | Confronto semantico col servizio Full; nessuna presentazione di conformity falsa o percentuali inventate. |
| [T-MIG-P6-01](TASK_PROMPTS/T-MIG-P6-01.md) | DataGrid DOM virtualizzata paginata | T-MIG-P5-08 | DOM e memoria bounded su dataset sintetici, preferire libreria scelta in DEC-MIG-08 con test. |
| [T-MIG-P6-02](TASK_PROMPTS/T-MIG-P6-02.md) | Cinque filtri paritari e query stale safe | T-MIG-P6-01 | Risultati esattamente uguali a query Rust e nessun sort/FTS extra. |
| [T-MIG-P6-03](TASK_PROMPTS/T-MIG-P6-03.md) | Dettaglio diagnostico e selezione offset | T-MIG-P6-01, T-MIG-P6-02 | Valore/descrizione provenienti da Rust, nessun dangerouslySetInnerHTML. |
| [T-MIG-P6-04](TASK_PROMPTS/T-MIG-P6-04.md) | Field guide e posizione/schema | T-MIG-P6-03 | Test snapshot e oracle su schemi sintetici; nessuna regola ricostruita in TS. |
| [T-MIG-P6-05](TASK_PROMPTS/T-MIG-P6-05.md) | Dominio ammesso statico e range | T-MIG-P6-04 | Parità field_guide; JSON string per big integers; UI non calcola domini. |
| [T-MIG-P6-06](TASK_PROMPTS/T-MIG-P6-06.md) | Dominio reference/evidenza provider | T-MIG-P6-05 | Reference snapshot consistent; no “valid” when missing/unverified data. |
| [T-MIG-P6-07](TASK_PROMPTS/T-MIG-P6-07.md) | Stati di risultati, malformed e partial | T-MIG-P6-01, T-MIG-P6-03, T-MIG-P6-06 | Non presentare conformità globale per LocalOnly o dati incompleti; error states accessibili. |
| [T-MIG-P6-08](TASK_PROMPTS/T-MIG-P6-08.md) | Export UI con receipt e overwrite | T-MIG-P6-07 | Export sempre intero e byte-identico al core; due pubblicazioni non dette atomic pair. |
| [T-MIG-P6-09](TASK_PROMPTS/T-MIG-P6-09.md) | Verifica numerica e golden report | T-MIG-P6-02, T-MIG-P6-03, T-MIG-P6-04, T-MIG-P6-05, T-MIG-P6-06, T-MIG-P6-07, T-MIG-P6-08 | G-MIG-6 con matrix completa risultati/export e riferimento al SHA, assenti feature future. |
| [T-MIG-P6-10](TASK_PROMPTS/T-MIG-P6-10.md) | React Full: risultati cross-trace, detail ed export canonico | T-MIG-P6-09, T-MIG-P5-10 | Parità Full e LocalOnly dell’oracolo Rust provata; nessuna logica cross-record ricreata in React. |
| [T-MIG-P7-01](TASK_PROMPTS/T-MIG-P7-01.md) | Negative tests IPC e payload fuzz | T-MIG-P6-09 | Server rejects typed and no panic; negative test evidence. |
| [T-MIG-P7-02](TASK_PROMPTS/T-MIG-P7-02.md) | XSS, content injection e CSP offline | T-MIG-P7-01 | No execution/no remote call; DOM text-only, policy production inspected. |
| [T-MIG-P7-03](TASK_PROMPTS/T-MIG-P7-03.md) | Privacy DOM/log/temp/cache e crash recovery | T-MIG-P7-01, T-MIG-P7-02 | Policy/limitazioni dichiarate; dati veri restano gated DEC-MIG-09. |
| [T-MIG-P7-04](TASK_PROMPTS/T-MIG-P7-04.md) | Benchmark 300k/1M e profiling end-to-end | T-MIG-P7-01 | Mediana/p95/memoria confondenti riportati; soglie già ratificate soddisfatte o blocker. |
| [T-MIG-P7-05](TASK_PROMPTS/T-MIG-P7-05.md) | Race/run lifecycle/cleanup/responsiveness | T-MIG-P7-01 | No stale view masquerading current, no leak unbounded; shutdown duration measured. |
| [T-MIG-P7-06](TASK_PROMPTS/T-MIG-P7-06.md) | A11y, keyboard, DPI e localizzazione | T-MIG-P7-02 | Parità di flussi e contrasti ratificati; pixel compare non identico cross-engine. |
| [T-MIG-P7-07](TASK_PROMPTS/T-MIG-P7-07.md) | Test regressione core/CLI/PERSIST | T-MIG-P7-03, T-MIG-P7-04, T-MIG-P7-05 | Golden byte-stable Linux/Windows; G-MIG-7 vietato se regressioni senza decisione. |
| [T-MIG-P7-08](TASK_PROMPTS/T-MIG-P7-08.md) | Gate sicurezza/performance e release candidate | T-MIG-P7-01, T-MIG-P7-02, T-MIG-P7-03, T-MIG-P7-04, T-MIG-P7-05, T-MIG-P7-06, T-MIG-P7-07, T-MIG-P7-09 | Gate security/performance include scorecard Full end-to-end con 300k/1M sintetici. |
| [T-MIG-P7-09](TASK_PROMPTS/T-MIG-P7-09.md) | Qualifica Full end-to-end Linux/Windows con fixture sintetiche | T-MIG-P7-03, T-MIG-P7-04, T-MIG-P7-05, T-MIG-P6-10 | Scorecard Full PASS con evidenze e SHA, oppure G-MIG-7 BLOCKED; nessun waiver implicito. |
| [T-MIG-P8-01](TASK_PROMPTS/T-MIG-P8-01.md) | CI dual Rust + TypeScript senza abbassare gate | T-MIG-P7-08 | QG-1/QG-2 con test frontend, cargo workspace, TS lint/typecheck, Windows MSVC. |
| [T-MIG-P8-02](TASK_PROMPTS/T-MIG-P8-02.md) | Audit licenze/SBOM/lock/asset | T-MIG-P8-01 | Versioni lock e liste licenze, policy vulnerability/exception, asset local only. |
| [T-MIG-P8-03](TASK_PROMPTS/T-MIG-P8-03.md) | Verifier artefatti UI Linux distinto da CLI | T-MIG-P8-01, T-MIG-P8-02 | UI artifact verifier eseguito nativo; verifier CLI immutato e verde. |
| [T-MIG-P8-04](TASK_PROMPTS/T-MIG-P8-04.md) | Windows NSIS offline e installazione WebView2 | T-MIG-P8-01, T-MIG-P8-02 | NSIS offlineInstaller+perMachine Windows11 senza rete; update/uninstall/reinstall conservano dati sintetici per-account. |
| [T-MIG-P8-05](TASK_PROMPTS/T-MIG-P8-05.md) | UI CachyOS: build, smoke e runtime locale | T-MIG-P8-01, T-MIG-P8-03 | Build personale e smoke sulla macchina CachyOS reale, non release GUI Linux ufficiale. |
| [T-MIG-P8-06](TASK_PROMPTS/T-MIG-P8-06.md) | Immutabilità, checksum e provenienza CI | T-MIG-P8-03, T-MIG-P8-04, T-MIG-P8-05 | SHA reproducibility policy effettivamente testata, no claim byte-identical unproven WebView packaging. |
| [T-MIG-P8-07](TASK_PROMPTS/T-MIG-P8-07.md) | Smoke multi-OS estrazione e casi negativi | T-MIG-P8-04, T-MIG-P8-05, T-MIG-P8-06 | Risultato registrato per ciascun OS con versioni WebView e limiti, non una CI generica. |
| [T-MIG-P8-08](TASK_PROMPTS/T-MIG-P8-08.md) | Gate CI/release/artifact e policy CLI intatta | T-MIG-P8-01, T-MIG-P8-02, T-MIG-P8-03, T-MIG-P8-04, T-MIG-P8-05, T-MIG-P8-06, T-MIG-P8-07 | G-MIG-8 solo con URL CI verificabili e prove run artifact, o DONE_UNVERIFIED. |
| [T-MIG-P9-01](TASK_PROMPTS/T-MIG-P9-01.md) | Scorecard parità firmata | T-MIG-P8-08 | Signoff parità LocalOnly + evidenza Full obbligatoria separata P9-08. |
| [T-MIG-P9-02](TASK_PROMPTS/T-MIG-P9-02.md) | Piano rollback e coesistenza verificato | T-MIG-P9-01 | Prova rollback nativa e versioned release; no deleting legacy yet. |
| [T-MIG-P9-03](TASK_PROMPTS/T-MIG-P9-03.md) | Cutover release controllato UI | T-MIG-P9-01, T-MIG-P9-02, T-MIG-P9-08 | Tag/version/source SHA, SHA256SUMS, manifests and notices; gate umano cutover. |
| [T-MIG-P9-04](TASK_PROMPTS/T-MIG-P9-04.md) | Rimozione Slint in fase separata | T-MIG-P9-03 | No Slint in product graph, archived source/tag accessible, per-OS native CI green. |
| [T-MIG-P9-05](TASK_PROMPTS/T-MIG-P9-05.md) | Aggiornamento ARCH/DET/PLAN/ADR e status | T-MIG-P9-03, T-MIG-P9-04 | QG-8: documenti-to-update vuoto, aggiornamenti revisionati e collegati ad ADR. |
| [T-MIG-P9-06](TASK_PROMPTS/T-MIG-P9-06.md) | Release finale e prove cross-OS post cleanup | T-MIG-P9-04, T-MIG-P9-05 | Suite native Linux e Windows verified final SHA; non riusare test pre-rimozione. |
| [T-MIG-P9-07](TASK_PROMPTS/T-MIG-P9-07.md) | Gate finale e handoff maintenance | T-MIG-P9-01, T-MIG-P9-02, T-MIG-P9-03, T-MIG-P9-04, T-MIG-P9-05, T-MIG-P9-06, T-MIG-P9-08 | G9 soltanto con LocalOnly e Full signoff, Windows11 NSIS offline, CachyOS smoke, rollback, diff final e change log. |
| [T-MIG-P9-08](TASK_PROMPTS/T-MIG-P9-08.md) | Signoff autonomo Full e controllo visibilità capability | T-MIG-P9-01, T-MIG-P7-09 | Accettazione umana esplicita di Full, con report referenziato alla stessa source SHA del cutover. |

## 10. Quality gate e prove minime

| Gate | Quality gate originale/conservato | Evidenza migrazione richiesta |
|---|---|---|
| G-MIG-0 | QG-3/QG-8; N-24 | ADR formalmente recepite + baseline SHA/CI + PERSIST08 decision + LocalOnly/Full obbligatorio + policy UX/security/distribuzione |
| G-MIG-1 | QG-2/QG-5 | WebKitGTK su **PC CachyOS/Hyprland reale** + **installer NSIS offline perMachine Windows11 clean-room** già in P1 (T-MIG-P1-07), senza rete; non basta check o CI Ubuntu |
| G-MIG-2 | QG-3/QG-6, N-31 | contratto wire versionato **LocalOnly + Full**, bounded paging/eventi, lifecycle, minimizzazione PHI, permissions/CSP approved |
| G-MIG-3 | QG-1/QG-2/QG-3 | Slint+CLI verdi, nuovo Tauri scaffold, React lint/typecheck/build, CI smoke Linux+Windows |
| G-MIG-4 | QG-1/QG-3/QG-4/QG-6 | worker/cancel/results/domain/export **LocalOnly+Full** backend, DTO negative tests, zero bulk report IPC |
| G-MIG-5 | QG-1/QG-7 | redesign responsive, config/preflight/run **LocalOnly+Full**/outcomes/About e accessibilità, su build browser locale/offline |
| G-MIG-6 | QG-4/QG-6/QG-7 | filtri/dettaglio/domini/export completi **LocalOnly+Full**, goldens e comparazione per SHA |
| G-MIG-7 | QG-1..4/QG-6/QG-7 + N-31 | XSS/CSP/PHI/races, qualifica **Full end-to-end**, benchmark 300k/1M p95/RSS, native regression SQLite senza dati reali |
| G-MIG-8 | QG-2/QG-5/QG-8, N-23 | Windows11 MSVC CI + NSIS offline perMachine install/update/uninstall/reinstall con dati conservati; smoke **CachyOS personale**; core/CLI CI cross-OS, allowlist separate, SHA256SUMS/provenance e politica build locale coerente con N-23 |
| G-MIG-9 | QG-1..8 | signoff **LocalOnly+Full**, cutover/rollback eseguiti, Slint removed AFTER approval, docs/ADR e status sincronizzati, CI final SHA |

**QG-1:** `cargo fmt/clippy/test --workspace` e frontend CI; **QG-2:** Linux + Windows MSVC native **su stesso SHA**, no semplice check cross-GNU; **QG-3:** layer constraints; **QG-4:** goldens byte-identical e ordine; **QG-5:** dependencies/artifact verify; **QG-6:** memoria streaming; **QG-7:** semantic visual parity OS-specific, non pixel-identità RGB cross-WebView; **QG-8:** doc sync e registri risolti.

### 10.1 Performance: disciplina di misura, non promesse di soglie arbitrarie
- Baseline stessa fixture, hash, OS/hardware, warm/cold, build release Slint/Tauri, WebView version, CPU, resident memory, heap Rust e frontend dove misurabile; distinguere RSS processo UI da memoria del core.
- Fixture sintetiche 10k/100k/300k/1M diagnostici; scroll virtualizzato, p50/p95 pagina 50/100, filtri cold/warm, cancel, export e input empty; misurare indicatori di disponibilità UI.
- Non trasferire indice globale o error list in JS. Rust `ReportIndex` O(n) e scan filtri O(n) sono **debito attuale**, non promessa di memoria costante; va mantenuto un budget esplicito e benchmarkato.
- **Soglie finali non fissate in questo piano**: vanno ratificate in P0/P2 dopo misure e insieme al requisito prodotto. Una prova non eseguita resta `NOT_MEASURED`, non “veloce”.

### 10.2 Matrice di test parità più critica
| Scenario | Oracle | Failure da bloccare |
|---|---|---|
| Flow/version/trace e bundle LocalOnly/Full | schema catalog/compiled/hash preflight | latest implicito, mix embedded+external |
| Input BOM/file preflight | InputPreview Rust (fino 64 KiB) | modificare byte, campione spacciato per file completo |
| Run LocalOnly + Full | ValidationService/validate_full, golden rust | Full UI mancante oppure falsa global conformity |
| Outcomes | WorkerController legacy | Cancelled/Error presentati come valido, parzialità nascosta |
| 5 filtri | ResultsFilter Rust | casefold/trim automatico, selection stale |
| 1M JSONL | ReportIndex/results e test benchmark | 1M DTO in JS, crash UI, cache illimitata |
| Domain/field guide | compiled schema + captured reference | off-by-one, rounding >u128, reference cambiata dopo run |
| Export | writers Rust e receipt | export filtrato implicito, falso multi-file atomicity |
| PERSIST/SQLite/CLI | native regression baseline, golden | approval involontario/commit ambiguo/byte change |
| Security | CSP capabilities deny tests | XSS, command scope abuse, PHI in console/log/cache |

## 11. Protocollo agentico vincolante

Usare `MIGRATION_AGENT_PROTOCOL.md` insieme a GOV corrente. Singolo agente, sequenziale, senza subagenti; task READY → PREFLIGHT → IMPLEMENT → TEST → CI/GATE → STATUS → HANDOFF → next READY **quando autorizzato**. Una decisione mancante richiede BLOCCO e firma umana; il codice non approva un ADR.

**Prima sessione raccomandata:** `T-MIG-P0-01` (fotografia baseline), non lo scaffold Tauri. **Stop immediato** se l'attuale working tree è dirty e non si può separare dal lavoro T-PERSIST; niente `git reset`, `git clean`, force push, rebase di rami altrui o commit altrui per “ripulire”.

## 12. Consuntivo, handoff e change control

`MIGRATION_IMPLEMENTATION_STATUS.md` contiene task states, git SHA/branch, test/CI, gate, decisioni ratificate, blocker, docs da aggiornare, change log, handoff (template in file status). Il preflight resta fonte con la sua impronta; non si riclassifica una feature basandosi su nome ADR. Ogni modifica al piano richiede change request, impatto, approval e versione nuova.

**Handoff obbligatorio:** data/time; task e stato; file, comportamento e API toccate; prove eseguite e non eseguite; URL/sha CI; decisioni; problemi/risks; branch/commit/working tree; prossima task READY con dipendenze.

## 13. Criteri finali di completamento

Migrazione completata solo se: GUI React/Tauri conserva **l'intera parità osservata** nello scope LocalOnly, compilata e funzionante sulle piattaforme ratificate, rispetta privacy e runtime policy, non degrada engine/CLI/persistence, non altera byte golden, mantiene cronologia dei report e invarianti worker, produce artefatti CI versionati con rollback provato, Slint dismessa dopo approvazione separata, documenti/stato sincronizzati e tutte le decisioni/gate associate chiuse con evidenze. L'uso di dati clinici reali resta bloccato da requisiti **indipendenti** (DEC-MIG-09/ADR002), non viene autorizzato dal successo della UI.

## 14. Fonti, evidenze e limiti

**[AUDIT]** `SOURCES/SLINT_TO_TAURI2_PREFLIGHT_REPORT.md` rev. 2026-10-30, §§1–15; PREHEAD `fe5ada8…` più WIP drift.
**[DESIGN]** `docs/ARCHITECTURE.md` §39D/§44/§45; `docs/DETAILED_DESIGN.md` Addendum B/§166–174; `docs/MASTER_IMPLEMENTATION_PLAN.md` §§4,11,14–15; ADR-PERSIST-001/002.
**[EXT]** documentazione ufficiale Tauri2: https://v2.tauri.app/start/prerequisites/ ; https://v2.tauri.app/security/capabilities/ ; https://v2.tauri.app/security/csp/ ; https://v2.tauri.app/develop/calling-rust/ ; https://v2.tauri.app/reference/webview-versions/ ; https://v2.tauri.app/distribute/windows-installer/ ; TanStack Virtual https://tanstack.com/virtual/latest/docs/introduction . Consultata a data documento, **non** pin release.

Le scelte su librerie frontend specifiche sono approvate, ma Node/Tauri patch, pin, WebView runtime, soglie performance e DTO/security concreti vanno qualificati e ratificati, non sono già implementate. Stato della CI presente nel report è HISTORY o NOT_VERIFIED per la revisione finale audit.

## 15. Riconciliazione finale della versione 1.1 — obblighi di esecuzione

- **Scelte utente approvate:** DEC-MIG-01/02/03/04/05/07/08/09. Vedi `MIGRATION_DECISION_REGISTER.md` e ADR-MIG-001..004. **Nessun gate tecnico superato in questo kit**.
- **Bloccante ancora aperto:** DEC-MIG-06 / CNF-MIG-06: SHA Git stabile e CI post-PERSIST08/isolamento, senza reset/clean del lavoro preesistente.
- **Contratti da chiudere da agenti e approvare ai gate:** versione Node/Rust/Tauri2, frontend dependencies + licensing, DTO/versioning/hard limits/Channel, protocollo delle finestre, CSP/capabilities, budget UX/memoria, installazione NSIS reale e machine/account ACL. L'agente non può inventare decisioni normative o chiudere gate senza prove.
- **Non confondere** `Full` (funzione del servizio già presente) con certificazione clinica completa di CAD/SIOC: fixture di sviluppo sintetiche, capability dichiarata, no real data autorizzato.
- **Backup-B** è policy approvata, ma la implementazione reale del backup/restore verificato e dell'upgrade SQLite va pianificata separatamente in PERSIST con un gate reale, non finta come lavoro consegnato dalla migrazione UI.
- **Checklist prima del cutover:** LocalOnly/Full QA e screenshot/a11y, stress test 300k/1M, installer Windows11 offline per-machine testato come utente non admin + reinstallazione conservativa e isolamento 2 account, smoke CachyOS, CLI/core/golden su SHA finale, rollback provato e signoff umano.
- **Evidenze:** stato e prove vive nel `MIGRATION_IMPLEMENTATION_STATUS.md`, manifest SHA dell'artefatto e CI. I documenti sorgente storici allegati non rimpiazzano repository corrente.
