# MASTER_IMPLEMENTATION_PLAN.md
## Regulatory Fixed-Width Validation Engine — piano operativo di implementazione

| | |
|---|---|
| **Versione del piano** | 1.1 (revisione architetturale approvata 2026-10-16) |
| **Data** | 2026-10-16 |
| **Fonti primarie** | `regulatory_fixed_width_validator_ARCHITECTURE.md` (rev. fino al 2026-10-09), `regulatory_fixed_width_validator_DETAILED_DESIGN.md` (rev. fino al 2026-10-09) |
| **Stato del codice** | Non accessibile al redattore: si assume **non iniziato** (vedi §5) |
| **Registro di stato** | `IMPLEMENTATION_STATUS.md` (file separato, vedi §15) |

> Questo documento è il riferimento **stabile** della strategia. Non contiene consuntivo: lo stato effettivo vive solo in `IMPLEMENTATION_STATUS.md`.

---

## 1. Scopo, ambito e modalità d'uso

**Scopo.** Guidare, controllare e rendere verificabile l'implementazione dell'intero progetto, dalla formalizzazione normativa alla distribuzione, usando agenti AI con finestra di contesto limitata.

**Ambito.** Motore di validazione generico, CLI, interfaccia desktop (Slint), export XLSX, schema tooling, CI e release per Linux x86_64 e Windows x86_64. Non è in ambito: REST API, database server obbligatorio, dataset temporali generici e dashboard analytics (rinviati, vedi §13.5). **La persistenza core delle identità e degli snapshot fra invii è ora requisito in ambito**, con driver sostituibili e policy versionate (§20).

**Come si usa in una sessione.** Non iniettare tutto il documento. Pacchetto minimo di contesto:

1. §4 (decisioni non negoziabili) e §14 (`AI_AGENT_EXECUTION_PROTOCOL`);
2. la scheda della fase attiva (§9) e la scheda della task attiva (§10);
3. le sole sezioni dei documenti sorgente citate dalla task (§19 dà la mappa);
4. `IMPLEMENTATION_STATUS.md`.

**Convenzioni di identificativo** (stabili, mai riutilizzati): `P<n>` fase · `T-P<n>-<nn>` task · `M<n>` milestone · `G<n>` gate · `N-<nn>` vincolo non negoziabile · `DEC-<nn>` decisione aperta · `CNF-<nn>` conflitto tra fonti · `RSK-<nn>` rischio · `QG-<nn>` quality gate · `STD-*` blocchi standard.

**Stati di una task:** `NOT_STARTED` · `IN_PROGRESS` · `BLOCKED` · `DONE` (con evidenze) · `DONE_UNVERIFIED` (implementata, verifiche non eseguite o non disponibili).

**Controllo delle modifiche al piano.** Il piano cambia solo con una voce nel *Plan change log* di `IMPLEMENTATION_STATUS.md` e approvazione umana. **La revisione 1.1 è autorizzata dall’autore il 2026-10-16** per sincronizzare ARCH/DET e introdurre persistenza core (ADR-PERSIST-001); l’eccezione è registrata nel changelog dello starter kit. Un agente non effettua ulteriori modifiche autonome.

---

## 2. Fonti di verità e gerarchia documentale

| Priorità | Fonte | Ruolo |
|---|---|---|
| 1 | `ARCHITECTURE.md` (rev. 2026-10-16) | Source of truth delle decisioni architetturali e persistenza core |
| 2 | `DETAILED_DESIGN.md` (rev. 2026-10-16) e `PERSISTENCE_CORE_SPEC.md` | Contratti implementativi, storage e sessioni; concretizzano l'architettura |
| 3 | Repository e codice (se accessibili) | Evidenza dello stato reale; non autorizza a ignorare le specifiche |
| 4 | Conoscenza esterna | Solo per verificare API, dipendenze, licenze, toolchain. Sempre marcata **[EXT]** e distinta dalle decisioni dei documenti |
| 5 | Documenti normativi futuri (`SCHEMA_AUTHORING.md`, `REFERENCE_DATA_SPEC.md`) | Prodotti in P0; diventano fonte per tutto ciò che il design lascia intenzionalmente aperto (DET §106) |

**Regole.**

- Conflitto tra architettura e dettaglio: l'agente si ferma sulle attività interessate, cita le sezioni, descrive l'impatto e chiede la decisione (§13.2, §14).
- Un'informazione normativa mancante non si inventa: si lascia una boundary esplicita (tipo, trait, policy, TODO documentato) — DET §146, §165.
- **[EXT] usate nel redigere questo piano** (da riverificare alla versione effettivamente adottata): `rust_xlsxwriter` ha la feature `constant_memory` e la modalità low-memory, e la feature `zlib` introduce una dipendenza C; Slint è utilizzabile in app desktop proprietarie con licenza royalty-free a condizione di attribuzione visibile; su GitHub Actions i minuti Windows dei repository privati consumano la quota al doppio del ritmo di Linux.

---

## 3. Visione sintetica del sistema e architettura di riferimento

> **Il motore sa come validare. Lo schema sa cosa è valido.** (ARCH §3, §44)

Un file `.txt` a record posizionali viene letto in streaming, ogni riga viene validata come `&[u8]` contro uno **schema compilato** (versionato, esterno al codice), gli errori escono verso un `ValidationSink`, il risultato è un summary. Le regole tra record e tra tracciati usano due passate con proiezione minima del contesto.

```text
CLI / UI (adapter)
      │  ValidationRequest, CancellationToken, ProgressObserver, ValidationSink
      ▼
ValidationService (application)
      │
SchemaRegistry → SchemaRepository(raw) → SchemaCompiler(lint+compile) → CompiledFlowSchema
      │
TraceValidator / FlowValidator (engine)
   RecordReader → RecordView → field pipeline
   (presence → alignment → datatype → constraints) → local rules
   → IdentityExtractor → ContextProjection → ContextStore
      │ pass 2: cross-record rules
      ▼
ValidationSink: JSONL (canonico) · XLSX (analisti) · CSV · human · summary.json
```

**Moduli** (DET §4 e §166–174): `domain`, `schema`, `input`, `engine`, `validators`, `context`, `persistence`, `reference_data`, `report`, `application`, `cli`; più `ui` (crate separato, ARCH §39D). **Direzione delle dipendenze** (DET §5): `domain` ← `schema`/`reference_data`/`input` ← `validators` ← `context` ← `engine` ← `application` ← `cli`/`ui`. La `persistence` core implementa porte dipendenti solo da `domain`/compiled projection e viene orchestrata da `application` senza ciclo di dipendenza.

**Distribuzione** (ARCH §39A–C): eseguibili nativi Linux e Windows costruiti solo dalla CI, autosufficienti; schemi e reference data come bundle di dati con ciclo di vita separato e snapshot incorporato di default.

---

## 4. Decisioni non negoziabili

Ogni riga è un vincolo da rispettare. Modificarne uno richiede una nuova decisione architetturale esplicita (ARCH §44).

| ID | Vincolo | Fonte |
|---|---|---|
| N-01 | Separazione rigorosa motore generico / conoscenza normativa; vietato `if flow == "SIOC"` e `if field_id == "2.29"` nel core (salvo test circoscritti) | ARCH §3, §37; DET §2 |
| N-02 | Rust; library API stabile; CLI e UI sono adapter; il core non dipende da `clap` né da GUI | ARCH §4, §30, §32; DET §3.1, §5, §162 |
| N-03 | Schema esterno al codice compilato, versionato; chiave `flow_id + schema_version + trace_id`; mai «ultima versione» implicita | ARCH §5.1–5.2, §37 |
| N-04 | Raw schema ≠ compiled schema; l'engine accetta solo `CompiledFlowSchema`, prodotto esclusivamente da `SchemaCompiler` | DET §3.2, §8.2, §131, §157 |
| N-05 | Elaborazione byte-oriented (`&[u8]`), zero-copy, nessuna `String` come rappresentazione primaria della riga; nessun `trim()` implicito | ARCH §5.6, §15; DET §28–32, §160 |
| N-06 | Streaming: nessun caricamento dell'intero file, nessun `Vec` globale di errori; errori verso `ValidationSink` | ARCH §16; DET §3.4, §116, §156 |
| N-07 | Niente `required: bool` né flag normativi mutabili; le regole si valutano contro un contesto già disponibile, indipendentemente dall'ordine dei campi | ARCH §5.5, §9, §20 |
| N-08 | `Vec<FieldSpec>` per l'ordine fisico, `HashMap` solo per il lookup | ARCH §5.3 |
| N-09 | DSL come AST controllato; vietati `eval`, scripting, codice dinamico | ARCH §11 |
| N-10 | Logica a tre valori (`TruthValue::Unknown`); un valore non disponibile non è `false` | DET §19 |
| N-11 | Violazione normativa ≠ errore di esecuzione: `Ok(CompletedInvalid)` vs `Err(ValidationRunError)`; `Cancelled` è un esito controllato | DET §3.3, §59–61, §159 |
| N-12 | Ordine degli errori deterministico e contrattuale (trace → riga → campo → stage → regola) | DET §78–79, §161 |
| N-13 | Nessuna nota in linguaggio naturale diventa regola senza revisione; nessun LLM nel runtime normativo | ARCH §5.7, §36; DET §144 |
| N-14 | Dato normativo mancante: boundary esplicita, mai assunzione | DET §146, §165 |
| N-15 | Cross-record in due passate; il Context Index conserva solo i campi richiesti; `ProjectedRecord` è l'unico dato posseduto oltre la riga | ARCH §21; DET §68–73 |
| N-16 | Report di validazione e log applicativo sono canali distinti | ARCH §27; DET §100 |
| N-17 | Posizioni: one-based nella fonte normativa e nel report agli analisti, zero-based/end-exclusive nel runtime; conversione unica e testata | ARCH §7.3; DET §7, §52 |
| N-18 | Nessun default globale «tutti zeri = vuoto»; assenza descritta da `MissingValuePolicy` nello schema | ARCH §19; DET §14 |
| N-19 | Linux x86_64 e Windows x86_64 supportati dall'MVP; CI su entrambi; un cambiamento che passa solo su Linux non è completo | ARCH §39A; DET (sezione Linux/Windows) |
| N-20 | Binario autosufficiente: nessuna dipendenza runtime dall'host di build; path compile-time vietati a runtime; crate pure-Rust salvo decisione esplicita | ARCH §5.8, §39C.1 |
| N-21 | Modalità ibrida: snapshot incorporato + cartella esterna prioritaria; fallback a livello di bundle, mai misto; cartella incompleta o schema mancante = errore; `schema_format_version` verificato | ARCH §39C.2 |
| N-22 | JSONL UTF-8 senza BOM, `\n`, **identico byte per byte** tra OS; XLSX formato principale per gli analisti (celle di dato come testo); CSV solo RFC 4180; timestamp UTC | ARCH §39C.3; DET Addendum A.2 |
| N-23 | Release solo da CI, immutabili, `SHA256SUMS`; Windows `msvc` sul runner Windows; nessuna firma Authenticode; tre ambiti (sorgente, binari, dati) | ARCH §39B.2, §39C.4–5 |
| N-24 | UI: Slint, eseguibile separato, renderer software di riferimento, font incorporati, token di design unici, attribuzione Slint in «Informazioni», test visivi in CI; client di `ValidationService`, nessuna logica normativa | ARCH §39D; DET Addendum B |
| N-25 | Auditabilità: versione software, versione e hash schema, versioni reference data, input (nome, dimensione, SHA-256), opzioni, timestamp UTC | ARCH §35, §39C.3 |
| N-26 | Nessuna dipendenza, framework o database aggiunti «perché potrebbero servire» | Prompt di pianificazione §9 |
| N-27 | La formalizzazione dei tracciati regionali precede le attività di sviluppo (gate G0), salva autorizzazione P1/P2 anticipata | Prompt di pianificazione §4.4; A-02 |
| N-28 | Persistenza fra esecuzioni nel core generico, separata dal `ContextStore` effimero; policy normative esterne e driver storage sostituibili | ARCH §45; DET §166–174; ADR-PERSIST-001 |
| N-29 | Snapshot solo con preview e approvazione, commit atomico/rollback; sessioni CAD definitivamente approvate immutabili; isolamento namespace e tracciati mancanti non annullati | ARCH §45; ADR-PERSIST-001 |
| N-30 | SIOC: PK intra-trace già dichiarate FROZEN; audit persistente inter-invio non equivale automaticamente a errore sulla ripetizione cross-run | SIOC `record_key`; ADR-PERSIST-001 |
| N-31 | Persistenza minimizzata e protetta; niente record sanitari grezzi e identificativi in chiaro nello storage default; privacy by design, test sintetici | DET §172; ADR-PERSIST-001 |

---

## 5. Stato iniziale, evidenze e assunzioni

**Accesso al repository: nessuno.** Nessuno stato di codice, test, CI o configurazione Cargo è stato verificato dal redattore.

| Componente | Stato assunto | Evidenza |
|---|---|---|
| Tutti i moduli del motore (§3) | `NOT_STARTED` | Nessuna: assunzione dichiarata |
| CI, release, packaging | `NOT_STARTED` | Nessuna |
| Tracciati e reference data formalizzati | `NOT_STARTED` | I documenti dichiarano la serializzazione «da definire» (DET §12 nota iniziale, §106) |
| Prototipo UI «Risultati» (Slint) | `DONE_UNVERIFIED` | L'autore ha riferito che «builda»; i criteri di accettazione di ARCH §39D.6 **non** sono stati verificati; Linux/Windows non confrontati |
| Mockup UI (due schermate, font IBM Plex) | Esiste come artefatto di design | Fonte per T-P16-* |
| Nome commerciale | **RISOLTO — Lintra (DEC-03)** | package `lintra`, artefatti `lintra-*` |

**Assunzioni adottate.**

- **A-01** Nessun codice esiste nel repository di progetto.
- **A-02** Per N-27 la fase **P0** precede tutto il resto. Il lavoro di P1 (repository, CI) e dei tipi generici di P2 non dipende da contenuti normativi: potrebbe partire in parallelo a P0 solo con **autorizzazione esplicita dell'autore** (DEC-01). Senza autorizzazione, P1+ resta bloccato dal gate G0.
- **A-03** Si parte con un singolo package (ARCH §31); la conversione in workspace avviene in T-P16-01, quando la UI diventa un confine reale (ARCH §31, §39D.2).
- **A-04** I test del motore usano **schemi sintetici generici** (non normativi, dichiarati tali) finché i golden SIOC non esistono; nessun dato SIOC viene inventato.
- **A-05** CI su GitHub Actions; repository separati per sorgente, binari e dati (ARCH §39C.5).
- **A-06** Il toolchain Rust viene fissato in T-P1-02 con `rustup` e `rust-toolchain.toml` (ARCH §39B.1); la versione è una scelta operativa di quella task.

---

## 6. Mappa delle dipendenze

| Fase | Dipende da | Può procedere in parallelo con |
|---|---|---|
| P0 Formalizzazione normativa | Input ufficiali (DET §154) | — (ma vedi A-02) |
| P1 Fondamenta repository e CI | G0 (o autorizzazione A-02) | P0, P2 |
| P2 Dominio | P1 | P0 |
| P3 Schema (raw, compiled, linter, compiler) | P2, **G0** (JSON canonico) | P4 |
| P4 Input layer | P2 (`RecordView` richiede il tipo compilato di P3) | P3, P5 |
| P5 Error model, sink, summary | P2, P3 (parziale: `CompiledFieldSpec`) | P4 |
| P6 Validazione locale | P3, P4, P5 | — |
| P7 Application service | P6 | — |
| P8 CLI | P7 | P9, P10, P11 |
| P9 Vincoli semantici (range, date, decimal, regex) | P6, P7, G0 | P10, P11, P12 |
| P10 Rule DSL e presenza condizionale | P6, P7, G0 (sintassi DSL) | P9, P11, P12 |
| P11 Reference data | P7, G0 (formato dataset) | P9, P10, P12 |
| P12 Report XLSX e CSV | P5, P7; wiring CLI: P8 | P9, P10, P11 |
| P13 Cross-record e cross-trace | P10, P7, G0 (join, cardinalità, policy) | P14 |
| P14 Schema tooling | P3, G0 | P13 |
| P15 Packaging, autosufficienza, release | P8 (+P1); snapshot incorporato: P3 + schemi reali | P9–P14 |
| P16 Interfaccia desktop Slint | P7, P8, P12; spike: P1 (CI) | P9–P15 |
| P17 Hardening | P8–P13 | P16 |
| P18 Documentazione e manutenzione | tutte | — |

**Percorso critico:** G0 → P3 → P6 → P7 → P8 → P10 → P12 → P15 → M5.

