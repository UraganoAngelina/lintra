# T-MIG-P2-09 — Contratto parity e fixture

15 novembre2026. **DONE nel perimetro design**; P2-03..08 DONE.
[Harness definito](../../migration/ipc-v1/PARITY.md),14fixture sintetiche
preesistenti con attesi indipendenti e SHA congelati in parity-fixtures.json.
Confronto interamente byte per byte per producer a clock di test fisso;
CLI ordinaria con campi runtime dichiarati separatamente, nessun golden alterato.

Verifica hash e self-check comparatore PASS/exit0: oracolo cambiato e byte LF
aggiunto rifiutati, copie attese su tempfile confrontate senza produrre dati
nuovi dal motore. [Receipt SHA](T-MIG-P2-09-checks.json). Nessuna parity nativa
eseguita o desunta dal solo confronto; provenienza binario/artefatti è un
obbligo delle task successive, insieme a fault/race e runtime DOM.

Core/CLI/Slint/lock/fixture canoniche invariati; nessuna CI/VM/suite core
ripetuta. **45/128 DONE (35,2%)**. SegueP2-11 Full; P2-10/G-MIG-2 ancora aperto.
