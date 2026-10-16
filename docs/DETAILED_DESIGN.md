# Regulatory Fixed-Width Validation Engine
## Detailed Design / Low-Level Design per agenti AI

> **Stato:** specifica di progettazione di dettaglio, revisione consolidata 2026-10-16.
>
> **Relazione con il documento architetturale:** questo documento dettaglia e concretizza `ARCHITECTURE.md`. In caso di conflitto, l'agente deve fermarsi, segnalare il conflitto e proporre una modifica esplicita all'architettura prima di procedere.
>
> **Scopo:** portare il progetto al livello in cui un agente AI possa implementare moduli, contratti, interscambi dati, error handling, pipeline e test senza dover reinventare decisioni strutturali.
>
> **Revisione 2026-10-16:** riconciliati Addendum A/B con il master plan e stabilita persistenza core come servizio/trait condiviso SIOC/CAD; vedere §166–174 e `PERSISTENCE_CORE_SPEC.md`.
>
> **Revisione 2026-10-08:** aggiornato il contesto di sviluppo (CachyOS / Arch Linux) e il requisito di distribuzione (binario Windows autosufficiente). Sezioni toccate: §4 (layout), §30 (BOM), §92 (dipendenze runtime), §119 (cancellazione), sezione finale su Linux/Windows e nuovo **Addendum A**. Revisione 2026-10-09: nuovo **Addendum B** (interfaccia desktop con Slint). Le decisioni rimaste aperte sono state chiuse il 2026-10-08 (vedi `ARCHITECTURE.md` §39B/§39C): msvc per la release Windows, modalità ibrida per schemi e reference data, export XLSX come formato principale dei report, nessuna firma dell'eseguibile.
>
> **Importante:** la serializzazione normativa definitiva degli schemi dei flussi e la struttura concreta dei reference dataset saranno definite quando saranno disponibili tutti i tracciati regionali e le tabelle di codifica ufficiali. Questo documento stabilisce però già i **contratti runtime** che tali dati dovranno soddisfare.

---

# 1. Obiettivo del detailed design

Questo documento definisce:

- struttura del package Rust;
- confini tra moduli;
- dipendenze ammesse tra moduli;
- tipi dominio;
- tipi runtime;
- distinzione tra schema raw e schema compiled;
- contratti delle funzioni principali;
- contratti delle trait di I/O e integrazione;
- ownership e lifetime dei dati principali;
- pipeline di validazione;
- comportamento in caso di errori;
- interscambio dati tra componenti;
- gestione cross-field e cross-record;
- report streaming;
- progress reporting;
- cancellazione;
- test strategy;
- ordine di implementazione consigliato;
- vincoli operativi per agenti AI.

L'obiettivo non è descrivere genericamente il sistema, ma indicare **come organizzare concretamente il codice**.

---

# 2. Principio di progettazione da preservare

Il progetto mantiene la separazione:

```text
VALIDATION ENGINE                REGULATORY KNOWLEDGE
────────────────────             ─────────────────────
come leggere                     quali campi esistono
come fare slicing                dove si trovano
come validare un tipo            quali tipi hanno
come valutare regole             quali regole applicare
come interrogare dataset         quali dataset usare
come produrre errori             quali condizioni sono normative
```

Regola sintetica:

> **Il motore sa come validare. Lo schema sa cosa è valido.**

Qualunque implementazione che introduca regole SIOC direttamente nel core viola il design.

---

# 3. Raffinamenti introdotti nel detailed design

Rispetto all'architettura di base vengono rese esplicite alcune scelte.

## 3.1 Il package deve esporre una library

Il progetto deve avere un boundary di libreria stabile.

```text
CLI ──────┐
          │
UI ───────┼──> public library API ──> engine
          │
future API┘
```

La futura UI non deve invocare la CLI e non deve duplicare logica applicativa.

## 3.2 Schema raw e schema compiled sono distinti

Il runtime non deve interpretare continuamente:

- alias `D` / `Data`;
- posizioni one-based;
- regex testuali;
- riferimenti a field ID;
- dipendenze normative;
- join cross-trace.

Tali elementi vengono normalizzati **una sola volta** durante la fase di schema compilation.

```text
RawFlowSchema
      │
      ▼
SchemaCompiler
      │
      ├── lint
      ├── normalize
      ├── resolve refs
      ├── compile regex
      ├── derive validation plan
      └── derive context projection plan
      │
      ▼
CompiledFlowSchema
```

L'engine accetta solo `CompiledFlowSchema`.

## 3.3 Gli errori di conformità non sono errori di esecuzione

Un file non conforme è un esito normale del programma.

```rust
Ok(ValidationRunOutcome::CompletedInvalid { ... })
```

Un `Err(...)` significa invece che il processo non ha potuto essere completato.

Esempi di `Err(...)`:

- schema non caricabile;
- schema non compilabile;
- file non leggibile;
- dataset richiesto assente;
- report non scrivibile;
- configurazione impossibile.

## 3.4 Gli errori devono essere streammati

Il core non deve accumulare automaticamente milioni di `ValidationError` in memoria.

Gli errori vengono inviati ad un `ValidationSink`.

Il risultato finale contiene principalmente il summary.

---

# 4. Layout del package Rust

Struttura iniziale raccomandata:

```text
lintra/
├── Cargo.toml
├── README.md
├── docs/
│   ├── ARCHITECTURE.md
│   └── DETAILED_DESIGN.md
│
├── schemas/
│   └── ...
│
├── reference-data/
│   └── ...
│
├── src/
│   ├── lib.rs
│   ├── main.rs
│   │
│   ├── application/
│   │   ├── mod.rs
│   │   ├── service.rs
│   │   ├── request.rs
│   │   ├── outcome.rs
│   │   ├── progress.rs
│   │   └── cancellation.rs
│   │
│   ├── domain/
│   │   ├── mod.rs
│   │   ├── ids.rs
│   │   ├── byte_range.rs
│   │   ├── value.rs
│   │   ├── identity.rs
│   │   ├── errors.rs
│   │   └── summary.rs
│   │
│   ├── schema/
│   │   ├── mod.rs
│   │   ├── raw.rs
│   │   ├── compiled.rs
│   │   ├── repository.rs
│   │   ├── compiler.rs
│   │   ├── linter.rs
│   │   ├── expression.rs
│   │   ├── presence.rs
│   │   ├── constraint.rs
│   │   ├── projection.rs
│   │   └── importer.rs
│   │
│   ├── input/
│   │   ├── mod.rs
│   │   ├── reader.rs
│   │   ├── record.rs
│   │   ├── encoding.rs
│   │   └── source.rs
│   │
│   ├── engine/
│   │   ├── mod.rs
│   │   ├── trace_validator.rs
│   │   ├── flow_validator.rs
│   │   ├── record_validator.rs
│   │   ├── field_pipeline.rs
│   │   ├── expression_evaluator.rs
│   │   ├── identity_extractor.rs
│   │   ├── error_factory.rs
│   │   └── preflight.rs
│   │
│   ├── validators/
│   │   ├── mod.rs
│   │   ├── presence.rs
│   │   ├── alignment.rs
│   │   ├── datatype.rs
│   │   ├── domain.rs
│   │   ├── range.rs
│   │   ├── date.rs
│   │   ├── decimal.rs
│   │   ├── regex.rs
│   │   └── reference_data.rs
│   │
│   ├── persistence/                 # core, generic persistence: contratto §166–174
│   │   ├── mod.rs
│   │   ├── repository.rs
│   │   ├── policy.rs
│   │   ├── session.rs
│   │   ├── snapshot.rs
│   │   └── transaction.rs
│   │
│   ├── context/
│   │   ├── mod.rs
│   │   ├── join_key.rs
│   │   ├── projected_record.rs
│   │   ├── store.rs
│   │   └── in_memory.rs
│   │
│   ├── reference_data/
│   │   ├── mod.rs
│   │   ├── provider.rs
│   │   ├── dataset.rs
│   │   └── filesystem.rs
│   │
│   ├── report/
│   │   ├── mod.rs
│   │   ├── sink.rs
│   │   ├── jsonl.rs
│   │   ├── csv.rs
│   │   ├── xlsx.rs
│   │   ├── human.rs
│   │   ├── composite.rs
│   │   └── value_renderer.rs
│   │
│   └── cli/
│       ├── mod.rs
│       ├── args.rs
│       ├── command.rs
│       ├── exit_code.rs
│       └── paths.rs
│
└── tests/
    ├── integration/
    ├── golden/
    ├── fixtures/
    └── support/
```

Non tutti i file devono essere creati nel primo commit. La struttura rappresenta il target organizzativo.

---

# 5. Regole di dipendenza tra moduli

Direzione ammessa:

```text
                  domain
                    ▲
                    │
        ┌───────────┼─────────────┐
        │           │             │
      schema   reference_data    input
        ▲           ▲             ▲
        │           │             │
        └────── validators ────────┘
                    ▲
                    │
                 context
                    ▲
                    │
                  engine
                    ▲
                    │
               application
                    ▲
             ┌──────┴───────┐
             │              │
            CLI            UI
```

Vincoli:

- `domain` non dipende da moduli applicativi;
- `schema` non dipende da `engine`;
- `validators` non dipende da CLI/UI;
- `report` non deve contenere logica normativa;
- `application` orchestra ma non implementa regole di validazione;
- `cli` è adapter;
- futura `ui` è adapter;
- `engine` non conosce `clap`, finestre, widget o argomenti CLI.

---

# 6. Convenzioni di tipo

