# T-P7-05 — Preflight / CNF-SERVICE-DI-01

2026-10-21. **Proposta A APPROVATA** dall’autore con le precisazioni vincolanti
CNF-SERVICE-DI-01. Decisione registrata in IMPLEMENTATION_STATUS.md e
[report di consegna](T-P7-05.md). Stop obbligatorio dopo T-P7-05, nessuna
autorizzazione esecutiva alle task successive.

Il testo seguente conserva il preflight storico, precedente all’implementazione.
Nessun codice T-P7-05 scritto. T-P7-04: servizio locale e clock reale disponibili,
280 test locali passati; CI 39587379476 Linux/Windows success su dcb8c59.
Il gate runtime è soddisfatto e distinto da questa decisione.

## Evidenze del ciclo

Letti prompt T-P7-05, DET §84/92–94/166–174, ARCH §45,
PERSISTENCE_CORE_SPEC e prompt persistenti. Il contratto completo richiede:

| Dipendenza del costruttore | Stato/assegnazione | Ciclo o prerequisito mancante |
|---|---|---|
| Arc<SchemaRegistry> | Reale e verificata | Nessuno. |
| Clock iniettabile, DET §166 | Reale con with_clock e SystemClock, test fisso | Nessuno. |
| Arc<dyn ReferenceDataProvider> e fake in-memory, DET §92–93/T-P7-05 | T-P11-01 e T-P11-02, modulo attuale vuoto | T-P11-01 richiede proprio T-P7-05. |
| PersistentContextRepository, DET §166–167 | T-PERSIST-03, payload T-PERSIST-02 | Richiede P13 e T-PERSIST-02; quest'ultima richiede T-PERSIST-01/P10. |

CNF-RECORD-01 vieta di anticipare P11/P13, usare placeholder o dichiarare
implementati controlli mancanti. La persistenza è core obbligatoria, non plugin
opzionale: non la rimuoviamo dai requisiti finali. Anche un NoOp repository
conforme a DET §166 necessita del trait reale e dei payload, non di un tipo
vuoto dichiarato come repository funzionante. Passare un reference_data_root
ignorato non realizza il costruttore del piano.

Le ratifiche precedenti risolvono il record locale e gli errori incrementali,
ma non assegnano espressamente i costruttori/fake provider alla consegna locale.
La scelta cambia lo scope dichiarato di T-P7-05 e i contratti di costruzione,
quindi viene sottoposta all'autore prima di implementare o dichiarare DONE.

## Visibilità compiled: fonte prioritaria ed eccezione circoscritta

Il prompt chiede pub(crate); DET §94 dice invece «possono essere pub(crate)
finché non serve esporli». Gli handle compiled sono già pubblici in sola
lettura perché SchemaCompiler.compile/SchemaRegistry.resolve e l'integrazione
ne hanno bisogno. I costruttori/RecordParts restano strettamente privati,
nessun Deserialize/Default o setter: i doctest di non-forgiabilità passano.
Non proponiamo di rendere pubblici moduli engine/persistence o costruttori.
La proposta A ratifica il mantenimento degli handle necessari, secondo §94,
con audit esplicito degli export e nessuna espansione indiscriminata dell'API.

## Proposta A — costruttori LocalOnly reali, porte complete rinviate esplicitamente

1. Consegnare in T-P7-05 il costruttore esplicito:
   build_default_local_service(schema_root: PathBuf, encoding: RawResolvedEncoding)
   -> Result<ValidationService, SchemaLoadError>. Root non vuota, nessuna
   ricerca della working directory, locale o directory utente implicita.
   Usa FileSystemSchemaRepository/SchemaRegistry e SystemClock reali.
2. Conservare ValidationService::new(Arc<SchemaRegistry>) come comodità sistema
   e with_clock(Arc<SchemaRegistry>, Arc<dyn Clock>) per DI esplicita: supporta
   repository custom/in-memory e clock deterministico senza mock del servizio.
3. Non aggiungere argomenti reference/storage ignorati o provider/repository
   fittizi. Le firme complete con ReferenceDataProvider, relativi in-memory/
   filesystem provider e PersistentContextRepository restano consegne esplicite
   T-P11-01..03/T-PERSIST-02..03, con successivo wiring nell'application.
   Non dichiarare consegnato ora build_default_service(schema_root, reference_root).
4. API primaria per CLI/UI resta ValidationService, non il RecordValidator.
   Audit degli export con giustificazione; handle compiled necessari pubblici
   immutabili secondo l'eccezione §94, moduli/constructor interni privati e
   doctest di privacy/non-forgiabilità preservati. Nessun nuovo export inutile.
5. Test doubles realmente disponibili: InMemorySchemaRepository e VecSink
   esistenti, NoopProgressObserver/CollectingProgressObserver nei test support.
   InMemoryReferenceDataProvider non simulato, resta T-P11-02.
6. Test: costruttore filesystem realmente eseguito, equivalenza con DI in-memory,
   root vuota errore, encoding esplicito preservato (nessun fallback su profilo
   non supportato), clock iniettato, API consumer esterno e privacy.
7. Chiudere T-P7-05 solo come **DONE — LocalOnly API scope approved by
   CNF-SERVICE-DI-01**, mantenendo un ledger delle firme/porte rinviate. Poi
   autorizzare l'avanzamento sulle dipendenze atomiche reali: T-P7-06/CLI locale
   possono dipendere dalla consegna API locale verificata, senza P11/PERSIST
   già complete. Ogni task conserva gate/CI; T-P2-03/M1 rimangono aperte.

Non cambia la semantica normativa né la distinzione LocalOnly/Full, non si
aggiunge storage o dataset, nessun ARCH/DET/Piano viene riscritto. È una
ratifica circoscritta dell'ordine e del confine API corrente, non una rinuncia
alle capacità core finali.

## Alternativa B

Mantenere il blocco e definire prima il contratto completo delle dipendenze e
un ordine alternativo compatibile, con eventuale anticipazione espressamente
ratificata dei soli trait/payload reali. Nessun fake vuoto o backend automatico.

## Handoff storico prima della ratifica

T-P7-04 e la sua CI sono completate con successo. Nessuna factory, namespace,
provider o repository di T-P7-05 implementato. Dopo il gate del servizio,
T-P7-05 e le task dipendenti attendono la decisione dell'autore.
