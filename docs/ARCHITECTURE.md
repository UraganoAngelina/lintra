# Regulatory Fixed-Width Validation Engine
## Guida architetturale e di implementazione per agenti AI

> **Stato:** specifica architetturale di riferimento, revisione consolidata 2026-10-16 (Lintra + persistenza core).
>
> Questo documento è la **source of truth** per l'implementazione del progetto. Un agente AI deve leggerlo prima di modificare il codice e trattare i principi qui descritti come decisioni architetturali deliberate.
>
> **Revisione 2026-10-16:** riconciliazione approvata con MASTER_IMPLEMENTATION_PLAN: aggiunte §39A–D dalla revisione architetturale 2026-10-09; decisioni CNF-01..11 allineate; §45 sulla persistenza core schema-driven per SIOC/CAD, con contratto dettagliato in `PERSISTENCE_CORE_SPEC.md`. Nome progetto: Lintra.
>
> **Revisione 2026-10-08:** aggiornato il contesto di sviluppo (CachyOS / Arch Linux) e il requisito di distribuzione (binario Windows autosufficiente, senza dipendenze dalla macchina Linux di sviluppo). Sezioni toccate: §5.1 (chiarimento), §5.8 (nuova), §39A (riscritta), §39B e §39C (nuove), §42 e §43 (checklist). Le quattro decisioni rimaste aperte sono state chiuse il 2026-10-08 (§39B.2, §39C.2, §39C.3, §39C.4); sono aggiunte §39C.5 e la voce XLSX in §4, §28, roadmap e §43. **Revisione 2026-10-09:** scelta del framework UI (Slint) in §39 e nuova §39D con i vincoli dell'interfaccia desktop.

---

# 1. Visione

Realizzare un **motore generico, modulare, configurabile e versionato di validazione per flussi sanitari a record posizionali fixed-width**.

Il primo caso d'uso è SIOC, ma il core non deve dipendere da SIOC. Nuovi flussi devono poter essere aggiunti principalmente tramite nuovi schemi e reference data, senza modificare il motore.

Input minimi:

- file `.txt` posizionale;
- `flow_id`;
- `trace_id`;
- `schema_version`;
- eventuale configurazione di encoding;
- eventuali reference data richieste.

Output:

- report strutturato degli errori;
- summary dell'esecuzione;
- messaggio esplicito di conformità se non vengono rilevati errori.

Nome prodotto: **Lintra**. Nome descrittivo del motore: **Configurable Fixed-Width Regulatory Validation Engine**.

---

# 2. Contesto funzionale

I flussi sanitari vengono prodotti secondo tracciati regionali che definiscono, per ciascun campo:

- ID campo;
- descrizione;
- tipo;
- posizione iniziale;
- lunghezza;
- obbligatorietà;
- note/regole.

Le regole reali possono includere:

- campi obbligatori;
- campi facoltativi;
- obbligatorietà condizionale;
- divieti condizionali di compilazione;
- padding/allineamento;
- tipo lessicale;
- dominio dei valori;
- intervalli numerici;
- date;
- codifiche esterne;
- dipendenze tra campi;
- dipendenze tra tracciati;
- consistenza cross-record.

Il software non deve limitarsi al parsing: deve essere un **validation engine rule-driven**.

---

# 3. Principio architetturale centrale

Separare rigorosamente:

```text
COME si valida                    COSA è valido
──────────────────────            ──────────────────────
Validation Engine                 Schema normativo
Rust core                         JSON/canonical schema
tipi                              posizioni
validator                         lunghezze
DSL                              domini
reference lookup                  obbligatorietà
reporting                         dipendenze normative
```

Il motore conosce concetti generici.

Lo schema conosce SIOC o qualunque altro flusso.

Sono vietati nel core pattern come:

```rust
if flow == "SIOC" { ... }

if field_id == "2.29" { ... }
```

salvo codice di test esplicitamente circoscritto.

---

# 4. Linguaggio

Implementazione preferita:

> **Rust**

La scelta non è motivata solo dalla performance, ma soprattutto da:

- memory safety;
- tipizzazione forte;
- enum espressivi;
- parsing efficiente di byte;
- deployment tramite singolo binario;
- performance prevedibili;
- facilità nel separare core fortemente tipizzato e configurazione normativa dinamica.

Librerie candidate, non vincolanti:

- `serde`, `serde_json`;
- `clap`;
- `calamine`;
- `encoding_rs`;
- `thiserror` / `anyhow`;
- `tracing`;
- `csv`;
- `rust_xlsxwriter` (export XLSX; da verificare, in particolare la modalità a memoria costante);
- libreria date adeguata.

La scelta puntuale delle crate può essere aggiornata senza cambiare l'architettura.

---

# 5. Decisioni non negoziabili

## 5.1 Lo schema è esterno al binario

Le regole soggette a variazione normativa non devono essere hardcoded.

Esempi da tenere nello schema:

- posizione;
- lunghezza;
- dominio;
- obbligatorietà;
- condizioni;
- versione;
- reference data.

Le regole invarianti possono stare nel codice:

```rust
match field_type {
    FieldType::Numeric => validate_numeric(...),
    FieldType::Alphanumeric => validate_alphanumeric(...),
    FieldType::Date => validate_date(...),
}
```

> **Chiarimento (revisione 2026-10-08).** «Esterno al binario» significa esterno al *codice compilato*, non legato al filesystem di una macchina specifica. Schemi e reference data sono un **bundle di dati** con ciclo di vita proprio, distribuito separatamente dall'eseguibile; l'eseguibile contiene solo uno snapshot di default e una cartella esterna ha sempre la precedenza (§39C.2). Non è ammesso alcun path assoluto o compile-time della macchina di sviluppo.

## 5.2 Lo schema è versionato

La chiave minima è:

```text
flow_id + schema_version + trace_id
```

Non usare implicitamente l'ultima versione disponibile.

## 5.3 Ordine e lookup sono esigenze diverse

Non usare una `HashMap` come struttura primaria di scansione.

Preferire:

```rust
struct RecordSchema {
    fields: Vec<FieldSpec>,
    field_index: HashMap<FieldId, usize>,
}
```

- `Vec`: ordine fisico;
- `HashMap`: lookup rapido.

## 5.4 Niente tuple opache

Preferire tipi nominati:

```rust
struct FieldSpec {
    id: FieldId,
    name: String,
    start: usize,
    length: usize,
    field_type: FieldType,
    presence: Vec<PresenceRule>,
    constraints: Vec<Constraint>,
}
```

## 5.5 Niente flag booleani per le condizioni normative

Non implementare:

```text
leggo Y
→ x_required = true
→ quando arrivo a X controllo il flag
```

Le regole devono essere valutate rispetto a un contesto già disponibile.

La logica non deve dipendere dall'ordine di scansione.

## 5.6 Niente `deepcopy` della riga

Usare buffer e slice:

```text
BufReader
   ↓
Vec<u8>
   ↓
&[u8]
   ↓
slice dei campi
```

## 5.7 Niente interpretazione automatica non revisionata delle note

