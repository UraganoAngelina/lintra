# Durabilità controllata e recovery — D15

U-REL-03. Le prove sono sintetiche e il loro esito corrente è nel
[dossier](verification/unified-post-m9/U-REL-03.md). Questo contratto descrive
le operazioni effettive; la loro presenza nel codice non equivale a un gate PASS.

## Pubblicazione di un file

`AtomicReportFile` possiede un `.part` creato esclusivamente nella stessa
directory della destinazione. Parti preesistenti non vengono troncate o
cancellate. Cancel, truncation, errore del produttore e guasti prima del rename
mantengono il precedente file definitivo, con cleanup/retention già previsti.
`CompletedInvalid` può produrre un report completo delle violazioni.

`complete` verifica flush e `sync_all` del file, chiude l'handle e pubblica:

- Linux: rename sullo stesso filesystem, poi apertura e `sync_all` della
  directory contenente il nuovo nome. Un file sincronizzato non implica da
  solo la persistenza della sua directory; vedi [fsync(2)](https://man7.org/linux/man-pages/man2/fsync.2.html).
- Windows: `MoveFileExW` con REPLACE_EXISTING e WRITE_THROUGH, senza COPY_ALLOWED,
  delete-first o rinvio al reboot. Si conserva il supporto a nomi Unicode e
  path lunghi canonicalizzando il parent, senza seguire un eventuale symlink
  della destinazione. È una richiesta all'OS, non una prova fisica universale;
  i limiti dell'opzione sono quelli di [MoveFileExW](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-movefileexw).

L'ownership del `.part` termina subito dopo il rename riuscito. Il successivo
errore di directory-sync produce `PublishedDurabilityUnconfirmed`, errore
tecnico aggiuntivo nel `ReportError` già non-exhaustive. Non è un rollback e
non modifica le violazioni normative. `output_was_published()` distingue quel
caso dai fallimenti precedenti: l'export conserva nella propria failure il
file già pubblicato, interrompe gli altri salvataggi e mostra che la scrittura
non è stata confermata. Non restituisce una receipt di successo.

Questo vale anche per una cartella scrivibile ma non leggibile: il file può
essere salvato prima dell'errore di apertura della directory. Non viene negata
preventivamente la destinazione, né viene inventata una conferma di durabilità.
Un filesystem remoto/opaco mantiene il percorso D04 con consenso. Se l'OS
rifiuta una sincronizzazione, l'errore conserva la sua causa e il numero reale
di file pubblicati. Nessuna denylist di export o nuova policy privacy.

Report e summary sono due file e non diventano una transazione unica. Un report
pubblicato prima di un errore non attesta che lo storico SQLite sia approvato.
Nella CLI storica un errore del callback report impedisce l'approvazione CAS;
le transazioni e i loro esiti ambigui mantengono i contratti esistenti.

## SQLite e storico

Il repository configura e verifica già journal DELETE e synchronous EXTRA,
oltre alle altre impostazioni di integrità. U-REL-03 non cambia driver, schema,
HMAC, chiave, formato, CAS o modalità journal. EXTRA estende le operazioni di
sincronizzazione del rollback journal; non è una garanzia contro ogni difetto
hardware. Riferimento: [SQLite synchronous](https://www.sqlite.org/pragma.html#pragma_synchronous).

Le prove del writer interrotto aprono un lettore indipendente prima del lock,
verificano visibilità solo di snapshot approvati e ammettono StorageBusy come
errore tecnico durante la fase SQL. Dopo kill o uscita dentro COMMIT si riapre
lo stesso archivio. Fatti e manifest devono appartenere integralmente alla
revisione precedente oppure a quella nuova; nulla di approvato a metà.
Un commit riuscito senza risposta si risolve contro il candidato immutabile,
senza duplicare fatti o generazioni. Prove corrotte restano errori, non vuoti.

## Matrice delle prove

| Punto | Prova effettiva prevista | Osservazione richiesta |
| --- | --- | --- |
| Create, write, cancel, rename rifiutato | Test filesystem/producer esistenti | Nessun falso definitivo; vecchio output invariato |
| Flush, sync file, rename | Fault injection privata del test | Errore tecnico e cleanup, nessuna pubblicazione |
| Create/write/file sincronizzato | Kill reale di un figlio posseduto dal test | Vecchio definitivo; parte residua riconoscibile e non riutilizzata |
| Rename, ack pubblicazione, risposta persa | Kill reale del figlio | Nuovo definitivo completo; nessun `.part` presentato come report |
| Sync directory dopo rename | Fault injection; permessi Linux reali con account non privilegiato | File pubblicato contato; errore tecnico, nessun successo finto |
| Stage, pagine SQL/journal, COMMIT, risposta persa | Kill/exit del writer SQLite con fatti e manifest firmati | Reopen coerente e risoluzione idempotente |
| CAS/concorrenza, manifest danneggiato, chiave errata | Regressioni SQLite/storico già presenti | Nessuna baseline non approvata o falsa evidenza negativa |
| Copia chiusa e riapertura isolata/cross-OS | Fixture di trasporto SQLite già presente | Snapshot/fatti/manifest autenticati; non confondere con backup live |

Hook e selettori dei fault esistono solo sotto `cfg(test)`. I figli sono creati
dal test sincrono, terminati e attesi anche in caso di assertion fallita; nessun
processo delegato o servizio viene lasciato attivo. Il helper ignored viene
invocato dai parent reali. La prova Linux dei permessi è eseguita esplicitamente
con `--ignored` in CI: eseguirla da root non costituisce una prova valida.

## Recovery e limiti

Una parte residua non attesta Complete e non viene eliminata automaticamente.
L'operatore conserva il precedente definitivo, isola il residuo identificato
senza sovrascriverlo e ripete l'operazione con un percorso libero dopo aver
verificato l'esito corrente. I test usano solo parti e directory proprie.
Nessuna nuova retention, purge, riparazione permissiva o rotazione della chiave.

Backup-B consistente e verificato prima di upgrade irreversibile e restore
applicativo isolato restano U-OPS-06/08; le fixture di copia non li sostituiscono.
Windows11 clean-room, due account e installer restano G-MIG/G-REL. La CI MSVC
su Server2025 non equivale a quelle prove. Ext4/Btrfs locali e runner Windows
devono essere indicati con gli esiti realmente osservati nel dossier.

**NOT_TESTED_HARDWARE:** nessuna perdita fisica di alimentazione è stata
indotta. Kill/exit non svuotano le cache del kernel o del controller. Firmware,
dischi, controller, storage remoto e filesystem possono non rispettare le
richieste di flush come previsto. Sono qualificabili le garanzie software sui
filesystem provati, mantenendo esplicito questo limite.