Gli identificativi devono essere newtype, non `String` anonime passate ovunque.

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FlowId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TraceId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FieldId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SchemaVersion(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DatasetId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DatasetVersion(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct JoinId(pub String);
```

Motivi:

- impedire scambi accidentali;
- rendere le signature leggibili;
- semplificare validazione costruttori;
- facilitare `serde` e logging.

---

# 7. ByteRange: contratto fondamentale

Il runtime usa posizioni zero-based e intervalli end-exclusive.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ByteRange {
    start: usize,
    len: usize,
}
```

API:

```rust
impl ByteRange {
    pub fn new(start: usize, len: usize) -> Result<Self, ByteRangeError>;

    pub fn start(&self) -> usize;
    pub fn len(&self) -> usize;
    pub fn end_exclusive(&self) -> usize;

    pub fn slice<'a>(&self, bytes: &'a [u8]) -> Option<&'a [u8]>;
}
```

## Precondizioni

- `len > 0` per i campi normativi ordinari;
- `start + len` non deve overfloware `usize`.

## Postcondizioni

`slice()`:

- non panica;
- restituisce `None` se il range eccede il record;
- non effettua copie.

Lo schema compiler converte una posizione normativa one-based:

```text
start_normativo = 1
```

in:

```text
start_runtime = 0
```

La conversione avviene una sola volta.

---

# 8. Raw schema vs compiled schema

## 8.1 Raw schema

`RawFlowSchema` rappresenta il contenuto deserializzato dalla configurazione canonica.

È permissivo abbastanza da rappresentare la fonte, ma non è considerato sicuro per l'esecuzione.

Esempio concettuale:

```rust
pub struct RawFlowSchema {
    pub flow_id: String,
    pub version: String,
    pub encoding: RawEncodingSpec,
    pub traces: Vec<RawRecordSchema>,
    pub reference_data: Vec<RawReferenceDataDependency>,
}
```

`RawRecordSchema`:

```rust
pub struct RawRecordSchema {
    pub id: String,
    pub name: String,
    pub record_length: Option<usize>,
    pub fields: Vec<RawFieldSpec>,
    pub rules: Vec<RawRule>,
    pub identity: Option<RawIdentitySpec>,
    pub joins: Vec<RawJoinSpec>,
}
```

La forma JSON esatta è **provisionale** fino alla formalizzazione dei tracciati regionali.

## 8.2 Compiled schema

Il compiled schema è un artefatto runtime già verificato.

```rust
pub struct CompiledFlowSchema {
    pub key: SchemaKey,
    pub encoding: EncodingSpec,
    pub traces: HashMap<TraceId, CompiledRecordSchema>,
    pub required_reference_data: Vec<DatasetKey>,
    pub cross_record_plan: CrossRecordValidationPlan,
}
```

`CompiledRecordSchema`:

```rust
pub struct CompiledRecordSchema {
    pub id: TraceId,
    pub name: Arc<str>,
    pub record_length: usize,
    pub fields: Vec<CompiledFieldSpec>,
    pub field_index: HashMap<FieldId, usize>,
    pub local_rules: Vec<CompiledRule>,
    pub identity_plan: Option<IdentityExtractionPlan>,
    pub projection_plan: ContextProjectionPlan,
}
```

`CompiledFieldSpec`:

```rust
pub struct CompiledFieldSpec {
    pub id: FieldId,
    pub name: Arc<str>,
    pub range: ByteRange,
    pub field_type: FieldType,
    pub missing_value_policy: MissingValuePolicy,
    pub presence_policy: CompiledPresencePolicy,
    pub alignment: Option<AlignmentRule>,
    pub constraints: Vec<CompiledConstraint>,
    pub regulatory_metadata: Option<RegulatoryMetadata>,
}
```

### Invariante

Un `CompiledFlowSchema` deve essere costruibile esclusivamente attraverso `SchemaCompiler`.

Il codice dell'engine non deve accettare `RawFlowSchema`.

---

# 9. SchemaKey

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SchemaKey {
    pub flow_id: FlowId,
    pub version: SchemaVersion,
}
```

Il `trace_id` non appartiene alla chiave dello schema completo, ma alla selezione del tracciato dentro lo schema.

---

# 10. SchemaRepository contract

Porta astratta:

```rust
pub trait SchemaRepository: Send + Sync {
    fn load_raw(&self, key: &SchemaKey) -> Result<RawFlowSchema, SchemaLoadError>;
}
```

Prima implementazione:

```rust
pub struct FileSystemSchemaRepository {
    root: PathBuf,
}
```

## Responsabilità

- trovare il file di schema;
- leggerlo;
- deserializzarlo;
- restituire `RawFlowSchema`.

## Non responsabilità

Non deve:

- validare la normativa;
- risolvere field references;
- caricare dataset;
- eseguire validazioni sui record.

---

# 11. SchemaCompiler contract

```rust
pub struct SchemaCompiler;

impl SchemaCompiler {
    pub fn compile(
        &self,
        raw: RawFlowSchema,
    ) -> Result<CompiledFlowSchema, SchemaCompilationError>;
}
```

## Pipeline interna obbligatoria

```text
1. validate top-level identity
2. normalize identifiers
3. normalize field types
4. validate source positions
5. convert one-based → zero-based
6. validate gaps / overlap / record length
7. normalize missing-value policies
8. normalize presence policies
9. resolve local FieldRef
10. resolve cross-trace FieldRef
11. compile regex
12. validate constraint/type compatibility
13. validate identity extraction refs
14. validate joins
15. classify local vs cross-record rules
16. derive ContextProjectionPlan
17. derive CrossRecordValidationPlan
18. freeze CompiledFlowSchema
```

## Errore

```rust
pub struct SchemaCompilationError {
    pub issues: Vec<SchemaIssue>,
}
```

Il compiler deve preferire il reporting aggregato di più issue invece di fermarsi sempre alla prima, quando possibile.

---

# 12. SchemaIssue

```rust
pub struct SchemaIssue {
    pub code: SchemaIssueCode,
    pub flow_id: Option<FlowId>,
    pub trace_id: Option<TraceId>,
    pub field_id: Option<FieldId>,
    pub message: String,
    pub source_note: Option<String>,
}
```

Esempi di `SchemaIssueCode`:

```rust
pub enum SchemaIssueCode {
    InvalidIdentifier,
    UnknownFieldType,
    InvalidPosition,
    ZeroLength,
    RangeOverflow,
    Gap,
    Overlap,
    RecordLengthMismatch,
    DuplicateFieldId,
    MissingFieldReference,
    MissingTraceReference,
    InvalidRegex,
    ConstraintTypeMismatch,
    InvalidIdentityReference,
    InvalidJoin,
    SelfReferentialRule,
    PresenceConflict,
    MissingReferenceDatasetDeclaration,
}
```

---

# 13. FieldType runtime

```rust
pub enum FieldType {
    Numeric,
    Alphabetic,
    Alphanumeric,
    Date(DateFormat),
    Decimal(DecimalSpec),
}
```

La lista potrà crescere solo quando emerge un requisito reale.

Non creare tipi ipotetici non presenti nei tracciati.

---

# 14. MissingValuePolicy

L'assenza deve essere descritta esplicitamente.

```rust
pub enum MissingValuePolicy {
    Spaces,
    Exact(Vec<u8>),
    AnyOf(Vec<Vec<u8>>),
    NeverMissing,
}
```

Non introdurre `AllZeros` come default globale.

Se un campo numerico usa zero-padding e un valore tutto zero rappresenta assenza, tale comportamento deve essere dichiarato nello schema specifico.

Funzione:

```rust
pub fn is_missing(
    policy: &MissingValuePolicy,
    raw: &[u8],
) -> bool;
```

Contratto:

- pura;
- nessuna allocazione necessaria nel caso standard;
- non interpreta semantica non dichiarata.

---

# 15. AlignmentRule

Modello generico:

```rust
pub enum AlignmentRule {
    RightPad {
        fill: u8,
        content_policy: ContentBoundaryPolicy,
    },
    LeftPad {
        fill: u8,
        content_policy: ContentBoundaryPolicy,
    },
    Exact,
    Builtin(BuiltinAlignmentId),
}
```

La semantica definitiva delle regole di padding dipenderà dai tracciati reali.

Vincolo:

> Non dedurre regole di alignment esclusivamente dal nome del tipo se la normativa non lo garantisce.

---

# 16. Constraint runtime

```rust
pub enum CompiledConstraint {
    AllowedValues(Vec<Vec<u8>>),
    Range(NumericRange),
    Regex(CompiledRegex),
    ReferenceData(DatasetKey),
    MustEqual(Vec<u8>),
    Builtin(BuiltinConstraintId),
}
```

Le constraint devono essere già compatibili con il tipo del campo al termine dello schema compiler.

---

# 17. Rule DSL runtime

Le espressioni compilate devono usare riferimenti risolti.

```rust
pub enum CompiledExpr {
    Eq(ResolvedFieldRef, Vec<u8>),
    Ne(ResolvedFieldRef, Vec<u8>),
    In(ResolvedFieldRef, Vec<Vec<u8>>),
    NotIn(ResolvedFieldRef, Vec<Vec<u8>>),
    IsMissing(ResolvedFieldRef),
    IsPresent(ResolvedFieldRef),
    And(Vec<CompiledExpr>),
    Or(Vec<CompiledExpr>),
    Not(Box<CompiledExpr>),
}
```

Estendere solo su requisito reale.

---

# 18. Riferimenti ai campi risolti

```rust
pub enum ResolvedFieldRef {
    Local {
        field_index: usize,
        field_id: FieldId,
    },
    External {
        trace_id: TraceId,
        field_id: FieldId,
        join_id: JoinId,
    },
}
```

Per i campi locali il lookup runtime deve poter usare direttamente `field_index`.

Il `field_id` resta disponibile per diagnostica.

---

# 19. Tre-valued logic

Non tutte le espressioni possono sempre essere valutate.

Esempio:

- record troppo corto;
- record correlato assente;
- input di un altro tracciato non fornito.

Non trattare automaticamente un valore non disponibile come `false`.

Usare:

```rust
pub enum TruthValue {
    True,
    False,
    Unknown(UnknownReason),
}
```

Esempi:

```rust
pub enum UnknownReason {
    FieldUnavailable,
    RelatedRecordMissing,
    TraceInputUnavailable,
    AmbiguousJoin,
}
```

Regole logiche:

```text
FALSE AND UNKNOWN = FALSE
TRUE  AND UNKNOWN = UNKNOWN
TRUE  OR  UNKNOWN = TRUE
FALSE OR  UNKNOWN = UNKNOWN
NOT UNKNOWN = UNKNOWN
```

Questa semantica deve essere testata esplicitamente.

---

# 20. ExpressionEvaluator contract

```rust
pub struct ExpressionEvaluator;

impl ExpressionEvaluator {
    pub fn evaluate(
        &self,
        expr: &CompiledExpr,
        ctx: &EvaluationContext<'_>,
    ) -> TruthValue;
}
```

Deve essere:

- deterministico;
- senza side effect;
- non panicking;
- indipendente dall'ordine di dichiarazione dei campi.

---

# 21. Presence policy normalizzata

Il compiled schema deve preferire una forma normalizzata:

```rust
pub struct CompiledPresencePolicy {
    pub base: BasePresence,
    pub required_if: Vec<CompiledExpr>,
    pub forbidden_if: Vec<CompiledExpr>,
}

pub enum BasePresence {
    Required,
    Optional,
}
```

Questo sostituisce internamente una lista arbitraria di `PresenceRule` raw.

---

# 22. Presence resolution

```rust
pub enum PresenceState {
    Required,
    Optional,
    Forbidden,
    Unknown(UnknownReason),
    Conflict,
}
```

Contratto:

```rust
pub fn resolve_presence(
    policy: &CompiledPresencePolicy,
    ctx: &EvaluationContext<'_>,
) -> PresenceState;
```

Semantica base:

1. valutare `required_if`;
2. valutare `forbidden_if`;
3. integrare con `base`;
4. se una condizione rende contemporaneamente il campo required e forbidden → `Conflict`;
5. se la decisione dipende da un'espressione `Unknown` e non è altrimenti determinabile → `Unknown`.

Uno `PresenceState::Conflict` indica un problema della regola/schema sul particolare stato dei dati e non deve essere nascosto.

---

# 23. Input scope

Il sistema deve supportare due scope concettuali.

```rust
pub enum ValidationScope {
    Trace(TraceValidationInput),
    Flow(FlowValidationInput),
}
```

## Trace scope

```rust
pub struct TraceValidationInput {
    pub trace_id: TraceId,
    pub source: InputSource,
}
```

Serve per:

- MVP;
- validazione locale;
- validazioni cross-field interne al record.

## Flow scope

```rust
pub struct FlowValidationInput {
    pub traces: Vec<TraceValidationInput>,
}
```

Serve per:

- regole cross-trace;
- correlazione di record;
- validazione completa del flusso.

---

# 24. ValidationMode

```rust
pub enum ValidationMode {
    LocalOnly,
    Full,
}
```

## LocalOnly

- valida struttura record;
- presenza locale;
- datatype;
- alignment;
- dominio;
- cross-field locale;
- reference data locali;
- non richiede input di altri tracciati.

Le regole esterne vengono dichiarate `skipped` nel summary.

## Full

- esegue tutto quanto sopra;
- esegue preflight delle dipendenze cross-trace;
- fallisce se mancano input indispensabili definiti dallo schema.

---

# 25. ValidationRequest

Contratto pubblico:

```rust
pub struct ValidationRequest {
    pub schema_key: SchemaKey,
    pub scope: ValidationScope,
    pub mode: ValidationMode,
    pub options: ValidationOptions,
}
```

`ValidationOptions` iniziale:

```rust
pub struct ValidationOptions {
    pub record_length_policy: RecordLengthPolicy,
    pub newline_policy: NewlinePolicy,
    pub max_errors: Option<u64>,
}
```

Evitare di riempire `ValidationOptions` con flag prematuri.

---

# 26. RecordLengthPolicy

Default raccomandato:

```rust
pub enum RecordLengthPolicy {
    StrictSkipRecord,
}
```

Semantica `StrictSkipRecord`:

- se lunghezza effettiva != lunghezza prevista;
- emettere `RECORD_LENGTH`;
- non tentare la field validation del record;
- non produrre cascata di errori derivati.

In futuro potrà essere introdotta una modalità diagnostica diversa solo su richiesta esplicita.

---

# 27. InputSource

Prima versione:

```rust
pub enum InputSource {
    Path(PathBuf),
}
```

L'API interna deve però essere costruita in modo da poter testare con `BufRead` senza filesystem.

---

# 28. RecordReader

Problema da evitare: un `Iterator` standard che restituisce riferimenti al proprio buffer introduce lifetime complessi.

Design raccomandato:

```rust
pub struct RecordReader<R> {
    inner: R,
    line_number: u64,
    newline_policy: NewlinePolicy,
}
```

API:

```rust
impl<R: BufRead> RecordReader<R> {
    pub fn new(inner: R, newline_policy: NewlinePolicy) -> Self;

    pub fn read_next<'a>(
        &mut self,
        buffer: &'a mut Vec<u8>,
    ) -> Result<Option<RawRecord<'a>>, InputError>;
}
```

`RawRecord`:

```rust
pub struct RawRecord<'a> {
    pub line_number: u64,
    pub bytes: &'a [u8],
}
```

## Contratto di `read_next`

1. `buffer.clear()` viene eseguito prima della lettura;
2. legge fino al terminatore di record;
3. rimuove esclusivamente il terminatore configurato;
4. non effettua `trim()`;
5. gli spazi finali sono dati significativi;
6. aggiorna `line_number` solo quando un record è stato letto;
7. restituisce `Ok(None)` a EOF pulito;
8. non conserva reference al buffer dopo il ritorno.

---

# 29. NewlinePolicy

```rust
pub enum NewlinePolicy {
    AutoLfOrCrlf,
    LfOnly,
    CrlfOnly,
}
```

Regola fondamentale:

> Mai rimuovere spazi finali del record.

Con `AutoLfOrCrlf`:

- rimuovere `\n`;
- se immediatamente preceduto da `\r`, rimuovere anche `\r`.

---

# 30. BOM policy

Un BOM può spostare le posizioni del primo record.

Preflight input raccomandato:

```rust
pub enum BomPolicy {
    Reject,
    StripKnownBom,
}
```

Default iniziale consigliato: `Reject` finché i formati ufficiali non stabiliscono diversamente.

Nota di portabilità: su Windows, Blocco note e altri editor possono aggiungere un BOM UTF-8 e convertire i terminatori in CRLF. Il rifiuto per BOM è un errore di esecuzione di preflight (`Err(...)`, non un errore di conformità) e il messaggio deve dire esplicitamente che il file inizia con un BOM e come procedere (es. opzione `--bom strip`, se prevista).

---

# 31. RecordView

```rust
pub struct RecordView<'a> {
    raw: &'a [u8],
}
```

API:

```rust
impl<'a> RecordView<'a> {
    pub fn new(raw: &'a [u8]) -> Self;

    pub fn raw(&self) -> &'a [u8];

    pub fn field(
        &self,
        spec: &CompiledFieldSpec,
    ) -> Option<FieldSlice<'a>>;
}
```

`FieldSlice`:

```rust
#[derive(Clone, Copy)]
pub struct FieldSlice<'a> {
    pub raw: &'a [u8],
}
```

Nessuna copia nel path valido.

---

# 32. FieldSlice helper

Metodi consentiti:

```rust
impl<'a> FieldSlice<'a> {
    pub fn as_bytes(&self) -> &'a [u8];
    pub fn len(&self) -> usize;
    pub fn is_ascii(&self) -> bool;
}
```

Non aggiungere automaticamente `trimmed()` se ciò rischia di confondere valore fisico e valore semantico.

Ogni validator deve sapere se e come il padding entra nella propria semantica.

---

# 33. Encoding

La validazione deve restare byte-oriented finché possibile.

```rust
pub enum EncodingSpec {
    Utf8,
    Windows1252,
    Iso8859_1,
    Other(Arc<str>),
}
```

La lista concreta sarà definita dai file reali.

L'encoding serve soprattutto per:

- rendering diagnostico;
- eventuali validator alfabetici non puramente ASCII;
- conversione controllata quando necessaria.

---

# 34. Diagnostic value rendering

L'allocazione di `String` deve avvenire principalmente quando viene prodotto un errore.

Contratto:

```rust
pub fn render_diagnostic_value(
    bytes: &[u8],
    encoding: &EncodingSpec,
) -> DiagnosticValue;
```

```rust
pub struct DiagnosticValue {
    pub display: String,
    pub raw_hex: Option<String>,
}
```

Se il decoding fallisce:

- non perdere l'informazione;
- usare rappresentazione escaped/lossy;
- opzionalmente includere `raw_hex`.

---

# 35. Record identity

```rust
#[derive(Debug, Clone, Default)]
pub struct RecordIdentity {
    pub sioc_record_id: Option<Arc<str>>,
    pub fiscal_code: Option<Arc<str>>,
    pub episode_id: Option<Arc<str>>,
    pub technical_ids: BTreeMap<Arc<str>, Arc<str>>,
}
```

La struttura resta genericamente riutilizzabile.

Il nome `sioc_record_id` resta ammesso perché è un requisito diagnostico esplicito del primo flusso; l'engine non deve però sapere da quale campo estrarlo.

---

# 36. IdentityExtractionPlan

Lo schema compilato contiene il piano di estrazione.

```rust
pub struct IdentityExtractionPlan {
    pub bindings: Vec<IdentityBinding>,
}