Le note normative in linguaggio naturale possono essere importate come metadata, ma non devono diventare regole eseguibili senza normalizzazione e revisione.

## 5.8 Il binario non dipende dalla macchina di sviluppo

L'ambiente di sviluppo (CachyOS / Arch Linux) è un dettaglio di chi sviluppa, non del prodotto.

L'eseguibile compilato non deve richiedere a runtime librerie, file, utility, variabili d'ambiente, locale, timezone o directory presenti sulla macchina su cui è stato compilato. Un utente Windows deve poter estrarre l'archivio di release in una cartella qualsiasi ed eseguirlo, senza toolchain Rust, senza runtime aggiuntivi e senza file del PC Linux.

Le verifiche concrete sono in §39C.

---

# 6. Architettura logica

```text
                    ┌──────────────────────────┐
                    │        CLI / UI          │
                    │ flow / trace / version   │
                    │ input / output           │
                    └────────────┬─────────────┘
                                 │
                                 ▼
                    ┌──────────────────────────┐
                    │     Schema Registry      │
                    └────────────┬─────────────┘
                                 │
                                 ▼
                    ┌──────────────────────────┐
                    │      Schema Linter       │
                    └────────────┬─────────────┘
                                 │
                                 ▼
                    ┌──────────────────────────┐
                    │       File Reader        │
                    │       buffered I/O       │
                    └────────────┬─────────────┘
                                 │
                                 ▼
                    ┌──────────────────────────┐
                    │      Record Slicer       │
                    │       byte slices        │
                    └────────────┬─────────────┘
                                 │
                                 ▼
               ┌──────────────────────────────────┐
               │        Validation Engine         │
               │                                  │
               │ Presence                         │
               │ Padding / Alignment              │
               │ Datatype                         │
               │ Domain                           │
               │ Cross-field                      │
               │ Reference data                   │
               │ Cross-record                     │
               └──────────────┬───────────────────┘
                              │
             ┌────────────────┴────────────────┐
             ▼                                 ▼
     Context Extractor                  Error Collector
             │                                 │
             ▼                                 ▼
     Cross-record store               JSONL / CSV / human
             │
             ▼
     Flow-level validator
```

---

# 7. Modello dati dello schema

## 7.1 FlowSchema

```rust
struct FlowSchema {
    flow_id: FlowId,
    version: SchemaVersion,
    traces: HashMap<TraceId, RecordSchema>,
    encoding: EncodingSpec,
    reference_data: Vec<ReferenceDataDependency>,
}
```

## 7.2 RecordSchema

```rust
struct RecordSchema {
    id: TraceId,
    name: String,
    record_length: usize,
    fields: Vec<FieldSpec>,
    field_index: HashMap<FieldId, usize>,
    cross_field_rules: Vec<Rule>,
    cross_record_rules: Vec<Rule>,
}
```

## 7.3 FieldSpec

```rust
struct FieldSpec {
    id: FieldId,
    name: String,
    start: usize,
    length: usize,
    field_type: FieldType,
    presence: Vec<PresenceRule>,
    constraints: Vec<Constraint>,
    regulatory_metadata: Option<RegulatoryMetadata>,
}
```

Le posizioni possono essere memorizzate one-based nello schema e normalizzate internamente a zero-based. La conversione deve essere unica, esplicita e testata.

---

# 8. Tipi fisici

Esempio:

```rust
enum FieldType {
    Numeric,
    Alphabetic,
    Alphanumeric,
    Date(DateFormat),
    Decimal(DecimalSpec),
}
```

Gli alias normativi vanno normalizzati durante l'import.

Esempio:

```text
A     → Alphabetic
AN    → Alphanumeric
N     → Numeric
D     → Date
Data  → Date
```

Un tipo non riconosciuto è un **SchemaError**.

---

# 9. Presenza e obbligatorietà

Non usare `required: bool`.

Modello raccomandato:

```rust
enum PresenceRule {
    Required,
    Optional,
    RequiredIf(Expr),
    RequiredUnless(Expr),
    ForbiddenIf(Expr),
}
```

Un campo può avere più regole contemporaneamente.

Esempio:

```text
required_if A
forbidden_if B
```

deve essere rappresentabile senza perdita di informazione.

---

# 10. Separare tipo fisico e significato normativo

Un campo può essere fisicamente `AN`, ma semanticamente ammettere solo alcuni valori.

Separare:

```text
Physical representation
───────────────────────
Numeric
Alphabetic
Alphanumeric
Date
Decimal
```

da:

```text
Semantic constraints
────────────────────
AllowedValues
Range
Regex
ReferenceData
CrossField
CrossRecord
```

Esempio:

```rust
enum Constraint {
    AllowedValues(Vec<String>),
    Range(RangeSpec),
    Regex(String),
    ReferenceData(ReferenceDataRule),
    MustEqual(String),
    CustomBuiltin(BuiltinValidatorId),
}
```

---

# 11. Rule DSL

Le condizioni devono essere modellate come un AST controllato.

Non usare:

- `eval()`;
- scripting arbitrario;
- esecuzione dinamica di codice.

Esempio:

```rust
enum Expr {
    Eq(FieldRef, Value),
    Ne(FieldRef, Value),
    In(FieldRef, Vec<Value>),
    NotIn(FieldRef, Vec<Value>),
    And(Vec<Expr>),
    Or(Vec<Expr>),
    Not(Box<Expr>),
}
```

`FieldRef` deve poter puntare a campi locali o ad altri tracciati:

```rust
enum FieldRef {
    Local(FieldId),
    Trace {
        trace_id: TraceId,
        field_id: FieldId,
    },
}
```

Esempio schema:

```json
{
  "field": "2.29",
  "presence": [
    {
      "required_if": {
        "not_in": {
          "field": "2.2",
          "values": ["01", "02"]
        }
      }
    }
  ]
}
```

Esempio cross-tracciato:

```json
{
  "when": {
    "eq": {
      "trace": "2",
      "field": "2.7",
      "value": "8"
    }
  },
  "then": {
    "trace": "3",
    "field": "3.20",
    "must_equal": "99"
  }
}
```

La sintassi finale può cambiare; il modello concettuale no.

---

# 12. Schema Registry

Struttura raccomandata:

```text
schemas/
└── sioc/
    ├── 2025-12-16/
    │   ├── flow.json
    │   ├── trace-1.json
    │   ├── trace-2.json
    │   ├── trace-3.json
    │   ├── trace-4.1.json
    │   └── trace-4.2.json
    └── <future-version>/
        └── ...
```

Il runtime deve ricevere esplicitamente la versione.

---

# 13. Import dei tracciati regionali

Il file Excel regionale è una fonte normativa, non il formato runtime ideale.

Pipeline:

```text
XLS/XLSX regionale
        │
        ▼
Schema Importer
        │
        ▼
Canonical Schema
        │
        ▼
Schema Linter
        │
        ▼
Validated Schema
```

L'importer può estrarre:

- ID;
- descrizione;
- posizione;
- lunghezza;
- tipo;
- obbligatorietà;
- note originali.

