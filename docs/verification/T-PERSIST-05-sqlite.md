# T-PERSIST-05 — adapter SQLite locale

## Preflight e piano

ADR-PERSIST-002 ratificata con prescrizioni nel registro dedicato. Prereq03 e04
DONE con CI native positive. UI05/4f7acb9 ora qualificata da CI40668541764 e
scrolling40668592173; nessuna nuova attività UI qui. Single-agent.

File previsti: src/persistence/sqlite/{mod,codec,security}.rs, port/mod ed
export root; protezione04 e soli metadati privati nei tipi dominio;
test unit/integration tra processi, benchmark reale e dossier/dipendenze.
Prima del prodotto risolto workspace temporaneo: rusqlite0.41.0 bundled/cache/
hooks, SQLite3.54.1, sei package nuovi e tutti i pin/checksum precedenti intatti.
Native APIs sicurezza usano package già qualificati e DLL OS già allowlisted.

## Confine di protezione

I DTO02 e il fake03 accettano token opachi dichiarati, non provano da soli un
HMAC. SQLite deve rifiutare quelle scorciatoie. Le API04 produrranno evidenze
MAC private per identità/record e invio protetto: le prove non fanno parte del
wire pubblico02 né di Eq/Hash dei valori. Deserializzare un DTO pubblico non
ricrea una prova. Il codec privato05 conserva e verifica le prove con il
protector configurato; una query prodotta con altro materiale anche sotto lo
stesso keyID diventa errore tecnico invece di falsa assenza.

Gli HMAC identity/record04 e il golden di framing restano invariati. Nuovi
purpose separati autenticano il valore protetto e i metadati. Il fingerprint
opaco originario dell’invio viene avvolto in un HMAC con i suoi metadati, prima
della persistenza, attraverso API esplicita: non è interpretato come dato
normativo o approvazione. Sigilli separati consentono verifica dopo restart;
provider/chiavi restano esterni. Nessun SHA non salato sostituisce una PK keyed.

## Storage/commit/versioni

Create/open espliciti su root locale privata, file fisso lintra.db. Open non
crea né reinizializza archivi mancanti/sconosciuti. Versione fisica1 separata
da software/schema/reference; stato non inizializzato/unsupported/corrotto
tecnico e tipizzato. Nessuna migrazione implicita. DELETE/EXTRA, foreign_keys,
trusted_schema, cache e mmap impostati e riletti. SQL parametrizzato e indici
su namespace/identità/invio/revisione; solo load_snapshot materializza O(n).

Stage possiede/condivide soltanto il candidato, mai tutte le versioni. Commit
acquisisce la transazione SQLite, ricontrolla CAS, scrive snapshot/record/indice
latest/audit e committa. Omissione non cancella le vecchie identità. Duplicate
submission resta l'errore03; risoluzione read-only di un commit ambiguo permette
di confermare la pubblicazione senza riapplicarla. Nessun commit tecnico è una
approvazione CAD. Lifecycle, backup completo e migrazioni future esclusi.

Test pianificati: roundtrip, process restart/contesa/crash dentro COMMIT e
perdita risposta dopo commit, rollback, namespace/omissione/immutabilità,
versioni/corruzione/privacy/permessi/traversal/symlink, query plan e carichi
10k/100k/300k sul DB reale. Picco processo include anche SQLite C, non solo heap
Rust. Metriche senza soglie DEC14 inventate; linking nativo e CI obbligatori.
Nessuna durabilità da perdita di alimentazione dichiarata dai soli process test.

## Lettura del codice e ragioni di correttezza

`SQLitePersistenceAdapter` condivide una sola `Inner` tramite Arc. `Mutex`
serializza la connessione per adapter senza fingere che serializzi altri processi:
SQLite IMMEDIATE e il CAS nel DB gestiscono quelli. Gli handle root/file vivono
fino alla chiusura della connessione. Debug ed errori tecnici non mostrano path,
query, token, payload o chiavi. Nessun servizio storico/UI viene attivato qui.