**Ordine di sviluppo di ARCH vs DET.** ARCH §38 colloca i reference data prima della DSL (Fase 2 → 3); DET §150 colloca la DSL (G) prima dei reference data (H). P10 e P11 sono **indipendenti** nel grafo: entrambi gli ordini sono validi. Si veda CNF-02.

---

## 7. Strategia e principi di implementazione

1. **Fette verticali con contratto prima del codice.** Ogni task rispetta il contratto semantico dei documenti (DET §147: le signature possono adattarsi allo stile Rust solo se si preservano ownership, responsabilità e boundary, documentando la differenza).
2. **Il test fa parte della task.** Nessuna task è `DONE` senza i test indicati e le evidenze di esecuzione (§11).
3. **Una causa primaria per stage.** La pipeline di campo si ferma sul primo errore semanticamente primario (DET §40).
4. **Determinismo prima delle prestazioni.** Priorità: correttezza, testabilità, auditabilità, benchmark, ottimizzazione (ARCH §33).
5. **Fixture sintetiche per il motore, golden normativi per SIOC.** I golden SIOC arrivano da P0 (T-P0-08) e diventano il contratto regressivo principale (DET §111).
6. **Dipendenze minime e pure-Rust.** Ogni nuovo crate richiede una nota nel registro di stato e il controllo di §39C.1 (nessuna libreria di sistema).
7. **Task piccole ma coese.** Ogni task è eseguibile in una sessione, con file e responsabilità circoscritti.
8. **Parallelismo.**
   - *Tra task:* ammesso solo se i file non si sovrappongono e non condividono un contratto non ancora fissato.
   - *A runtime:* escluso dall'MVP (DET §115); valutato in T-P17-05 e adottato solo se (a) il guadagno è misurato, (b) il determinismo dell'output resta provato, (c) la complessità di ownership non annulla il vantaggio.
9. **Documentazione sincronizzata.** Una task che cambia un contratto pubblico aggiorna il documento sorgente pertinente o apre un `CNF-*` (non lo fa in silenzio).
10. **Nessun lavoro su ciò che è rinviato** (§13.5).

---

## 8. Roadmap globale e milestone

| Fase | Nome | Milestone | Priorità | Mappa su ARCH §38 | Mappa su DET §150 |
|---|---|---|---|---|---|
| P0 | Formalizzazione normativa | **M0** (gate G0) | Critica | prerequisito (DET §106–108, §154–155) | — |
| P1 | Fondamenta repository e CI | M1 | Alta | — | A (parziale) |
| P2 | Dominio e tipi condivisi | M1 | Alta | Fase 1 | A |
| P3 | Schema: raw, compiled, linter, compiler | M2 | Critica | Fase 1 | B |
| P4 | Input layer | M2 | Alta | Fase 1 | C |
| P5 | Error model, sink, summary | M2 | Alta | Fase 1 | D |
| P6 | Validazione locale di campo e record | M2 | Critica | Fase 1 | E |
| P7 | Application service | **M3** | Critica | Fase 1 | F |
| P8 | CLI | M3 | Alta | Fase 1 (CLI) | — |
| P9 | Vincoli semantici e tipi avanzati | M4 | Alta | Fase 2 | E (estensione) |
| P10 | Rule DSL e presenza condizionale | M4 | Critica | Fase 3 | G |
| P11 | Reference data | M4 | Alta | Fase 2 | H |
| P12 | Report XLSX e CSV | M4 | Alta | Fase 2 | D (estensione) |
| P13 | Cross-record e cross-trace | **M6** | Alta | Fase 4 | I |
| P14 | Schema tooling | M7 | Media | Fase 5 | — |
| P15 | Packaging, autosufficienza, release | **M5**, M9 | Alta | Fase 6, §39A–C | — |
| P16 | Interfaccia desktop (Slint) | M8 | Media | §39, §39D | J |
| P17 | Production hardening | M9 | Media | Fase 6 | — |
| P18 | Documentazione e manutenzione | M9 | Media | Fase 6 | — |

| Milestone | Definizione | Raggiunta quando |
|---|---|---|
| **M0** | Formalizzazione chiusa | G0 superato (T-P0-09) |
| **M1** | Repository e dominio | P1–P2 verdi su Linux e Windows |
| **M2** | Core locale | P3–P6: record validati end-to-end in test di libreria |
| **M3** | MVP locale con CLI | DET §151 soddisfatto; CLI funzionante |
| **M4** | Funzionalità locali complete | P9–P12: vincoli semantici, DSL, reference data, XLSX |
| **M5** | **MVP** (ARCH §43) | M4 + release CI Linux/Windows con verifica di autosufficienza |
| **M6** | Full flow | DET §152 soddisfatto |
| **M7** | Tooling schema | Importer, diff, hash canonico |
| **M8** | UI | Criteri di ARCH §39D.6 soddisfatti e UI distribuita |
| **M9** | 1.0 | Hardening, documentazione, manutenzione operativa |

**Sequenza raccomandata.** P0 → (P1 ‖ P2 se autorizzato) → P3 → (P4 ‖ P5) → P6 → P7 → P8 → P9 → P10 → P11 → P12 → P15 (parte iniziale) → P13 → P14 → P16 → P17 → P18. Il collaudo del prototipo UI (T-P16-00) richiede la CI di P1 e può essere anticipato a corsia parallela.

---

## 9. Schede dettagliate delle fasi

Formato di ogni scheda: **Obiettivo** · **Motivazione e priorità** · **Prerequisiti** · **Moduli/contratti** · **Task** · **Test minimi** · **Accettazione** · **DoD** · **Blocchi e dipendenze esterne** · **Milestone e passo successivo**.

### P0 — Formalizzazione normativa
- **Obiettivo:** esistono `SCHEMA_AUTHORING.md` (JSON Schema del flow schema canonico, sintassi DSL, presence policies, join, identity, reference data bindings, versioning, `schema_format_version`), la bozza di `REFERENCE_DATA_SPEC.md`, gli schemi canonici SIOC di bozza e l'elenco delle fixture; ogni voce di DET §106 è chiusa o rinviata con motivazione scritta.
- **Motivazione e priorità:** Critica. N-27; sblocca la forma JSON, `FieldType`, `AlignmentRule`, DSL, join, identità (DET §106–108, §155).
- **Prerequisiti:** input ufficiali di DET §154 (tracciati completi, documenti di accompagnamento, versioni/date di efficacia, esempi di file validi, codifiche esterne, chiavi di correlazione; per i dataset: file ufficiali, versione, fonte, regole di validità, storicizzazione).
- **Moduli/contratti:** nessun codice. Produce `docs/normative/*`, `docs/SCHEMA_AUTHORING.md`, `docs/REFERENCE_DATA_SPEC.md`, `schemas/sioc/<version>/*`.
- **Task:** T-P0-01 … T-P0-09.
- **Test minimi:** checklist di completezza per tracciato; ogni nota classificata; ogni campo con tipo, posizione, lunghezza e presenza.
- **Accettazione:** checklist G0 (T-P0-09) firmata dall'autore.
- **DoD:** M0 raggiunta; `IMPLEMENTATION_STATUS.md` riporta G0 superato.
- **Blocchi:** fonti ufficiali non disponibili (esterno); note ambigue (decisione umana, DEC-05).
- **Passo successivo:** P3 (e P1/P2 se già avviati).

### P1 — Fondamenta repository e CI
- **Obiettivo:** package con `lib.rs` e `main.rs`, toolchain fissato, `Cargo.lock` committato, `.gitattributes` che protegge fixture e golden, CI su Linux e Windows verde.
- **Motivazione e priorità:** Alta; ogni task successiva si verifica qui (N-19).
- **Prerequisiti:** G0 o autorizzazione A-02; repository GitHub creato.
- **Moduli/contratti:** root del package, `.github/workflows/ci.yml`, `docs/`, `IMPLEMENTATION_STATUS.md`.
- **Task:** T-P1-01 … T-P1-03.
- **Test minimi:** un test banale e un test che dipende dal path eseguito su entrambi i runner.
- **Accettazione:** workflow verde su `ubuntu` e `windows` con `--locked`.
- **DoD:** M1 (con P2).
- **Blocchi:** nessuno sul nome: DEC-03 risolta (`lintra`).
- **Passo successivo:** P2.

### P2 — Dominio e tipi condivisi
- **Obiettivo:** newtype di identificativi, `ByteRange`, `ValidationCode` con mappatura di serializzazione unica, `RecordIdentity`, tipi di summary e stato.
- **Motivazione e priorità:** Alta; è il modulo più stabile e meno dipendente (DET §122).
- **Prerequisiti:** P1; DEC-02 già risolta per la serializzazione dei codici.
- **Moduli/contratti:** `domain/*` (DET §6, §7, §35, §51, §57–60).
- **Task:** T-P2-01 … T-P2-03.
- **Test minimi:** newtype, `ByteRange` (nessun panic, `None` fuori range, overflow), conversioni di posizione.
- **Accettazione:** QG-1, QG-2.
- **DoD:** M1.
- **Blocchi:** DEC-02 risolta; il mapping deve essere testato e versionato.
- **Passo successivo:** P3.

### P3 — Schema: raw, compiled, repository, linter, compiler
- **Obiettivo:** da un file di schema canonico si ottiene un `CompiledFlowSchema` verificato, con tutti i problemi di schema riportati in forma aggregata.
- **Motivazione e priorità:** Critica; l'engine non accetta altro (N-04).
- **Prerequisiti:** P2; G0 (forma JSON canonica).
- **Moduli/contratti:** `schema/{raw,compiled,repository,compiler,linter,registry}`; DET §8–12, §62, §141–142.
- **Task:** T-P3-01 … T-P3-07.
- **Test minimi:** gap, overlap, ID duplicati, tipo sconosciuto, posizione/lunghezza invalide, `record_length` incoerente, conversione one→zero-based, issue aggregate e ordinate in modo deterministico.
- **Accettazione:** QG-1–QG-3.
- **DoD:** schema sintetico e (se disponibile) schema SIOC di bozza compilano o falliscono con issue attese.
- **Blocchi:** G0.
- **Passo successivo:** P4 ‖ P5.

### P4 — Input layer
- **Obiettivo:** lettura buffered di record fixed-width senza copie, con gestione LF/CRLF, BOM, encoding e rendering diagnostico dei valori.
- **Motivazione e priorità:** Alta (N-05).
- **Prerequisiti:** P2; per `RecordView` il tipo compilato di P3.
- **Moduli/contratti:** `input/*`; DET §26–34.
- **Task:** T-P4-01 … T-P4-05.
- **Test minimi:** LF, CRLF, spazi finali preservati, riga vuota, EOF senza newline, BOM, record più corto/lungo, byte non-UTF-8, nessun panic.
- **Accettazione:** QG-1–QG-3, QG-6.
- **DoD:** reader testabile con `BufRead` in memoria.
- **Blocchi:** DEC-12 (semantica riga vuota) risolvibile con default provvisorio dichiarato.
- **Passo successivo:** P6.

### P5 — Error model, sink, summary
- **Obiettivo:** `ValidationError` completo, `ErrorFactory`, `ValidationSink` (`VecSink`, `CompositeSink`, `JsonlSink`), summary builder, summary JSON, output atomico, harness golden.
- **Motivazione e priorità:** Alta: l'error report è una feature primaria (ARCH §24).
- **Prerequisiti:** P2; DEC-02; tipi compilati di P3.
- **Moduli/contratti:** `domain/errors.rs`, `domain/summary.rs`, `engine/error_factory.rs`, `report/{sink,jsonl,composite}`; DET §50–60, §103–104.
- **Task:** T-P5-01 … T-P5-07.
- **Test minimi:** campi stabili del JSON, posizioni one-based/inclusive, errore di scrittura = `Err(Report)`, ordine deterministico, `.part` rimosso su fallimento.
- **Accettazione:** QG-1–QG-4.
- **DoD:** un `ValidationError` costruito a mano viene scritto in JSONL byte-identico su due OS (verificato in CI).
- **Blocchi:** nessuno su DEC-02, ormai risolta.
- **Passo successivo:** P6.

### P6 — Validazione locale di campo e record
- **Obiettivo:** `RecordValidator` che, per un record, applica lunghezza → slicing → presenza (Required/Optional) → alignment → datatype → AllowedValues, estrae l'identità e produce errori con una sola causa primaria per stage.
- **Motivazione e priorità:** Critica.
- **Prerequisiti:** P3, P4, P5; per le definizioni lessicali dei tipi G0 (T-P0-07).
- **Moduli/contratti:** `validators/*`, `engine/{field_pipeline,record_validator,identity_extractor}`; DET §14–16, §37, §40–49.
- **Task:** T-P6-01 … T-P6-08.
- **Test minimi:** validi/invalidi/boundary per ogni validator; nessuna cascata di errori derivati; record corto senza slicing fuori range.
- **Accettazione:** QG-1–QG-4.
- **DoD:** `RecordValidator` verificato con schema sintetico.
- **Blocchi:** G0 per le definizioni lessicali.
- **Passo successivo:** P7.

### P7 — Application service
- **Obiettivo:** `ValidationService::validate` per scope Trace e modalità `LocalOnly`: preflight, loop streaming, progress, cancellazione, summary, `Ok`/`Err` corretti.
- **Motivazione e priorità:** Critica: punto unico che CLI e UI chiamano (DET §84).
- **Prerequisiti:** P6.
- **Moduli/contratti:** `application/*`, `engine/{trace_validator,preflight}`, `schema/registry`; DET §23–25, §75–76, §81–87, §92–94.
- **Task:** T-P7-01 … T-P7-06.
- **Test minimi:** file valido, invalido, schema non caricabile, dataset mancante, cancellazione, errore I/O alla riga N, errore del sink.
- **Accettazione:** QG-1–QG-4, QG-6.
- **DoD:** DET §151 soddisfatto a livello di libreria.
- **Blocchi:** —
- **Passo successivo:** P8.

### P8 — CLI
- **Obiettivo:** `validate`, `schema lint`, exit code documentati, Ctrl+C multipiattaforma, risoluzione directory senza default impliciti.
- **Motivazione e priorità:** Alta; chiude M3.
- **Prerequisiti:** P7.
- **Moduli/contratti:** `cli/*`; DET §95–97, §119, Addendum A.1 (livelli 1–3).
- **Task:** T-P8-01 … T-P8-06.
- **Test minimi:** exit code 0/1/2/130, percorsi con spazi, working directory diversa, riga di comando senza opzioni.
- **Accettazione:** QG-1–QG-3.
- **DoD:** **M3**.
- **Blocchi:** DEC-03 (nome binario); DEC-12 (`--bom strip`).
- **Passo successivo:** P9 ‖ P10 ‖ P11 ‖ P12.

### P9 — Vincoli semantici e tipi avanzati
- **Obiettivo:** `Range`, `Date(DateFormat)`, `Decimal(DecimalSpec)`, `Regex`, controlli di compatibilità vincolo/tipo nel compiler.
- **Motivazione e priorità:** Alta (ARCH §38 Fase 2; DoD ARCH §43 richiede enum/domain e date).
- **Prerequisiti:** P6, P7; formati e tipi da G0.
- **Moduli/contratti:** `validators/{range,date,decimal,regex}`; DET §13, §16, §136–139.
- **Task:** T-P9-01 … T-P9-06.
- **Test minimi:** boundary, anni bisestili, range solo su valore lessicalmente valido, regex invalida = `SchemaIssue`.
- **Accettazione:** QG-1–QG-4.
- **DoD:** golden per `E_OUT_OF_RANGE`, `E_INVALID_DATE`, `E_INVALID_DOMAIN` (nomi finali da DEC-02).
- **Blocchi:** G0 (formati data, decimali, set di caratteri).
- **Passo successivo:** P10.

### P10 — Rule DSL e presenza condizionale
- **Obiettivo:** `CompiledExpr`, `TruthValue`, `ExpressionEvaluator`, `PresenceState`, regole cross-field locali con contesto diagnostico.
- **Motivazione e priorità:** Critica: la DoD di ARCH §43 richiede *conditional required*.
- **Prerequisiti:** P6, P7; sintassi DSL da G0.
- **Moduli/contratti:** `schema/{expression,presence}`, `engine/expression_evaluator.rs`; DET §17–22, §47, §140.
- **Task:** T-P10-01 … T-P10-06.
- **Test minimi:** ogni operatore con true/false/unknown; indipendenza dall'ordine dei campi; `Conflict` non nascosto; regole in ordine di compilazione.
- **Accettazione:** QG-1–QG-4.
- **DoD:** esempio 2.29/2.2 di ARCH §11/§25 riprodotto con schema sintetico equivalente.
- **Blocchi:** G0.
- **Passo successivo:** P11 o P12.

