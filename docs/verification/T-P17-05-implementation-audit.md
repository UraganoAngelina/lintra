# T-P17-05 — audit dell’implementazione reale, 2026-11-05

IN_PROGRESS, prima di progettare/eseguire lo spike. Mandato DEC-17 Aestesa in
DEC-17-ratification.md. Prerequisito17-02:9d641d4/CI41846160280 success,
36scenari di baseline, nessuno speedup parallelo desumibile da quelle misure.
Sorgente analizzata: codice engine/application/context presente in46d574f;
nessuna modifica produttiva per il parallelismo.18-03 oraDONE con asset reali,
CAD01/10 suCI42004833488 indipendente eancora in corso alla prima lettura.

## Pipeline effettiva e corrispondenza col design

FlowValidator è implementato in src/engine/flow_execution.rs; il file
flow_validator.rs ospita la passata cross. validate_with_record_observer esegue
prepare_flow, begin_full e un ciclo di trace preordinati. Crea ProjectionCollector
per trace e chiama TraceValidator.validate_full_path. Solo dopo completamento
osservato di tutti gli input costruisce coverage, emette CrossStarted e chiama
run_cross_record_rules sullo store completo. Nessuna rilettura del raw in passata2.

TraceValidator possiede stato failed e un RecordValidator oReferenceRecordValidator.
BOM Reject precede framing RecordReader/BufReader; buffer riusato; stop/cancel
fra record e trace, notifiche ogni1024 echeckpoint terminali. FullPassOne è
una fase reale distinta da LocalOnly: non sostituire validate_full_pass_one con
validate_local per simulare una passata Full. API delle fasi/collector sono in
parte pub(crate), non tutte pubbliche come negli esempi concettuali DET75–90.
Non cambiarle nel prodotto per accomodare lo spike.

RecordValidator è un owner per trace, failed terminale tecnico, skipped_cross_rules
precompilati. Field pipeline e local rules in ordine, poi identità e contatori.
Il collector della passata1 gestisce PK intra-trace e proiezioni. Record invalido
ma di larghezza corretta può contribuire evidenza proiettata indipendente;
record di larghezza errata incrementa unprojectable/inconclusive e non entra
nello store. Un prototipo che proiettasse soltanto record validi cambierebbe
missing/Unknown e non sarebbe equivalente.

ContextProjectionPlan proviene dal compiler: project_record copia solo valori
fisici dei campi selezionati, identità,trace/line echiavi join risolte/MissingComponent.
Non possiede l’intero record. ProjectedRecord non attesta da solo validità locale.
InMemoryContextStore usa Arc per condividere un payload traBTreeMap di riga e
indici target; insert mantiene matches ordinati perline anche se inseriti fuori
ordine. ContextStore non ha interfacce SQL/persistenza; appartiene a un solo run.

La passata2 verifica prima gli input/namespace, poi il ciclo di emissione reale
è trace(compiled vector)→record(store.iter_trace ordinato)→regole dichiarate.
Il primo ciclo perplan nel file è preflight, non emissione: non inferirne un
ordine rule-major. Locali passata1 precedono le cross passata2. Confrontare con
questa sequenza effettiva, senza ordinare/mescolare le due passate per ottenere
uno speedup. DET73/133/158 descrivono questa separazione; DET79 elenca l’ordine
minimo. Verificare con casi misti anche il confine tra passate e documentare
qualsiasi ambiguità di interpretazione, senza cambiare il report canonico.

## Stato mutabile e condivisione

FlowState possiede store, insieme invalid(trace,line), coverage, evidenze PK,
evidenze cross e totali. ProjectionCollector aggiunge keys(first_line) pertrace,
sink/summary eobserver storico sincrono borrowing del buffer. FullCrossObserver
muta summary,invalid/evidenze eobserveresterno. Non condividere questi owner tra
worker; worker non invocano observer storico né PersistentContextRepository.

CompiledFlowSchema è letto immutabilmente. PreparedReferenceData espone lookup
Arc<dynLookupSet:Send+Sync>; provare effettivamente Send/Sync nel codice dello
spike per schema/prepared/evidence, non assumere che la sicurezza dei dati di
lookup renda condivisibili gli observer mutabili. Ogni reader/buffer/validator/
observer/counter intermedio deve essere posseduto dal suo worker.

