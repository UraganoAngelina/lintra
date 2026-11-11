# Registro MIG corrente — 11 novembre2026

La fonte30 ottobre resta storica. Questo registro collega le decisioni già
approvate dall'autore10 novembre, gli ADR recepiti e le prove; non inventa
nuove approvazioni. Stato runtime/implementazione nello stato unificato.

| Decisione | Stato corrente | Evidenza / gate residuo |
| --- | --- | --- |
| DEC-MIG-01 | USER_APPROVED / RECEIVED | ADR-MIG-001; coesistenza e rollback fino al cutover qualificato |
| DEC-MIG-02 | USER_APPROVED / RECEIVED | ADR-MIG-002; Win11 NSIS offline/perMachine e CachyOS personale, prove P1/P8 future |
| DEC-MIG-03 | USER_APPROVED / RECEIVED | ADR-MIG-003; policy offline/privacy, test runtime/CSP/attacchi ancora necessari |
| DEC-MIG-04 | USER_APPROVED product / TECHNICAL_CONTRACT_P2 | Un job, IPC bounded/typed approvati; firme/limiti/lifecycle concreti al G-MIG-2 |
| DEC-MIG-05 | USER_APPROVED / RECEIVED | ADR-MIG-001 e brief UX, parità comportamentale/redesign per OS |
| DEC-MIG-06 | TECHNICAL_BASELINE_VERIFIED | cb70bee, CI42990775714 success7 job; nessun WIP08 attuale |
| DEC-MIG-07 | USER_APPROVED / RECEIVED | Scope LocalOnly+Full obbligatorio; storico U-CAD e SIOC nel master/gate propri |
| DEC-MIG-08 | USER_APPROVED product / SUPPLY_CHAIN_PENDING | Stack recepito, versioni/lock/licenze da verificare nei gate tecnici |
| DEC-MIG-09 | USER_APPROVED / RECEIVED | ADR-MIG-004; test sintetici, G-REAL limitato alle autorizzazioni dei dati specifici |

| Conflitto storico | Esito per G-MIG-0 | Residuo effettivo |
| --- | --- | --- |
| CNF-MIG-01 N-24 | CLOSED: ADR-MIG-001 + ARCH/DET/N-24 | Parità/cutover e rollback da provare |
| CNF-MIG-02 OS/WebView | POLICY_CLOSED: ADR-MIG-002, eccezioni GUI-only | Native Win11/NSIS/offline/due account e CachyOS inP1/P8 |
| CNF-MIG-03 pixel/UX | POLICY_CLOSED: brief/protocollo P0-07 | Screenshot e UX/a11y/performance della nuova WebView inP5/P7 |
| CNF-MIG-04 stati storici | CLOSED: matrici U-GOV-02 eP0-02 | Nessuna task originale da rieseguire |
| CNF-MIG-05 package/clap/build Git | CLOSED: confini reali documentati | Nessun refactor implicito; mantenere boundary nei futuri adapter |
| CNF-MIG-06 WIP08/baseline | CLOSED: P0-01 e CI sucb70bee | Ogni nuovo delta richiede le sue prove, non eredita il PASS di cb70bee |

Clap è dipendenza del package che contiene anche la CLI; gli import sono nei
moduli CLI. Il core non importa una GUI. build.rs legge Git durante la build
per la provenienza, non a runtime. Lo script workspace e le prove native
qualificano i confini correnti; nessuna ricostruzione del package è richiesta
per rendere vere formule storiche troppo generiche.

I rischi RSK-MIG-01..10 della fonte restano aperti fino alle rispettive prove:
RAM/paging, late events, XSS/privacy, receipt parziali, SHA/CI, scope regionale,
SQLite/recovery, temp/chiave, account/installer e Full effettivo. Le policy
chiuse non sono evidenze di runtime, né un'autorizzazione di pubblicazione.

Riferimenti: [ratifica](RATIFICATION_2026-11-10.md),
[ADR001](../adr/ADR-MIG-001.md), [ADR002](../adr/ADR-MIG-002.md),
[ADR003](../adr/ADR-MIG-003.md), [ADR004](../adr/ADR-MIG-004.md),
[scope](MIGRATION_SCOPE_PARITY.md),
[brief UX](../ui-design-documentation/MIGRATION_TAURI_BRIEF.md),
[budget](MIGRATION_PERFORMANCE_BUDGET.md),
[gate0](../verification/unified-post-m9/G-MIG-0.md).