Le note naturali devono essere conservabili per audit, ma trasformate manualmente/esplicitamente in DSL quando descrivono regole complesse.

---

# 14. Schema Linter

Il validatore deve rifiutare per default uno schema che non supera il lint.

Controlli minimi:

## Struttura fisica

- `start > 0` nella fonte normativa;
- `length > 0`;
- nessun overlap;
- nessun gap non dichiarato;
- ordine coerente;
- `record_length` coerente.

## Tipi

- tipo riconosciuto;
- alias normalizzabile;
- parametri validi.

## Riferimenti

- ogni `FieldRef` esiste;
- ogni `TraceId` esiste;
- nessuna regola punta a campi inesistenti.

## Vincoli

- domini compatibili con la lunghezza;
- range coerenti;
- formati data riconosciuti;
- reference dataset disponibile.

## Regole

- AST valido;
- contraddizioni evidenti rilevate quando possibile.

Distinguere sempre:

```text
SchemaError
```

da:

```text
ValidationError
```

---

# 15. Elaborazione byte-oriented

La rappresentazione primaria del record deve essere:

```rust
&[u8]
```

non necessariamente `String`.

Motivazione: un tracciato posizionale può definire posizioni in byte; UTF-8 può introdurre caratteri multibyte.

L'encoding è parte della configurazione.

Esempio:

```rust
struct RecordView<'a> {
    raw: &'a [u8],
}
```

Interfaccia:

```rust
impl<'a> RecordView<'a> {
    fn slice(&self, start: usize, len: usize) -> &'a [u8] {
        &self.raw[start..start + len]
    }
}
```

Evitare copie e allocazioni non necessarie.

---

# 16. Streaming

Il file deve essere letto tramite buffered I/O.

Non caricare l'intero `.txt` in memoria salvo requisito esplicito.

Obiettivo:

```text
memoria ≈ buffer riga + context store necessario
```

non:

```text
memoria ≈ intero file
```

---

# 17. Pipeline di validazione del record

Ordine raccomandato:

```text
1. RECORD LENGTH
2. FIELD SLICING
3. PRESENCE
4. PADDING / ALIGNMENT
5. LEXICAL DATATYPE
6. SEMANTIC DOMAIN
7. CROSS-FIELD RULES
8. REFERENCE DATA
9. CONTEXT EXTRACTION
10. CROSS-RECORD / FLOW RULES
```

L'ordine deve ridurre gli errori derivati.

Esempio: se la riga è più corta del previsto, non fare slicing fuori range.

---

# 18. Padding / Alignment

Usare il termine tecnico **padding/alignment validation**.

Esempi generali:

```text
ALPHANUMERIC
ABC_____
padding a destra con spazi
```

```text
NUMERIC
00000123
padding a sinistra con zero
```

Le regole reali devono provenire dallo schema o da builtin generici e documentati.

---

# 19. Numerici e valore zero

Non assumere universalmente:

```text
000000 = campo vuoto
```

Zero può essere un valore semanticamente valido.

Lo schema deve distinguere:

- valore zero;
- assenza;
- placeholder tecnico.

Non implementare una regola globale del tipo:

```rust
is_empty_numeric = all_zeros
```

---

# 20. Validazione cross-field

Una regola può dipendere da campi precedenti o successivi.

Esempio:

```text
X obbligatorio se:
(Y = 1 OR Z = 4)
AND W != 00
```

La condizione deve essere valutata direttamente contro il `RecordView`/context, non tramite flag temporanei.

---

# 21. Cross-record e cross-tracciato

Il motore deve poter utilizzare informazioni provenienti da altri record o tracciati.

Architettura preferita:

```text
PASS 1
────────────────────────
record
  ├── local validation
  └── minimal context extraction
                │
                ▼
         Context Index


PASS 2
────────────────────────
record/context
      │
      ▼
cross-record validation
```

Il `Context Index` conserva soltanto i campi richiesti dalle dipendenze.

Non conservare automaticamente tutti i record completi.

---

# 22. Identità e correlazione dei record

I dataset trattati dal progetto sono già anonimizzati.

Il report diagnostico può quindi mantenere, quando disponibili:

- Codice Fiscale;
- ID record SIOC;
- ID scheda/episodio;
- altri identificativi tecnici utili alla correlazione.

Modello:

```rust
struct RecordIdentity {
    sioc_record_id: Option<String>,
    fiscal_code: Option<String>,
    episode_id: Option<String>,
}
```

La strategia per estrarre tali valori appartiene allo schema/configurazione del flusso.

Il core non deve conoscere la posizione del Codice Fiscale SIOC.

---

# 23. Reference Data Registry

Predisporre un componente separato per:

- Regioni;
- ULSS;
- Comuni ISTAT;
- STS11;
- strutture;
- classificazioni;
- altre tabelle normative.

Esempio:

```rust
Constraint::ReferenceData {
    dataset: "istat-comuni",
    version: "...",
}
```

Le reference data devono essere versionabili e aggiornabili indipendentemente dal binario.

---

# 24. Error Model

Il report degli errori è una feature primaria.

Ogni errore deve consentire a un analista di capire immediatamente:

1. quale record;
2. quale campo;
3. dove si trova;
4. cosa è stato letto;
5. cosa era atteso;
6. quale regola è stata violata;
7. quale condizione ha attivato la regola.

Modello concettuale:

```rust
struct ValidationError {
    code: ErrorCode,

    flow_id: FlowId,
    schema_version: SchemaVersion,
    trace_id: TraceId,

    line_number: u64,
    record_identity: RecordIdentity,

    field_id: Option<FieldId>,
    field_name: Option<String>,

    start: Option<usize>,
    end: Option<usize>,

    actual_value: Option<String>,
    expected: Option<String>,

    context: Vec<DiagnosticContext>,
    message: String,
}
```

Ogni errore di campo dovrebbe contenere almeno:

```text
flusso
versione schema
tracciato
numero riga
ID record SIOC
Codice Fiscale se disponibile
field ID
field name
start/end
actual
expected
reason
```

---

# 25. Esempio di errore JSON

```json
{
  "code": "E_CONDITIONAL_REQUIRED",
  "flow": "SIOC",
  "schema_version": "2025-12-16",
  "trace": "2",
  "line": 184,

  "record_identity": {
    "record_key_digest": "REDACTED-IN-EXAMPLE"
  },

  "field": {
    "id": "2.29",
    "name": "Codice Regione struttura di provenienza",
    "start": 118,
    "end": 120
  },

  "actual": "   ",
  "expected": "codice Regione di 3 caratteri",

  "context": {
    "2.2": "05"
  },

  "message": "Il campo 2.29 è obbligatorio perché il campo 2.2 Provenienza del Paziente vale '05', valore diverso da '01' e '02'. Il campo risulta non valorizzato."
}
```

---

# 26. Codici errore

I codici **serializzati pubblicamente** sono stringhe simboliche stabili `E_*`, mai numerazioni incrementali implicite. I vecchi esempi `E001..E013` sono **obsoleti e non vincolanti**.

