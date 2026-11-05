# T-P17-05 / DEC-17 — valutazione sperimentale

Stato: IN_PROGRESS. A1 misurata; A2 prototipo isolato implementato, A3 e gate
finali da completare. Nessuna adozione nel runtime. Ratifica A estesa dell'autore
in `DEC-17-ratification.md`; proposta originaria conservata come storia.

## Architettura e indipendenza

Ricostruzione prima del codice: `T-P17-05-implementation-audit.md`.
FlowValidator esegue FullPassOne tramite TraceValidator/RecordValidator,
ProjectionCollector costruisce PK/identità e proiezioni minime secondo il piano
compilato; ContextStore effimero distinto da SQLite, passata2 sequenziale via
CrossRecordValidationPlan. Invalidi di larghezza corretta conservano evidenze
indipendenti; righe malformate rendono il contesto esplicitamente parziale.
Lo schema compilato e le reference preparate hanno bounds Send+Sync verificati;
reader, summary, sink, observer e audit sono mutabili, non liberamente condivisi.

Dettagli del candidato: branch `spike/dec17-selective-pass1`,
`spikes/dec17-selective/A1_IMPLEMENTATION.md` e `FULL_IMPLEMENTATION.md`.
Solo cfg(test), worker std limitati fra trace, nessun parallelismo intra-trace,
nessuna API/CLI/UI/dipendenza nuova. Le regole compilate non sono riclassificate. Il formato attuale ammette soltanto
RawRecordKeyUniquenessScope::Trace: la verifica PK resta nel collector originale
della prima passata e nel medesimo trace intero. Non è stato dedotto un perimetro
locale dal nome del trace; nessuna unicità globale o relazione storica viene
spostata in un worker.
Il coordinatore mantiene ordine e callback del sink, aggregazione e barriera del
contesto prima della passata2 originale. Nessun errore globale accumulato, nessuno
spool worker e nessun record grezzo conservato come proiezione.

L'ordine effettivo sequenziale è phase-major: diagnostiche locali/PK di tutti i
trace in passata1, poi cross trace/linea/regola in passata2. Il candidato preserva
esattamente quell'ordine, non riordina per eliminare la separazione tra passate.
La formulazione generale di DET79 va letta con questo contratto implementato e
qualificato, non reinterpretata nello spike per ottenere un confronto favorevole.

## Metodo e A1

Hardware verificato: Intel i7-1185G7,4core/8thread logici, RAM15.688MiB,
Linux CachyOS kernel7.3.1-1-cachyos, workspace su filesystem qualificato. Nessun
carico operativo reale o dati clinici: fixture sintetiche riproducibili.
Build release identica tra varianti, compilazione esclusa, un processo foreground
alla volta; schema condiviso, nessuna ottimizzazione per-thread dei Regex.
RSS campionato2ms distinto dall'heap richiesto. Cache input scaldata dalla baseline;
non sono numeri di cold storage. DEC-14 non riceve soglie inventate.

A1:84campioni,7workload,3ripetizioni e baseline/1/2/4worker; più96campioni di
controllo on/off/off/on del contatore heap. Evidenze complete in
`T-P17-05/A1-linux-metered/` e `T-P17-05/A1-linux-control/`;
interpretazione e tabella del controllo in `T-P17-05/A1-control.md`.
Sorgenti invariati, JSONL identici, summary originali conservati. L'unico diff
ammesso è `/elapsed`, variabile per contratto monotonic Instant già documentato
prima del benchmark; nessuna riscrittura, omissione o azzeramento del report.

Con500.000record bilanciati e contatore heap off, mediana A1 sequenziale0,909s
contro0,346s con4worker (pochi errori,2,63x); molti errori6,837s contro2,857s
(2,39x). Il controllo mostra che il contatore atomico condiviso distorce soprattutto
il carico ad alta emissione. Le prime misure tracciate mostrano regressioni con
trace sbilanciati: non trasferire lo speedup bilanciato a ogni configurazione.

## Equivalenza A2 e limite vincolante

I confronti finora verificati coprono run completati, diagnostiche JSONL byte per
byte, ogni campo deterministico del summary e ogni proiezione alla barriera
(identità,trace,linea,valori,chiavi join). Regole cross fra trace e nello stesso
trace, zero/uno/più correlati, PK duplicate, proiezioni ampie, target vuoto e
malformato. Gate locali completati: fmt, Clippy workspace/alltargets/allfeatures,904test
workspace passati/0falliti/8ignorati,24test Python e controllo confini. Sorgenti
invariati durante i cinque gate; ledger T-P17-05-local-gates.json. La CI nativa
e le misure A3 restano pendenti.

max_errors configurato e pre-cancellazione usano il percorso originale e ne
mantengono il prefisso; il candidato non dimostra la loro esecuzione parallela.
Reference dipendenti e observer storici usano anch'essi fallback esplicito.
Sink failure e I/O falliti restano errori tecnici; tutti i worker vengono raccolti.

**La cancellazione dinamica NON è equivalente**: il prototipo abortisce con errore
tecnico e nessun summary anziché restituire Cancelled/Partial. Il test registra
questa condizione fallita; non la promuove a contratto valido. Non pubblica un
contesto incompleto come completo e non entra in passata2. Risolvere correttamente
richiede checkpoint per-record riconosciuti dal coordinatore per conciliare
prefisso canonico, conteggi e audit reali del lavoro speculativo. Non alterare gli
audit osservati o scartare campi per nascondere la divergenza.

## Decisione di adozione

Nessuna adozione autorizzata. Qualsiasi proposta successiva richiede misure A3 e
garanzie complete; la divergenza di cancellazione impedisce già di proporre questo
prototipo per la produzione. La decisione finale dello studio verrà registrata
insieme ai numeri Full e ai gate effettivi, senza inferirla dai soli worker A1.