pub struct IdentityBinding {
    pub target: IdentityTarget,
    pub field_index: usize,
}
```

```rust
pub enum IdentityTarget {
    SiocRecordId,
    FiscalCode,
    EpisodeId,
    Technical(Arc<str>),
}
```

---

# 37. IdentityExtractor contract

```rust
pub struct IdentityExtractor;

impl IdentityExtractor {
    pub fn extract(
        &self,
        record: &RecordView<'_>,
        schema: &CompiledRecordSchema,
    ) -> RecordIdentity;
}
```

## Precondizione

Il record deve aver superato il controllo di lunghezza.

## Comportamento

- legge solo i field index configurati;
- converte in forma diagnostica coerente;
- non valida la correttezza normativa dei campi;
- non hardcoda posizioni.

---

# 38. Validation context locale

```rust
pub struct LocalEvaluationContext<'a> {
    pub record: &'a RecordView<'a>,
    pub schema: &'a CompiledRecordSchema,
}
```

Metodo:

```rust
impl<'a> LocalEvaluationContext<'a> {
    pub fn resolve_local(
        &self,
        field_index: usize,
    ) -> Option<&'a [u8]>;
}
```

---

# 39. EvaluationContext generale

Per il rule evaluator:

```rust
pub struct EvaluationContext<'a> {
    pub local: Option<LocalEvaluationContext<'a>>,
    pub projected: Option<&'a ProjectedRecord>,
    pub related: Option<&'a RelatedRecordSet<'a>>,
}
```

Le implementazioni possono essere rifinite, ma il rule evaluator deve dipendere da un'astrazione che permetta accesso sia locale sia cross-record.

---

# 40. Field validation pipeline

Per ogni campo il controllo deve rispettare questo ordine:

```text
presence
   │
   ├── missing + required → errore e STOP field pipeline
   │
   ├── missing + optional → OK e STOP field pipeline
   │
   ├── present + forbidden → errore e STOP field pipeline
   │
   ▼
alignment
   ▼
datatype
   │
   ├── invalid → errore e STOP semantic constraints
   │
   ▼
semantic constraints
   ├── allowed values
   ├── range
   ├── regex
   └── reference data
```

Questa sequenza evita errori ridondanti.

Esempio da evitare:

```text
campo vuoto obbligatorio
→ REQUIRED_FIELD
→ INVALID_TYPE
→ INVALID_DOMAIN
→ INVALID_DATE
```

Deve produrre il solo errore semanticamente primario.

---

# 41. FieldPipelineResult

```rust
pub enum FieldPipelineResult {
    Valid,
    MissingOptional,
    Invalid,
    SkippedDueToUnknownPresence,
}
```

L'orchestrator non deve usare questo risultato per attivare flag su altri campi.

---

# 42. Contratto presence validator

```rust
pub fn validate_presence(
    field: &CompiledFieldSpec,
    value: FieldSlice<'_>,
    ctx: &EvaluationContext<'_>,
) -> Result<PresenceValidationResult, FieldViolation>;
```

`PresenceValidationResult`:

```rust
pub enum PresenceValidationResult {
    Present,
    MissingOptional,
    Unknown,
}
```

`FieldViolation` è un errore interno leggero, non ancora arricchito di metadata di record.

---

# 43. FieldViolation

```rust
pub struct FieldViolation {
    pub code: ValidationCode,
    pub expected: Option<String>,
    pub reason: String,
    pub diagnostic_context: Vec<DiagnosticContextItem>,
}
```

Il `ValidationError` completo viene costruito da `ErrorFactory`.

---

# 44. Alignment validator contract

```rust
pub fn validate_alignment(
    rule: &AlignmentRule,
    value: FieldSlice<'_>,
) -> Result<(), FieldViolation>;
```

Non deve:

- convertire il campo in numero;
- validare il dominio;
- conoscere SIOC.

---

# 45. Datatype validator contract

```rust
pub fn validate_datatype(
    field_type: &FieldType,
    value: FieldSlice<'_>,
) -> Result<(), FieldViolation>;
```

Responsabilità:

- verificare rappresentazione lessicale;
- verificare formato base del tipo;
- non verificare il dominio normativo.

Esempi:

```text
Numeric: solo rappresentazione numerica consentita
Date: formato coerente con DateFormat
Decimal: struttura coerente con DecimalSpec
```

---

# 46. Constraint validator contract

```rust
pub fn validate_constraint(
    constraint: &CompiledConstraint,
    value: FieldSlice<'_>,
    refs: &PreparedReferenceData,
) -> Result<(), FieldViolation>;
```

Per constraint che non richiedono reference data, `refs` non viene interrogato.

---

# 47. Local cross-field rule contract

```rust
pub fn validate_local_rule(
    rule: &CompiledRule,
    ctx: &EvaluationContext<'_>,
) -> RuleValidationResult;
```

```rust
pub enum RuleValidationResult {
    Satisfied,
    Violated(FieldViolation),
    Unknown(UnknownReason),
}
```

L'ordine delle regole deve essere quello stabilito dal compiled schema per garantire output deterministico.

---

# 48. RecordValidator

```rust
pub struct RecordValidator<'a> {
    pub schema: &'a CompiledRecordSchema,
    pub encoding: &'a EncodingSpec,
    pub reference_data: &'a PreparedReferenceData,
}
```

Contratto principale:

```rust
impl<'a> RecordValidator<'a> {
    pub fn validate(
        &self,
        record: RawRecord<'_>,
        sink: &mut dyn ValidationSink,
        summary: &mut ValidationSummaryBuilder,
    ) -> Result<RecordValidationArtifacts, ValidationRunError>;
}
```

`RecordValidationArtifacts`:

```rust
pub struct RecordValidationArtifacts {
    pub identity: Option<RecordIdentity>,
    pub projected_record: Option<ProjectedRecord>,
    pub valid: bool,
}
```

## Nota

Il record validator può produrre un `ProjectedRecord` solo se richiesto dal `projection_plan`.

---

# 49. Record length handling

Dentro `RecordValidator::validate`:

```text
actual_len != expected_len
       │
       ├── build RECORD_LENGTH error
       ├── emit error
       ├── mark record invalid
       └── return senza field validation
```

Nessuna slice fuori range deve poter causare panic.

---

# 50. ErrorFactory

Centralizzare la costruzione del report diagnostico.

```rust
pub struct ErrorFactory<'a> {
    pub flow_id: &'a FlowId,
    pub schema_version: &'a SchemaVersion,
    pub trace_id: &'a TraceId,
    pub encoding: &'a EncodingSpec,
}
```

API:

```rust
impl<'a> ErrorFactory<'a> {
    pub fn field_error(
        &self,
        line_number: u64,
        identity: &RecordIdentity,
        field: &CompiledFieldSpec,
        raw_value: &[u8],
        violation: FieldViolation,
    ) -> ValidationError;

    pub fn record_length_error(
        &self,
        line_number: u64,
        identity: Option<&RecordIdentity>,
        expected: usize,
        actual: usize,
    ) -> ValidationError;
}
```

Vantaggi:

- formato uniforme;
- nessun validator costruisce manualmente metadata ripetitivi;
- rendering actual value centralizzato;
- test snapshot semplici.

---

# 51. ValidationCode

Usare enum + serializzazione stabile.

```rust
pub enum ValidationCode {
    RecordLength,
    RequiredField,
    ConditionalRequired,
    ForbiddenField,
    Alignment,
    InvalidType,
    InvalidDomain,
    OutOfRange,
    InvalidDate,
    CrossField,
    ReferenceData,
    CrossRecord,
    RuleConflict,
    RelatedRecordMissing,
    DuplicateRecordKey,
    /// Rule code registrato dal compiled schema (es. E_CAD_*)
    RegulatoryRule(Arc<str>),
}
```

La serializzazione pubblica **deve usare stringhe simboliche stabili**, ad esempio:

```text
RecordLength -> E_RECORD_LENGTH
ConditionalRequired -> E_CONDITIONAL_REQUIRED
RelatedRecordMissing -> E_RELATED_RECORD_MISSING
DuplicateRecordKey -> E_DUPLICATE_RECORD_KEY
```

Gli esempi numerici `E001_*` appartengono a una versione storica e non sono API. I codici specifici (`E_CAD_*`) sono registrati nella policy/schema di flusso. La mappatura deve stare in un unico punto, con test di stabilità. Errori di lettura/compilazione dello schema non sono un record di violazione normativa.

---

# 52. ValidationError

Modello target:

```rust
pub struct ValidationError {
    pub code: ValidationCode,

    pub flow_id: FlowId,
    pub schema_version: SchemaVersion,
    pub trace_id: TraceId,

    pub line_number: u64,
    pub record_identity: RecordIdentity,

    pub field: Option<FieldDiagnostic>,

    pub actual: Option<DiagnosticValue>,
    pub expected: Option<String>,

    pub context: Vec<DiagnosticContextItem>,
    pub message: String,
}
```

`FieldDiagnostic`:

```rust
pub struct FieldDiagnostic {
    pub id: FieldId,
    pub name: Arc<str>,
    pub start_one_based: usize,
    pub end_one_based_inclusive: usize,
}
```

Nota importante:

- internamente `ByteRange` è zero-based/end-exclusive;
- nel report destinato agli analisti si torna alle posizioni normative one-based/inclusive.

---

# 53. DiagnosticContextItem

```rust
pub struct DiagnosticContextItem {
    pub trace_id: TraceId,
    pub field_id: FieldId,
    pub value: DiagnosticValue,
}
```

Serve a spiegare condizioni come:

```text
2.29 è obbligatorio perché 2.2 = 05
```

---

# 54. ValidationSink

Contratto fondamentale per output streaming:

```rust
pub trait ValidationSink {
    fn on_error(
        &mut self,
        error: &ValidationError,
    ) -> Result<(), ReportError>;

    fn on_summary(
        &mut self,
        summary: &ValidationSummary,
    ) -> Result<(), ReportError>;
}
```

Implementazioni previste:

```text
JsonlSink
CsvSink
HumanReadableSink
CompositeSink
VecSink        // test
ChannelSink    // SOLO progress/counters live nella UI, NON per la tabella errori (legge JSONL)
```

L'engine dipende solo dalla trait.

---

# 55. JSONL contract

Ogni `ValidationError` = una riga JSON indipendente.

Il summary può essere:

- scritto in file separato `.summary.json`;
- oppure inviato a un sink secondario.

Non mescolare una riga summary arbitraria nello stesso JSONL senza una discriminante di tipo formalizzata.

Prima scelta raccomandata:

```text
errors.jsonl
errors.summary.json
```

---

# 56. Output atomico

Per CLI/file output:

```text
errors.jsonl.part
       │
       │ analisi completata
       ▼
errors.jsonl
```

Se l'esecuzione fallisce per errore tecnico:

- non pubblicare come definitivo un report incompleto;
- il comportamento sul `.part` deve essere documentato;
- default consigliato: eliminarlo oppure mantenerlo solo con flag diagnostico esplicito.

---

# 57. ValidationSummary

```rust
pub struct ValidationSummary {
    pub run_status: RunStatus,
    pub schema_key: SchemaKey,
    pub scope_summary: ScopeSummary,