ValidationSummaryBuilder non è un contatore sommabile ciecamente: transizioni
valid→invalid avvengono anche per PKduplicate e cross multiple; invalidHashSet
impedisce doppio conteggio del record. Solo un coordinatore può committare
conteggi,errori,ordine/progress e inserimenti nello store finale.
ProgressObserver è FnMut sincrono senza obbligo Send/Sync/GUI runtime: notifiche
rimangono sul chiamante, non nei worker. CancellationToken è Arc<AtomicBool>
cooperativo Send/Sync; non sincronizza i payload e non resetta.

ErrorBudget è Arc con maximum/AtomicBoolreached; i contatori hanno comunque un
solo owner. observe avviene dopo sink acknowledgement econtatori checked.
Condividerlo con più emettitori sicuri da race non prova il prefisso normativo.
FlowValidator.validate pubblico resta unlimited e rifiuta budget sul builder;
application usa l’entry pub(crate) controllata. Esperimento budget non richiede
né autorizza un nuovo metodo pubblico.

InputAuditSession produce metadati dai reader reali e controlla EOF/prefix/hash.
Un lettore perworker è possibile; registro/finalizzazione e ordine audit vanno
coordinati. Non fondere copie costruite a mano come falsi inputComplete.
Lettura speculativa di trace successivi può rendere diversi gli audit al cancel:
misurare e registrare il problema, non nascondere bytes_read né fingere EOF.

## Summary e metadati variabili — problema documentato prima del confronto

ValidationService ha Clock iniettabile per started_at; SummaryBuilder misura
elapsed con Instant::now/elapsed internamente. Il contratto effettivo del SummaryBuilder e la qualifica
precedente distinguono questo tempo monotono dai contenuti deterministici.
Un FixedClock non rende elapsed identico tra varianti. Non mutare API o azzerare
silenziosamente i JSON per far passare il confronto.

Strategia da verificare nello spike: conservare raw summary di entrambe le
varianti e confrontare tutti i campi contrattualmente deterministici, registrando
separatamente il diff esatto del solo elapsed legittimamente variabile, che
resta intatto negli originali. Started_at controllato mediante clock già previsto;
input/hash/ordine/coverage/conteggi/codici/skipped/nondefinitive/versioni non sono
normalizzabili. JSONL diagnostico va confrontato byte-per-byte senza filtri.
Tempi misurati saranno una metrica, non prova di equivalenza semantica.

## Candidato da progettare dopo l’audit

Isolamento su branch sperimentale, modulo di test/benchmark che riusa i confini
interni reali, senza cambi API o runtime produttivo. Worker pertrace fino1/2/4,
canali bounded e passata2 seriale dopo barriera; niente scheduling dentrotrace.
Proiezioni minime e diagnostici perrecord limitati dallo schema, non una Vec
globale di errori. Trasferimento di ownership al coordinatore, nessuno spool
sensibile non protetto. PK e contesto coordinati senza spostare controlli
normativi o raw completi nei messaggi. Costo aggregazione/reordering incluso.

Punti da dimostrare, non già risolti: invarianti di max_errors e speculative I/O,
cancel fra trace e durante worker bloccato sul bounded channel, errori/panic/
sink failure senza thread residui e senza false Complete, equivalenza lookup/
join zero/uno/molti intra/cross-trace, metadati progress/inputaudit. Possibile
fallback sequenziale dichiarato su opzioni non dimostrabili; non chiamarlo
speedup parallelo. Nessuna conclusione prestazionale prima delle misure.


## Ulteriore riscontro audit: mutex globale durante la lettura

InputAuditSession.state èArc<Mutex<State>> con slots di tutti i reader e un
fallimento globale. AuditedReader::read mantiene quel lock durante read ehash:
non è un mutex per singolo stream. wrap/finish preservano ordine registrazione,
non riordinano pertrace. Condividere una sessione tra worker serializzerebbe I/O
e hashing e registrerebbe gli input in ordine di apertura, potenzialmente variabile.
Questo è un vincolo reale scoperto dall’analisi, non un bug corretto nel prodotto.
A1 usa sessioni autonome per validazioni LocalOnly indipendenti; A2 deve progettare
un’aggregazione di stati audit reali, non auditComplete assemblati fittiziamente,
senza cambiare l’API pubblica. Ogni eventuale adattatore resta solo nello spike.
Hardware osservato Linux:i7-1185G7,4core/8thread, memoria15.688MiB e circa9.089MiB
available all’osservazione, swap preesistente2.260MiB. Niente promessa di RSS per
worker o misure di speedup già raccolte.1/2/4worker sono sperimentabili.