Vocabolario iniziale da P2/T-P2-02:

```text
E_RECORD_LENGTH
E_REQUIRED_FIELD
E_CONDITIONAL_REQUIRED
E_FORBIDDEN_FIELD
E_ALIGNMENT
E_INVALID_TYPE
E_INVALID_DOMAIN
E_OUT_OF_RANGE
E_INVALID_DATE
E_CROSS_FIELD
E_REFERENCE_DATA
E_CROSS_RECORD
E_DUPLICATE_RECORD_KEY
E_RULE_CONFLICT
E_RELATED_RECORD_MISSING
```

I codici CAD specializzati (`E_CAD_*`) provengono dal profilo CAD. Un problema di compilazione dello schema è un errore di preflight/runtime (`SchemaError`), non un record normativo con `E_SCHEMA_ERROR`. `ValidationCode` e mapping risiedono in un'unica tabella/versione; i vecchi codici numerici non devono essere serializzati in report.

---

# 27. Validation report vs application log

Sono due canali distinti.

## Application log

Per sviluppatori/operatori:

```text
schema loaded
file opened
records processed
elapsed time
I/O error
```

## Validation report

Per chi corregge il dato:

```text
record 184
field 2.29
conditional required violated
```

Non mescolare i due.

---

# 28. Formati output

Formato primario consigliato:

> **JSON Lines (`.jsonl`)**

Vantaggi:

- streaming;
- un errore per riga;
- facile parsing;
- adatto ad analisi automatiche.

Formati aggiuntivi:

- XLSX (formato principale per la lettura da parte degli analisti, §39C.3);
- CSV;
- testo human-readable;
- summary JSON;
- futura visualizzazione UI.

Tutti devono derivare dallo stesso modello `ValidationError`.

---

# 29. Summary di esecuzione

A fine processo produrre:

```text
flow
trace
schema version
input
records processed
valid records
invalid records
total errors
errors by code
elapsed time
result
```

Esempio:

```text
Result: INVALID

Records processed: 125430
Invalid records: 318
Errors: 411

E003 CONDITIONAL_REQUIRED: 180
E007 INVALID_DOMAIN:       122
E005 ALIGNMENT:             73
E009 INVALID_DATE:          36
```

Se non ci sono errori:

```text
Result: VALID
No validation errors found.
```

---

# 30. CLI

Interfaccia concettuale:

```bash
lintra validate \
  --flow SIOC \
  --trace 2 \
  --schema-version 2025-12-16 \
  --input ./input.txt \
  --output ./errors.jsonl
```

Altri comandi:

```bash
lintra schema lint ...
lintra schema import ...
lintra schema list ...
lintra reference-data list ...
```

La CLI è un adapter.

Il core non deve dipendere da `clap`.

---

# 31. Struttura iniziale del progetto

```text
src/
├── main.rs
├── cli.rs
│
├── schema/
│   ├── mod.rs
│   ├── model.rs
│   ├── loader.rs
│   ├── importer.rs
│   ├── linter.rs
│   └── expression.rs
│
├── engine/
│   ├── mod.rs
│   ├── reader.rs
│   ├── record.rs
│   ├── validator.rs
│   └── context.rs
│
├── validators/
│   ├── mod.rs
│   ├── presence.rs
│   ├── padding.rs
│   ├── datatype.rs
│   ├── domain.rs
│   ├── range.rs
│   ├── date.rs
│   ├── lookup.rs
│   ├── cross_field.rs
│   └── cross_record.rs
│
├── report/
│   ├── mod.rs
│   ├── error.rs
│   ├── jsonl.rs
│   ├── csv.rs
│   └── human.rs
│
└── reference_data/
    ├── mod.rs
    └── registry.rs
```

Iniziare con il package Rust `lintra` e una libreria pubblica stabile. È ammessa la migrazione a workspace con eseguibile Slint separato nella fase P16: prima non anticiparla senza una necessità verificata.

Valutare un workspace multi-crate solo quando emergono confini reali tra:

- core library;
- CLI;
- schema tooling;
- UI.

---

# 32. API interna desiderata

Obiettivo:

```rust
validate_file(
    input,
    flow_id,
    schema_version,
    trace_id,
    options,
) -> ValidationReport
```

CLI, futura UI e possibili API REST devono essere adapter sopra il core.

---

# 33. Performance

Non ottimizzare prematuramente.

Priorità:

1. correttezza;
2. testabilità;
3. auditabilità;
4. benchmark;
5. ottimizzazione.

Tecniche già coerenti con il design:

- buffered I/O;
- zero-copy dove ragionevole;
- riuso del buffer;
- evitare allocazioni per campo;
- reference data indicizzate;
- context store minimale.

Parallelizzare solo dopo aver distinto:

```text
validazioni indipendenti
```

da:

```text
validazioni che richiedono contesto globale o ordine
```

---

# 34. Testing

## 34.1 Unit test

Per:

- Numeric;
- Alphabetic;
- Alphanumeric;
- Date;
- Decimal;
- Padding;
- Enum;
- Range;
- Presence.

## 34.2 Test DSL

Per:

- Eq;
- Ne;
- In;
- NotIn;
- And;
- Or;
- Not.

Testare sempre true e false.

## 34.3 Schema linter

Testare:

- gap;
- overlap;
- tipo sconosciuto;
- field reference inesistente;
- trace inesistente;
- lunghezza incoerente;
- vincoli incompatibili.

## 34.4 Golden tests

Struttura:

```text
tests/fixtures/sioc/
├── valid/
├── invalid-required/
├── invalid-padding/
├── invalid-domain/
└── cross-record/
```

Confrontare il report ottenuto con output atteso versionato.

## 34.5 Regression

Regola:

> Ogni bug di validazione corretto deve aggiungere un test che lo riproduca.

---

# 35. Auditabilità

Ogni esecuzione deve rendere ricostruibile:

- versione software;
- versione schema;
- hash schema;
- reference data e versioni;
- input;
- opzioni;
- timestamp.

Metadata suggeriti:

```text
validator_version
schema_version
schema_hash
reference_data_versions
execution_timestamp
```

---

# 36. Regulatory metadata

Le note originali possono essere mantenute:

```rust
struct RegulatoryMetadata {
    source_note: Option<String>,
    source_document: Option<String>,
}
```

Il motore non esegue il linguaggio naturale.

Il testo sorgente e la regola eseguibile sono due cose distinte.

---

# 37. Anti-pattern vietati

Un agente non deve introdurre senza una nuova decisione architetturale esplicita:

### Hardcoding di flusso

```rust
if flow == "SIOC" { ... }
```

### Hardcoding di field ID nel core

```rust
if field_id == "2.29" { ... }
```

### Flag normativi mutabili

```rust
x_required = true;
```

### Parsing esclusivamente `String`

quando lo schema ragiona per byte.

### Caricamento dell'intero file

quando lo streaming è sufficiente.

### HashMap come ordine del tracciato

L'ordine appartiene a `Vec<FieldSpec>`.

