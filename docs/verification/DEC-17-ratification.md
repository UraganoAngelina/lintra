# DEC-17 — ratifica A estesa, 5 novembre 2026

Fonte autorevole: risposta integrale dell’autore del 5 novembre 2026 alle domande
DEC-17 e CNF-CAD-NAMESPACE-BINDING-01. Questo è il registro operativo della ratifica,
non una trascrizione verbatim. La proposta storica rimane conservata e viene
estesa esplicitamente; non attribuire la nuova autorizzazione al testo originario.

**RATIFICATA: valutazione sperimentale del parallelismo selettivo nella pipeline
ibrida. Nessuna adozione nel prodotto.** T-P17-02 è prerequisito, non misura dello
speedup parallelo. DEC-14 non riceve soglie numeriche implicite.

## Tre attività sequenziali autorizzate

A1: benchmark LocalOnly con tracce indipendenti, schema compilato identico,
worker Rust standard limitati e confronto con sequenziale.

A2: spike Full con passata1 selettivamente parallela per tracciato, validazione
locale/identità/proiezione, aggregazione deterministica, ContextStore completo e
barriera; passata2 cross-record/cross-trace obbligatoriamente sequenziale.
Nessun parallelismo dentro un singolo trace e nessuna terza passata normativa.

A3: confronto end-to-end Full inclusivo di lettura, locale, proiezione, worker,
aggregazione/indicizzazione, passata2, output e finalizzazione. Non presentare
il solo tempo dei worker come tempo Full. Dipendenze nello stesso trace e fra
trace, join zero/uno/molti devono essere rappresentate realmente.

## Verifica dell’implementazione prima del progetto sperimentale

Ricostruire RecordValidator, TraceValidator, FlowValidator, ValidationService,
ContextProjectionPlan, ProjectedRecord, ContextStore/InMemoryContextStore,
CrossRecordValidationPlan, ValidationSink, ValidationSummaryBuilder,
ProgressObserver e CancellationToken. Verificare Send/Sync/condivisione effettiva
di schema/reference, lettori isolati, proiezioni e mutabilità, emissioni/summary,
progress/cancel/max_errors. Registrare divergenze fra documenti e codice prima
di concludere, senza modificare API pubbliche per accomodare lo spike.

Il compiler/piano stabilisce la dipendenza normativa; il runtime può schedulare
solo indipendenza dimostrata. Non riclassificare regole, PK duplicate, join,
storia o tempo come locali in base a flow/trace/campo. Non mutare cardinalità,
missing semantics, identità, dati proiettati, algoritmi o ordine degli output.
Non sostituire l’architettura a due passate.

## Isolamento e ownership

Schema, piani, configurazione e reference condivisibili solo se sicuri in sola
lettura. Per worker isolare RecordReader, buffer, stato locale, diagnostici,
proiezioni, conteggi ed errori tecnici. Coordinatore autorevole per ContextStore,
ordine, summary/progress, propagazione errori, cancel e budget globale.
Nessun Mutex globale sull’intera validazione locale né stato globale dipendente
dallo scheduling. Il ContextStore resta effimero, separato da SQLite; worker
non mutano PersistentContextRepository/storico.

ContextProjectionPlan è la fonte dei soli dati da mantenere. Preservare identità,
trace/line, chiavi join e valori richiesti/metadati esistenti. Non conservare raw
record come scorciatoia, non perdere/duplicare proiezioni. ContextStore finale
semanticamente equivalente, join/cardinalità identiche. Inserimento sequenziale
ammesso se incluso nel tempo misurato. Passata2 mai su contesto incompleto
presentato come completo.

## Equivalenza e guasti

Confrontare risultati, codici/provenienza/trace/line/campo/ordine regole,
conteggi/summary, completezza, skipped/inconcludenti, sink e JSONL. Byte-per-byte
sui report deterministici; nessuna eliminazione/normalizzazione silenziosa di
campi. Metadata legittimamente variabili vanno prima ricondotti al contratto.
Verificare anche l’ordine sequenziale reale rispetto a trace→line→ordine fisico
campo→stage→regola; non cambiarlo per facilitare lo spike o usare completion order.

max_errors conserva DEC-16 A: selezione errori indipendente dall’ordine dei
worker, prefisso globale e classificazione effettiva. Cancel propaga, interrompe,
non lascia thread, non espone Complete falso o passata2 su contesto parziale.
I/O/sink/guasti restano tecnici, mai violazioni normative. Tutti i thread
appartengono al run foreground e vengono terminati/raccolti, niente detached.

## Streaming, privacy e misure

Niente Vec globale di errori, copie integrali input o duplicazione indiscriminata
delle proiezioni. Valutare bounded queue/backpressure/staging/riordino/emissione
incrementale. Spool ammesso solo protetto, misurato e conforme a privacy; se non
sicuro nel perimetro dello spike documentare il limite, senza bypass.

Misurare picco memoria, temporanei, I/O extra, riordino, aggregazione contesto e
overhead. Build release equivalenti, compilazione esclusa, singolo processo
foreground controllato, nessun’altra build/benchmark locale simultaneo.
Hardware effettivo, 1/2/4 worker se risorse consentono, più ripetizioni/variabilità,
volumi piccolo/medio/grande, bilanciati/sbilanciati, pochi/molti errori,
proiezioni consistenti. Distinguere LocalOnly, passata1, aggregazione, passata2,
Full totale, memoria/I/O/speedup/regressioni; sintetico non è carico operativo.

## Isolamento del codice e conclusione

Branch sperimentale dedicato o isolamento equivalente autorizzato, single-agent.
Worktree aggiuntivo non obbligatorio. Preferire std::thread/scope/canali bounded;
vietati Rayon/Tokio/runtime async/crate scheduling/nuove dipendenze native.
Nessuna modifica API/CLI/UI o percorso di produzione; non chiamare spike una
modifica al runtime del prodotto. Se Full costa sproporzionatamente, documentare
il vincolo e limitare a ciò che è verificabile senza compromessi.

Report obbligatorio: architettura reale/candidata e divergenze, stato condiviso e
isolato, staging/contesto/barriera/memoria, hardware/build/input/worker/ripetizioni,
tempi/memoria/I/O/speedup/regressioni, equivalenza/JSONL/join/cancel/guasti/budget.
Conclusione motivata: proporre adozione futura, approfondire/rinviare/non adottare;
dire se il vantaggio LocalOnly si traduce davvero in Full. Esito possibile anche
contratti non dimostrati: nessuna adozione. Eventuale ADR di adozione separata.

T-P17-05 non DONE prima di esperimento/report/gate reali. Dopo handoff proseguire
READY. Decisione non blocca18-03/CAD indipendenti. Restano vietati subagenti,
Superpowers, delega/sessioni autonome/task concorrenti; worker Rust interni
foreground esplicitamente autorizzati. Commit/push/CI pertinenti autorizzati.