### P11 — Reference data
- **Obiettivo:** provider, `LookupSet`, `PreparedReferenceData` caricato prima del parsing, normalizzazione dichiarata, constraint `ReferenceData`, versioni nell'audit.
- **Motivazione e priorità:** Alta; formato concreto dipende da G0 (DET §63, §108).
- **Prerequisiti:** P7; `REFERENCE_DATA_SPEC.md`.
- **Moduli/contratti:** `reference_data/*`, `validators/reference_data.rs`; DET §46, §63–65.
- **Task:** T-P11-01 … T-P11-06.
- **Test minimi:** dataset assente = `Err(ReferenceData)`; nessun trim/uppercase/zero-strip implicito; codici con zeri iniziali.
- **Accettazione:** QG-1–QG-4.
- **DoD:** un dataset reale (o di prova dichiarato) usato da una regola.
- **Blocchi:** dataset ufficiali (esterno).
- **Passo successivo:** P12.

### P12 — Report XLSX e CSV
- **Obiettivo:** `CsvSink` RFC 4180 e report XLSX (fogli `Riepilogo` ed `Errori`, celle di dato come testo, intestazione fissa, filtri, continuazione oltre il limite di righe) generato a fine analisi dal flusso JSONL.
- **Motivazione e priorità:** Alta: formato principale per gli analisti (N-22).
- **Prerequisiti:** P5, P7; wiring CLI: P8; crate validato in T-P12-02.
- **Moduli/contratti:** `report/{csv,xlsx}.rs`; ARCH §39C.3; DET Addendum A.2.
- **Task:** T-P12-01 … T-P12-05.
- **Test minimi:** `00123` e `05` riletti come stringhe identiche; ordine righe = JSONL; superamento limite righe per foglio; nessuna dipendenza C.
- **Accettazione:** QG-1–QG-5.
- **DoD:** **M4** (con P9–P11).
- **Blocchi:** licenza del crate da verificare [EXT] (DEC-06).
- **Passo successivo:** P15.

### P13 — Cross-record e cross-trace (contesto effimero; predisposizione per PERSIST)
- **Obiettivo:** `ValidationMode::Full` con join schema-driven, projection plan, context store in memoria, pass 2, policy esplicite su record correlato mancante e cardinalità.
- **Motivazione e priorità:** Alta.
- **Prerequisiti:** P10, P7; join, cardinalità e policy da G0.
- **Moduli/contratti:** `context/*`, `engine/flow_validator.rs`, `schema/projection.rs`; DET §23, §66–74, §77, §90–91, §133, §152.
- **Task:** T-P13-01 … T-P13-08.
- **Test minimi:** join zero/uno/molti; chiave composta senza collisioni; projection completa; memoria proporzionale ai soli campi proiettati.
- **Accettazione:** QG-1–QG-4, QG-6.
- **DoD:** **M6** (DET §152).
- **Blocchi:** G0 (join e policy reali) — senza evidenza normativa la task resta con boundary (DET §91, §165).
- **Passo successivo:** P14.

### P14 — Schema tooling
- **Obiettivo:** importer XLSX regionale → `RawFlowSchema`, lint avanzato, canonical schema e hash canonico, diff tra versioni, comandi `schema import|list` e `reference-data list`.
- **Motivazione e priorità:** Media; mai dentro il validation loop (DET §149).
- **Prerequisiti:** P3, G0.
- **Moduli/contratti:** `schema/importer.rs`; DET §102, §143; ARCH §13.
- **Task:** T-P14-01 … T-P14-05.
- **Test minimi:** importer su fixture XLSX; round-trip canonico; hash stabile rispetto all'ordine JSON.
- **Accettazione:** QG-1–QG-4.
- **DoD:** **M7**.
- **Blocchi:** formato XLSX regionale reale (esterno).
- **Passo successivo:** P16/P17.

### P15 — Packaging, autosufficienza, release
- **Obiettivo:** artefatti nativi da CI, verifica automatica di autosufficienza, bundle ibrido, release immutabili con `SHA256SUMS`.
- **Motivazione e priorità:** Alta: requisito dell'MVP (ARCH §43).
- **Prerequisiti:** P8 e P1; per lo snapshot incorporato almeno uno schema reale/di prova.
- **Moduli/contratti:** `.cargo/config.toml`, `xtask`, workflow di release; ARCH §39A–C; DET Addendum A.
- **Task:** T-P15-01 … T-P15-07.
- **Test minimi:** DLL importate solo di sistema; assenza di path host nel binario; esecuzione da cartella con spazi e cwd diversa; JSONL identico tra runner.
- **Accettazione:** QG-1–QG-5.
- **DoD:** **M5** (con M4).
- **Blocchi:** token per la pubblicazione tra repository (DEC-09); repository creati (DEC-08).
- **Passo successivo:** P13/P16.

### P16 — Interfaccia desktop (Slint)
- **Obiettivo:** UI desktop congruente al mockup (§39D.4), worker thread, tabella virtualizzata che legge dal JSONL, export, attribuzione Slint, test visivi in CI.
- **Motivazione e priorità:** Media; dopo la stabilizzazione di core e CLI (DET §150 J).
- **Prerequisiti:** P7, P8, P12; spike: T-P16-00.
- **Moduli/contratti:** crate `ui/`; ARCH §39D; DET §98–99, Addendum B.
- **Task:** T-P16-00 … T-P16-08.
- **Test minimi:** criteri di §39D.6; cancellazione; progress con throttling.
- **Accettazione:** QG-1–QG-5, QG-7.
- **DoD:** **M8**.
- **Blocchi:** esito dello spike (piano B `egui`, DEC-10); font OFL da verificare (DEC-07).
- **Passo successivo:** P17.

### P17 — Production hardening
- **Obiettivo:** benchmark su 10k/100k/1M record, profiling, soglie esplicite, robustezza I/O, valutazione del parallelismo, semantica di `max_errors`.
- **Motivazione e priorità:** Media; le soglie vanno definite dopo aver conosciuto le dimensioni reali (DET §114).
- **Prerequisiti:** P8–P13.
- **Moduli/contratti:** `benches/`, DET §113–116.
- **Task:** T-P17-01 … T-P17-06.
- **Test minimi:** record/s, memoria di picco, file con molti errori vs molti validi, costo della projection.
- **Accettazione:** QG-1–QG-6.
- **DoD:** soglie documentate e rispettate **oppure** esplicitamente non definite con motivo.
- **Blocchi:** dimensioni reali dei flussi (esterno, DEC-14).
- **Passo successivo:** P18.

### P18 — Documentazione e manutenzione
- **Obiettivo:** documentazione operativa, runbook di aggiornamento annuale dei tracciati, runbook di release, sincronizzazione finale dei documenti sorgente.
- **Motivazione e priorità:** Media; chiude **M9**.
- **Prerequisiti:** tutte.
- **Moduli/contratti:** `README.md`, `docs/*`.
- **Task:** T-P18-01 … T-P18-04.
- **Test minimi:** i comandi del README eseguiti da zero su Linux e Windows.
- **Accettazione:** QG-8.
- **DoD:** M9.
- **Blocchi:** —
- **Passo successivo:** manutenzione.

---

## 10. Task implementative con identificativi stabili

### 10.1 Blocchi standard (valgono per ogni task salvo diversa indicazione)

- **STD-VERIFY** — comandi di verifica (ARCH §42, §39A):
  ```text
  cargo fmt --check
  cargo clippy --locked -- -D warnings
  cargo test --locked
  cargo check --locked --target x86_64-pc-windows-gnu    # ciclo di sviluppo su Arch; non è un gate
  ```
  La CI su runner Windows reale (QG-2) è il gate di merge.
- **STD-STOP** — l'agente si ferma e produce un *blocco* (§14) se incontra: conflitto tra fonti, decisione mancante, dato normativo non verificabile, dipendenza non disponibile, contratti incompatibili, necessità di violare un vincolo N-*, impossibilità di dimostrare la correttezza.
- **STD-OUT** — output da riportare: file modificati, comportamento introdotto, contratti pubblici aggiunti/modificati, test aggiunti **con esito reale**, impatto architetturale, elementi rinviati (DET §148) + handoff (§15).
- **STD-TEST** — ogni nuovo comportamento ha test per caso valido, invalido e boundary; ogni regola ha test true/false (ARCH §34.2; DET §163).

Ogni scheda riporta: **Obiettivo/comportamento · Rif. e prerequisiti · File · Casi limite e test · Accettazione**.

### 10.2 Fase P0 — Formalizzazione normativa (lavoro umano assistito dall'AI; l'AI non inventa dati)

**T-P0-01 — Inventario degli input normativi.** Obiettivo: elenco completo e stato di ricezione di ciò che DET §154 richiede. Rif.: DET §106, §154. Prereq.: nessuno. File: `docs/normative/INPUT_INVENTORY.md`. Casi limite: documenti con più versioni/date di efficacia; fonti non ufficiali (da segnalare, non da usare). Accettazione: ogni voce di §154 è `ricevuto`/`mancante`/`non applicabile` con motivo; l'elenco dei mancanti è consegnato all'autore.

**T-P0-02 — Estrazione tabellare dei campi per tracciato.** Obiettivo: per ogni tracciato, tabella fedele alla fonte (ID, descrizione, tipo, start, lunghezza, obbligatorietà, note originali). Rif.: DET §107 pt. 1–4. Prereq.: T-P0-01. File: `docs/normative/<flow>/<trace>-fields.*`. Casi limite: posizioni sovrapposte o con buchi nella fonte (da segnalare, non correggere); alias di tipo. Accettazione: revisione umana a campione + controllo meccanico di continuità posizioni; nessun dato aggiunto non presente in fonte.

**T-P0-03 — Matrice di presenza, assenza e padding.** Obiettivo: mappa dei codici OBB/FAC/NBB e di ogni convenzione di assenza/padding/valore zero. Rif.: DET §14, §15, §106, §107 pt. 4, 11; ARCH §19. Prereq.: T-P0-02. File: `docs/normative/<flow>/presence-matrix.md`. Casi limite: zero valido vs placeholder vs assenza; eccezioni al padding. Accettazione: ogni campo ha una `MissingValuePolicy` e una regola di alignment dichiarata o «nessuna, con motivo».

**T-P0-04 — Matrice delle note normative.** Obiettivo: classificare ogni nota come *metadata*, *regola eseguibile candidata* o *ambigua (decisione umana)*. Rif.: ARCH §5.7, §36; DET §107 pt. 6, §144. Prereq.: T-P0-02. File: `docs/normative/<flow>/notes-matrix.md`. Casi limite: note che implicano più regole; note contraddittorie. Accettazione: nessuna nota senza classe; le ambigue sono registrate come DEC-05 con citazione.

**T-P0-05 — Regole cross-field, cross-trace, join, identità.** Obiettivo: elencare regole tra campi/tracciati, chiavi di correlazione (anche composte), cardinalità attesa, policy su record correlato mancante, campi di identità (ID record, CF, episodio). Rif.: ARCH §21–22; DET §66–72, §91, §107 pt. 5–10. Prereq.: T-P0-04. File: `docs/normative/<flow>/rules-and-joins.md`. Casi limite: join con più risultati (cardinalità obbligatoria, DET §149). Accettazione: ogni join ha cardinalità e `MissingRelatedRecordPolicy` o è marcata `DEC-*`.

**T-P0-06 — Onboarding dei reference dataset.** Obiettivo: scheda per dataset con id, nome ufficiale, versione/validità, fonte, formato sorgente, campo chiave, lunghezza codice, encoding, normalizzazione ammessa, validità temporale. Rif.: DET §108–109. Prereq.: T-P0-01. File: `docs/normative/reference-datasets.md`. Casi limite: dataset con validità temporale o attributi aggiuntivi (non ridurre a semplice insieme di stringhe). Accettazione: scheda completa o lacune elencate.

**T-P0-07 — Redazione di `SCHEMA_AUTHORING.md`.** Obiettivo: JSON Schema del flow schema canonico, sintassi DSL, presence policies, join, identity extraction, reference data bindings, definizioni lessicali dei tipi (set di caratteri, formati data/decimali), regole di versioning, `schema_format_version`, struttura `MANIFEST.json`. Rif.: DET §106, §155; ARCH §7–12, §39C.2. Prereq.: T-P0-03, T-P0-04, T-P0-05. File: `docs/SCHEMA_AUTHORING.md`. Casi limite: ogni costrutto del documento deve essere compilabile dal modello DET §8–21. Accettazione: revisione umana; ogni tipo/regola ha almeno un esempio completo.

**T-P0-08 — Schema canonico SIOC di bozza e fixture richieste.** Obiettivo: file di schema per ogni tracciato nella struttura di ARCH §12 e l'elenco delle fixture (valid / required / datatype / domain / cross-trace) con gli output attesi da produrre. Rif.: DET §107 pt. 12–14, §111. Prereq.: T-P0-07. File: `schemas/sioc/<version>/*.json`, `docs/normative/fixtures-plan.md`. Casi limite: i valori degli output attesi vanno derivati dalla fonte, mai dal comportamento del software. Accettazione: lint a mano rispetto a SCHEMA_AUTHORING; il lint automatico si riesegue appena T-P3-04 esiste (tracciato in IMPLEMENTATION_STATUS).

**T-P0-09 — Chiusura del gate G0.** Obiettivo: verificare che ogni voce di DET §106 sia chiusa o rinviata per iscritto. Rif.: DET §106. Prereq.: T-P0-01…08. File: sezione G0 di `IMPLEMENTATION_STATUS.md`. Accettazione (checklist G0): ☐ forma JSON canonica definita ☐ set `FieldType` ☐ set `AlignmentRule` ☐ sintassi DSL ☐ mapping OBB/FAC/NBB ☐ regole cross-trace ☐ identità e join ☐ formato dataset ☐ versioning tabelle — ciascuna *chiusa* o *rinviata con motivo e task bloccate elencate*; approvazione dell'autore registrata.

### 10.3 Fase P1 — Fondamenta repository e CI

**T-P1-01 — Scaffold del package.** Obiettivo: package con `lib.rs`, `main.rs`, albero di cartelle di DET §4 (solo `mod.rs` vuoti dove serve), `docs/` con i due documenti sorgente e questo piano, `IMPLEMENTATION_STATUS.md` iniziale. Rif.: DET §4, §94, §145. Prereq.: A-02/G0, DEC-03. File: `Cargo.toml`, `src/lib.rs`, `src/main.rs`, `docs/*`, `.gitignore`. Casi limite: `lib.rs` non esporta nulla di interno (DET §94). Accettazione: `cargo build`; nessun codice di dominio.

**T-P1-02 — Toolchain, lockfile, attributi Git, ciclo di sviluppo.** Obiettivo: `rust-toolchain.toml` (profile minimal, rustfmt, clippy, target Windows), `Cargo.lock` committato, `.gitattributes` con `* text=auto`, `tests/fixtures/** -text`, `tests/golden/** -text`, `.cargo/config.toml` di sviluppo (linker mingw e runner wine, non usato in CI), nota di sviluppo su Arch. Rif.: ARCH §39B; DET Addendum A.4. Prereq.: T-P1-01. File: come elencati. Casi limite: l'attributo `-text` evita che Git converta LF/CRLF nelle fixture sui runner Windows. Accettazione: STD-VERIFY verde in locale.

**T-P1-03 — CI su Linux e Windows.** Obiettivo: workflow con matrice `ubuntu`/`windows`: `fmt --check`, `clippy --locked -- -D warnings`, `test --locked`, `build --release --locked`. Rif.: ARCH §39A; DET (sezione CI). Prereq.: T-P1-02. File: `.github/workflows/ci.yml`. Casi limite: includere un test che costruisce un path con spazi e verifica la lettura; i job di sviluppo non consumano minuti Windows oltre il necessario (ARCH §39C.5). Accettazione: workflow verde su entrambi i runner (evidenza: link/log alla run).

### 10.4 Fase P2 — Dominio

**T-P2-01 — Newtype e `ByteRange`.** Obiettivo: `FlowId`, `TraceId`, `FieldId`, `SchemaVersion`, `DatasetId`, `DatasetVersion`, `JoinId`; `ByteRange` con `new/start/len/end_exclusive/slice`. Rif.: DET §6, §7. Prereq.: T-P1-03. File: `src/domain/{ids,byte_range}.rs`. Casi limite: `len == 0`, overflow di `start + len`, slice fuori record → `None`, mai panic. Test: unit + (opzionale) property test `ByteRange` non panica. Accettazione: STD-VERIFY + CI.

