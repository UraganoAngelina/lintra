# T-MIG-P2-01 — Ownership processo e boundary

14 novembre2026. **DONE nel perimetro architetturale già ratificato**;
G-MIG-2 NOT_VERIFIED. Prerequisito P1-06 DONE/G-MIG-1 GO su7320b88.

[Contratto e diagrammi](../../migration/TAURI_PROCESS_OWNERSHIP.md) collegano
ADR-MIG-001/003 e D01–D15 ai veri entrypoint della library e agli owner legacy.
Non si richiede nuovamente la ratifica dello stack o dei boundary già approvati.
Nessuna firma/primitive wire nuova viene dichiarata definitiva o approvata.

Ricostruiti ValidationService e request LocalOnly/Full, progress sincrono,
CancellationToken, WorkerHandle/AnalysisReport/ReportIndex, servizi storici e
review Serialize-only, repository/protector e writer atomici. Status, completezza
e evaluation restano separati: Partial/Unknown non diventano enum RunStatus
inventati. Il renderer non riceve file/indici interi, SQL, chiavi o autorità C1.
Un job posseduto, terminale distinto dal progresso, cancellazione e join reali;
nessun riuso della crate Slint nel nuovo adapter o server locale aggiuntivo.

Verifica mirata dei manifest e del lock core: nessuna dipendenza tauri/wry/
webview2-com nella library; lo spike ha workspace/lock indipendente. Moduli
domain/schema/engine senza import WebView; riferimenti ai simboli esistenti
controllati nel codice, non dedotti dai nomi. Receipt SHA consultabile accanto
al dossier. Solo documenti: nessun bridge, dipendenza, tipo pubblico, norma,
fixture, sorgente prodotto o CI modificato. Nessuna suite Rust/VM necessaria.

L’accettazione “diagrammi e boundary ratificati” usa l’autorità già acquisita
degli ADR; non inventa una nuova firma dell’autore sui futuri DTO. Restano
P2-02..11 e approvazione G-MIG-2 prima del codice applicativoP3.
**37/128 DONE (28,9%)**. Prossima READY **T-MIG-P2-02**.
