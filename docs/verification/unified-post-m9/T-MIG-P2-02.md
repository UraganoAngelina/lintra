# T-MIG-P2-02 — Proposta envelope e taxonomy

14 novembre2026. **IN_PROGRESS — proposta tecnica pronta, non ratificata**.
Prerequisito P2-01 DONE. Nessun DONE per i soli test della reference; G-MIG-2
NOT_VERIFIED e nessun bridge P3 avviato.

[Contratto reviewabile](../../migration/ipc-v1/CONTRACT.md),
[parametri](../../migration/ipc-v1/proposal.json) e codec/vettori adiacenti.
Proposti JSON UTF-8 su raw Tauri, v1 esplicita,1MiB/messaggio e profondità32,
correlazione1..64ASCII, root strict e9errori tecnici senza testo arbitrario.
Interi grandi come testo esatto, esiti normativi separati dai guasti IPC.
Non sono soglie normative/DEC-14 o limiti ai record/report Rust.

Controllate le vere API Request/InvokeBody/Response di Tauri2.12.5 e la firma
TS invoke installata. Le pagine docs.rs non erano accessibili; il sorgente
locale pinned è la fonte verificata e identificata dal receipt. Nessuna API,
feature o dipendenza di prodotto aggiunta. Il codec non concede authority o
valida i payload specifici delle task successive.

Reference Python stdlib: **71vettori PASS, exit0**, incluse dimensioni±1byte,
UTF-8 multibyte, profondità/duplicati/surrogate, limiti precisione, versione e
shape, errori non contenenti path/testo libero. Non è una prova nativa Tauri,
Rust↔TS o PHI/flood. Il consumo memoria del framework prima del body handler
non viene descritto come già bounded da questo codec.

Acceptance richiede “Wire schema version, compatibilità, limiti byte e casi
valid/invalid/boundary approvati”. I nuovi parametri e la forma wire non sono
coperti dalla sola ratifica D01–D15: questa proposta resta da ratificare prima
di considerareP2-02 DONE. G-MIG-2 richiede inoltre tutte le taskP2 e il proprio
checkpoint P2-10; la proposta non concede ingresso al codice bridge.
