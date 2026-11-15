# T-MIG-P2-08 — Capability, CSP, path e log

15 novembre2026. **DONE nel perimetro design**; P2-02/07 DONE. Threat model
già approvato ADR-MIG-003, attuato nella [matrice IPC](../../migration/ipc-v1/SECURITY.md)
con obblighi negativi attribuiti ai task implementativi esistenti. Nessun nuovo
consenso/crypto/enterprise gate; G-MIG-2 NOT_VERIFIED.

Audit locale exit0 delle ACL generate e della configurazione effettiva dello
spike: capability solo locale/probe, runtime_probe/render_ack, niente permission
generiche concesse, devtools off e CSP restrittiva. Sorgenti runtime/config/lock
identici a608ccdc; digest delle receipt native P1-04 Linux/Win11 verificati.
Negative invoke/payload/CSP e0TCP/0HTTP sono evidenze precedenti riusate nello
scope corretto, non nuove esecuzioni native. [Receipt SHA](T-MIG-P2-08-checks.json).

P2-02..07 forniscono i negativi wire, owner/stato/stale/bounded e consenso
export non forgiabile via DTO. Nessuna operation C1/PERSIST/approve ammessa.
Path e writer restano Rust, testi riservati solo nel dettaglio richiesto e mai
in persistenza web/log. Nessun file runtime modificato o CI/VM avviata.
Il futuro bridge deve qualificare le sue permission/handler e i negativi reali
inP3/P7: questo dossier non gli attribuisce il PASS dello spike.

**44/128 DONE (34,4%)**. Successive READY: P2-09 parity e P2-11 Full;
approvazione complessiva e codice bridge restano subordinati aP2-10/G-MIG-2.
