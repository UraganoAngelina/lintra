# CNF-DIAGNOSTIC-ORDER-01 — audit attuativo, 8 novembre 2026

A ratificata; qualifica tecnica in corso. Solo documenti e test, nessun algoritmo
modificato. Fonti: ratifica8novembre, DET73/77–80/115/133/156/158 e codice corrente.

## Ordine effettivamente verificato prima della modifica a DET

| Livello | Comportamento esistente | Fonte |
|---|---|---|
| trace PASS1 | prepare_flow ricostruisce i percorsi enumerando schema.traces; ordine richiesta/HashMap non governa l'output | engine/flow_execution.rs::prepare_flow |
| righe | RecordReader in ordine fisico, linee reali one-based, terminatore finale non crea record | input/record_reader.rs e engine/trace_validator.rs::run |
| larghezza | RecordLength è il solo errore della riga malformata, senza campo/identità inventati; niente slicing/regole/PK utilizzabili | record_validator.rs::validate_record |
| campi | compiled.fields in ordine fisico; raw non fisicamente ordinato è rifiutato dal linter | schema/linter.rs e record_validator.rs |
| pipeline | presenza→valore logico→datatype→constraint compilati; primo errore primario termina quel campo, non gli altri | field_pipeline.rs::after_presence |
| constraint | compiler raggruppa prima non-reference e poi reference, mantenendo l'ordine relativo dentro ciascun gruppo | compiler.rs::compile_field |
| namespace | controllo dopo pipeline del campo valida, prima di passare al campo successivo; nessun mismatch derivato da campo invalido | record_validator.rs |
| regole locali | dopo TUTTI i campi, ordine relativo di dichiarazione delle regole classificate locali; prima assertion violata per regola | record_validator.rs e rule_validator.rs |
| PK duplicate | collector dopo validazione locale e regole della stessa riga; se budget già raggiunto non esegue il controllo successivo; chiave leggibile e namespace qualificato necessari | trace_validator.rs e ProjectionCollector::collect |
| trace PASS2 | enumera di nuovo schema.traces, non le mappe di indice join | flow_validator.rs::run_cross_record_rules |
| righe PASS2 | store.iter_trace su BTreeMap di linee reali; solo proiezioni disponibili, niente righe fabricate | context/in_memory.rs |
| regole PASS2 | filtra il piano cross del source trace in ordine compilato: regole cross dichiarate prima, controlli temporali aggiunti nell'ordine della policy | compiler.rs e flow_validator.rs |
| assertion cross | ordine dichiarato; prima violazione ricordata fra quelle valutate; violazione prevale su Unknown; una diagnostica per regola | flow_validator.rs::evaluate_rule |
| eventi / senza campo | nel medesimo piano cross e con la reale riga owner; field=None quando non esiste un campo locale pertinente | cross_diagnostics.rs e error_factory.rs |
| run / tecnico | preflight/I/O/schema/sink/summary failure sono errori tecnici; causeUnknown e inventario sono evidenza del summary, non ValidationError con linee arbitrarie | application/service.rs e domain/errors.rs |

Quindi non è corretta una chiave globale field→stage→rule nemmeno all'interno
PASS1: un errore sul campoDATA può precedere una regola locale sul campoPK,
e il duplicatoPK arriva dopo entrambi. Anche PASS2 segue regole del piano, non
riordina le diagnostiche per il campo eventualmente scelto come anchor.
È un'ambiguità di formulazione coperta dalla ratificaA: si documentano le
precedenze reali, non si modifica il prodotto per simulare una gerarchia diversa.

## Esempio indipendente: tests/diagnostic_order.rs

Schema S0,T1, richiesta T1,S0. Due righe per trace. PK/KEY/DATA hanno posizioni
1/3/5 e larghezza2. T1DATA ha dominioOK; localePK richiedeZZ, poi PK duplicata
sulla seconda riga. S0 confronta DATA, poi PK, poi un'esistenza senza campo.
Sequenza oracle scritta nel test, non calcolata dall'evaluator:

```text
T1:1 DATA E_INVALID_DOMAIN
T1:1 PK   E_CROSS_FIELD (regola locale)
T1:2 DATA E_INVALID_DOMAIN
T1:2 PK   E_CROSS_FIELD (regola locale)
T1:2 -    E_DUPLICATE_RECORD_KEY
S0:1 DATA E_CROSS_FIELD
S0:1 PK   E_CROSS_FIELD
S0:1 -    E_RELATED_RECORD_MISSING
S0:2 DATA E_CROSS_FIELD
S0:2 PK   E_CROSS_FIELD
S0:2 -    E_RELATED_RECORD_MISSING
```

L'errore evento ha field=None e linea realeS0, non coordinate cliniche inventate.
Un secondo test inverte l'ordine schema e prova che traceorder segue lo schema,
non le stringheTraceId o la richiesta. Il JSONL è confrontato byte-per-byte da
JsonlSink reale senza normalizzazione; timestampiniettato uguale e contatori,
fullaudit, inputaudit e schemahash sono confrontati indipendentemente.
Elapsed è tempo monotono realmente misurato: non si altera né si spaccia per
metrica riproducibile. Un preliminare lo ha evidenziato, log preservato.

## Budget e cancellazione osservati

Budget2 termina inPASS1 con le prime due diagnostiche, zero cross valutate;
budget6 termina inPASS2 dopo la prima cross. In entrambi i casi prefisso byte-
esatto dell'illimitato, total_errors reale, truncated ePartial verificati.
Nessuna redistribuzione o quota minima di cross.

Cancellazione aCrossStarted: cinque locali emesse e nessuna cross, Cancelled/
Partial e nontruncated. Cancellazione dal sink durante prima crossS0: termina
le tre regole della riga corrente e si arresta prima della seconda rigaS0,
con otto diagnostiche complessive. Il checkpoint è perrecord, non perassertion;
è preservato, non è stato trasformato nel budget che controlla anche fra regole.
Precancellazione e altri checkpoint rimangono coperti da full_service e
cancellation_progress. Sinkfailure/errori tecnici restano distinti da cancellazione.

## Implementazione dei soli test

model costruisce lo schema sintetico esplicito; request inverte la richiesta e
imposta un budget pubblico Option<u64> verificato dal servizio. service riusa
repository/registry/preparatore veri conClock fisso, senza modificare il servizio.
run usa la reale ValidationService e valida il summary. signature osserva
trace/line/field/code; jsonl usa il sink reale. CancellingSink è un decoratore di
test che cancella il token al callback cross; non cambia codice di prodotto.
I piccoli Vec di fixture/test non sono buffer globali introdotti nel runtime.

Quattro test preliminari finali positivi. Errori preliminari soltanto nel test:
Option<u64> pubblico distinto dal NonZero interno; codice di dominio canonico
E_INVALID_DOMAIN; elapsed reale non deterministico. Nessun golden, algoritmo,
serializer, codice diagnostico o politica di cancellazione è stato cambiato.
Gate finali e propria CI da registrare prima di RESOLVED tecnico.


## Gate locali effettivi

Cinque comandi exit0 con477impronte sorgenti e indice/HEAD invariati: fmt,
Clippyworkspaceall-targetsallfeatures, corelib/tests957pass/0fail/6ignored,
47regressioni allfeatures/0fail/0ignored e boundary/pixel. Log conteggi e impronte
correnti verificati, inventario conservato. Test nuovi4/4 inclusi nelle suite.
CI nativa propria ancora necessaria prima di chiusura tecnica del CNF; non si
attribuisce al nuovo test la CI08 del codice precedente.
