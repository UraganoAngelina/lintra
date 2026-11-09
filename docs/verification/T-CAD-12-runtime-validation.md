# T-CAD-12 — gate locali runtime e CAD83, 9 novembre 2026

Stato: QUALIFICATO LOCALMENTE, CI RUNTIME NATIVA ANCORA DA ESEGUIRE.
CAD12 rimane IN_PROGRESS; P18-04 non READY, nessun FROZEN/M9/release/tag.

## Input e risultati

15gate sequenziali positivi,504input invariati e HEAD750b1e1 fermo durante
compilazione/test/misure. Working tree dirty dichiarato, nessuna mutazione di
HEAD/index fatta dall'agente. Ledger runtime-local-gates.json e manifest
runtime-local-inputs.json conservano comandi, hash, tempi ed esiti reali.
La CI successiva dovrà qualificare il commit che pubblica questi stessi sorgenti.

|Gate|Esito effettivo|
|---|---|
|fmt e Clippy workspace/alltargets/allfeatures|exit0, warnings negate|
|workspace default|1059esiti/124riepiloghi,0fail,7ignored|
|workspace allfeatures|1059esiti/124riepiloghi,0fail,7ignored|
|core no-default-features|1016esiti,0fail,7ignored|
|Python tooling|24pass|
|workspace dependency boundary|exit0, GUI solo adapter|
|tooling fmt/Clippy/test includeignored|18pass, incluso bundle estratto con binario esplicito|
|benchmark smoke validation/persistence/profiles|exit0; profili componenti dichiarati, non sostituiscono CAD83|
|runner CAD83 release no-run|compilato realmente|
|release riproducibile/artifact|binari identici; sole4librerieGNU approvate, OpenSSL assente, probe negativo rifiutato|

I conteggi workspace sono somme dei riepiloghi, inclusi sottoprocessi, non
numero di test unici. Fra i test:142lib/4ignored,21applicativi storici generici,
9CAD83 e una misura CAD83 ignorata esplicitamente nel test ordinario.
Le altre ignorate richiedono native exchange/sonde esplicite; non sono PASS
presunti. Le misure CAD ignorate sono state eseguite a parte come descritto sotto.

## Misure sul profilo completo

Eseguibile release identificato dal log Cargo, hash nel ledger
runtime-measurements.json. Invocazione diretta del runner ignorato
`measure_full_cad83_history_application --exact --ignored --nocapture`, con
LINTRA_CAD83_EVENT_COUNT=100 e poi300. Nessuna compilazione nel processo misurato.
Ogni invoco: due namespace, tre trimestri, sei validation/preview/approval reali
e chiusure/riapertureSQLite. Nessun errore, Complete;8400righe complessive al
volume300 (2800alvolume100). Timings dei6campioni escludono costruzione fixture;
RSS wait4 copre tutto il processo, inclusa fixture; IO blocchiLinux512byte.

|Eventi per namespace|p95 empirico ms|picco RSS KiB|DB byte|
|---|---|---|---|
|100|758.8|25936|40878080|
|300|1919.5|28276|122519552|

Il p95 su sei campioni è descrittivo (qui il massimo), non uno SLA.
Nessuna soglia prodotto di tempo/RAM è stata definita o inventata. Dati pubblici
sintetici,83campi reali,5file, due cataloghi verificati e SQLite reale. Il primo
invoco debug precedente includeva la compilazione nel RSS e non è usato come
misura RAM runtime. Nessuna ottimizzazione/parallellizzazione adottata.

## Tentativi non qualificati e correzioni

Primo tentativo: golden root CAD non registrata. Registrata la radice versionata
col proprio runner applicativo; invariati gli attesi. Secondo: il vecchio
esempio storico formato1 non riportava più UnsupportedConstruct. Ripristinato
il guard formato3 nelle join, senza indebolire i test;8persistence_profile e
13golden mirati positivi prima della campagna finale. Manifest/ledger preliminari
sono conservati separatamente.

Nella campagna finale il primo invoco tooling includeignored mancava della
variabile LINTRA_TEST_BIN richiesta. Fallimento conservato nel ledger; ripetuto
soltanto quel comando con target/debug/lintra esplicito (hash registrato),18pass.
Sorgenti immutati: i gate1..9 già positivi non sono stati ripetuti inutilmente.
Le condizioni CI erano già corrette; workflow non modificato.

## Prossimo gate autorizzato

Commit/push sul branch work/p1-foundations, una sola CI completa sullo SHA
esatto. Servono7job: Linux/MSVC, JSONL, due pixel e due importSQLite opposti.
La fixture crossOS ora verifica batch logici1/2 e nullable2,4snapshot/2namespace.
Nessun risultato dello storage precedente è riutilizzato come qualifica runtime.
Dopo CI effettiva: chiusuraCAD12 e avvioP18-04 secondo il mandato, senza nuovo
checkpoint ordinario.

## Pubblicazione e CI avviata

Push riuscito sul branch autorizzato: codice
c59b0def0915d8cca6dae7f59048aa38f50b6807. Dispatch CI42634997501 riuscito,
prima lettura SHA/branch confermati e due verifyjob attivi. Nessun esito finale
ancora letto. Il ledger runtime-native-ci.json conserva questa prima evidenza.
Le7qualifiche native e i due import devono ancora essere verificati; CAD12
rimane IN_PROGRESS/WAITING_GATE. Nessuna CI duplicata.
