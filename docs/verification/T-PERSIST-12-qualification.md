# T-PERSIST-12 — qualifica sintetica della persistenza core

4 novembre 2026, IN_PROGRESS. [Preflight](T-PERSIST-12-preflight.md),
[operazioni e limiti](../PERSISTENCE_OPERATIONS.md). Prerequisiti09/10/11
qualificati con CI reale.17-02 è indipendente e ora DONE su CI41846160280.
Nessun DONE o gate Windows del nuovo codice anticipato.

## Fixture condivisa, schema e protezione

`tests/support/persistence_profiles.rs` contiene solo dati e costruzione di
carichi sintetici. `Shape` seleziona arità PK dei cinque trace, rispettivamente
[3,3,4,4,2] e[2,2,2,5,5], corrispondenti alle forme elencate da ADR-PERSIST-001.
`label` rende esplicita la natura sintetica. Non sono schemi regionali: larghezze
8byte, campi generici, payload16byte, regole assenti e modalità LocalOnly.
Non si eseguono controlli temporali CAD o si dichiara conformità SIOC completa.

`Provider` accetta solo il Key ID sintetico e restituisce32byte pubblici di test,
con Zeroizing attraverso il tipo esistente. Non genera materiale operativo.
`namespace` usa una coppia tipizzata esplicita. `Fixture` contiene compiled schema,
hash della sorgente JSON sintetica e cardinalità. `new` limita i volumi5–300000,
compone i cinque trace/PK dichiarative e chiama il vero SchemaCompiler; nessun
oggetto compiled forgiato o parsing regionale implicito.

`row` costruisce una riga transiente, estrae PK e projection tramite il vero
build_join_key e invoca IdentityProtector con namespace/flusso/versione/trace
espliciti. Il port riceve solo StoredIdentity attestati. Il secondo payload cambia
il fingerprint ma conserva la PK; nessun raw byte viene aggiunto al repository.
`snapshot` distribuisceNrighe sui cinque trace, registra conteggi e hash input,
UTC fisso e fingerprint sintetico, poi protegge la SourceSubmission. Il fingerprint
originale contiene una sentinella proprio per verificare che non venga archiviato.
`publish` usa il port generico begin/stage/commit e generazione attesa: è un helper
di qualifica storage, non il servizio UI di approvazione o una policy normativa.

## Benchmark reale e delimitazione delle misure

`benches/persistence_profiles.rs` è un target opt-in senza nuovi crate. La normale
suite stampa le istruzioni ed esce senza benchmark estesi. `--output` obbligatorio,
create_new sul risultato finale prima del lavoro; `--smoke` seleziona100record,
altrimenti10k/100k/300k. `save` rifiuta parti esistenti. Nessun output contiene
righe, chiavi o payload grezzi; stdout contiene solo backend/forma/conteggio.

`OwnedChild` possiede ogni processo, lo termina e raccoglie anche in unwind;
`child` attende sequenzialmente successo con deadline600s. Timeout/failure sono
errori, non misure. Nessun processo in background indipendente o agente delegato.
Ogni backend/forma/volume riparte in un processo nuovo, evitando che il picco RSS
di un caso precedente contamini il successivo.

`peak_bytes` legge VmHWM Linux o PeakWorkingSetSize Windows con buffer/API nativi
qualificati. È il picco del processo, include C/Rust/fixture e overhead, non la
metrica heap richiesto di17-02. Non sono confrontabili come valori equivalenti.
`sample` prepara il fake memory con audit esplicito oppure il vero SQLite locale;
non sceglie un backend runtime o introduce storage alternativo nel prodotto.
Genera e pubblica due revisioni complete, misura separatamente preparazione e
commit, verifica100lookup con percentili, legge la prima revisione e ne verifica
conteggio/record, controlla il namespace vuoto separato e misura la dimensione DB
solo dopo chiusura. Il carico memory è un fake transazionale, senza durabilità.

`restart` usa un altro processo e apre SQLite con la stessa chiave: verifica
generazione2, source della revisione1, un'identità corrente per ciascuno dei cinque
trace e namespace separato. RSS/reopen sono misurati nel processo fresco senza
caricare preventivamente l'intero storico. Il JSON lega harness/helper/adapter a
SHA256, toolchain, commit, OS, modalità optimized e soglie null. Nessuna soglia
DEC14 o conclusione operativa sui dati regionali è introdotta.