`create` è una scelta esplicita: provider validato prima di toccare il filesystem,
root vuota privata e create_new del solo file fisso. `open` non crea: controllo
header/appID/versione prima della recovery SQLite su file sconosciuti, quick_check,
struttura delle tabelle, audit della chiave e foreign_key_check. Un file vuoto è
Uninitialized; magic corrotto è Corrupt; altra applicationID è Unrecognized;
user_version non supportata è Unsupported. Nessuna migrazione/reset automatici.
I controlli integrità all'apertura hanno costo proporzionale all'archivio; non
si ripetono per ogni record. Le query puntuali non caricano tutti gli snapshot.

`configure` imposta/rilegge DELETE, EXTRA=3, FK=ON, trusted_schema=OFF, cache_size
-2048 (budget pagina suggerito di2MiB per connessione), mmap=0; temp_store=FILE,
statement cache8 e defensive=ON. Il budget cache non è un limite dell'intero
processo: codec, statement, snapshot del chiamante e SQLite hanno altri costi.
Non si converte implicitamente un file WAL esistente. Nessun fallback meno durevole.

Il formato fisico1 ha5tabelle: metadati protezione, generazione per namespace,
snapshot immutabili con UNIQUE invio, record ordinati con PK/UNIQUE, indice latest
con PK(namespace,identity). I FK tengono insieme le pubblicazioni. SQL sempre
parametrizzato per valori esterni. Revisioni SQLite fino i64::MAX: oltre questo
limite tecnico si restituisce RevisionOverflow, senza cast che ricicli valori;
il port pubblico conserva il proprio u64. Versione formato/appID sono distinti
da schema normativo, software e reference. Future migrazioni restano da ratificare.

`stage_snapshot` cancella prima il candidato precedente, controlla namespace,
revisione e tutte le prove MAC; poi clona l'Arc del solo snapshot. Non apre una
transazione SQL di scrittura. `commit` acquisisce IMMEDIATE, verifica nuovamente
generazione e invio, inserisce snapshot/record/provenienza e aggiorna latest in
un'unica transazione. Omettere una chiave non ne cancella la riga latest. Nessun
UPDATE di snapshot approvati; duplicate submission resta errore del contratto03.
Abort e Drop dello staging non scrivono. Questo commit tecnico non approva un invio.

Prima di COMMIT un errore fa scartare la transazione. Dopo aver invocato COMMIT,
un errore è CommitOutcomeUnknown conservativo. `resolve_commit` confronta invio,
revisione e candidato immutabile: Published/NotPublished/ConflictingSubmission;
non riapplica un'approvazione. NotPublished descrive la lettura attuale, non un
rollback immaginato di un altro processo ancora attivo. Se un errore I/O impedisce
anche il rollback e la connessione resta non-autocommit, `lock` rifiuta le letture:
serve riaprire per recovery, evitando di chiamare pubblicata una vista privata.

Il codec privato separa DTO02 da prove: deserialize del wire pubblico non dà
capability; solo producer04 o codec verificato le ricreano. HMAC con purpose
identity-proof/record-proof/submission/submission-proof distinti. Il MAC del
record lega identità e fingerprint; quello dell'invio lega tutti i metadati.
Le conferme usano verifica constant-time. Eq/Hash restano semantica dei valori,
non autenticazione; una prova privata non è mai autenticata tramite uguaglianza.
Il provider e il materiale chiave non sono serializzati. I metadati tecnici
(ID invio/versione/namespace) devono essere scelti dal chiamante come metadati
tecnici, mai riutilizzati per infilare identificativi paziente in campi liberi.

## Sicurezza e limiti operativi iniziali

