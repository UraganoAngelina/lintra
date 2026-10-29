# T-PERSIST-06 — preparazione del perimetro

Prerequisito PERSIST05 ancora in qualificazione finale: questa lettura non
costituisce avvio né completamento06. Implementare soltanto dopo gate05 positivo.

Le strutture02 e il codec05 già conservano presence/count, hash input e schema,
UTC, namespace, periodo opaco, versioni reference, fingerprint e stato dichiarato.
Occorrono prove reali SQLite e restart con Present(0), Present(n), Absent distinti,
metadati invalidi e nessuna perdita della provenienza delle vecchie revisioni.

Gap da risolvere nel perimetro06: il consenso PROCEED per tracciati assenti è
approvato da ADR001/DET171, ma non ha ancora rappresentazione nel SourceSubmission.
Non può essere dedotto da Validated/ApprovedImmutable o dall'assenza di errori.
ABORT non deve produrre mutazioni. Il lifecycle applicativo resta successivo,
non si implementano approvazione clinica, servizio storico o UI qui.

Qualsiasi estensione deve preservare il wire02 e la verifica HMAC degli snapshot
esistenti: introdurre un campo serializzato di default cambierebbe il messaggio
firmato e farebbe apparire corrotto lo storico. Verificare una forma additiva con
assenza esplicita del consenso non registrato; nessuna riscrittura o ri-firma
silenziosa. SourceSubmissionParts pubblico e port03 vanno mantenuti compatibili.

Il versionamento storage è indipendente dallo schema normativo. Se i nuovi
metadati richiedono una capacità di formato nuova, scegliere un contratto di
lettura/versioni esplicito e di eventuale upgrade non distruttivo, mai un reset
o una migrazione automatica in open. Definire prima le invarianti e i relativi
casi di compatibilità; non assumere che il semplice deserialize Rust valga come
contratto di migrazione approvato.

File previsti: dominio metadati/wire dedicato, protezione/verifica del payload,
codec/config storage solo dove strettamente necessario e test di roundtrip,
compatibilità e restart. Nessuna norma CAD/SIOC hardcoded o dati reali.

## Perimetro scelto dopo il gate05

05DONE su3712ed6, CI40844540287/release40844543949 tutti success.06 avviata.
Prima di cambiare il modello generata legacy-storage-v1.json usando realmente
l'adapter05, source/proof/audit dal DB; helper e comando nel dossier. Solo chiave
pubblica0xaa×64 e metadati SYNTHETIC. Risoluzione helper verificata contro tutti
pin/checksum qualificati; un primo lock rigenerato latest è stato rifiutato prima
di generare fixture, nessun manifest/lock prodotto alterato.

Implementazione: consenso PROCEED tipizzato con scope non vuoto di trace assenti
e UTC; nessun default implicito, ABORT non rappresentabile come consenso durevole.
Validazione dei metadati separata dalla decisione applicativa di approvazione.
Nuovo costruttore SourceSubmission con consenso; SourceSubmissionParts invariato.
Wire esteso additivo con campo assente per dati legacy, per preservare byte/MAC.

Formato storage2 dichiara la nuova capacità: nuovi DB2, lettura1/2; il vecchio
reader05 deve rifiutare2 come versione non supportata. Su1 nessuna scrittura del
nuovo consenso senza upgrade esplicito. Upgrade1→2 soltanto su richiesta,
transazione esclusiva che cambia la versione fisica, senza riscrivere snapshot,
indici, source/proof o audit e senza ri-firmare dati. Richiede istanze precedenti
chiuse; non è una sincronizzazione fra versioni/postazioni. Open non migra.
Commit con esito upgrade ambiguo è tecnico esplicito e richiede riapertura.
Migrazioni ulteriori e backup/restore completo rimangono nelle task competenti.

Test: payload/mac legacy congelati, presenza0/assente/nonzero dopo processo
restart, UTC/hash/reference/namespace/periodo/stato, consenso valido/invalido/
ABORT, vecchie revisioni immutabili, nuova versione/upgrade idempotente e rollback.
File: src/domain/persistence{.rs,/consent.rs,/source_wire.rs}, port errori,
protezione esistente (nessun algoritmo nuovo), adapter SQLite e test metadata.
