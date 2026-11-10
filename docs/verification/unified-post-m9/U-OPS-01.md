# U-OPS-01 — Modello operativo interno

Data: 2026-11-10, Europe/Rome. Prerequisito U-GOV-03 `2cd4bdf`.
Stato: DONE per contratto di scope operativo. G-OPS0 aggregato NOT_VERIFIED.

docs/INTERNAL_USE.md recepisce D02/D03/D05/D06/D07/D14: autore e colleghi
autorizzati, app locale mono-PC, root per account Windows/Linux personale,
app perMachine e run non-admin, HMAC/chiave invariati, no nuova crypto/password
custom/TTL, Backup-B e restore isolato. Non crea gate enterprise. G-REAL resta
solo per specifiche autorizzazioni aziendali non documentate, se applicabile.

La tabella distingue installazione/data root Tauri **da consegnare** da CLI,
library e Slint esistenti: nessun per-account provisioning, installer, dialogo
shared o workflow CAD UI è dichiarato qualificato per effetto del documento.
Le prove reali sono U-OPS-03/04/06/08 e MIG, non questa task di scope.

PERSISTENCE_OPERATIONS non richiede più ratifica di cifratura o retention già
decise; preserva le procedure qualificate e ne mantiene i limiti effettivi.
Consuntivo 140/140, storage e interfacce runtime invariati.

Verifica: confronto esplicito con tabella D01–D15/master §2 e codice corrente
CLI/bundle, Slint Trace/LocalOnly e key provider già ispezionati; diff check.
Nessuna modifica di codice richiede test/CI nuovi. Nessuna qualifica nativa
attribuita ai nuovi percorsi ancora da creare. File: INTERNAL_USE,
PERSISTENCE_OPERATIONS, stato unificato e dossier. SHA nella cronologia Git.
Next READY U-OPS-02; AUTO_CONTINUE, nessun blocker di scope.
