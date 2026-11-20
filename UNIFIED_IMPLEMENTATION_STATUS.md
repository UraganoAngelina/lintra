# Stato unificato post-M9

Aggiornato: 2026-11-20, Europe/Rome. Un agente; task tracciate singolarmente,
sviluppo sequenziale per blocchi coerenti secondo AGENTS.md.
Autorità: [ratifica D01–D15](docs/unified-post-m9/RATIFICATION_2026-11-10.md),
[master v1.0](docs/unified-post-m9/MASTER_PLAN_UNIFICATO.md) e
[grafo completo](docs/unified-post-m9/DEPENDENCY_GRAPH.json).
Consuntivo storico separato: **140/140 DONE**, QG-8/M9 invariati.

Conteggio unificato: **66/128 DONE;62 rimanenti** (18U-* e50MIG).
Le rimanenti comprendono le task avviate, sospese e bloccate; nessun gate
pendente viene contato come task conclusa.

Gate correnti: G-U0 PASS documentale; G-HARD PASS nativo su cb70bee/
CI42990775714; G-SIOC0 PASS documentale; G-MIG-0 PASS governance.
G-DUR PASS software su3aa0d14/CI43078048297; hardware NOT_TESTED_HARDWARE.
G-MIG-1 PASS/GO tecnico piattaforme: dossier P1-02/03/04/05/07 e decisioneP1-06.
G-MIG-2 APPROVED design, ratifica autore15/11 sul pacchetto45fce38.
G-MIG-3 PASS: CI desktop43758529844 e step QG-1/2/3 nativi della CI43758531087 su ce344d4.
G-MIG-4 PASS: [checkpoint P4](docs/verification/unified-post-m9/G-MIG-4.md), CI44706977518 e desktop44706949307 su 1ab34d8.
G-OPS0/1/2, G-SIOC1, G-MIG-5..9, G-CADUI0..3, G-PERF e G-REL:
NOT_VERIFIED. Prove Win11/offline/due desktop e scale osservate distinte dalla
CI Windows Server; VM ripristinata/spenta. Non è un gate di distribuzione.
G-REAL riguarda autorizzazioni effettive per specifici dati, non presunte.
CI antecedenti non qualificano nuovi delta runtime.

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
| U-OPS-04 | U-OPS-02 | DONE | [Policy Rust e integrazione export](docs/verification/unified-post-m9/U-OPS-04.md) | 3aa0d14;CI43078048297 SUCCESS,7job;SMB/drive mappato Windows e suite Linux PASS | Acceptance backend/CLI/Slint PASS;G-OPS1 aggregato attende03;React in MIG |
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
| U-UI-01 | U-GOV-04 | DONE | [Ponte 89 MIG/post-audit](docs/verification/unified-post-m9/U-UI-01.md) | Overlay master §6, sorgenti76fa08a distinti dalla HEAD/CI; controlli grafo/documenti | Acceptance ponte PASS; G-MIG-0 PASS governance |
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
| U-PERF-01 | U-GOV-02 | DONE | [Volumi aggregati e oracle](docs/verification/unified-post-m9/U-PERF-01.md) | Fonte anonima CAD187926 record;conteggi/larghezze/byte verificati;SIOC non calibrato;solo documenti | Acceptance analisi PASS;G-PERF aggregato NOT_VERIFIED,misure end-to-end in02 |
| U-PERF-02 | U-PERF-01, T-MIG-P7-08 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/INTEGRAZIONE_39/U-PERF-02.md) | —; Non eseguite | G-PERF NOT_VERIFIED |
| U-REL-01 | U-UI-02, U-CAD-09, U-NORM-04, U-OPS-08 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/INTEGRAZIONE_39/U-REL-01.md) | —; Non eseguite | G-REL NOT_VERIFIED |
| U-REL-02 | U-REL-01, U-OPS-06, U-REL-03 | NOT_STARTED | [Prompt](docs/unified-post-m9/PROMPT_ATOMICI/INTEGRAZIONE_39/U-REL-02.md) | —; Non eseguite | G-REL NOT_VERIFIED |
| U-REL-03 | U-OPS-01, U-FIX-01 | DONE | [Sync report e crash/reopen manifest](docs/verification/unified-post-m9/U-REL-03.md) | 3aa0d14;CI43078048297 SUCCESS,7job;kill/fault/reopen3volte per OS,permessi Linux e path Windows PASS | G-DUR PASS software;hardware NOT_TESTED_HARDWARE |
| T-MIG-P0-01 | U-UI-01 | DONE | [Baseline post-audit qualificata](docs/verification/unified-post-m9/T-MIG-P0-01.md) | cb70bee;CI42990775714 success7 job;HEAD documentale42620cf con520 input runtime identici | G-MIG-0 PASS governance; prove UI/runtime successive separate |
| T-MIG-P0-02 | T-MIG-P0-01 | DONE | [Matrice corrente/storia](docs/verification/unified-post-m9/T-MIG-P0-02.md) | Baselinecb70bee qualificata;22 discrepanze riconciliate senza riaprire140 task | G-MIG-0 PASS governance; prove UI/runtime successive separate |
| T-MIG-P0-03 | T-MIG-P0-02 | DONE | [ADR N-24 recepita](docs/verification/unified-post-m9/T-MIG-P0-03.md) | ADR-MIG-001;autore10 novembre,recepimento11;ARCH/DET/N-24 allineati | G-MIG-0 PASS governance; prove UI/runtime successive separate |
| T-MIG-P0-04 | T-MIG-P0-03 | DONE | [Scope e parità obbligatoria](docs/verification/unified-post-m9/T-MIG-P0-04.md) | D01/DEC-MIG-07 recepite;16 capability e stato reale; codice invariato | G-MIG-0 PASS governance; prove UI/runtime successive separate |
| T-MIG-P0-05 | T-MIG-P0-03 | DONE | [ADR OS e offline](docs/verification/unified-post-m9/T-MIG-P0-05.md) | ADR-MIG-002,D14/DEC-MIG-02 recepite;policy core/CLI invariata | G-MIG-0 PASS governance; prove UI/runtime successive separate |
| T-MIG-P0-06 | T-MIG-P0-03 | DONE | [Policy e threat model](docs/verification/unified-post-m9/T-MIG-P0-06.md) | ADR-MIG-003/004,policy autore recepita;test attacco/runtime futuri espliciti | G-MIG-0 PASS governance; prove UI/runtime successive separate |
| T-MIG-P0-07 | T-MIG-P0-04, T-MIG-P0-05 | DONE | [Brief UX e budget misurabili](docs/verification/unified-post-m9/T-MIG-P0-07.md) | Copper Ledger esteso,stati/viewport/a11y e protocollo;nessun tempo DEC-14 inventato | G-MIG-0 PASS governance; prove UI/runtime successive separate |
| T-MIG-P0-08 | T-MIG-P0-01, T-MIG-P0-02, T-MIG-P0-03, T-MIG-P0-04, T-MIG-P0-05, T-MIG-P0-06, T-MIG-P0-07 | DONE | [Gate G-MIG-0](docs/verification/unified-post-m9/T-MIG-P0-08.md) | ADR/evidenze/prerequisiti verificati;cb70bee CI42990775714 positiva | G-MIG-0 PASS governance; ingressoP1 autorizzato |
| T-MIG-P1-01 | T-MIG-P0-08 | DONE | [Spike nativo React/Tauri Linux](docs/verification/unified-post-m9/T-MIG-P1-01.md) | 996947e app invariata;build/Clippy/IPC-render Wayland/capture PASS;144 righe ldd senza missing | Acceptance Linux personale PASS;G-MIG-1 aggregato NOT_VERIFIED |
| T-MIG-P1-02 | T-MIG-P0-08 | DONE | [Spike Windows e prove reali](docs/verification/unified-post-m9/T-MIG-P1-02.md) | bdca1cf/CI43014153420;13/11 Win11 offline runtime presente/assente, due desktop standard, ACL/uninstall/restore PASS | Acceptance Windows spike PASS; G-MIG-1 aggregato NOT_VERIFIED |
| T-MIG-P1-03 | T-MIG-P1-01 | DONE | [Profilo e prove CachyOS](docs/verification/unified-post-m9/T-MIG-P1-03.md) | 151 ELF/99 package;relocated Wayland e loader negativo PASS;binario c53fd4ba invariato | Acceptance personale PASS;G-MIG-1 aggregato bloccato su Win11 |
| T-MIG-P1-04 | T-MIG-P1-01, T-MIG-P1-02 | DONE | [CSP e guard nativi](docs/verification/unified-post-m9/T-MIG-P1-04.md) | 608ccdc/CI43595367523 SUCCESS;9test/Clippy/TS/MSVC/NSIS; native Linux/Win11 18controlli/0TCP0HTTP PASS; VM ripristinata/spenta | G-MIG-1 NOT_VERIFIED |
| T-MIG-P1-05 | T-MIG-P1-01, T-MIG-P1-02 | DONE | [Profilo WebView/font/DPI](docs/verification/unified-post-m9/T-MIG-P1-05.md) | 413b7e4/CI43494464261 SUCCESS; Wayland e Win11 standard/font/DPI100%/capture PASS; VM ripristinata/spenta | Acceptance profilo PASS; G-MIG-1 NOT_VERIFIED |
| T-MIG-P1-06 | T-MIG-P1-02, T-MIG-P1-03, T-MIG-P1-04, T-MIG-P1-05, T-MIG-P1-07 | DONE | [GO piattaforme](docs/verification/unified-post-m9/T-MIG-P1-06.md) | Matrice nativa e CI dei prerequisiti consolidate; nessun nuovo test equivalente | G-MIG-1 PASS; ingressoP2, non release |
| T-MIG-P1-07 | T-MIG-P1-02, T-MIG-P1-04 | DONE | [Installer offline](docs/verification/unified-post-m9/T-MIG-P1-07.md) | CI43595367523/608ccdc; setup/uninstall0, desktop standard e0TCP0HTTP; sentinelle conservate; VM ripristinata/spenta | Acceptance installer PASS; G-MIG-1 inP1-06 |
| T-MIG-P2-01 | T-MIG-P1-06 | DONE | [Ownership processo](docs/verification/unified-post-m9/T-MIG-P2-01.md) | Diagrammi derivati da ADR ratificati; entrypoint reali e confine dipendenze verificati; nessun bridge | G-MIG-2 NOT_VERIFIED; wire non congelato |
| T-MIG-P2-02 | T-MIG-P2-01 | DONE | [Proposta envelope](docs/verification/unified-post-m9/T-MIG-P2-02.md) | Ratifica autore14/11 con7precisazioni; schemaVersion1 envelope/DTO;84vettori PASS/exit0 | AcceptanceP2-02 PASS; G-MIG-2 NOT_VERIFIED |
| T-MIG-P2-03 | T-MIG-P2-02 | DONE | [Run/lifecycle](docs/verification/unified-post-m9/T-MIG-P2-03.md) | Transizioni/race complete;53casi PASS/exit0; fonti Rust e identity/owner/terminali verificati nel modello | Gate design; G-MIG-2 NOT_VERIFIED |
| T-MIG-P2-04 | T-MIG-P2-03 | DONE | [Progresso/terminale](docs/verification/unified-post-m9/T-MIG-P2-04.md) |49scenari PASS/exit0;10k update/consumer lento, slot bounded, terminale indipendente; regressione53 lifecycle PASS | Gate design; G-MIG-2 NOT_VERIFIED |
| T-MIG-P2-05 | T-MIG-P2-02, T-MIG-P2-03 | DONE | [Paging/query](docs/verification/unified-post-m9/T-MIG-P2-05.md) |70casi PASS/exit0; max128righe/2pagine,1MiB, filtri/ordine/handle; JSONL invariato | Gate design; G-MIG-2 NOT_VERIFIED |
| T-MIG-P2-06 | T-MIG-P2-05 | DONE | [Dettaglio/domìni](docs/verification/unified-post-m9/T-MIG-P2-06.md) |58casi PASS/exit0; Unicode/context64, schema/reference catturati, digest/stati espliciti e nessuna enum regex | Gate design; G-MIG-2 NOT_VERIFIED |
| T-MIG-P2-07 | T-MIG-P2-03 | DONE | [Export nativo](docs/verification/unified-post-m9/T-MIG-P2-07.md) |60casi PASS/exit0; path/conferme solo native, replay/stale, receipt parziali; nessuna transazione multi-file | Gate design; G-MIG-2 NOT_VERIFIED |
| T-MIG-P2-08 | T-MIG-P2-02, T-MIG-P2-07 | DONE | [Sicurezza/ACL](docs/verification/unified-post-m9/T-MIG-P2-08.md) | Threat model ADR-MIG-003 già ratificato; ACL generate auditate/exit0; input nativi608ccdc invariati, evidenzeP1-04 riusate | Gate design; G-MIG-2 NOT_VERIFIED |
| T-MIG-P2-09 | T-MIG-P2-03, T-MIG-P2-04, T-MIG-P2-05, T-MIG-P2-06, T-MIG-P2-07, T-MIG-P2-08 | DONE | [Parity/fixture](docs/verification/unified-post-m9/T-MIG-P2-09.md) |14scenari/64file con SHA e attesi indipendenti; comparatore/negativi PASS, native parity NOT_RUN | Gate design; G-MIG-2 NOT_VERIFIED |
| T-MIG-P2-10 | T-MIG-P2-01, T-MIG-P2-02, T-MIG-P2-03, T-MIG-P2-04, T-MIG-P2-05, T-MIG-P2-06, T-MIG-P2-07, T-MIG-P2-08, T-MIG-P2-09, T-MIG-P2-11 | DONE | [Ratifica gate](docs/verification/unified-post-m9/T-MIG-P2-10.md) | Approvazione esplicita autore15/11 del pacchetto45fce38; prerequisiti verificati | G-MIG-2 APPROVED design; ingresso P3 autorizzato |
| T-MIG-P2-11 | T-MIG-P2-03, T-MIG-P2-05, T-MIG-P2-08 | DONE | [Full ratificato](docs/verification/unified-post-m9/T-MIG-P2-11.md) | DTO/ownership e18casi verificati; ratifica autore15/11 in G-MIG-2 | Acceptance design PASS; integrazione nativa ai task previsti |
| T-MIG-P3-01 | T-MIG-P2-10 | DONE | [Scaffold Rust/React](docs/verification/unified-post-m9/T-MIG-P3-01.md) | c8ad22e: Linux release/2test/TS PASS; Windows CI43731922567 test/Clippy/release PASS | Acceptance scaffold PASS; G-MIG-3 NOT_VERIFIED |
| T-MIG-P3-02 | T-MIG-P3-01 | DONE | [Pin/supply chain](docs/verification/unified-post-m9/T-MIG-P3-02.md) | Versioni/SSRI/license e toolchain verificati; npm ci offline/engine-strict exit0, nessun nuovo registry package | Acceptance pin PASS; G-MIG-3 NOT_VERIFIED |
| T-MIG-P3-03 | T-MIG-P3-01 | DONE | [Font e token locali](docs/verification/unified-post-m9/T-MIG-P3-03.md) | Font/OFL byte-identici, fonti/hash e due test asset PASS; baseline WebView distinta | Acceptance copia asset PASS; G-MIG-3 NOT_VERIFIED |
| T-MIG-P3-04 | T-MIG-P3-01, T-MIG-P3-02 | DONE | [Dispatcher Rust](docs/verification/unified-post-m9/T-MIG-P3-04.md) | 10 test Rust e smoke nativo Wayland avvio/chiusura PASS; nessun servizio validazione | Acceptance dispatcher PASS; G-MIG-3 NOT_VERIFIED |
| T-MIG-P3-05 | T-MIG-P3-02, T-MIG-P3-04 | DONE | [Client TS](docs/verification/unified-post-m9/T-MIG-P3-05.md) | 14 test TS/11 Rust e handshake React–Rust nativo Wayland PASS; errori statici e schema strict | Acceptance client PASS; G-MIG-3 NOT_VERIFIED |
| T-MIG-P3-06 | T-MIG-P3-01, T-MIG-P3-02, T-MIG-P3-03, T-MIG-P3-04, T-MIG-P3-05 | DONE | [Integrazione P3](docs/verification/unified-post-m9/T-MIG-P3-06.md) | ce344d4: CI desktop43758529844 SUCCESS + smoke; QG-1/2/3 Linux/Windows CI43758531087 SUCCESS per step; ulteriori prove della run in corso | G-MIG-3 PASS; non qualifica release |
| T-MIG-P4-01 | T-MIG-P3-06 | DONE | [Catalogo e selezione](docs/verification/unified-post-m9/T-MIG-P4-01.md) | 16 Rust/20 Node/8 regressioni configurazione Slint PASS locali; versioni registry già qualificate; CI nativa al blocco | G-MIG-4 NOT_VERIFIED |
| T-MIG-P4-02 | T-MIG-P4-01 | DONE | [Broker file e preflight](docs/verification/unified-post-m9/T-MIG-P4-02.md) | 19 Rust/22 Node/8 regressioni Slint PASS locali; vero dialogo GTK sintetico PASS; Windows al blocco | G-MIG-4 NOT_VERIFIED |
| T-MIG-P4-03 | T-MIG-P4-01, T-MIG-P4-02 | DONE | [Preparazione immutabile](docs/verification/unified-post-m9/T-MIG-P4-03.md) | 23 Rust/24 Node/8 Slint PASS locali; GTK selezione/teardown PASS; MSVC al checkpoint blocco | G-MIG-4 NOT_VERIFIED |
| T-MIG-P4-04 | T-MIG-P4-03 | DONE | [Dossier](docs/verification/unified-post-m9/T-MIG-P4-04.md) | 5e3acdb;32 Rust/28 Node/25 regressioni locali PASS;CI44065663299 SUCCESS Linux/Windows,build e smoke nativi PASS | Acceptance adapter PASS; G-MIG-4 aggregato resta aperto |
| T-MIG-P4-05 | T-MIG-P4-04 | DONE | [Dossier](docs/verification/unified-post-m9/T-MIG-P4-05.md) | 5e3acdb;32 Rust/28 Node/25 regressioni locali PASS;CI44065663299 SUCCESS Linux/Windows,build e smoke nativi PASS | Acceptance adapter PASS; G-MIG-4 aggregato resta aperto |
| T-MIG-P4-06 | T-MIG-P4-04, T-MIG-P4-05 | DONE | [Dossier](docs/verification/unified-post-m9/T-MIG-P4-06.md) | 5e3acdb;32 Rust/28 Node/25 regressioni locali PASS;CI44065663299 SUCCESS Linux/Windows,build e smoke nativi PASS | Acceptance adapter PASS; G-MIG-4 aggregato resta aperto |
| T-MIG-P4-07 | T-MIG-P4-06 | DONE | [Dossier](docs/verification/unified-post-m9/T-MIG-P4-07.md) | a68d17d+5e9cacd (=565623b/3b15f0a);gate merge locale su albero codice identico 1.146 Rust+28 Node PASS/0 fail;review Claude approve round 2;CI nativa al checkpoint blocco | Acceptance indice/cache PASS; G-MIG-4 aggregato resta aperto |
| T-MIG-P4-08 | T-MIG-P4-07 | DONE | [Dossier](docs/verification/unified-post-m9/T-MIG-P4-08.md) | ab679c5;43 nativi/1.112 workspace/31 Node PASS locali, smoke Wayland PASS;review Codex APPROVED con repliche native/Node;CI nativa al checkpoint blocco | Acceptance query/paging PASS; G-MIG-4 aggregato resta aperto |
| T-MIG-P4-09 | T-MIG-P4-08 | DONE | [Dossier](docs/verification/unified-post-m9/T-MIG-P4-09.md) | 5027ee4+7d5cdbc;55 nativi/38 Node/1.113 workspace PASS locali,smoke PASS;review Codex round 2 APPROVED con repliche native/Node;CI nativa al checkpoint blocco | Acceptance dettaglio/guide/reference PASS; G-MIG-4 aggregato resta aperto |
| T-MIG-P4-10 | T-MIG-P4-06, T-MIG-P4-08, T-MIG-P4-09, T-MIG-P4-12 | DONE | [Dossier](docs/verification/unified-post-m9/T-MIG-P4-10.md) | 40ebd23+8ec47f8;62 nativi/41 Node/1.116 workspace PASS locali,smoke PASS;review Codex round 2 APPROVED con repliche native/Node;check Windows BLOCKED (mingw);CI nativa al checkpoint blocco | Acceptance export/About/bridge PASS; G-MIG-4 aggregato resta aperto |
| T-MIG-P4-11 | T-MIG-P4-06, T-MIG-P2-11 | DONE | [Dossier](docs/verification/unified-post-m9/T-MIG-P4-11.md) | f97cb7c+1f33be8;gate merge locale 47 nativi/32 Node/1.112 workspace PASS,smoke PASS;review Codex CHANGES su P411-01,rework senza 2ª review per deroga autore;CI nativa al checkpoint blocco | Acceptance bridge Full PASS; G-MIG-4 aggregato resta aperto |
| T-MIG-P4-12 | T-MIG-P4-11, T-MIG-P4-05 | DONE | [Dossier](docs/verification/unified-post-m9/T-MIG-P4-12.md) | cab4a04;58 nativi/38 Node PASS locali,smoke PASS;review Codex APPROVED con repliche native/Node;CI nativa al checkpoint blocco | Acceptance progresso/cancel/cleanup Full PASS; G-MIG-4 aggregato resta aperto |
| T-MIG-P5-01 | T-MIG-P4-10 | DONE | [Dossier](docs/verification/unified-post-m9/T-MIG-P5-01.md) | 50 Node, 68 Rust Linux/67 MSVC, Clippy PASS; capture/tastiera native 1× Linux 10/10, Win11 VM 11/11, 2× 7/7+7/7; e3c316c+e726310;review Codex round 3 APPROVED;CI delta NOT_RUN,al checkpoint blocco | Acceptance shell/tema PASS; G-MIG-5 NOT_VERIFIED |
| T-MIG-P5-02 | T-MIG-P5-01 | IN_REVIEW | [Dossier](docs/verification/unified-post-m9/T-MIG-P5-02.md) | 60 Node/lint/build PASS; parità Slint analysis_configuration 8/8 e schema_binding 1/1; selezione nativa Linux 12/12; CI delta NOT_RUN | G-MIG-5 NOT_VERIFIED |
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
T-MIG-P0-01..08 DONE; G-MIG-0 PASS per governance e ingresso agli spike, non per runtime/installer.
U-OPS-07 DONE documentale; U-OPS-03 preflight sospeso sull'integrazione root/worker Tauri reale, nessun DONE implicito.
**T-MIG-P1-01 DONE** per spike nativo Linux personale: build debug/Clippy e IPC-render/capture Wayland PASS. P1-02 build CI43014153420 e collaudo Win11 PASS; P1-03 DONE personale. U-PERF-01 DONE per volumi aggregati/oracle. U-OPS-04 DONE dopo CI43078048297 SUCCESS su3aa0d14 (SMB/drive mappato e Linux PASS); U-REL-03 DONE, G-DUR PASS software sulla stessa CI positiva;hardware NOT_TESTED_HARDWARE. P1-04 BLOCKED sul contenimento TCP Windows; P1-05 DONE per profilo/font/DPI100% osservati. P1-06/07 e P2/P3 attendono i prerequisiti. G-MIG-1 aggregato e gli altri gate futuri restano NOT_VERIFIED. Nessuna task indipendente READY, NO_READY_TASK.
Le task non avviate non hanno CI/evidenze implicite. SHA dei dossier risolvibile
con `git log --format=%H -- <percorso-dossier>`, evitando hash autoreferenziali.
U-GOV è sola governance: nessun dato, schema, golden o comportamento modificato.


