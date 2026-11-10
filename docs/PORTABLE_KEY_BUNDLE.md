# Chiave comune e bundle Slint privato portabile

Contratto di T-PERSIST-10,4novembre2026. Fonte vincolante:
[PERSIST-KEY-PROVISIONING-01](verification/PERSIST-KEY-PROVISIONING-01-ratification.md).
Non è un installer, una rotazione, un restore SQLite o una preparazione di
materiale operativo eseguita dall'agente. Test/CI usano soltanto chiavi pubbliche
sintetiche; nessuna chiave operativa è contenuta nel repository.

## Layout effettivo e convenzione autorizzata

`scripts/release.py::pack(product="ui")` e `scripts/ui_package.py` producono un
archivio con questi file alla **radice**, senza una directory `bin` intermedia:

```text
lintra-ui.exe                 # Windows; lintra-ui su Linux
BUILD.json
SHA256SUMS
LICENSE-IBM-PLEX-OFL.txt
LICENSE-SLINT-ROYALTY-FREE-2.0.md
```

Il bundle generico non contiene chiavi. Schemi/dataset esterni restano un bundle
di dati separato, versionato; il piccolo snapshot incorporato non diventa un
profilo regionale completo per la presenza della chiave.

La convenzione per la copia **privata** aggiunge `hmac.key` accanto al vero
eseguibile. Il file contiene esattamente32byte binari, senza header, newline,
JSON, password testuale o Base64. Il Key ID non segreto è configurato separatamente
e deve rimanere identico fra destinatari/versioni compatibili. Ad esempio un
operatore può registrare `lintra-private-slint-v1` come etichetta comune; non è
un ID o un materiale operativo generato automaticamente dal software.

La funzione pubblica `portable_persistence_key_path(executable)` calcola soltanto
`executable.parent()/hmac.key`, da un percorso assoluto dell'eseguibile effettivo,
normalmente ottenuto con `std::env::current_exe()`. Non legge la cwd, home,
temporanei, environment di segreti o directory alternative. Il client passa quel
percorso e il Key ID al `FilePersistenceKeyProvider`, che verifica il file reale.
Il repository non conosce layout o percorso della chiave.

Questa task consegna API/configurazione per il client Slint; le schermate concrete
di preview storico rimangono nell'integrazione GUI prevista dal piano. Il form
Slint LocalOnly corrente non viene presentato come una UI storica implementata.
La CLI richiede sempre `--key-id` e `--key-file` espliciti e non usa la convenzione
Slint come fallback nascosto.

## Assemblaggio privato, una sola chiave per la distribuzione

1. Il proprietario verifica l'artefatto software generico e i suoi `SHA256SUMS`
   secondo il runbook release esistente. L'artefatto pubblico resta immutato.
2. Fuori da Git/CI/log/report/prompt, genera **una sola volta**32byte con un CSPRNG
   del sistema in un file nuovo e protetto, senza stamparne il contenuto. Non
   ricavare la chiave da password, identificativi o esempi di test. La generazione
   operativa e la preparazione effettiva del pacchetto restano alle task packaging
   e alla procedura offline controllata del proprietario.
3. Conserva una copia di riferimento protetta, fuori dal repository e dalle
   cache/build, associata al Key ID comune. Nessuna rigenerazione durante build,
   aggiornamento o nuova consegna. Non salvare il materiale in documenti o log.
4. Nella copia di lavoro privata dell'artefatto generico, colloca lo **stesso file**
   `hmac.key` e lo stesso Key ID per tutti i destinatari. Il pacchetto privato non
   viene caricato in una release pubblica o in artifact/cache CI accessibili.
   `SHA256SUMS` generico continua a descrivere i soli file generici; non viene
   modificato per fingere che la chiave fosse parte della build pubblica.
5. Consegna il bundle privato ai destinatari autorizzati tramite il processo
   privato scelto dal proprietario. La conservazione dei segreti e la cifratura
   del mezzo di consegna non vengono dichiarate qualificate da questa task.

Il proprietario deve verificare che il pacchetto privato aggiunga una sola chiave
e nessuna copia di storico, input, backup o credenziale. I file operativi non
diventano fixture. `.gitignore` esclude `.key` e copie `hmac.key*`; questa difesa
aggiuntiva non sostituisce la conservazione fuori Git richiesta dalla ratifica.

## Estrazione e accessi senza installer

