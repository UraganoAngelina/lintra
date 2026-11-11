# Gestione manuale di archivi, report e copie

D05 ratificata il10 novembre2026: Lintra non applica scadenze, TTL, purge o
cancellazione dei dati persistenti in base all'età. Non stabilisce un termine
di conservazione aziendale. La gestione manuale riguarda i dati che l'operatore
è effettivamente autorizzato a trattare e le copie di cui può disporre.

I percorsi correnti e quelli ancora da integrare in Tauri sono distinti
nell'[inventario](LOCAL_DATA_INVENTORY.md). La root Tauri prevista per Windows
è `%LOCALAPPDATA%/Lintra`; non spostare un database presso l'eseguibile in
Program Files. CLI e Slint correnti conservano i propri percorsi documentati.

## Archivi e continuità dello storico

Uno snapshot approvato resta immutabile. Non eliminare righe SQLite, sessioni,
indici o singole identità per liberare spazio, correggere un invio o consentire
il riuso di una PK. L'omissione da un nuovo invio non annulla il record storico.
Per CAD valgono le decisioni C1/L1; per SIOC valgono la baseline approvata,
la ricopertura1–3 e il divieto di riuso4.1/4.2. Nessuna di queste regole viene
sostituita da una scelta di conservazione.

Un database nuovo non annulla le PK già trasmesse. Non inizializzare o approvare
una baseline vuota per aggirare un archivio perso, incompleto o non apribile:
recuperare la storia pertinente oppure mantenere Unknown secondo la causa.
La chiave originale e il Key ID devono restare recuperabili anche quando si
spostano dati o si aggiorna il software: [custodia](KEY_CUSTODY_RECOVERY.md).

Per trasferire o dismettere un archivio locale:

1. Terminare validazioni, preview, approval ed export; chiudere le istanze che
   usano quell'archivio. Identificare root, namespace, revisioni e copie interessate.
2. Se serve continuità, preservare una copia chiusa e verificarne il ripristino
   in una destinazione isolata prima di rimuovere l'originale. Seguire le
   [operazioni supportate](PERSISTENCE_OPERATIONS.md), mantenendo chiave e Key ID.
   Non presentare questa procedura come backup live o wizard già disponibile.
3. Operare sull'archivio chiuso completo, senza scegliere pezzi interni da
   cancellare. Un journal residuo può essere necessario al recovery SQLite:
   non rimuovere `lintra.db-journal`, né altri file transazionali, per forzare
   l'apertura. Un errore di recovery richiede diagnosi, non un database vuoto.
4. Gestire manualmente soltanto le copie di cui è consentita la rimozione.
   Preservare la storia necessaria ai confronti successivi; non dichiarare
   completo un archivio da cui sono state sottratte prove necessarie.
5. Dopo uno spostamento o ripristino verificare l'apertura con la chiave
   originale e le revisioni attese. Il solo successo di key-confirmation non
   dimostra che la copia sia completa o aggiornata.

BACKUP-B verificato prima di upgrade persistenti irreversibili e restore
isolato restano requisiti delle task U-OPS-06/08. L'uninstaller Windows deve
conservare database e report; questa guida non ne anticipa la qualifica.

## Report, export e temporanei

JSONL/CSV/XLSX e summary esportati sono copie persistenti: l'operatore ne gestisce
posizione e durata. La sostituzione esplicita di un report segue il contratto
di export/overwrite; non è una scadenza automatica. Export locale scrivibile
consentito; destinazioni shared/UNC/remote con avviso e consenso secondo D07,
senza denylist generale o nuova conferma privacy locale.

La pulizia dei temporanei posseduti dal job è distinta dal purge dei dati:
CLI chiude lo spool temporaneo; Slint conserva la directory del risultato
mentre serve a consultazione/export; il writer atomico gestisce il proprio
`.part` non pubblicato. Un `.part` preesistente non viene troncato o rimosso
come recupero automatico di una collisione. Dopo un crash non presumere che
ogni file temporaneo sia stato rimosso: identificare il job, chiudere le istanze
e preservare ciò che serve alla diagnosi prima della gestione manuale.

Eliminare una copia o svuotare il cestino non dimostra secure erase da SSD,
backup, snapshot OS o altre copie. Lintra non offre tale garanzia e non
introduce cifratura o un nuovo protocollo di cancellazione. Non usare dati
sanitari reali nelle prove di pulizia/recovery, nei log o negli screenshot.