Handoff 12 novembre2026: U-OPS-04 e U-REL-03 chiuse con evidenze CI effettive;
**31/128 DONE (24,2%)**,97rimanenti. Per questa chiusura solo documenti/log/receipt:
nessun nuovo runtime o CI duplicata. Snapshot dei prerequisiti e dei residui in
`docs/verification/unified-post-m9/U-REL-03-qualification-43078048297.json`.
Prossimo passo del piano invariato: rendere disponibile il collaudo Windows11
clean-room P1-02; build MSVC/NSIS già positiva ma insufficiente per quel gate.
U-OPS-03/U-NORM-04 attendono integrazione Tauri reale; non diventano DONE per
permettere U-OPS-06/08 o G-SIOC1. **NO_READY_TASK**, nessuna delega/background.

Prima ripresa 13 novembre2026 (precedente al mandato amministrativo): recepita in AGENTS.md la direttiva sull'uso selettivo
della VM, senza nuovi gate o task. Per il requisito nativo già previsto da
P1-02 verificati SSH `lintra-win`/`lintra-agent`, preflight in sola lettura e
digest dei due artefatti CI esistenti. Nessun avvio VM, sync, build, installazione
o smoke. Blocco ristretto alla clean-room ripristinabile e ai passaggi elevati/
due account da predisporre o verificare oltre il solo accesso SSH standard; chiesta
disponibilità all'autore. Dettagli e risultati nel dossier P1-02. **31/128 DONE**
e gate invariati; nessuna task indipendente READY.