**T-P2-02 — `ValidationCode` e mappatura unica.** Obiettivo: enum (DET §51) e **un solo punto** di mappatura verso la forma serializzata. Rif.: ARCH §26; DET §51; CNF-05. Prereq.: T-P2-01, **DEC-02**. File: `src/domain/errors.rs`. Casi limite: codici in output esterno non cambiano arbitrariamente (ARCH §26). Test: round-trip, unicità, ordine stabile. Accettazione: mappatura documentata nel documento sorgente corretto.

**T-P2-03 — Identità, summary, stato.** Obiettivo: `RecordIdentity`, `ValidationSummary`, `RunStatus`, `RunMetadata`, `ValidationRunOutcome`, `ValidationRunError` (senza `ValidationError` dentro). Rif.: DET §35, §57–61, §101. Prereq.: T-P2-01. File: `src/domain/{identity,summary}.rs`. Casi limite: `schema_hash` presente anche se l'hash canonico arriva in T-P14-03; `truncated` previsto (DET §80). Accettazione: STD-VERIFY.

### 10.5 Fase P3 — Schema

**T-P3-01 — Raw model.** Obiettivo: `RawFlowSchema`, `RawRecordSchema`, `RawFieldSpec`, `RawRule`, `RawIdentitySpec`, `RawJoinSpec`, `RawEncodingSpec`, `RawReferenceDataDependency` (solo strutture serializzabili), nella forma di `SCHEMA_AUTHORING.md`. Rif.: DET §8.1, §131. Prereq.: T-P2-01, G0. File: `src/schema/raw.rs`. Casi limite: campi sconosciuti nel JSON (comportamento esplicito e testato, non silenzioso); posizioni one-based mantenute così come nella fonte. Test: deserializzazione degli esempi di SCHEMA_AUTHORING. Accettazione: STD-VERIFY.

**T-P3-02 — Compiled model e `SchemaKey`.** Obiettivo: `CompiledFlowSchema`, `CompiledRecordSchema`, `CompiledFieldSpec`, `SchemaKey` (senza `trace_id`). Rif.: DET §8.2, §9. Prereq.: T-P2-01, T-P3-01. File: `src/schema/compiled.rs`. Casi limite: costruibile **solo** da `SchemaCompiler` (costruttori non pubblici); `Vec` per l'ordine fisico, `HashMap` solo per `field_index` (N-08). Test: invarianti (ogni `field_index` esiste). Accettazione: STD-VERIFY.

**T-P3-03 — `SchemaRepository`.** Obiettivo: trait `load_raw`, `FileSystemSchemaRepository` (layout ARCH §12) e `InMemorySchemaRepository`. Rif.: DET §10; ARCH §12. Prereq.: T-P3-01. File: `src/schema/repository.rs`. Casi limite: versione assente = errore, **mai** «la più recente»; path costruiti solo con `Path::join`; non valida né risolve riferimenti. Test: schema presente/assente/JSON malformato; percorso con spazi. Accettazione: STD-VERIFY + CI Windows.

**T-P3-04 — Linter strutturale.** Obiettivo: `SchemaIssue`, `SchemaIssueCode`, `SchemaIssueSeverity`, `lint_raw_schema(&RawFlowSchema) -> Vec<SchemaIssue>` con i controlli fisici e di tipo di ARCH §14. Rif.: DET §12, §141–142; ARCH §14. Prereq.: T-P3-01. File: `src/schema/linter.rs`. Casi limite: gap non dichiarato, overlap, lunghezza zero, overflow `usize`, ID duplicato, `record_length` incoerente; più issue riportate insieme, in ordine deterministico. Test: un caso per ogni codice. Accettazione: ogni `SchemaIssueCode` ha un test; al termine rieseguire il lint sugli schemi di P0 (T-P0-08) e registrare l'esito.

**T-P3-05 — `SchemaCompiler` (nucleo).** Obiettivo: pipeline di DET §11 per gli step supportati dal sottoinsieme MVP (tipi `Numeric`/`Alphabetic`/`Alphanumeric`, presenza `Required`/`Optional`, posizione/lunghezza), con conversione one→zero-based **unica**. Rif.: DET §11, §13, §157. Prereq.: T-P3-02, T-P3-04. File: `src/schema/compiler.rs`. Casi limite: `compile()` chiama il lint e fallisce se ci sono issue `Error`; gli step non ancora supportati restano boundary esplicite (non no-op silenziosi). Test: schemi validi/invalidi, issue aggregate. Accettazione: STD-VERIFY.

**T-P3-06 — `schema_format_version` e compatibilità.** Obiettivo: ogni schema dichiara la versione di formato; `EngineCapabilities` dichiara il range supportato; formato non supportato = errore di esecuzione con la versione minima di eseguibile richiesta. Rif.: ARCH §39C.2 pt. 3; DET Addendum A.1. Prereq.: T-P3-05, G0. File: `src/schema/{raw,compiler}.rs`. Test: formato supportato / troppo nuovo / assente. Accettazione: messaggio d'errore verificato.

**T-P3-07 — `SchemaRegistry`.** Obiettivo: `resolve(&SchemaKey) -> Arc<CompiledFlowSchema>` (compila a ogni run; cache solo dietro la stessa API, non ora). Rif.: DET §62, §120. Prereq.: T-P3-03, T-P3-05. File: `src/schema/mod.rs` o `registry.rs`. Casi limite: nessuna cache globale mutabile. Test: con `InMemorySchemaRepository`. Accettazione: STD-VERIFY.

### 10.6 Fase P4 — Input layer

**T-P4-01 — `RecordReader` e `NewlinePolicy`.** Obiettivo: `read_next(&mut Vec<u8>) -> Result<Option<RawRecord<'_>>, InputError>` con i 8 punti del contratto di DET §28. Rif.: DET §28–29. Prereq.: T-P2-01. File: `src/input/{reader,record}.rs`. Casi limite: spazi finali preservati; `\r` isolato resta dato; `\r\n` rimosso interamente; EOF senza newline; linea vuota (DEC-12: comportamento provvisorio dichiarato in `IMPLEMENTATION_STATUS.md`); `line_number` aggiornato solo se un record è stato letto. Test: matrice di §110 (input). Accettazione: nessun `trim()`; buffer riusato.

**T-P4-02 — Preflight BOM.** Obiettivo: `BomPolicy::{Reject, StripKnownBom}` con `Reject` come default. Rif.: DET §30. Prereq.: T-P4-01. File: `src/input/source.rs`. Casi limite: il rifiuto è `Err(Input(...))`, non un errore di conformità; il messaggio spiega la causa e l'opzione disponibile (DEC-12). Test: file con/senza BOM, BOM e CRLF come da Blocco note. Accettazione: STD-VERIFY.

**T-P4-03 — `RecordView` e `FieldSlice`.** Obiettivo: accesso zero-copy ai campi tramite `ByteRange::slice`. Rif.: DET §31–32. Prereq.: T-P2-01, T-P3-02. File: `src/input/record.rs`. Casi limite: range oltre il record → `None`; **nessun** `trimmed()`. Test: slice valide e fuori range. Accettazione: STD-VERIFY.

**T-P4-04 — Encoding e rendering diagnostico.** Obiettivo: `EncodingSpec` e `render_diagnostic_value -> DiagnosticValue` (display + `raw_hex` opzionale). Rif.: DET §33–34. Prereq.: T-P2-01. File: `src/input/encoding.rs`. Casi limite: byte non decodificabili non perdono informazione (escape/lossy + hex); la lista concreta di encoding segue i file reali (DEC-13). Test: UTF-8 valido/invalido, Windows-1252. Accettazione: STD-VERIFY.

**T-P4-05 — Matrice di test dell'input layer.** Obiettivo: test su `BufRead` in memoria e su file reali del runner. Rif.: DET §110 (input), §117. Prereq.: T-P4-01…04. File: `tests/integration/input_*.rs`. Casi limite: errore di I/O iniettato alla riga N → `InputError` con riga; riga molto lunga; file vuoto. Accettazione: gira su Linux e Windows (QG-2).

### 10.7 Fase P5 — Error model, sink, summary

**T-P5-01 — `ValidationError` e diagnostica.** Obiettivo: `ValidationError`, `FieldDiagnostic` (start one-based, end inclusivo), `DiagnosticContextItem`. Rif.: DET §52–53, §164; ARCH §24. Prereq.: T-P2-02, T-P2-03. File: `src/domain/errors.rs`. Casi limite: conversione zero-based/end-exclusive → one-based/inclusiva in un solo punto. Test: conversione, invarianti di §164 (flow/version/trace/line sempre valorizzati; campi field-level completi). Accettazione: STD-VERIFY.

**T-P5-02 — `ErrorFactory` e template dei messaggi.** Obiettivo: `field_error`, `record_length_error`; messaggi da template controllati in un unico modulo. Rif.: DET §50, §103–104. Prereq.: T-P5-01, T-P4-04, T-P3-02. File: `src/engine/error_factory.rs`. Casi limite: nessun validator costruisce metadata ripetitivi a mano; il consumer non deve parsare `message`. Test: snapshot per ogni codice. Accettazione: STD-VERIFY.

**T-P5-03 — `ValidationSink` e implementazioni di base.** Obiettivo: trait (`on_error`, `on_summary`), `VecSink`, `CompositeSink`. Rif.: DET §54. Prereq.: T-P5-01. File: `src/report/{sink,composite}.rs`. Casi limite: errore del sink = `Err(Report)` e l'esecuzione si interrompe (DET §118). Test: sink che fallisce. Accettazione: STD-VERIFY.

**T-P5-04 — `JsonlSink`.** Obiettivo: un errore = una riga JSON; UTF-8 senza BOM; terminatore `\n`; campi in ordine stabile. Rif.: DET §55; ARCH §25, §28, §39C.3. Prereq.: T-P5-03, DEC-02. File: `src/report/jsonl.rs`. Casi limite: nessuna riga di summary nello stesso file; mappe con ordine deterministico (`BTreeMap`). Test: confronto **byte per byte** con attesi versionati; identico su runner Windows. Accettazione: QG-4.

**T-P5-05 — Summary builder e `summary.json`.** Obiettivo: `ValidationSummaryBuilder`, `RunMetadata`, scrittura `errors.summary.json`; audit dell'input (nome, dimensione, SHA-256 calcolato durante la lettura, nessun path assoluto). Rif.: DET §57–58, §101–102; ARCH §29, §35, §39C.3. Prereq.: T-P5-04. File: `src/domain/summary.rs`, `src/report/*`. Casi limite: `truncated`; nuova dipendenza per SHA-256 non citata dai documenti → registrare in DEC-15 (pure-Rust). Test: contatori, VALID/INVALID. Accettazione: STD-VERIFY.

**T-P5-06 — Output atomico.** Obiettivo: scrittura su `.part` e rename a fine analisi; su fallimento eliminare `.part` (default consigliato, DET §56) o conservarlo solo con flag diagnostico. Rif.: DET §56, §117, §119. Prereq.: T-P5-04. File: `src/report/*`. Casi limite: rename su Windows con destinazione esistente; report incompleto mai presentato come definitivo. Test: successo, errore tecnico, cancellazione. Accettazione: QG-2.

**T-P5-07 — Harness golden.** Obiettivo: runner che esegue gli scenari di `tests/golden/**` (`input.txt`, `expected.errors.jsonl`, `expected.summary.json`) e confronta byte per byte; aggiornamento degli attesi solo con una variabile esplicita. Rif.: DET §111; ARCH §34.4. Prereq.: T-P5-04, T-P5-05. File: `tests/golden.rs`, `tests/support/*`. Casi limite: `.gitattributes` di T-P1-02 attivo; nessuna normalizzazione sul JSONL. Accettazione: gira su entrambi i runner.

### 10.8 Fase P6 — Validazione locale

**T-P6-01 — `MissingValuePolicy`.** Obiettivo: enum e `is_missing(&policy, &[u8]) -> bool`, pura. Rif.: DET §14. Prereq.: T-P3-02. File: `src/validators/presence.rs`. Casi limite: nessun `AllZeros` globale; nessuna allocazione nel caso standard. Test: ogni variante, campo vuoto/pieno/boundary. Accettazione: STD-VERIFY.

**T-P6-02 — Presenza di base.** Obiettivo: `validate_presence` per `Required`/`Optional`, con `PresenceValidationResult` e `FieldViolation`. Rif.: DET §40–43. Prereq.: T-P6-01, T-P5-02. File: `src/validators/presence.rs`. Casi limite: `Forbidden`/condizionali sono P10, non anticiparli. Test: required vuoto = errore, optional vuoto = ok. Accettazione: STD-VERIFY.

**T-P6-03 — Alignment.** Obiettivo: `AlignmentRule::{RightPad, LeftPad, Exact, Builtin}` e `validate_alignment`. Rif.: DET §15, §44. Prereq.: T-P6-01. File: `src/validators/alignment.rs`. Casi limite: non convertire in numero, non validare dominio, non dedurre regole dal solo nome del tipo; `Builtin` come boundary. Test: padding corretto/errato, campo tutto fill. Accettazione: STD-VERIFY.

**T-P6-04 — Datatype lessicale.** Obiettivo: `validate_datatype` per `Numeric`, `Alphabetic`, `Alphanumeric` (insiemi di caratteri e rappresentazioni da `SCHEMA_AUTHORING.md`). Rif.: DET §45. Prereq.: T-P6-01, T-P0-07. File: `src/validators/datatype.rs`. Casi limite: byte non ASCII; non verifica il dominio normativo. Test: validi/invalidi/boundary. Accettazione: STD-VERIFY.

**T-P6-05 — `AllowedValues`.** Obiettivo: constraint su byte, confronto fisico senza normalizzazione implicita. Rif.: DET §16, §46, §139. Prereq.: T-P6-04. File: `src/validators/domain.rs`. Casi limite: il padding fa parte del valore; compatibilità con la lunghezza verificata nel compiler. Test: valore ammesso/non ammesso. Accettazione: STD-VERIFY.

**T-P6-06 — Field pipeline.** Obiettivo: sequenza presenza → alignment → datatype → constraints con le regole di STOP di DET §40; `FieldPipelineResult`. Rif.: DET §40–41. Prereq.: T-P6-02…05. File: `src/engine/field_pipeline.rs`. Casi limite: un campo obbligatorio vuoto produce **solo** l'errore di presenza; nessun flag su altri campi. Test: ogni percorso del diagramma. Accettazione: QG-3.

**T-P6-07 — `IdentityExtractor`.** Obiettivo: estrazione di `RecordIdentity` dal `IdentityExtractionPlan`. Rif.: DET §35–37; ARCH §22. Prereq.: T-P3-02, T-P4-03. File: `src/engine/identity_extractor.rs`. Casi limite: solo record con lunghezza valida; nessuna posizione hardcoded; non valida il contenuto. Test: identità presente/assente/parziale. Accettazione: QG-3.

**T-P6-08 — `RecordValidator`.** Obiettivo: lunghezza (`StrictSkipRecord`), slicing, pipeline per campo, identità, `RecordValidationArtifacts`; aggiornamento del summary. Rif.: DET §26, §48–49, §87–88, §134. Prereq.: T-P6-06, T-P6-07, T-P5-05. File: `src/engine/record_validator.rs`. Casi limite: record corto/lungo → solo `RECORD_LENGTH`, nessuna cascata, nessuno slicing fuori range; nessun `unwrap()` su input esterno. Test: integrazione con schema sintetico. Accettazione: STD-VERIFY + QG-3.

### 10.9 Fase P7 — Application service

**T-P7-01 — Request, scope, modalità, preflight.** Obiettivo: `ValidationRequest`, `ValidationScope`, `ValidationMode`, `ValidationOptions`, `InvalidRequestError`, `preflight`. Rif.: DET §23–25, §86. Prereq.: T-P6-08. File: `src/application/{request,service}.rs`, `src/engine/preflight.rs`. Casi limite: `Full` non ancora supportato → errore esplicito (arriva con P13); trace inesistente; percorso vuoto. Test: richieste valide/invalide. Accettazione: STD-VERIFY. Nota: la forma della richiesta è quella di DET §25 (CNF-07).

**T-P7-02 — Cancellazione e progresso.** Obiettivo: `CancellationToken`, `ProgressObserver`, `ProgressEvent`, `ValidationPhase`. Rif.: DET §81–83. Prereq.: T-P2-03. File: `src/application/{progress,cancellation}.rs`. Casi limite: nessun evento obbligatorio per record; throttling demandato all'implementazione dell'observer. Test: token condiviso tra thread. Accettazione: STD-VERIFY.

