# T-PERSIST-07 — coordinatore applicativo delle approvazioni

Stato: implementazione e test mirati positivi; gate completi e CI ancora pendenti.
Prerequisiti06/P7 verificati nel preflight. Single-agent; nessuna dipendenza nuova,
modifica CAD/SIOC/FROZEN o nuova capability Full dichiarata.

## Policy e binding

`compile_persistence_profile` abbassa ora il solo `SnapshotPlan`: richiede la
capability dichiarata, approvazione esplicita/immutabile, tracce presenti complete,
scope non vuoto con riferimenti unici, anno/trimestre integer distinti e warning
W_* valido. Riferimenti errati, flag false, scope/periodo sovrapposti e piani
invarianti/temporali non implementati restano errori. La normale compilazione
eseguibile continua a rifiutare la capability storica: non si cancella un piano
normativo per far passare Full. `trace_ids` conserva l'inventario completo del
profilo, necessario per distinguere un'omissione da una trace dimenticata.

`bind` controlla tutti i parametri forniti contro i tipi e i domini compilati;
non ignora parametri sconosciuti, non sceglie il primo allowed_value e non converte
stringhe in numeri. Namespace/scope/periodo devono essere disponibili esplicitamente.
I limiti integer sono confrontati con IntegerLiteral, senza restringere i limiti
dello schema a un cast numerico nativo. NamespaceValue resta il contratto02.

`SessionBinding` contiene flow, proiezione ordinata dello scope e coppie
nome/valore dei parametri anno/trimestre. L'identità è sempre interna al namespace;
la versione dello schema non deve diventare un modo per sbloccare una sessione
approvata. Il confronto delle definizioni rifiuta cambi di nomi/tipi dello scope
o dei parametri periodo, invece di interpretarli automaticamente come storia nuova.

Il binding usa il campo **periodo opaco già esistente**, con protocollo tecnico
`lintra-session-v1:` seguito da JSON canonico. È una rappresentazione della chiave
di sessione, non un formato normativo di trimestre né un path. Solo i nuovi draft
possono ricevere il periodo derivato quando il chiamante passa None; se un periodo
è fornito deve coincidere. Mai sostituire il periodo di un invio già memorizzato.
Il formato è documentato qui prima dell'integrazione e non cambia il wire02/06,
il formato fisico2, schema SQL, HMAC o Parts pubblico. Il binding appartiene ai
metadati di sessione ammessi, non contiene identità paziente o segreti.

`SessionBinding::decode` richiede prefix e serializzazione canonica esatta. Una
Source ApprovedImmutable legacy senza binding verificabile causa
UnboundApprovedHistory, non «sessione inesistente», migrazione o ri-firma. Serve
un contratto esplicito di riconciliazione prima di approvare attraverso questo
coordinatore su quel precedente storico. Non è un blocco artificiale sugli archivi
nuovi: impedisce di inventare il significato di un periodo prima opaco. Vecchie
Source rimangono leggibili attraverso i port precedenti. Namespace diversi non
vengono correlati; un archivio locale vuoto non prova l'assenza organizzativa.

## Stati, preview e protezione

`ApprovalDraft` possiede una copia del profilo non forgiabile, parametri, Parts e
consenso opzionale. Il costruttore richiede Draft, namespace/schema/hash coerenti,
inventario completo e controlli dominio02/06. È soltanto preparazione privata:
nessuna lettura storica o scrittura. `absent_traces` e `missing_trace_warning_code`
permettono di presentare il warning prima del consenso. `with_proceed_consent`
verifica la ricevuta con le API06; non crea implicitamente una decisione PROCEED.
Cancel/drop consumano o scartano solo preparazione privata.

`validated` prende il risultato di validazione, non un booleano. Verifica il
contratto `ValidationSummary::validate`, stato valido/non cancellato/non troncato,
Full, schema/hash/versione validator, UTC iniziale, reference versioni, parametri
Full, inventario, hash input completi e conteggi/coverage delle trace presenti.
Un risultato Cancelled rimane un annullamento; gli errori non diventano uno stato
Validated. Ogni trace assente richiede copertura PROCEED esplicita; Present(0)
è comunque presente e non richiede consenso per omissione.

