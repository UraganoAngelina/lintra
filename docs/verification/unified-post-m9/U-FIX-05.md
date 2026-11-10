# U-FIX-05 — Ignored e fonte originale dell'audit

Data: 2026-11-10, Europe/Rome. Prerequisito U-GOV-02 `bf86511`.
Stato: DONE per inventario, prove esplicite e confronto della fonte.
G-HARD aggregato ancora NOT_VERIFIED; nessuna nuova qualifica normativa totale.

## GAP-17: fonte ora reperita

La fonte originale importata in FONTI/RETROSPECTIVE_ENGINEERING_AUDIT_ORIGINAL.md
ha SHA256 `e8d1e999b20a65d9f58b66a7e0058087d2e28359820dd55b2cfd7aa5154e81b7`.
Il mandato effettivamente incollato all'audit ha 970 righe contro le 112 della
fonte: il suo digest conservato `24fc3db2...` è stato riconfermato sul file
originale, letto senza modificarlo. I testi non sono byte-identici: il mandato
incollato è più dettagliato. Il confronto seguente è semantico, non una falsa
attestazione d'identità. Provenienza e digest in U-FIX-05-provenance.json.

| Requisito della fonte breve | Mandato ampliato / riscontro nel report post-M9 | Delta materiale |
|---|---|---|
| Ricostruire stato, handoff, contratti e prove reali | Mandato §§1–4; report §§2–4, baseline.json e fonti | Nessun requisito nuovo |
| SQLite atomicità/CAS/rollback/ambiguità/crash/namespace/versioni | Mandato §7; report §6.1, ledger e suite storage | Nessun requisito nuovo |
| Riesaminare specificamente 4e44703, senza giudizio assunto | Mandato §§3/7; report §6 cita il diff e zero/un vincitore con retry esplicito risolto | Eseguito, nessuna nuova correzione dedotta |
| Compiler/DSL, tipi/join/determinismo/errori/no norme implicite | Mandato §8; report §§3/6.2 | Nessun requisito nuovo |
| Engine/CAD/SIOC e capability future distinte da eseguibili | Mandato §§5/9; report §§4/6.3, matrici profili | Nessuna conformità universale affermata |
| SHA/identità/reference boundary e evidenza parziale | Mandato §10; report §§5/6.4, reference suite | Nessun requisito nuovo |
| Errori/audit/memoria/privacy/API | Mandato §§11–13; report §§5/6/8 | Limiti e rischi espliciti, non nuova promessa |
| Invarianti → codice/test → edge cases → classificazione | Mandato §6; report §§3/5/6, due RED/GREEN SQLite | Nessun requisito nuovo |
| Correzioni minime compatibili; non cambiare test per garanzie inesistenti | Mandato §§2/14; report §§5/7, patch af3bccc | Nessun refactor aggiuntivo richiesto |
| Gate locali/nativi, nessun PASS senza prove, report finale | Mandato §§16–18; report §§7/9 e dossier | Evidenze presenti, limiti preservati |
| Single-agent, priorità audit, ripresa dopo completamento | Mandato §19/conclusione; audit concluso, nuova ratifica post-M9 ora attiva | Nessuno stop residuo o consenso da richiedere |

Esito GAP-17: confronto di provenienza **risolto**, nessun requisito materiale
aggiuntivo individuato nella fonte breve. Non si riapre l'audit completo e
non si modifica retroattivamente AUD-M9-010, che documentava un'assenza reale
al momento dell'audit. Il criterio non dimostra assenza di tutti i difetti.

## Sette ignored Linux: trattamento esplicito

| Test | Evidenza / stato effettivo |
|---|---|
| process_worker | Helper invocato dai parent crash/contesa nella suite workspace U-FIX-03; non lanciare senza parametri |
| metadata_restart_worker | Helper invocato dai parent upgrade/restart, stessa suite |
| export_native_snapshot_fixture | Due esecuzioni esplicite positive nei log CI42823787199 su 4c169cd; dati sintetici per OS opposto |
| import_opposite_native_snapshot_fixture | Due esecuzioni esplicite positive e job import reciproci nella stessa CI; non equiparate a reopen locale |
| measure_full_cad83_history_application | Misura separata storica post-m9-audit/performance.json; NON rieseguita qui, nessuna soglia o PASS nuovo |
| original_sioc_workbook_is_imported_through_the_production_adapter | Rieseguito esplicitamente sul workbook ricevuto, 1 pass su 539b9bc |
| xlsx_memory_probe | Rieseguito esplicitamente con 10.000 righe sintetiche, 1 pass su 539b9bc; non una soglia di prodotto |

Windows ha inoltre l'helper console cfg(windows); il numero sette non viene
spacciato per totale universale di tutti gli OS. Gli ignored della suite
ordinaria restano ignored: non vengono riclassificati come esecuzioni ordinarie.

## Comando e risultati nuovi

Ambiente: LINTRA_SIOC_XLSX punta al workbook normativo ricevuto sotto schemas/xlsx;
LINTRA_XLSX_PROBE_ROWS=10000. Comando:
`cargo test -p lintra --locked --offline --test schema_importer --test xlsx_writer -- --ignored --nocapture`.
U-FIX-05-explicit-ignored-linux.json e log.gz: exit0, **2 pass, zero fail**,
446 input invariati. L'importer verifica il digest esatto del workbook,
135 campi, 130 formule cached e otto aggiornamenti di note; le capability full
non supportate restano rifiutate. Non sono dati paziente né nuovi dataset.
Probe XLSX: 10.000 righe, JSONL 3.418.894 byte, XLSX 282.343 byte, VmHWM
17.072 KiB; misura descrittiva di questo processo, nessuno SLA.

Delta: dossier, log/JSON/provenienza e stato unificato. Nessun sorgente/test,
golden o norma modificati. CI nuova non richiesta per questo delta documentale;
CI42829144043 sullo SHA539b9bc è pendente per il diverso delta U-FIX-03.
SHA del dossier nella cronologia Git. Next READY U-FIX-04, AUTO_CONTINUE.