Handoff successivo 13 novembre2026: mandato amministrativo della sola VM
recepito in AGENTS.md, accessi standard/admin verificati, snapshot attuale
ripristinato prima e dopo le prove. **T-MIG-P1-02 DONE** sull’installer esatto
CI43014153420/bdca1cf: NSIS offline perMachine nei casi runtime presente e
realmente assente dai percorsi/registrazioni, due account standard con desktop/
ACK/capture reali, ACL e conservazione sintetica SQLite/config, uninstall e
ripristino finale PASS. Distinti: assenza preparata controllata vs OS nuovo,
sentinelle vs persistenza prodotto, VM vs CI e gate aggregato futuro.
**32/128 DONE (25,0%)**,96rimanenti. Evidenze nel dossier P1-02; nessun nuovo
gate VM ricorrente o requisito di completamento. P1-04 e P1-05 READY; scegliere
**T-MIG-P1-04** in sequenza, AUTO_CONTINUE. P1-06/07 e P2/P3 non ancora READY.

Avvio sequenziale P1-04: guard origine/navigazioni/popup e prova CSP/ACL nativa
implementati nello spike; Linux strict/unit/Clippy/smoke/security PASS. Windows
MSVC e CI del delta in corso; task IN_PROGRESS, nessun DONE o G-MIG-1 anticipato.