### NLP/LLM automatico sulle note in produzione

Una nota non revisionata non diventa una regola normativa eseguibile.

### Ultima versione implicita

La versione deve essere selezionata e tracciata.

### Validator monolitico

I controlli devono rimanere modulari.

---

# 38. Roadmap

## Fase 1 — Core minimale

Implementare:

- `FlowSchema`;
- `RecordSchema`;
- `FieldSpec`;
- loader JSON;
- schema linter base;
- file reader;
- `RecordView`;
- record length;
- slicing;
- required;
- datatype;
- padding;
- JSONL report.

**Target:** un tracciato con regole locali.

## Fase 2 — Vincoli semantici

Aggiungere:

- enum/domain;
- range;
- date;
- decimal;
- reference data;
- summary;
- export XLSX.

## Fase 3 — Rule DSL

Aggiungere:

- `Expr`;
- `FieldRef`;
- `RequiredIf`;
- `ForbiddenIf`;
- cross-field.

**Target:** rappresentare le regole senza hardcoding.

## Fase 4 — Cross-record

Aggiungere:

- `RecordIdentity`;
- Context Index;
- correlazione;
- cross-trace;
- secondo passaggio o strategia equivalente.

## Fase 5 — Schema tooling

Aggiungere:

- XLSX importer;
- normalizzazione;
- lint avanzato;
- canonical schema;
- diff tra versioni normative.

## Fase 6 — Production hardening

Aggiungere:

- benchmark;
- profiling;
- file grandi;
- audit metadata;
- packaging;
- gestione robusta I/O;
- documentazione operativa.

---

# 39. Obiettivo futuro: UI in Rust

Dopo la stabilizzazione del core e della CLI, predisporre una **UI desktop in Rust** per avviare e controllare il processo di analisi.

La UI deve essere un client del core.

Non deve duplicare logica normativa.

Funzioni desiderate:

- selezione file `.txt`;
- selezione flusso;
- selezione tracciato;
- selezione versione;
- avvio analisi;
- cancel/stop dove sicuro;
- avanzamento;
- record elaborati;
- errori trovati;
- summary;
- tabella errori;
- filtri per:
  - codice errore;
  - field ID;
  - record SIOC;
  - Codice Fiscale;
  - numero riga;
- dettaglio errore;
- export JSONL/CSV.

Architettura:

```text
              ┌─────────────┐
              │   Rust UI   │
              └──────┬──────┘
                     │
                     ▼
              ┌─────────────┐
              │ Core API    │
              └──────┬──────┘
                     │
                     ▼
              Validation Engine
```

**Decisione (2026-10-09): il framework GUI è Slint.** Vincoli e criteri di accettazione in §39D. Il piano B, se il prototipo di §39D.6 fallisce, è `egui`.

---


# 39A. Requisito non funzionale: supporto multipiattaforma

Il software deve supportare ufficialmente, già dall'MVP:

- **Linux x86_64**;
- **Windows x86_64** (Windows 10 o successivo; verificare il minimo effettivo sulla platform-support page di `rustc` per il toolchain pinnato).

Lo sviluppo avviene principalmente su **CachyOS (Arch Linux)**, ma questo non rende Linux la piattaforma di riferimento: il codice del core, della CLI e della futura UI deve rimanere platform-agnostic e Windows è una piattaforma di consegna di pari dignità.

Vincoli di implementazione:

