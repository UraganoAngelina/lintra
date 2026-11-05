# T-P17-05 / DEC-17 — valutazione sperimentale

Stato: DONE — valutazione sperimentale completata, adozione non autorizzata.
Ratifica conclusiva dell'autore: `DEC-17-final-ratification.md`, esito D.
CI finale 42038174791 completed/success sullo SHA sperimentale
8a02f55fba06be0e9784465cb8aa6ee347cd583b: sette job richiesti positivi.
Run, job, artefatti scaricati e log conservati in `T-P17-05/native-final/`.
Runtime di produzione sequenziale; nessun codice sperimentale trasferito.

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
**Divergenza documentale da risolvere prima di un'adozione:** se la sequenza
trace→linea→campo→stage→regola è intesa globalmente sull'intero JSONL, l'attuale
sequenziale non la soddisfa: nella fixture con errori locali suT1 e cross suS0,
le diagnostiche locali diT1 precedono le cross diS0. La pipeline reale è ordinata
per passata, poi trace/linea. Lo spike confronta e preserva questi byte effettivi;
non dichiara risolto il conflitto con una lettura globale della specifica e non
cambia l'ordine del prodotto. Non è stata ratificata qui una nuova regola d'ordine.

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
invariati durante i cinque gate; ledger T-P17-05-local-gates.json. La CI finale e A3 sono completate; vedi ratifica conclusiva e ledger di chiusura.

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

## Verifiche di sviluppo e affinamento della fixture

Il filesystem temporaneo `/tmp` è stato rifiutato dalla policy di storage
qualificato; i file sperimentali sono stati spostati nella directory privata sul
filesystem workspace, senza disattivare controlli. Il primo clone dei due gruppi
di trace aveva join ID duplicati: il compilatore li ha correttamente rifiutati;
la fixture usa ora ID flow-unique, senza modificare il compilatore. Un'asserzione
di test sui guasti è stata corretta per rispettare l'unwrapping effettivo
Flow/Trace→Report/Input dell'applicazione.

Dopo i cinque gate locali su41defb9, ulteriore affinamento della sola fixture:
la chiave sorgente punta alla riga precedente, e un'asserzione prova che l'errore
SELF_COMPARE della riga5 dipende da DATA della riga4. Non basta un self-join che
ritorni sempre il record corrente. Ripetuti fmt, Clippy e cinque test mirati prima
della build e delle misure definitive; nessun algoritmo di esecuzione cambiato.
La prima CI42036498302 su41defb9 è intermedia, non qualifica questa revisione.

## A3 — risultati end-to-end Full

Build release Rust1.99.1 (58cf8f9c3, LLVM23.1.1), commit
`8a02f55fba06be0e9784465cb8aa6ee347cd583b`, eseguibile identico fra varianti.
96campioni:8workload×3ripetizioni×(sequenziale,1,2,4worker), ordine ruotato,
più8baseline di equivalenza/cachescaldate.601,631s di processo foreground,
365hash sorgenti e SHA256 eseguibile invariati. Compilazione Rust esclusa;
compilazione schema/preflight/lettura/audit/local/proiezione/merge/indicizzazione/
cross/report/finalizzazione inclusi. Contatore heap off per evitare la contesa
misurata inA1; RSS reale campionato e I/O di processo conservati per ogni campione.

| Record | Carico | Seq s | 1worker s | 2worker s | 4worker s | Speedup4 |
|---:|---|---:|---:|---:|---:|---:|
|10,000|bilanciato, pochi errori locali|0.1014|0.1184|0.1050|0.1002|1.012x|
|100,000|bilanciato, pochi errori locali|1.1495|1.3693|1.1833|1.1141|1.032x|
|100,000|bilanciato, pochi errori locali, proiezioni ampie|1.5517|1.6893|1.5325|1.5115|1.027x|
|100,000|bilanciato, molti errori locali|2.5070|3.9532|3.8992|3.8272|0.655x|
|100,000|sbilanciato, pochi errori locali|1.6336|1.7336|1.8198|1.7164|0.952x|
|500,000|bilanciato, pochi errori locali|6.2408|7.6742|7.0418|6.5584|0.952x|
|500,000|bilanciato, molti errori locali|12.9864|21.1607|20.2771|19.7499|0.658x|
|500,000|sbilanciato, pochi errori locali|8.6725|10.0639|9.7808|9.2767|0.935x|

I nomi “pochi/molti errori” indicano la densità degli errori locali: le fixture
comprendono comunque errori PK, join mancanti/ambigue e cross reali. I conteggi
esatti sono nei summary originali: per500k bilanciati,74.374errori nel caso locale
sparse e1.287.426nel caso heavy. Lo sbilanciamento80/10/5/5 sposta anche il numero
di record sorgente soggetti ai due controlli cross: confrontare speedup dentro
ciascun workload, non attribuire tutta la differenza fra workload allo scheduler.
Le proiezioni ampie conservano gli otto payload perché richiesti da altrettante
asserzioni cross compilate, non come copia opportunistica del record.