## Test privacy e ripristino

`tests/persistence_qualification.rs`: `has_sentinel` cerca marcatori sintetici
noti di payload/PK/fingerprint e materiale test, sia byte originali sia esadecimale
(case-insensitive) usato dai DTO; controlli positivi/negativi
impediscono uno scanner vacuo. `scan` legge i file realmente prodotti nella root
privata, dopo ciascun commit e dopo chiusura, senza ignorare i journal presenti.
Il primo test usa entrambe le forme, confronta identità/fingerprint fra revisioni,
scansiona storage/audit/Debug diagnostico e verifica snapshot precedente e scope.
La sentinella non è un riconoscitore generale di dati sanitari: il documento
operativo distingue metadati leggibili e report dalla protezione dei valori.

Il secondo test chiude SQLite e copia i byte con digest in un sandbox privato.
La root di destinazione è creata esplicitamente dal backend solo per predisporre
i suoi permessi nativi, poi il relativo database sintetico chiuso è sostituito
con la copia. Verifica audit, snapshot1 e snapshot2; chiave diversa con stesso ID
rifiutata. Una copia alterata cambia digest ed è rifiutata all'apertura; l'originale
resta identico per byte. Nessuna API prodotto di restore o sostituzione dello
storico attivo, nessuna riparazione, nuova chiave o fallback implicito.

Primo tentativo compile: ProtectionAudit non implementa PartialEq; corretto il
confronto sui DTO JSON serializzati, senza cambiare il tipo pubblico. Primo run
usava tempdir di sistema su filesystem non supportato: corretta la sola fixture
per usare target sul filesystem qualificato, nessuna protezione disabilitata.
Entrambi i test ora passano, Clippy mirato exit0 e smoke quattro casi/restart
positivo. Questi non sostituiscono i gate finali o le prove Windows future.

## Conservazione e rischi residui

PERSISTENCE_OPERATIONS documenta nessun purge/scadenza implicita, snapshot e
occorrenze conservati, copia solo dopo chiusura verificabile e ripristino isolato
con originale preservato, verifica integrità/chiave/provenienza e controllo umano
della freschezza. Un backup vecchio può essere autentico: nessun antirollback
esterno viene inventato. HMAC non cifra SQLite o metadati, non anonimizza e la
chiave comune aumenta l'esposizione condivisa. Retention, cifratura e backup
operativo completo/dati reali restano aperti secondo ratifica04novembre.

Misure complete ripetute e gate finali in corso. Nessuna modifica runtime,
HMAC/canonizzazione/formato SQL/norma/installer o baseline regionale. Le due
forme hanno uguale arità totale16, distribuita diversamente fra cinque trace:
poiché le identità persistite sono HMAC a lunghezza fissa, non si deve attribuire
alle differenze di tempi un significato normativo o clinico.


## Canali del binario reale

Aggiunto a tests/cli_history.rs un test dedicato: riusa la fixture con file chiave
privato e lo stesso binario effettivo, amplia il campo sintetico a una sentinella
16byte, attende la preview reale e ne invia la conferma esatta. Scansiona stdout,
stderr, audit JSONL, summary storica e file effettivi dell'archivio. Poi sostituisce
solo il materiale sintetico del file chiave, conserva il Key ID e verifica errore
tecnico redatto senza mutazione del database. Il file chiave è input esplicito:
è escluso dai file output da scansionare, non si pretende che non contenga i32byte.
Il test è scritto ma non ancora eseguito durante le misure prestazionali, per
non interferire con esse; sarà incluso nel gate standard e mirato all-features.


## Riproduzione e review dei confini

```sh
cargo test --test persistence_qualification --locked
cargo test --test cli_history --locked synthetic_record_and_key_sentinels_never_reach_cli_logs_or_historical_audit -- --exact
cargo test --bench persistence_profiles --locked -- --output target/profiles-smoke.json --smoke
cargo bench --bench persistence_profiles --locked -- --output target/profiles-full.json
```

Ogni output deve essere nuovo: non sovrascrivere un dossier precedente. Il caso
completo contiene12misure (tre volumi × due forme × due backend) e due snapshot
per misura. La seconda esecuzione estesa locale usa direttamente lo stesso
eseguibile ottimizzato prodotto dalla prima compilazione. Tempi di build e IO
JSON sono esclusi dalle misure di commit/lookup; non si avviano altri test o
compilazioni durante le misure. La macchina resta un ambiente di sviluppo,
non un banco dedicato certificato: niente SLA o soglie dedotte dai tempi.

