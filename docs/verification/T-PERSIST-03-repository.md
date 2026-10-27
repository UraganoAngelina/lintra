# T-PERSIST-03 — port repository e fake transazionale

## Prerequisiti e perimetro

PERSIST-02 codee9f477b/CI40528607856 qualificata su Linux/MSVC con tutti4job
success. P13 completa:13-08 code52762c8/CI40192491629 e ratifica referenziale
approvata; nessun residuo sul contesto effimero. Fonti ADR-PERSIST-001,
ARCH45, DET166–174, piano20. T-P16-03/0a2121b attendeCI40577920052 e non è un
prerequisito. Task eseguita da un solo agente durante il gate UI indipendente.

Nessun backend disco/DB, protector, policy CAD/SIOC o interpretazione clinica.
Le nuove API restano separate da ContextStore e non abilitano la modalità
storica in ValidationService/CLI/UI. L'adapter in memoria è un fake dichiarato.

## Tipi e contratti

src/persistence/mod.rs estende lo scaffold già dichiarato da lib.rs. Export
additivi in root, nessuna variazione delle API esistenti di validazione.

- PersistenceError distingue mismatch namespace, collisioni strutturali, revisione,
  overflow, generazione, staging assente e storage indisponibile. Error/Display
  riportano cause tecniche statiche senza tenant/token/ID invio o payload. Una
  PK già osservata in un altro invio non è una violazione normativa qui.
- StoredIdentity contiene PersistentRecordIdentity e fingerprint opaco del record.
  Non accetta righe o projection raw; protezione/chiavi devono essere qualificate
  inPERSIST-04 prima di storage reale. Debug dei tipi sottostanti è redatto.
- HistoricalSnapshot è un contenitore immutabile di SourceSubmission/revisione
  e Arc<[StoredIdentity]>. Namespace deriva dalla sorgente, senza seconda copia
  incoerente. Constructor verifica namespace di tutte le identità e duplicati
  esatti interni; ordine record del chiamante conservato. Non calcola diff,
  cumulativi, annullamenti o certificazioni. Contenitori condivisi readonly.
- CommitReceipt espone revisione della pubblicazione riuscita; costruttore privato.
  Non attesta validità normativa, approvazione/consenso né scrittura di report.
- PersistentContextRepository è Send+Sync e separa lookup protetto, snapshot
  corrente/di revisione, recupero metadati invio e apertura transazione.
  lookup_identity ritorna l'ultima occorrenza committata per identità completa,
  conservando memoria anche quando uno snapshot seguente omette il record.
  Una lettura non inventa annullamenti né conflitti normativi. key_kind consente
  PK/progressivo/evento risolti dal compiler, senza nomi di campi hardcoded.
- begin_transaction usa Option<SnapshotRevision>: None significa namespace senza
  snapshot; Some(0) non è un alias dell'assenza. Precisazione idiomatica ammessa
  da DET167 (firme concettuali): elimina una generazione sentinella inventata.
  Prima revisione1, poi checked next; generazione iniziale e CAS sono esatti.
- PersistenceTransaction è Send; stage sostituisce il candidato privato, commit/
  abort consumano l'handle. Stage fallito cancella un precedente candidato, così
  commit non pubblica accidentalmente un vecchio intento; un nuovo stage esplicito
  può ritentare. Drop/abort scartano soltanto memoria privata, senza modifiche.

## Fake e atomicità

src/persistence/memory.rs: InMemoryPersistenceImage condivide Arc<Mutex<...>>;
open crea un nuovo handle sulla stessa immagine committata. È una simulazione di
riapertura in processo; non dimostra restart del processo, crash recovery, WAL,
backup, cifratura o durabilità su disco. Queste capacità restano alle task future.

Ogni namespace mantiene revisioni immutabili, indice dell'ultima occorrenza e
metadati invii committati. La chiave Hash/Eq include scope completo dei tipi02;
nessuna coercizione fra tenant/versioni/token context. HashMap non viene iterata
per produrre output ordinato. Letture restituiscono clone readonly dei dati.