Handoff P1-04 successivo: CI43437870531 SUCCESS su337697d, distinta dal nuovo
filtro SDK. Riprodotto GET di navigazione Windows nonostante il guard; filtro
WebResourceRequested lo elimina. Restano3TCP senza dati: prova zero-network
FAIL esplicita, P1-04 BLOCKED e G-MIG-1 NOT_VERIFIED. Linux0TCP/0HTTP PASS,
3 test/Clippy MSVC PASS; nuova CI richiesta per il delta. P1-02 conserva DONE;
32/128 DONE invariato. Proseguire sequenzialmente con P1-05 indipendente READY.


Handoff P1-05, 13 novembre2026: **T-MIG-P1-05 DONE**. Runtime413b7e4,
CI43494464261 SUCCESS (4test/Clippy/MSVC/NSIS); Wayland/WebKitGTK2.54.2
e Win11/WebView2157.0.4341.46 nel desktop standard/sessione1: font locali,
corpo14/titolo28CSSpx, scala100%, viewport e capture PASS, exit0. Win32DPI96
coerente; nessuna pixel parity universale o qualifica di scale non osservate.
Corretto solo il launcher PowerShell5 per il titolo Unicode; app byte-identica
alla CI, primo FAIL conservato. Tentativo build SSH255 distinto dalla ripresa
riuscita e dal binario CI esatto. VM inizialmente spenta, avviata per il requisito
nativo di questa task, poi ripristinata al checkpoint esistente: modifiche
console/task rimosse, accessi standard/admin conservati e stato finale shut off
verificato dopo virsh shutdown. Nessun gate VM ricorrente o nuova milestone.
**33/128 DONE (25,8%)**,95rimanenti. P1-04 resta BLOCKED su3TCP Windows,
P1-07/06 e P2/P3 attendono quel prerequisito; U-OPS-03/U-NORM-04 attendono
integrazione reale. Nessuna NOT_STARTED con dipendenze tutte DONE:
**NO_READY_TASK**. Le prove P1-05 non risolvono il blocco di rete P1-04.

