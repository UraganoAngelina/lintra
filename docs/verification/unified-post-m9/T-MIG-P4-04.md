# T-MIG-P4-04 — Worker nativo e start LocalOnly

17 novembre 2026. **WAITING_GATE — implementazione e verifiche locali PASS;
CI nativa Linux/Windows pendente.** G-MIG-4 resta aperto.

[Implementazione, invarianti e limiti](P4-native-lifecycle/IMPLEMENTATION.md).
[Ricevuta locale, digest sorgenti e comandi](P4-native-lifecycle/local-checks.json).
Baseline d78e8d0; commit del delta risolvibile con git log di questo dossier.

32 test Rust nativi PASS / 1 ignored GTK preesistente, 28 Node PASS,
25 regressioni golden/report PASS; fmt, Clippy con warning negati, lint,
TypeScript strict, build e smoke nativo React/IPC PASS. Le race usano thread
reali e finalizzazione trattenuta; il test del worker confronta il JSONL con
il servizio diretto. Golden esistenti verificati byte per byte e invariati.
Log compressi conservati con il primo FAIL Clippy corretto.

Nessun DONE o PASS Windows dedotto dalla baseline; attesa la CI desktop dello
SHA del blocco. Nessuna nuova task dopo commit/push e qualifica CI, secondo
lo stop richiesto dall'autore. Nessuna VM o installer in questo checkpoint.
