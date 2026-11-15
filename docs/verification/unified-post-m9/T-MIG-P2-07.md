# T-MIG-P2-07 — Export, path e overwrite

15 novembre2026. **DONE nel perimetro design**; prerequisito P2-03 DONE.
[Contratto](../../migration/ipc-v1/EXPORT.md): selezione/conferme native, path
solo Rust, piano monouso legato a report/RunRef/formato/destinazioni, warning
shared/undetermined distinto da overwrite, locale senza nuova conferma.
Writer canonici e policy D04 invariati; nessuna transazione multi-file promessa.

**60casi PASS/exit0**, [receipt SHA](T-MIG-P2-07-checks.json): callback sintetici
collegati a codec/RunRef, direct invoke abusivi, owner, stale/replay, scope
destinazioni mutato, report catturato, conferme tardive e pubblicazioni parziali.
Corretto il catch che trasformava IPC_STALE_CONTEXT in errore tecnico generico;
nessun PASS attribuito al tentativo fallito. Nessuna scrittura di dati reali.

Non è una qualifica Tauri/dialogo/FS/writer. Protezione di tutti gli input Full,
job native, secondo preflight, parity e fault injection restano alle task
implementative previste. G-MIG-2 NOT_VERIFIED; nessuna CI/VM/suite core ripetuta.
**43/128 DONE (33,6%)**; prosegue **T-MIG-P2-08**.
