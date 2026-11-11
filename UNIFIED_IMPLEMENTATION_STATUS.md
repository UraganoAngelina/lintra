# Stato unificato post-M9

Aggiornato: 2026-11-10, Europe/Rome. Un agente, una task per volta.
Autorità: [ratifica D01–D15](docs/unified-post-m9/RATIFICATION_2026-11-10.md),
[master v1.0](docs/unified-post-m9/MASTER_PLAN_UNIFICATO.md) e
[grafo completo](docs/unified-post-m9/DEPENDENCY_GRAPH.json).
Consuntivo storico separato: **140/140 DONE**, QG-8/M9 invariati.

G-U0: PASS documentale dopo validazione U-GOV-04. Gate runtime G-HARD,
G-OPS0/1/2, G-SIOC0/1, G-MIG-0..9, G-CADUI0..3, G-DUR, G-PERF e G-REL:
NOT_VERIFIED. G-REAL: autorizzazioni effettive per specifici dati, non presunte.
CI audit non qualifica alcun nuovo delta runtime.

| ID | Prerequisiti | Stato | File / evidenze | SHA / verifiche e CI | Gate / blocchi |
|---|---|---|---|---|---|
| U-GOV-01 | — | DONE | [Dossier](docs/verification/unified-post-m9/U-GOV-01.md) | 8e6bc55; Baseline 7e2ff155; CI42789338814 af3bccc Linux/Windows success | Acceptance documentale PASS; G-U0 aggregato PASS alla chiusura 04 |
| U-GOV-02 | U-GOV-01 | DONE | [Dossier](docs/verification/unified-post-m9/U-GOV-02.md) | bf86511; Matrice fonti storiche/correnti, nessun delta runtime | Acceptance documentale PASS; G-U0 aggregato PASS alla chiusura 04 |
| U-GOV-03 | U-GOV-02 | DONE | [Dossier](docs/verification/unified-post-m9/U-GOV-03.md) | 2cd4bdf; Ratifica, 140 fonti fedeli, 39+89 prompt sincronizzati | Acceptance documentale PASS; G-U0 aggregato PASS alla chiusura 04 |
| U-GOV-04 | U-GOV-03 | DONE | [Dossier](docs/verification/unified-post-m9/U-GOV-04.md) | Cronologia dossier; Validatore: 128 nodi, dipendenze e fonti; nessun delta runtime | Acceptance documentale PASS; G-U0 aggregato PASS alla chiusura 04 |
| U-FIX-01 | U-GOV-01 | DONE | [Dossier, log nativi e digest](docs/verification/unified-post-m9/U-FIX-01.md) | 4c169cd; Linux38 pass/4 ignored; CI42823787199 success, regressioni 3 volte per OS verificate | Acceptance PASS; G-HARD PASS su CI42990775714 |
| U-FIX-02 | U-GOV-03 | DONE | [Dossier, contratto e log](docs/verification/unified-post-m9/U-FIX-02.md) | 52ec340; Linux 18 pass/1 ignored; delta solo documentale, CI nuova non richiesta | Acceptance PASS; G-HARD PASS su CI42990775714 |
| U-FIX-03 | U-GOV-03 | DONE | [Budget backend/CLI e CI nativa](docs/verification/unified-post-m9/U-FIX-03.md) | 4864c34/539b9bc sorgenti identici; locale1073pass/7ignored; CI42829144043 success, nuovi oracle 3 volte/OS | Acceptance backend/CLI PASS; G-HARD PASS; React budget in MIG-P5-05 |
| U-FIX-04 | U-GOV-03 | DONE | [Tree controllata e qualifica nativa](docs/verification/unified-post-m9/U-FIX-04.md) | cb70bee;CI42990775714 success, test ACL Windows3 volte;1096 pass locali | Acceptance PASS; G-HARD PASS |
| U-FIX-05 | U-GOV-02 | DONE | [Ignored e confronto fonte](docs/verification/unified-post-m9/U-FIX-05.md) | 539b9bc; due ignored espliciti pass, workbook SHA oracle confermato; nessun delta runtime | Acceptance PASS; G-HARD PASS su CI42990775714 |
| U-OPS-01 | U-GOV-03 | DONE | [Modello uso interno](docs/verification/unified-post-m9/U-OPS-01.md) | Scope ratificato; documenti soltanto, provisioning Tauri ancora da qualificare | Acceptance di scope PASS; G-OPS0 aggregato NOT_VERIFIED |
| U-OPS-02 | U-OPS-01 | DONE | [Inventario dati/report/spool](docs/verification/unified-post-m9/U-OPS-02.md) | Riscontri statici dei percorsi attuali, target Tauri separati; nessun delta runtime | Acceptance inventario PASS; G-OPS0 tecnico ancora NOT_VERIFIED |
| U-OPS-03 | U-OPS-02 | WAITING_INTEGRATION | [Preflight permessi e continuità](docs/verification/unified-post-m9/U-OPS-03.md) | Controlli backend esistenti qualificati cb70bee; root Tauri per-account non ancora collegata | G-OPS1 NOT_VERIFIED; handoff a MIG per path/worker reali |
| U-OPS-04 | U-OPS-02 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/INTEGRAZIONE_39/U-OPS-04.md) | —; Non eseguite | G-OPS1 NOT_VERIFIED |
| U-OPS-05 | U-OPS-01 | DONE | [Custodia e drill sintetici](docs/verification/unified-post-m9/U-OPS-05.md) | 556c5c6;18pass, file32B/copia corretta/KeyMismatch/restore/sentinelle; crypto invariata | Acceptance provider/runbook PASS; G-OPS1 aggregato NOT_VERIFIED |
| U-OPS-06 | U-OPS-03, U-OPS-05 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/INTEGRAZIONE_39/U-OPS-06.md) | —; Non eseguite | G-OPS2 NOT_VERIFIED |
| U-OPS-07 | U-OPS-01, U-OPS-02 | DONE | [Gestione manuale e conservazione](docs/verification/unified-post-m9/U-OPS-07.md) | D05 documentata con riscontri su archive/report/spool; delta solo documentale | Acceptance gestione manuale PASS; G-OPS2 tecnico NOT_VERIFIED |
| U-OPS-08 | U-OPS-06 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/INTEGRAZIONE_39/U-OPS-08.md) | —; Non eseguite | G-OPS2 NOT_VERIFIED |
| U-OPS-09 | U-OPS-03, U-OPS-04, U-OPS-05, U-OPS-07, U-OPS-08 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/INTEGRAZIONE_39/U-OPS-09.md) | —; Non eseguite | G-REAL NOT_VERIFIED |
| U-NORM-01 | U-GOV-03 | DONE | [PK e deferred inventariati](docs/verification/unified-post-m9/U-NORM-01.md) | 10dc66b;1 oracle pass;5 tuple identiche .1/.2/FROZEN/component;83+70+3 deferred | G-SIOC0/1 NOT_VERIFIED; D11 enforcement resta richiesto |
| U-NORM-02 | U-NORM-01 | DONE | [Freeze dataset D12](docs/verification/unified-post-m9/U-NORM-02.md) | Otto fonti con hash/versioni;12 assenze SIOC dichiarate definitive; nessun sourcing o delta runtime | Acceptance inventario PASS; G-SIOC0/1 NOT_VERIFIED |
| U-NORM-03 | U-NORM-01, U-NORM-02, U-NORM-05 | DONE | [Servizio normativo e profilo](docs/verification/unified-post-m9/U-NORM-03.md) | cb70bee;1096 pass/7 ignored;CI42990775714 success,21 oracle3 volte per OS | Acceptance backend PASS; G-SIOC1 aggregato attende04 e bridge usabile |
| U-NORM-04 | U-NORM-03 | WAITING_INTEGRATION | [Oracle nativi e residuo UI](docs/verification/unified-post-m9/U-NORM-04.md) | cb70bee;21 oracle3 volte/OS,JSONL canonico; frontend/contesto amministrativo ancora da collegare | G-SIOC1 NOT_VERIFIED; riprendere dopo G-MIG-4 |
| U-NORM-05 | U-NORM-01 | DONE | [Addendum autore e oracle](docs/verification/unified-post-m9/U-NORM-05.md) | DEC-SIOC-HIST-02-A01; fonte C20/C18 e PK4.19.1 confermata; check piano locale | G-SIOC0 documentale PASS; G-SIOC1 NOT_VERIFIED |
| U-UI-01 | U-GOV-04 | DONE | [Ponte 89 MIG/post-audit](docs/verification/unified-post-m9/U-UI-01.md) | Overlay master §6, sorgenti76fa08a distinti dalla HEAD/CI; controlli grafo/documenti | Acceptance ponte PASS; G-MIG-0 NOT_VERIFIED |
| U-UI-02 | T-MIG-P9-07 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/INTEGRAZIONE_39/U-UI-02.md) | —; Non eseguite | G-MIG-9 NOT_VERIFIED |
| U-CAD-01 | U-UI-02, U-OPS-02 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/INTEGRAZIONE_39/U-CAD-01.md) | —; Non eseguite | G-CADUI0 NOT_VERIFIED |
| U-CAD-02 | U-CAD-01 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/INTEGRAZIONE_39/U-CAD-02.md) | —; Non eseguite | G-CADUI1 NOT_VERIFIED |
| U-CAD-03 | U-CAD-02, U-OPS-05 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/INTEGRAZIONE_39/U-CAD-03.md) | —; Non eseguite | G-CADUI1 NOT_VERIFIED |
| U-CAD-04 | U-CAD-03 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/INTEGRAZIONE_39/U-CAD-04.md) | —; Non eseguite | G-CADUI1 NOT_VERIFIED |
| U-CAD-05 | U-CAD-04 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/INTEGRAZIONE_39/U-CAD-05.md) | —; Non eseguite | G-CADUI2 NOT_VERIFIED |
| U-CAD-06 | U-CAD-05, U-OPS-04 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/INTEGRAZIONE_39/U-CAD-06.md) | —; Non eseguite | G-CADUI2 NOT_VERIFIED |
| U-CAD-07 | U-CAD-06 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/INTEGRAZIONE_39/U-CAD-07.md) | —; Non eseguite | G-CADUI2 NOT_VERIFIED |
| U-CAD-08 | U-CAD-07 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/INTEGRAZIONE_39/U-CAD-08.md) | —; Non eseguite | G-CADUI3 NOT_VERIFIED |
| U-CAD-09 | U-CAD-08 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/INTEGRAZIONE_39/U-CAD-09.md) | —; Non eseguite | G-CADUI3 NOT_VERIFIED |
| U-PERF-01 | U-GOV-02 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/INTEGRAZIONE_39/U-PERF-01.md) | —; Non eseguite | G-PERF NOT_VERIFIED |
| U-PERF-02 | U-PERF-01, T-MIG-P7-08 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/INTEGRAZIONE_39/U-PERF-02.md) | —; Non eseguite | G-PERF NOT_VERIFIED |
| U-REL-01 | U-UI-02, U-CAD-09, U-NORM-04, U-OPS-08 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/INTEGRAZIONE_39/U-REL-01.md) | —; Non eseguite | G-REL NOT_VERIFIED |
| U-REL-02 | U-REL-01, U-OPS-06, U-REL-03 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/INTEGRAZIONE_39/U-REL-02.md) | —; Non eseguite | G-REL NOT_VERIFIED |
| U-REL-03 | U-OPS-01, U-FIX-01 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/INTEGRAZIONE_39/U-REL-03.md) | —; Non eseguite | G-DUR NOT_VERIFIED |
| T-MIG-P0-01 | U-UI-01 | DONE | [Baseline post-audit qualificata](docs/verification/unified-post-m9/T-MIG-P0-01.md) | cb70bee;CI42990775714 success7 job;HEAD documentale42620cf con520 input runtime identici | Acceptance baseline PASS; G-MIG-0 attendeP0-02..08 |
| T-MIG-P0-02 | T-MIG-P0-01 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P0-02.md) | —; Non eseguite | G-MIG-0 NOT_VERIFIED |
| T-MIG-P0-03 | T-MIG-P0-02 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P0-03.md) | —; Non eseguite | G-MIG-0 NOT_VERIFIED |
| T-MIG-P0-04 | T-MIG-P0-03 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P0-04.md) | —; Non eseguite | G-MIG-0 NOT_VERIFIED |
| T-MIG-P0-05 | T-MIG-P0-03 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P0-05.md) | —; Non eseguite | G-MIG-0 NOT_VERIFIED |
| T-MIG-P0-06 | T-MIG-P0-03 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P0-06.md) | —; Non eseguite | G-MIG-0 NOT_VERIFIED |
| T-MIG-P0-07 | T-MIG-P0-04, T-MIG-P0-05 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P0-07.md) | —; Non eseguite | G-MIG-0 NOT_VERIFIED |
| T-MIG-P0-08 | T-MIG-P0-01, T-MIG-P0-02, T-MIG-P0-03, T-MIG-P0-04, T-MIG-P0-05, T-MIG-P0-06, T-MIG-P0-07 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P0-08.md) | —; Non eseguite | G-MIG-0 NOT_VERIFIED |
| T-MIG-P1-01 | T-MIG-P0-08 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P1-01.md) | —; Non eseguite | G-MIG-1 NOT_VERIFIED |
| T-MIG-P1-02 | T-MIG-P0-08 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P1-02.md) | —; Non eseguite | G-MIG-1 NOT_VERIFIED |
| T-MIG-P1-03 | T-MIG-P1-01 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P1-03.md) | —; Non eseguite | G-MIG-1 NOT_VERIFIED |
| T-MIG-P1-04 | T-MIG-P1-01, T-MIG-P1-02 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P1-04.md) | —; Non eseguite | G-MIG-1 NOT_VERIFIED |
| T-MIG-P1-05 | T-MIG-P1-01, T-MIG-P1-02 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P1-05.md) | —; Non eseguite | G-MIG-1 NOT_VERIFIED |
| T-MIG-P1-06 | T-MIG-P1-02, T-MIG-P1-03, T-MIG-P1-04, T-MIG-P1-05, T-MIG-P1-07 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P1-06.md) | —; Non eseguite | G-MIG-1 NOT_VERIFIED |
| T-MIG-P1-07 | T-MIG-P1-02, T-MIG-P1-04 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P1-07.md) | —; Non eseguite | G-MIG-1 NOT_VERIFIED |
| T-MIG-P2-01 | T-MIG-P1-06 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P2-01.md) | —; Non eseguite | G-MIG-2 NOT_VERIFIED |
| T-MIG-P2-02 | T-MIG-P2-01 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P2-02.md) | —; Non eseguite | G-MIG-2 NOT_VERIFIED |
| T-MIG-P2-03 | T-MIG-P2-02 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P2-03.md) | —; Non eseguite | G-MIG-2 NOT_VERIFIED |
| T-MIG-P2-04 | T-MIG-P2-03 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P2-04.md) | —; Non eseguite | G-MIG-2 NOT_VERIFIED |
| T-MIG-P2-05 | T-MIG-P2-02, T-MIG-P2-03 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P2-05.md) | —; Non eseguite | G-MIG-2 NOT_VERIFIED |
| T-MIG-P2-06 | T-MIG-P2-05 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P2-06.md) | —; Non eseguite | G-MIG-2 NOT_VERIFIED |
| T-MIG-P2-07 | T-MIG-P2-03 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P2-07.md) | —; Non eseguite | G-MIG-2 NOT_VERIFIED |
| T-MIG-P2-08 | T-MIG-P2-02, T-MIG-P2-07 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P2-08.md) | —; Non eseguite | G-MIG-2 NOT_VERIFIED |
| T-MIG-P2-09 | T-MIG-P2-03, T-MIG-P2-04, T-MIG-P2-05, T-MIG-P2-06, T-MIG-P2-07, T-MIG-P2-08 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P2-09.md) | —; Non eseguite | G-MIG-2 NOT_VERIFIED |
| T-MIG-P2-10 | T-MIG-P2-01, T-MIG-P2-02, T-MIG-P2-03, T-MIG-P2-04, T-MIG-P2-05, T-MIG-P2-06, T-MIG-P2-07, T-MIG-P2-08, T-MIG-P2-09, T-MIG-P2-11 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P2-10.md) | —; Non eseguite | G-MIG-2 NOT_VERIFIED |
| T-MIG-P2-11 | T-MIG-P2-03, T-MIG-P2-05, T-MIG-P2-08 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P2-11.md) | —; Non eseguite | G-MIG-2 NOT_VERIFIED |
| T-MIG-P3-01 | T-MIG-P2-10 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P3-01.md) | —; Non eseguite | G-MIG-3 NOT_VERIFIED |
| T-MIG-P3-02 | T-MIG-P3-01 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P3-02.md) | —; Non eseguite | G-MIG-3 NOT_VERIFIED |
| T-MIG-P3-03 | T-MIG-P3-01 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P3-03.md) | —; Non eseguite | G-MIG-3 NOT_VERIFIED |
| T-MIG-P3-04 | T-MIG-P3-01, T-MIG-P3-02 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P3-04.md) | —; Non eseguite | G-MIG-3 NOT_VERIFIED |
| T-MIG-P3-05 | T-MIG-P3-02, T-MIG-P3-04 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P3-05.md) | —; Non eseguite | G-MIG-3 NOT_VERIFIED |
| T-MIG-P3-06 | T-MIG-P3-01, T-MIG-P3-02, T-MIG-P3-03, T-MIG-P3-04, T-MIG-P3-05 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P3-06.md) | —; Non eseguite | G-MIG-3 NOT_VERIFIED |
| T-MIG-P4-01 | T-MIG-P3-06 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P4-01.md) | —; Non eseguite | G-MIG-4 NOT_VERIFIED |
| T-MIG-P4-02 | T-MIG-P4-01 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P4-02.md) | —; Non eseguite | G-MIG-4 NOT_VERIFIED |
| T-MIG-P4-03 | T-MIG-P4-01, T-MIG-P4-02 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P4-03.md) | —; Non eseguite | G-MIG-4 NOT_VERIFIED |
| T-MIG-P4-04 | T-MIG-P4-03 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P4-04.md) | —; Non eseguite | G-MIG-4 NOT_VERIFIED |
| T-MIG-P4-05 | T-MIG-P4-04 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P4-05.md) | —; Non eseguite | G-MIG-4 NOT_VERIFIED |
| T-MIG-P4-06 | T-MIG-P4-04, T-MIG-P4-05 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P4-06.md) | —; Non eseguite | G-MIG-4 NOT_VERIFIED |
| T-MIG-P4-07 | T-MIG-P4-06 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P4-07.md) | —; Non eseguite | G-MIG-4 NOT_VERIFIED |
| T-MIG-P4-08 | T-MIG-P4-07 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P4-08.md) | —; Non eseguite | G-MIG-4 NOT_VERIFIED |
| T-MIG-P4-09 | T-MIG-P4-08 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P4-09.md) | —; Non eseguite | G-MIG-4 NOT_VERIFIED |
| T-MIG-P4-10 | T-MIG-P4-06, T-MIG-P4-08, T-MIG-P4-09, T-MIG-P4-12 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P4-10.md) | —; Non eseguite | G-MIG-4 NOT_VERIFIED |
| T-MIG-P4-11 | T-MIG-P4-06, T-MIG-P2-11 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P4-11.md) | —; Non eseguite | G-MIG-4 NOT_VERIFIED |
| T-MIG-P4-12 | T-MIG-P4-11, T-MIG-P4-05 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P4-12.md) | —; Non eseguite | G-MIG-4 NOT_VERIFIED |
| T-MIG-P5-01 | T-MIG-P4-10 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P5-01.md) | —; Non eseguite | G-MIG-5 NOT_VERIFIED |
| T-MIG-P5-02 | T-MIG-P5-01 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P5-02.md) | —; Non eseguite | G-MIG-5 NOT_VERIFIED |
| T-MIG-P5-03 | T-MIG-P5-02 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P5-03.md) | —; Non eseguite | G-MIG-5 NOT_VERIFIED |
| T-MIG-P5-04 | T-MIG-P5-02 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P5-04.md) | —; Non eseguite | G-MIG-5 NOT_VERIFIED |
| T-MIG-P5-05 | T-MIG-P5-03, T-MIG-P5-04 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P5-05.md) | —; Non eseguite | G-MIG-5 NOT_VERIFIED |
| T-MIG-P5-06 | T-MIG-P5-05 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P5-06.md) | —; Non eseguite | G-MIG-5 NOT_VERIFIED |
| T-MIG-P5-07 | T-MIG-P5-01, T-MIG-P5-06 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P5-07.md) | —; Non eseguite | G-MIG-5 NOT_VERIFIED |
| T-MIG-P5-08 | T-MIG-P5-01, T-MIG-P5-02, T-MIG-P5-03, T-MIG-P5-04, T-MIG-P5-05, T-MIG-P5-06, T-MIG-P5-07, T-MIG-P5-10 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P5-08.md) | —; Non eseguite | G-MIG-5 NOT_VERIFIED |
| T-MIG-P5-09 | T-MIG-P5-03, T-MIG-P4-12 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P5-09.md) | —; Non eseguite | G-MIG-5 NOT_VERIFIED |
| T-MIG-P5-10 | T-MIG-P5-09, T-MIG-P5-06 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P5-10.md) | —; Non eseguite | G-MIG-5 NOT_VERIFIED |
| T-MIG-P6-01 | T-MIG-P5-08 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P6-01.md) | —; Non eseguite | G-MIG-6 NOT_VERIFIED |
| T-MIG-P6-02 | T-MIG-P6-01 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P6-02.md) | —; Non eseguite | G-MIG-6 NOT_VERIFIED |
| T-MIG-P6-03 | T-MIG-P6-01, T-MIG-P6-02 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P6-03.md) | —; Non eseguite | G-MIG-6 NOT_VERIFIED |
| T-MIG-P6-04 | T-MIG-P6-03 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P6-04.md) | —; Non eseguite | G-MIG-6 NOT_VERIFIED |
| T-MIG-P6-05 | T-MIG-P6-04 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P6-05.md) | —; Non eseguite | G-MIG-6 NOT_VERIFIED |
| T-MIG-P6-06 | T-MIG-P6-05 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P6-06.md) | —; Non eseguite | G-MIG-6 NOT_VERIFIED |
| T-MIG-P6-07 | T-MIG-P6-01, T-MIG-P6-03, T-MIG-P6-06 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P6-07.md) | —; Non eseguite | G-MIG-6 NOT_VERIFIED |
| T-MIG-P6-08 | T-MIG-P6-07 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P6-08.md) | —; Non eseguite | G-MIG-6 NOT_VERIFIED |
| T-MIG-P6-09 | T-MIG-P6-02, T-MIG-P6-03, T-MIG-P6-04, T-MIG-P6-05, T-MIG-P6-06, T-MIG-P6-07, T-MIG-P6-08 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P6-09.md) | —; Non eseguite | G-MIG-6 NOT_VERIFIED |
| T-MIG-P6-10 | T-MIG-P6-09, T-MIG-P5-10 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P6-10.md) | —; Non eseguite | G-MIG-6 NOT_VERIFIED |
| T-MIG-P7-01 | T-MIG-P6-09 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P7-01.md) | —; Non eseguite | G-MIG-7 NOT_VERIFIED |
| T-MIG-P7-02 | T-MIG-P7-01 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P7-02.md) | —; Non eseguite | G-MIG-7 NOT_VERIFIED |
| T-MIG-P7-03 | T-MIG-P7-01, T-MIG-P7-02 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P7-03.md) | —; Non eseguite | G-MIG-7 NOT_VERIFIED |
| T-MIG-P7-04 | T-MIG-P7-01 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P7-04.md) | —; Non eseguite | G-MIG-7 NOT_VERIFIED |
| T-MIG-P7-05 | T-MIG-P7-01 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P7-05.md) | —; Non eseguite | G-MIG-7 NOT_VERIFIED |
| T-MIG-P7-06 | T-MIG-P7-02 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P7-06.md) | —; Non eseguite | G-MIG-7 NOT_VERIFIED |
| T-MIG-P7-07 | T-MIG-P7-03, T-MIG-P7-04, T-MIG-P7-05 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P7-07.md) | —; Non eseguite | G-MIG-7 NOT_VERIFIED |
| T-MIG-P7-08 | T-MIG-P7-01, T-MIG-P7-02, T-MIG-P7-03, T-MIG-P7-04, T-MIG-P7-05, T-MIG-P7-06, T-MIG-P7-07, T-MIG-P7-09 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P7-08.md) | —; Non eseguite | G-MIG-7 NOT_VERIFIED |
| T-MIG-P7-09 | T-MIG-P7-03, T-MIG-P7-04, T-MIG-P7-05, T-MIG-P6-10 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P7-09.md) | —; Non eseguite | G-MIG-7 NOT_VERIFIED |
| T-MIG-P8-01 | T-MIG-P7-08 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P8-01.md) | —; Non eseguite | G-MIG-8 NOT_VERIFIED |
| T-MIG-P8-02 | T-MIG-P8-01 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P8-02.md) | —; Non eseguite | G-MIG-8 NOT_VERIFIED |
| T-MIG-P8-03 | T-MIG-P8-01, T-MIG-P8-02 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P8-03.md) | —; Non eseguite | G-MIG-8 NOT_VERIFIED |
| T-MIG-P8-04 | T-MIG-P8-01, T-MIG-P8-02 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P8-04.md) | —; Non eseguite | G-MIG-8 NOT_VERIFIED |
| T-MIG-P8-05 | T-MIG-P8-01, T-MIG-P8-03 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P8-05.md) | —; Non eseguite | G-MIG-8 NOT_VERIFIED |
| T-MIG-P8-06 | T-MIG-P8-03, T-MIG-P8-04, T-MIG-P8-05 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P8-06.md) | —; Non eseguite | G-MIG-8 NOT_VERIFIED |
| T-MIG-P8-07 | T-MIG-P8-04, T-MIG-P8-05, T-MIG-P8-06 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P8-07.md) | —; Non eseguite | G-MIG-8 NOT_VERIFIED |
| T-MIG-P8-08 | T-MIG-P8-01, T-MIG-P8-02, T-MIG-P8-03, T-MIG-P8-04, T-MIG-P8-05, T-MIG-P8-06, T-MIG-P8-07 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P8-08.md) | —; Non eseguite | G-MIG-8 NOT_VERIFIED |
| T-MIG-P9-01 | T-MIG-P8-08 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P9-01.md) | —; Non eseguite | G-MIG-9 NOT_VERIFIED |
| T-MIG-P9-02 | T-MIG-P9-01 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P9-02.md) | —; Non eseguite | G-MIG-9 NOT_VERIFIED |
| T-MIG-P9-03 | T-MIG-P9-01, T-MIG-P9-02, T-MIG-P9-08 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P9-03.md) | —; Non eseguite | G-MIG-9 NOT_VERIFIED |
| T-MIG-P9-04 | T-MIG-P9-03 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P9-04.md) | —; Non eseguite | G-MIG-9 NOT_VERIFIED |
| T-MIG-P9-05 | T-MIG-P9-03, T-MIG-P9-04 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P9-05.md) | —; Non eseguite | G-MIG-9 NOT_VERIFIED |
| T-MIG-P9-06 | T-MIG-P9-04, T-MIG-P9-05 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P9-06.md) | —; Non eseguite | G-MIG-9 NOT_VERIFIED |
| T-MIG-P9-07 | T-MIG-P9-01, T-MIG-P9-02, T-MIG-P9-03, T-MIG-P9-04, T-MIG-P9-05, T-MIG-P9-06, T-MIG-P9-08 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P9-07.md) | —; Non eseguite | G-MIG-9 NOT_VERIFIED |
| T-MIG-P9-08 | T-MIG-P9-01, T-MIG-P7-09 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P9-08.md) | —; Non eseguite | G-MIG-9 NOT_VERIFIED |

U-FIX-01..05 DONE: G-HARD PASS con CI42990775714 sucb70bee,7 job richiesti positivi.
U-NORM-03 DONE per backend normativo,21 oracle3volte/OS;04 WAITING_INTEGRATION per frontend/contesto amministrativo, G-SIOC1 complessivo aperto.
T-MIG-P0-01 DONE; G-MIG-0 non ancora PASS prima delle altre sette taskP0.
U-OPS-07 DONE documentale; U-OPS-03 preflight sospeso sull'integrazione root/worker Tauri reale, nessun DONE implicito.
**next READY T-MIG-P0-02** per proseguire il percorso UI autorizzato. AUTO_CONTINUE.
Le task non avviate non hanno CI/evidenze implicite. SHA dei dossier risolvibile
con `git log --format=%H -- <percorso-dossier>`, evitando hash autoreferenziali.
U-GOV è sola governance: nessun dato, schema, golden o comportamento modificato.
