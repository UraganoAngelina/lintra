# T-P17-02 — profilazione misurata prima delle ottimizzazioni

4 novembre 2026, **DONE** (CI41846160280 qualificata; chiusura in fondo). Al preflight T-P17-02 era indipendente da
T-PERSIST-10, allora in attesa del gate41822083629, ora DONE. Nessun prerequisito
aggirato:
T-P17-01 DONE su 485c98f, CI40476599897 Linux/MSVC e confronto JSONL positivi,
due benchmark ottimizzati completi già conservati. ARCH §33, DET §113 e prompt
17-02 applicati, single-agent.

Prima modifica limitata a `benches/validation.rs`: aggiungere conteggi di
allocazioni riuscite/reallocazioni/byte richiesti cumulati e scenario isolato
per `build_join_key`, mantenendo tutti gli oracle Full/projection. I vecchi tempi
17-01 non sono un confronto diretto: cambia il costo dello strumento. Usare
nuove baseline e candidati con **stesso harness**, toolchain e condizioni.

Possibile costo da misurare: costruzione chiave riestrae ogni componente logica
in due passate, prima per calcolare la riserva esatta e poi per codificare. Non
è ancora un bottleneck dimostrato, né una ragione per cambiare tipi pubblici,
firme, ordine errori o byte. Nessun core modificato prima delle misure.

Niente soglie inventate DEC14, parallelismo DEC17, nuovo crate o profiler
installato. La metrica resta heap richiesto Rust, non RSS. Source hash separati
identificano harness, helper schema e codice delle chiavi, anche fra tentativi
nella stessa working tree. Nessuna norma o dato regionale reale coinvolti.

Gate futuri: baseline ripetuta 10k/100k/1M, eventuale candidato con confronto
prima/dopo (scartare se non migliora), byte/golden invariati e regressioni reali.
Nessuna misura o conclusione prestazionale anticipata in questo preflight.

## Strumento e oracle

Il formato benchmark passa a 2, aggiungendo un quarto scenario e metriche senza
rimuovere i precedenti risultati Full/projection. `HeapTraffic` serializza il
numero di allocazioni/reallocazioni riuscite e la somma delle dimensioni
richieste. La dimensione target intera di realloc viene contata anche se il
sistema estende in posto: non è una misura di copie effettive o memoria viva.
`requested` usa solo atomiche, senza allocazioni; viene chiamata dopo successo
System e soltanto durante la finestra `measure`. Quest'ultima azzera i contatori
prima del timer e acquisisce i risultati dopo aver disabilitato il meter.

`join_key` compila la fixture prima del timer e controlla i byte attesi
(prefisso big-endian di lunghezza 8, poi `00000001`). La misura black-boxa gli
input e distrugge ogni chiave. L'oracle dei contatori impone esattamente una
allocazione da 16 byte per iterazione, zero reallocazioni e picco 16 byte: verifica
anche che il meter non includa compilazione/serializzazione. Il vecchio smoke
resta opt-in e conserva gli oracle sui validi/errori/Complete. Nessuna nuova
asserzione su velocità, hardware o soglia di accettazione.

Smoke locale eseguito con successo prima delle baseline; contatori/byte corretti.
Tre baseline release in esecuzione sequenziale, senza processi in background.
Le vecchie misure 17-01 non vengono mescolate alle nuove per dichiarare un guadagno.

## Valutazione preliminare del candidato

Le prime due baseline mostrano che la costruzione isolata della singola chiave
richiede 0,115–0,132 s per 1M chiamate, contro 8,90–9,12 s Full valido. Sono scenari
isolati: non si sommano e non dimostrano una percentuale CPU interna al Full.
L'ipotesi di eliminare la seconda estrazione logica per chiavi monocomponente
non è quindi un bottleneck dimostrato del servizio. Non si introduce il relativo
ramo speciale nel core sulla base della sola lettura del sorgente.

Le metriche rendono invece visibile la pressione delle allocazioni: 19.668.870
allocazioni Full valido e 11M nella projection isolata per 1M record. Le proiezioni
possiedono tre campi e mappe secondo il contratto pubblico esistente; cambiarne
rappresentazione/lifetime richiederebbe una proposta mirata e misure proprie,
non una riscrittura opportunistica in questa task. Il picco Full circa 903 MB è
riferito alla fixture sintetica e al ContextStore effimero, non a SQLite o a
carichi regionali reali. Nessuna soglia operativa, budget o idoneità dedotti.

