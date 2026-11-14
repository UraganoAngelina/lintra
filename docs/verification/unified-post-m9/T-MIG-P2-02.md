# T-MIG-P2-02 — Proposta envelope e taxonomy

14 novembre2026. **DONE — ratifica esplicita dell’autore e84vettori PASS**.
Prerequisito P2-01 DONE. [Ratifica integrale](../../migration/ipc-v1/RATIFICATION_2026-11-14.md) acquisita
con sette precisazioni vincolanti. G-MIG-2 NOT_VERIFIED; nessun bridge P3 avviato.

[Contratto reviewabile](../../migration/ipc-v1/CONTRACT.md),
[parametri](../../migration/ipc-v1/proposal.json) e codec/vettori adiacenti.
Ratificati JSON UTF-8 su raw Tauri, `schemaVersion = 1` su envelope eDTO,
1MiB/messaggio completo in entrambe le direzioni e profondità32,
correlazione1..64ASCII, root strict e9errori tecnici senza testo arbitrario.
Interi grandi come testo esatto, esiti normativi separati dai guasti IPC.
Non sono soglie normative/DEC-14 o limiti ai record/report Rust.

Controllate le vere API Request/InvokeBody/Response di Tauri2.12.5 e la firma
TS invoke installata. Le pagine docs.rs non erano accessibili; il sorgente
locale pinned è la fonte verificata e identificata dal receipt. Nessuna API,
feature o dipendenza di prodotto aggiunta. Il codec non concede authority o
valida i payload specifici delle task successive.

Reference Python stdlib aggiornata: **84vettori PASS, exit0**, incluse dimensioni±1byte,
UTF-8 multibyte, profondità/duplicati/surrogate, limiti precisione, versione e
shape, errori non contenenti path/testo libero. Non è una prova nativa Tauri,
Rust↔TS o PHI/flood. Il consumo memoria del framework prima del body handler
non viene descritto come già bounded da questo codec.

Acceptance soddisfatta dalla ratifica e dai nuovi test di confine, inclusi
DTO versionati e categorie `protocol`/`technical` verificate sul codice.
Nessun dataset completo, record sanitario raw o bulk di diagnostici in IPC;
query/paging/handle Rust rimangono obbligatori. La reference non sostituisce
il controllo dei DTO specifici o le prove applicative Rust↔TS.

Il primo receipt71vettori identifica0f65539 e resta storico.
[Evidenze ratificate attuali](T-MIG-P2-02-ratified-checks.json):84vettori con
exit code e SHA. Nessuna suite core, CI o VM rieseguita dopo la sola ratifica.

Compatibilità additiva soltanto se documentata e validata; nessun alias per
`wire_version` della proposta non ratificata. I dettagli ordinari delle task
successive sono autorizzati senza ulteriori richieste. G-MIG-2 mantiene i propri
requisiti complessivi e il checkpointP2-10 prima del bridge.
**38/128 DONE (29,7%)**. Prossima READY **T-MIG-P2-03**, poiP2-04.
