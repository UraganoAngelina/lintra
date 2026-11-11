# Destinazioni dei report — D04 / U-OPS-04

Le destinazioni sono ispezionate da Rust prima di creare file di staging.
Un filesystem riconosciuto locale non richiede un nuovo consenso privacy.
Una destinazione remota/condivisa o non determinabile richiede un avviso e
una scelta esplicita per l'export selezionato. Dopo il consenso si procede;
non esiste una denylist generale di share o filesystem.

## Servizio comune

`ReportDestinationAssessment::inspect` risolve i soli output richiesti senza
creare cartelle/file o modificare ACL. `ReportDestinationLocation` distingue
Local, Remote e Undetermined. Linux usa statfs e tipi UAPI; FUSE/overlay o
tipi non riconosciuti restano indeterminati. Windows considera UNC, prefissi
estesi e il volume effettivo, preservando anche l'endpoint UNC/drive mappato
scelto prima della canonicalizzazione. Questi fatti non attestano chi può
leggere una cartella locale o se altro software OS la sincronizza.

`allow_local` concede soltanto l'esportazione su destinazioni riconosciute
locali. `acknowledge_warning` esprime la scelta dell'operatore e produce
`AuthorizedReportDestinations`, con campi non costruibili dal chiamante.
`verify_unchanged` ricontrolla l'esatto insieme ordinato di path canonici e
classificazioni prima di scrivere e prima di pubblicare. Un cambio di file,
formato/percorso o tipo richiede nuova ispezione; non si conserva un bypass
globale. Non è una promessa contro sostituzioni ostili fra syscalls.

`ReportDestinationReceipt` contiene conteggi Local/Remote/Undetermined e la
scelta esplicita, senza path o contenuti. Debug e messaggi della policy non
espongono i percorsi. Errori I/O restano tecnici; Undetermined della
destinazione è distinto da Unknown dei controlli normativi.

## Adapter effettivi

La CLI applica la stessa policy a report, summary, XLSX e audit storico.
`--allow-shared-export` conferma il salvataggio per quel comando, non approva
sessioni storiche o record. Se richiesto il consenso manca, non si creano
`.part`; con flag esplicito viene emesso un evento stderr con soli conteggi.
La CLI mantiene i suoi contratti esistenti sui path espliciti e pubblicazione
per-file. Il flag non ignora input/bundle protetti, errori, cancellazione,
budget o vincoli del validatore. La pubblicazione storica resta nel callback
reale che precede l'approvazione CAS, e un errore la impedisce.

Slint usa lo stesso servizio nel worker export: il primo tentativo mostra il
verdetto Rust e la checkbox di consenso quando necessario. La scelta riguarda
il report catturato dalla finestra. Cambio path/formato o riapertura la
azzera. Overwrite e consenso destinazione sono scelte indipendenti e restano
entrambe visibili quando richieste. Enter e pulsante rispettano i controlli;
il backend li verifica anche se un callback viene invocato direttamente.

`export_report_with_authorization` mantiene i writer canonici e la gestione
errori già presenti. Le receipt positive conservano i file effettivamente
pubblicati e i conteggi della destinazione. Fallimenti/parzialità conservano
`published`, cleanup e cancellazione. Nessuna transazione multi-file o con DB
è inventata. Gli spool interni sono privati; lo staging `.part` eredita il
confine della destinazione esplicitamente scelta, come nell'inventario U-OPS-02.

La futura UI React/Tauri userà questo servizio Rust nelle task export MIG;
qui non viene anticipato un protocollo IPC o ricostruita la policy in JS.

## Qualifica

Le prove locali includono locale senza consenso, tipologie indeterminate,
scope cambiato, errori prima/dopo consenso, canonicalità dei tre formati,
overwrite, cancellazione e controller reale. Le due prove Windows con share
SMB e drive mappato richiedono la fixture loopback del runner usa e getta:
sono esplicitamente ignored nella suite generica ed eseguite dalla CI con
`--ignored`. L'assenza della fixture è un errore, non un PASS simulato.
La fixture autorizza il solo account del job, non introduce server nel prodotto
e viene rimossa dalla CI. Metodi OS verificati nella documentazione Microsoft
di [New-SmbShare](https://learn.microsoft.com/en-us/powershell/module/smbshare/new-smbshare)
e [New-SmbMapping](https://learn.microsoft.com/en-us/powershell/module/smbshare/new-smbmapping).

Un test Server 2025 non sostituisce Windows 11 clean-room, account distinti o
la UI Tauri. G-OPS1 aggregato resta subordinato anche all'integrazione U-OPS-03;
G-MIG-1 e G-REL mantengono i loro requisiti nativi separati.