Ripresa 14 novembre 2026: conservata la frontiera c33cdb8 (33/128 DONE).
Recepita la direttiva sulle verifiche progressive in AGENTS.md; setup skill
esclusivamente utente, ricevuta fuori repo in ~/.codex/setup-evidence/.
Ripresa la diagnosi/correzione circoscritta P1-04 con Connection-Allowlist
sulle risposte asset Windows, senza cambiare CSP/ACL o criterio zero TCP.
Cinque test Rust mirati e Clippy Linux PASS; sorgenti/font/dist identificati
da manifest di39file, cache MSVC riutilizzata. Prove Windows del delta in corso:
il precedente blocco resta aperto fino a evidenza nativa e CI pertinenti.
Nessuna suite workspace/golden/CI/Windows avviata per il solo setup globale.


Handoff P1-04,14 novembre2026: **DONE**, commit608ccdc eCI43595367523 SUCCESS.
Contenimento pre-trasporto e recovery del documento integrati, checkpoint
sintetico privato senza storage web; invarianti CSP/ACL e zeroTCP invariati.
9test Rust, Clippy/TS, native Wayland eWin11 PASS; VM ripristinata/spenta.
**34/128 DONE (26,6%)**. Avviata in sequenza **T-MIG-P1-07** sul nuovo artefatto
CI; nessuna nuova build/CI o regressione workspace richiesta per la sola prova
installer. P1-06/G-MIG-1 attendono gli esiti effettivi di07. AUTO_CONTINUE.