Sono accettate solo identità già protette e verificate, della flow/schema-scope e
delle chiavi record_key dichiarate nel profilo. Namespace e duplicati sono
ricontrollati dal costruttore HistoricalSnapshot. Source Validated e futura Source
ApprovedImmutable vengono costruite e protette separatamente **prima** di
qualsiasi adapter; modificare lo stato dopo la firma non sarebbe corretto.
`with_source` è crate-private e condivide le identità immutabili Arc: non duplica
la proiezione intera né altera Source storiche. Non è una nuova API pubblica di
mutazione di snapshot.

Il risultato Full pubblico non è una capability crittografica non forgiabile:
come gli altri risultati pubblici dell'API, il chiamante deve passare l'output
reale del servizio. Questi controlli non eseguono di nuovo regole normative e non
certificano conformità clinica. L'integrazione reale del servizio storico e della
reconciliation rimane08/09/CAD. I test07 usano outcome sintetici di componente,
validati dal contratto summary: non sono presentati come esecuzioni del servizio
storico non ancora implementato. Complete/Partial e diagnostica del run devono
restare visibili nel relativo report; CompletedValid non inventa Complete.

`ValidatedApproval::preview` legge la generazione, cerca precedenti e prepara il
candidato/base esatti. Non apre una transazione né calcola il diff cumulativo08.
Un retry esatto viene risolto sullo snapshot originale e restituisce la sua
revisione/Source approvata, anche dopo sessioni successive; una nuova revisione
non viene inventata. Per una nuova azione il risultato della preview ha Source
Validated; la futura Source Approved rimane privata fino al commit. Lo snapshot
di preview è il contenuto revisionato dal chiamante; lo stato dell'operazione è
`ApprovalPreview::state`, anche dopo il commit. Le Source pubbliche restano dati,
non token di approvazione.

`find_approval` percorre pagine di soli metadati in ordine revisioni, entro la
generazione osservata inizialmente. Controlla cursor monotono, namespace e limite
pagina, distingue altri flussi, binding legacy e cambi di definizione. Una diversa
submission per una sessione già approvata dà il rifiuto normativo
E_APPROVED_SESSION_IMMUTABLE. Lo stesso submission_id deve avere contenuto e
snapshot identici. Modificare un invio approvato della stessa flow, anche
retrodatandone il periodo, è E_APPROVED_SESSION_IMMUTABLE; una collisione con
altra flow o un invio non approvato resta DuplicateSubmission. L'esatto retry usa resolve_commit
senza riscrivere lo storico. Prima di una nuova preview, la generazione viene
ricontrollata: se cambiata è conflitto CAS, non un falso risultato corrente.
Il limite superiore evita di inseguire indefinitamente scrittori concorrenti.
`preview_with_cancellation` controlla il token prima e fra pagine.

## Pubblicazione, errori e ambiguità

`approve_commit` richiede una preview valida e un'azione esplicita. Su una receipt
già acquisita restituisce la stessa receipt senza ripubblicare il report o riaprire
una transazione. Su Cancelled non scrive. Su Uncertain non riprova né considera
l'errore un rollback.

Per una nuova approvazione il callback deve completare il writer reale del report;
un errore torna prima di begin_transaction. Non è promessa atomicità fra filesystem
report e DB: dopo un report riuscito, il DB può ancora fallire e l'esito va segnalato.
Seguono controllo cancellazione, begin con generazione esatta, stage del candidato
protetto e ultimo controllo cancellazione con abort. Solo allora il commit
consumante pubblica snapshot/indici/provenienza tramite l'adapter05. Dopo quel
checkpoint il commit non è annullabile: non si promette di rimuovere un'approvazione
pubblicata per un token arrivato troppo tardi.

