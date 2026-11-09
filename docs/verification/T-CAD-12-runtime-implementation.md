# T-CAD-12 — integrazione storica generica e CAD83

9 novembre2026. Mandato attuativo completo; C1/L1 ratificate. Single-agent.
Questo report descrive codice e prove; i gate finali e la CI sul commit esatto
sono evidenze separate. Nessuna equivalenza fra componente storage e CAD12DONE.

## Dal compilatore al record

`schema/history_existence.rs` accetta solo formato3, capability esplicita,
snapshot, namespace coincidente, join history_including_current zero_or_many
senza riduzione e set_exists con Eq locale/literal. Ogni join deve essere usata.
Predicati diversi richiedono join distinte; un contesto corrente non può
nascondere dipendenze storiche. Il digest include hash schema, encoding effettivo,
join, metadati operando/presenza, literal compilato e PK target. La closure del
when ammette dipendenze locali risolte e rifiuta relazioni implicite.
Il risultato è compiler-owned, non costruibile deserializzando DTO pubbliche.
`compile_historical_snapshot`/registry selezionano esplicitamente l'esecutore;
ordinary e audit-only non ottengono questo supporto tacitamente.

`RecordValidator` aggiorna uno stato per campo dopo la pipeline ordinata.
Missing/Invalid/Unavailable restano distinti da Valid; una membership non
definitiva non diventa un operando definitivo. Reset per record e campo evita
riuso di stato. L'artefatto pubblico `local.valid` non cambia.
Il collector riceve dalla stessa PASS1 RawRecord/PK/stati e proietta una sola
volta identità fisica e invarianti. Nessuna apertura successiva degli input.
Correlation/PK/operando invalidi producono indeterminatezza; un errore locale
indipendente non cancella un positivo. Il run invalido resta non approvabile.
Conteggio prodotti contro righe/EOF impedisce che righe non osservate perché
senza PK siano interpretate come insieme completo privo di witness.

## Query e dipendenze PASS2

`HistoryCollector` verifica key-confirmation, fonti e inventari autenticati;
pagine metadata di128, snapshot uno alla volta, senza caricare i record clinici
storici. Pinna la generazione prima/dopo preparazione e attach; CAS al commit.
Non esegue query SQL per record. Il contesto engine espone soltanto risultati
trivalenti e validità di campo, senza SQL/chiavi. Un piano storico senza tale
contesto viene rifiutato prima di aprire input.

La vista live è sostituita dal target Present qualificato; Absent conserva il
witness, lasciando gap. Rettifiche e annullamenti L1 non recuperano positivi dai
vecchi snapshot. `True` richiede un positivo live valido; `False` richiede C1 e
tutti i trimestri pertinenti qualificati, senza gap; ogni altra mancanza è
`Unknown`. Incompatibilità/legacy mantengono gap, non inventano copertura.
Corruzione o prove mancanti sono errori tecnici anche con un positivo corrente.
L'indice delle occorrenze non decide la validità normativa.

Le dipendenze when sono valutate per regola: l'unione usata dal piano non
maschera un controllo indipendente. Il resolver PASS2 qualifica anche operandi
scalari/consensus con gli stati reali: una data invalida non diventa ammissibile
solo perché i byte soddisfano una regex. Gli altri percorsi senza questo
contesto mantengono il comportamento precedente.

## Servizio e approvazione

`snapshot_validation.rs` usa schema/run/input effettivi, stessi porte e summary.
Il consenso esatto alle trace mancanti precede qualsiasi lettura storico/input;
nessuna trace assente viene sostituita con un file vuoto. I controlli dichiarati
su source Unavailable rimangono nel summary con zero valutazioni/righesource;
il dominio ammette questa rappresentazione solo con tutti i contatori a zero.
Un'altra combinazione incoerente continua a essere rifiutata.

Invalid/Cancelled/truncated/reference incompleta/Unknown restituiscono
ValidationOnly. PASS2 emette la violazione dello schema solo per False.
Il coordinatore usa le invarianti del profilo effettivo: CAD ha cross_run_audit
vuoto e quattro invarianti; non si aggiungono dichiarazioni audit per aggirare
un requisito implementativo. Draft, preview, audit e approval sono quelli reali.
Il qualificatore interno associa batch alla Source approvata prima del controllo
replay e dello staging. La review opzionale impegna fatti e origini C1; il
vecchio review audit-only omette il nuovo campo e mantiene i byte precedenti.

