# Storico locale: conservazione, copie e ripristino

Questo documento descrive le capacità qualificate con dati sintetici. Non
autorizza l'uso con dati sanitari reali e non stabilisce una durata legale di
conservazione. Fonti: ADR-PERSIST-001, ADR-PERSIST-002, DET §166–174 e
[ratifica della chiave comune](verification/PERSIST-KEY-PROVISIONING-01-ratification.md).
CLI disponibile in [HISTORICAL_CLI.md](HISTORICAL_CLI.md); collocazione e accessi
del bundle in [PORTABLE_KEY_BUNDLE.md](PORTABLE_KEY_BUNDLE.md).

## Cosa viene conservato

SQLite conserva snapshot approvati, provenienza, generazioni e indice delle
occorrenze. Omettere un'identità in un nuovo snapshot non cancella le occorrenze
precedenti e non equivale a revoca. Gli snapshot già pubblicati restano leggibili.
Non esistono scadenza automatica, purge, compattazione con cancellazione logica,
rotazione o comando per riscrivere retroattivamente le sessioni. La continuità
dei namespace attraversa aggiornamenti e cambi di directory.

Questo comportamento tecnico **non è una policy di retention approvata**. Prima
dei dati reali vanno definiti durata, responsabilità, eventuali obblighi di
cancellazione e trattamento delle copie. Eliminare un file o una riga SQLite
non dimostra la cancellazione sicura da filesystem, supporti o backup.

## Review dei dati protetti e visibili

| Contenuto | Protezione effettiva |
|---|---|
| PK e valori della proiezione usata per il confronto | HMAC con scope e canonizzazione ratificati |
| Fingerprint dell'invio persistito e attestazioni | Protezione keyed e verifica di provenienza |
| Key ID, policy, namespace, flusso, trace, versioni e hash | Metadati tecnici leggibili |
| ID invio, periodo, UTC, conteggi, presenza dei file | Metadati leggibili, da mantenere privi di dati paziente |
| Database, journal, copie, report | Nessuna cifratura automatica fornita da HMAC |

La minimizzazione impedisce di archiviare i record originali tramite il port
minimo. Non bonifica testo arbitrario inserito dall'operatore nei metadati o nei
nomi dei file. I report di validazione possono avere contesti selezionati dalla
policy del report: questa qualifica dello storage non certifica l'assenza di
dati sensibili in qualsiasi report possibile. Audit storico e diagnostica tecnica
sono distinti dai report normativi. Namespace isolati non sono un sistema di
autorizzazioni multiutente.

Le sentinelle sintetiche dei test verificano PK, payload, fingerprint originale
e materiale chiave (byte e rappresentazioni esadecimali) nei file prodotti e nell'audit/Debug; non costituiscono una
prova universale di anonimizzazione. HMAC produce pseudonimi, non dati anonimi.
Chi possiede la chiave comune può correlare input ipotizzati nello stesso scope.
Tutti i colleghi condividono il rischio di compromissione della chiave; ciascuno
mantiene il proprio SQLite indipendente. Nessuna sincronizzazione è presente.

## Copia manuale a database chiuso

Non è disponibile un comando prodotto di backup/restore, un servizio di copie
automatiche o un backup live. La seguente procedura è controllata e offline;
la sua qualifica sintetica non sostituisce la futura policy di backup operativo.
È una procedura per il proprietario/manutentore della distribuzione. La verifica
completa tramite API richiede il suo tooling controllato: il bundle attuale non
fornisce un wizard o una CLI di ripristino completo per l’utente finale. Questo
non aggiunge prerequisiti all’avvio ordinario di Lintra.

1. Arrestare tutte le istanze CLI/UI che possono aprire l'archivio e impedirne
   l'avvio per l'intera operazione. Non copiare un database mentre è in uso.
2. Verificare l'archivio originale con `history check` e la chiave originale,
   quindi chiudere anche questa istanza. Il normale recovery SQLite resta quello
   del backend. Se resta un journal o la chiusura non è verificabile, preservare
   l'insieme dei file e fermare la copia: non cancellare journal manualmente.
3. Preparare una destinazione privata su filesystem supportato, senza link,
   con ownership/permessi/ACL conformi. Su Linux root archivio 0700 e database 0600;
   su Windows NTFS locale e DACL privata user/System secondo il contratto backend.
   La directory del bundle o un nome come `Backup` non garantiscono protezione.
4. Copiare il solo `lintra.db` dopo chiusura pulita. Conservare un inventario
   privato con versione/formato, Key ID, SHA256 del database e generazioni/invii
   attesi. Non includere chiavi, righe o identificativi paziente nell'inventario.
   Un digest verifica il trasporto, non garantisce da solo autenticità o freschezza.
5. Conservare separatamente la copia di riferimento protetta dei 32 byte originali,
   associata al Key ID stabile. Nessuna chiave in Git, artifact pubblici, cache CI,
   log o prompt. Non rigenerarla, neppure al successivo aggiornamento.

## Ripristino controllato in una destinazione isolata

1. Preservare l'archivio attuale e la copia originale. Chiudere ogni utilizzatore;
   usare una nuova directory privata, senza sovrascrivere l'archivio attivo.
2. Verificare inventario, provenienza e digest della copia prima di installarla.
   Un backup autentico ma vecchio può essere coerente: il backend non possiede
   un registro esterno che ne dimostri automaticamente la freschezza.
3. Copiare `lintra.db` nella destinazione isolata e verificare nuovamente accessi,
   ownership, filesystem e digest del file effettivo. La copia può ereditare ACL
   diverse: non disabilitare i controlli se il backend la rifiuta.
4. Usare la chiave di riferimento originale e il medesimo Key ID. Eseguire
   `history check` sulla destinazione ripristinata. Non usare `history init`,
   una chiave nuova o un archivio alternativo come recupero implicito.
5. Verificare gli invii e le generazioni attesi con le API di lettura qualificate,
   incluse sessioni precedenti e namespace pertinenti; non basta un messaggio di
   apertura riuscita. La CLI `check` non è un inventario completo dello storico.
6. In caso di chiave errata, corruzione, versione incompatibile o risultato
   incompleto, fermarsi e conservare gli originali. Non migrare o riparare
   implicitamente. Scegliere esplicitamente una copia valida con provenienza nota.
7. Solo dopo verifica completa scegliere esplicitamente l'archivio ripristinato
   nella configurazione. Conservare la possibilità di tornare all'originale;
   evitare che due copie divergenti siano usate come se fossero sincronizzate.

La key-confirmation è obbligatoria anche senza namespace popolati. Un Key ID
uguale con byte differenti deve fallire. Una chiave comune compromessa interessa
tutte le copie: sostituire semplicemente `hmac.key` interromperebbe la continuità;
serve una futura procedura di migrazione/rotazione approvata.

## Evidenze e decisioni ancora aperte

T-PERSIST-11 qualifica crash/recovery e trasferimento di SQLite chiuso fra Linux
e Windows. T-PERSIST-12 aggiunge copia chiusa con due revisioni, rifiuto di chiave
errata e danno, invariabilità dell'originale e test delle sentinelle. Le fixture
creano una destinazione privata usa e getta tramite il backend e sostituiscono
solo il relativo database sintetico chiuso: non espongono un'API di restore.

Prima dei dati reali restano da ratificare cifratura a riposo, retention, backup
completo (destinazioni, frequenza, accessi, custodia, verifiche e responsabilità)
e autorizzazione all'uso operativo. Il futuro installer React/Tauri valuterà la
migrazione del provisioning; questa versione Slint resta portabile senza wizard.
