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