Passate/aggregazione, mediana500k bilanciati, secondi:

| Carico | Variante | Pass1 incl. merge | Merge/indice | Pass2 |
|---|---|---:|---:|---:|
| pochi errori locali | sequenziale |3,0353|0|1,7342|
| pochi errori locali |4worker|3,5449|0,9108|1,5057|
| molti errori locali | sequenziale |8,1398|0|3,3074|
| molti errori locali |4worker|15,0908|1,1412|2,7534|

Le mediane di intervalli distinti non vanno sommate come se fossero il medesimo
campione. Il totale include anche preflight/finalizzazione e distruzione contesto.
Il controllo1worker quantifica l'overhead combinato di thread/code/aggregazione:
500k sparse7,6742s contro6,2408s; heavy21,1607s contro12,9864s. Non è una misura
isolata della sola creazione thread. Il merge costruisce gli indici una seconda
volta; emissione ordinata e backpressure fanno parte del costo reale del candidato.
Non attribuire causalità CPU esclusiva senza profiling ulteriore.

Variabilità, min–max dei3campioni500k bilanciati:
sparse sequenziale6,2099–6,2504s,4worker6,4577–6,8012s;
heavy sequenziale12,9437–13,0270s,4worker19,7167–19,9580s.
Per piccoli/medi workload i guadagni mediani circa1–3% convivono con intervalli
sovrapposti e non dimostrano un vantaggio robusto. Nessuna soglia DEC-14 inventata.

## Memoria, temporanei e I/O

Nessuno spool worker:0byte aggiuntivi su disco per tutti i campioni Full. Le code
hanno64messaggi per trace,4trace; proiezioni minime e contesto mantengono il costo
O(N) già richiesto daFull, senza copie dei payload durante il merge. Il report
canonico500k sparse è33.337.870byte, heavy883.126.454byte; input73.500.000byte.
Il file report di riferimento e quello del campione coesistono nella directory
privata di test; non sono uno spool di worker e vengono rimossi esplicitamente.
I/O reale di lettura/scrittura, syscalls e byte fisici è conservato come snapshot
`io_before/io_after`; include le letture/proc del campionatore e cache calde.
Non equivale a throughput su storage freddo.

Massimo RSS campionato del processo nell'intera matrice circa3,04GB decimali.
Il processo conserva arena/cache dell'allocatore tra campioni: ad esempio il caso
dense100k eseguito dopo casi500k parte già con RSS elevato. Le righe raw includono
RSS iniziale e picco: non interpretare il picco come sola memoria viva del contesto,
né confrontare workload diversi come processi freschi. Heap richiesto off è null,
mai zero. Per un'adozione futura servono anche misure di memoria in processi nuovi;
questa limitazione non viene nascosta. Nessun dato clinico grezzo o identificativo
operativo nei file, solo fixture sintetiche. Tutti i TempDir/NamedTempFile chiusi.

## Equivalenza finale e decisione ratificata

96/96confronti JSONL byte-per-byte, hash progress e ogni campo deterministico del
summary superati; i report originali e tutti i diff `/elapsed` sono conservati in
`T-P17-05/Full-linux/results.json`. Le proiezioni alla barriera sono confrontate
nei test separati; il capture è disabilitato nelle misure. Zero/uno/molti correlati,
self-join verso un'altra riga e cross-trace, PK, invalidi locali, contesti vuoti/
malformati, limiti con fallback e guasti tecnici verificati come sopra.
**Cancellazione dinamica: condizione NON superata**, non trasformata inPASS dal
fatto che il test del fallimento atteso termini correttamente.

**Ratifica conclusiva: non adottare il prototipo, esitoD (contratti non dimostrati), con
evidenze anche di beneficio Full insufficiente/regressioni.** Il vantaggio A1
non si traduce in un miglioramento consistente diFull:500k sparse peggiora circa
5,1% e heavy circa52,1% con4worker. I piccoli guadagni su alcuni casi non risolvono
cancellazione, interpretazione dell'ordine globale o scope esclusi dal parallelismo.
Nessuna ADR di adozione, nessuna attivazione selettiva automatica, prodotto
sequenziale. Eventuali approfondimenti LocalOnly o strategie di merge/cancellazione
richiedono una proposta separata e nuove evidenze; non sono implementati qui.

Lo studio è chiuso dopo verifica del gate nativo finale effettivo; nessuna
funzionalità parallela di produzione è stata introdotta. Il percorso CAD resta
indipendente; T-CAD-02 è il prossimo lavoro autorizzato dai prerequisiti reali.
