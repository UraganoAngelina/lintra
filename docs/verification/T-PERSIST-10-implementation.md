# T-PERSIST-10 — provider file, client API e CLI storica

4novembre2026. **DONE**: gate locali e nativi qualificati; vedere la chiusura in fondo.
[Preflight](T-PERSIST-10-preflight.md), [ratifica dell'autore](PERSIST-KEY-PROVISIONING-01-ratification.md).
Prerequisiti07/8-03/09 qualificati;11 conclusa indipendentemente su633f61f,
CI41805357515. La CI11 non qualifica il codice10 ancora in working tree.

## Perimetro e contratti

Provider file32byte, configurazione portabile limitata all'eseguibile, API client
per testo/revisione, CLI init/check/preview con approvazione separata. HMAC,
canonizzazione, policy normative, namespace, formato SQLite e binding delle
sessioni sono invariati. Nessun database/driver, installer, credential manager,
servizio remoto, sincronizzazione o chiave operativa aggiunti. UI Slint concreta
storica esclusa dal prompt10; consegnate le API e la convenzione per il suo client.

La chiave comune e il medesimo Key ID sono una decisione esplicita dell'autore,
non un default compilato. L'assemblaggio privato e la generazione operativa sono
descritti nel [contratto portabile](../PORTABLE_KEY_BUNDLE.md), senza eseguirli.
La [CLI](../HISTORICAL_CLI.md) documenta argomenti, review, esiti e limiti reali.

## Sorgente della chiave e memoria

`FileKeyError` contiene soltanto categorie statiche redatte. Non conserva path,
errori OS contestuali, materiale letto o payload; Display/Debug non possono
esporli. `FilePersistenceKeyProvider` contiene Key ID non segreto e
`Arc<PersistenceKey>`; Debug esplicito redatto, nessuna serializzazione.

`load` valida l'ID non vuoto/non padded/senza controlli, apre la sorgente esplicita
e trasferisce la proprietà del buffer al `PersistenceKey` esistente. `resolve`
accetta solo l'ID configurato e clona soltanto l'Arc: niente rilettura per record,
fallback, ambiente, generazione, rotazione o duplicazione del materiale.
Gli handle si chiudono prima che `load` ritorni; il materiale rimane protetto
dal contratto Zeroizing esistente fino all'ultimo proprietario.

`portable_persistence_key_path` è una funzione pura: richiede un percorso
assoluto dell'eseguibile senza componenti parent/curdir e restituisce il sibling
`hmac.key`. Non assume che quel file esista o sia sicuro. Il client configura
separatamente l'ID; il provider riceve la sorgente precisamente identificata.
Un file passato esplicitamente come relativo viene risolto una sola volta,
senza cercare altri file; la convenzione automatica Slint non usa la cwd.

`file_security::key::Opened` possiede il file e gli handle dei suoi antenati.
`read` usa un unico buffer inizializzato Zeroizing di33byte e letture limitate
sulle slice residue: nessun `read_to_end` che possa riallocare una copia parziale
del segreto. Accetta solo32byte effettivi, anche se la dimensione metadata è
diversa; il33esimo rileva dati aggiunti senza leggere un file arbitrariamente
grande. Le uscite per errore azzerano l'intero buffer; il byte extra inutilizzato
è zero prima del truncate32. `mem::take` sposta la Vec nel nuovo wrapper
Zeroizing di PersistenceKey, senza copiarla in un buffer non protetto.

## Handle, filesystem e race

`src/persistence/sqlite/security.rs` e il figlio Windows sono stati spostati in
`src/persistence/file_security.rs`/`file_security/windows.rs`. SQLite usa un alias
interno: root privata, file/journal, filesystem e policy ACL qualificati restano
gli stessi. Il provider non dipende da SQL o dal modulo driver. I port pubblici
di repository/transazione non acquisiscono path della chiave o layout del bundle.

Su Unix `open` cammina ogni componente con `openat(O_DIRECTORY|O_NOFOLLOW)` da
directory fd possedute e apre il file con `O_NOFOLLOW|O_NONBLOCK`: un FIFO non
può bloccare il caricamento prima del controllo di regolarità. I descrittori sono
CLOEXEC e chiusi da RAII. `validate` confronta oggetti aperti e nomi correnti,
tipo, antenati non scrivibili da altri salvo sticky, UID, modo0600 e singolo link.
`signature` confronta device/inode, dimensione, mode, owner, nlink e mtime/ctime
prima/dopo lettura; atime è escluso perché la lettura stessa può aggiornarlo.
`fstatfs` sul file aperto verifica ext4/Btrfs, senza una prova riferita solo al path.

Su Windows il walk apre gli antenati con OPEN_REPARSE_POINT e senza condivisione
DELETE. Sono ammesse sorgenti locali su drive NTFS fisso, senza UNC/device path
come discovery alternativa. Il file finale non concede condivisione WRITE o
DELETE. Tipo, reparse, identità volume/file-index e NumberOfLinks vengono
verificati tramite handle; i nomi devono continuare a identificare gli oggetti
tenuti aperti. La signature comprende ID, dimensione e last-write-time.

`check_open_key` usa **GetSecurityInfo sul handle**, non l'ACL di un altro file
raggiunto tramite nome. `validate_descriptor` è condiviso col controllo storage
preesistente: owner e sole ACE user/System con accessi qualificati, senza
allargare trust o omettere diritti. Il file nel bundle non è fidato per vicinanza
all'eseguibile. Errori di accesso/volume e sostituzioni sono tecnici, mai violazioni
normative o inizializzazione automatica di un altro storico.

## API di richiesta testuale e revisione

`HistoricalTextValidationRequest` serve CLI/form. L'enum privato RequestInput e
`execute` condividono il percorso con l'API tipizzata esistente. Si risolve una
sola source compilata, poi `full::bind_parameters` interpreta il testo secondo
le definizioni reali. Text rimane String byte-esatta; Integer viene convertito
nel contratto storico i64 con errore esplicito in overflow. Il binder namespace
e la validazione Full conservano controlli esistenti, senza default/ID normativi.
Il profilo e la pipeline sono quelli della medesima risoluzione, non due letture
separate potenzialmente discordanti del filesystem.

`HistoricalReview` è un DTO owned con versione, generazione di base, revisione
candidata e fingerprint protetto della source. Campi privati, Serialize e Clone,
nessun Deserialize: il client lo ottiene dalla preview reale. `confirmation`
genera `APPROVE <revision> <fingerprint-hex>` dopo la preview. È un binding di
revisione umana, non autenticazione multiutente né prova di anonimizzazione.

`approve_reviewed` confronta l'intero DTO col candidato detenuto prima di
callback/storage. Un review di altro candidato fallisce subito; un repository
che cambia dopo la review viene comunque fermato dal CAS esistente. Le vecchie
API tipizzate/callback rimangono compatibili. Nessuna scrittura avviene perché
il client costruisce o serializza una preview.

## CLI e report

`ArchiveArgs` distingue la configurazione tecnica dalle `HistoricalArgs` di un
run. `history init` è l'unica creazione esplicita; `history check` apre e verifica
la key-confirmation anche senza namespace; `history preview` richiede il resto
degli argomenti Full e ID d'invio non sensibile. Main conserva exit0/1/2/130.
I comandi ordinari LocalOnly/Full non attivano lo storico.

Il provider viene caricato prima degli input. I controlli dei percorsi risolvono
identità dei file e antenati dei nuovi output, comprendono `.part`, chiave,
input, manifest, schema/reference e archivio. Gli output non possono sovrascrivere
la chiave o entrare nell'archivio; la chiave non può diventare un input validato
o un file del bundle schema/reference. Non si canonicalizza la sorgente passata
al provider/driver eliminando così i controlli originari contro symlink.

`AuditSink` scrive ogni diff direttamente in JSONL e il summary separato in un
envelope `phase=historical_preview`. Le source projected non sono presentate
come prova di un commit effettuato. Un esito ValidationOnly scrive
`historical_audit=not_executed`; invalid resta normativo exit2, mentre valid
con evidenze incomplete è errore della richiesta storica exit1 senza inventare
una violazione normativa. Cancel interrompe e pulisce gli staging posseduti.

`Reports` possiede ErrorSpool e tutti gli AtomicReportFile. `create` abortisce
ogni guardia già creata al primo errore; `parts` trasferisce ownership per
cleanup/pubblicazione; `fail` conserva causa e tutti gli errori di abort;
`discard` restituisce cancellazione solo dopo cleanup riuscito. Un part estraneo
mai posseduto non viene eliminato. XLSX continua a leggere il JSONL canonico
post-analisi, non entra nel validation loop.

Senza review si pubblicano report completi e nessun nuovo commit. Con review i
report restano staged e lo stdout presenta candidato/base/audit reali. EOF,
ABORT, token diverso o cancel non scrivono lo storico. Per un candidato nuovo,
`approve_reviewed` pubblica tutti i report **nel callback** prima della
transazione. `pending_reports` assicura cleanup esplicito anche se la cancellazione
precede il callback; `publication_error` conserva l'errore completo dell'adapter
senza perderlo nel tipo ReportError del callback applicativo.

La pubblicazione rimane atomica **per file**: nessuna falsa transazione comune
fra output e DB. Un errore successivo può lasciare precedenti file completi di
preview; non file finali con byte parziali o una falsa receipt. CAS può fallire
dopo i report e resta tecnico. Per un replay già approvato il servizio fornisce
la receipt precedente: nessuna nuova approvazione viene inventata dal client.

`report_approval` emette historical_commit solo per una receipt riuscita.
CommitOutcomeUnknown effettua una sola risoluzione in lettura, scrive lo stato
effettivo Published/NotPublished/Conflicting/Unavailable e rimane errore tecnico.
Nessun retry di scrittura e nessun falso rollback, anche dopo risposta persa.

## Attesa della decisione e lifecycle dei test

`review_input::Line` limita il testo a256byte ASCII e preserva gli spazi. Unix
usa poll50ms/read su fd pronto; EOF è un'interruzione controllata. Windows usa
PeekNamedPipe per le pipe, lettura immediata per file e WaitForSingleObject con
ReadConsoleInputW per la console: un evento tastiera non avvia una read-line
bloccante che impedirebbe Ctrl+C. Il branch console verifica EventType/union,
key-down/repeat e caratteri prima dell'echo. Nessun thread stdin distaccato.

I soli worker di test hanno ownership/deadline: writer killed/reaped, thread
lettore stdout joined dopo successo o kill, console nativa allocata solo in un
processo figlio isolato. Pipe/token e console sintetiche non modificano stdin,
console o directory di credenziali dell'utente. Le latenze di test non sono una
soglia prodotto DEC-14 inventata.

## Evidenze e residui

Prima qualificazione Linux:5test provider,6CLI adapter iniziali,2input pipe,
13SIOC inclusi i2nuovi APItest e1test del binario reale passati. Aggiunti in seguito
check, mappatura dell'esito ambiguo e caso reference incompleto: da riqualificare
nel gate finale. Gli errori di compilazione iniziali (lunghezza fixture/visibilità)
e il lint del guard di test sono stati corretti, non trattati come gate passati.

Restano gate completi sull'ultimo sorgente e CI Linux/MSVC, incluse ACL/reparse/
condivisione Windows, pipe e console reali. Il materiale operativo, packaging
privato effettivo, schermate Slint storiche, backup SQLite, installer/Tauri,
rotazione e autorizzazione ai dati reali non sono risultati conseguiti qui.


## Revisione finale — collisione chiave/manifest e gate ripetuti

Il primo gate workspace default ha concluso869esiti positivi, ma la review ha
individuato una collisione di ruolo: compose poteva leggere il file chiave come
manifest esplicito, prima dei controlli successivi. Interrotto l’assetto
all-features con exit130; quel tentativo non è il gate finale.

`preparation_paths` ora usa soltanto metadata e canonical identity prima di
comporre repository/preparers o aprire il backend: rifiuta chiave sotto root
schema/reference, chiave usata come manifest esplicito o MANIFEST.json implicito,
input uguale alla chiave o dentro l’archivio. Nessuna lettura JSON non azzerabile
può trattare la sorgente configurata come manifest per una collisione statica.
I controlli dei report/part restano prima di creare le guardie di output.

Il nuovo test verifica entrambi i manifest con una chiave binaria sintetica,
richiedendo lo specifico errore IO InvalidInput del preflight, non un BundleError
prodotto da un parsing tardivo.9test adapter tutti passati dopo correzione.
Il tentativo precedente rimane in target/verification-persist10 con motivo
persistito; i gate finali usano target/verification-persist10-final e nuovi hash.


## Gate locali finali completati

[Ledger](T-PERSIST-10-local-gates.json):11comandi tutti exit0,397impronte sorgenti
prima/dopo identiche, CARGO_BUILD_JOBS=2 e modalità offline. Il tentativo interrotto
non è incorporato nei pass del ledger finale.

| Gate | Esito effettivo |
|---|---|
| fmt workspace e tooling standalone | PASS |
| Clippy workspace default/all-features, all-targets | PASS entrambi |
| Test workspace default |870pass,0fail,6ignored;110riepiloghi |
| Test workspace all-features |870pass,0fail,6ignored;110riepiloghi |
| Core senza bundle |827pass,0fail,6ignored;93riepiloghi |
| Grafo core/workspace | PASS |
| Tooling Python |23test pass |
| Clippy tooling standalone locked/offline | PASS |
| Test tooling standalone |17pass,0fail,1ignored |

I conteggi includono gli helper subprocessi come nei ledger precedenti. I probe
ignored non sono pass impliciti; il probe release e i controlli Windows saranno
eseguiti realmente in CI. Commit/push e gate nativi restano da concludere prima
che10siaDONE. Nessun materiale operativo o UI storica concreta consegnati.

## Inventario della modifica

| File/gruppo | Funzione |
|---|---|
| src/persistence/file_security.rs e file_security/windows.rs | Helper storage spostati, validator ACL condiviso e test nativi |
| src/persistence/file_security/key.rs | Handle walk, identità/accessi/race, lettura Zeroizing limitata |
| src/persistence/key_file.rs e key_file/tests.rs | Provider esplicito, convenzione portabile, prove comuni/isolamento |
| src/persistence/mod.rs, sqlite/mod.rs, protection.rs | Wiring interno/export e doc del trait, algoritmi/SQL invariati |
| src/application/historical_validation.rs, src/lib.rs | APItext/review additive e re-export |
| src/cli/args.rs, mod.rs, historical.rs e historical/tests.rs | Parsing/init/check/preview, report e approvazione reale |
| src/cli/review_input.rs | Input bounded/cancellabile e test fd/pipe/console |
| src/cli/command.rs | Collisioni extra e cleanup condiviso, percorso ordinario preservato |
| tests/cli_history.rs, tests/sioc_persistence.rs | Eseguibile reale, exit/signal e API a source unica/stale review |
| Cargo.toml, docs/DEPENDENCIES.md | Feature Console/Pipes del binding OS già fissato; nessun crate nuovo |
| .gitignore, README e documenti CLI/bundle/decisioni/report | Protezione materiale locale e contratto operativo consultabile |
| IMPLEMENTATION_STATUS e dossier10 | Stato fedele, hash e risultati, nessun gate nativo anticipato |


Pubblicazione: codec884692, [CI41822083629](https://github.com/UraganoAngelina/lintra/actions/runs/41822083629)
avviata con dispatch riuscito. Stato WAITING_GATE; nessun successo Windows
inferito. [Registro nativo](T-PERSIST-10-native-ci.json).


## Chiusura nativa effettiva

CI [41822083629](https://github.com/UraganoAngelina/lintra/actions/runs/41822083629)
su c884692e29dbc0db9a9405414606f09b1156d8bc completed/success. Tutti sette i job
richiesti positivi, incluso MSVC, confronti byte JSONL/pixel e SQLite tra OS.
Log Windows letto: nuovi test del provider con ACL/link/sostituzioni, pipe native
e console isolata passano realmente. La qualifica include il codice10; non usa
la vecchia CI11 come sostituto. Ledger completo T-PERSIST-10-native-ci.json.
Packaging UI opzionale saltato, nessuna release o assemblaggio operativo eseguiti.
