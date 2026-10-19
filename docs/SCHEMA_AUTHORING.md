# Schema authoring Lintra — bozza di riconciliazione T-P0-07

Stato: **DRAFT / BLOCKED su decisione di scope**, 2026-10-19.
Questo documento presenta scelte concrete da revisionare; non è il contratto
canonico approvato, non contiene ancora il JSON Schema completo e non
certifica che i JSON ricevuti siano compilabili. Nessun compilatore o
validatore normativo è stato implementato.

## Fonti e prerequisiti verificati

- [ARCH](ARCHITECTURE.md) §7–12, §21–22, §39C.2, §45.
- [DET](DETAILED_DESIGN.md) §8–21, §35–37, §66–72, §91, §106, §135,
  §155, Addendum A.1, §166–174.
- [Piano](MASTER_IMPLEMENTATION_PLAN.md) §4, §9/P0, §13.5, §14.
- [Persistenza core](PERSISTENCE_CORE_SPEC.md), ADR-PERSIST-001.
- T-P0-03/04/05 verificati con i checker documentali; non basta lo stato DONE.
- [Decisioni umane](normative/DECISIONS.md) e dossier
  [SIOC](normative/sioc/rules-and-joins.md) / [CAD](normative/cad/rules-and-joins.md).

La fonte SIOC ricevuta `SCHEMA_INPUTS/SIOC_2026-04-20_FROZEN/SCHEMA_AUTHORING.md`
versiona la DSL con `canonical_schema_version: "1.0.0"`; non è un sostituto
automatico del presente contratto generico. Il flow aggregato è autorevole
per quel profilo; i file split sono viste documentali. La discrepanza
2026-04-20 del flow contro 2025-12-16 dei trace va conservata come provenienza,
non risolta scegliendo silenziosamente una versione.

## CNF-LINTRA-DSL-01 — scope dei valori derivati

**Nuova evidenza**, distinta dai CNF-01..11 già chiusi:

- DET §135: «Questa feature non è MVP.» dopo DerivedValueId/DerivedValuePlan.
- Piano §13.5 rinvia esplicitamente i valori derivati (DET §135).
- SIOC trace 2 richiede `sending_region`, derivato substring da 1.17 tramite
  join, usato dalla presenza di 2.30/2.31.
- SIOC trace 3 richiede `age_at_admission_years`, età compiuta da 1.9/2.5,
  usata da R-3.34-AGE. Non equivale a sottrarre gli anni di calendario.
- CAD_RUNTIME_PROFILE richiede `calendar_year_delta` per A.1.24 e
  B1.2.17/2.18; non equivale all’età compiuta SIOC.
- DET §17 non contiene `const`, `condition`, `derived_eq`. I riferimenti
  nominati sono risolvibili staticamente; un confronto con un valore derivato
  richiede un piano tipizzato che non è presente nell’enum di esempio.

Non si possono contemporaneamente dichiarare il profilo interamente
compilabile e mantenere ogni valore derivato fuori scope, né sostituire
questi controlli con esito positivo o disabilitarli senza decisione.

### Proposta A — estensioni minime richieste dalle fonti (consigliata)

Autorizzare una deroga circoscritta al rinvio del Piano §13.5/DET §135:

1. Nel contratto di authoring prevedere piani dichiarativi tipizzati per
   `substring`, `age_in_completed_years` e `calendar_year_delta`, soltanto
   perché richiesti dai profili ricevuti. Nessuna funzione arbitraria,
   scripting o stato globale mutabile.
2. Risolvere i nomi di condizioni e valori derivati a compile time con
   rilevamento di riferimenti mancanti/cicli. Esplicitare il lowering di
   const e confronti derivati; nessuna equivalenza implicita con false.
3. Prevedere capabilities controllate dal compiler: un operatore/piano
   richiesto e non supportato causa schema error, non regola saltata.
4. Documentare ora il contratto; implementare solo nelle task autorizzate
   per compiler/DSL e dopo i loro gate. La decisione non avvia codice Rust,
   backend persistente, regole temporali CAD o UI in T-P0-07.
5. Prima della futura implementazione, registrare una proposta esplicita di
   adeguamento delle task interessate (P3/P10/P13 e profili CAD), senza
   modificare autonomamente il Master Plan o DET. La deroga non autorizza
   tutte le funzioni derivate ipotizzabili.

### Alternativa B — conservare integralmente il rinvio

Mantenere questi costrutti non eseguibili fino a una futura revisione di
scope. Il compiler deve rifiutare i profili che li richiedono; lo stato
FROZEN documentale SIOC non dimostra il superamento di G0 o l’eseguibilità.
Non è consentito eliminare le regole o dichiarare equivalenza con una
validazione parziale. La chiusura T-P0-07 deve registrare questo limite e
non può certificare tutti i costrutti ricevuti come compilabili.

