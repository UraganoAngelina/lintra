# T-MIG-P2-06 — Dettaglio e domini catturati

14 novembre2026. **DONE nel perimetro design**, G-MIG-2 NOT_VERIFIED.
Prerequisito P2-05 DONE. [Contratto](../../migration/ipc-v1/DETAIL.md) e
reference collegati a paging/codec: un diagnostico selezionato, componenti
testuali16KiB, contesto e domini a pagine64, handle owner/query/run-bound.
Copertura campo da schema catturato; niente record intero/raw_hex/dataset IPC.
Guide Full risolte sul trace del diagnostico. JSONL/export canonici immutati.

Domini dichiarati e reference riusano le API Rust verificate: cattura immutabile,
encoding/literal esistenti, digest e disponibilità espliciti. Verified-empty
distinto da degraded/unavailable/not_captured/unsupported/cancelled. Regex non
enumerata, fallback e condizioni etichettati, range arbitrari non ridotti a JS.

**58casi PASS/exit0**, [receipt con SHA](T-MIG-P2-06-checks.json): integrazione
codec/paging su fixture sintetica, Unicode/offset/limiti, pin e stale, context64,
privacy del record, guide per trace, digest, reader falliti e fonti mutate.
Un’aspettativa di test errata sul prefisso byte UTF-8 è stata corretta;
il tentativo fallito non è stato contato come PASS.

Reader Rust iniettati nella reference: nessuna qualifica del bridge, worker,
DOM o provider nativo. Queste prove restano ai task/gate già previsti; nessuna
suite core, CI o VM ripetuta. **42/128 DONE (32,8%)**. Segue **T-MIG-P2-07**.
