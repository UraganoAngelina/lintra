# T-MIG-P0-04 — Scope LocalOnly+Full e capability

Data11 novembre2026. Stato: **DONE documentale**.
Prerequisito P0-03 DONE, ADR-MIG-001/commit92d6d3d. Ingresso92d6d3d clean;
source baselinecb70bee, CI42990775714 qualificata, runtime invariato.

MIGRATION_SCOPE_PARITY.md recepisce lo scope dell'autore D01/D02 e DEC-MIG-07:
LocalOnly e Full multi-tracciato obbligatori, CAD storico con le9 U-CAD e SIOC
D11 con il bridge reale. Sedici capacità hanno stato attuale, task/percorso e
accettazione, senza equiparare library/CLI/Slint a disponibilità React.

Lettura del codice: service.rs validate/validate_with_parameters/typed,
registry.rs risoluzioni ordinarie e storiche, snapshot_validation.rs,
historical_validation.rs e analysis.rs Slint Trace/LocalOnly. La matrice
instrada ciascuna policy al producer corretto; non cambia firme, rimuove
capability o simula Full con singole validate. I profili regionali/deferred
restano nel perimetro effettivo; i26 campi SIOC non diventano il workbook135.

Criteri di parità: confronto adapter/Rust su esito e completezza distinti,
JSONL/audit/contatori/ordine, cancel/truncated/errori tecnici, cinque filtri AND,
dettagli bounded, export canonici, eventi run/report e ownership. Nessuna
approval anticipata alle89 MIG; storico segue i propri gate successivi.
Non vengono inventati limiti IPC o tempi DEC-14: contratti e prove inP2/P7.

La matrice approvata è il recepimento delle scelte già ratificate in chat e
ADR-POST-M9-001, non una nuova autorizzazione o un gate PASS. Controlli:
fonti/prompts e simboli attuali, collegamenti locali, check_unified_plan.py,
diff check e source immutato. Nessun test/CI nuovo necessario per soli documenti.
File: matrice, status e dossier. G-MIG-0 NOT_VERIFIED.
Next READY P0-05 (OS/offline), poi06/07/08; AUTO_CONTINUE.