- usare `std::path::Path` e `PathBuf` per tutti i path;
- non concatenare manualmente separatori `/` o `\`;
- gestire correttamente terminatori di riga `\n` e `\r\n`, separandoli dalla lunghezza logica del record;
- non hardcodare path assoluti o directory specifiche della macchina di sviluppo;
- non usare a runtime path derivati da costanti compile-time dell'host (`env!("CARGO_MANIFEST_DIR")`, `file!()`, `concat!(env!(...))` e simili): sono ammessi solo in test e build script;
- non dipendere a runtime da utility Unix come `grep`, `sed`, `awk`, `bash` o equivalenti, salvo tooling di sviluppo non necessario all'esecuzione del prodotto;
- non usare API specifiche di un sistema operativo (`std::os::unix`, `std::os::windows`) fuori da moduli isolati e protetti da `#[cfg(...)]`, con equivalente per l'altro sistema;
- non dipendere da locale, timezone locale o code page del sistema: timestamp sempre UTC (RFC 3339), output sempre con encoding esplicito (vedi §39C.3);
- non esporre in report o audit path canonicalizzati (su Windows `canonicalize()` produce prefissi `\\?\`): registrare nome file, dimensione e hash dell'input;
- mantenere schema e reference data caricabili tramite path/configurazione portabile;
- produrre artefatti nativi distinti per sistema operativo.

La CI deve verificare almeno:

```text
Linux runner (CI)
  cargo fmt --check
  cargo clippy --locked -- -D warnings
  cargo test --locked
  cargo build --release --locked

Windows runner (CI)
  cargo fmt --check
  cargo clippy --locked -- -D warnings
  cargo test --locked
  cargo build --release --locked
```

I runner CI sono indipendenti dal sistema usato per sviluppare: il fatto che lo sviluppo avvenga su Arch non modifica la matrice.

La **cross-compilazione da Arch verso Windows** (§39B) è uno strumento del ciclo di sviluppo per intercettare presto errori di compilazione: **non sostituisce** il runner Windows reale, che resta il gate di merge e la fonte degli artefatti Windows di release.

Le release devono poter distribuire almeno:

```text
Release dell'eseguibile
  lintra-<version>-linux-x86_64.tar.gz
  lintra-<version>-windows-x86_64.zip
  SHA256SUMS

Release dei dati (ciclo di vita separato)
  schemas-<bundle_version>.zip
  SHA256SUMS
```

Gli archivi dell'eseguibile contengono il binario nativo con lo snapshot incorporato di default; l'archivio dei dati contiene schemi e reference data aggiornabili senza ricompilare (§39C.2).

Gli artefatti di release sono prodotti **solo dalla CI**, mai dal laptop di sviluppo (motivi in §39C.4).

Il supporto Windows non è una futura estensione: è un **requisito non funzionale dell'MVP**.

---

# 39B. Ambiente di sviluppo: CachyOS / Arch Linux

Questa sezione riguarda chi sviluppa; nulla di ciò che segue deve diventare un requisito runtime del prodotto.

## 39B.1 Toolchain

CachyOS è una distribuzione rolling: i pacchetti di sistema cambiano spesso. Per ottenere build riproducibili:

- installare Rust tramite `rustup` (non tramite il pacchetto `rust` di sistema, che non gestisce comodamente più target e versioni);
- fissare la versione con `rust-toolchain.toml` nella root del repository, con `profile = "minimal"`, `components = ["rustfmt", "clippy"]` e `targets` che includano il target Windows usato;
- committare `Cargo.lock` e usare `--locked` in locale e in CI.

## 39B.2 Compilare per Windows da Arch

Opzione consigliata per il ciclo di sviluppo (`x86_64-pc-windows-gnu`):

```text
pacman -S rustup mingw-w64-gcc wine
rustup target add x86_64-pc-windows-gnu
```

`.cargo/config.toml` (solo sviluppo):

```toml
[target.x86_64-pc-windows-gnu]
linker = "x86_64-w64-mingw32-gcc"
runner = "wine"
```

Comandi:

```text
cargo check --target x86_64-pc-windows-gnu          # veloce, intercetta cfg/API Linux-only
cargo build --release --target x86_64-pc-windows-gnu
cargo test  --target x86_64-pc-windows-gnu          # esegue i test sotto Wine
```

Alternativa per ottenere un binario con ABI MSVC (quella usata dalla maggior parte dell'ecosistema Windows): `cargo-xwin` (`cargo install --locked cargo-xwin`, poi `cargo xwin build --release --target x86_64-pc-windows-msvc`). Scarica CRT e SDK Microsoft e quindi comporta condizioni di licenza da valutare. **Non adottato in questo progetto**, perché la build msvc di release avviene sul runner Windows.

**Decisione (2026-10-08):** la release Windows è **`msvc`**, costruita sul runner Windows ospitato da GitHub (nessuna licenza Windows da acquistare: la macchina di build è del provider CI). L'eseguibile usa la CRT statica e non richiede il Visual C++ Redistributable sui PC degli utenti. Il target `gnu` da Arch serve solo al ciclo di sviluppo. Nota sui costi: su repository privati i minuti dei runner Windows contano il doppio rispetto a Linux; su repository pubblici i runner standard sono gratuiti. Se in futuro la build msvc dovesse risultare non praticabile, il ripiego è `gnu` con la stessa verifica delle DLL (§39C.1).

Wine non è Windows: i test sotto Wine sono un controllo rapido, non un gate.

## 39B.3 Tooling di sviluppo

Script Linux-only (`bash`, `Makefile`, `justfile`) sono ammessi solo per comodità locale. Tutto ciò che serve alla CI o alla release deve essere eseguibile su entrambi i runner: comandi `cargo` diretti oppure un crate `xtask` in Rust.

---

# 39C. Binario autosufficiente e distribuzione

## 39C.1 Cosa significa «autosufficiente»

L'eseguibile non richiede a runtime nulla che esista solo sulla macchina di build. È verificato automaticamente (job CI o `cargo xtask verify-artifact`):

```text
Windows (.exe)
  - le DLL importate sono solo DLL di sistema Windows
    (KERNEL32, ntdll, api-ms-win-*, ecc.)
  - nessuna libgcc_s_*, libwinpthread-*, VCRUNTIME*, MSVCP*
  - target msvc: linking statico della CRT
    (rustflags: -C target-feature=+crt-static)

Linux
  - eseguibile CLI: dipendenze dinamiche limitate alla libc di base (oppure binario musl statico)
  - eseguibile UI: vedi §39D.2

Entrambi
  - nessun path dell'host nel binario
    (RUSTFLAGS con --remap-path-prefix per $PWD e $CARGO_HOME;
     controllo con `strings` per /home/ e simili)
```

Politica sulle dipendenze:

- preferire crate pure-Rust (`serde`, `regex`, `encoding_rs`, `calamine` sono già in questa categoria);
- vietate dipendenze che richiedono librerie di sistema (`openssl`, `libsqlite3-sys`, `libz-sys`, ecc.) salvo feature `vendored`/`static` e decisione esplicita; se in futuro servisse TLS (REST API), usare `rustls`;
- `build.rs` non deve invocare comandi di sistema;
- controllo periodico: `cargo tree -i openssl-sys` deve risultare vuoto.

## 39C.2 Schemi e reference data: ciclo di vita separato e modalità ibrida

Resta valida la decisione §5.1: schemi e reference data sono versionati e aggiornabili senza ricompilare (§23).

**Decisione (2026-10-08): approccio ibrido.**

- L'eseguibile incorpora uno **snapshot di default** di schemi e reference data (feature `embedded-bundle`, attiva nelle release).
- Una cartella esterna, se presente, ha sempre la precedenza.
- I dati hanno un ciclo di vita proprio: un cambio di tracciato (almeno annuale) produce una nuova release del *bundle di dati*, non dell'eseguibile.

Regole:

1. **Fallback a livello di bundle, non di singolo file.** Se la cartella esterna è presente viene usata solo quella, senza mescolarla con lo snapshot. Se la cartella esiste ma non contiene lo schema o la versione richiesti, o è incompleta (es. `schemas/` senza `reference-data/`), l'esito è un errore esplicito: nessun ripiego silenzioso sullo snapshot (coerente con §5.2).
2. **Tracciabilità.** Report e audit indicano sempre l'origine dei dati (`external` oppure `embedded`), `bundle_version`, `schema_hash` e le versioni dei dataset (§35).
3. **Compatibilità motore/dati.** Ogni schema dichiara un `schema_format_version`; l'eseguibile dichiara i formati che supporta. Formato non supportato: errore di esecuzione che indica la versione minima dell'eseguibile richiesta. Finché il formato non cambia, gli schemi evolvono senza toccare il motore.
4. **Lo snapshot incorporato non è una fonte di aggiornamento.** Si rinnova solo alle release dell'eseguibile.

Risoluzione delle directory (responsabilità degli adapter CLI/UI; il core riceve sempre `PathBuf` espliciti oppure repository già costruiti), in ordine:

1. opzioni esplicite `--schema-dir` e `--reference-data-dir`;
2. variabili d'ambiente `LINTRA_SCHEMA_DIR` e `LINTRA_REFERENCE_DATA_DIR`;
3. directory `schemas/` e `reference-data/` accanto all'eseguibile (`std::env::current_exe()`, parent);
4. se nessuna delle precedenti è presente: snapshot incorporato.

Un'opzione o una variabile che punta a una directory inesistente è un errore, non un motivo per scendere al livello successivo. La working directory non è mai un default (un doppio click su Windows ha una cwd imprevedibile) e non si usa mai un path compile-time.

Layout dell'archivio dei dati:

```text
schemas-<bundle_version>.zip
├── schemas/
├── reference-data/
├── MANIFEST.json     (bundle_version, schema_format_version, hash dei file)
└── SHA256SUMS
```

## 39C.3 Comportamenti identici su Linux e Windows

- **JSONL**: UTF-8 senza BOM, terminatore di riga `\n` su entrambi i sistemi; a parità di input e schema l'output deve essere **identico byte per byte**.
- **Report su file**: documentare e privilegiare `--output <file>`. In Windows PowerShell 5.1 l'operatore `>` scrive in UTF-16, quindi un report reindirizzato con `>` può risultare illeggibile ad altri strumenti.
- **XLSX** (**decisione 2026-10-08**): formato principale per la lettura degli errori da parte di analisti umani, prioritario. Generato a fine analisi a partire dallo stesso flusso di `ValidationError` (non dentro il ciclo di validazione). Fogli `Riepilogo` ed `Errori`; tutte le celle di dato scritte come **testo**, per non perdere zeri iniziali (`00123`, `05`); intestazione fissa e filtri sulle colonne principali (codice errore, field ID, ID record, Codice Fiscale, riga). Oltre il limite di righe di un foglio Excel (1.048.576, intestazione inclusa) il report continua su fogli successivi.
- **CSV**: formato secondario per elaborazioni automatiche, solo RFC 4180 (UTF-8 senza BOM, `,`, CRLF). Nessun profilo specifico per Excel: per la lettura in Excel si usa l'XLSX.
- **Timestamp** di esecuzione sempre in UTC, formato RFC 3339.
- **Audit dell'input**: nome file, dimensione e SHA-256, non il path assoluto.
- **Interruzione (Ctrl+C)**: usare un crate multipiattaforma (es. `ctrlc`) per impostare il `CancellationToken`; il codice di uscita 130 è un contratto applicativo valido su entrambi i sistemi.
- **File prodotti su Windows**: un file modificato con Blocco note può avere BOM e CRLF; il comportamento è definito da `BomPolicy` e `NewlinePolicy` (documento di dettaglio) e l'errore deve spiegare chiaramente la causa.

## 39C.4 Release engineering

- **Windows**: artefatto costruito sul runner Windows (fonte autorevole). **Decisione (2026-10-08): nessuna firma Authenticode**, perché il software è usato solo dall'autore e da colleghi tecnici in locale. Al primo avvio Windows può mostrare l'avviso «app non riconosciuta»: l'utente sceglie «Ulteriori informazioni → Esegui comunque» (oppure sblocca lo zip dalle Proprietà del file). `SHA256SUMS` viene generato automaticamente dalla CI e la sua consultazione è facoltativa. Se in futuro il software uscisse da questo perimetro, la firma andrebbe rivalutata.
- **Linux**: l'artefatto non va costruito sul laptop Arch. Arch ha una glibc molto recente e un binario costruito lì può non partire su distribuzioni più vecchie. Costruirlo in CI su un runner con glibc di riferimento oppure con target `x86_64-unknown-linux-musl` (statico).
- **Profilo release** consigliato: `lto = "thin"`, `codegen-units = 1`, `strip = true`, da validare con i benchmark del §33.
- **Naming** versione e commit nell'output di `lintra --version`.

## 39C.5 Distribuzione su GitHub

CI su GitHub Actions. Tre ambiti separati, ciascuno con il proprio ciclo di vita:

```text
repo sorgente    codice del motore
repo binari      solo Release dell'eseguibile (una release per versione)
repo dati        schemas/ + reference-data/ (una release per bundle_version)
```

Regole:

- le release pubblicate sono **immutabili**: un aggiornamento è una nuova versione, non la sovrascrittura degli asset di una release esistente (trasparenza e auditabilità); una ripubblicazione dello stesso contenuto, se necessaria, è una nuova versione con nota esplicativa;
- la CI del repo sorgente pubblica nel repo binari; serve un token con permesso di scrittura su quel repo;
- su repository privati i minuti Windows consumano la quota al doppio del ritmo di Linux: tenere il job Windows di release breve e separato dai job di sviluppo;
- prima di rendere pubblico il repo dati, verificare che i tracciati regionali e le tabelle di codifica siano ridistribuibili; in caso di dubbio, repo dati privato.

---

# 39D. Interfaccia desktop: vincoli (Slint)

## 39D.1 Decisione e licenza

Il framework GUI è **Slint** (decisione 2026-10-09). Motivi: layout dichiarativo con misure esplicite in px, renderer proprio (anche software, senza GPU), lista virtualizzata per tabelle grandi, comportamento uguale su Linux e Windows.

Licenza: Slint si può usare in applicazioni desktop proprietarie senza costi con la licenza royalty-free, a condizione di dichiarare l'uso di Slint. Quindi l'applicazione include una schermata «Informazioni» con l'attribuzione (widget `AboutSlint` o equivalente indicato dalla licenza). Senza attribuzione servirebbe la licenza commerciale. Le condizioni vanno riverificate sul testo della licenza della versione adottata; la versione di Slint è fissata in `Cargo.lock`.

## 39D.2 Eseguibile separato e dipendenze

- La UI è un **eseguibile distinto dalla CLI**, in un crate membro di un workspace (la condizione di §31 «confini reali tra core, CLI, schema tooling, UI» è ora soddisfatta). Il core e la CLI non dipendono da Slint né da librerie grafiche.
- Il controllo di autosufficienza di §39C.1 si applica così:
  - Windows: solo DLL di sistema, come per la CLI.
  - Linux: sono ammesse le librerie di sistema necessarie alla finestra (X11 o Wayland, ecc.); il controllo diventa un elenco esplicito di librerie ammesse, non più «solo libc». Nessuna dipendenza dall'ambiente di build.

## 39D.3 Rendering, font, token

- **Renderer di riferimento: software.** Non si richiede la GPU. Un eventuale renderer GPU è un'opzione futura, ammessa solo dopo prove di identità visiva.
- **Font incorporati nel binario** (candidati: IBM Plex Sans e IBM Plex Mono, licenza OFL, da verificare per l'incorporamento). Gli stessi font sono usati nel mockup, che quindi smette di dipendere dai font di sistema.
- **Token di design** (colori, dimensioni dei caratteri, spaziature, raggi, bordi) in un unico file della UI, derivato dal mockup; nessun valore sparso nei componenti.
- Nessun widget standard con stile proprio dove il mockup ne ha uno diverso: i componenti sono costruiti con primitive e token.

## 39D.4 Definizione di «congruente al mockup»

Congruenza esatta, verificabile, su:

- posizione e dimensione di ogni riquadro, bordo e raggio;
- colori;
- font, dimensione e peso dei caratteri.

Il testo può differire dal rendering del browser solo nell'anti-aliasing dei bordi delle lettere, entro una tolleranza dichiarata. Nessuna libreria Rust garantisce l'identità pixel-per-pixel con il rendering di un browser: l'identità pixel-per-pixel è garantita tra Linux e Windows (stesso renderer, stessi font), e tra versioni successive dell'app tramite i riferimenti di §39D.5.

## 39D.5 Test visivi

- Riferimenti: PNG esportati dal mockup a viewport fissa, a scala 1× e 2×.
- Test in CI su entrambi i runner: rendering headless con il renderer software, confronto con tolleranza per zona (quasi nulla su riquadri e bordi, più larga sul testo).
- Dopo la prima approvazione visiva, il PNG prodotto dall'app diventa il riferimento per la regressione.
- Ogni deviazione nota dal mockup (es. barra di scorrimento, interlinea) è elencata e accettata esplicitamente, non nascosta da una soglia più larga.

## 39D.6 Prototipo e criteri di accettazione

Prima di costruire l'intera UI si realizza un prototipo della schermata **Risultati** (la più difficile). Criteri:

1. geometria uguale al mockup entro la tolleranza di §39D.5;
2. scorrimento fluido della tabella con centinaia di migliaia di righe;
3. output dei test visivi identico tra runner Linux e Windows.

Se i criteri non sono soddisfatti si ripiega su `egui`, con la stessa definizione di congruenza.

## 39D.7 Confini architetturali

La UI è un client di `ValidationService` (§32): nessuna logica normativa, nessuna copia del motore. Avanzamento e annullamento passano da `ProgressObserver` e `CancellationToken`. La tabella degli errori legge dal flusso già prodotto (JSONL), senza chiedere al core di accumulare gli errori in memoria.

---

# 40. Evoluzioni compatibili

Non fanno parte dell'MVP ma non devono essere precluse:

- supporto multi-flow;
- diff tra versioni normative;
- report analytics;
- REST API;
- CI/CD validation;
- profili di validazione;
- dashboard UI.

---

# 41. Check architetturale per ogni nuova feature

Prima di implementare una feature chiedere:

1. È generica o specifica di SIOC?
2. Se è specifica, può stare nello schema?
3. Richiede davvero stato mutabile?
4. Può essere espressa nella DSL?
5. Introduce dipendenza dall'ordine dei campi?
6. Rompe lo streaming?
7. Conserva dati non necessari?
8. È testabile?
9. È auditabile?
10. Funzionerebbe per un secondo flusso?

Se la risposta alla decima domanda è no, verificare che non sia stata introdotta logica normativa nel core.

---

# 42. Workflow obbligatorio per agenti AI

Prima di modificare il codice:

1. leggere questo documento;
2. identificare il componente coinvolto;
3. classificare la modifica come:
   - core;
   - schema;
   - reference data;
   - report;
   - CLI/UI;
4. evitare hardcoding;
5. verificare impatto sui test.

Durante l'implementazione:

- mantenere moduli piccoli;
- preferire tipi espliciti;
- evitare logica duplicata;
- documentare decisioni non ovvie;
- evitare dipendenze circolari.

Prima di dichiarare completata una task:

```text
cargo fmt
cargo clippy
cargo test
cargo check --target x86_64-pc-windows-gnu
```

L'ultimo comando serve a intercettare codice dipendente da API Linux-only quando si lavora su Arch; non sostituisce la CI su runner Windows.

e gli integration/golden test pertinenti.

L'agente deve riportare:

- file modificati;
- comportamento introdotto;
- test aggiunti;
- nuove decisioni architetturali.

---

# 43. Definition of Done dell'MVP

Il **gate M3** (DET §151) è soltanto un traguardo *intermedio locale*; il **MVP M5** segue questa checklist completa. Il prototipo UI Slint non è un MVP. I flussi CAD/SIOC possono avere un gate runtime addizionale, compresa la persistenza core, senza rinominare l'MVP.

L'MVP è completato quando:

- viene caricato uno schema versionato esterno;
- lo schema supera il lint;
- il file fixed-width è letto in streaming;
- ogni riga viene verificata per lunghezza;
- i campi vengono estratti tramite posizione/lunghezza;
- sono supportati almeno:
  - required;
  - conditional required;
  - padding;
  - datatype;
  - enum/domain;
  - date;
- gli errori contengono:
  - flow;
  - schema version;
  - trace;
  - numero riga;
  - ID record ricavato dalla PK configurata, se disponibile;
  - identificativo sensibile, se autorizzato dalla policy di privacy (altrimenti redatto);
  - field ID;
  - field name;
  - posizione;
  - actual;
  - expected;
  - reason;
- viene prodotto JSONL;
- viene prodotto un summary;
- esistono test automatici;
- il core non contiene regole hardcoded specifiche SIOC.
- la build e i test passano su Linux x86_64 e Windows x86_64;
- gli artefatti release includono un eseguibile nativo per entrambi i sistemi operativi;
- gli artefatti release sono prodotti dalla CI, non dalla macchina di sviluppo;
- l'eseguibile Windows importa solo DLL di sistema e non contiene path dell'host di build (verifica automatica, §39C.1);
- senza cartella esterna l'eseguibile usa lo snapshot incorporato; con una cartella `schemas/` e `reference-data/` esterna (percorso con spazi, working directory diversa) usa quella, e produce lo stesso output della versione Linux;
- aggiornare schemi e reference data non richiede di ricompilare né ripubblicare l'eseguibile, purché il `schema_format_version` sia supportato;
- viene prodotto il report XLSX leggibile da un analista (§39C.3);
- l'output JSONL è identico byte per byte su Linux e Windows a parità di input e schema.

---

# 44. Regola per le future sessioni

Questo file è la **source of truth architetturale**.

Quando una richiesta futura entra in conflitto con il documento, l'agente deve:

1. segnalare il conflitto;
2. spiegare l'impatto;
3. proporre una soluzione compatibile;
4. cambiare il piano solo dopo una decisione esplicita.

Il principio da preservare è:

> **Il motore sa come validare. Lo schema sa cosa è valido.**


---

# 45. Decisione consolidata 2026-10-16: persistenza come capacità CORE

La persistenza **fa parte del core applicativo generico Lintra**. Il core deve disporre di un `PersistentContextRepository`, di operazioni transazionali e di servizi per deduplicazione, comparazione cross-run, versioni di sessione, snapshot e consultazione storica. L'archiviazione effettiva è fornita da adapter sostituibili e la policy normativa è risolta dal compiled schema/profilo, mai hardcoded per flusso.

Il context store di due passate rimane una **struttura effimera di proiezione** nel corso di un singolo run (DET §71–74); il repository persistente è un componente distinto ma **non CAD-specific**. L'inserimento dei dati avviene tramite preview esplicita e commit atomico dopo approvazione; errori di validazione, cancellazioni, fallimenti I/O e sessioni storiche definitivamente approvate non mutano lo stato corrente.

**SIOC.** Lo schema congelato dichiara già PK composte per tutti i cinque trace, `uniqueness_scope: trace` e `E_DUPLICATE_RECORD_KEY`. Quindi il controllo **intra-invio** è già previsto; si aggiunge memorizzazione persistente delle PK/digest e confronto audit fra invii. Non trasformare la semplice ricomparsa della stessa PK in un nuovo errore normativo cross-run finché non viene approvata `DEC-SIOC-HIST-01`.

**CAD.** Usa lo stesso repository, con policy aggiuntive per invii cumulativi trimestrali immutabili dopo approvazione, ULSS continuativa, identità progressivo-evento, annullamenti/riattivazioni, tracking dei file presenti/assenti e controlli temporali. Tutti i progressivi C/D restano nelle PK complete; non fanno parte della correlazione D→C.

**Privacy.** Non persistere per default i record originali o identificativi in chiaro: solo proiezioni minimizzate, identità protette e metadati indispensabili; il formato di storage e la gestione chiavi devono essere scelti con evidenze prima dell'uso reale.

Il requisito modifica le precedenti voci che rinviavano il *context store persistente* oltre il progetto. Documenti attuativi: `PERSISTENCE_CORE_SPEC.md`, DET §166–174 e MASTER_IMPLEMENTATION_PLAN §20. Ulteriori aggiornamenti richiedono change control, non una modifica implicita del comportamento dei flussi congelati.