**Decisione richiesta all’autore:** A oppure B. Nessuna delle due opzioni
è attiva soltanto perché scritta nella bozza.

## DEC-AUTHORING-FORMAT-01 — proposta di formato, non ancora ratificata

DET Addendum A.1 richiede `SchemaFormatVersion(pub u32)`, presente in ogni
RawFlowSchema e nel MANIFEST.json del bundle. Nei flow SIOC/CAD ricevuti
non compare `schema_format_version`. La stringa `canonical_schema_version`
versiona il contratto DSL ricevuto: non è un u32 e non si può convertire
implicitamente in un formato motore.

Proposta per il contratto da completare dopo la decisione di scope:

- Primo formato motore numerato `schema_format_version: 1` (u32).
- Versione del formato, versione operativa dello schema, versione della
  fonte normativa e versione del bundle sono nozioni distinte.
- Configurazioni nuove, revisionate e versionate dichiarano esplicitamente
  il formato. Importare/normalizzare una fonte legacy è un’attività esplicita
  con provenienza/hash e diff revisionato; non è un fallback del runtime.
- I file FROZEN originali rimangono byte per byte invariati. Nessun default
  implicito di formato, nessuna patch silenziosa o combinazione di bundle.
- Formato o capability incompatibile produce errore prima di validare dati.
  Non inventare una versione minima del binario: sarà stabilita dalla prima
  release che implementerà il formato/capability e registrata nel contratto.

Esempio **parziale di proposta**, con identificativi sintetici, non flow
completo né file da caricare oggi:

```json
{
  "schema_format_version": 1,
  "flow_id": "EXAMPLE",
  "schema_version": "example-revision-1",
  "regulatory_version": "example-source-1"
}
```

La forma `schema_version` proposta deve essere definita e revisionata nel
JSON Schema completo e mappata esplicitamente a SchemaKey.version. Non
reinterpretare automaticamente i metadati legacy dei singoli trace.

### MANIFEST.json del bundle

La struttura obbligatoria da ARCH §39C.2 contiene bundle_version,
schema_format_version e hash dei file, ed è uguale per embedded/external.
Il `manifest.json` del singolo profilo ricevuto non è automaticamente il
manifest del bundle: non rinominarlo né alterarlo.

Proposta concreta: mappa `files` da path relativo a SHA-256 dei byte esatti;
JSON UTF-8, path relativi portabili, senza assoluti, `..` o traversal. Nessuna
serializzazione canonica/hash semantico dello schema viene anticipata
rispetto a T-P14-03. Il manifest non contiene l’hash di se stesso: il checksum
dell’archivio e gli ulteriori artefatti di release restano responsabilità P15.

Esempio **sintetico**: il digest è quello reale del contenuto vuoto, non
l’attestazione di un bundle completo o valido.

```json
{
  "bundle_version": "example-1",
  "schema_format_version": 1,
  "files": {
    "schemas/example/empty-file.json": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
  }
}
```

Le directory `schemas/` e `reference-data/` e tutte le dipendenze effettive
vanno presenti e verificate: il digest corretto di un singolo file non rende
completo il bundle. Cartella esterna selezionata incompleta → errore,
nessun fallback per singolo file. Versione esplicita, mai latest implicita.

## Contratti già determinati da conservare

| Aspetto | Contratto / limite |
|---|---|
| Modello | RawFlowSchema → SchemaCompiler → CompiledFlowSchema; engine solo compiled; conversione byte range una volta |
| Tipi osservati | numeric, alphanumeric, date; T CAD resta alphanumeric + constraint posizionale, non nuovo tipo fisico |
| Alias | Usare il mapping confermato dai profili (A SIOC → alphanumeric); non assumere che l’esempio generale A→Alphabetic prevalga sulla normalizzazione confermata del profilo |
| Valore | Raw esatto distinto da logical ottenuto col padding dichiarato; nessun trim globale |
| Date | DDMMYYYY dove dichiarato, con validità calendario; nessun ordinamento lessicografico dei byte come confronto cronologico |
| Encoding | runtime_required, posizioni byte; nessuna code page scelta dal sistema operativo; DEC-13 non chiusa senza evidenze |
| Lessico | Non dedurre ASCII-only o un set alfabetico esteso da AN; il contratto deve specificare il profilo lessicale e la compatibilità con l’encoding esplicito |
| Decimal/Alphabetic | Varianti concettuali DET §13, nessun campo ricevuto le richiede; non fissare un separatore decimale o repertorio di lettere per ipotesi |
| Presenza | Base required/optional, liste required_if/forbidden_if, logica a tre valori, conflitti visibili; OBB/FAC/NBB preservati con decisioni del profilo |
| Assenza | Spaces/Exact/AnyOf/NeverMissing del design, senza default AllZeros; rappresentazioni raw della larghezza del campo |
| Join | Chiavi ordinate collision-safe, cardinalità e missing policy esplicite, nessun first() su risultati ambigui |
| Diagnostica | Errori simbolici E_* stabili; codici legacy E010/E003/E012 ricevuti conservati come provenienza, mapping esplicito verso i codici del design, nessun export legacy implicito |
| Identità | Binding schema-driven DET §36, inventario T-P0-05; estrazione dopo controllo lunghezza, senza validazione normativa dell’identità |
| Reference | Dipendenze dichiarate con chiave/versione esplicita; dataset indisponibile è errore di preparazione, non dominio paziente invalido |
| Esecuzione | Condizioni e constraint in AST chiuso; non esecuzione deliberata distinta da capability obbligatoria non supportata |