Handoff P1-07,14 novembre2026: **DONE**. Nuovo installer CI43595367523
installato/disinstallato offline nel guest; desktop standard, security0TCP0HTTP,
conservazione di SQLite/config e ripristino/spento PASS. Driver focus Windows
corretto e provato senza cambiare gli input compilati o generare CI equivalente.
**35/128 DONE (27,3%)**. Avviata la decisione tecnica **T-MIG-P1-06** sui dossier
nativi esistenti; nessuna suite o ciclo VM aggiuntivo per consolidare il gate.


Handoff P1-06,14 novembre2026: **DONE — G-MIG-1 GO/PASS** per fattibilità
nativa dello spike. Windows Server/build, Win11/installer/desktop e CachyOS
restano evidenze distinte; runtime/scale/limiti sono espliciti. Nessuna nuova
CI o regressione per il consolidamento; tutte le acceptance del piano intatte.
**36/128 DONE (28,1%)**. Iniziata **T-MIG-P2-01** sui boundary già ratificati e
sui reali entrypoint Rust; nessun nuovo adapter di prodotto prima diG-MIG-2.


Handoff P2-01,14 novembre2026: **DONE** sui boundary già ratificati. Diagrammi
processo/ownership collegati ai servizi reali, nessuna dipendenza WebView nel
core e nessun nuovo bridge. Distinte RunStatus/completeness/evaluation e autorità
storica non deserializzabile. **37/128 DONE (28,9%)**. Prosegue **P2-02**:
wire/limiti concreti da qualificare e ratificare aG-MIG-2, non firme già approvate.


P2-02,14 novembre2026: proposta `MIG-IPC-WIRE-V1-2026-11-14` pronta per
ratifica, non DONE.71vettori stdlib PASS; buffer raw/UTF-8/1MiB/depth32,
versione1, correlazione e taxonomy separata dagli esiti normativi. Il codec è
reference di design, non bridge o qualifica nativa Rust↔TS. Parametri concreti
nuovi da approvare secondo l’acceptanceP2-02; nessuna nuova richiesta D01–D15.
P2-03 e successive attendono il prerequisito; non avviare P3 prima diG-MIG-2.


Ratifica P2-02,14 novembre2026: **DONE**, sette precisazioni dell’autore
conservate integralmente. `schemaVersion = 1` envelope/DTO,1MiB completo
bidirezionale,32livelli, categorie errore esplicite;84vettori mirati PASS.
Niente dataset/raw sanitario/bulk diagnostici in IPC, nessun nuovo consenso
per dettagli ordinari. **38/128 DONE (29,7%)**; avviataP2-03 e successivaP2-04
secondo dipendenze. G-MIG-2 resta aperto; nessun bridge, CI o VM anticipati.


P2-03,14 novembre2026: **DONE** design identità/ownership/stati;53casi PASS.
Nessun successo anticipato da RunCompleted, ack prima di sostituzione, esiti
tecnici e normativi separati;39/128 DONE (30,5%). Prosegue **P2-04** con slot
progresso e terminale distinti, senza nuove firme core o bridge anticipati.


P2-04,14 novembre2026: **DONE** design progresso bounded/terminale indipendente;
49scenari e regressione53lifecycle PASS, nessun rerun del workspace/CI/VM.
**40/128 DONE (31,25%)**. Prosegue **P2-05** su paging/offset/filtri Rust, poi
contratti dettaglio ed export. Tutti i gate applicativi rimangono aperti.


P2-05,14 novembre2026: **DONE** design paging/query.70casi PASS, finestra
massimo2pagine/128righe per pagina e cap intero1MiB. Indici/file/valori restano
Rust; nessun raw sanitario o bulk in IPC.41/128 DONE (32,0%). ProsegueP2-06
su dettaglio minimo e domini catturati; nessuna nuova CI o VM automatica.


Blocco P2-06..09,15 novembre2026: **DONE nel perimetro design**. Dettaglio58
casi ed export60 PASS; ACL effettive generate auditate con input nativi
608ccdc invariati; harness parity definito su14scenari/64file e attesi
indipendenti. Non è nuova parity nativa o funzionalità Rust di prodotto.
**45/128 DONE (35,2%)**. **P2-11 IN_PROGRESS**: Full DTO/ownership e18casi
pubblicati, verifica locale PASS; acceptance approvativa pendente.
[Pacchetto G-MIG-2 pronto](docs/migration/ipc-v1/G-MIG-2_REVIEW.md), ancora
NON APPROVED. Il gate è richiesto esplicitamente prima del bridge/P3; la
ratifica P2-02 non lo supera. P2-10 non DONE e nessuna approvazione inventata.
64digest di evidenze P2 controllati senza mismatch, grafo128task/140fonti
invariato. Nessuna nuova suite Rust/CI/VM per questi contratti. Prossimo
avanzamento dopo ratifica complessiva: chiudere P2-11/P2-10, avviare P3.


15 novembre2026: autore approva esplicitamente **G-MIG-2 e avvioP3** sul
pacchetto45fce38. P2-11/P2-10 DONE,47/128 (36,7%); avviataP3-01 scaffold
affiancato. Nessuna ripetizione CI/VM o reference Python per la sola ratifica.


P3-01 implementata su c8ad22e:4file Rust nel nuovo adapter, manifest/lock
separati e core/CLI/Slint invariati. Build release Linux e2test Rust mirati
PASS, frontend strict PASS. CI Windows43731922567 in corso; task non DONE.
Corretto separatamente il comando YAML inline del workflow diagnostico
SQLite (persistence::), senza cambiare il comando di test o il suo trigger
manuale. La failure43731921269 non ha eseguito job/test SQLite.