    pub records_processed: u64,
    pub valid_records: u64,
    pub invalid_records: u64,
    pub total_errors: u64,

    pub errors_by_code: BTreeMap<ValidationCode, u64>,
    pub skipped_rules: u64,

    pub validator_version: String,
    pub schema_hash: String,
    pub reference_data_versions: BTreeMap<DatasetId, DatasetVersion>,

    pub elapsed: Duration,
}
```

---

# 58. Summary builder

Per evitare mutazioni sparse:

```rust
pub struct ValidationSummaryBuilder {
    // private counters
}
```

Metodi:

```rust
impl ValidationSummaryBuilder {
    pub fn record_processed(&mut self);
    pub fn record_valid(&mut self);
    pub fn record_invalid(&mut self);
    pub fn error_emitted(&mut self, code: &ValidationCode);
    pub fn rule_skipped(&mut self);
    pub fn finish(self, metadata: RunMetadata) -> ValidationSummary;
}
```

---

# 59. RunStatus

```rust
pub enum RunStatus {
    CompletedValid,
    CompletedInvalid,
    Cancelled,
}
```

`Cancelled` è un esito controllato, non necessariamente un errore tecnico.

---

# 60. ValidationRunOutcome

```rust
pub struct ValidationRunOutcome {
    pub status: RunStatus,
    pub summary: ValidationSummary,
}
```

---

# 61. ValidationRunError

Un `Err` significa impossibilità di completare correttamente il job.

```rust
pub enum ValidationRunError {
    SchemaLoad(SchemaLoadError),
    SchemaCompilation(SchemaCompilationError),
    TraceNotFound(TraceId),
    Input(InputError),
    ReferenceData(ReferenceDataError),
    Report(ReportError),
    InvalidRequest(InvalidRequestError),
    Context(ContextError),
}
```

Non inserire `ValidationError` dentro `ValidationRunError`.

---

# 62. Schema Registry service

```rust
pub struct SchemaRegistry {
    repository: Arc<dyn SchemaRepository>,
    compiler: SchemaCompiler,
}
```

API:

```rust
impl SchemaRegistry {
    pub fn resolve(
        &self,
        key: &SchemaKey,
    ) -> Result<Arc<CompiledFlowSchema>, ValidationRunError>;
}
```

Prima versione può compilare ad ogni run.

Caching futuro consentito dietro la stessa API.

---

# 63. Reference data contracts

La struttura concreta dei file dei dataset viene rimandata alla disponibilità delle codifiche ufficiali.

Il core dipende però già da queste interfacce.

`DatasetKey`:

```rust
pub struct DatasetKey {
    pub id: DatasetId,
    pub version: DatasetVersion,
}
```

`LookupSet`:

```rust
pub trait LookupSet: Send + Sync {
    fn contains(&self, value: &[u8]) -> bool;
    fn key(&self) -> &DatasetKey;
}
```

`ReferenceDataProvider`:

```rust
pub trait ReferenceDataProvider: Send + Sync {
    fn load(
        &self,
        key: &DatasetKey,
    ) -> Result<Arc<dyn LookupSet>, ReferenceDataError>;
}
```

---

# 64. PreparedReferenceData

Il provider non deve essere interrogato per aprire file a ogni singolo campo.

Preflight:

```rust
pub struct PreparedReferenceData {
    sets: HashMap<DatasetKey, Arc<dyn LookupSet>>,
}
```

Factory:

```rust
pub fn prepare_reference_data(
    dependencies: &[DatasetKey],
    provider: &dyn ReferenceDataProvider,
) -> Result<PreparedReferenceData, ReferenceDataError>;
```

Tutti i dataset obbligatori vengono caricati prima del parsing del file.

---

# 65. Reference data normalization

La policy di normalizzazione deve essere dichiarata.

Non fare implicitamente:

```text
trim
uppercase
remove leading zero
```

prima di un lookup, salvo regola esplicita.

I codici posizionali possono essere sensibili a zero iniziali e lunghezza.

---

# 66. Cross-record: concetto di join

Una relazione tra tracciati deve essere schema-driven.

```rust
pub struct CompiledJoinSpec {
    pub id: JoinId,
    pub source_trace: TraceId,
    pub target_trace: TraceId,
    pub source_fields: Vec<usize>,
    pub target_fields: Vec<usize>,
}
```

Un join può avere chiave composta.

---

# 67. JoinKey

Non concatenare stringhe con un separatore ambiguo.

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct JoinKey(Vec<u8>);
```

Costruzione raccomandata:

```text
[len component 1][component 1][len component 2][component 2]...
```

oppure una struttura equivalente collision-safe.

API:

```rust
pub fn build_join_key(
    record: &RecordView<'_>,
    field_indexes: &[usize],
    schema: &CompiledRecordSchema,
) -> Result<JoinKey, JoinKeyError>;
```

---

# 68. ContextProjectionPlan

Il compiler deve calcolare quali campi devono sopravvivere oltre la singola riga.

```rust
pub struct ContextProjectionPlan {
    pub required: bool,
    pub field_indexes: Vec<usize>,
    pub join_plans: Vec<JoinProjectionPlan>,
}
```

Il piano include:

- campi usati da cross-record rules;
- campi target da confrontare;
- campi necessari per le join key;
- campi di identity necessari per diagnostica.

---

# 69. ProjectedRecord

```rust
pub struct ProjectedRecord {
    pub trace_id: TraceId,
    pub line_number: u64,
    pub identity: RecordIdentity,
    pub join_keys: HashMap<JoinId, JoinKey>,
    pub values: HashMap<FieldId, Vec<u8>>,
}
```

Questo è **owned data**.

Solo qui vengono copiati i byte necessari per le fasi successive.

Non salvare l'intera riga.

---

# 70. Perché la projection è importante

Senza projection:

```text
milioni di record × lunghezza completa
```

Con projection:

```text
milioni di record × soli campi richiesti dalle regole cross-record
```

La quantità di memoria resta funzione delle dipendenze reali.

---

# 71. ContextStore trait

```rust
/// Store effimero per il join di UNA singola esecuzione (non repository persistente).
pub trait ContextStore {
    fn insert(
        &mut self,
        record: ProjectedRecord,
    ) -> Result<(), ContextError>;

    fn find(
        &self,
        trace_id: &TraceId,
        join_id: &JoinId,
        key: &JoinKey,
    ) -> Result<RelatedRecordSet<'_>, ContextError>;

    fn iter_trace<'a>(
        &'a self,
        trace_id: &TraceId,
    ) -> Box<dyn Iterator<Item = &'a ProjectedRecord> + 'a>;
}
```

Prima implementazione:

```rust
InMemoryContextStore
```

In futuro può essere sostituita con storage disk-backed mantenendo il contratto.

---

# 72. Ambiguità delle join

Una join può trovare:

- zero record;
- un record;
- più record.

Il rule model deve dichiarare o assumere una cardinalità.

Placeholder runtime:

```rust
pub enum JoinCardinality {
    ZeroOrOne,
    ExactlyOne,
    OneOrMany,
}
```

La formalizzazione definitiva verrà guidata dai tracciati regionali.

Non scegliere arbitrariamente `first()` quando una join restituisce più record.

---

# 73. Strategia cross-record raccomandata

Evitare, se possibile, la rilettura completa dei file.

```text
PASS 1 PER OGNI INPUT
────────────────────────────
read record
local validation
identity extraction
context projection
store projected record

PASS 2
────────────────────────────
iterate projected records
resolve join keys
apply cross-record rules
emit errors
```

Il compiler garantisce che tutti i field value necessari alla pass 2 siano inclusi nella projection.

---

# 74. CrossRecordValidationPlan

```rust
pub struct CrossRecordValidationPlan {
    pub rules: Vec<CompiledCrossRecordRule>,
    pub required_traces: HashSet<TraceId>,
}
```

Il preflight di `ValidationMode::Full` verifica la presenza dei trace richiesti.

---

# 75. TraceValidator contract

```rust
pub struct TraceValidator<'a> {
    pub flow_schema: &'a CompiledFlowSchema,
    pub trace_schema: &'a CompiledRecordSchema,
    pub reference_data: &'a PreparedReferenceData,
}
```

API interna:

```rust
impl<'a> TraceValidator<'a> {
    pub fn validate_path(
        &self,
        path: &Path,
        sink: &mut dyn ValidationSink,
        context_store: Option<&mut dyn ContextStore>,
        summary: &mut ValidationSummaryBuilder,
        progress: &mut dyn ProgressObserver,
        cancellation: &CancellationToken,
    ) -> Result<(), ValidationRunError>;
}
```

Deve usare una funzione più bassa `validate_reader` per i test.

---

# 76. validate_reader contract

```rust
pub fn validate_reader<R: BufRead>(
    reader: R,
    trace_schema: &CompiledRecordSchema,
    flow_schema: &CompiledFlowSchema,
    reference_data: &PreparedReferenceData,
    sink: &mut dyn ValidationSink,
    context_store: Option<&mut dyn ContextStore>,
    summary: &mut ValidationSummaryBuilder,
    progress: &mut dyn ProgressObserver,
    cancellation: &CancellationToken,
) -> Result<(), ValidationRunError>;
```

È il punto principale per integration test senza filesystem.

---

# 77. FlowValidator contract

```rust
pub struct FlowValidator<'a> {
    pub schema: &'a CompiledFlowSchema,
    pub reference_data: &'a PreparedReferenceData,
}
```

API:

```rust
impl<'a> FlowValidator<'a> {
    pub fn validate(
        &self,
        input: &FlowValidationInput,
        sink: &mut dyn ValidationSink,
        summary: &mut ValidationSummaryBuilder,
        progress: &mut dyn ProgressObserver,
        cancellation: &CancellationToken,
    ) -> Result<(), ValidationRunError>;
}
```

Sequenza:

```text
preflight input bundle
create context store
for each trace in deterministic order:
    validate trace
cross-record pass
```

---

# 78. Ordine deterministico dei trace

Non affidarsi all'iterazione di una `HashMap`.

Per reproducibilità:

- usare l'ordine dichiarato dallo schema;
- oppure ordinare esplicitamente per `TraceId`;
- la scelta deve essere stabile e testata.

---

# 79. Ordine deterministico degli errori

Per lo stesso input/schema, il report deve avere ordine stabile.

Ordine minimo:

```text
trace order
→ line number
→ field physical order
→ validation stage order
→ rule declaration order
```

Questo è essenziale per:

- golden tests;
- diff di report;
- audit.

---

# 80. Max errors

`ValidationOptions.max_errors` serve come protezione operativa.

Semantica proposta:

- una volta emesso il numero massimo di errori;
- terminare in modo controllato;
- summary deve indicare `truncated = true`.

Aggiungere quindi:

```rust
pub struct ValidationSummary {
    // ...
    pub truncated: bool,
}
```

Non classificare il file come completamente validato se l'analisi è stata troncata.

Possibile status futuro:

```rust
CompletedWithLimitReached
```

Per MVP è accettabile lasciare `max_errors = None` se non si vuole ancora gestire la semantica completa.

---

# 81. ProgressObserver

La futura UI richiede feedback senza contaminare il core.

```rust
pub trait ProgressObserver {
    fn on_event(&mut self, event: ProgressEvent);
}
```

Eventi iniziali:

```rust
pub enum ProgressEvent {
    RunStarted,
    PhaseStarted(ValidationPhase),
    TraceStarted { trace_id: TraceId },
    RecordsProcessed { trace_id: TraceId, count: u64 },
    TraceCompleted { trace_id: TraceId, count: u64 },
    RunCompleted,
}
```

Non inviare obbligatoriamente un evento per ogni record.

L'implementazione deve poter fare throttling per la UI.

---

# 82. ValidationPhase

```rust
pub enum ValidationPhase {
    Preflight,
    LocalValidation,
    CrossRecordValidation,
    FinalizingReport,
}
```

---

# 83. CancellationToken

Niente runtime async obbligatorio.

```rust
#[derive(Clone, Default)]
pub struct CancellationToken {
    cancelled: Arc<AtomicBool>,
}
```

API:

```rust
impl CancellationToken {
    pub fn cancel(&self);
    pub fn is_cancelled(&self) -> bool;
}
```

L'engine deve controllarlo:

- tra record;
- tra trace;
- prima della cross-record pass.

Non interrompere nel mezzo della costruzione di una singola struttura critica.

---

# 84. ValidationService: public application API

```rust
pub struct ValidationService {
    schema_registry: Arc<SchemaRegistry>,
    reference_data_provider: Arc<dyn ReferenceDataProvider>,
}
```

API principale:

```rust
impl ValidationService {
    pub fn validate(
        &self,
        request: ValidationRequest,
        sink: &mut dyn ValidationSink,
        progress: &mut dyn ProgressObserver,
        cancellation: &CancellationToken,
    ) -> Result<ValidationRunOutcome, ValidationRunError>;
}
```

Questa è la funzione principale che CLI e UI devono chiamare.

---

# 85. ValidationService sequence

```text
validate(request)
    │
    ├─ validate request syntax
    │
    ├─ schema_registry.resolve(schema_key)
    │      ├─ repository.load_raw
    │      └─ compiler.compile
    │
    ├─ preflight scope vs schema
    │
    ├─ prepare reference data
    │
    ├─ initialize summary
    │
    ├─ notify progress: RunStarted
    │
    ├─ mode == LocalOnly
    │      └─ TraceValidator / local flow traces
    │
    ├─ mode == Full
    │      └─ FlowValidator + ContextStore + cross pass
    │
    ├─ build ValidationSummary
    │
    ├─ sink.on_summary
    │
    ├─ notify progress: RunCompleted
    │
    └─ return ValidationRunOutcome
```