La bozza finale dovrà contenere esempi completi per ciascun costrutto
ammesso, JSON Schema non permissivo su operatori sconosciuti e un mapping
esplicito raw→compiled. Questa tabella non sostituisce tali deliverable.

## Decisioni normative già approvate da trasferire nella configurazione

- CAD B1.2.12: obbligatorio, 000 valido, tre spazi mancanti.
- SIOC 1.1/2.1/3.1 per ingressi 2025 conclusi nel 2026: alfanumerico
  valorizzato diverso da tutti zeri, 20 byte fisici, padding ricevuto;
  niente lunghezza logica otto o prefisso/progressivo.
- CAD iniziale: almeno una nello storico dell’evento incluso invio corrente,
  namespace ULSS conservato; non una nuova iniziale per ogni invio.
- SIOC parent opzionale assente: `NotApplicable`, esito con spiegazione
  esplicita di anagrafica presente e dati ricovero correlato assenti.
  Non equiparare a controllo superato, dato invalido o target ambiguo.

Non riscrivere gli snapshot ricevuti dei dossier per applicare queste
correzioni: appartengono alla successiva configurazione versionata revisionata.

## Persistenza e insiemi: contratto da completare, nessun backend scelto

DET §168 richiede serializzazione da definire/approvare qui. Sono già
approvati i significati di run_uniqueness, cross_run_audit, cumulative_snapshot,
history_identity_invariants, snapshot_scope, temporal_rules, missing_trace_policy.
Non è ancora approvata una forma JSON per i loro parametri/riferimenti.
La presenza di uno storage non può attivare implicitamente alcuna policy.

La forma finale deve dichiarare namespace, compatibilità fra versioni,
PK/progressivo/evento, proiezioni minime, periodo/sessione, assenze dei trace,
azione esplicita di preview/approve_commit e requisiti di protezione esterni.
Mai credenziali o chiavi in JSON normativo. Nessuna riga grezza o CF in chiaro
nello storage default. File assente non produce annullamenti; approvazione
immutabile e commit atomico/idempotente restano invarianti del core.

DEC-05-CAD-SET-CONTRACT: sospensioni zero-o-più e aggregazioni temporali non
sono rappresentabili forzando ogni insieme in OneOrMany. Servono un modello
per insiemi vuoti leciti e un piano deterministico tipizzato, da sottoporre
alla revisione del contratto completo; nessuna cancellazione di regole.
DEC-05-CAD-EVENT-CONTEXT: più B1 nello stesso evento non consentono di
scegliere casualmente data inizio o conclusione. Il profilo impone diagnostica
su conflitti; il contratto dovrà definirla e propagare l’indeterminatezza.

I sette requisiti del runtime profile CAD rimangono richiesti e non eseguibili
finché non implementati e testati nelle task pertinenti. ADR-PERSIST-001
prevale sulla vecchia frase «outside generic core» del registry ricevuto.

## Checklist di completamento T-P0-07

- [x] Prerequisiti documentali verificati contro le fonti.
- [x] Decisioni normative ricevute registrate e riportate senza ampliarle.
- [x] Conflitto sullo scope derivati e impatto individuati con fonti puntuali.
- [x] Proposta di riconciliazione e alternative sottoposte all’autore.
- [ ] Decisione autore su CNF-LINTRA-DSL-01 ricevuta.
- [ ] JSON Schema del flow e del manifest completi, con esempi validi/invalidi.
- [ ] DSL completa e mapping di ogni costrutto al modello compilabile.
- [ ] Definizioni lessicali/padding non ambigue e boundary encoding esplicita.
- [ ] Identity/reference binding e policy persistenti serializzati e revisionati.
- [ ] Verifica strutturale degli esempi e tracciabilità della copertura.
- [ ] Revisione umana del contratto completo (accettazione della task).

**T-P0-07 non completata.** Approvare l’opzione di scope non equivale ad
approvare automaticamente il formato proposto o a chiudere questa checklist.
Evidenze e handoff: [verifica T-P0-07](verification/T-P0-07.md).
