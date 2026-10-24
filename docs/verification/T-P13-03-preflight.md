# T-P13-03 — Preflight ContextStore effimero

2026-10-24. Nessun codice avviato: 13-02 commit 1d210d1 pubblicato, dispatch
CI accettato alle 09:55 UTC circa, esito in attesa. Gate locali 548 test/64
suite passati. Task 13-03 richiede trait insert/find/iter_trace e indice in
memoria, zero/uno/molti match, ordine deterministico. Prerequisito 13-02 da
chiudere nativamente prima del codice successivo.

Fonti lette: DET §71–74, ARCH §45, DET §166–174 e PERSISTENCE_CORE_SPEC intera,
Piano §4/§9 P13/§14 e prompt atomico. Store per una singola esecuzione;
PersistentContextRepository/storia/transazioni/sessioni non implementati qui.
Non introdurre backend, dataset o policy normative. Nessuna dipendenza nuova.

## API e rappresentazione proposta

Preservare le firme concettuali DET §71. find(trace,join,key) cerca i binding
Target di 13-02, compresa la self-join: le chiavi Source rimangono nel record
per il futuro chiamante, non devono indicizzare falsi target. MissingComponent
non è indicizzabile. Conservare ogni record del trace anche senza join o con
chiavi mancanti. Non deduplicare chiavi uguali: tutti i match restano presenti,
la cardinalità e le policy normative appartengono al futuro evaluator.

InMemoryContextStore possiede una mappa per trace. Records ordinati per numero
riga one-based, senza dipendere dall’ordine di inserimento né dalla HashMap.
Indice per JoinId/JoinKey verso riferimenti condivisi immutabili ai soli
ProjectedRecord; Arc evita copie ripetute di values/identity nei vari indici.
Lookup borrowed senza ricreare Vec di record o copiare le chiavi di ricerca.
RelatedRecordSet view borrowed con len/is_empty/iter, senza selezione scalare
arbitraria. Restituire tutti i match in ordine di riga crescente.

Controllare riga zero, ID trace/join vuoti e doppia posizione trace/riga prima
di mutare lo store. La doppia posizione è incoerenza tecnica del contesto;
due record con la stessa PK ma righe differenti restano entrambi presenti:
la violazione di unicità normativa non è un side effect del driver di storage.
Un Err di inserimento deve lasciare invariati record e indici esistenti.

## Confini che il consumer dovrà rispettare

Set vuoto significa nessuna proiezione indicizzata, non dimostra da solo
presenza/assenza del file del trace o una violazione normativa. Disponibilità
input, scope, namespace da parametri espliciti e cardinalità sono verificati
dal piano/preflight/evaluator delle successive task. Nessuna falsa garanzia
di isolamento multi-run o multi-tenant: uno store nuovo per ogni singolo run
nel namespace preflightato, nessun registry globale o riuso storico.
find non inventa record quando l’input manca. Il caller futuro deve distinguere
TraceInputUnavailable dall’insieme vuoto di un input effettivamente disponibile.

## Verifiche previste

Proiezioni da compiler/projector reali, zero/uno/molti match, ordine indipendente
dall’inserimento, isolamenti per trace e JoinId, source/target self-join diversi,
missing non indicizzato ma record iterabile, duplice posizione rifiutata senza
mutazioni, payload condiviso (identità dello stesso record), trait object reale,
nessuna copia di righe complete o filesystem. Suite, fmt/Clippy/build e gate
Linux/MSVC. Fonte/baseline/grammar preservate; spiegazione completa nel dossier.