Decisione implementativa corrente: completare le tre baseline e qualificare lo
strumento migliorato; nessuna ottimizzazione runtime senza evidenza sufficiente.
Non viene dichiarato alcun miglioramento percentuale prima/dopo. Le misure
precedenti hanno strumentazione diversa; i confronti diretti sarebbero scorretti.

## Risultati delle tre esecuzioni complete

Tutti i 12 scenari per run (10k/100k/1M × quattro scenari), optimized=true,
formato 2. Verificati gli hash del codice benchmark, chiavi e projection contro
il working tree e la costanza dei contatori fra tutte le ripetizioni.
I tre JSON originali e il riepilogo `T-P17-02-measurements.json` sono conservati.
La provenienza compilata può riportare commit documentali diversi: sorgenti
misurati, harness e toolchain risultano identici. Tempi di compilazione esclusi.

| Scenario, 1M record | Mediana secondi | Intervallo secondi | Picco heap aggiuntivo byte | Allocazioni riuscite |
|---|---:|---:|---:|---:|
| Full valido |8,899|8,882–9,118|903294644|19668870|
| Full error-heavy |11,559|11,525–11,750|913041407|32668890|
| Proiezione isolata |1,199|1,176–1,259|535|11000000|
| Chiave isolata |0,115|0,114–0,132|16|1000000|

La memoria richiesta cumulata è 1.886.186.860 byte Full valido,
2.762.393.416 byte error-heavy,535 Mbyte projection e16 Mbyte chiavi. Rispettivamente
14, 1.000.014, 0 e 0 riallocazioni. Il costo diagnostico resta distinto dal valido;
le tre ripetizioni conservano gli stessi conteggi di errori attesi.
Non sono misure RSS, sampling CPU, profili SQLite o una promessa di scalabilità.

Confutata come base sufficiente per cambiare il core la sola doppia estrazione
nella funzione chiavi. Nessun candidato runtime applicato e nessun guadagno
prima/dopo dichiarato. Questa task consegna misure più utili e una decisione
motivata di non introdurre un'ottimizzazione prematura, coerente con ARCH33.
Golden e codice normativo restano identici nel diff; gate di regressione in corso.

Negativo reale: riuso dell'output baseline 1 rifiutato con exit 101, SHA256 del
file invariato. Smoke 100 record positivo; nessun benchmark completo lanciato
implicitamente nella normale suite. CI futura richiederà lo stesso smoke con
quattro scenari su Linux e Windows. Nessun nuovo crate, profiler o parallelismo.

## Gate locali e limiti della qualifica

Sei comandi richiesti eseguiti con exit0: fmt workspace, Clippy all-targets
workspace default e all-features, test workspace standard, smoke benchmark
nelle due configurazioni. Test standard:870pass/110riepiloghi,0fail/6ignored,
inclusi golden. Gli smoke sono eseguibili con assertion, non suite libtest:
ognuno verifica tutti quattro gli scenari100record con i rispettivi oracle.
399impronte sorgenti/manifest immutate durante i controlli, inclusi entrambi
benches. Ledger T-P17-02-local-gates.json con comandi/durate/impronte.

La ripetizione locale opzionale dell'intero workspace all-features è stata
interrotta con exit130 dopo il successo standard: nessun runtime o ramo feature
è modificato in questa task. Non è dichiarata passata. Completati invece entrambi
gli smoke pertinenti, Clippy all-features e la regressione standard. La futura
CI completa all-features su entrambi gli OS resta un gate nativo obbligatorio.
Nessun esito Windows anticipato; IN_PROGRESS fino alla CI reale.


## Chiusura nativa

DONE: CI41846160280 su9d641d4478ad8026441696a1f1bd4b099524e19f conclusa con
successo, tutti sette i job richiesti positivi. Entrambi gli smoke qualificano
realmente il nuovo scenario e meter; full all-features nativo passato su entrambi
gli OS, insieme ai confronti JSONL/pixel e trasferimenti SQLite. Packaging UI
opzionale saltato. Ledger T-P17-02-native-ci.json, tre letture distanziate.
Nessun gate residuo17-02; golden/runtime invariati e nessun guadagno dichiarato.