begin verifica la generazione ma non crea un namespace. Stage non accede allo
storage. Commit verifica di nuovo la generazione sotto lo stesso mutex delle
letture, rifiuta ID invio già committato e costruisce la sostituzione completa in
privato. Solo l'inserimento finale pubblica insieme revisione/indice/audit.
Due transazioni sulla stessa base possono fare staging; una sola committa.
L'altra riceve un conflitto tecnico senza perdere metadati del vincitore.

Gli snapshot precedenti restano consultabili; ID invio è un fatto immutabile,
non l'ID di una sessione normativa modificabile. Draft/sessioni/approvazioni e
reinvii restano al service diPERSIST-06. Identità ripetuta in un invio distinto
aggiorna l'indice osservato e conserva lo snapshot precedente, senza errore SIOC.
Un mutex avvelenato produce StorageUnavailable, senza recupero permissivo dello
stato. Abort di memoria privata riesce anche con storage indisponibile.
Il fake clona indici per preparare la sostituzione: non è il backend ottimizzato
per grandi volumi e non costituisce una scelta di database/formato storage.

## Test ed evidenze

tests/persistence_repository.rs: sette integrazioni sintetiche, staging invisibile
su tutte le letture; commit snapshot/indice/metadati insieme; abort/drop/emptycommit;
riapertura senza staging e recupero vecchie revisioni; isolamento namespace e
record omesso conservato come fatto osservato; expectedNone vsSome0, stalecommit,
ID invio collidente; struttura snapshot errata, stage fallito non committabile;
due veri thread sincronizzati Barrier con esattamente un vincitore CAS.

Unit nel fake: revisioneMAX senza API pubbliche di seed/receipt forgiabile,
storage poisoned con fault sintetico e abort sempre possibile. Nessun dato PHI,
nessuna chiave segreta reale, nessuna dipendenza aggiunta. Sette mirati iniziali
positivi; gate finali dopo rafforzamento stage e unit boundary in esecuzione.

Il gate core senza embedded ha rilevato un import std::io rimasto dalla estrazione
UI03: ora cfg embedded-bundle, senza cambio del codice runtime/politica resolver.
Clippy core all-targets senza feature positivo, seguito da workspace default e
all-features entrambi positivi. Nessuna suppression o dipendenza per aggirare gate.

## Residuo e prosecuzione

Completare regressioni finali, aggiornare JSON gate/handoff e pubblicare.
CI nuova solo dopo fineCIUI40577920052 per non cancellare un gate pendente.
PERSIST-03 non DONE prima dei runner nativi; nessuna milestone storica anticipata.
Next16-04 dopo gate16-03; PERSIST04 dopo03 e preflight sicurezza reale.
AUTO_CONTINUE secondoAGENTS, nessuna nuova approvazione ordinaria richiesta.

## Gate locali finali

Pipeline sequenziale terminata con exit0 per tutti6gate: fmt, Clippy workspace
all-targets default/all-features, test workspace default/all-features, test core
senza default.735/735test nelle due varianti workspace/90suite;717core/84suite,
0fail/2ignored previsti. Passano anche la unit overflow/poison e lo stage fail-closed.
Clippy core standalone all-targets già positivo. JSON locale conserva comandi,
esiti/tempi e hash dei sorgenti. Nessun nuovo crate, golden precedente modificato,
backend scelto o capability storica dichiarata. Commit/push/CI nuovi ancora residui;
non DONE prima della verifica nativa. UI03CI40577920052 Linux success, MSVC pendente.

## Chiusura nativa

DONE: code49a3f4e, CI40581933176 su5bc8402 completed/success, Linux/MSVC e
confronti JSONL/RGB tutti pass. Evidenza condivisa con18-01 nel JSONnative-ci.
Tutti i test del fake e della regressione passati sui dueOS; nessun residuo03.
Backend/crypto/history service/policy rimangono taskfuture, senza capacità attivata.
Next PERSIST04 pronta per preflight; UI04 già corrente, AUTO_CONTINUE.