Blocco funzionale P3, 15 novembre 2026: **P3-03/04/05 DONE**, **52/128 DONE**.
Dispatcher Rust e client TS bounded v1 integrati con asset/font locali e
provenienza preservata; nessun servizio di validazione anticipato. Fonte
runtime ce344d4; CLI/Slint/core/lock/golden invariati. Regole LF circoscritte
in `.gitattributes` correggono il primo FAIL asset Windows senza normalizzare
i test. CI desktop43758529844 **SUCCESS Linux/Windows**, 16 test Node per OS,
11 Rust Linux/12 Windows e smoke nativi PASS. Regressione locale workspace
core/CLI/Slint e doc test exit0; log/receipt conservati nel dossier P3-06.
**P3-06 IN_PROGRESS / WAITING_GATE** sulla CI ufficiale43758531087 dello stesso
SHA; G-MIG-3 ancora NOT_VERIFIED, G-MIG-4..9/G-REL invariati. Il run legacy
precedente è stato cancellato dalla concurrency esistente, non qualificato.
Nessuna VM, installer o subagente. Nessuna nuova task indipendente READY:
prossima **P4-01 soltanto dopo G-MIG-3 effettivo**, senza nuove ratifiche.


Ripresa 15 novembre 2026: **P3-06 DONE / G-MIG-3 PASS**, 53/128. Verificati
su ce344d4 tutti gli step QG-1/2/3 nativi Linux/Windows e la CI desktop con
smoke. L'attesa aggregata della pipeline ufficiale era più ampia dei criteri
P3: nessuna acceptance modificata e nessuna intera CI dichiarata SUCCESS
mentre ancora in corso. Dossier/receipt conservano ogni singolo step.
**AUTO_CONTINUE P4-01**, poi P4-02/03 nel blocco coerente secondo dipendenze.


Blocco P4-01/02/03, 15 novembre 2026: **56/128 DONE**. Library condivisa
per acquisizione bundle/preflight, broker nativo senza path in JS, parametri
tipizzati e snapshot immutabile LocalOnly/Full. 23 Rust/24 Node/8 Slint PASS
locali, lint/typecheck/build/Clippy PASS, picker GTK reale selezione e teardown
PASS; prima prova del driver di teardown FAIL conservata e corretta.
CI MSVC del blocco in avvio; nessuna qualifica Windows UI anticipata.
La precedente CI ufficiale P3 43758531087 è ora interamente SUCCESS.
G-MIG-4 aperto; **AUTO_CONTINUE P4-04**, poi progresso/cancel/esito.

Ripresa 17 novembre 2026: mandato autore per workflow monoagente e una sola
verifica/attivazione CI dopo implementazione di un blocco funzionale coerente.
Selezionato P4-04/05/06; baseline d78e8d0, branch work/p4-native-lifecycle.
Worker parziale completato con progresso/cancel, report legato al RunRef e
shutdown con join. Blocco IN_PROGRESS fino agli esiti finali locali e nativi;
56/128 DONE invariato, nessun gate o prerequisito sostituito. Nessuna delega,
VM, modifica aiproto/istruzioni/CI. P4-07 e Full attendono la qualifica del blocco.

Checkpoint locale P4-04/05/06 del 17 novembre: implementazione completa,
32 Rust/28 Node/25 regressioni PASS; un ignored GTK preesistente esplicito.
Fmt/Clippy/lint/strict/build e smoke nativo React/IPC PASS. Dossier e receipt
con digest dei sorgenti/log conservati; stato WAITING_GATE sulla CI desktop
Linux/Windows del commit del blocco. L'autore richiede stop dopo commit/push,
con completamento della CI necessaria: nessuna task successiva autorizzata
in questa ripresa. G-MIG-4/G-REL aperti e conteggio56/128 invariato.

Pubblicazione del blocco isolata su work/p4-native-lifecycle-publish dal parent
remoto fa57d35. Commit locale originario f6318d6 e tre precedenti commit locali
di governance/configurazione conservati nel worktree originale, non inviati.
Parità byte degli input runtime verificata rispetto alla build locale; incluse
le due correzioni native preesistenti di d78e8d0. Autorizzazione autore al
push del blocco e completamento CI; repository pubblico di UraganoAngelina
verificato in sola lettura. Hook e configurazioni preservati.

Chiusura blocco P4-04/05/06, 17 novembre 2026: **DONE**, **59/128 (46,1%)**,
69 rimanenti. Runtime5e3acdb, CI44065663299 **SUCCESS** per entrambi i job
Linux/Windows e i singoli step nativi Rust/TS/build/smoke. Parità completa
degli input runtime con il commit locale f6318d6 verificata; il commit di
chiusura aggiunge soltanto evidenze/status, senza duplicare CI. G-MIG-4 e
G-REL restano aperti; UI prodotto e Full appartengono ai blocchi successivi.
**STOP richiesto dall'autore dopo commit/push e CI: nessuna nuova task.**
Prossime tecnicamente disponibili P4-07 e P4-11, da riprendere solo su mandato.

Chiusura T-MIG-P4-07, 18 novembre 2026: **DONE**, **60/128 (46,9%)**, 68
rimanenti. Autore Codex (Lane 3), commit 565623b + correzione 3b15f0a; review
Claude CHANGES al round 1 e approve al round 2 su 3b15f0a; approvazione umana
registrata; merge locale f358db6. Gate di merge locale (9 comandi
configurati) PASS: 1.146 test Rust e 28 Node superati, 0 falliti, 10 ignored
preesistenti; log `gate-merge-1795013997.log`, SHA-256
a54a1324cbd9ef901ae1ff245d1edeca86911f82adc57c30c1c7b90f393b22ad, conservato
nell'archivio locale del runtime dismesso. Integration è stata poi ricostruita
linearmente sul remoto 9171501 senza i commit di governance AIProto: i commit
pubblicati a68d17d e 5e9cacd riapplicano 565623b/3b15f0a e il codice è
identico byte per byte a quello di f358db6 (differiscono solo file di
istruzioni agenti). Nessuna CI remota o prova Windows su questi SHA: la qualifica nativa resta al checkpoint del
blocco P4, come per P4-01..03. G-MIG-4 e G-REL aperti. Workflow AIProto
dismesso; sviluppo successivo autorizzato all'autore Claude, review Codex. Prossime tecnicamente disponibili P4-08 e P4-11.

