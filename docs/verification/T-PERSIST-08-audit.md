# T-PERSIST-08 — audit storico generico (in qualificazione)

Base UI fe5ada8: UI 9/9 e M8 verificate. Prerequisiti PERSIST07/P13 effettivi
nel preflight. Nessuna modifica normativa, a UI, dipendenze, schema fisico SQLite
o formato dello storage. Tutto il materiale di prova è sintetico.

## API e responsabilità

`HistoricalAuditService` riceve profilo compilato, protector e repository
espliciti. `prepare` verifica namespace e dominio dei parametri, versione/hash
correnti del candidato, policy cross_run_audit e autenticità delle proiezioni.
Non usa ValidationEngine, SQL o nomi di flusso hardcoded. La compilazione Full
con persistenza resta separata: questa API non certifica un esito normativo.

La generazione del namespace viene fissata prima delle letture. Le pagine di
metadati sono limitate a128 voci e controllate per continuità/ordine, namespace
e autenticità. Un buco, cursore incoerente o storico non leggibile produce un
errore tecnico, mai un archivio apparentemente vuoto. I flussi estranei sono
isolati. Per ciascun trace selezionato viene individuato l'ultimo invio con file
PRESENTE, indipendentemente dall'ultimo invio di altri trace o flussi.

Il profilo conserva i veri hash dei membri della compatibilità dichiarata;
versioni o hash non qualificati non vengono equiparati. Lo scope HMAC deve essere
quello della versione o del gruppo compilato; le chiavi non sono ricalcolate da
identificativi grezzi nel servizio. La verifica HMAC precede qualsiasi confronto.

`PreparedHistoricalAudit` mantiene stato privato immutabile, metadati consultabili
in sola lettura e soltanto le baseline selezionate. Ogni source/snapshot recuperato
è verificato. I metadati format_version=1 identificano un formato report distinto
da versione normativa, applicazione e storage; conservano generazione fissata,
profilo/hash/scope, sorgente candidata e baseline per trace. La copertura è sempre
`local_repository_only`: nessuna prova di completezza storica dell'organizzazione.

## Lookup puntuale con provenienza

`HistoricalIdentityOccurrence` contiene una identità protetta, revisione e source.
Il costruttore richiede revisione positiva e coerenza namespace/flow/trace dichiarato;
lo scope di versione esatta deve concordare con la source. Il gruppo rimane
responsabilità del profilo compilato. Questi controlli non attribuiscono una
semantica clinica all'identità e non sostituiscono la verifica crittografica.

`PersistentContextRepository::lookup_identity_occurrence` è additivo. Il default
fallisce esplicitamente con errore tecnico; un adapter precedente non restituisce
falsamente None per una query non supportata. La precedente lookup_identity
rimane compatibile. Il fake conserva anche la revisione nel suo indice latest;
nessuna scansione delle identità di tutti gli snapshot per ogni lookup.

SQLite esegue una sola query parametrizzata: latest → record → snapshot/source.
Gli indici esistenti e le rispettive PK servono tutte le ricerche, senza nuova
migrazione. Il query plan reale è verificato dal test adapter. LEFT JOIN conserva
un eventuale puntatore latest corrotto: BLOB mancanti o provenienza incoerente
producono CorruptStorage, anziché sparire dal risultato con un INNER JOIN.
Codec e protector autenticano sia identità sia source prima della restituzione.
Configurazione DELETE/EXTRA, locking, commit e formato2 restano invariati.

## Algoritmo e significato dei diff

`write_diffs` ordina esplicitamente i trace per ID e le identità per token protetto
(scheme, key ID, byte), poi esegue un merge dei due insiemi ordinati. L'ordine non
dipende da HashMap, ordine degli input o ordine delle dichiarazioni nel profilo.

- Solo candidato, senza occorrenza storica: Inserted nel repository locale.
- Entrambi: Unchanged o Updated secondo fingerprint della stessa definizione di
  proiezione e contesto di protezione; contesti diversi sono errore tecnico.
- Solo baseline, file candidato presente: Removed nel tipo dominio esistente,
  esposto come conteggio disappeared. È un'assenza osservata, non annullamento
  clinico né una nuova violazione normativa.
- Solo candidato, identità già osservata prima della baseline presente che ora
  non la contiene: Reactivated. Provenienza e revisione devono dimostrare che
  l'occorrenza appartiene alla generazione fissata. Un successivo snapshot con
  file assente può conservare la chiave: non sostituisce la baseline presente
  e non costituisce una nuova osservazione del file. Provenienza incompatibile
  o un'occorrenza presente più recente della baseline sono errore tecnico.
- File candidato assente: nessun diff per quel trace, nessuna cancellazione.
  Un file precedente assente non sostituisce la baseline presente, evitando
  anche false riattivazioni. Un file presente vuoto resta invece confrontabile.

Generazione e cancellazione vengono controllate prima dei lookup, dopo le
letture puntuali e alla fine. Un commit concorrente invalida la comparazione
con GenerationConflict, anche se il sink ha già ricevuto righe. Nessun begin,
staging, commit o riscrittura dello storico avviene nel servizio di audit.

