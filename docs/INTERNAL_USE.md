# Uso interno di Lintra

Decisioni D01–D07/D14 ratificate il 10 novembre 2026. Destinatari: autore e
colleghi autorizzati dall'autore e dall'azienda. L'obiettivo è software pronto
all'uso interno effettivo con LocalOnly, Full e CAD storico, dopo i gate tecnici
e il signoff di distribuzione. La GUI corrente Slint non offre ancora tutti
questi percorsi; la qualifica Tauri è tracciata nello stato unificato.

Test, CI, esempi e screenshot usano dati sintetici. Per specifici dati personali
forniti dall'organizzazione valgono le autorizzazioni aziendali effettive: questa
ratifica tecnica non le attesta. G-REAL riguarda soltanto tali autorizzazioni,
quando applicabili; non blocca sviluppo, test sintetici o package tecnico.
Non sono richiesti server, autenticazione remota o un programma enterprise.
Recepimento MIG: [ADR-MIG-003](adr/ADR-MIG-003.md) per confini/threat model
e [ADR-MIG-004](adr/ADR-MIG-004.md) per test sintetici e G-REAL circoscritto.
Le policy sono approvate; prove WebView/IPC/account restano nei gate tecnici.

## Account e cartelle

Ogni account OS ha dati indipendenti. Namespace ULSS e chiave comune non sono
login, né autorizzano la lettura degli archivi di altri account. Nessuna
sincronizzazione automatica o selezione implicita di database condivisi.

| Superficie | Percorso/modello da consegnare | Stato verificabile attuale |
|---|---|---|
| App Windows11 x86_64 | Program Files, NSIS offline/perMachine; installazione elevata, uso ordinario non-admin | Installer Tauri da qualificare in MIG-P1/P8/P9 |
| Dati Windows per account | `%LOCALAPPDATA%/Lintra`, con sottocartelle per archivi, report, spool, configurazione e cache runtime | Contratto approvato; provisioning/UI in MIG e U-OPS-03, non ancora attestato |
| Linux personale CachyOS | Root locale per account in `$XDG_DATA_HOME/Lintra`, oppure `$HOME/.local/share/Lintra` se XDG_DATA_HOME non impostata | Solo build/smoke personale futuro Tauri; nessuna release GUI Linux ufficiale |
| CLI/library correnti | Percorsi espliciti scelti dall'operatore, verificati dagli adapter pertinenti | Esistenti; nessuna nuova scoperta implicita dell'archivio |
| Slint corrente | Bundle portabile e spool privato; analisi Trace/LocalOnly | Coesistenza conservata fino a parità e rollback |
| Chiave comune | File esterno 32 byte, Key ID stabile, copia di riferimento custodita separatamente | Provider esistente; nessuna chiave operativa in installer pubblico/Git/CI/log |

La root selezionabile deve essere locale, per-account e verificata quanto a
ownership/ACL, senza overwrite accidentali. Dati mutabili non vanno in Program
Files o nella directory dell'eseguibile. Un path denominato «privato» non basta:
servono le verifiche OS effettive di U-OPS-03 e le prove multi-account native.

## Protezioni e operazioni

Preservare ACL/ownership, temporanei privati, HMAC e custodia della chiave.
Nessuna nuova cifratura applicativa, password custom o retention automatica.
Protezione/password della cartella tramite strumenti OS è facoltativa; le
protezioni ordinarie dei file e dei temporanei restano obbligatorie. HMAC non
cifra database o report. Nessuna rotazione automatica della chiave comune.

Export locale sempre disponibile se scrivibile; destinazione shared/UNC/remota
riconosciuta con avviso e conferma, poi salvataggio. Overwrite e receipt restano
espliciti. La gestione completa del warning shared è U-OPS-04/MIG export, non
una capacità già attribuita alla UI corrente. Formati canonici invariati.

Conservazione e cancellazione sono manuali, sotto responsabilità operatore e
prescrizioni aziendali applicabili; niente TTL/purge o secure erase promesso.
BACKUP-B consistente e verificato è obbligatorio prima di upgrade storage
potenzialmente irreversibili; restore isolato e chiave originale recuperabile.
U-OPS-06/08 qualificano le operazioni effettive: la procedura di copia chiusa
già esistente non viene chiamata backup live o wizard completo.

Nessuna rete applicativa, cloud, CDN, telemetria o auto-updater. Aggiornamento
manuale/offline; uninstall conserva database, chiave, configurazione e report.
La distribuzione ai colleghi richiede G-MIG-9, G-CADUI3, G-SIOC1, G-OPS2 e G-DUR
verificati, installer Windows11 provato su due account e autorizzazione di
pubblicazione prevista. Questo documento non crea tag o release.