**T-P7-03 — Loop di traccia.** Obiettivo: `validate_reader` e `TraceValidator::validate_path` secondo lo pseudocodice di DET §87. Rif.: DET §75–76, §87–88, §113. Prereq.: T-P7-01, T-P7-02, T-P4-01. File: `src/engine/trace_validator.rs`. Casi limite: cancellazione controllata tra record; errore di lettura alla riga N = `Err(Input)`; `max_errors = None` per l'MVP (DET §80). Test: `BufRead` in memoria. Accettazione: QG-3, QG-6.

**T-P7-04 — `ValidationService::validate`.** Obiettivo: sequenza di DET §85 per `LocalOnly`: schema, preflight, reference data (vuoti finché P11), summary, `sink.on_summary`, `RunStatus`. Rif.: DET §59–61, §84–85, §159. Prereq.: T-P7-03, T-P3-07. File: `src/application/service.rs`. Casi limite: `CompletedInvalid` è `Ok`; `Cancelled` non produce un report che sembri completo. Test: tutti gli esiti di §159. Accettazione: STD-VERIFY.

**T-P7-05 — API pubblica e costruttori.** Obiettivo: `lib.rs` con gli export minimi di DET §94; `ValidationService` costruibile sia da percorsi (`build_default_service`) sia da dipendenze già costruite (`Arc<SchemaRegistry>`, `Arc<dyn ReferenceDataProvider>`); test doubles di DET §93. Rif.: DET §84, §92–94, Addendum A.1 (CNF-04). Prereq.: T-P7-04. File: `src/lib.rs`, `src/application/mod.rs`, `tests/support/*`. Casi limite: i tipi compiled restano `pub(crate)`. Accettazione: nessun export non necessario.

**T-P7-06 — Test di integrazione end-to-end.** Obiettivo: scenari con schema sintetico: valido, invalido, schema non compilabile, I/O error, sink error, cancellazione, record length. Rif.: DET §110 (application), §151. Prereq.: T-P7-05, T-P5-07. File: `tests/integration/*`, `tests/golden/synthetic/*`. Casi limite: fixture dichiarate sintetiche e non normative (A-04). Accettazione: DET §151 soddisfatto in libreria.

### 10.10 Fase P8 — CLI

**T-P8-01 — Scheletro CLI, exit code, percorsi.** Obiettivo: `clap` → `ValidationRequest`; `cli/exit_code.rs` (0/1/2/130); `cli/paths.rs` con risoluzione livelli 1–3 e `EnvProvider` testabile; nessuna working directory come default. Rif.: DET §95–97, Addendum A.1; ARCH §30, §39C.2. Prereq.: T-P7-05, DEC-03. File: `src/cli/*`. Casi limite: opzione o variabile che punta a una directory inesistente = errore, non fallback; cartella incompleta = errore. Test: ogni livello con `EnvProvider` fittizio. Accettazione: STD-VERIFY.

**T-P8-02 — `HumanReadableSink`.** Obiettivo: summary testuale secondo ARCH §29 (`Result: VALID/INVALID`, conteggi per codice). Rif.: ARCH §29; DET §54. Prereq.: T-P8-01. File: `src/report/human.rs`. Casi limite: nessun errore = messaggio esplicito di conformità. Test: snapshot. Accettazione: STD-VERIFY.

**T-P8-03 — Comando `validate`.** Obiettivo: costruzione dei sink (`--errors`, `--summary`, output atomico), exit code, `--version` con versione e commit, log applicativo separato dal report. Rif.: DET §96–97, §100; ARCH §27, §39C.3–4. Prereq.: T-P8-01, T-P8-02, T-P5-06. File: `src/cli/{command,args}.rs`, `src/main.rs`. Casi limite: privilegiare `--output <file>` (il reindirizzamento `>` di PowerShell 5.1 scrive UTF-16); nessuna regola normativa nella CLI. Test: processo reale. Accettazione: QG-3.

**T-P8-04 — Comando `schema lint`.** Obiettivo: stessa logica del compiler, nessuna seconda implementazione. Rif.: DET §141; ARCH §30. Prereq.: T-P3-04, T-P8-01. File: `src/cli/command.rs`. Casi limite: exit code distinti per schema con errori e per errore tecnico. Test: schema valido/invalido. Accettazione: STD-VERIFY.

**T-P8-05 — Ctrl+C.** Obiettivo: intercettazione con crate multipiattaforma (es. `ctrlc`) che imposta il `CancellationToken`; exit 130 come contratto applicativo su entrambi i sistemi. Rif.: DET §119; ARCH §39C.3. Prereq.: T-P8-03. File: `src/cli/*`. Casi limite: l'output definitivo non sembra un report completo. Test: segnale simulato (dove possibile) + test del token. Accettazione: registrare la dipendenza in DEC-15.

**T-P8-06 — Test della CLI.** Obiettivo: test a livello di processo (`CARGO_BIN_EXE_*`, nessun crate di test aggiuntivo) di exit code, percorsi con spazi, working directory diversa, assenza di opzioni. Rif.: DET (sezione Linux/Windows), Addendum A.4. Prereq.: T-P8-03…05. File: `tests/cli/*`. Accettazione: verde su Linux e Windows → **M3**.

### 10.11 Fase P9 — Vincoli semantici

**T-P9-01 — Range numerico.** Obiettivo: `NumericRange` e validator, eseguito solo su valore lessicalmente valido. Rif.: DET §16, §138. Prereq.: T-P6-06. File: `src/validators/range.rs`. Casi limite: estremi inclusi/esclusi come da schema; nessun range se il datatype è fallito. Test: boundary. Accettazione: STD-VERIFY.

**T-P9-02 — Date.** Obiettivo: `DateFormat` e validazione calendariale. Rif.: DET §136. Prereq.: T-P6-06, G0. File: `src/validators/date.rs`. Casi limite: anni bisestili, mesi/giorni fuori range; nessun formato assunto senza evidenza. Test: tabella di date. Accettazione: STD-VERIFY.

**T-P9-03 — Decimal.** Obiettivo: `DecimalSpec` e validator. Rif.: DET §137. Prereq.: T-P6-06, G0. File: `src/validators/decimal.rs`. Casi limite: separatore, scala, segno secondo schema. Test: boundary. Accettazione: STD-VERIFY.

**T-P9-04 — Regex.** Obiettivo: constraint `Regex` compilato nel compiler su byte. Rif.: DET §11 step 11, §16. Prereq.: T-P6-06. File: `src/validators/regex.rs`, `src/schema/compiler.rs`. Casi limite: regex invalida = `SchemaIssue::InvalidRegex`. Test: validi/invalidi. Accettazione: pure-Rust confermato (QG-5).

**T-P9-05 — Compatibilità vincolo/tipo nel compiler.** Obiettivo: step 12 di DET §11 e controlli di ARCH §14 (domini compatibili con la lunghezza, range coerenti). Rif.: DET §11, §16. Prereq.: T-P9-01…04. File: `src/schema/{compiler,linter}.rs`. Test: `ConstraintTypeMismatch`. Accettazione: STD-VERIFY.

**T-P9-06 — Codici, messaggi e golden.** Obiettivo: errori per range/date/dominio nel `ErrorFactory` e golden dedicati. Rif.: ARCH §26. Prereq.: T-P9-05, T-P5-07. File: `src/engine/error_factory.rs`, `tests/golden/*`. Accettazione: QG-4.

### 10.12 Fase P10 — Rule DSL

**T-P10-01 — Modello `Expr` e riferimenti locali.** Obiettivo: raw e compiled (`Eq`, `Ne`, `In`, `NotIn`, `IsMissing`, `IsPresent`, `And`, `Or`, `Not`), `ResolvedFieldRef::Local`. Rif.: DET §17–18; ARCH §11. Prereq.: T-P3-05, G0. File: `src/schema/expression.rs`. Casi limite: `SelfReferentialRule`, `MissingFieldReference` come issue; nessun `eval`. Test: risoluzione corretta e riferimenti inesistenti. Accettazione: STD-VERIFY.

**T-P10-02 — `TruthValue`.** Obiettivo: enum, `UnknownReason`, operatori logici secondo le tavole di DET §19. Rif.: DET §19. Prereq.: T-P10-01. File: `src/engine/expression_evaluator.rs`. Test: **tutte** le combinazioni delle tavole. Accettazione: QG-3.

**T-P10-03 — `ExpressionEvaluator`.** Obiettivo: `evaluate(&CompiledExpr, &EvaluationContext) -> TruthValue`, deterministico, senza side effect, non panicking. Rif.: DET §20, §38–39. Prereq.: T-P10-02. File: `src/engine/expression_evaluator.rs`. Casi limite: record troppo corto → `Unknown(FieldUnavailable)`; confronto su byte, senza trim. Test: ogni operatore true/false/unknown. Accettazione: STD-VERIFY.

**T-P10-04 — Presenza condizionale.** Obiettivo: `CompiledPresencePolicy` normalizzata, `resolve_presence → PresenceState`, integrazione nella pipeline (`Forbidden`, `Conflict`, `SkippedDueToUnknownPresence`, contatore `skipped_rules`). Rif.: DET §21–22, §40–41; ARCH §9 (CNF-10). Prereq.: T-P10-03, T-P6-06. File: `src/schema/presence.rs`, `src/engine/field_pipeline.rs`. Casi limite: `Conflict` non nascosto; `RequiredUnless` normalizzato in modo documentato. Test: ogni `PresenceState`. Accettazione: QG-3.

**T-P10-05 — Regole cross-field locali e contesto diagnostico.** Obiettivo: `CompiledRule`, `validate_local_rule`, `RuleValidationResult`; contesto con i valori che hanno attivato la regola; messaggi da template. Rif.: DET §47, §103, §140; ARCH §24 pt. 7. Prereq.: T-P10-04, T-P5-02. File: `src/engine/*`. Casi limite: ordine delle regole = ordine del compiled schema. Test: esempio 2.29/2.2 di ARCH §11/§25 con schema sintetico equivalente. Accettazione: STD-VERIFY.

**T-P10-06 — Suite di test della DSL.** Obiettivo: copertura completa degli operatori e dell'indipendenza dall'ordine dei campi; golden con regole condizionali. Rif.: ARCH §34.2; DET §110 (rules). Prereq.: T-P10-05, T-P5-07. File: `tests/*`. Casi limite: `PresenceConflict` rilevato staticamente quando possibile. Accettazione: QG-3, QG-4.

### 10.13 Fase P11 — Reference data

**T-P11-01 — Contratti e preparazione.** Obiettivo: `DatasetKey`, `LookupSet`, `ReferenceDataProvider`, `PreparedReferenceData`, `prepare_reference_data` (tutti i dataset caricati **prima** del parsing). Rif.: DET §63–64. Prereq.: T-P7-05. File: `src/reference_data/{provider,dataset}.rs`. Casi limite: nessuna apertura di file per singolo campo; lookup O(1) quando possibile. Test: dipendenze presenti/assenti. Accettazione: STD-VERIFY.

**T-P11-02 — Provider in memoria.** Obiettivo: `InMemoryReferenceDataProvider` per i test. Rif.: DET §93. Prereq.: T-P11-01. File: `src/reference_data/*`, `tests/support/*`. Accettazione: STD-VERIFY.

**T-P11-03 — Provider su filesystem.** Obiettivo: lettura dei dataset nel formato di `REFERENCE_DATA_SPEC.md`, con directory configurabile via `PathBuf`. Rif.: DET §63, §108–109, §128. Prereq.: T-P11-01, T-P0-06. File: `src/reference_data/filesystem.rs`. Casi limite: dataset con validità temporale o attributi extra → boundary esplicita (`lookup(value, effective_date)` solo se richiesto, DET §109); encoding dei file sorgente. Test: dataset di prova dichiarato come tale. Accettazione: STD-VERIFY + CI Windows.

**T-P11-04 — Policy di normalizzazione.** Obiettivo: normalizzazione dichiarata (mai trim/uppercase/rimozione zeri impliciti) prima del lookup. Rif.: DET §65. Prereq.: T-P11-03. File: `src/reference_data/*`, `src/schema/*`. Casi limite: codici con zeri iniziali e lunghezza significativa. Test: stesso valore con e senza normalizzazione dichiarata. Accettazione: STD-VERIFY.

**T-P11-05 — Constraint `ReferenceData`.** Obiettivo: il compiler popola `required_reference_data` (issue `MissingReferenceDatasetDeclaration`); `validate_constraint` interroga `PreparedReferenceData`; errore `ReferenceData`. Rif.: DET §16, §46; ARCH §26. Prereq.: T-P11-04, T-P9-05. File: `src/validators/reference_data.rs`, `src/schema/compiler.rs`. Casi limite: il constraint è l'ultimo stage della pipeline. Test: valore presente/assente/dataset non dichiarato. Accettazione: QG-3.

**T-P11-06 — Preflight e audit dei dataset.** Obiettivo: dataset mancante = `Err(ReferenceData)` prima della lettura; `reference_data_versions` nel summary. Rif.: DET §57, §85, §101. Prereq.: T-P11-05, T-P7-04. File: `src/application/service.rs`. Test: dataset assente, versione riportata. Accettazione: STD-VERIFY.

### 10.14 Fase P12 — Report XLSX e CSV

**T-P12-01 — `CsvSink` e colonne di report.** Obiettivo: CSV RFC 4180 (UTF-8 senza BOM, `,`, CRLF) e definizione **unica** dell'insieme di colonne dell'errore, riusata dall'XLSX. Rif.: ARCH §39C.3; DET Addendum A.2. Prereq.: T-P5-04. File: `src/report/{csv,columns}.rs`. Casi limite: virgolette, virgole e ritorni a capo nei valori. Test: byte-esatto. Accettazione: STD-VERIFY.

**T-P12-02 — Validazione del crate XLSX e scheletro.** Obiettivo: verificare `rust_xlsxwriter` (versione pinned in `Cargo.lock`, licenza, MSRV, features: `constant_memory` sì, `zlib` **no**), confermare l'assenza di dipendenze C con `cargo tree`, scrivere un workbook minimo su entrambi i runner. Rif.: ARCH §4, §39C.1; DEC-06 [EXT]. Prereq.: T-P12-01. File: `Cargo.toml`, `src/report/xlsx.rs`. Casi limite: crate ancora pre-1.0: nessun aggiornamento senza rilettura del changelog. Accettazione: esito registrato in DEC-06.

