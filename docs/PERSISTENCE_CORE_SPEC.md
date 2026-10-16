# Lintra — Specifica architetturale della persistenza CORE (ADR-PERSIST-001)

> **Decisione approvata dall'autore, 2026-10-16.** La persistenza tra esecuzioni è una capacità del **core applicativo generico** di Lintra, non un plugin esclusivo CAD. L'implementazione concreta dello storage rimane sostituibile e la scelta del backend richiede evidenze e una successiva decisione tecnica. Questa ADR integra ARCH §45 e DET §166–174 e prevale sulle precedenti esclusioni di persistenza del piano v1.0.

## 1. Capacità obbligatoria e separazione delle responsabilità

- Il **core** fornisce contratti di persistenza, transazioni, snapshot, recupero identità, confronto fra invii, isolamento dei tenant, audit e gestione degli errori.
- Il **compiled schema / profilo normativo** definisce *che cosa* conservare e *come* interpretare variazioni e duplicazioni, non il driver di memorizzazione.
- Lo **storage adapter** fornisce l'implementazione (`FilesystemPersistenceAdapter`, eventuale backend DB in seguito) dietro trait. Non si impone SQLite, database server, ORM o un formato definitivo prima dei test con carichi reali.
- Il **contesto in memoria della singola esecuzione** rimane un indice/proiezione per cross-record a due passate; non va confuso con il repository persistente fra esecuzioni.
- Un flusso può utilizzare solo la deduplicazione, oppure versioni/snapshot e regole temporali. Nessun `if flow == "SIOC"`, `if flow == "CAD"` o controllo di ID campo hardcoded nel core.

## 2. Identità, scope e policy

Un'identità persistente contiene almeno `flow_id`, `schema_version`/compatibility group, namespace dell'organizzazione inviante, `trace_id`, tipo identificativo della chiave e digest canonico della PK. Per la ricerca cross-version lo schema deve dichiarare esplicitamente una policy di compatibilità: non equiparare automaticamente versioni di tracciato con PK differenti. L'indice storico può conservare anche progressivo ed evento assistenziale dove previsti dal profilo.

Il compilatore deve risolvere ed esaminare le definizioni dichiarative di `record_key` e le ulteriori `persistence_policy`; il loro schema dati preciso va versionato in `SCHEMA_AUTHORING.md`/manifest (non aggiungere JSON arbitrario ai file SIOC già congelati). Un namespace assente o ambiguo costituisce errore di preflight quando è necessario alla policy, senza effettuare join tra ULSS diverse.

**Politiche minime:**
1. `RunUnique`: stessa PK ripetuta nel medesimo tracciato/invio → `E_DUPLICATE_RECORD_KEY`.
2. `CrossRunAudit`: PK già vista in un altro invio → confronto/diagnostica secondo policy, NON errore automatico indiscriminato.
3. `HistoricalIdentityInvariant`: progressivo persistente legato all'evento originale; riuso su evento differente → errore, anche dopo annullamento.
4. `CumulativeSnapshot`: riconciliazione di inserimenti, aggiornamenti, assenze/annullamenti e riattivazioni per sessione normativa approvata; non dedurre annullamenti dai tracciati totalmente mancanti.
5. `TemporalRules`: piani genericamente compilati dal DSL e dati di contesto. Le regole temporali CAD restano nello schema/profilo CAD.

Le policy non si attivano sulla sola esistenza di un backend: ogni validazione distingue `local`, `full`, `historical_preview` e un'azione esplicita di `approve_commit`.

## 3. Applicazione ai primi due flussi

**SIOC** — I cinque `record_key` già presenti nella baseline `FROZEN` dichiarano `uniqueness_scope: "trace"` e `E_DUPLICATE_RECORD_KEY`. Sono prescritti il loro controllo **nel singolo invio**, l'estrazione e la memorizzazione persistente delle chiavi/digest e l'audit fra invii, ma NON esiste nella baseline una norma approvata che vieti tutte le riapparizioni della stessa PK *in invii differenti*. Il confronto inter-invio va quindi mantenuto `CrossRunAudit` finché l'autore non approva una policy SIOC più restrittiva (DEC-SIOC-HIST-01). Nessuna modifica tacita alla baseline già `FROZEN`.

PK SIOC: 1 = `(1.1,1.17,1.18)`; 2 = `(2.1,2.49,2.50)`; 3 = `(3.1,3.2,3.35,3.36)`; 4.1 = `(4.1,4.7,4.8,4.19.1)`; 4.2 = `(4.20,4.22)`.