CommitOutcomeUnknown cambia la fase privata in Uncertain; `state` restituisce
None, mai Approved o Draft per presunzione. `resolve_outcome` è una successiva
azione esplicita di sola lettura: Published conserva la receipt, NotPublished
consente un nuovo tentativo esplicito, ConflictingSubmission lascia l'esito
irrisolto. Non esegue una scrittura o un retry automatico. Report/storage/metadati
sono errori tecnici separati da SessionRejection e dal codice normativo immutabilità.
Debug dei preparati e Display degli errori non includono valori identificativi.

Le transazioni del repository rimangono capacità tecniche generiche. Chiamarle
direttamente non equivale a un'approvazione clinica; i client applicativi devono
usare il lifecycle e le integrazioni previste, non falsificare Source.state.
Nessuna UI/CLI o flusso CAD viene automaticamente abilitato da questa consegna.
Audit esteso/diff08, integrazione09, backup e migrazioni legacy non sono anticipati.

## Lettura dei metadati e limiti di costo

`current_generation` e `load_submission_page` sono port additivi. Il primo conserva
un default compatibile che può materializzare l'ultimo snapshot; SQLite/fake lo
overridano con lettura dei soli metadati. Il secondo ha un default che fallisce:
un implementatore esterno non può inventare un archivio vuoto solo perché non
ha ancora implementato la lettura paginata. Nessun SQL o path nei port pubblici.

SQLite usa la PK(ns,revision), seek cursor e LIMIT128; legge solo revision,
submission, Source e proof. Ogni Source è validata/autenticata e confrontata con
namespace e identificativo della riga. La generazione deve coincidere con MAX
revision, così una head corrotta non maschera storia già esistente. I test alterano
realmente questi indici e richiedono CorruptStorage. La query plan reale richiede
SEARCH USING PRIMARY KEY, non SCAN.

Il costo della ricerca di una sessione è **O(numero di invii nel namespace)** in
metadati, a pagine bounded; non è un lookup SQL puntuale per periodo. Si esegue una
volta per preview, mai per record, e non carica identità o tutti gli snapshot in
RAM. Un futuro indice dedicato alle sessioni può ridurre questo costo attraverso
un'estensione esplicita del port/formato: non si nasconde tale limite e non si
introduce una migrazione SQL implicita per ottenerlo. Cache2MiB/mmap0 restano
invariati. Staging e caso raro di resolve_commit/load_snapshot restano O(n) per
contratto05. Nessuna soglia DEC14 o benchmark di velocità è dedotto da questi test.

## Evidenze da chiudere

`tests/session_approval.rs`:13 test su binding e flag/piani/capability, metadati
incompatibili, errori/cancel/outcome incompleti, warning/consenso, drop/abort,
reportIO, sessioni precedenti, retry esatto, cambio versione schema, namespace,
CAS/doppio tentativo, esiti ambigui e loro risoluzione, restart SQLite, metadati
legacy, cursor128/130, contesa fra connessioni e corruzione indici. Fake usato
solo per fault applicativi deterministici, non per prestazioni/durabilità SQLite.
Il test SQLite usa un filesystem locale supportato sotto target; /tmp è tmpfs
e il primo tentativo è stato correttamente rifiutato dal preflight. Nessuna
allowlist filesystem o protezione è stata attenuata per far passare la fixture.

13test07+8profile attualmente passano; Clippy workspace default/all-features
positivo. Pipeline completa in corso, nessun DONE prima degli esiti effettivi
locali e Linux/MSVC. Invarianti05/06 di crash/restart restano nel gate completo;
non si dichiara un nuovo test fisico power-loss. Privacy e decisioni prima dei
dati reali invariate. Dopo07, priorità autore: UI16-06→07→08 con requisito
DIAGNOSTIC_REQUIREMENTS.md, non PERSIST08.

## Revisione del vincolo al repository