Review core: diff runtime src vuoto. Ricerca eseguita dei literal SIOC/CAD in
persistence, engine e application/historical_validation.rs senza corrispondenze,
exit1 di rg atteso e nessun errore di lettura. Dossier
T-PERSIST-12-core-boundary.json. Le due forme sono soltanto nel supporto test e
benchmark; non vengono inserite policy normative o rami per flusso nel motore.


## Prima esecuzione completa qualificata

12casi ottimizzati terminati con successo; hash harness/helper/adapter verificati
contro i sorgenti e JSON originale conservato in T-PERSIST-12-benchmark-run1.json.
Seconda esecuzione in corso sullo stesso binario, senza ricompilazione.

A300k identità per snapshot × due revisioni, SQLite usa3.347.546.112byte nella
forma SIOC e3.347.189.760byte nella forma CAD. Picco processo rispettivamente
509.378.560 e509.562.880byte. Commit1/2:231,14/129,37s e229,68/127,58s.
Lookup p95 circa226/215microsecondi su100campioni. Il processo fresco di restart
verifica tutti cinque i trace e generazione2, apertura12,86/13,11s e picco circa
9,8MB, senza caricare l'intero snapshot. Il fake memory raggiunge circa1,78GB;
non offre la stessa durabilità/sicurezza e i tempi non giustificano sostituire SQL.

Sono costi significativi di questo formato/carico: non vengono nascosti né
trasformati in un test pass/fail su una soglia DEC14 mai approvata. Il DB contiene
600k record storici e indice corrente, non righe grezze. Dimensione e tempi non
sono estrapolazioni certificate di archivi regionali: servono a dimensionare
prove successive. Non vengono cambiati PRAGMA, indici, formato o garanzie per
far apparire migliori i risultati. Non è dichiarata prontezza per dati reali.


## Ripetizione completa e test mirati finali

Seconda run estesa exit0, stesso eseguibile:24casi complessivi, due revisioni per
caso. Verificati matrice dei volumi/backend/forme, optimized=true, impronte dei
quattro sorgenti e dimensioniDB identiche fra ripetizioni per ogni caso SQLite.
Originali T-PERSIST-12-benchmark-run1/2.json e riepilogo measurements.json.
A300k, commit1/2 SIOC227,33–231,14s /120,13–129,37s; CAD224,46–229,68s /
117,28–127,58s. Picco SQLite509,38–509,89MB e lookup p95211–226microsecondi.
Questi sono intervalli delle sole due esecuzioni, non intervalli di confidenza.

Negativo no-clobber eseguito sul vero binario: output preesistente rifiutato,
exit101 atteso e digest del risultato invariato; ledger no-clobber.json.
Scanner aggiornato byte/hex: due test privacy/closed-copy positivi. Test CLI
reale delle sentinelle anch'esso passato, inclusi approvazione postpreview,
stdout/stderr/audit e rifiuto della chiave errata senza alterazione database.
Nessun fallimento mirato residuo. Gate workspace finali ora in esecuzione.

Misure estese eseguite su Linux. Lo smoke nativo futuro su Windows qualificherà
API memoria, transazioni, dimensione e reopen con100record: non sarà presentato
come benchmark Windows300k. I precedenti dossier05/11 qualificano backend e
trasferimenti nativi; questa task richiede comunque la CI sul proprio codice.


## Gate locali finali conclusi

Dieci comandi exit0,402impronte sorgenti/manifest invariate; fmt workspace,
Clippy all-targets default/all-features, workspace standard873pass/111riepiloghi,
0fail/6ignored; mirati all-features2privacy/copia +1CLI sentinelle; smoke quattro
casi in entrambi gli assetti; dependency-boundary eactionlint1.7.7 positivi.
Gli smoke sono eseguibili con assertion, non suite libtest. Nessuna ripetizione
locale dell'intero workspace all-features necessaria per queste sole fixture:
la CI completa nativa resta richiesta e qualificherà anche tutte le feature.
Ledger completo T-PERSIST-12-local-gates.json. Nessun codice modificato durante
i controlli. IN_PROGRESS fino a pubblicazione, CI propria e verifica dei due
artifact di misure smoke; non si riutilizza il successo17 per certificare12.