Root assoluta esplicita e archivio per singola installazione. Primo perimetro
filesystem da qualificare: ext4 (runner Linux), Btrfs (volume locale misurato)
e NTFS locale Windows; altri filesystem,
volumi remoti, tmpfs e sidecar WAL/SHM rifiutati. Nessuna promessa su SMB/NFS,
XFS non collaudato o condivisione fra postazioni. Linux owner effettivo,
0700/0600, file non hardlinked, no symlink, antenati scrivibili da altri ammessi
solo sticky. Windows directory con DACL protetta ereditabile solo utente/System,
owner verificato, file con ACL ereditate verificate, reparse rifiutati e handle
senza share-delete per root/file. Il controllo non difende da un amministratore
o da codice ostile eseguito con l'identità dello stesso utente autorizzato.

SQLite non cifra il file: pseudonimi non sono anonimizzazione. Restano aperti
cifratura/retention/provisioning e rotazione chiavi/backup sicuri prima di dati
reali. I test di processo e i PRAGMA attestano il comportamento osservato, non
un esperimento di power-loss su hardware/firmware/filesystem arbitrari.
`load_snapshot` e il candidato esistente materializzano O(n); niente copia totale
del DB o HashMap permanente di tutti gli snapshot. Estensione futura: lettura e
preparazione streaming dietro nuovo contratto approvato, senza rompere i port05.

Harness `benches/persistence.rs`: processo fresco per volume10k/100k/300k,
1000lookup, p50/p95/max, commit, bytes DB, picco RSS Linux/working set Windows
comprensivo C/Rust/fixture, riapertura in ulteriore processo e3identità recuperate.
Confronto SQL indicizzato/non indicizzato su tabella su disco solo di test;
EXPLAIN richiede SEARCH senza SCAN nel percorso reale. Contesa e crash sono
provati dai test funzionali a processi reali. Nessuna soglia DEC14 inventata.
`qualify_sqlite.py` compila l'harness release con remapping/CRT già approvati,
applica l'allowlist ELF/PE completa di xtask e lo esegue: un CLI che elimina
SQLite con LTO da solo non dimostrerebbe il linking di un utilizzatore reale.

Gate locali in corso, nessun DONE prima degli esiti Linux/MSVC. Cross-check GNU
Windows locale non eseguito fino in fondo: manca x86_64-w64-mingw32-gcc; non
sostituisce il runner MSVC. Recuperato spazio eliminando solo438 eseguibili test
ricostruibili in target/debug/deps anteriori27/10 (~26,8GB), nessuna fonte/evidenza.


## Checkpoint di integrazione per CI

Primo ciclo unit:10pass/1helper ignorato (invocato dai parent fra processi).
Fmt e Clippy workspace default/all-targets iniziali positivi; oracle indipendente04
positivo, golden originali invariati. Aggiunti poi flush delle pagine nel test
crash, guard non-autocommit e test ACL Windows: gate finali ancora in corso.
Commit implementativo pubblicato per eseguire CI nativa/benchmark; NON chiusura
della task. Evidenze finali verranno registrate dopo gli esiti effettivi.


## Diagnosi dei primi gate — 2026-10-28

CI09f2bdd3: i due job Windows falliscono nella compilazione Rust della maschera
ACL: AceFlags è u8, i flag SDK sono u32. Correzione non troncante u32::from
prima dell'AND; nessuna ACL allargata. Nuove prove symlink directory/file anche
nel backend Windows, oltre al test DACL negativa. Il DNS locale ha temporaneamente
impedito il recupero log; nessun TLS disabilitato o configurazione di rete mutata.

Primo workspace default:768test principali +2esecuzioni helper restart,95suite
principali +2helper,0fallimenti/3ignorati (helper incluso). Quello all-features
finale fallisce11test perché il filesystem locale è Btrfs, non ext4: rilevato
con statfs/stat, rimosso per errore dalla prima allowlist. Ripristinato solo Btrfs
che è il volume effettivo dei test/benchmark; XFS resta non supportato/non provato.
Non è un indebolimento dei PRAGMA. Il precedente ciclo default era passato sul
Btrfs con l'allowlist iniziale. Ripetere ora i gate sulla fonte corretta.