---

# 86. Preflight contract

```rust
pub fn preflight(
    request: &ValidationRequest,
    schema: &CompiledFlowSchema,
) -> Result<PreflightPlan, InvalidRequestError>;
```

Deve verificare:

- trace richiesto presente;
- nessun input duplicato per trace;
- mode compatibile con scope;
- input richiesti per cross-record presenti in `Full`;
- file paths non vuoti;
- eventuali opzioni coerenti.

`PreflightPlan` può contenere l'ordine di esecuzione già risolto.

---

# 87. Validazione locale: pseudocodice normativo

```rust
for record in reader {
    if cancellation.is_cancelled() {
        return cancelled_outcome;
    }

    summary.record_processed();

    if record.bytes.len() != trace_schema.record_length {
        emit_record_length_error(...)?;
        summary.record_invalid();
        continue;
    }

    let view = RecordView::new(record.bytes);
    let identity = identity_extractor.extract(&view, trace_schema);

    let mut record_valid = true;

    for field in &trace_schema.fields {
        let value = view.field(field).expect("compiled schema guarantees range");

        match run_field_pipeline(...) {
            Valid | MissingOptional => {}
            Invalid => record_valid = false,
            SkippedDueToUnknownPresence => { ... }
        }
    }

    for rule in &trace_schema.local_rules {
        if violated {
            emit(...)?;
            record_valid = false;
        }
    }

    if projection_plan.required {
        let projected = project_record(...)?;
        context_store.insert(projected)?;
    }

    if record_valid {
        summary.record_valid();
    } else {
        summary.record_invalid();
    }
}
```

L'uso di `expect` è ammesso solo su invarianti garantite dal compiled schema e preferibilmente evitato in path esterni.

---

# 88. Regola su panic

Il core non deve panicare su input utente o dati normativi.

Panic accettabili solo per:

- bug interni;
- invarianti realmente impossibili dopo schema compilation.

Preferire comunque API safe (`Option` / `Result`) nei confini principali.

---

# 89. Project record contract

```rust
pub fn project_record(
    record: &RecordView<'_>,
    schema: &CompiledRecordSchema,
    identity: RecordIdentity,
) -> Result<ProjectedRecord, ContextError>;
```

Deve copiare solo:

- campi elencati da `projection_plan.field_indexes`;
- join keys;
- identity.

---

# 90. Cross-record validation contract

```rust
pub fn validate_cross_record_rules(
    plan: &CrossRecordValidationPlan,
    store: &dyn ContextStore,
    sink: &mut dyn ValidationSink,
    summary: &mut ValidationSummaryBuilder,
    progress: &mut dyn ProgressObserver,
    cancellation: &CancellationToken,
) -> Result<(), ValidationRunError>;
```

Non deve rileggere i file se la projection plan è completa.

---

# 91. Missing related record policy

Le regole cross-record necessitano una semantica esplicita per record correlati mancanti.

Placeholder:

```rust
pub enum MissingRelatedRecordPolicy {
    Violation,
    NotApplicable,
    Unknown,
}
```

La scelta deve provenire dallo schema/regola, non dal motore.

Questo punto verrà formalizzato analizzando i tracciati completi.

---

# 92. Runtime dependencies

L'application service deve ricevere le dipendenze dall'esterno.

Esempio costruttore:

```rust
pub fn build_default_service(
    schema_root: PathBuf,
    reference_data_root: PathBuf,
) -> ValidationService;
```

Questo costruttore è soltanto una comodità per repository esterni. **Il contratto primario** prevede `ValidationService::new(dependencies)` con repository di schema/reference già risolti e repository persistente iniettato, così da supportare anche snapshot bundle incorporati; la risoluzione dei path resta negli adapter CLI/UI (Addendum A.1).

La funzione può costruire:

```text
FileSystemSchemaRepository
SchemaRegistry
FileSystemReferenceDataProvider
ValidationService
```

I test useranno fake/in-memory implementation.