Chiusura T-MIG-P4-08, 19 novembre 2026: **DONE**, **61/128 (47,7%)**, 67
rimanenti. Autore Claude Code, commit ab679c5 su base 42aa890; review Codex
APPROVED senza rilievi, con repliche indipendenti (43 test nativi, Clippy,
fmt, 31 Node, lint, build) tutte exit 0; integrazione autorizzata dall'autore.
Nessuna CI remota o prova Windows su questo SHA: la qualifica nativa resta al
checkpoint del blocco P4. G-MIG-4 e G-REL aperti. Prossime tecnicamente
disponibili P4-09 e P4-11.

Chiusura T-MIG-P4-11, 19 novembre 2026: **DONE**, **62/128 (48,4%)**, 66
rimanenti. Autore Claude Code. Commit f97cb7c + rework 1f33be8, ribasati
linearmente dopo la chiusura P4-08. Review Codex CHANGES_REQUESTED, con il
solo rilievo P411-01 (E0609 nei test combinati con P4-08), corretto nel
rework. L'autore ha autorizzato l'integrazione senza una seconda review Codex.
Gate di merge locale PASS sull'albero integrato: 47 test nativi, 32 Node e
1.112 workspace (con e senza all-features), Clippy, fmt, build, smoke; log
`T-MIG-P4-11.merge-gate.log` fuori repo, SHA-256 f1a903b2…cd3c. Nessuna CI
remota o prova Windows: la qualifica resta al checkpoint del blocco P4.
G-MIG-4 e G-REL aperti. Prossime tecnicamente disponibili P4-09 e P4-12.

Chiusura T-MIG-P4-09, 19 novembre 2026: **DONE**, **63/128 (49,2%)**, 65
rimanenti. Autore Claude Code, commit 5027ee4 + rework 7d5cdbc. Review Codex
CHANGES_REQUESTED al round 1 (P409-01: tetto di 64 fonti; P409-02: guardia
`rule.when` omessa) e APPROVED al round 2, senza rilievi e con repliche
indipendenti (55 nativi, 38 Node, Clippy, DSL, riproduzioni 64/65 fonti).
Integrazione autorizzata dall'autore: fast-forward di integration su
7d5cdbc. Nessuna CI remota o prova Windows: la qualifica resta al checkpoint
del blocco P4. G-MIG-4 e G-REL aperti. Prossima tecnicamente disponibile
P4-12, poi P4-10.

Chiusura T-MIG-P4-12, 19 novembre 2026: **DONE**, **64/128 (50,0%)**, 64
rimanenti. Autore Claude Code, commit cab4a04 su base 02f5000 (sviluppato da
56627a1 e ribasato dopo P4-09). Review Codex APPROVED senza rilievi, con
repliche indipendenti (58 nativi, 38 Node, lint, build, fmt, Clippy) tutte
exit 0. Integrazione autorizzata dall'autore: fast-forward di integration su
cab4a04, nessun push. Nessuna CI remota o prova Windows: la qualifica resta al
checkpoint del blocco P4. G-MIG-4 e G-REL aperti. Prossima tecnicamente
disponibile P4-10, ultima task del blocco P4.

Chiusura T-MIG-P4-10, 19 novembre 2026: **DONE**, **65/128 (50,8%)**, 63
rimanenti. Autore Claude Code, commit 40ebd23+8ec47f8 su base ec6a9e0. Review
Codex round 1 CHANGES_REQUESTED (P410-01 BLOCKER, protezione input via alias),
round 2 APPROVED con repliche indipendenti (62 nativi, 41 Node, lint, build,
fmt, Clippy, sonda P410-01) tutte exit 0. Integrazione autorizzata
dall'autore: fast-forward di integration su 8ec47f8. Tutte le task del blocco
P4 sono DONE; G-MIG-4 resta aperto fino al checkpoint CI nativo Linux/Windows
del blocco. G-REL aperto. Prossimo passo: checkpoint G-MIG-4, poi T-MIG-P5-01.

Checkpoint G-MIG-4, 20 novembre 2026: **PASS** su 1ab34d8 (fix test-only del
`dead_code` Windows emerso nella CI44588006731 su 1ea8220). Push autorizzato di
integration e work/p1-foundations allo stesso SHA; CI44706977518 (7 job,
Verify MSVC 3.287 e Linux 3.323 test, layer, JSONL byte-identici, snapshot
cross-OS, profili memoria) e Desktop adapter 44706949307 (61/62 nativi, 41
Node, smoke X11) tutte success. Non qualifica Windows 11, installer o release:
G-MIG-8 e G-REL aperti. Conteggio task invariato 65/128. Prossima
tecnicamente disponibile T-MIG-P5-01.

Chiusura T-MIG-P5-01, 20 novembre 2026: **DONE**, **66/128 (51,6%)**, 62
rimanenti. Autore Claude Code, commit funzionali e3c316c+e726310 (prove
e1874f6, documentazione 3af2b3b) sul checkpoint G-MIG-4 73e28ee. Review
Codex round 1 BLOCKED (P501-01 consegna non isolata, P501-02 prove Windows
assenti), round 2 BLOCKED (P501-03 scala 2×), round 3 APPROVED con P501-04
MINOR documentale corretto. La prova a 200% su Windows ha rivelato una
finestra aperta oltre lo schermo: corretta in e726310 (finestra dentro la
work area su Windows). Evidenze native 1×/2× Linux e Windows 11 VM sui
binari finali. Commit separato dell'autore 8337525 (CLAUDE.md, graphify-out)
incluso su sua richiesta. Integrazione autorizzata dall'autore:
fast-forward di integration e work/p1-foundations, nessun push. CI remota sul
delta NOT_RUN, al checkpoint del blocco P5. G-MIG-5 e G-REL aperti. Prossima
tecnicamente disponibile T-MIG-P5-02.
