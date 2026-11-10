# Ponte operativo MIG v1.1 → post-M9 unificato

U-UI-01, 10 novembre 2026. Autorità: ratifica D01–D15 e master unificato v1.0,
poi le ratifiche specifiche/ARCH/DET per le parti non modificate. Le FONTI
del 30 ottobre sono fotografie storiche e non vengono riscritte.

## Baseline e registri

| Elemento | Fatto corrente |
|---|---|
| Piano originario | 140/140 DONE, QG-8/M9 raggiunti; nessuna task originale riaperta |
| HEAD alla preparazione del ponte | cb5c11f, branch work/p1-foundations, working tree pulita |
| Ultimo delta eseguibile | 76fa08a: reference-tree permissions; locale89test + fmt/Clippy, native Windows ancora da qualificare |
| Delta budget precedente | 4864c34; locale workspace1073pass/7ignored; CI42829144043 avviata su539b9bc con sorgenti identici, esito da verificare |
| Audit baseline | af3bccc incluso; CI42789338814 storica; nuova baselineCI42823787199 su4c169cd verificata, sette job success |
| Stato MIG | 89 ID originali, nessuno DONE per ratifica; fonte di stato unificata nella root |
| Task entry | U-UI-01 → T-MIG-P0-01; G-MIG-0 ancora NOT_VERIFIED |

Non attribuire la CI539b9bc alla modifica tree76fa08a. P0-01 deve qualificare
la baseline effettiva; DONE_UNVERIFIED/WAITING_GATE non soddisfano un gate.
Lo stato PERSIST08 WIP/117task del preflight non descrive il checkout attuale.
CAD83/C1/L1 library già qualificati non significano Full o CAD nella Slint.

Indice completo e archi preservati: [DEPENDENCY_GRAPH](DEPENDENCY_GRAPH.json),
[master §10](MASTER_PLAN_UNIFICATO.md), 89 prompt in
PROMPT_ATOMICI/MIGRAZIONE_89. Nessuna nuova task MIG duplicata. La procedura
di recepimento è già avvenuta in ADR-POST-M9-001; gli ADR MIG specifici si
completano nelle task P0. Nessuna ratifica globale D01–D15 da chiedere.

## Overlay vincolante sui prompt originali

| Task interessate | Delta operativo / acceptance preservata |
|---|---|
| P0-01/02/08 | Baseline post-audit e CI effettiva, 140/140, ratifiche già acquisite; niente WIP08 inventato |
| P0-03/05 e policy di tutte le fasi | React/TS strict/Vite/Tailwind/shadcn/TanStack/Tauri2 approvati; Windows11 x86_64 NSIS offline/perMachine, CachyOS solo personale; core/CLI OS policy invariata |
| P0-06, P2-08, P7-03 | Uso interno reale, file locali per account, ACL/temp privati, niente nuova crypto/password/retention o web storage PHI; autorizzazioni aziendali specifiche non presunte |
| P2-07, P4-10, P6-08 | Locale scrivibile libero; shared/UNC/remoto con avviso e conferma; overwrite/receipt e byte canonici invariati |
| P4-11/12, P5-09/10, P6-10, P7-09, P9-08 | Full obbligatorio, input esplicito per trace, due passate reali, Unknown/Partial/cancel/errore tecnico distinti; medesimo servizio Rust |
| P5-05 | Soft budget con Annulla/Procedi comunque; oggetto Rust di preflight legato alla richiesta/file/schema, hard non aggirabili; integrare U-FIX-03 senza norme JS |
| P7-04 | 300k/1M solo stress UI; misure realistiche separate e ipotesi dichiarate, nessun parallelismo o soglie DEC-14 inventate |
| P8-04/07, P9-01/03/06/07 | Qualifica installer interno effettivo: Windows11 clean-room/WebView2 offline, due account/non-admin, update/rollback/uninstall conservativi, SHA artefatto e CI nativa |
| P9 cutover/MM9 | Parità LocalOnly+Full e rollback Slint effettivi; nessuna consegna ai colleghi dedotta dal solo MM9 |
| U-UI-02 → U-CAD-01..09 | CAD storico aggiunto dal piano unificato, non finto incluso nelle 89 task; namespace/C1/L1/preview/review/CAS/T1-T2-T3 da nuova UI |
| U-NORM-03/04, bridge dopo G-MIG-4 | D11 nuovo profilo/capability generica, niente audit-only come soluzione finale, niente regole simulate in renderer |
| G-REL | Richiede G-MIG-9 + G-CADUI3 + G-SIOC1 + G-OPS2 + G-DUR e signoff/installer; nessun tag o pubblicazione arbitraria |

L'acceptance nativa Windows/CachyOS, i negativi IPC/XSS, privacy DOM/cache,
bounded paging, byte golden, cancellazione e garanzie CAS non vengono ridotti.
Nessuna architettura server/cloud, telemetria, CDN o auto-updater. Una validazione
attiva per istanza; Rust possiede job, file, validazione, query/indici e storage.

## Interfacce con gli altri workstream

U-FIX-03/04 hanno implementazioni e prove locali proprie, non implicano
qualifica UI. U-OPS-01/02 documentano scope e superfici; provisioning/ACL,
shared warning, chiave, Backup-B/restore restano da verificare nelle task proprie.
G-DUR richiede fault injection/kill/reopen/report, senza riscrivere SQLite.

U-NORM-05 ha isolato la contraddizione fra una lettura indiscriminata di D11 e
la ricopertura1–3/aggiornamento UDO prescritti dal workbook: decisione mirata
pendente, nessun ritorno audit-only. Ciò blocca l'enforcement dipendente e il
signoff SIOC, **non** preparazione/migrazione UI sintetica o LocalOnly/Full.
G-REAL non è un gate enterprise per costruire un package tecnico funzionante.

I blocchi realmente nativi (VM Windows11, offline WebView2, due account) vanno
registrati quando verificati; non si considera Windows CI una clean-room Win11.
Slint resta operativo fino a parità e rollback qualificati. Nessuna nuova UI o
baseline Tauri è affermata eseguibile da questo ponte documentale.
