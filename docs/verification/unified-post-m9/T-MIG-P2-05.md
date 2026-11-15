# T-MIG-P2-05 — Risultati paginati, offset e filtri

14 novembre2026. **DONE nel perimetro design**, G-MIG-2 NOT_VERIFIED.
Prerequisiti P2-02/03 DONE. [Contratto](../../migration/ipc-v1/PAGING.md) e
reference integrati: max128/default64righe, due pagine,1MiB intero envelope,
query/righe opache e owner-bound, ordinali logici e byte offset solo Rust.

Filtri esatti AND, ordine relativo JSONL, report/summary catturati. Coda
malformata o conteggi/scope errati falliscono tecnicamente anche se il primo
match era già trovato. Nessun dataset o indice in JS, righe senza actual,
raw_hex/context/identity; preview metadata marcate esplicitamente incomplete.
Query nuova o pagina espulsa invalida i token; fallimenti non pubblicano
candidati/pagine parziali. Rust mantiene file/indici e ogni validazione normativa.

70casi PASS/exit0; [receipt con SHA](T-MIG-P2-05-checks.json). Tempfile sintetico,
confini, filtri/owner/handle, mismatch e nonce/collisioni, pagine in ordine,
preview UTF-8 e size intera. JSONL invariato dopo lettura. Non è un port del
decoder normativo né una qualifica Tauri/worker reale: integrazione e stress
restano ai gate del piano. Nessuna suite core, CI o VM rieseguita.
**41/128 DONE (32,0%)**. Prosegue **T-MIG-P2-06**, poi export/security/Full.