`HistoricalAuditCounts` usa incrementi checked e `HistoricalAuditSummary` è
restituito soltanto con Ok. Il callback riceve un diff alla volta e propaga il
ReportError originale in una categoria tecnica distinta. Su Err può esistere
output parziale: pubblicazione atomica e cleanup appartengono al chiamante.
Questa task non aggiunge un falso writer transazionale né report su disco impliciti.

## Memoria, sicurezza e limiti

Le pagine di source sono bounded. Gli oggetti posseduti imposti da load_snapshot
hanno costo proporzionale alle baseline selezionate, non a tutto lo storico;
una stessa revisione selezionata da più trace viene caricata una sola volta.
Il merge usa due vettori di riferimenti alle righe del trace corrente, non un
Vec globale di diff. Restano in memoria gli snapshot selezionati: un futuro
port con cursore/proiezione per trace è necessario per eliminarne la
materializzazione completa. Non viene dichiarata un'API DB streaming inesistente.
Nessuna soglia DEC-14 o nuova misura prestazionale inventata.

Nessun record grezzo, CF, segreto o dato clinico viene introdotto negli output
persistenti o nei log (il servizio non emette log). Display degli errori omette
valori caller, identità e token. Le proiezioni provengono dalle API approvate
T-PERSIST-04; pseudonimizzazione non equivale ad anonimizzazione. Il callback
rimane responsabile dei propri messaggi I/O e della gestione del report.

Una source con file assente può contenere identità conservate in uno snapshot
cumulativo: non viene rifiutata come corruzione. Le righe del trace assente non
vengono confrontate. Se esiste una chiave storica conservata ma manca qualsiasi
baseline presente, NoComparableBaseline dichiara esplicitamente l'impossibilità
del confronto: mai un falso Inserted o una riattivazione clinica inventata.

## Prove effettive e gate pendenti

12 test di componente: trimestri multipli e tutti i change, ordine stabile,
file assenti e presenti vuoti, due flussi e namespace distinti, membri di gruppo
con hash verificati, versioni/hash non compatibili, chiave incompatibile,
proiezione incompatibile, DTO non attestato,130 invii/pagine multiple, namespace
non ammesso, righe con trace non dichiarato, cancellazione/sink e pubblicazione concorrente.
SQLite reale: scrittura, riapertura e recupero dell'ultima occorrenza omessa;
nessuna mutazione prodotta dall'audit. Test adapter aggiuntivo: EXPLAIN SEARCH
su tutte3PK, source con HMAC valido ma flow incoerente, puntatore dangling.
18 test SQLite passati,2 helper ignorati dal runner principale ma invocati come
sottoprocessi. Clippy all-features positivo dopo la correzione clone_on_copy;
12 test audit positivi. Suite complete e gate nativi ancora da qualificare.
Il fake serve ai fault applicativi, mai come prova di prestazioni SQLite.

Revisione in corso dopo le prime suite: rimossa una restrizione del nuovo codice
che equiparava erroneamente snapshot membership e presenza del file. Aggiunte
prove con identità conservate sotto TracePresence::Absent, riapertura SQLite,
riattivazione osservata rispetto alla baseline presente e assenza di baseline
confrontabile. La prima suite workspace default è terminata; la catena successiva
è stata interrotta intenzionalmente (exit130) per applicare questa correzione.
Log precedenti conservati con suffisso before-retained/interrupted; non sono il
gate della revisione corretta. Tutte le verifiche pertinenti vanno completate
sulla revisione corrente prima di dichiarare DONE.

Aggiornamento locale sulla revisione corretta:14 test audit,18 test SQLite e
Clippy all-features passati. Hash sorgenti/log in T-PERSIST-08-local-gates.json;
suite complete ancora in corso, nessuna CI o chiusura dichiarata.

Nota sui port terzi: SQLite e fake implementano current_generation con lettura
solo metadati. Il default compatibile del trait preesistente può materializzare
load_current_snapshot: un adapter terzo deve fornire l'override per evitare quel
costo a ogni guardia di generazione. La qualifica di lookup puntuale descritta
qui riguarda l'adapter SQLite concreto, non promette prestazioni di driver non
implementati. Entrambi gli assetti workspace correnti hanno concluso con831
esiti positivi/108riepiloghi (inclusi helper/doctest),0fail/4ignored ciascuno;
core-no-bundle e controllo boundary ancora in corso.

## Gate locali completati sulla revisione corretta

Fmt, Clippy default/allfeatures e boundary passati. Workspace default e
allfeatures:831esiti positivi/108riepiloghi ciascuno,0fail/4ignored; core senza
bundle:788esiti/91riepiloghi,0fail/4ignored. I conteggi includono helper e doctest.
Tutti gli hash sorgente coincidono con il manifest registrato dopo i test
mirati14/18; nessuna modifica di codice durante la qualifica completa.
CI nativa necessaria prima di DONE. Materiale esterno docs/migration preservato
e non incluso nei file della task.
