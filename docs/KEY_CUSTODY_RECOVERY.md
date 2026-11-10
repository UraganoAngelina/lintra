# Custodia e recupero della chiave comune

D06 ratificata il10novembre2026: mantenere **gli stessi32byte originali** e
lo stesso Key ID. Questa procedura non genera, ruota o migra una chiave
operativa; non modifica HMAC1 o archivi già approvati.

Il responsabile della copia di riferimento conserva il file originale fuori
da Git/CI/log/installer generici. Ogni account autorizzato riceve una copia
privata con gli accessi OS richiesti. Documentare Key ID e provenienza della
copia senza stampare materiale chiave o inserirlo in screenshot/report. Custodia
e backup dei database sono operazioni distinte: il DB non contiene la chiave.

Per un aggiornamento compatibile distribuire solo software/asset verificati,
preservando chiave, Key ID, archivio e report. Non sovrascrivere hmac.key con
un file del nuovo pacchetto. Il futuro installer per-machine non rende la
chiave operativa un asset pubblico comune di Program Files: il provisioning
per account deve rispettare il contratto di accesso e le prove MIG/OPS.

## Perdita o copia errata

1. Fermare le operazioni storiche che richiedono la chiave. Non usare history
   init o un nuovo DB per mascherare un archivio non apribile.
2. Preservare l'archivio e il file sospetto, senza tentare conversioni. File
   assente, non regolare, lungo diversamente da32byte o con accessi non ammessi
   è un errore esplicito: nessuna chiave predefinita o generata come fallback.
3. Recuperare la copia di riferimento originale nel percorso configurato,
   mantenendo il Key ID e ristabilendo gli accessi OS. Usare il normale provider
   di file, non incollare il materiale in terminale, prompt o config JSON.
4. Verificare l'archivio tramite il normale preflight di key-confirmation.
   La CLI attuale offre history check con archive/key-id/key-file espliciti;
   il percorso GUI Tauri deve usare la stessa API e verrà qualificato in MIG/CAD.
5. Se il Key ID è uguale ma i byte differiscono, l'apertura deve fallire. Non
   cancellare audit, modificare metadati o considerare il DB vuoto per proseguire.
6. Per ripristino dati completo verificare separatamente inventario, namespace,
   revisioni e snapshot attesi in destinazione isolata, come U-OPS-08. Un check
   positivo della chiave non è già la prova di freschezza/completamento del backup.

## Compromissione

La chiave comune compromessa riguarda tutte le copie che la utilizzano: HMAC
è pseudonimizzazione/integrità, non cifratura. Sospendere le nuove operazioni
storiche coinvolte, preservare gli archivi e registrare l'incidente senza
pubblicare la chiave. Il responsabile decide l'intervento circoscritto: una
nuova procedura di migrazione/rotazione richiede un contratto approvato e prove
proprie. Sostituire il file silenziosamente rompe la continuità e non è recovery.
Non viene introdotta rotazione automatica, crittografia nuova o un servizio
enterprise. La cancellazione di una copia non dimostra secure erase fisico.

Le prove di U-OPS-05 usano soltanto materiale pubblico sintetico: assenza e
lunghezze errate, stessa chiave in due bundle, riapertura del vero SQLite con
la copia corretta, stesso Key ID con byte errati, sentinelle di redazione,
restore chiuso con storico e rifiuto di danni. Non sono operazioni sulla chiave
dell'autore. La guida Slint portabile resta in PORTABLE_KEY_BUNDLE; il nuovo
provisioning e le prove account Tauri non sono dedotti da questa procedura.
