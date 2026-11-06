# Recovery CAD03 — journal SQLite concorrente, 6 novembre 2026

Stato: locale verificata, prova Windows nativa da avviare; nessunDONE CAD03/11.
Il conteggio resta130/140. Scope di recovery ordinaria del gate, nessuna norma
sanitaria, formato storage, SQL, HMAC, chiave operativa o politicaACL modificata.

## Fallimento effettivo e isolamento

CI42190202683 su42f92ed completed/failure: Linuxsuccess, Windows workspace e
allfeatures positivi, core senza bundle fallito. Il worker di
`two_process_contenders_preserve_cas_and_resolved_retry_publishes_one_generation`
ha fallito riaprendo per risolvere l'esito, con UnsafeStorageLocation anziché
la risoluzione attesa. Nessuna conferma di pubblicazione è stata inventata.
Log e snapshot originali in CAD03_SQLITE_RECOVERY/windows-original-failure.log
ed original-native-ci.json. Gli altri job dipendenti sono skipped, non success.

Recovery sviluppata in checkout isolato del commitd81b520 (CAD03+CAD11), senza
trasferire il lavoroCAD09 ancora non qualificato. Il lavoro09 resta nel checkout
principale, conservato con hash prima/dopo. Nessuna delega o processo background.

La causa è coerente con il journal opzionale che SQLite elimina fra enumerazione
e metadata/ACL. Regressione deterministica sulla medesima condizione: prima del
fix, rimozione del journal appena enumerato produce UnsafeStorageLocation invece
diStorageBusy. Log regression-before.log (exit101), hash dello scaffold precedente
in before-source-sha256.json. Il test non si limita a simulare un enum: elimina
realmente il file alla boundary di filesystem. Non attribuisce al logWindows
un rawOSerror o uno stage preciso che quel log non registrava.

## Codice e motivi di correttezza

file_security::check_files mantiene firma/guardie e delega al corpo condiviso
con callback vuoto. FileCheckStage/check_files_with_hook sono soltanto interni
al modulo persistence; gli hook dei test consentono una race deterministica.
La callback produttiva non altera l'inventario. Nessuna nuova API pubblica.

Prima di leggere metadata, un NotFound èStorageBusy soltanto per il nome esatto
lintra.db-journal già ammesso come rollbackjournal. L'apertura fallisce: non si
salta il file, non si apre laConnection, non si accetta un inventario non verificato.
Errori su database obbligatorio, nomi inattesi, link, tipi nonregolari o permessi
restano rifiuti con le categorie precedenti.

Windows legge laACL nominata dopo metadata. Se quel controllo fallisce, viene
verificato se il solo journal è realmente scomparso (NotFound); soltanto allora
il tentativo fallito èBusy. Un oggetto ancora presente conACL errata rimane
UnsafeStorageLocation; non si catturano genericamente AccessDenied o erroriACL.
Su un eventuale retry tutti i controlli devono ripartire da capo. Non si modifica
la verifica del filechiave aperto, rootpinned, owner/DACL/link o filesystem ammessi.

Non viene aggiunto alcun retry di scritture/commit. Il resolver di test esistente
ritenta soltanto read/openBusy entro20s, mantenendo CommitOutcomeUnknown finché
una nuova connessione verifica davvero Published/NotPublished. CAS/generazioni,
consenso, atomicità e ricevute idempotenti sono invariati.

## Test e gate

La regressione metadata rimuove davvero il journal prima di symlink_metadata,
richiedeBusy e poi una nuova preflightpositiva. La varianteWindows rimuove il
file fra metadata eACL nominata. I negativi mantengonoUnsafe per journal-link,
nomi inattesi, journal0644Unix e, suWindows, journalconDACLpubblica. I test di
contesa originali conservano le attese: nessun codice101 accettato come successo,
nessun rilassamento dei risultati0/3 o delle asserzioni sull'unica generazione.

Gate locali isolati: fmt, Clippy root all-targets/allfeatures e due suite
persistence senza bundle/SQLite allfeatures, tutti exit0;35esiti nel primo filtro,
28nel secondo includendo3riepiloghi subprocess (25principali),4ignored per ciascuno.
Hash prima/dopo identici. Ledger CAD03_SQLITE_RECOVERY-local-gates.json.
Non è una nuova suiteworkspace o un gateWindows: la prova nativa deve ancora
qualificare la varianteACL e la contesa sul sistema interessato.

sqlite-probe.yml estende il filtro diagnostico a persistence:: e rende esplicito
--no-default-features, coprendo anche le boundaryACL/key sul medesimo nativeMSVC.
Non sostituisce laCI completa; dopo provaWindows positiva avviare una solaCI sul
commit prodotto che comprende recovery+CAD03+CAD11, senza codiceCAD09 incompleto.
LaCI iniziale fallita rimane fallita e conservata. Nessuna release/tag avviata.