**CAD** — PK A `(1.1,1.23)`; B1 `(2.1,2.36)`; B2 `(3.1,3.5)`; C `(4.1,4.2,4.4,4.5,4.11)`; D `(5.1,5.2,5.3,5.4,5.7)`. Il progressivo è sempre nella PK C/D; la correlazione D→C avviene *senza* confronto fra progressivi diversi, usando evento/data/tipo/figura. Storico continuativo per ULSS e sessioni trimestrali cumulativamente approvate, immutabili; progressivo→evento immutabile, altri campi correggibili. I file mancanti sono registrati (present/absent), producono warning e richiedono `PROCEED`/`ABORT` per la revisione storica: nessun annullamento per file mancante. L'assenza di B2 impedisce certe regole temporali; l'assenza di D rende CAD-17 non valutabile. Nessun algoritmo storico CAD nel modulo generico di storage.

## 4. Ciclo di vita transazionale

`validate → preflight storico → preview diff → (warning/consenso quando pertinente) → approve_commit → commit atomico → snapshot attivo`. Le validazioni fallite o cancellate non aggiornano lo stato corrente. Un invio approvato ha provenienza, identificativo e fingerprint idempotente; non si applica due volte per errore. Una nuova versione della stessa sessione è possibile **solo prima della sua approvazione definitiva**. Retrodatare o riscrivere una sessione definitivamente approvata → `E_APPROVED_SESSION_IMMUTABLE`. Concorrenza: una sola approvazione per namespace/sessione, controllo optimistic generation/CAS (o lock e transazione equivalente). Il commit registra numero versione e indice atomici, o torna allo stato precedente. Backup/restore e recupero da scrittura interrotta devono avere test dedicati.

Tracce presenti/assenti e consenso `PROCEED` sono metadati durevoli del run; un file presente anche con 0 record non equivale a file assente. Snapshot storici immutabili, stato corrente derivabile per replay, mai unicamente una `HashSet` delle PK dell'ultimo trimestre.

## 5. Minimo contenuto persistente e protezione dei dati

Per default **non archiviare righe originali** né codice fiscale in chiaro nei file di stato. Persisti projection minima, chiave pseudonimizzata stabile (es. HMAC con chiave gestita dall'applicazione), record fingerprint, metadati di schema/reference e sessione, eventuali valori strettamente necessari alle regole temporali (date e stati). La scelta esatta di cifratura/gestione chiavi e retention va documentata prima di salvare dati reali; un semplice SHA-256 non salato di CF o identificativi prevedibili non è una garanzia di anonimato. Non emettere PHI in log, eccezioni, filename o fixture. Isolare storage per namespace organizzativo e controllare l'accesso ai file. Metadati audit: hash input, versione/hash schema, versione reference data, UTC e risultato validazione. Nessun repository in Git di dati dei pazienti.

## 6. Interfacce ed errori

- `PersistentContextRepository`: snapshot read, query PK, query progressivo/evento, store/recover metadata, begin/commit/abort transazione.
- `HistoricalValidationService`: costruisce la preview con il `CompiledFlowSchema` e il repository, non interpreta nomi di flussi.
- `PersistenceTransaction`: staging, commit atomico, abort, collision detection, rollback.
- `TracePresence`: enum `Present{record_count}`/`Absent`, serializzabile in snapshot.
- `SessionState`: `Draft`, `Validated`, `ApprovedImmutable`; una `ApprovedImmutable` non può essere aggiornata retroattivamente.
- Errori normativi (es. riutilizzo PK) → `ValidationError`, esito `CompletedInvalid`; errore di I/O/corruzione/concorrenza → `ValidationRunError`; rifiuto di consenso o annullamento → nessuna mutazione persistente.

## 7. Minimi test di accettazione

SIOC: doppia PK nello stesso file → errore; stessa PK fra run → confronto/audit, nessun errore automatico senza policy; namespace diversi isolati; recovery dopo restart; file reali mai usati. CAD: T1→T2→T3 con annullamento e riattivazione, riuso progressivo/evento differente bloccato, tranche D assente senza annullamenti D, B2 mancante e conseguente `Unknown`, freeze della sessione approvata, rollback atomic, contese concorrenti, fallimento/annullamento no commit. Test su Linux e Windows. Test prestazionali su volumi sintetici rappresentativi (fino a centinaia di migliaia di righe).

## 8. Dipendenze di implementazione

P2: tipi per identità/sessione/namespace, schema policy. P3: `PersistencePolicy` raw/compiled e lint dei riferimenti. P5: errori e audit. P7: service boundary; P13: projection locale; successivamente **T-PERSIST-01…12** (generic core) e T-CAD-01…12 (profilo CAD). La persistenza SIOC di base è un acceptance gate prima della stabilizzazione dei controlli cross-run CAD, ma non blocca il bootstrap P1. Non associare la scelta concreta del backend al T-P1-01.

## 9. Decisione residua delimitata

**DEC-SIOC-HIST-01 (non bloccante per P1 e per la persistenza):** stabilire se una PK SIOC già presente in una sessione precedente debba essere accettata come ritrasmissione identica, trattata come modifica, o vietata. Fino all'approvazione: solo unicità nel singolo trace + persistenza e audit cross-run, nessun errore normativo cross-run inventato.
