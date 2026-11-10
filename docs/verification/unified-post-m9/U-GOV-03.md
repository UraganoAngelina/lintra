# U-GOV-03 — Ratifica recepita e prompt sincronizzati

Data: 2026-11-10, Europe/Rome. Prerequisito U-GOV-02 `bf86511`.
Stato: DONE per il recepimento documentale; G-U0 aggregato NOT_VERIFIED.

Creati ADR-POST-M9-001 e DEC-SIOC-HIST-02 con fonte, data, distinzione fra
decisione e gate, supersessione prospettica e tracciabilità alla tabella D01–D15
integrale. Nessuna nuova approvazione richiesta. Fonte normativa indipendente
ancora da esaminare in U-NORM-05: nessuna falsa attestazione.

Copiato il kit documentale in docs/unified-post-m9 con SOURCE_MANIFEST.json:
master, zero, fonti storiche e 128 prompt conservano gli ID. Nelle 89 copie MIG
rimossa la clausola residua «ratifiche nuove necessarie solo per il Decision Pack
post-M9», ora sostituita dal recepimento già avvenuto. Le acceptance specifiche
e i vincoli più dettagliati dei prompt restano validi sotto il master §6.
Il kit dell'autore è immutato; digest originali nel manifest e baseline.

AGENTS/master prompt/ARCH/DET/piano puntano al nuovo atto prevalente per il
solo delta autorizzato. Il consuntivo storico IMPLEMENTATION_STATUS non cambia.
Vecchi report e preflight non vengono riscritti retroattivamente.

Controlli: originali/fonti byte-fedeli tramite SHA-256; 39+89 prompt senza ID
duplicati; nessuna clausola MIG residua di nuova ratifica Decision Pack; diff
check. Nessun sorgente, formato, schema o golden modificato; CI runtime nuova
non richiesta e non attribuita. Commit: cronologia di questo dossier.
Next READY U-GOV-04, AUTO_CONTINUE. Nessun blocker documentale.