Il replay ricostruisce la storia precedente alla sessione originale: origini
aggiunte dopo non ne modificano il candidato. Le approvazioni successive sono
comunque verificate tecnicamente. Fatti/origini diversi non possono passare
come replay della sola Source. Report/cancel/rollback/CAS non pubblicano fatti;
gli stessi transaction/recovery/receipt gestiscono l'intera unità.
Il contratto ricevuto richiede sessioni trimestrali cronologiche: il nuovo
ingresso rifiuta una richiesta nuova retrodatata prima di aprire gli input.
Non usa fatti futuri per convalidarla e non riscrive lo stato attivo con un
periodo precedente. Il replay immutabile è invece ricostruito alla sua frontiera
originale; la successiva verifica della Source impedisce di cambiarne periodo.

## Formato e privacy

Writer logico2 per correlation nullable solo Indeterminate/UnqualifiedCorrelation;
reader1/2, format1 mai null, formato ignoto tecnico. Some serializza come il
precedente Fingerprint concreto; i byte canonici1 rimangono identici.
SQLite fisico3 e manifest restano invariati; archivi1/2 leggibili, nessuna
migrazione tacita. Le golden HMAC/proiezioni1 non sono riscritte. C1 conserva
hash/fonte/responsabilità/data approvati, non documento/chiaveevento/riga/raw.
Il claim pubblico non può diventare da solo un batch autenticato.

## Composizione e matrice di verifica

Profilo operativo `schemas/operational/CAD/2024-06.runtime-candidate.1` acquisito
dal FileSystemSchemaRepository e due cataloghi reali via preparatore Strict.
83/83 campi sorgente,5trace, note/presenza/PK complete; nessun filler. D→C
esclude i progressivi, che restano nelle PK. Regole parent, date, età, T70,
lookup, sospensione/conclusione/inattività e storia sono dichiarative.
Le esclusioni sono quelle già deliberate, mai nuove rimozioni per compilare.
Il generatore verifica gli hash ricevuti prima di scrivere; rigenerazione
byte-identica verificata. Status candidato, non FROZEN.

| Accettazioni HX | Evidenze eseguibili |
|---|---|
|01–10, storicità/trivalore/L1/C1/isolation/anni/operandi|tests/history_existence.rs e tests/cad_history_runtime.rs|
|11–12, incompatibilità/legacy/codec|test applicativi hash/predicato e inventario senza fatti; sqlite::tests::history reader1/2/no-upgrade|
|13–15, integrità/reopen/replay/report/cancel/CAS|test applicativi e sqlite::tests::history; input eliminati all'ingresso PASS2|
|16, crash/COMMIT ambiguo|sqlite::tests::qualification e regressioni persistence/session della suite completa, con snapshot e batch indivisibili|
|17–18, scope/lifecycle/privacy|annullamenti/namespace; scansione archivio senza evento/riga/documento/chiave raw; transplant/root/manifest storage|
|19, misure|misurazione ignorata esplicita CAD83, due namespace/6approvazioni/reopen e volumi dichiarati; nessuna soglia inventata|
|20, native|CI nuova richiesta sul commit esatto; fixtureSQLite crossOS con batch logici1/2 e nullable2|
|21, fail-closed|compilatore ordinario/audit rifiuta; subset non supportati e context mancante rifiutati|

Golden T1/T2/T3: oracle fisso `tests/golden/cross-trace/cad-history/2024-06.runtime-candidate.1/expected-timeline.json`,
reopen ad ogni trimestre e fatti effettivamente caricati. Golden negativo:
False con C1 genera un solo E_CAD_INITIAL_EVALUATION_REQUIRED suB1/linea1;
JSONL confrontato byte per byte su due esecuzioni. Partial con B2/D mancanti
non ottiene approval. Date invalide non generano falsa violazione codice15.

Limiti dichiarati: ingresso snapshot pubblico library, CLI storica audit-only
immutata; dati pubblici sintetici e chiave fixture0xaa. Nessun provisioning
operativo, certificazione universale dei domini non forniti o release implicita.