**T-P12-03 — Foglio `Errori` e `Riepilogo`.** Obiettivo: lettura **streaming** del JSONL e scrittura in modalità low-memory; celle di dato come testo; larghezze colonna esplicite (non dipendere dall'autofit); intestazione fissa; filtri su codice, field ID, ID record, CF, riga; continuazione su fogli successivi oltre 1.048.576 righe (intestazione inclusa). Rif.: ARCH §39C.3; DET Addendum A.2. Prereq.: T-P12-02. File: `src/report/xlsx.rs`. Casi limite: ordine identico al JSONL; il writer **non** è un `ValidationSink` dentro il loop (CNF-08). Test: vedi T-P12-05. Accettazione: memoria costante verificata.

**T-P12-04 — Integrazione CLI.** Obiettivo: opzione di output XLSX generata a fine analisi dal JSONL prodotto (se non richiesto, JSONL temporaneo nella directory temporanea del sistema); scrittura atomica. Rif.: ARCH §39C.3; DET §56. Prereq.: T-P12-03, T-P8-03. File: `src/cli/*`. Casi limite: nessun XLSX in caso di `Err` o `Cancelled`. Test: processo reale. Accettazione: QG-2.

**T-P12-05 — Test dell'XLSX.** Obiettivo: rilettura del file con un parser (es. `calamine` come dev-dependency) verificando: `00123`/`05` identici come stringhe; ordine righe = JSONL; limite righe per foglio (limite parametrizzabile nei test); valori del Riepilogo. Rif.: ARCH §39C.3. Prereq.: T-P12-03. File: `tests/xlsx.rs`. Casi limite: l'identità tra OS è sul **contenuto**, non sui byte del file. Accettazione: verde su entrambi i runner → **M4** (con P9–P11).

### 10.15 Fase P13 — Cross-record e cross-trace

**T-P13-01 — Join e `JoinKey`.** Obiettivo: `CompiledJoinSpec`, `JoinKey` collision-safe (componenti con prefisso di lunghezza), `build_join_key`, `JoinCardinality`, `MissingRelatedRecordPolicy` letti dallo schema. Rif.: DET §66–67, §72, §91. Prereq.: T-P10-01, G0. File: `src/context/join_key.rs`, `src/schema/*`. Casi limite: chiave composta; cardinalità e policy obbligatorie nello schema (altrimenti `SchemaIssue`). Test: collisioni tra componenti simili. Accettazione: STD-VERIFY.

**T-P13-02 — Projection.** Obiettivo: `ContextProjectionPlan` (campi di regole, target, join key, identità) e `project_record` che copia **solo** il necessario. Rif.: DET §68–70, §89. Prereq.: T-P13-01. File: `src/schema/projection.rs`, `src/context/projected_record.rs`. Casi limite: mai l'intera riga. Test: piano completo per ogni regola cross-record (invariante di DET §164). Accettazione: QG-3.

**T-P13-03 — `ContextStore`.** Obiettivo: trait e `InMemoryContextStore` (`insert`, `find`, `iter_trace` in ordine deterministico). Rif.: DET §71. Prereq.: T-P13-02. File: `src/context/{store,in_memory}.rs`. Test: zero/uno/molti match. Accettazione: STD-VERIFY.

**T-P13-04 — Riferimenti esterni e `Unknown`.** Obiettivo: `ResolvedFieldRef::External`, `EvaluationContext` con `projected`/`related`, motivi `RelatedRecordMissing`/`AmbiguousJoin`/`TraceInputUnavailable`. Rif.: DET §18–19, §39, §149. Prereq.: T-P13-03, T-P10-03. File: `src/schema/expression.rs`, `src/engine/expression_evaluator.rs`. Casi limite: mai `first()` su join multipla senza cardinalità; record correlato assente mai `false` senza policy. Test: tavole di verità estese. Accettazione: QG-3.

**T-P13-05 — Validazione cross-record.** Obiettivo: `CrossRecordValidationPlan`, `validate_cross_record_rules`, contesto diagnostico cross-trace. Rif.: DET §74, §90. Prereq.: T-P13-04. File: `src/engine/flow_validator.rs`, `src/schema/*`. Casi limite: non rileggere i file. Test: regole con policy `Violation`/`NotApplicable`/`Unknown`. Accettazione: STD-VERIFY.

**T-P13-06 — `FlowValidator`.** Obiettivo: preflight del bundle (trace richiesti, nessun duplicato), ordine deterministico dei trace (ordine dello schema), pass 1 per trace, controllo di cancellazione prima della pass 2, progress. Rif.: DET §75, §77–78, §83, §133. Prereq.: T-P13-05, T-P7-02. File: `src/engine/flow_validator.rs`. Test: ordine stabile, trace mancante in `Full`. Accettazione: QG-4.

**T-P13-07 — Servizio `Full` e CLI.** Obiettivo: `ValidationMode::Full` nel servizio; `validate-flow --trace-input <id>=<file>`. Rif.: DET §85, §96. Prereq.: T-P13-06, T-P8-03. File: `src/application/service.rs`, `src/cli/*`. Casi limite: input indispensabili mancanti = errore di preflight. Test: processo reale. Accettazione: QG-2.

**T-P13-08 — Golden cross-trace e test di memoria.** Obiettivo: golden con almeno una regola cross-trace **reale** (G0) e test che la memoria cresce con i soli campi proiettati. Rif.: DET §152. Prereq.: T-P13-07, T-P0-08. File: `tests/golden/cross-trace/*`, `tests/memory.rs`. Casi limite: misura con allocatore di test, senza nuove dipendenze. Accettazione: DET §152 soddisfatto → **M6**.

### 10.16 Fase P14 — Schema tooling

**T-P14-01 — Importer XLSX.** Obiettivo: `SchemaImporter` → `RawFlowSchema` dal file regionale, conservando le note originali. Rif.: DET §143; ARCH §13. Prereq.: T-P3-01, G0. File: `src/schema/importer.rs`. Casi limite: l'import non garantisce validità; mai dentro il validation loop (DET §149). Test: fixture XLSX regionale di prova. Accettazione: STD-VERIFY.

**T-P14-02 — Normalizzazione e lint avanzato.** Obiettivo: alias di tipo, normalizzazioni, avvisi per note non trasformate, contraddizioni evidenti. Rif.: ARCH §8, §14; DET §142. Prereq.: T-P14-01, T-P3-04. File: `src/schema/{importer,linter}.rs`. Test: warning vs error. Accettazione: STD-VERIFY.

**T-P14-03 — Canonical schema e hash.** Obiettivo: serializzazione canonica e `schema_hash` su rappresentazione canonicalizzata. Rif.: DET §101–102. Prereq.: T-P3-05. File: `src/schema/*`. Casi limite: ordine JSON casuale → stesso hash. Test: round-trip e stabilità. Accettazione: audit con hash reale.

**T-P14-04 — Diff tra versioni normative.** Obiettivo: report delle differenze tra due versioni di schema. Rif.: ARCH §38 Fase 5, §40. Prereq.: T-P14-03. File: `src/schema/*`. Test: campi aggiunti/rimossi/modificati. Accettazione: STD-VERIFY.

**T-P14-05 — Comandi CLI di schema e reference data.** Obiettivo: `schema import`, `schema list`, `reference-data list`. Rif.: ARCH §30. Prereq.: T-P14-01, T-P8-04, **DEC-18** (contratto di elenco di schemi, versioni e tracciati). File: `src/cli/*`. Test: elenco su directory di prova. Accettazione: **M7**.

### 10.17 Fase P15 — Packaging, autosufficienza, release

**T-P15-01 — Profilo release e configurazione.** Obiettivo: `lto = "thin"`, `codegen-units = 1`, `strip = true`; `+crt-static` per `msvc`; `--remap-path-prefix` nelle build di release; tutte le build con `--locked`. Rif.: ARCH §39B.1, §39C.1, §39C.4. Prereq.: T-P8-06. File: `Cargo.toml`, `.cargo/config.toml`, workflow. Casi limite: validare il profilo con i benchmark quando esistono (T-P17-01). Accettazione: build di release riproducibile in CI.

**T-P15-02 — Verifica automatica dell'artefatto.** Obiettivo: job CI o `cargo xtask verify-artifact` che fallisce se: l'exe Windows importa DLL non di sistema (o `libgcc_s_*`, `libwinpthread-*`, `VCRUNTIME*`, `MSVCP*`); l'eseguibile Linux CLI ha dipendenze oltre la libc di base (o non è statico se musl); il binario contiene path dell'host; `cargo tree -i openssl-sys` non è vuoto. Rif.: ARCH §39C.1; DET Addendum A.3. Prereq.: T-P15-01. File: `xtask/*` o job CI. Casi limite: l'implementazione deve girare sull'OS del runner (non solo su Linux). Accettazione: il job fallisce su un artefatto volutamente errato (test del test).

**T-P15-03 — Snapshot incorporato e risoluzione completa.** Obiettivo: feature `embedded-bundle`, `EmbeddedSchemaRepository` e provider di reference data incorporato, livello 4 di `resolve_bundle_source`, `bundle_source` nell'audit. Rif.: ARCH §39C.2; DET Addendum A.1. Prereq.: T-P15-01, T-P8-01, almeno uno schema (reale o di prova dichiarato). File: `src/cli/paths.rs`, `src/schema/*`, `src/reference_data/*`. Casi limite: cartella esterna presente = **solo** quella, mai mista con lo snapshot; schema/versione mancante in cartella esterna = errore, senza ripiego. Test: i quattro livelli, cartella incompleta, directory inesistente. Accettazione: stesso output con `External` e `Embedded` a parità di contenuto.

**T-P15-04 — Bundle dei dati.** Obiettivo: archivio `schemas-<bundle_version>.zip` con `schemas/`, `reference-data/`, `MANIFEST.json` (`bundle_version`, `schema_format_version`, hash dei file), `SHA256SUMS`. Rif.: ARCH §39C.2. Prereq.: T-P15-03. File: `xtask/*`. Test: manifest coerente con il contenuto. Accettazione: l'eseguibile accetta il bundle senza ricompilazione.

**T-P15-05 — Workflow di release dell'eseguibile.** Obiettivo: build Windows `msvc` sul runner Windows e Linux su baseline glibc o `musl` (DEC-11); archivi e `SHA256SUMS`; pubblicazione nel repository dei binari; release **immutabile** (fallire se la versione esiste). Rif.: ARCH §39A, §39C.4–5. Prereq.: T-P15-02, DEC-08, DEC-09, DEC-11. File: `.github/workflows/release.yml`. Casi limite: nessuna firma Authenticode; job Windows di release breve (minuti al doppio su repository privati, [EXT]). Accettazione: release di prova prodotta solo da CI.

**T-P15-06 — Workflow di release dei dati.** Obiettivo: pubblicazione del bundle nel repository dei dati con le stesse regole di immutabilità. Rif.: ARCH §39C.5. Prereq.: T-P15-04, DEC-08. File: workflow del repo dati. Casi limite: verificare la ridistribuibilità dei tracciati prima di rendere pubblico (DEC-08). Accettazione: release dati di prova.

**T-P15-07 — Test cross-OS e di bundle.** Obiettivo: JSONL identico byte per byte tra i runner (un job confronta gli output prodotti dai due OS); bundle estratto in cartella con spazi e lanciato da working directory diversa; precedenza esterna su incorporata. Rif.: ARCH §39C.3, §43; DET Addendum A.4. Prereq.: T-P15-03, T-P15-05. File: workflow, `tests/*`. Accettazione: **M5** (con M4).

### 10.18 Fase P16 — Interfaccia desktop (Slint)

**T-P16-00 — Collaudo del prototipo.** Obiettivo: verificare i criteri di ARCH §39D.6 sul prototipo esistente: geometria entro tolleranza rispetto al riferimento del mockup; scorrimento con 300.000 e 1.000.000 di righe; output visivo identico tra runner Linux e Windows. Rif.: ARCH §39D.5–6. Prereq.: T-P1-03 (CI), PNG di riferimento esportati dal mockup a 1× e 2×. File: `docs/ui/SPIKE_REPORT.md`. Casi limite: elencare le deviazioni note (scrollbar, interlinea, menu, soglia grezza). Accettazione: esito scritto; se un criterio fallisce, il piano B è `egui` con la stessa definizione di congruenza (DEC-10).

**T-P16-01 — Conversione a workspace.** Obiettivo: root workspace con il package del motore e `ui/` come membri; la CI continua a passare. Rif.: ARCH §31, §39D.2; DET Addendum B.1. Prereq.: T-P16-00 (esito positivo). File: `Cargo.toml`, `ui/Cargo.toml`, workflow. Casi limite: core e CLI **non** dipendono da Slint (`cargo tree -p <core>` senza `slint`). Accettazione: QG-2, QG-5.

**T-P16-02 — Bootstrap della UI.** Obiettivo: `tokens.slint` come unica fonte dei valori, font incorporati (verifica licenza, DEC-07), renderer software, `build.rs`, versione di Slint pinned. Rif.: ARCH §39D.1, §39D.3. Prereq.: T-P16-01. File: `ui/ui/*`, `ui/build.rs`. Casi limite: l'assenza dei font fa fallire la build (nessun ripiego su font di sistema). Accettazione: finestra vuota con font corretti.

**T-P16-03 — Schermata «Nuova analisi».** Obiettivo: selezione di file, flusso, tracciato, versione **esplicita**, encoding, formati di report; controllo preliminare (terminatori, BOM); origine dei dati (incorporati/esterni) visibile. Rif.: ARCH §39; mockup. Prereq.: T-P16-02, T-P14-05 o DEC-18. File: `ui/src/*`, `ui/ui/*`. Casi limite: nessuna «ultima versione» implicita. Test: nessuna logica normativa nella UI (QG-3). Accettazione: confronto con il riferimento visivo.

**T-P16-04 — Esecuzione su worker thread.** Obiettivo: `ValidationService::validate` su thread dedicato; `ProgressObserver` → canale con throttling; annullamento via `CancellationToken`; stati `Cancelled` e `Err` mostrati come tali. Rif.: DET §98–99, Addendum B.2. Prereq.: T-P16-03, T-P7-02. File: `ui/src/*`. Casi limite: il thread UI non esegue la validazione; aggiornamenti dal worker con il meccanismo di Slint per altri thread. Test: cancellazione, errore, completamento. Accettazione: QG-3.

**T-P16-05 — Schermata «Risultati».** Obiettivo: tabella virtualizzata che legge dal JSONL prodotto (indice di offset, nessun `Vec` globale), filtri (codice, field ID, record SIOC, CF, riga), pannello di dettaglio con contesto. Rif.: ARCH §39, §39D.7; DET §99 (CNF-11: ARCH prevale, `ChannelSink` solo per contatori live). Prereq.: T-P16-04, T-P5-04. File: `ui/src/*`, `ui/ui/*`. Casi limite: centinaia di migliaia di righe; riga selezionata fuori dalla finestra visibile. Accettazione: criterio 2 di §39D.6.

**T-P16-06 — Export e «Informazioni».** Obiettivo: pulsanti XLSX/JSONL/CSV che richiamano i writer del core (nessuna reimplementazione); schermata «Informazioni» con attribuzione di Slint e versioni di eseguibile, schema e bundle. Rif.: ARCH §39D.1; DET Addendum B.2. Prereq.: T-P16-05, T-P12-04. File: `ui/src/*`. Accettazione: l'XLSX esportato dalla UI è identico nel contenuto a quello della CLI.

**T-P16-07 — Test visivi in CI.** Obiettivo: rendering headless con renderer software, confronto con riferimenti 1×/2× con tolleranza per zona (quasi nulla su riquadri e bordi, più larga sul testo), su entrambi i runner; aggiornamento dei riferimenti solo esplicito. Rif.: ARCH §39D.5; DET Addendum B.3. Prereq.: T-P16-05. File: `ui/tests/visual.rs`, `ui/tests/reference/*`. Casi limite: le deviazioni note sono elencate e accettate, non nascoste da soglie larghe. Accettazione: criterio 3 di §39D.6.

**T-P16-08 — Packaging della UI.** Obiettivo: archivio dell'eseguibile UI con le stesse regole di release; verifica Linux con **elenco esplicito** di librerie di sistema ammesse (finestra e input); Windows solo DLL di sistema. Rif.: ARCH §39D.2; DET Addendum B.1. Prereq.: T-P16-07, T-P15-05. File: workflow, `xtask/*`. Accettazione: **M8**.

### 10.19 Fase P17 — Production hardening

**T-P17-01 — Harness di benchmark.** Obiettivo: benchmark su 10k, 100k, 1M record con record/s, memoria di picco, input ricchi di errori vs validi, costo della projection. Rif.: DET §114. Prereq.: T-P13-08. File: `benches/*`. Casi limite: dati sintetici dichiarati; se si adotta un crate di benchmark, registrarlo in DEC-15. Accettazione: risultati riproducibili e archiviati.

**T-P17-02 — Profiling e ottimizzazioni.** Obiettivo: ottimizzare solo ciò che i benchmark indicano. Rif.: ARCH §33; DET §113. Prereq.: T-P17-01. Casi limite: i contratti e il determinismo non cambiano; ogni ottimizzazione riporta numeri prima/dopo. Accettazione: golden invariati.

**T-P17-03 — Soglie prestazionali.** Obiettivo: proporre soglie dopo aver conosciuto le dimensioni reali dei flussi (DEC-14) e farle approvare; oppure registrare «non definite» con il motivo. Rif.: DET §114. Prereq.: T-P17-01, DEC-14. File: `docs/PERFORMANCE.md`. Accettazione: nessuna soglia inventata.

**T-P17-04 — Semantica di `max_errors`.** Obiettivo: terminazione controllata, `truncated = true`, eventuale stato dedicato; un file troncato non è mai «completamente validato». Rif.: DET §80. Prereq.: DEC-16. File: `src/application/*`. Casi limite: in assenza di decisione resta `max_errors = None`. Accettazione: test su file con molti errori.

**T-P17-05 — Valutazione del parallelismo.** Obiettivo: spike misurato (es. tracce indipendenti in `LocalOnly`) con prova di determinismo dell'output; adozione solo se i criteri di §7 pt. 8 sono soddisfatti. Rif.: ARCH §33; DET §115. Prereq.: T-P17-02, DEC-17. File: spike su branch dedicato. Accettazione: relazione con numeri; se non conviene, nessuna modifica.

**T-P17-06 — Robustezza e test di proprietà.** Obiettivo: fault injection su I/O e sink, file grandi, test di proprietà (`ByteRange`, parser di newline, collisioni di `JoinKey`, round-trip dello schema canonico). Rif.: DET §112, §117–118. Prereq.: T-P17-01. File: `tests/*`. Casi limite: eventuali crate di test vanno in DEC-15. Accettazione: nessun panic su input esterno.

### 10.20 Fase P18 — Documentazione e manutenzione

**T-P18-01 — Documentazione operativa.** Obiettivo: README con exit code, risoluzione delle directory, avvertenza su PowerShell 5.1 e `>`, BOM/CRLF, uso dell'XLSX, avviso «app non riconosciuta» di Windows, `--version`. Rif.: DET §97; ARCH §39C.3–4. Prereq.: M5. File: `README.md`, `docs/*`. Accettazione: comandi eseguiti da zero su entrambi i sistemi.

**T-P18-02 — Runbook di aggiornamento annuale.** Obiettivo: procedura per un nuovo tracciato: nuovo schema, fixture, lint, golden, release dei dati **senza** ricompilare l'eseguibile (se il formato è supportato). Rif.: ARCH §39C.2; DET §107, §155. Prereq.: M5. File: `docs/RUNBOOK_SCHEMA_UPDATE.md`. Accettazione: dry-run completo con uno schema di prova.

**T-P18-03 — Runbook di release.** Obiettivo: schema di versioni, tag, immutabilità, `SHA256SUMS`, politica di ripubblicazione (nuova versione con nota). Rif.: ARCH §39C.4–5. Prereq.: T-P15-05. File: `docs/RUNBOOK_RELEASE.md`. Accettazione: release di prova seguendo il documento.

**T-P18-04 — Sincronizzazione finale dei documenti sorgente.** Obiettivo: eliminare le incongruenze registrate (CNF-*) e allineare i riferimenti; verificare che nessuna decisione presa in implementazione resti solo nel codice. Rif.: ARCH §44. Prereq.: tutte. File: `docs/*`. Accettazione: nessun CNF aperto; QG-8 → **M9**.

---

## 11. Strategia di testing e quality gates

Il testing è parte di ogni task (§7 pt. 2). I test verificano i **contratti documentati**, non soltanto che il codice compili. **Nessuna soglia prestazionale numerica è definita dalle fonti**: va stabilita esplicitamente in T-P17-03.

### 11.1 Test per componente

| Componente | Test richiesti | Fonte |
|---|---|---|
| domain | newtype, `ByteRange` (nessun panic, `None` fuori range, overflow), conversioni di posizione, mappatura codici | DET §110 |
| schema (linter, compiler) | alias, gap/overlap, ID duplicati, riferimenti, regex invalida, `schema_format_version`, projection plan, issue aggregate e ordinate | ARCH §34.3; DET §110 |
| input | LF, CRLF, spazi finali, riga vuota, EOF senza newline, BOM, byte non UTF-8, errore di I/O iniettato | DET §110, §117 |
| validators | casi validi, invalidi, boundary per ogni validator | DET §110 |
| DSL e `TruthValue` | ogni operatore true/false/unknown; indipendenza dall'ordine; tavole di verità complete | ARCH §34.2; DET §19, §110 |
| pipeline di campo | una causa primaria per stage, nessuna cascata | DET §40 |
| context e cross-record | join zero/uno/molti, chiave composta, projection completa, policy su record mancante | DET §110, §152 |
| report | campi stabili, ordine deterministico, errori di encoding, scrittura atomica, XLSX riletto con parser | DET §110; ARCH §39C.3 |
| application | file valido/invalido, schema non caricabile, dataset mancante, cancellazione, errore del sink | DET §110 |
| streaming e memoria | nessun caricamento completo del file; memoria ∝ buffer riga + context store | ARCH §16; DET §113, §116 |
| CLI | exit code, percorsi con spazi, working directory diversa, assenza di opzioni | DET §97; Addendum A.4 |
| UI | criteri di §39D.6, cancellazione, progress con throttling, test visivi | ARCH §39D; DET Addendum B |
| golden | input + `expected.errors.jsonl` + `expected.summary.json`, confronto byte per byte, nessuna normalizzazione sul JSONL | DET §111, Addendum A.4 |
| regressione | ogni bug di validazione corretto aggiunge un test che lo riproduce | ARCH §34.5 |
| multipiattaforma | stessa suite su Linux e Windows; JSONL identico tra OS | ARCH §39A, §39C.3 |
| packaging | autosufficienza dell'eseguibile; esecuzione da cartella con spazi e cwd diversa | ARCH §39C.1; DET Addendum A.3–A.4 |
| benchmark | 10k/100k/1M record: record/s, memoria di picco, error-heavy vs valid-heavy, costo projection | DET §114 |

### 11.2 Quality gate

| ID | Gate | Verifica |
|---|---|---|
| **QG-1** | Build e controlli locali | `STD-VERIFY` verde, con output reale riportato |
| **QG-2** | CI multipiattaforma | Workflow verde su runner Linux **e** Windows (link/log). È il gate di merge |
| **QG-3** | Conformità architetturale | Checklist DET §163 e anti-pattern ARCH §37: nessun hardcoding normativo nel core (ricerca testuale di `"SIOC"` e di field ID letterali in `src/`, con revisione dei falsi positivi); nessun clone/trim indiscriminato; nessuna dipendenza CLI/UI nell'engine; nuova regola con test true/false; nessun `unwrap()` su input esterno |
| **QG-4** | Determinismo | Golden confrontati byte per byte; ordine degli errori stabile; nessuna iterazione di `HashMap` che influenzi l'output |
| **QG-5** | Autosufficienza e dipendenze | `verify-artifact` verde; `cargo tree` senza `openssl-sys` e senza dipendenze di sistema non autorizzate; nessun path dell'host nel binario |
| **QG-6** | Streaming e memoria | Test su input grande sintetico: nessun caricamento completo; nessun `Vec` globale di errori |
| **QG-7** | Congruenza UI | Test visivi su entrambi i runner entro le tolleranze di §39D.5 |
| **QG-8** | Documentazione sincronizzata | Elenco «documenti da aggiornare» di `IMPLEMENTATION_STATUS.md` vuoto; nessun `CNF-*` aperto per le sezioni toccate |

### 11.3 Gate minimi per concludere una fase

| Fase | Gate | Fase | Gate |
|---|---|---|---|
| P0 | G0 (T-P0-09) | P10 | QG-1…4 |
| P1 | QG-1, 2 | P11 | QG-1…4 |
| P2 | QG-1, 2 | P12 | QG-1…5 |
| P3 | QG-1…3 | P13 | QG-1…4, 6 |
| P4 | QG-1…3, 6 | P14 | QG-1…4 |
| P5 | QG-1…4 | P15 | QG-1…5 |
| P6 | QG-1…4 | P16 | QG-1…5, 7 |
| P7 | QG-1…4, 6 | P17 | QG-1…6 |
| P8 | QG-1…3 | P18 | QG-8 |
| P9 | QG-1…4 | | |

---

## 12. CI, distribuzione e release

**CI (GitHub Actions).** Ogni pull request e il branch principale: matrice `ubuntu` / `windows` con `cargo fmt --check`, `cargo clippy --locked -- -D warnings`, `cargo test --locked`, `cargo build --release --locked`. Un cambiamento che passa solo su Linux non è completo (N-19). Il toolchain è fissato da `rust-toolchain.toml`; `Cargo.lock` è committato; il ciclo di sviluppo su Arch può usare `cargo check --target x86_64-pc-windows-gnu` e Wine, che **non** sostituiscono la CI (ARCH §39B).

**Artefatti** (denominazione confermata da DEC-03: Lintra):

```text
Release dell'eseguibile (repository binari)
  <nome>-<versione>-linux-x86_64.tar.gz
  <nome>-<versione>-windows-x86_64.zip
  SHA256SUMS

Release dei dati (repository dati)
  schemas-<bundle_version>.zip   (schemas/, reference-data/, MANIFEST.json, SHA256SUMS)
```

**Regole di release** (N-23): solo da CI; Windows `msvc` sul runner Windows con CRT statica; Linux costruito in CI su baseline glibc o `musl` (DEC-11), mai sul laptop di sviluppo; nessuna firma Authenticode; release immutabili (un aggiornamento è una nuova versione); `SHA256SUMS` generato automaticamente; token con permesso di scrittura sul repository dei binari (DEC-09).

**Autosufficienza** (N-20): `verify-artifact` (T-P15-02) è un gate di release. L'eseguibile UI ha un elenco esplicito di librerie di sistema ammesse su Linux (ARCH §39D.2).

**Ciclo di vita dei dati.** Un cambio di tracciato produce una nuova release del bundle di dati, non dell'eseguibile, purché `schema_format_version` sia supportato; in caso contrario l'errore indica la versione minima di eseguibile richiesta (N-21).

**Costi.** Su repository privati i minuti Windows consumano la quota al doppio del ritmo di Linux **[EXT]**: tenere il job Windows di release breve e separato dai job di sviluppo (ARCH §39C.5).

---

## 13. Registro dei rischi, dei blocchi e delle decisioni aperte

### 13.1 Decisioni aperte

Responsabile di default: **autore del progetto**.

| ID | Descrizione | Fonte | Stato | Blocca / condiziona | Evidenze necessarie | Azione successiva |
|---|---|---|---|---|---|---|
| DEC-01 | Autorizzare P1/P2 in parallelo a P0 (A-02) | Prompt §4.4 | **RISOLTA: approvata** | P1, P2 | Conferma scritta | Chiedere all'autore |
| DEC-02 | Forma serializzata e numerazione dei codici errore | ARCH §25–26; DET §51 (CNF-05) | **RISOLTA: codici `E_*` simbolici stabili; numerazioni legacy non serializzate** | T-P2-02, T-P5-04, golden | Scelta unica documentata | Decidere prima di P2 |
| DEC-03 | Nome commerciale e nomi di package, binario, variabili `LINTRA_*`, artefatti | CNF-06 | **RISOLTA: Lintra / package `lintra`** | T-P1-01, T-P8-01, release | Verifica marchio/crate/dominio | Decidere prima di P1 |
| DEC-04 | Definizione di MVP (ARCH §43 vs DET §151) | CNF-01 | **RISOLTA: M3 intermedio, M5 MVP** | Criterio di avanzamento M3/M5 | Conferma | Confermare la lettura di §8 |
| DEC-05 | Note normative ambigue | ARCH §5.7 | Si popola in P0 | P3+, P10, P13 | Fonte ufficiale o scelta umana documentata | Registrare in T-P0-04 |
| DEC-06 | Adozione di `rust_xlsxwriter`: versione, licenza, MSRV, feature | ARCH §4; [EXT] | Aperta | P12 | Esito di T-P12-02 | Eseguire T-P12-02 |
| DEC-07 | Licenza dei font IBM Plex per l'incorporamento; attribuzione Slint | ARCH §39D.1, §39D.3 | Aperta | P16 | Testo delle licenze | Verificare in T-P16-02 |
| DEC-08 | Creazione e visibilità dei repository (sorgente, binari, dati); ridistribuibilità dei tracciati | ARCH §39C.5 | Aperta | P15 | Verifica sui documenti regionali | Decidere prima di P15 |
| DEC-09 | Token per la pubblicazione tra repository | ARCH §39C.5 | Aperta | T-P15-05/06 | Credenziale configurata | Configurare in P15 |
| DEC-10 | Esito dello spike UI (Slint vs piano B `egui`) | ARCH §39D.6 | Aperta | P16 | `SPIKE_REPORT.md` | Eseguire T-P16-00 |
| DEC-11 | Versione di toolchain, minimo Windows supportato, strategia Linux (glibc baseline o `musl`) | ARCH §39A, §39B.1, §39C.4 [EXT] | Aperta | T-P1-02, T-P15-05 | Pagina di platform-support del toolchain scelto | Decidere in T-P1-02 |
| DEC-12 | Riga vuota e riga finale vuota; esistenza di `--bom strip`; default `BomPolicy` | DET §30, §110 | Aperta (default provvisorio: riga vuota = record di lunghezza 0; `Reject`) | T-P4-01, T-P4-02, T-P8-01 | Esempi reali di file | Confermare il default |
| DEC-13 | Lista concreta di `EncodingSpec` | DET §33 | Si popola in P0 | T-P4-04 | File reali | Registrare in G0 |
| DEC-14 | Dimensioni reali dei flussi e soglie prestazionali | DET §114 | Aperta | T-P17-03 | Dimensioni reali | Raccogliere dati |
| DEC-15 | Nuove dipendenze non nominate dai documenti (SHA-256, `regex`, `ctrlc`, eventuale crate di benchmark o test) | N-26; ARCH §39C.1 | Aperta, voce per voce | T-P5-05, T-P8-05, T-P9-04, T-P17-01 | Verifica pure-Rust e licenza | Una riga di registro per crate |
| DEC-16 | Semantica di `max_errors` e stato `CompletedWithLimitReached` | DET §80 | Rinviata | T-P17-04 | — | Decidere in P17 |
| DEC-17 | Parallelismo a runtime | ARCH §33; DET §115 | Rinviata | T-P17-05 | Misure | Decidere in P17 |
| DEC-18 | Contratto di elenco di schemi, versioni e tracciati (le trait di DET §10 hanno solo `load_raw`) | ARCH §30; DET §10 | Aperta | T-P14-05, T-P16-03 | Proposta di contratto | Decidere prima di P14 |

### 13.2 Conflitti documentali risolti (revisione approvata 2026-10-16)

La presente revisione ha adottato la versione ARCH del 2026-10-09 (con §39A–D) e DET con Addendum A/B, aggiornandole con ADR-PERSIST-001. Il testo dettagliato delle verifiche è in `CONFLICT_RESOLUTION_MATRIX.md`; questa tabella è il registro sintetico delle risoluzioni **approvate documentalmente** e non equivale a codice Rust già verificato.

| ID | Risoluzione adottata | Riferimento consolidato |
|---|---|---|
| CNF-01 | DET §151 = M3 intermedio; ARCH §43 = MVP M5 completo | ARCH §43, DET §151 |
| CNF-02 | P10 DSL e P11 dataset indipendenti; ordine libero entro prerequisiti | Piano §9 |
| CNF-03 | Slint vincolante, `egui` solo piano B dopo spike; UI prototipo ≠ MVP | ARCH §39D, DET §98, Addendum B |
| CNF-04 | Inizialmente package `lintra`; workspace UI in P16, `ValidationService` con dependency injection e snapshot bundle | DET §4, §92, Addendum A/B |
| CNF-05 | Codici diagnostici stringa `E_*` stabili (es. `E_DUPLICATE_RECORD_KEY`); vecchi esempi numerici non sono API | ARCH §26, DET §51, registro codici P2 |
| CNF-06 | Prodotto Lintra, package Cargo `lintra`; artefatti `lintra-*` | ARCH §1, §39C, DET §4 |
| CNF-07 | `ValidationRequest` di DET §25 è normativo; esempio I/O multipiattaforma è illustrativo | DET §25, addendum piattaforme |
| CNF-08 | Export XLSX da report JSONL canonico post-validazione, non `ValidationSink` in loop | ARCH §28/§39C, DET §54, A.2 |
| CNF-09 | CI Linux/Windows con `--locked`; release Linux da runner baseline definita, non laptop Arch | ARCH §39A–C |
| CNF-10 | `RequiredUnless(E)` compilato in `RequiredIf(Not(E))` | ARCH §9, DET §21 |
| CNF-11 | Tabella UI legge JSONL; canale live solo progresso/contatori | ARCH §39D.7, DET §98–99 |
| CNF-PERSIST-01 | Persistenza CORE generica (non CAD-only); inter-run SIOC audit e CAD storico su stessa infrastruttura | ARCH §45, DET §166–174, ADR-PERSIST-001 |

**Decisione ancora da raccogliere senza bloccare P1:** `DEC-SIOC-HIST-01` (se PK SIOC ripetuta in un diverso invio sia errore o ritrasmissione lecita). Questa non è una contraddizione documentale: la baseline SIOC dichiara soltanto `uniqueness_scope: trace`.

### 13.3 Rischi

| ID | Rischio | Impatto | Mitigazione |
|---|---|---|---|
| RSK-01 | Tracciati o dataset ufficiali non disponibili o ambigui | Blocca P3 e le fasi dipendenti | Gate G0; boundary esplicite; nessuna invenzione (N-14) |
| RSK-02 | Lo spike Slint non soddisfa i criteri o l'API cambia | Ritardo UI | T-P16-00 prima di impegnarsi; piano B `egui`; versione pinned |
| RSK-03 | `rust_xlsxwriter` pre-1.0 [EXT]: API instabile; licenza non verificata | P12 | Versione pinned, T-P12-02, DEC-06 |
| RSK-04 | Rendering e font diversi tra OS nella UI | Congruenza | Font incorporati, renderer software, test visivi su entrambi i runner |
| RSK-05 | Minuti Windows al doppio su repository privati [EXT] | Costo CI | Job Windows di release breve e separato |
| RSK-06 | Git converte LF/CRLF nelle fixture sul runner Windows | Test falsati | `.gitattributes` (T-P1-02) |
| RSK-07 | Binario Linux costruito su Arch non parte su distribuzioni più vecchie | Distribuzione | Release solo da CI |
| RSK-08 | Contesto limitato dell'agente: deriva dalle decisioni | Divergenza | §14, §15, task atomiche, handoff |
| RSK-09 | Context store in memoria insufficiente su flussi reali e snapshot persistenti troppo grandi | P13/PERSIST | Projection minima; repository persistente core separato; benchmark e backend da scegliere dopo misure |
| RSK-10 | Dipendenza da una sola persona (rischio gestionale, **non** ricavato dalle fonti) | Continuità | Documenti dettagliati, runbook (P18), test come documentazione |
| RSK-11 | Versione minima di Windows e supporto del toolchain **[EXT]** | Compatibilità | DEC-11 |
| RSK-12 | Eseguibile non firmato: avviso «app non riconosciuta» | Esperienza d'uso | Decisione accettata (ARCH §39C.4); documentata in T-P18-01 |
| RSK-13 | Tracciati non ridistribuibili in repository pubblici | Aspetti legali | DEC-08 |
| RSK-14 | Dipendenze non previste introdotte senza controllo | Autosufficienza | DEC-15, QG-5 |

### 13.4 Classificazione dei punti

| Categoria | Elenco |
|---|---|
| **Decisioni già prese (da rispettare)** | N-01…N-31 |
| **Dettagli delegabili all'agente** | Signature idiomatiche (DET §147), nomi interni, crate puntuali non vincolanti (ARCH §4), organizzazione dei test, throttling del progress, implementazione di `verify-artifact` per OS, normalizzazione di `RequiredUnless` (CNF-10) |
| **Richiedono fonti esterne** | Input di P0, reference dataset, `EncodingSpec`, `DateFormat`, `DecimalSpec`, dimensioni reali dei flussi, formato XLSX regionale |
| **Richiedono decisione umana** | DEC-01…DEC-18 |

### 13.5 Funzionalità esplicitamente rinviate (non anticipare)

REST API; **database server obbligatorio o scelta anticipata di backend persistente** (la persistenza core è IN AMBITO, §20); versioning temporale dei dataset; DSL oltre gli operatori necessari; auto-import completo dell'XLSX; parallelismo a runtime (fino a T-P17-05); valori derivati (`DerivedValueId`, DET §135); cache di compilazione dello schema (DET §120); `CompletedWithLimitReached` (DEC-16); property test (T-P17-06); dashboard e analytics (ARCH §40).

---

## 14. `AI_AGENT_EXECUTION_PROTOCOL`

### 14.1 Regole di ogni sessione

1. **Leggere** questo piano (almeno §4, §14) e `IMPLEMENTATION_STATUS.md`; individuare la fase e la task attive.
2. **Ispezionare** il repository e lo stato Git (`git status`, branch, ultimo commit) prima di intervenire. Se il repository non è accessibile, dichiararlo e non inventare lo stato.
3. **Verificare prerequisiti e dipendenze** della task (task, decisioni `DEC-*`, gate). Una dipendenza non soddisfatta = blocco.
4. **Consultare** solo le sezioni dei documenti sorgente citate dalla task e le sezioni pertinenti dell'architettura; non reinterpretare l'intera architettura.
5. **Dichiarare l'approccio** in poche righe prima di modifiche sostanziali: file toccati, contratti, test previsti.
6. **Implementare solo lo scope autorizzato** dalla task. Niente funzionalità extra, niente dipendenze «per il futuro» (N-26).
7. **Non rifattorizzare** componenti estranei senza una motivazione documentata e una task.
8. **Eseguire i test e i controlli** pertinenti (STD-VERIFY e i gate della fase) e riportare i risultati **effettivi**.
9. **Non dichiarare** test, build o funzionalità completati senza evidenza (output, link a una run CI). Cosa non è stato eseguito si scrive: «non eseguito, motivo».
10. **Aggiornare** lo stato della task in `IMPLEMENTATION_STATUS.md` (`DONE`, `DONE_UNVERIFIED`, `BLOCKED`) e registrare problemi residui e decisioni prese.
11. **Lasciare il repository coerente**: niente modifiche non committate di altri; non sovrascrivere modifiche preesistenti dell'utente; niente file temporanei; nessun segreto nei file.
12. **Produrre l'handoff** (§15.3).

### 14.2 Regola di arresto

Fermare **le attività interessate** e segnalare esplicitamente il problema quando si incontra: conflitto tra documenti; decisione architetturale mancante; dipendenza non disponibile; requisito normativo non verificabile; incompatibilità tra contratti; errore che impedisce di garantire la correttezza; necessità di cambiare una decisione non negoziabile.

L'agente può continuare le attività **indipendenti e sicure** (che non aggirano il problema e non introducono assunzioni nascoste).

Formato del blocco:

```text
BLOCCO
Descrizione:
Evidenze (sezioni di fonte, file/righe, output dei comandi):
Impatto (task bloccate; attività indipendenti che proseguono):
Alternative possibili:
Decisione richiesta (e a chi):
```

### 14.3 Regole di evidenza e onestà

- Un'affermazione di correttezza richiede un test, un comando o un controllo eseguito. Altrimenti è `DONE_UNVERIFIED`.
- I valori normativi attesi nei test derivano dalla fonte, **mai** dal comportamento del software.
- L'informazione [EXT] (API, licenze, versioni) si ricontrolla sulla versione effettivamente adottata e si distingue dalle decisioni dei documenti.
- Se manca un dato normativo: tipo, trait o policy configurabile e TODO documentato — non un'assunzione (DET §165).

### 14.4 Disciplina di scope e di documentazione

- Ogni cambiamento a un contratto pubblico definito nei documenti sorgente va dichiarato; se la signature cambia solo per esigenze idiomatiche, valgono i vincoli di DET §147.
- Una decisione presa durante l'implementazione si registra nel registro delle decisioni di `IMPLEMENTATION_STATUS.md`, con la sezione sorgente collegata.
- Se un documento sorgente risulta superato, si apre un `CNF-*` nel registro; non si modifica in silenzio.

### 14.5 Dipendenze e Git

- Nuovo crate: una riga in DEC-15 (scopo, licenza, pure-Rust sì/no) **prima** di aggiungerlo; `Cargo.lock` aggiornato nello stesso commit.
- Commit piccoli e coesi, riferiti all'ID della task; nessun `--force` sul branch principale; nessun commit di artefatti di build.

### 14.6 Contesto limitato

Se il contesto non basta per la task, **non** ridurre la verifica: dividere la task proponendo sotto-task (ID `T-P<n>-<nn>.a`, `.b`) nel registro di stato e chiedere conferma.

---

## 15. Gestione del progresso e handoff tra sessioni

### 15.1 Piano e consuntivo

- **Questo documento** = strategia stabile.
- **`IMPLEMENTATION_STATUS.md`** = consuntivo: non contiene strategia.

### 15.2 Contenuto di `IMPLEMENTATION_STATUS.md`

1. **Posizione corrente:** fase e task attive; prossima task raccomandata.
2. **Gate e milestone:** stato di G0 e M0–M9 con data ed evidenza.
3. **Tabella delle task:** ID · stato · ultima sessione · evidenza (comando/esito o link alla run CI) · note.
4. **Registro dei test:** comando · data · esito reale · ambiente (Linux/Windows/CI).
5. **Problemi noti.**
6. **Registro delle decisioni prese in implementazione** (con sezione sorgente) e stato di `DEC-*`/`CNF-*`.
7. **Documenti da aggiornare** (alimenta QG-8).
8. **Plan change log** (modifiche approvate a questo piano).
9. **Registro degli handoff** (in ordine cronologico, l'ultimo in cima).

### 15.3 Formato standard di handoff

```text
HANDOFF — <data e ora> — <identificativo agente/sessione>
Fase/Task attiva:
Stato della task: NOT_STARTED | IN_PROGRESS | BLOCKED | DONE | DONE_UNVERIFIED
Fatto (file toccati, comportamento introdotto):
Evidenze (comando → esito reale; link alla run CI):
Test aggiunti / eseguiti / non eseguiti (con motivo):
Contratti pubblici aggiunti o modificati:
Decisioni prese (con sezione sorgente):
Documenti da aggiornare:
Problemi noti / blocchi (formato §14.2):
Elementi rinviati:
Stato Git: branch · ultimo commit · working tree pulito sì/no
Prossima task raccomandata: ID · motivo · prerequisiti verificati
```

### 15.4 Regole di aggiornamento

- Lo stato si aggiorna **a fine task**, non a fine progetto.
- `DONE` richiede almeno un'evidenza per ogni gate della fase (§11.3).
- Una task `BLOCKED` indica il `DEC-*`/`CNF-*`/gate che la blocca.
- La prossima task raccomandata rispetta le dipendenze di §6 e le decisioni aperte.

---

## 16. Definition of Done complessiva

**MVP (M5) — ARCH §43**
- ☐ Schema versionato esterno caricato, compilato e sottoposto a lint
- ☐ File letto in streaming; lunghezza verificata per ogni riga; campi estratti per posizione/lunghezza
- ☐ Supportati: required, conditional required, padding, datatype, enum/domain, date
- ☐ Ogni errore contiene flow, versione schema, trace, riga, ID record e CF se disponibili, field ID e nome, posizione, actual, expected, reason
- ☐ JSONL e summary prodotti; report XLSX leggibile da un analista
- ☐ Test automatici esistenti; nessuna regola SIOC hardcoded nel core
- ☐ Build e test verdi su Linux x86_64 e Windows x86_64 (CI)
- ☐ Artefatti release nativi per entrambi i sistemi, prodotti dalla CI
- ☐ L'exe Windows importa solo DLL di sistema; nessun path dell'host nel binario
- ☐ Senza cartella esterna usa lo snapshot; con cartella esterna usa quella; stesso output tra OS
- ☐ Aggiornare schemi e dati non richiede di ricompilare né ripubblicare l'eseguibile (formato supportato)
- ☐ JSONL identico byte per byte tra Linux e Windows

**MVP locale (M3) — DET §151:** `ValidationService` esposto; schema da file; compiler e linter; streaming; LF/CRLF senza trim; record length; `ByteRange`; presence, Numeric/AN, alignment minimo, `AllowedValues`; JSONL streammato; summary valid/invalid; `CompletedInvalid` è `Ok`; unit/integration/golden verdi; nessun hardcoding SIOC.

**Full flow (M6) — DET §152:** join nello schema; projection plan; pass 1 e pass 2; context store con indice sui join key; policy esplicita su record correlato mancante; contesto cross-trace nel report; golden con almeno una regola cross-trace reale.

**UI (M8) — ARCH §39D.6:** geometria entro tolleranza; scorrimento fluido con centinaia di migliaia di righe; output visivo identico tra runner; attribuzione Slint; eseguibile separato distribuito.

**1.0 (M9):** hardening e soglie documentate (o dichiarate non definite); README e runbook; nessun `CNF-*` aperto; `IMPLEMENTATION_STATUS.md` coerente.

---

## 17. Checklist di avvio di una nuova sessione

- ☐ Letti §4 e §14 di questo piano
- ☐ Letto `IMPLEMENTATION_STATUS.md`: task attiva, ultimo handoff, blocchi
- ☐ `git status`, branch e ultimo commit ispezionati; working tree compreso
- ☐ Prerequisiti, `DEC-*` e gate della task verificati
- ☐ Sezioni sorgente della task lette (§19)
- ☐ Scope e comandi di verifica (STD-VERIFY + gate di fase) chiari
- ☐ Approccio dichiarato prima delle modifiche sostanziali

## 18. Checklist di chiusura di una sessione

- ☐ Solo lo scope della task è stato modificato
- ☐ STD-VERIFY e i gate pertinenti eseguiti; **esiti reali** annotati
- ☐ Test aggiunti per ogni nuovo comportamento (true/false/boundary)
- ☐ Checklist di code review DET §163 applicata
- ☐ Nessun dato normativo inventato; boundary documentate dove manca la fonte
- ☐ `IMPLEMENTATION_STATUS.md` aggiornato (stato, evidenze, decisioni, documenti da aggiornare)
- ☐ Repository coerente, nessuna modifica altrui sovrascritta
- ☐ Handoff scritto (§15.3) con la prossima task raccomandata

---

## 19. Riferimenti puntuali ai documenti sorgente

| Tema | ARCHITECTURE | DETAILED_DESIGN |
|---|---|---|
| Principio centrale, anti-pattern | §3, §37, §41, §44 | §2, §149, §163, §165 |
| Linguaggio e crate candidati | §4 | — |
| Decisioni non negoziabili | §5 | §3 |
| Architettura logica | §6 | §4–5 |
| Modello schema, tipi, presenza, vincoli | §7–10 | §8–18, §21–22 |
| Rule DSL e logica a tre valori | §11 | §17–20 |
| Schema registry, import, linter | §12–14 | §10–12, §62, §141–143 |
| Byte-oriented, streaming, record | §15–16 | §7, §28–34 |
| Pipeline di validazione | §17–20 | §40–49, §87 |
| Cross-record e identità | §21–22 | §35–37, §66–74, §77, §89–91 |
| Reference data | §23 | §63–65, §108–109 |
| Error model, codici, report, summary | §24–29 | §50–61, §103–105 |
| CLI | §30 | §95–97, §119 |
| Struttura del progetto, API interna | §31–32 | §4, §84–86, §92–94 |
| Performance, testing | §33–34 | §110–116 |
| Auditabilità | §35–36 | §101–102 |
| Roadmap | §38 | §150 |
| UI | §39, §39D | §98–99, Addendum B |
| Portabilità, ambiente, autosufficienza, release | §39A–§39C | sezione Linux/Windows, Addendum A |
| Evoluzioni compatibili | §40 | §153 |
| Workflow agenti, Definition of Done | §42–44 | §146–152, §163–164 |
| Parti aperte e input per la normativa | — | §106–109, §154–155 |

**Tracciabilità della DoD di ARCH §43 verso le fasi:** schema esterno, lint → P3; streaming, lunghezza, slicing → P4, P6, P7; required/padding/datatype/enum → P6; conditional required → P10; date → P9; errori completi → P5; JSONL, summary → P5; XLSX → P12; build e test su due OS → P1; release, autosufficienza, bundle, identità byte → P15; assenza di hardcoding SIOC → QG-3 in ogni fase.


---

## 20. Revisione approvata 2026-10-16 — Persistenza core e starter kit armonizzato

**Change request:** `PCR-LINTRA-PERSIST-001`, approvata esplicitamente dall'autore durante la conversazione del 2026-10-16. Sostituisce la proposta `PCR-CAD-001` che trattava la persistenza come CAD-specific. I file architetturali sono stati aggiornati **prima** dell'avvio del repository.

**Ambito:** `PersistentContextRepository`/transaction/snapshot/audit nel **core** per tutti i flussi. SIOC: chiavi persistite, controllo duplicazioni all'interno del tracciato già dichiarato, audit fra invii; non creare norma nuova sui duplicati fra due trasmissioni (`DEC-SIOC-HIST-01`). CAD: stessa infrastruttura con politica di sessioni trimestrali, progressivi vincolati a evento, controlli temporali e file mancanti. La scelta concreta del backend rimane in una task con confronto tecnico documentato; non è una dipendenza del bootstrap. Fonte vincolante: `PERSISTENCE_CORE_SPEC.md`, ARCH §45 e DET §166–174.

**Task:** le 116 task originarie P0–P18 non vengono ridenominate né riutilizzate; si aggiungono **12 task core** `T-PERSIST-01`…`T-PERSIST-12` elencate in `ATOMIC_TASKS_INDEX.md` e una suite di 12 task di integrazione CAD `T-CAD-01`…`T-CAD-12`. Rispettare le dipendenze: P2/P3 progettano i contratti; P13 dà il cross-record in memoria; PERSIST implementa storage; CAD aggiunge profili, non driver del core.

**Criteri di accettazione aggiunti:** roundtrip persistente da file e restart; serializzazione stabile, commit atomico/rollback; ripetizione PK intra-run; cross-run audit SIOC senza falsi errori; isolamento per namespace e versioni; CAD T1/T2/T3 con annullamento, riattivazione e riuso illecito; incompleti/warning/consenso, nessun annullamento file mancante; `CAD-17` mesi di calendario; benchmark su dataset sintetici; zero dump PHI. Nessuna nuova `FROZEN` CAD prima dei test Rust e del gate normativo.

**Change log iniziale:** 2026-10-16, autore autorizza revisione di entrambe le architetture e riconciliazione del Master Plan, porta persistenza a core e chiede rigenerazione completa dello starter kit. Le modifiche documentali sono state integrate e gli ID P0–P18 mantenuti. Stato codice: ancora non iniziato.
