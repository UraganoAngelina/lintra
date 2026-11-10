# U-OPS-02 — Inventario delle superfici locali

Data: 2026-11-10, Europe/Rome. Prerequisito U-OPS-01 `c42d44e`.
Stato: DONE per inventario implementato/target separati. G-OPS0 non promosso
a qualifica tecnica per-account di una GUI Tauri ancora assente.

docs/LOCAL_DATA_INVENTORY.md mappa input, schema/reference, DB/journal/temp SQL,
chiave, spool CLI/Slint, cache Rust, temporanei XLSX, report/part/receipt,
budget, log, config, WebView futura e copie. Ogni riga indica owner, contenuto,
durata e qualifica ancora richiesta, senza leggere dati reali.

Riscontri effettivi: CLI NamedTempFile e close esplicito; Slint TempDir
lintra-analysis-* owned dal risultato, JSONL e part interni, cache128;
XLSX UI usa with_temp_dir privato, CLI il temp OS; AtomicReportFile part nella
destinazione con create_new; SQLite DELETE/EXTRA e temp_store=FILE. Il report
normativo può contenere dati selezionati, non va chiamato log anonimizzato.

Il prodotto non ha ancora Tauri/React/WebView/web storage. La policy vieta
PHI persistenti nel renderer e lascia gli oggetti di preflight/dati/indici a
Rust; i test WebView reali rimangono MIG-P7, non vengono dedotti dall'assenza.
Root Windows/Linux target e cartelle per-account sono esplicite ma **non già
provisionate** da questa task. U-OPS-03 e le task MIG restano necessarie.

Delta solo documentale: inventario, dossier e stato unificato. Controlli:
lettura dei simboli/file elencati e confronto al master/ratifica, diff check.
Runtime invariato, nessuna CI nuova richiesta/attribuita. Test pertinenti di
spool/worker/protection già eseguiti nella suite locale U-FIX-03 sullo SHA
specificato; non sono una nuova prova di Windows11/account o WebView.
SHA task nella cronologia del dossier. Next READY indipendente U-NORM-01;
U-OPS-03/04 rimangono READY per backend/prove applicabili, senza bloccare MIG.