La directory vicina all'eseguibile non è automaticamente affidabile. Il provider
verifica il file aperto: regolare, singolo hard link, nessun symlink/reparse nella
catena, ownership/accessi supportati e identità stabile durante la lettura.
Supporto qualificato previsto: Linux ext4/Btrfs e Windows NTFS locale fisso,
come il confine già adottato. Il gate nativo della task deve confermarlo.

Su Linux usare una cartella controllata dall'utente, non scrivibile da altri;
la chiave deve essere dell'utente corrente con modo esatto `0600`. Una cartella
nuova `0700` consente di estrarre senza esporre il materiale mentre si sistemano
i permessi. Non caricare chiavi da mount remoti o filesystem non qualificati.
Una destinazione possibile è `~/Lintra`, scelta dall'operatore: non è un
percorso che il programma cerca automaticamente nella home.

Su Windows il file deve avere il DACL atteso: pieno accesso soltanto all'utente
corrente e LocalSystem; gli owner privilegiati già ammessi per i file mantengono
il medesimo controllo DACL. Un comune ZIP non garantisce questi ACL dopo
l'estrazione. Si può predisporre una **cartella nuova** privata mediante le
proprietà Sicurezza di Windows, disabilitando le regole ereditate estranee e
assegnando all'utente corrente e SYSTEM pieno accesso ereditabile a file/cartelle.
Poi si estrae il bundle dentro quella cartella. Non occorre un installer o un
servizio credenziali; non modificare ACL di directory di sistema o condivise.
Una destinazione possibile è `%LOCALAPPDATA%\Lintra`; il suo nome non garantisce
gli ACL richiesti, che vanno predisposti e verificati prima dell'estrazione.

L'estrazione in una directory generica e un successivo test di sola presenza
del file non dimostrano protezione. Se gli accessi non corrispondono, il provider
rifiuta il caricamento. Non modifica ACL automaticamente, non copia segreti in
altre directory e non propone un'altra chiave. Un conflitto non risolvibile con
queste condizioni richiede la proposta circoscritta prevista dalla ratifica.

## Continuità negli aggiornamenti e recupero manuale

Per gli aggiornamenti compatibili distribuire i nuovi file software generici
verificati, mantenendo il file HMAC, il Key ID e il percorso dello storico
precedenti. Non estrarre un bundle privato su un'installazione esistente con una
procedura che sovrascriva silenziosamente `hmac.key`. Per spostare il programma,
spostare il bundle completo preservando la stessa chiave e ristabilire/verificare
gli accessi; la funzione di risoluzione segue la nuova posizione dell'eseguibile.

Se il file manca o è danneggiato:

1. Fermare le operazioni storiche. Non generare una chiave sostitutiva e non
   inizializzare un archivio alternativo per aggirare l'errore.
2. Il proprietario recupera la copia di riferimento del materiale originale,
   abbinata al medesimo Key ID, e la colloca nel percorso configurato con gli
   accessi richiesti. Non mostrarla in terminale, report o prompt.
3. Verificare l'apertura dell'archivio esistente con la key-confirmation. La CLI
   complementare, distribuita dal proprio artefatto generico, offre:

   ```text
   lintra history check --archive <archivio-esistente> --key-id <ID-stabile> --key-file <file-originale>
   ```

   Non è un prerequisito di installazione della GUI. Il comando carica il file e
   apre il backend attraverso i controlli reali, anche per namespace ancora vuoti;
   non crea, migra o approva invii. La normale recovery SQLite all'apertura resta
   responsabilità del backend, senza una nuova API di restore.
4. Un errore di key-confirmation/ID deve interrompere la procedura, lasciando
   esplicita l'incompatibilità. Non cancellare metadati o tentare conversioni.

La compromissione della chiave comune interessa potenzialmente tutte le copie
che la usano. Qualunque rotazione richiede una procedura futura esplicita: questa
task non la implementa. Il mandato del10novembre2026 ha ratificato file locali
per account, nessuna nuova cifratura/retention automatica e BACKUP-B verificato:
restano da completarne i collaudi operativi U-OPS. Il [runbook corrente di
custodia/recupero](KEY_CUSTODY_RECOVERY.md) conserva chiave e Key ID; eventuali
autorizzazioni aziendali per specifici dati personali non sono presunte.

Ogni PC mantiene uno storico locale indipendente. La chiave comune rende
confrontabili gli HMAC a parità di canonizzazione e scope; non sincronizza sessioni
e non prova che un evento sia assente negli archivi di altre postazioni.
