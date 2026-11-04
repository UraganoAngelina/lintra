# T-P17-04 — limite operativo ratificato DEC-16 A

4 novembre 2026, IN_PROGRESS. [Ratifica dell'autore](DEC-16-MAX-ERRORS-01-ratification.md)
e [proposta A](../proposals/DEC-16-MAX-ERRORS-01.md). T-PERSIST-12 attende il
proprio gate41857167059;17-04 è indipendente, autorizzata dopo la scelta dell'autore.
Il preflight18-03 non ha modificato codice/versioni o avviato release.

## Contratto e confini

ValidationOptions.max_errors None resta illimitato; Some(N>0) è un budget unico
per tutta la richiesta supportata (LocalOnly/Trace oppure Full/Flow), non per
campo, record, trace o passata. Some(0) viene rifiutato prima degli input.
Contano soltanto ValidationError normativi accettati dal sink: niente errori
sintetici per spiegare il limite, warning/reference/Unknown fuori dal budget.

Dopo N emissioni, CompletedInvalid con truncated=true e completezza Partial,
anche sull'ultima riga o sull'ultima valutazione. Non si legge il resto solo per
completare hash/contatori. File che terminano prima di N mantengono i normali
esiti e non sono troncati. Un input può avere hash completo (Full prima passata
finita) e controllo regole parziale (limite nella seconda passata).

Non sono introdotti RunStatus, campi persistenti, algoritmi HMAC, norme o
formatiSQLite nuovi. In JSON LocalOnly senza audit resta la forma storica priva
del campo completeness: truncated=true espone l'interruzione e l'API restituisce
Partial. Full/reference conservano il campo completeness già previsto. Nessun
nuovo flag CLI o selettoreGUI; i client attuali continuano a richiedere None.
Il budget non è un limite di record/memoria: Full può costruire il contesto di
tutti gli input prima di incontrare errori nella seconda passata.

## Stato privato e contatore autorevole

`domain/error_budget.rs`: ErrorBudget possiede un Arc<State> con massimo
NonZeroU64 e latch AtomicBool. La configurazione positiva è immutabile; la flag
passa soltanto da false a true. Atomic preserva Send/Sync del builder; non crea
produttori concorrenti o parallelismo. None non alloca un Arc. `observe` viene
chiamata dal contatore dopo sink e incrementi controllati; `reached` offre ai
checkpoint la stessa flag senza duplicare un secondo conteggio divergente.
ExecutionStop è crate-private, non serializzato, distinto da cancellazione e
dalle enum pubbliche di completamento del motore.

`ValidationSummaryBuilder.with_error_limit` configura il builder fresco dopo
preflight. `error_budget` clona solo l'handle per la seconda passata e
`error_limit_reached` legge il latch. `error_emitted` mantiene validazione codice
e overflow, poi registra il raggiungimento e marca truncated. Una failure del
sink non arriva al contatore. `cancel` conserva il limite già osservato come
prima causa controllata; cancellazione osservata prima resta Cancelled. Errori
tecnici effettivamente incontrati in qualunque finalizzazione restano Err.

## Emissioni locali e lettura del prefisso

`FieldPipeline.error_limit_reached` espone il checkpoint senza rompere il borrow
del contatore. RecordValidator controlla prima di ogni campo e regola successivi:
nessun errore N+1, nessuna valutazione aggiuntiva, classificazione invalid della
riga corrente perché contiene già una violazione nota. Le emissioni di lunghezza
restano sink→contatore→record invalid, con le cause tecniche originali.

TraceRunCompletion è un DTO privato con causa e numero di righe. La nuova
`validate_controlled_path` riusa apertura audit, framing, BOM e buffer della
stessa implementazione `run`; non imita un errore I/O o un Cancelled per uscire.
Il loop controlla il budget prima di leggere altro e subito dopo il bookkeeping
della riga corrente. Notifica l'ultimo RecordsProcessed ma nessun TraceCompleted
falso. Un record già letto conserva il checkpoint atomico di cancellazione
preesistente; il limite può arrestare l'emissione fra campi/regole.

Le firme/varianti pubbliche TraceCompletion, TraceEvent e TraceControl non
cambiano. Gli adapter raw pubblici restano illimitati (non ricevono una request
ValidationOptions); la configurazione del budget è privata al servizio. I wrapper
rifiutano un eventuale uso interno scorretto prima dell'I/O, non convertono un
limite reale in Completed. Le opzioni pubbliche del servizio sono il percorso
per richiedere il nuovo comportamento.

## Full e compatibilità delle API pubbliche

FlowRunArtifacts mantiene privatamente ExecutionStop e le evidenze. Il wrapper
pubblico unlimited converte per movimento nei FlowValidationArtifacts e
FlowCompletion originali: nessuna enum pubblica estesa o copia delle proiezioni.
Il servizio usa direttamente il percorso controllato. FlowValidator termina
prima di altri trace o della passata cross se la prima passata ha raggiunto N.

ProjectionCollector registra la riga corrente e la sua invalidità, ma non estrae
ulteriori PK, non chiama il produttore storico e non crea proiezioni dopo il
limite. Una PK non esaminata è inconclusive; un record a larghezza corretta non
viene contato falsamente come malformato/unprojectable. Se il limite nasce dal
controllo duplicati, contatori PK e invalidazione della riga già classificata
valida sono completati prima dello stop, senza proiezioni successive.

CrossPassControl privato porta lo stesso ErrorBudget al loop cross mentre
FullCrossObserver possiede mutabilmente il contatore. Ogni violazione è prima
accettata dal sink, poi conteggiata dall'observer; solo dopo successo delle
callback si termina prima della regola successiva. Un observer/sink error dopo
N non viene nascosto. `run_cross_record_rules` restituisce una causa controllata
privata, non un booleano Cancelled riutilizzato per il limite.

FlowState.finish somma soltanto valutazioni omesse su righe già note: nessuna
stima delle righe mai lette e nessun Unknown/errore inventato. Non emette eventi
Completed/Cancelled falsi sul limite. Il normale builder finish_full imposta
Partial da truncated e valida contatori/evidenze esattamente come prima.

## Wiring applicativo e storico

I due preflight accettano positivi e rifiutano zero tramite la categoria tipizzata
UnsupportedMaxErrors esistente, con messaggio aggiornato; nessun nuovo errore
serializzato. Il servizio configura una sola volta il budget, conserva i port
sink/progress/cancellation e usa gli stessi audit/clock/schema risolti. La fase
FinalizingReport può ancora fallire tecnicamente. RunCompleted segnala fine
orchestrazione, non conformità completa del file.

HistoricalValidationService mantiene il guard esistente di validità/completezza.
Una validazione troncata è ValidationOnly, mai Preview approvabile. Il test SIOC
prova il contrasto: budget non raggiunto→review completa e approvazione separata;
limite raggiunto→nessun diff approvabile, nessun nuovo invio o cambio generazione,
snapshot precedente identico. Nessun trattamento SIOC/CAD hardcoded.

## Verifiche eseguite e in corso

Check libreria positivo. Primi mirati:10max/preflight6/service12/SIOC14 pass;
poi aggiunti prefix/progress, warning reference e review positiva non troncata.
I test max_errors coprono molti errori, N=1, più errori per campo/riga/regola,
budget globale trace/passate, confine EOF, input successivo non aperto, nessuna
cross-evaluation da contesto parziale, Unknown, cancellazione in entrambi gli
ordini e failure sull'N-esimo errore/summary. Verifica JSON degli stati esistenti.

Primi tentativi fixture corretti senza cambiare runtime: FieldDiagnostic usa id,
InputAuditCompletion ha Incomplete (non Prefix); clone superfluo nel test eliminato
per Clippy. Un'asserzione sul campo JSON completeness era errata per LocalOnly
storico: mantenuta omissione esistente. Una modifica della coda dopo il limite
non implica un guasto osservato su un audit incompleto: il test di I/O usa invece
troncamento del file prima della lettura, che produce un vero mismatch a EOF.
Non viene introdotto uno stat/read extra per rendere vera un'aspettativa inventata.

Clippy mirato all-targets ora positivo. Nuovi mirati/finali e CI propria ancora
necessari; nessun DONE17-04 anticipato. Runtime/proposte/formati estranei non
modificati. Fonte DEC-16 ratificata e aggiornamenti piano/DET consultabili.


## Gate locali finali e checkpoint

Otto comandi exit0,404impronte sorgenti immutate: fmt workspace, Clippy default/
allfeatures, workspace887pass/112riepiloghi, core-no-bundle844pass,42mirati con
allfeatures, smoke validation quattro scenari e dependency-boundary.0fail/6ignored
nei run completi. Ledger T-P17-04-local-gates.json. Golden non modificati.
Il contatore skipped_rules conserva i controlli omessi già esposti dall’audit;
non diventa una stima di tutti i campi/vincoli del resto del file. Truncated
segnala l’interruzione; i contatori righe/errori descrivono quanto osservato.

Residui: commit/push e CI propria, con regressione completa allfeatures nativa.
La CI12 positiva non certifica il nuovo codice17. Ultimo mandato dell’autore:
fermarsi dopo17-04; completare questi gate, poi STOP, nessuna task successiva.