Il core non ha directory di default: `schema_root` e `reference_data_root` sono sempre `PathBuf` espliciti. La risoluzione dei default (relativi all'eseguibile) è responsabilità degli adapter CLI/UI, vedi Addendum A.1.

---

# 93. Test doubles obbligatori

Predisporre:

```text
InMemorySchemaRepository
InMemoryReferenceDataProvider
VecValidationSink
NoopProgressObserver
CollectingProgressObserver
```

Questo permette integration test veloci e deterministici.

---

# 94. Public exports da `lib.rs`

`lib.rs` non deve esportare indiscriminatamente tutti i dettagli interni.

API pubblica minima iniziale:

```rust
pub use application::{
    ValidationRequest,
    ValidationMode,
    ValidationOptions,
    ValidationRunOutcome,
    ValidationRunError,
    ValidationService,
    CancellationToken,
    ProgressObserver,
    ProgressEvent,
};

pub use domain::{
    FlowId,
    TraceId,
    SchemaVersion,
    ValidationError,
    ValidationSummary,
};

pub use report::ValidationSink;
```

Tipi compiled interni possono essere `pub(crate)` finché non serve esporli.

---

# 95. CLI architecture

```text
clap args
   │
   ▼
CLI mapper
   │
   ▼
ValidationRequest
   │
   ▼
ValidationService
   │
   ▼
ValidationRunOutcome
   │
   ▼
exit code + human summary
```

`cli` non deve chiamare direttamente `RecordValidator`.

---

# 96. CLI validate command

Esempio:

```bash
lintra validate \
  --flow SIOC \
  --schema-version 2025-12-16 \
  --trace 2 \
  --input ./adt.txt \
  --errors ./errors.jsonl \
  --summary ./errors.summary.json
```

Full flow futuro:

```bash
lintra validate-flow \
  --flow SIOC \
  --schema-version 2025-12-16 \
  --trace-input 1=trace1.txt \
  --trace-input 2=trace2.txt \
  --trace-input 3=trace3.txt
```

La sintassi definitiva CLI può essere rifinita senza modificare il core.

---

# 97. Exit codes CLI

Proposta stabile:

```text
0   analisi completata, file conforme
1   errore tecnico / esecuzione non completata
2   analisi completata, violazioni normative presenti
130 cancellazione esplicita
```

Documentare gli exit code nel README.

---

# 98. Futura UI Rust

La UI deve eseguire `ValidationService::validate` su worker thread.

Non bloccare il thread grafico.

Interscambio consigliato:

```text
UI thread
   │
   ├── ValidationRequest
   ├── CancellationToken
   │
   ▼
worker thread
   │
   ├── ProgressObserver -> channel
   └── ProgressChannel -> contatori/progresso; la tabella analitica legge JSONL
            │
            ▼
         UI thread
```

Framework GUI vincolante: **Slint** (ARCH §39D); `egui` è soltanto piano B subordinato all’esito dello spike. Il prototipo Slint esistente non è un MVP.

---

# 99. UI data contract futuro

La UI deve ricevere dati dominio, non stringhe già formattate per terminale.

Canale risultati persistenti: **JSONL** paginato/indicizzato come fonte di dettaglio per la tabella; un eventuale canale live trasporta soltanto progresso e conteggi, non il backlog integrale degli errori.

Tipi di evento live (non sostituiscono la fonte JSONL):

```rust
ProgressEvent
ValidationSummaryDelta
```

Canale progresso:

```rust
ProgressEvent
```

Esito finale:

```rust
ValidationRunOutcome
```

Questo mantiene presentazione e core separati.

---

# 100. Logging tecnico

Usare logging strutturato per eventi applicativi.

Esempi:

```text
schema_resolved
reference_data_loaded
trace_started
trace_completed
cross_record_started
run_completed
input_error
report_error
```

Mai usare il log tecnico come report normativo.

---

# 101. Audit metadata

Creare:

```rust
pub struct RunMetadata {
    pub validator_version: String,
    pub schema_hash: String,
    pub reference_data_versions: BTreeMap<DatasetId, DatasetVersion>,
    pub started_at: SystemTime,
}
```

Lo `schema_hash` deve essere calcolato sul canonical schema effettivamente utilizzato o su una rappresentazione deterministica equivalente.

La strategia concreta di hashing può essere definita in implementazione.

---

# 102. Schema hash invariant

Due schema semanticamente identici ma serializzati con ordine JSON casuale non dovrebbero idealmente produrre hash diversi.

Preferire hash su rappresentazione canonicalizzata, non sui raw bytes originali del JSON.

Questa feature può essere implementata dopo l'MVP, ma il campo va previsto.

---

# 103. Error message generation

Il messaggio umano deve essere costruito da template controllati.

Esempio:

```text
Il campo {field_id} ({field_name}) è obbligatorio perché {condition}. Il valore risulta non compilato.
```

Non costruire messaggi completamente diversi in ogni validator.

La struttura machine-readable (`code`, `context`, `expected`) resta primaria.

---

# 104. Messaggio vs dati strutturati

Mai affidarsi al parsing del `message` per analisi successive.

Un consumer deve poter usare:

```text
code
field.id
record_identity
context
actual
expected
```

senza interpretare testo libero.

---

# 105. Gestione CF e ID record

Dato che i file di lavoro sono già anonimizzati, il report può conservare:

- Codice Fiscale;
- ID record SIOC;
- identificativi tecnici necessari.

Questi valori devono essere inclusi quando disponibili per permettere all'analista di risalire al record.

Il core non deve conoscere le posizioni di tali campi.

---

# 106. Schema formalization: cosa rimane intenzionalmente aperto

Non definire definitivamente prima di ricevere tutti i tracciati:

- JSON schema preciso dei flow schema;
- set definitivo di `FieldType`;
- set definitivo di `AlignmentRule`;
- forma completa della rule DSL;
- mapping dei codici OBB/FAC/NBB;
- regole di cross-trace;
- identità e join per ciascun flusso;
- formato dei reference dataset;
- versioning reale delle tabelle di codifica.

Queste non sono mancanze del design: sono punti che richiedono evidenza normativa.

---

# 107. Cosa deve essere fatto quando arrivano i tracciati regionali

Per ogni flusso:

1. identificare tutti i tracciati;
2. estrarre campi e posizioni;
3. normalizzare tipi;
4. classificare obbligatorietà;
5. estrarre domini;
6. individuare regole cross-field;
7. individuare regole cross-trace;
8. individuare reference dataset;
9. individuare campi identità;
10. individuare join keys;
11. individuare eventuali eccezioni al padding;
12. produrre canonical schema;
13. far passare schema linter;
14. creare fixture valide e invalide.

---

# 108. Reference dataset onboarding

Quando saranno disponibili le tabelle di codifica, per ogni dataset raccogliere:

```text
id dataset
nome ufficiale
versione / data validità
fonte
formato sorgente
campo chiave
lunghezza codice
encoding
normalizzazione ammessa
validità temporale
```

Poi definire il formato canonico runtime.

Non assumere che tutti i dataset siano semplici set di stringhe: alcuni potrebbero richiedere validità temporale o attributi aggiuntivi.

---

# 109. Possibile evoluzione dei reference dataset

Il contract `LookupSet::contains` copre il caso base.

Se emergono dataset temporali, il design potrà evolvere verso:

```rust
lookup(value, effective_date)
```

senza introdurre tale complessità finché non è richiesta.

---

# 110. Test strategy per modulo

## domain

- newtype validation;
- ByteRange;
- conversioni posizione.

## schema

- alias;
- gap/overlap;
- duplicate ID;
- ref resolution;
- invalid regex;
- projection plan.

## input

- LF;
- CRLF;
- trailing spaces;
- empty line;
- EOF senza newline finale;
- BOM.

## validators

- casi validi;
- casi invalidi;
- boundary.

## rules

- ogni operatore;
- `TruthValue`;
- unknown propagation.

## context

- join key;
- zero/one/multi match;
- projection.

## report

- JSON stable fields;
- Unicode/encoding error;
- deterministic ordering.

## application

- valid file;
- invalid file;
- schema failure;
- missing dataset;
- cancellation.

---

# 111. Golden tests

Struttura:

```text
tests/golden/
└── sioc/
    └── 2025-12-16/
        ├── trace-1/
        │   ├── valid/
        │   ├── required/
        │   ├── datatype/
        │   └── domain/
        ├── trace-2/
        └── cross-trace/
```

Ogni scenario contiene:

```text
input.txt
expected.errors.jsonl
expected.summary.json
```

Quando formalizzeremo SIOC, queste fixture diventeranno il contratto regressivo principale.

---

# 112. Property-based testing futuro

Candidati:

- `ByteRange` non panica;
- join key collision-safety per componenti semplici;
- parser newline;
- serializer/deserializer schema canonical round-trip.

Non è requisito MVP ma è compatibile.

---

# 113. Performance invariants

Nel path valido di una riga:

- un solo buffer riga riutilizzato;
- nessuna copia completa del record;
- slicing tramite `&[u8]`;
- niente `String` per ogni campo;
- lookup field O(1) quando la regola usa field ID compilato;
- lookup reference data idealmente O(1);
- allocazioni diagnostiche solo in caso di errore;
- nel loop di validazione si possiedono copie solo delle projection; il repository storico persistente CORE (§166–174) è una responsabilità separata, con retention e transazioni.

---

# 114. Benchmark targets

Non fissare ancora soglie arbitrarie.

Preparare benchmark per:

```text
10k record
100k record
1M record
```

misurando:

- record/s;
- peak memory;
- error-heavy vs valid-heavy;
- context projection cost.

La performance target va definita dopo aver conosciuto dimensioni reali dei flussi.

---

# 115. Parallelismo

Non introdurre nel primo MVP.

Motivi:

- output ordering;
- context store;
- cross-record dependencies;
- maggiore complessità ownership.

Il design non lo vieta in futuro.

Prima di parallelizzare bisogna mantenere determinismo del report.

---

# 116. Error-heavy inputs

Un file con ogni campo invalido può produrre un volume enorme di output.

Il motore deve:

- streammare;
- non collezionare tutto;
- aggiornare contatori incrementalmente;
- supportare eventualmente `max_errors`.

---

# 117. I/O errors

Se avviene un errore di lettura alla riga N:

- restituire `ValidationRunError::Input`;
- non classificare il file come validato;
- non produrre `CompletedInvalid`;
- il report parziale non deve essere presentato come completo.

---

# 118. Report errors

Se `sink.on_error()` fallisce:

- interrompere l'esecuzione;
- restituire `ValidationRunError::Report`;
- non continuare a validare perdendo errori.

---

# 119. Cancellation semantics

Se cancellato:

- chiudere correttamente le risorse;
- non eseguire ulteriori pass;
- finalizzare un summary di tipo `Cancelled` solo se il sink lo permette;
- l'output file definitivo non deve sembrare un report completo.

CLI può usare exit code 130.

Su Linux e Windows l'intercettazione di Ctrl+C avviene con un crate multipiattaforma (es. `ctrlc`) che imposta il `CancellationToken`. Il codice 130 è un contratto applicativo, valido su entrambi i sistemi, non una convenzione ereditata da Unix.

---

# 120. Schema compilation caching

Non necessario per MVP.

API future:

```text
SchemaKey → Arc<CompiledFlowSchema>
```

Cache invalidata se cambia il file dello schema o la versione.

Non introdurre premature global static mutable cache.

---

# 121. Modulo `application`

Responsabilità:

- orchestration top-level;
- use case API;
- preflight;
- progress/cancellation;
- costruzione summary.

Non deve:

- conoscere byte positions;
- validare datatype;
- implementare join logic di basso livello.

---

# 122. Modulo `domain`

Responsabilità:

- newtype;
- error/result model condivisi;
- summary;
- identity;
- byte range.

Deve essere il modulo più stabile e meno dipendente.

---

# 123. Modulo `schema`

Responsabilità:

- modelli raw;
- modelli compiled;
- repository interface;
- compiler;
- linter;
- DSL;
- validation plans.

Non deve leggere record `.txt`.

---

# 124. Modulo `input`

Responsabilità:

- apertura file;
- buffered record reading;
- newline handling;
- BOM handling;
- encoding metadata.

Non conosce regole normative.

---

# 125. Modulo `validators`

Responsabilità:

Funzioni pure o quasi pure per:

- presence;
- alignment;
- datatype;
- domain;
- range;
- date;
- reference lookup.

Non deve creare file o loggare errori applicativi.

---

# 126. Modulo `engine`

Responsabilità:

- pipeline;
- orchestration record/field;
- rule evaluation;
- error factory;
- identity extraction;
- trace validation;
- flow validation.

È il cuore runtime.

---

# 127. Modulo `context`

Responsabilità:

- join key;
- projected record;
- proiezioni effimere; la persistenza core è nel servizio dedicato (§166–174);
- lookup cross-record.

Non deve interpretare la normativa.

---

# 128. Modulo `reference_data`

Responsabilità:

- provider abstraction;
- caricamento dataset;
- lookup set;
- version metadata.

Il formato concreto verrà definito quando disponibili le fonti.

---

# 129. Modulo `report`

Responsabilità:

- sink abstraction;
- JSONL;
- CSV;
- rendering testo;
- output atomico;
- serialization.

Non deve decidere se un campo è valido.

---

# 130. Modulo `cli`

Responsabilità:

- parsing args;
- mapping in `ValidationRequest`;
- costruzione sink;
- rendering summary terminale;
- exit code.

---

# 131. Contratti file-per-file principali

## `schema/raw.rs`

Contiene solo strutture serializzabili/deserializzabili.

## `schema/compiled.rs`

Contiene strutture runtime già validate.

## `schema/compiler.rs`

Unico punto autorizzato a produrre compiled schema.

## `schema/linter.rs`

Funzioni di verifica strutturale riutilizzabili dal compiler e dal comando CLI `schema lint`.

## `input/reader.rs`

Implementa `RecordReader`.

## `engine/record_validator.rs`

Valida una riga.

## `engine/field_pipeline.rs`

Implementa sequenza presence → alignment → datatype → constraints.

## `engine/expression_evaluator.rs`

Implementa `TruthValue`.

## `engine/trace_validator.rs`

Loop buffered su un file/tracciato.

## `engine/flow_validator.rs`

Orchestra più trace e cross-record.

## `context/in_memory.rs`

Prima implementazione del context store.

## `report/jsonl.rs`

Output streaming.

---

# 132. Sequenza end-to-end: trace local

```text
CLI/UI
  │
  │ ValidationRequest
  ▼
ValidationService
  │
  ├── SchemaRegistry.resolve
  │       │
  │       ├── repository.load_raw
  │       └── compiler.compile
  │
  ├── prepare reference data
  │
  └── TraceValidator
          │
          ├── RecordReader
          │       │
          │       ▼
          │    RawRecord
          │
          ├── RecordValidator
          │       │
          │       ├── RecordView
          │       ├── IdentityExtractor
          │       ├── FieldPipeline
          │       └── local rules
          │
          ├── ValidationSink.on_error
          └── SummaryBuilder
```

---

# 133. Sequenza end-to-end: full flow

```text
ValidationService
  │
  ├── preflight full input bundle
  ├── load schema
  ├── prepare datasets
  ├── create ContextStore
  │
  ├── Trace 1 pass
  │      ├── local validate
  │      └── project context
  │
  ├── Trace 2 pass
  │      ├── local validate
  │      └── project context
  │
  ├── ...
  │
  └── cross-record pass
         ├── iterate projected records
         ├── resolve joins
         ├── evaluate rules
         └── emit errors
```

---

# 134. Stato del record

Non usare un oggetto record mutabile con decine di flag.

Stato minimo per iterazione:

```rust
struct RecordRuntimeState {
    line_number: u64,
    valid: bool,
    identity: RecordIdentity,
}
```

Non memorizzare `field_x_required`, `field_y_seen`, ecc.

---

# 135. Derived data

Se in futuro una regola necessita di un valore derivato, introdurre esplicitamente:

```rust
DerivedValueId
```

con un `DerivedValuePlan` nello schema compiler.

Non introdurre variabili globali mutate durante il parsing.

Questa feature non è MVP.

---

# 136. Date handling

`DateFormat` deve rappresentare il formato fisico atteso.

Esempio:

```rust
pub enum DateFormat {
    YyyyMmDd,
    DdMmYyyy,
    Custom(Arc<str>),
}
```

Non assumere `YYYYMMDD` per tutti i flussi senza evidenza.

---

# 137. Decimal handling

```rust
pub struct DecimalSpec {
    pub total_length: usize,
    pub scale: usize,
    pub decimal_separator: u8,
    pub sign_policy: SignPolicy,
}
```

La definizione concreta dipenderà dai tracciati.

---

# 138. Numeric range

Il validator range deve lavorare su valore già lessicalmente valido.

Non tentare di fare range check se il datatype check è fallito.

Questo preserva una sola causa primaria per stage.

---

# 139. AllowedValues

Confrontare sul valore fisico o semanticamente normalizzato secondo la regola.

La policy di normalization deve essere esplicita.

Non eseguire `trim()` implicitamente se il padding fa parte del tracciato.

---

# 140. Rule diagnostic context

Quando una regola dipende da campi X/Y/Z, il `ValidationError.context` deve includere almeno i valori che hanno determinato la violazione.

Il compiler può precomputare i field ref necessari alla diagnostica.

---

# 141. Schema linter e diagnostica

Il comando:

```bash
lintra schema lint ...
```

deve usare la stessa logica del compiler, non una seconda implementazione separata.

Possibile API:

```rust
pub fn lint_raw_schema(
    raw: &RawFlowSchema,
) -> Vec<SchemaIssue>;
```

`compile()` chiama internamente il lint e fallisce se ci sono issue `Error`.

---

# 142. Severity schema issue

```rust
pub enum SchemaIssueSeverity {
    Error,
    Warning,
}
```

Esempio warning possibile:

- nota normativa presente ma non trasformata in regola;
- metadata mancanti non indispensabili.

Gli errori impediscono compilazione.

---

# 143. Importer XLSX

`schema/importer.rs` deve essere tooling separato.

Contratto futuro:

```rust
pub trait SchemaImporter {
    fn import(
        &self,
        source: &Path,
    ) -> Result<RawFlowSchema, ImportError>;
}
```

L'import non garantisce automaticamente validità.

Dopo import:

```text
RawFlowSchema → lint → revisione → canonical schema
```

---

# 144. Nessun LLM nel runtime normativo

Un LLM può assistere l'autore dello schema durante l'analisi delle note.

Non deve essere parte della pipeline runtime che decide se un record è conforme.

Il runtime deve essere:

- deterministico;
- ripetibile;
- auditabile.

---

# 145. Repository dei documenti

Nel repository del progetto mantenere:

```text
docs/
  ARCHITECTURE.md
  DETAILED_DESIGN.md
  SCHEMA_AUTHORING.md        // futuro
  REFERENCE_DATA.md          // futuro
```

Quando formalizzeremo i tracciati, conviene produrre anche `SCHEMA_AUTHORING.md` con la sintassi canonica.

---

# 146. AI agent startup procedure

Un agente AI che implementa il progetto deve prima:

1. leggere `ARCHITECTURE.md`;
2. leggere `DETAILED_DESIGN.md`;
3. identificare il modulo da modificare;
4. verificare la direction delle dipendenze;
5. cercare contratti già definiti;
6. non inventare nuovi pattern se il contratto esiste;
7. se manca un requisito normativo, lasciare una boundary esplicita invece di inventare dati.

---

# 147. AI agent implementation policy

Quando una signature concreta differisce leggermente dallo pseudocodice di questo documento per esigenze idiomatiche Rust, l'agente può adattarla solo se:

- preserva ownership e responsabilità;
- non cambia il boundary;
- non introduce hardcoding;
- documenta la differenza.

Questo documento definisce **contratti semantici**, non obbliga a copiare ogni token di pseudocodice.

---

# 148. AI agent change report

A fine task l'agente deve riportare:

```text
Implemented
Files changed
Public contracts added/changed
Tests added
Architecture impact
Deferred items
```

Se cambia un public contract definito qui, deve dichiararlo esplicitamente.

---

# 149. AI agent forbidden shortcuts

Vietato:

- leggere direttamente XLSX dentro il validation loop;
- usare `HashMap` come ordine campo;
- usare `String` come rappresentazione primaria della riga;
- fare clone dell'intera riga per ownership;
- accumulare tutti gli errori per default;
- hardcodare codici SIOC nel core;
- trasformare note naturali in regole senza revisione;
- usare `unwrap()` su input esterno;
- trattare record correlato assente come `false` senza policy;
- prendere il primo record di una join multipla senza cardinalità;
- usare l'ultima versione schema implicitamente;
- mescolare application log e validation report.

---

# 150. Implementation order dettagliato

## Milestone A — Skeleton e domain

Creare:

- package;
- `lib.rs`;
- `domain/ids.rs`;
- `domain/byte_range.rs`;
- `domain/errors.rs`;
- `domain/summary.rs`.

Test:

- newtype;
- ByteRange.

## Milestone B — Schema core

Creare:

- raw model minimo;
- compiled model minimo;
- linter;
- compiler;
- file repository.

Supportare inizialmente:

- Numeric;
- Alphanumeric;
- Required/Optional;
- posizione/lunghezza.

## Milestone C — Input layer

Creare:

- RecordReader;
- LF/CRLF;
- record length;
- RecordView.

## Milestone D — Reporting

Creare:

- ValidationError;
- ValidationSink;
- JsonlSink;
- VecSink;
- ErrorFactory.

## Milestone E — Local field validation

Creare:

- missing policy;
- presence;
- alignment base;
- datatype;
- allowed values;
- field pipeline.

## Milestone F — Application service

Creare:

- ValidationRequest;
- ValidationService;
- summary;
- progress;
- cancellation.

A questo punto è disponibile un MVP locale.

## Milestone G — DSL

Creare:

- CompiledExpr;
- TruthValue;
- ExpressionEvaluator;
- RequiredIf;
- ForbiddenIf;
- cross-field.

## Milestone H — Reference data

Creare provider abstraction e dataset runtime quando disponibili dati reali.

## Milestone I — Cross-record

Creare:

- JoinKey;
- projection plan;
- ProjectedRecord;
- InMemoryContextStore;
- Full mode.

## Milestone J — UI

Dopo stabilizzazione CLI/core. Slint resta separato da library/CLI, come da ARCH §39D.

---

# 151. MVP detailed Definition of Done

Gate **M3** («MVP locale» nel testo storico): milestone intermedia, NON il **MVP complessivo M5** definito in ARCH §43 e nel piano §8. M3 pronto quando:

- il crate espone `ValidationService`;
- schema raw viene caricato da file;
- schema compiler produce compiled schema;
- linter blocca gap/overlap/ref invalidi;
- un `.txt` viene letto in streaming;
- LF/CRLF sono gestiti senza trim dei dati;
- record length viene verificata;
- campi vengono estratti con `ByteRange`;
- presence funziona;
- almeno Numeric/AN funzionano;
- alignment minimo funziona;
- AllowedValues funziona;
- errori vengono streammati in JSONL;
- summary distingue valid/invalid;
- `CompletedInvalid` è `Ok`;
- unit/integration/golden test passano;
- non esiste hardcoding SIOC nell'engine.

---

# 152. Full-flow Definition of Done

Full validation pronta quando:

- schema definisce join;
- compiler calcola projection plan;
- pass1 produce projected records;
- context store indicizza join key;
- pass2 esegue cross-record rules;
- missing related record ha policy esplicita;
- report include context cross-trace;
- golden tests coprono almeno una regola cross-trace reale.

---

# 153. Decisioni da non prendere ancora

Le seguenti **tecnologie opzionali** richiedono ancora evidenza prima di scegliere:

- eventuale driver database esterno specifico (la **persistenza core generica è già decisa**);
- parallelismo;
- REST API;
- formato definitivo reference data;
- versioning temporale dataset;
- DSL completa oltre gli operatori necessari;
- auto-import completo XLSX.

---

# 154. Input richiesti per la prossima fase di progettazione normativa

Per formalizzare gli schemi dei flussi servono:

1. tracciati regionali ufficiali completi;
2. eventuali documenti di accompagnamento;
3. note su versioni/date di efficacia;
4. esempi di file validi, se reperibili;
5. codifiche esterne richiamate;
6. eventuale identificazione delle chiavi di correlazione tra tracciati.

Per reference data:

1. file ufficiali;
2. versione/data;
3. fonte;
4. regole di validità;
5. eventuale storicizzazione.

---

# 155. Prima formalizzazione da produrre quando arrivano i tracciati

Il prossimo documento tecnico da creare dopo questo detailed design dovrebbe essere:

> `SCHEMA_AUTHORING.md`

con:

- JSON Schema del canonical flow schema;
- esempi completi;
- sintassi DSL;
- mapping da colonne regionali;
- presence policies;
- join definitions;
- identity extraction;
- reference data bindings;
- regole di versioning.

Successivamente:

> `REFERENCE_DATA_SPEC.md`

quando saranno disponibili i dataset.

---

# 156. Decision record: streaming errors

**Decisione:** `ValidationSink` invece di `Vec<ValidationError>` come output primario.

Motivo:

- file grandi;
- error-heavy input;
- UI futura;
- JSONL naturale;
- memoria prevedibile.

`VecSink` esiste solo per test o piccoli casi.

---

# 157. Decision record: compiled schema

**Decisione:** l'engine usa solo compiled schema.

Motivo:

- nessuna normalizzazione per record;
- field ref risolti;
- posizioni sicure;
- regex precompilate;
- projection plan già calcolato;
- invarianti forti.

---

# 158. Decision record: two-phase full validation

**Decisione:** local validation + context projection, poi cross-record validation.

Motivo:

- evita flag temporanei;
- evita dipendenza dall'ordine dei record;
- evita rilettura del file quando il projection plan è sufficiente;
- supporta relazioni cross-trace.

---

# 159. Decision record: error semantics

**Decisione:** violazione normativa ≠ runtime failure.

```text
VALID             → Ok(CompletedValid)
INVALID           → Ok(CompletedInvalid)
CANCELLED         → Ok(Cancelled)
I/O/SCHEMA/etc.   → Err(ValidationRunError)
```

---

# 160. Decision record: byte-oriented core

**Decisione:** `&[u8]` prima di `String`.

Motivo:

- posizioni fixed-width;
- encoding legacy;
- zero-copy slicing;
- nessun ambiguity UTF-8.

---

# 161. Decision record: deterministic output

**Decisione:** l'ordine degli errori è un contratto.

Motivo:

- audit;
- test golden;
- diff;
- riproducibilità.

Il parallelismo futuro deve preservare questo contratto.

---

# 162. Decision record: UI boundary

**Decisione:** UI come adapter sullo stesso `ValidationService`.

Non creare un secondo engine per GUI.

---

# 163. Checklist per code review AI

Per ogni PR/task verificare:

```text
[ ] Nessun hardcoding normativo nel core
[ ] Nessun clone completo della riga senza motivo
[ ] Nessun trim indiscriminato
[ ] Nessuna dipendenza CLI dentro engine
[ ] Nessun Vec globale di tutti gli errori
[ ] Error ordering deterministico
[ ] Result distingue invalid da runtime failure
[ ] Nuovo field ref risolto in compile phase
[ ] Nuova regola ha test true/false
[ ] Cross-record rule ha join/cardinality policy
[ ] Output machine-readable preservato
[ ] cargo fmt
[ ] cargo clippy
[ ] cargo test
```

---

# 164. Invarianti globali del sistema

Al termine del preflight:

```text
schema compiled
trace selection valid
reference dependencies loaded
output sink writable
```

Per ogni record a lunghezza valida:

```text
ogni ByteRange del trace è slice-safe
```

Per ogni compiled local ref:

```text
field_index esiste
```

Per ogni cross-record rule:

```text
projection plan contiene tutti i campi necessari
```

Per ogni errore emesso:

```text
flow/version/trace/line sono valorizzati
```

Per errori field-level:

```text
field id/name/start/end sono valorizzati
```

---

# 165. Nota conclusiva per gli agenti AI

Questo documento deve ridurre al minimo le decisioni improvvisate durante l'implementazione.

Quando manca un dato normativo reale, non riempire il vuoto con un'assunzione.

Creare invece:

- un tipo;
- una trait;
- una policy configurabile;
- un TODO documentato;

che mantenga aperto il punto fino a quando arriva la fonte ufficiale.

Il progetto deve restare:

```text
generico
+ deterministico
+ auditabile
+ schema-driven
+ byte-safe
+ testabile
+ estendibile
```

Principio finale:

> **La normativa entra nel sistema come dato versionato e validato; non come comportamento nascosto nel codice.**


# Requisito non funzionale: supporto Linux e Windows

## Supported platforms

L'MVP deve supportare ufficialmente:

```text
Linux x86_64
Windows x86_64
```

Questo requisito deve influenzare direttamente i contratti e le scelte implementative.

## Contratti di portabilità

### Filesystem paths

Tutte le API interne che accettano o restituiscono path devono usare `Path` / `PathBuf` e non `String` quando il valore rappresenta un percorso filesystem.

Esempio preferito:

```rust
pub struct ValidationRequest {
    pub input_path: PathBuf,
    pub output_path: PathBuf,
    // ...
}
```

Sono vietati:

```rust
let p = format!("{}/schemas/{}", base, name);
```

e path assoluti specifici della macchina di sviluppo.

### Line endings

`RecordReader` deve accettare record terminati da:

```text
LF   = \n
CRLF = \r\n
```

Il terminatore deve essere rimosso prima di applicare `record_length`. La lunghezza normativa riguarda il payload fixed-width, non i byte del line ending.

Il contratto del reader deve quindi garantire che il buffer consegnato a `RecordView` non includa `\n` o `\r\n`.

### Dipendenze di sistema

Il prodotto non deve richiedere utility Unix a runtime (`grep`, `sed`, `awk`, shell script Bash, ecc.). Ogni funzionalità necessaria all'applicazione deve essere implementata nel binario Rust o tramite crate multipiattaforma.

### Schema e reference data

`SchemaRepository` e `ReferenceDataProvider` devono ricevere directory configurabili tramite `PathBuf`; nessun path deve dipendere dal layout Linux della macchina di sviluppo.

### Output

L'output JSONL deve essere identico **byte per byte** sui due sistemi, a parità di input e schema: UTF-8 senza BOM e terminatore `\n`. Gli output CSV e XLSX devono avere contenuto semanticamente identico (stesse righe, stesso ordine, stessi valori). I test golden non devono normalizzare nulla sul JSONL; eventuali normalizzazioni sul CSV devono essere dichiarate e non nascondere differenze reali.

## CI matrix obbligatoria

Ogni pull request e branch principale devono essere verificati almeno su:

```yaml
os:
  - ubuntu-latest
  - windows-latest
```

Per entrambi:

```text
cargo fmt --check
cargo clippy -- -D warnings
cargo test
cargo build --release
```

Un cambiamento non è completato se passa soltanto su Linux.

## Release artifacts

La pipeline di release deve generare almeno, **solo da CI**:

```text
Release dell'eseguibile
  lintra-<version>-linux-x86_64.tar.gz
  lintra-<version>-windows-x86_64.zip
  SHA256SUMS

Release dei dati
  schemas-<bundle_version>.zip
  SHA256SUMS
```

L'eseguibile contiene uno snapshot di default di schemi e reference data; l'archivio dei dati si aggiorna separatamente (Addendum A.1).

Quando verrà introdotta la UI, lo stesso requisito si applicherà al relativo eseguibile desktop.

## Test specifici multipiattaforma

Prevedere test per:

- file con terminatori LF;
- file con terminatori CRLF;
- path con spazi;
- path relativi;
- directory temporanee tramite API Rust portabili;
- caricamento schema/reference data su entrambi i runner;
- output identico a parità di input e schema.

## Definition of Done aggiuntiva

Una feature che interagisce con filesystem, I/O, CLI, schema repository o report è completata solo se:

1. non introduce assunzioni Linux-specifiche;
2. compila su Windows;
3. supera i test Windows;
4. utilizza primitive Rust multipiattaforma;
5. non richiede modifiche manuali al codice per cambiare sistema operativo.

---


# Addendum A (revisione 2026-10-08): sviluppo su CachyOS e distribuzione Windows

Questo addendum concretizza `ARCHITECTURE.md` §5.8, §39A, §39B e §39C a livello di contratti.

## A.1 Origine di schemi e reference data (modulo `cli/paths.rs`)

La risoluzione sta negli adapter, non nel core (§92).

```rust
pub enum BundleSource {
    External {
        schema_root: PathBuf,
        reference_data_root: PathBuf,
    },
    Embedded,
}

pub struct CliDirs {
    pub schema_dir: Option<PathBuf>,
    pub reference_data_dir: Option<PathBuf>,
}

pub trait EnvProvider {
    fn var_os(&self, key: &str) -> Option<OsString>;
}

pub fn resolve_bundle_source(
    cli: &CliDirs,
    env: &dyn EnvProvider,
    exe_path: &Path,
) -> Result<BundleSource, BundleResolveError>;
```

Ordine di risoluzione:

1. opzioni CLI;
2. variabili d'ambiente `LINTRA_SCHEMA_DIR` / `LINTRA_REFERENCE_DATA_DIR`;
3. directory `schemas/` e `reference-data/` accanto all'eseguibile (`exe_path.parent()`);
4. altrimenti `BundleSource::Embedded`.

Regole:

- un'opzione o una variabile che punta a una directory inesistente è `BundleResolveError`, non un motivo per scendere al livello successivo;
- cartella esterna incompleta (una sola delle due directory presente) è `BundleResolveError` con l'elenco di ciò che manca;
- con `External` si usa solo la cartella esterna, mai un mix con lo snapshot; se manca lo schema o la versione richiesti il risultato è un errore di esecuzione, senza ripiego sullo snapshot;
- `exe_path` viene passato dal chiamante (`std::env::current_exe()` in `main`), così la funzione è testabile senza dipendere dal processo reale;
- `EnvProvider` evita test che modificano l'ambiente globale;
- la working directory non è mai un default;
- è vietato qualunque path ricavato da `env!("CARGO_MANIFEST_DIR")` o simili fuori da test e build script.

Costruzione del servizio: `ValidationService` continua a ricevere `Arc<dyn SchemaRepository>` e `Arc<dyn ReferenceDataProvider>` (§92). L'adapter costruisce `FileSystemSchemaRepository` / `FileSystemReferenceDataProvider` per `External`, oppure `EmbeddedSchemaRepository` / `EmbeddedReferenceDataProvider` per `Embedded`. Il core non conosce la distinzione. Le implementazioni incorporate vivono dietro la feature `embedded-bundle` e leggono dati inclusi a compile time (es. con `include_dir` o `rust-embed`, non vincolanti).

Compatibilità motore/dati:

```rust
pub struct SchemaFormatVersion(pub u32);

pub struct EngineCapabilities {
    pub supported_schema_formats: RangeInclusive<SchemaFormatVersion>,
    pub engine_version: String,
}
```

- ogni `RawFlowSchema` dichiara il proprio `schema_format_version`;
- il `SchemaCompiler` rifiuta un formato fuori dal range supportato con un errore di esecuzione che indica la versione minima dell'eseguibile richiesta;
- il bundle di dati ha un `MANIFEST.json` con `bundle_version`, `schema_format_version` e hash dei file; lo snapshot incorporato ha lo stesso manifest.

Audit (§35): ogni esecuzione registra `bundle_source` (`external` / `embedded`), `bundle_version`, `schema_hash` e le versioni dei dataset.

## A.2 Contratti di output cross-platform

- `JsonlSink` scrive UTF-8 senza BOM e termina ogni record con `\n`, mai tramite API che dipendano dalla piattaforma;
- `CsvSink` scrive solo RFC 4180 (UTF-8 senza BOM, `,`, CRLF); nessun dialetto specifico per Excel;
- `XlsxReportWriter` è il writer post-analisi principale per gli analisti (non implementa `ValidationSink`). Non scrive dentro il ciclo di validazione: genera il file a fine analisi leggendo il flusso di errori già prodotto (il JSONL, fonte canonica), così lo streaming del core non cambia. Contratto:
  - fogli `Riepilogo` ed `Errori`;
  - colonne dell'errore: codice, flusso, versione schema, tracciato, riga, ID record, Codice Fiscale, field ID, nome campo, start, end, actual, expected, messaggio;
  - ogni valore di dato è scritto come cella di testo, mai come numero o data;
  - intestazione fissa e filtri automatici;
  - superato il limite di righe per foglio (1.048.576 comprese le intestazioni) continua in fogli successivi (`Errori (2)`, ...), preservando l'ordine del JSONL;
  - verificare nel crate scelto (es. `rust_xlsxwriter`) la modalità a memoria costante prima di adottarla;
- il report `human` verso console usa le primitive di stdout di Rust (che su Windows gestiscono correttamente la console); il report su file passa sempre da `--output`;
- i timestamp di audit sono UTC, RFC 3339;
- l'audit dell'input registra nome file, dimensione e SHA-256; il path assoluto non entra nel report né negli hash;
- nessun path canonicalizzato in output (su Windows `canonicalize()` restituisce il prefisso `\\?\`).

## A.3 Verifica dell'artefatto

Un job CI (o `cargo xtask verify-artifact`) deve fallire se:

- l'eseguibile Windows importa DLL diverse da quelle di sistema o richiede `libgcc_s_*`, `libwinpthread-*`, `VCRUNTIME*`, `MSVCP*`;
- l'eseguibile CLI Linux ha dipendenze dinamiche oltre la libc di base (oppure, per musl, non è statico); per l'eseguibile UI vale l'elenco esplicito di librerie di sistema ammesse (Addendum B.1);
- il binario contiene sottostringhe che indicano path dell'host di build (`/home/`, directory del `CARGO_HOME`);
- `cargo tree -i openssl-sys` non è vuoto.

## A.4 Test aggiuntivi

Oltre a quelli già elencati nella sezione su Linux e Windows:

- file di fixture e golden **mai** alterati da Git: nel `.gitattributes` del repository

```text
* text=auto
tests/fixtures/** -text
tests/golden/**   -text
```

  Senza questa regola il runner Windows può convertire LF in CRLF nei file `.txt` di input e falsare sia i test sulla lunghezza del record sia i golden;
- file con BOM UTF-8 e CRLF come li produrrebbe Blocco note: verificare errore chiaro con `BomPolicy::Reject` e successo con `StripKnownBom`;
- bundle estratto in una directory temporanea con spazi nel nome e lanciato da una working directory diversa: schemi e reference data devono essere trovati tramite il livello 3 della risoluzione;
- `resolve_bundle_layout` con `EnvProvider` fittizio per ciascuno dei quattro livelli;
- `XlsxReportWriter`: rilettura del file prodotto con un parser XLSX di test, verificando che valori come `00123` e `05` tornino identici come stringhe, che l'ordine delle righe coincida con il JSONL e che il superamento del limite di righe per foglio venga gestito;
- `resolve_bundle_source` con `EnvProvider` fittizio per ciascun livello, incluse cartella incompleta e variabile che punta a una directory inesistente;
- schema con `schema_format_version` non supportato: errore che indica la versione minima dell'eseguibile;
- stesso input eseguito con `External` e con `Embedded` a parità di contenuto: stesso output, `bundle_source` diverso nell'audit;
- confronto byte per byte dell'output JSONL generato su runner Linux e Windows per gli stessi input.

## A.5 Ciclo di sviluppo su Arch

Prima di dichiarare completata una task (aggiunta rispetto al §163):

```text
cargo fmt
cargo clippy --locked
cargo test --locked
cargo check --target x86_64-pc-windows-gnu
```

L'ultimo passo non sostituisce la CI su runner Windows. Il toolchain è fissato da `rust-toolchain.toml` (installazione via `rustup`, vedi `ARCHITECTURE.md` §39B).

Aggiunta alla checklist di code review (§163):

```text
[ ] Nessun path o costante dell'host di build usato a runtime
[ ] Nessuna API Linux-only fuori da moduli isolati con #[cfg]
[ ] Nessuna dipendenza che richieda librerie di sistema
[ ] Output JSONL invariato byte per byte tra i due sistemi
```


# Addendum B (revisione 2026-10-09): interfaccia desktop con Slint

Concretizza `ARCHITECTURE.md` §39D.

## B.1 Struttura del workspace

Il package attuale diventa membro di un workspace; la UI è un secondo membro.

```text
Cargo.toml                 (workspace)
├── (package core + CLI)   come da §4, senza dipendenze grafiche
└── ui/                    crate dell'eseguibile UI
    ├── Cargo.toml
    ├── build.rs
    ├── ui/
    │   ├── tokens.slint
    │   ├── results.slint
    │   └── fonts/
    ├── src/
    └── tests/
        ├── reference/     PNG di riferimento (1x, 2x)
        └── visual.rs
```

Controllo delle dipendenze Linux per l'eseguibile UI: elenco esplicito di librerie di sistema ammesse (finestra e input: X11 o Wayland e relative), mantenuto nel job di verifica dell'artefatto (A.3). Qualunque libreria non in elenco fa fallire il job.

## B.2 Contratti

- La UI dipende solo dall'API pubblica della library (§94): `ValidationService`, `ValidationRequest`, `ProgressObserver`, `CancellationToken`.
- Il thread dell'interfaccia non esegue la validazione: l'analisi gira su un thread separato; avanzamento ed esito tornano al thread UI tramite il meccanismo di Slint per aggiornamenti da altri thread.
- L'annullamento imposta il `CancellationToken`; l'esito `Cancelled` è mostrato come tale, mai come report completo (§119).
- La tabella degli errori legge dal JSONL prodotto (letture a finestre/indice), non da un `Vec` globale (§3.4).
- La schermata «Informazioni» include l'attribuzione richiesta dalla licenza di Slint e le versioni di eseguibile, schema e bundle.

## B.3 Test visivi

- `tests/visual.rs` renderizza `Risultati` con il renderer software a dimensione fissa e confronta con `tests/reference/*.png` con tolleranza per zona.
- Se il riferimento manca, il test scrive l'immagine in `target/visual/` e non fallisce, ma lo segnala: il riferimento va approvato visivamente prima di essere versionato.
- Le deviazioni note dal mockup sono elencate nel README del prototipo e non nascoste da soglie larghe.


---

# 166. Decisione 2026-10-16 — Persistenza core e responsabilità

La persistenza non è un'estensione CAD esclusiva: è un modulo/servizio applicativo **core** usato secondo policy diverse da tutti i flussi. Le regole normative sono nello schema/versioned profile e non nei driver. Il precedente `ContextStore` (§71) è un indice effimero di proiezione nella validazione a due passate; non deve implementare implicitamente storia, sessioni e commit. Contratto normativo: `PERSISTENCE_CORE_SPEC.md` e ARCH §45.

La costruzione di `ValidationService` deve ricevere le dipendenze esplicite incluse `PersistentContextRepository` e `Clock` iniettabile/testabile; la modalità puramente locale può avere un repository `NoOp` dichiarato. CLI/UI risolvono percorsi e storage config. La scelta della libreria SQLite/file non è decisa implicitamente.

# 167. Tipi e contratto del repository persistente

Il dominio aggiunge tipi nominati: `PersistenceNamespace`, `PersistentRecordIdentity`, `SourceSubmission`, `SubmissionPeriod`, `TracePresence`, `HistoricalSnapshot`, `SessionState`, `RecordFingerprint`, `HistoryDiff`, `HistoricalError`. I tipi per record sono ottenuti dalla compilazione delle chiavi schema e dalle proiezioni minime.

Interfaccia **concettuale** (le firme Rust vengono precisate nella task senza cambiare la semantica):

```rust
pub trait PersistentContextRepository: Send + Sync {
    fn lookup_identity(&self, namespace: &PersistenceNamespace, identity: &PersistentRecordIdentity) -> Result<Option<StoredIdentity>, PersistenceError>;
    fn load_current_snapshot(&self, namespace: &PersistenceNamespace) -> Result<Option<HistoricalSnapshot>, PersistenceError>;
    fn begin_transaction(&self, namespace: &PersistenceNamespace, expected_generation: u64) -> Result<Box<dyn PersistenceTransaction>, PersistenceError>;
}

pub trait PersistenceTransaction {
    fn stage_snapshot(&mut self, snapshot: &HistoricalSnapshot) -> Result<(), PersistenceError>;
    fn commit(self: Box<Self>) -> Result<CommitReceipt, PersistenceError>;
    fn abort(self: Box<Self>) -> Result<(), PersistenceError>;
}
```

`StoredIdentity` **non contiene per default i valori sensibili in chiaro**. Identità PK/progressivo/evento sono pseudonimizzate in maniera stabile e verificabile secondo una policy di protezione; il preflight blocca uso reale con credenziali/chiavi necessarie non configurate. Il repository non conosce stringhe literal `SIOC`, `CAD` o ID campo.

# 168. PersistencePolicy raw e compiled

Un profilo normativo versionato specifica `run_uniqueness`, `cross_run_audit`, eventuale `cumulative_snapshot`, `history_identity_invariants`, `snapshot_scope`, `temporal_rules`, `missing_trace_policy`. Il `SchemaCompiler` normalizza e linter verifica riferimenti alle PK e versioni. Un profilo incompatibile o non supportato causa **schema error**, non il falso superamento di controlli. Il contratto di serializzazione definitivo deve essere definito/approvato in `SCHEMA_AUTHORING.md` senza modificare retroattivamente i JSON SIOC FROZEN.

# 169. Sequenza transazionale e gestione dei report

Validazione locale/full → estrazione contesto → preview storica → diagnostica/errori → eventuale consenso esplicito → approvazione sessione → commit atomico e idempotente. **Mai** aggiornare lo snapshot corrente a metà validazione o in fase di preview. `Cancelled`, errori I/O o sessione definitivamente approvata non producono mutazioni. Check di concorrenza CAS/locking; un risultato storico deve dichiarare esattamente lo snapshot di base e il numero di generazione. L'output di report e l'aggiornamento dello storage non devono essere confusi con un'unica transazione se non è garantita atomicità: al fallimento del report non committare; al fallimento del commit segnalare l'esito senza falso successo.

# 170. Policy SIOC minima e limiti normativi

Lo schema SIOC FROZEN ha `record_key` su tutti i cinque tracciati con `uniqueness_scope: trace` e `E_DUPLICATE_RECORD_KEY`. Il core deve validare tale unicità **dentro un invio** e persistere indice/digest delle chiavi per audit e successivi confronti. Non generare errore su una PK che ricompare in un invio differente **senza policy inter-invio approvata** (DEC-SIOC-HIST-01). Questo è un limite deliberato, non una lacuna dell'implementazione del repository.

# 171. Policy CAD sulla stessa infrastruttura

Per CAD usare sessioni per ULSS e trimestre, storico continuativo fra anni, tracciati presenti/assenti memorizzati, warning di missing e `PROCEED`/`ABORT` dell'utente. Un file assente non può annullare i record storici del suo trace. Progressivo/evento associati immutabilmente, ricomparsa stesso evento = riattivazione valida; altro evento = errore. Le sessioni approvate diventano immutabili; reinvio e sostituzione solo durante la fase di preparazione della stessa sessione, mai retroattivamente dopo approvazione. Le PK C e D includono i rispettivi progressivi; la correlazione D→C usa i quattro campi comuni senza progressivo. CAD-14 e CAD-17 restano piani temporali/versionati specifici del flusso.

# 172. Protezione dati nello storage

Persistere **soltanto proiezioni strettamente necessarie** e metadati audit: `flow_id`, versioni/hash, periodo, namespace, conteggi, `TracePresence`, identity digest, fingerprint record, eventuali date/stati per regole eseguibili. Vietati dump di righe/CF in chiaro nello storage di default. Un digest semplice di CF non è anonimizzazione; il design deve prevedere protezione keyed, configurazione sicura delle chiavi, ACL, backup/restore e retention prima di abilitare esecuzioni reali.

# 173. Esiti ed errori di persistenza

Duplicazioni e riutilizzo normativo illecito sono `ValidationError` (ad esempio `E_DUPLICATE_RECORD_KEY`); storage corrotto, I/O, lock, conflitti concorrenziali e chiavi mancanti sono `ValidationRunError`/preflight; `ABORT` esplicito preserva lo stato storico. Errori di report persistono nel JSONL seguendo il sink originale; un diff storico è un report ad hoc con schema/versione.

# 174. Testing e implementazione incrementale

Staging atomico, replay, interruzione/rollback, concurrency e restart; test SIOC intra-run e cross-run audit; CAD T1/T2/T3 reactivation, missing traces, tempo e invio approvato immutabile; test failpoint crash/recovery; benchmark memoria. Sequenza: tipi/trait minimi P2/P3/P7/P13 → task core `T-PERSIST-01..12` → profili CAD `T-CAD-01..12`; la UI e il database specifico non sono prerequisiti per definire i contratti. I test con informazioni sanitarie reali NON vanno versionati nel repository.