La prima misura locale10k/100k/300k è una baseline intermedia: durante lo sviluppo
i sorgenti erano cambiati dopo la compilazione. Il campo commit scritto a fine
run non basta a identificarne la fonte: non usarla come gate finale. Harness
corretto per fissare commit iniziale e hash di tutti gli input prima/dopo;
una variazione durante la run ora invalida la qualificazione. CI lavora su clone
immutabile; anche lì verranno registrati gli hash. Nessuna soglia DEC14 dedotta.

SecondaCI40701606305: Linux completed/success; Windows supera Clippy ma fallisce
11test al create con UnsafeStorageLocation. La prova DACL diretta passa. Nessuna
policy rilassata per tentativi: aggiunto probe nativo che separa volume raw/
canonico, ACL file e confini root con soli risultati tipizzati, senza path/SID.
Il gate SQLite esegue prima i test core nativi, così eventuali errori di confine
sono diagnosticati prima del benchmark release. Diagnosi successiva guidata da
queste evidenze; PERSIST05 resta IN_PROGRESS.

Recupero log riuscito tramite curl con DNS-over-HTTPS per la sola richiesta,
verifica TLS integra, token passato solo su stdin e rimosso sui redirect fra host.
Nessuna modifica DNS del sistema, credenziale scritta in file o controllo TLS
indebolito. I file di rete temporanei restano fuori Git; nessun URL firmato nel
report versionato. Il download gh ordinario aveva errori DNS/handshake transitori.

Misura locale finale66aafa4 terminata exit0 e hash before/after uguali: artefatto
concreto/JSON in T-PERSIST-05-local-benchmark.json. Working tree dirty dichiarato:
qualificatore aggiornato poi versionato in fb6281e; codice Linux produttivo66aafa4.
Btrfs reale, gcc16.2.1, sourceID amalgamation atteso confermato. Volumi10k/100k/300k;
lookup p95 rispettivamente0,266/6,392/0,186ms; commit61,131/284,290/370,532s;
picco processo20,0/137,8/400,3MB decimali. Processo riapertura~9,2MB,3identità
recuperate per volume. Misura eseguita durante gli altri gate locali: contesa I/O
non isolata, non un nuovo SLA. Cache2MiB, DELETE/EXTRA, indici SEARCH verificati;
confronto senza indice e prova busy/retry nel JSON. Nessuna SQLite runtime di
sistema negli import dell'eseguibile misurato. Il candidato materializzato spiega
il costo O(n) della preparazione: nessuna promessa di staging streaming.

Definizione dei tempi: `commit_seconds` misura la pubblicazione completa
`begin_transaction → stage_snapshot` (verifica prove inclusa) `→ commit`, non
soltanto la syscall di sincronizzazione o il COMMIT SQL finale. La preparazione
del candidato è misurata separatamente. Il picco RSS/working set comprende C,
Rust, fixture e overhead, non la cache globale del filesystem del sistema operativo.

## Correzione Windows basata sul probe nativo

Probe40731347905/0fc0242: volume raw/canonico e root OK; file rifiutato perché
owner_self=false, owner_system=false, owner_builtin_admin=true, directory=false.
Il token amministrativo Windows crea file posseduti dal gruppo builtin
Administrators. Owner e DACL sono proprietà distinte; il modello dichiarato non
pretende isolamento da amministratori OS/System, che possono già assumere il
controllo degli oggetti. Il controllo precedente imponeva un vincolo non richiesto
ai file, incompatibile con questa policy ordinaria del sistema.

Correzione minima: root tuttora owned dall'utente corrente; file owned da utente,
System o builtin Administrators soltanto. Ogni altra identità owner è rifiutata.
Per TUTTI i file la DACL rimane verificata integralmente: sole ACE allow per
utente corrente/System, pieni diritti necessari; nessuna ACE Administrators o
World viene introdotta o ammessa. Prova negativa ora anche sul file con owner
amministrativo e DACL World: deve restare rifiutato. Nessun VFS personalizzato,
precreazione journal o cambiamento di durabilità/transazioni SQLite per aggirare
l'owner di Windows. Root/namespace/HMAC invariati; errore generico redatto.