La review ha rilevato che ricevere un repository nuovamente in approve_commit
permetterebbe un cambio accidentale di archivio con identica generazione numerica.
`ApprovalPreview<'repo>` conserva quindi il riferimento al port originale:
approve_commit e resolve_outcome non accettano un repository alternativo. Il
borrow lo mantiene vivo e il tipo non è Clone/Deserialize; un compile-fail verifica
che il port non possa essere sostituito nell'azione di commit. Un callback report
che induce un'altra scrittura sullo stesso archivio rimane coperto dal CAS finale.
I port sono dipendenze fidate: non si pretende di difendere contro implementazioni
maliziose che cambino internamente il proprio archivio.

Se l'adapter richiede la chiusura/riapertura dopo un problema della connessione,
scartare la preview irrisolta e riaprire **la stessa configurazione di archivio**,
conservando la preparazione Validated e l'identità esatta del tentativo. Una nuova
preview cerca e verifica il commit precedente prima di proporre una nuova azione.
Nessun retry con nuovo submission_id o reinterpretazione automatica dell'errore;
il lifecycle non inventa un archivio nuovo o una policy di migrazione/recovery.

I primi gate completi sono intermedi rispetto a questa correzione di firma;
la qualifica finale ripete i controlli sulla versione con repository vincolato.

Il test nonempty_candidate verifica un caso SQLite con una identità sintetica
prodotta con le API reali IdentityProtector/build_join_key e riletta dopo reopen; nega flow,
trace o key-kind non dichiarati e valori non attestati prima dello storage.
L'estrazione della chiave usa gli stessi campi della fixture compilati localmente,
non è presentata come esecuzione Full del profilo storico.13test mirati07 e
8profile,13doctest e Clippy workspace nei due assetti positivi prima del push.
La regressione completa precedente alla correzione di firma è intermedia;
quella finale e CI sono ancora da verificare.

## Correzione timestamp esterni — qualifica finale

Il test foreign_clock_values_outside_chrono_range ha riprodotto un panic nella
conversione From<SystemTime> di chrono0.4.45 (sorgente locale datetime/mod.rs1931,
Utc.timestamp_opt(...).unwrap()). Un istante Unix rappresentabile può essere
fuori dal calendario chrono. Questo era un bug del coordinatore, non del DB o
della fixture; nessuna approvazione veniva committata ma l'API non doveva panicare.

`same_validation_start` confronta invece l'UTC del dominio con SystemTime tramite
checked_add/checked_sub. Usa secondi floor e nanos positivi anche prima dell'epoch,
rifiuta forme leap-second non rappresentabili dal clock e restituisce mismatch
per valori non rappresentabili: nessuna conversione panicking di dati esterni.
Due test coprono l'istante estremo (dove SystemTime può rappresentarlo) e la
frazione pre-epoch con differenza di1ms.15test07 nella run all-features corrente
sono positivi, inclusi questi due; il gate completo sta ancora terminando.
La CI40875525923 sul commit4c03608 è intermedia: richiesta cancellazione per
qualificare la correzione su un nuovo head, senza dichiarare DONE o rollback
fittizi. Il test che riproduce il panic resta in clock-boundary.log locale;
le evidenze finali distingueranno questo primo fallimento dalla successiva prova.

## Gate locali finali sul codice1e92b1f

Workspace default/all-features:795test principali/97suite per assetto, più3helper
riavviati dai parent,0fail/4ignored. Clippy workspace nei due assetti e fmt
positivi.15test di componente07, con correzione clock verificata in entrambi gli
assetti. Doctest separati:3merged+13altri=16positivi. Core senza bundle positivo,
conteggi/hash in local-gates.json. Sorgenti identici al commit pubblicato.
CI40877202683 su1e92b1f in corso, prima lettura entrambi superano test/default/
all-features/core-no-bundle; Linux release, Windows tooling. Nessun DONE prima
della conclusione dei job e dei confronti finali.

## Chiusura verificata

CI40877202683 su1e92b1f completed/success in tutti4job, Linux/MSVC e confronti
JSONL/RGB. Gate locali795/97suite per assetto, core763, Clippy/fmt positivi;
evidenze JSON archiviate.07 DONE, UI16-06 avviata come richiesto; poi07/08UI.
I checkpoint precedenti restano storici. Nessun nuovo gate umano ordinario.
