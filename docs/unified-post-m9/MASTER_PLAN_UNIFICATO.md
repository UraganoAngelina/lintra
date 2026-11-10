# LINTRA — MASTER PLAN UNIFICATO v1.0 — DECISIONI D01–D15 RATIFICATE
## Hardening post-M9 · SIOC cross-run · React/TypeScript/Tauri 2 · CAD storico · distribuzione interna

**Decisione dell'autore:** 10 novembre 2026, ratifica puntuale D01–D15 nel messaggio che accompagna questo piano.  
**Stato:** `USER_APPROVED / REPOSITORY_RATIFICATION_PENDING / GATES_NOT_VERIFIED`. L'atto dell'autore è già avvenuto: l'agente lo recepisce fedelmente; i gate tecnici NON diventano PASS per questo.  
**Gerarchia:** decisioni di questo piano (per le sole parti modificate) → ratifiche specifiche già approvate → contratti ARCH/DET/ADR e norme → audit post-M9 → preflight storico e migration kit v1.1. I requisiti nuovi non riscrivono retroattivamente i fatti o i risultati M9.  
**Deliverable:** applicazione Tauri 2 pronta per distribuzione interna ai colleghi autorizzati, con LocalOnly, Full multi-tracciato e workflow CAD storico raggiungibile; controlli SIOC cross-run implementati e collaudati nel perimetro normativo ratificato; affidabilità SQLite verificabile in caso di interruzione. **Non** un prodotto clinico certificato, multi-tenant enterprise o destinato a vendita.

## 0. Baseline verificabile e autorità delle fonti

- Le **140/140 task originali** rimangono `DONE`; M9 e QG-8 storici rimangono raggiunti. Non cambiare il loro consuntivo per introdurre la nuova roadmap.
- Audit post-M9: correzioni `AUD-M9-001/002` SQLite e `AUD-M9-003` README nel commit `af3bcccb6246a7ff00463c595f4c780a9f5c5195`, CI `42789338814` risultata positiva. Verificare comunque HEAD, diff, stato e corrispondenza della CI nel repository prima di usarla come baseline del nuovo lavoro.
- CAD83/C1/L1 library e snapshot approvati risultano qualificati nei dossier precedenti; la GUI Slint e la CLI di audit non espongono lo stesso workflow CAD operativo. Nessun FROZEN CAD, distribuzione nuova o release 1.0 si deduce da M9.
- Il `SLINT_TO_TAURI2_PREFLIGHT_REPORT.md` del **30 ottobre** fotografa una revisione precedente (117/140 task e PERSIST08 WIP): **non** usare quei numeri come stato attuale. Preservare invece i rilievi strutturali ancora applicabili: bridge Rust/JS, result paging bounded, offline, WebView, QG e installer.
- Il `LINTRA_MASTER_MIGRATION_PLAN.md` v1.1 conserva **89 task `T-MIG-P0..P9`**, i relativi prerequisiti e DEC-MIG-01/02/03/04/05/07/08/09 già approvate. Non crearne duplicati. `DEC-MIG-06` (baseline) si risolve mediante verifica tecnica, non chiedendo di nuovo la scelta di stack.
- Le **39 task U-*** rimangono la seconda famiglia atomica (vedere §8), con modifiche di scope e acceptance definite qui. Le loro schede precedenti, se discordanti, sono superate da questo documento e vanno sincronizzate prima dell'esecuzione.
- `DEC-SIOC-HIST-01` storico audit-only resta tracciato come decisione **superata prospetticamente** dalla nuova direttiva D11 per l'enforcement delle PK cross-run; nessuna eliminazione della storia delle decisioni o modifica retroattiva dei golden approvati.

## 1. Ratifica puntuale D01–D15 — vincoli di implementazione

| ID | Decisione dell'autore | Contratto operativo vincolante |
|---|---|---|
| **D01** | **A APPROVATA** | LocalOnly + Full nel percorso MIG; CAD83 storico/preview/review/approve nella stessa roadmap, con gate integrato e pubblicazione ai colleghi solo dopo qualifica delle funzioni promesse. Nessun bypass library. |
| **D02** | **MODIFICATA E APPROVATA** | Destinatari: **autore e colleghi autorizzati dall'autore e dall'azienda**, non clinici, non clienti, non enti sanitari. **UI e test devono essere pronti all'uso interno effettivo**, non soltanto demo sintetiche. Sviluppare/collaudare con fixture sintetiche; l'uso di dataset lavorativi eventualmente sensibili resta subordinato alle autorizzazioni aziendali effettive, non deducibili dalla sola ratifica tecnica. Niente programma di compliance enterprise inventato. |
| **D03** | **MODIFICATA E APPROVATA** | Dati, database, snapshot, configurazioni, spool e output salvati su **filesystem locale dell'utente** in cartelle per-account, con protezione ordinaria dell'OS/ACL e nessun server/cloud. Niente cifratura applicativa o password custom per cartella; eventuale protezione/password OS-native è **facoltativa**, non requisito d'avvio. HMAC esistente non viene spacciato per cifratura. Nessuna nuova retention automatica. |
| **D04** | **MODIFICATA E APPROVATA** | **Export sempre disponibili** quando tecnicamente possibile: path nello stesso filesystem locale ammessi senza nuovo consenso; per una cartella condivisa/remota, **avviso comprensibile e conferma esplicita dell'operatore**, poi procedere. Non proibire export condivisi per policy predefinita. Conservare byte/formato canonico, path/overwrite confirmation e receipt su errori/parzialità. |
| **D05** | **MODIFICATA E APPROVATA** | Nessun motore di retention, purge automatico, TTL, scadenza dei report o cancellazione storica programmata. Uso interno di operatori AlmaViva autorizzati; conservazione/cancellazione restano sotto gestione dell'operatore e delle eventuali prescrizioni aziendali applicabili. Non affermare che ciò esoneri da regole di trattamento dei dati; l'agente non introduce un progetto enterprise. |
| **D06** | **A APPROVATA** | Mantenere il file chiave comune ratificato (32 byte) e key ID, con backup/custodia e recupero documentati; nessuna rotazione automatica o cambio della semantica HMAC. Drill sintetici di perdita/copia errata/compromissione. |
| **D07** | **A APPROVATA** | `BACKUP-B`: backup manuale **consistente e verificato** prima di qualunque upgrade storage potenzialmente irreversibile; restore isolato e controllato, chiave necessaria recuperabile, blocco dell'upgrade se backup non verificato. Non confondere backup live con copia del database aperto. |
| **D08** | **A APPROVATA** | JSONL/CSV canonici byte-fedeli; XLSX con celle testo per uso spreadsheet. Nessuna sanificazione silenziosa del CSV canonico; eventuale variante safe separata richiede bisogno reale. |
| **D09** | **A APPROVATA CON OVERRIDE UTENTE** | Budget di input/reference motivati da misure, non numeri arbitrari. Se si supera un **budget operativo soft**, mostrare l'avviso e consentire **«Procedi comunque»**, con conferma e tracciatura della scelta; nessun taglio silenzioso. Limiti **hard** di correttezza/sicurezza (overflow, impossibilità di allocazione, errore I/O, limiti di formato, protezioni OS) non sono aggirabili: fallire esplicitamente. La disattivazione del soft budget non salta audit/EOF/SHA/cancellation. |
| **D10** | **A APPROVATA** | Reference tree locale e controllato, ownership/permessi verificati, caricamento realmente verificato, nessun modello ostile multiutente inventato. Se la tree non è attendibile, segnalare blocco/errore preciso; nessuna promessa anti-TOCTOU non testata. |
| **D11** | **ALTERNATIVA A RESPINTA — NUOVA DIRETTIVA NORMATIVA APPROVATA** | **SIOC: la non ripetizione delle chiavi primarie tra invii è una regola cross-run vincolante da implementare e portare in collaudo**, non solo audit. L'autore ne dichiara la provenienza normativa. Ratificare il cambiamento rispetto a `DEC-SIOC-HIST-01`, preservando la traccia della decisione storica; usare PK esatte per tracciato e namespace, distinguere nuove trasmissioni da record già conservati nello snapshot. Casi di rettifica/reinvio/annullamento non definiti non ricevono eccezioni inventate: ricostruire le clausole della fonte normativa disponibile e segnalare solo autentiche ambiguità di comportamento. |
| **D12** | **MODIFICATA E APPROVATA** | **Dataset disponibili definitivi**: non sono previsti ulteriori dataset. Conservare il **perimetro di controlli attualmente progettato e qualificato**, con deferred/assenze trasparenti; non avviare campagne di sourcing, non fabbricare cataloghi, non inventare nuove regole, non pretendere conformità regionale universale. Nuovo sviluppo normativo soltanto per D11 o per bug dimostrati negli attuali controlli. |
| **D13** | **A APPROVATA** | Profiling sui carichi realistici, niente nuove soglie DEC-14 o parallelismo senza evidenze. Lasciare DEC-17 D nel runtime sequenziale. |
| **D14** | **MODIFICATA E APPROVATA** | Obiettivo: **applicazione desktop realmente installabile e utilizzabile dai colleghi autorizzati**, Windows 11 x86_64 NSIS offline per-machine, app per-account, aggiornamento manuale e uninstall che conserva i dati; Linux solo build personale CachyOS. Gate reali install/update/rollback, LocalOnly/Full/CAD storico, export e SIOC cross-run. Una release interna qualificata è in scope; nessun FROZEN clinico o 1.0 pubblico implicito. |
| **D15** | **MODIFICATA E APPROVATA** | **Robustezza contro power loss hardware come requisito ingegneristico**: proteggere atomicità SQLite/snapshot, durabilità dei commit confermati secondo il modello OS/filesystem dichiarato, recupero post-riavvio e report/export senza falsa completezza; verifiche di fault injection, process termination e, se disponibile, test di interruzione reale controllata. Non promettere immunità assoluta a guasti fisici/firmware/storage che mente sui flush senza prova empirica. |

**Confermate senza nuova domanda:** React + TS strict + Vite + Tailwind/shadcn + TanStack + Tauri 2; Rust proprietario di PHI/dati, job manager e IPC DTO tipizzati bounded; una validazione attiva per istanza; no rete/telemetria/CDN/auto-update; Windows11 x86_64 NSIS `offlineInstaller`/`perMachine`, dati in `%LOCALAPPDATA%/Lintra`; Linux UI soltanto per build/smoke personale CachyOS/Hyprland; redesign moderno responsive; rollback di Slint sino al cutover qualificato; CLI/core invariati quanto a regole; no nuovi algoritmi crittografici.

## 2. Risultato d'uso finale e confini

Il prodotto consegnabile deve consentire a un collega autorizzato di installare su Windows 11 e usare, senza una CLI o codice manuale:

1. **LocalOnly:** selezionare schema/trace/file, validare, annullare, leggere diagnostiche/guide, esportare.
2. **Full:** selezionare esplicitamente file multi-trace, eseguire le due passate, consultare report cross-record/cross-trace, esportare, distinguere `CompletedValid`, `CompletedInvalid`, `Partial`, `Cancelled`, errore tecnico e `Unknown`.
3. **CAD storico:** gestire namespace/periodo/archivio/chiave, origini C1, omissioni e consenso, preview, review esatta, approvazione atomica con CAS, consultazione storico e annullamenti L1; **mai** default implicito che approvi o renda completo un Partial/Unknown.
4. **SIOC cross-run:** applicare la regola approvata di non ripetizione PK fra invii nel perimetro normativo qualificato; distinguere violazione, assenza di baseline, storico parziale/incompatibile ed errore tecnico, senza inventare eccezioni.
5. **Export:** scegliere liberamente una destinazione locale accessibile o shared, con avviso/consenso shared, preservando comportamento canonico e failure semantics.
6. **Durabilità:** interruzioni/crash/riavvio non devono lasciare snapshot approvati parziali né mostrare report incompleti come completi; ripristino controllato da backup.
7. **Distribuzione:** installer Windows offline, upgrade manuale non distruttivo, disinstallazione conservativa e smoke non-admin su più account. La GUI Linux personale resta non ufficiale.

La qualifica tecnica del software è indipendente dal ruolo clinico degli utenti. **L'uso di dati di pazienti o altre informazioni personali fornite dal datore di lavoro deve rispettare le autorizzazioni e i controlli aziendali applicabili**: una decisione del progetto non sostituisce tali autorizzazioni. Non inventare un gate enterprise né dichiarare un’autorizzazione organizzativa non documentata. Test/CI/screenshot sempre sintetici.

### File e dati

- App installata in Program Files, senza database mutabili nella directory dell'eseguibile.
- DB, spool, cache di lavoro, configurazione e snapshot sotto directory per-account dell'OS; cartella selezionabile solo con controllo di permessi/ownership e prevenzione dell'overwrite accidentale.
- Cartella locale sempre autorizzata per export se raggiungibile/scrivibile. Per UNC/mount remoto/shared riconosciuto: warning e conferma; se la natura shared non può essere determinata, dichiararlo e consentire conferma esplicita, senza false affermazioni di privacy.
- Non aggiungere password casalinghe, cifratura nuova o retention forzata. Protezioni OS di base non sono opzionali: preservare ACL/ownership e non esporre per errore file temporanei.
- Backup/restore funzionano solo con inventari, chiavi e stato compatibili; non promettere secure-erase fisico.

## 3. Nuova regola SIOC cross-run — supersessione controllata

1. Registrare **`DEC-SIOC-HIST-02`** (ID suggerito, controllare collisioni in repository): requisito richiesto dall'autore in questa ratifica. Essa **supera per le nuove versioni del profilo** la precedente policy di sola osservazione `DEC-SIOC-HIST-01`; la vecchia decisione rimane storicizzata, non riscritta.
2. Leggere il tracciato normativo SIOC effettivo presente, le PK per ogni trace, le regole di periodo/namespace e le indicazioni su rettifica/reinvio. Collegare il requisito alla fonte verificabile; non inventare contenuti di norme mancanti. La dichiarazione dell'autore è una decisione di prodotto vincolante a implementare il controllo, non prova indipendente del testo normativo.
3. Implementare tramite schema/capability/persistence **generica**, non branch hardcoded `if SIOC`; niente violazioni cross-run arbitrarie per CAD o per altri flussi.
4. In un confronto, considerare la **nuova trasmissione** contro le trasmissioni approvate pertinenti; non contare come nuovo duplicato una medesima occorrenza semplicemente conservata in ogni snapshot storico. Non usare la sola presenza di una PK in un indice cumulativo per inferire una nuova trasmissione.
5. Una PK in una nuova trasmissione che viola il vincolo cross-run comporta diagnostica normativa e blocco di approvazione/conformità secondo i contratti di validazione. Se una chiave storico è Unknown, mancante o incongruente, non dichiarare falsamente una nuova assenza; errore tecnico o Unknown secondo la causa.
6. Non introdurre eccezioni per resubmission/correzioni se non attestate; per casi realmente controversi, produrre una matrice concisa con le opzioni e continuare su test indipendenti, non disattivare il controllo.
7. Golden sintetici: due invii con medesima PK, due ULSS/namespaces distinti, snapshot multipli senza nuove occorrenze, replay idempotente, duplicati intra-run, annullamenti, Missing/Absent, history incompleto, restart/SQLite, report/JSONL identici su OS. Test chiari `PASS/FAIL/UNKNOWN`, niente aggiornamento dei golden legacy per occultare una regressione.
8. **Acceptance G-SIOC1:** policy versionata e compilabile, comportamento reale end-to-end su nuova versione del profilo, CLI e frontend che usano la stessa fonte di validazione e l'interfaccia non simula nessuna norma. Se la modalità `SIOC` storica richiede un nuovo entrypoint UI, qualificarlo prima del rilascio interno.

## 4. Budget risorse con override esplicito

- Distinguere **preflight soft** (dimensioni input/reference, durata stimata solo se supportata da evidenze, avviso) da **hard bounds** (safety/overflow, formato, limite hardware o integrità) che nessun click può ignorare.
- Se soft superato, l'utente sceglie **Annulla** o **Procedi comunque**; l'autorizzazione è legata alla richiesta/insieme file verificati (no reuse su file sostituiti) e annotata senza inviare identità cliniche nei log.
- Se procede, eseguire un run completo: non troncare record, non tagliare audit, non saltare EOF, hashing, vincoli normativi, max_errors/cancellazione originale. Se il run poi fallisce per limiti fisici, restituire un errore tecnico esplicito e report coerente.
- Applicare medesima semantica nella CLI dove ha senso (flag esplicito, non implicitamente `yes`), e nella GUI React con dialogo accessibile; compatibilità CLI precedente salvo contratto/versionamento approvato.
- Scegliere parametri soft con dati di prova e configurazione versionata; **nessun numero inventato** da DEC-14.

## 5. Power loss: obiettivo raggiungibile e verifiche

- Preservare il protocollo SQLite qualificato (`DELETE/EXTRA` dove presente) e verificare effettivamente le PRAGMA/journal/synchronous in uso prima di cambiarle. Non sostituire con WAL per intuito.
- Preparare test deterministici con fault injection ai punti critici: creazione file, scrittura dati, flush/sync, manifest, receipt, CAS, prima/durante/dopo COMMIT, report staging e rename, riapertura dopo terminazione improvvisa.
- Invarianti: mai snapshot approvato a metà; mai un fatto storico consultabile senza snapshot coerente; se commit riuscito ma risposta persa, recovery distingue esito ambiguo da commit assente e non duplica; corruzione/prova invalida produce errore; report e DB non sono erroneamente presentati come transazione atomica unica.
- Per file system, testare persistenza di rename/directory sync ove necessari, limitazioni Windows e Linux, spazio esaurito, errori permessi e recovery da copie; non promettere `fsync` miracoloso su hardware che non onora i flush.
- Definire chiaramente la garanzia: **atomicità e recovery testati contro interruzioni** sui filesystem/OS qualificati; durabilità assoluta dopo perdita di alimentazione **non dimostrabile** senza hardware controllato e un modello di guasto. Se non si può eseguire test fisico, l'agente non deve trasformare `NOT_TESTED_HARDWARE` in PASS ma può consegnare il software interno con limite noto, se i gate software obbligatori sono positivi.

## 6. Integrazione migrazione UI: reuse e delta

Il migration kit v1.1 aveva `STRATEGY_USER_APPROVED`, ma G-MIG-0 doveva ancora recepire le ADR. Questo messaggio **conferma la ratifica delle scelte di prodotto e l'autorizzazione alla migrazione**; G-MIG-0 resta condizionato alla baseline verificata, ai documenti coerenti e ai test.

### Decisioni MIG già approvate

- React + TypeScript strict + Vite + Tailwind CSS + shadcn/ui + TanStack Table/Virtual + Tauri 2.
- Una validazione attiva per istanza, Rust Job Manager, IPC tipizzato a payload limitato, progress bounded, query/filtri/paging e index su Rust. Il JS non valida campi, non riceve l'intero JSONL e non possiede PHI persistenti.
- Windows 11 x86_64 GUI ufficiale, NSIS `.exe` offline per macchina, WebView2 bundled/offline installer, installazione elevata ed esecuzione non elevata. `%LOCALAPPDATA%/Lintra` per account; installer conserva dati all'upgrade/uninstall. Nessun updater automatico, server, CDN o telemetria.
- Linux soltanto build e smoke personale sul PC CachyOS/Hyprland/Wayland; niente release Linux GUI ufficiale. CLI e core mantengono la propria matrice CI cross-OS.
- Redesign moderno responsive: parità **comportamentale**, non pixel-identità Slint/WebView; golden normativi e JSONL invariati. Slint conservato fino a cutover qualificato e rollback dimostrato.

### Delta sulle 89 task esistenti (senza rinumerazione)

1. `T-MIG-P0-01/02/08`: aggiornare la baseline da 30/10 pre-PERSIST08 alla HEAD post-audit e verifica CI; recepire il presente mandato, senza chiedere nuova ratifica D01–D15.
2. `T-MIG-P0-06`, `T-MIG-P2-08`, `T-MIG-P7-03`: policy locale e per-account, dati di test sintetici, nessuna nuova crypto o storage JS; uso interno effettivo non deve essere etichettato solo demo, ma nessuna autorizzazione aziendale presunta.
3. `T-MIG-P2-07`, `T-MIG-P4-10`, `T-MIG-P6-08`: export sempre disponibile con dialogo per shared/remote e conferma per overwrite; no alterazione byte del report.
4. `T-MIG-P4-11/12`, `T-MIG-P5-09/10`, `T-MIG-P6-10`, `T-MIG-P7-09`: Full multi-tracciato obbligatorio con parità Rust/CLI, niente logica normativa JS.
5. `T-MIG-P7-04`: conservare benchmark 300k/1M solo come stress test UI, aggiungere misure sui volumi realistici; non usarli per decidere parallelismo.
6. `T-MIG-P8-04/07`, `T-MIG-P9-01/03/06/07`: spostare il criterio finale da technical preview sintetica a **installer interno pronto ai colleghi**, con prove Windows clean-room, account distinti, backup pre-upgrade, uninstall conservativo, esiti Full/CAD/SIOC e release consentita solo dopo i gate pertinenti.
7. Le 89 task originarie avevano esplicitamente escluso CAD storico dalla UI: `U-CAD-01..09` lo aggiunge senza fingere che sia già parte della loro acceptance. Il cutover Full `MM9` e il gate CAD UI `G-CADUI3` restano separati; **la distribuzione interna finale `G-REL` richiede ENTRAMBI**, oltre a `G-SIOC1` per la funzione D11 promessa.
8. Un passaggio di cutover interno/temporaneo `MM9` non è ancora autorizzazione a distribuire ai colleghi: preservare rollback e non rimuovere Slint prima della propria parità effettiva; la release interna finale deve superare il gate aggregato.
9. I contratti IPC/gate di sicurezza non diventano informali perché il progetto è piccolo: attacchi XSS/JS bridge e integrità dei report rimangono problemi tecnici da collaudare con fixture sintetiche. Non creare processi/ruoli enterprise o nuovi sistemi di autenticazione remota.

## 7. Roadmap e gate (ordine per dipendenze, un solo agente)

| Gate | Attività | Uscita reale |
|---|---|---|
| **G-U0** | U-GOV-01..04 | Baseline HEAD e CI verificata, decisioni D01–D15 registrate nell'ADR e plan/status separato; niente nuova domanda di conferma. |
| **G-HARD** | U-FIX-01..05 | Regression SQLite preservate, CSV e reference verificati, analisi soft-budget e audit fonte GAP-17. |
| **G-OPS0/1/2** | U-OPS-01..08 | Setup per account e file locali, avvisi export, custodia chiave comune, Backup-B e restore; niente progetto cifratura o retention. Task indipendenti non bloccano test UI sintetici. |
| **G-SIOC0** | U-NORM-01, U-NORM-02, U-NORM-05 | Matrice di PK e condizioni cross-run; dataset fissi, norma individuata e nuova DEC superseding ratificata. |
| **G-SIOC1** | U-NORM-03/04 + U-NORM-05 | Nuovo profilo/versione che applica e collauda il vincolo PK cross-run, backend/CLI/test nativi, niente campi inventati. |
| **G-MIG-0..9** | 89 T-MIG + U-UI-01/02 | React/Tauri 2 con LocalOnly+Full, Windows11 NSIS offline e build personale CachyOS, parità e rollback. |
| **G-CADUI0..3** | U-CAD-01..09 | T1/T2/T3 storico dalla nuova GUI, preview/review/approve CAS, due namespace, cancel/Unknown/Partial. |
| **G-DUR** | U-REL-03 più test storage/FS | Matrice fault injection, recovery SQLite/report, nessuna attestazione hardware non eseguita. |
| **G-PERF** | U-PERF-01/02 | Volumi realistici e profiling motivato; stress data-grid separato. |
| **G-REL** | U-REL-01/02 e signoff finale | **Distribuzione interna ai colleghi** su SHA nativo qualificato: G-MIG-9, G-CADUI3, G-SIOC1, G-OPS2 e G-DUR positivi, nessun P0/P1 aperto. |
| **G-REAL** | U-OPS-09 (soltanto se dati personali/lavorativi lo richiedono) | Checklist delle autorizzazioni e regole aziendali effettive; non gate enterprise generico per i test sintetici o la funzionalità della GUI. |

**Critical path indicativo:** `G-U0 → G-MIG-0/1/2/3/4/5/6/7/8/9 → G-CADUI3` e in parallelo **logico, mai con agenti concorrenti**, `G-SIOC1`, `G-OPS2`, `G-DUR`; convergenza obbligatoria in `G-REL`. Se mancano attrezzature per Windows11 clean-room, registrare blocker tecnico: CI non sostituisce un installer testato.

**Politica STOP:** bloccare soltanto le task dipendenti da una decisione normativa davvero indefinita o da una prova tecnica non disponibile. Continuare con la successiva task indipendente READY, in sequenza. Non derogare a cancellazione, atomicità, provenienza, privacy minima e determinismo per ottenere una release.

## 8. Registro dei 17 GAP → risoluzione proposta

| GAP | Esito pianificato | Task |
|---|---|---|
| GAP-01 | Ambito interno autorizzato, setup utenti e comunicazione capacità, senza gate enterprise | U-OPS-01 / U-OPS-09 |
| GAP-02 | File solo locali + ACL/account, inventario superfici; niente cipher app | U-OPS-02/03 |
| GAP-03 | Nessun purge/TTL; gestione manuale con cautela su copie | U-OPS-07 |
| GAP-04 | Chiave comune invariata, recovery e compromissione | U-OPS-05 |
| GAP-05 | Backup-B verificato prima degli upgrade e restore isolato | U-OPS-06/08 |
| GAP-06 | Export locale libero, warning/consenso shared e test permessi | U-OPS-04 / T-MIG-P6-08 |
| GAP-07 | CSV raw invariato, XLSX per spreadsheet | U-FIX-02 |
| GAP-08 | Budget soft con override, hard solo correttezza/safety | U-FIX-03 / T-MIG-P5-05 |
| GAP-09 | Reference tree controllata/ownership | U-FIX-04 |
| GAP-10 | Full in Tauri già richiesto da MIG | T-MIG-P4-11/12, P5-09/10, P6-10, U-UI-01/02 |
| GAP-11 | CAD storico completo in UI, gate separato ma richiesto per release interna | U-CAD-01..09 |
| GAP-12 | Dataset attuali definitivi, controlli attuali senza nuovo sourcing | U-NORM-01/02/03/04 (ridimensionate) |
| GAP-13 | **PK SIOC cross-run enforce**, nuova DEC rispetto ad audit-only | U-NORM-05 + U-NORM-03/04 |
| GAP-14 | Profiling realistico/DEC-14 senza parallelismo | U-PERF-01/02 |
| GAP-15 | Installer e pubblicazione interna su SHA qualificata | T-MIG-P8/9, U-REL-01/02 |
| GAP-16 | Power-loss hardening tecnico e recovery/fault injection | U-REL-03; G-DUR |
| GAP-17 | Confronto del piano originale dell'audit, ora reperito | U-FIX-05 |

## 9. Task atomiche aggiuntive: ID, dipendenze e acceptance aggiornate

Le 39 task originali **non** sono tutte nuove: mantenerne ID e storico. Per ogni task eseguire solo la differenza rispetto al codice già qualificato. Nessuna task `U-*` è una riscrittura di una `T-MIG-*`.

| ID | Focus aggiornato | Prerequisiti vincolanti | Gate | Acceptance primaria |
|---|---|---|---|---|
| U-GOV-01 | Baseline post-audit e CI | — | G-U0 | HEAD/working tree, commit af3bccc e CI 42789338814 verificati, sorgenti qualificati distinguibili dalle modifiche nuove |
| U-GOV-02 | Riconciliazione preflight 30/10 vs 10/11 | U-GOV-01 | G-U0 | Matrice di contratti ancora validi e affermazioni obsolete, senza perdita storia |
| U-GOV-03 | Ratifica D01–D15 e DEC SIOC | U-GOV-02 | G-U0 | Decisioni del messaggio utente recepite testualmente/semanticamente, DEC-SIOC-HIST-01 superseded prospetticamente; nessuna riapprovazione richiesta |
| U-GOV-04 | Dependency map, dossier e status | U-GOV-03 | G-U0 | Grafo pronto per MIG+U; milestone originali conservate, nuova governance single-agent |
| U-FIX-01 | Conservare regressioni SQLite | U-GOV-01 | G-HARD | AUD-M9-001/002 riproducibili e verdi sul nuovo SHA; nessun refactoring |
| U-FIX-02 | Consumo CSV/XLSX | U-GOV-03 | G-HARD | Test/formati invariati, casi formula non interpretati come obbligo di mutare CSV |
| U-FIX-03 | Budget soft e hard | U-GOV-03 | G-HARD | Preflight + warning/override richiesta identica, overflow/hard rifiutati, no troncamento/audit alterato |
| U-FIX-04 | Reference tree controllata | U-GOV-03 | G-HARD | Permessi e symlink controllati, errori chiari se deployment non attendibile |
| U-FIX-05 | Test ignored e fonte audit | U-GOV-02 | G-HARD | Evidenze ignored e confronto retro-audit originario; nessun PASS inventato |
| U-OPS-01 | Scope uso interno / cartelle per account | U-GOV-03 | G-OPS0 | Modello mono-PC/local-only, utenti autorizzati, nessun gate enterprise creato |
| U-OPS-02 | Inventario dati/report/spool locale | U-OPS-01 | G-OPS0 | Mappa percorsi OS, nessun contenuto PHI in web storage/log, ownership backend |
| U-OPS-03 | Permessi OS e continuità file locali | U-OPS-02 | G-OPS1 | ACL/ownership corrette su Windows/Linux target; nessun nuovo crypto/password custom |
| U-OPS-04 | Export locale/shared | U-OPS-02 | G-OPS1 | Locale scrivibile libero, shared warning + conferma + receipt, failure e overwrite gestiti |
| U-OPS-05 | Chiave comune e recovery | U-OPS-01 | G-OPS1 | File 32B invariato, ID/restore veri su synthetic, no auto rotation |
| U-OPS-06 | BACKUP-B manuale consistente | U-OPS-03, U-OPS-05 | G-OPS2 | Backup verificato obbligatorio prima upgrade irreversibile, stop se fallito, no live-copy finta |
| U-OPS-07 | Nessun retention/purge automatico | U-OPS-01, U-OPS-02 | G-OPS2 | Documentare gestione manuale, preserve immutabilità e copie, zero TTL/erase promesso |
| U-OPS-08 | Restore isolato | U-OPS-06 | G-OPS2 | Reopen, manifest/namespace/fatti/chiave errata/danno verificati, nessuna sovrascrittura inconsapevole |
| U-OPS-09 | Check contesto d'uso e autorizzazioni effettive | U-OPS-03, U-OPS-04, U-OPS-05, U-OPS-07, U-OPS-08 | G-REAL | Checklist aziendale applicabile se si introducono dati personali; non blocca demo, GUI o gate tecnici senza motivo |
| U-NORM-01 | Inventario SIOC PK e esclusioni esistenti | U-GOV-03 | G-SIOC0 | PK esatte per trace, elementi deferred storici distinguibili da norme attive |
| U-NORM-02 | Freeze elenco dataset disponibili | U-NORM-01 | G-SIOC0 | Inventario fonti già consegnate con hash/versioni; niente sourcing di nuovi dataset |
| U-NORM-03 | Enforcement generico PK cross-run SIOC | U-NORM-01, U-NORM-02, U-NORM-05 | G-SIOC1 | Nuovo profilo versionato; stessi event scope/namespace, duplicate nuove trasmissioni rilevate, Unknown fail-closed |
| U-NORM-04 | Golden cross-run e regressioni | U-NORM-03 | G-SIOC1 | Duplicate/unique/due namespaces/legacy/restart/periodi/annullamenti e stessa CLI/UI, CI nativa |
| U-NORM-05 | Supersessione DEC-SIOC-HIST-01 | U-NORM-01 | G-SIOC0 | ADR/DEC con fonte normativa e semantica; casi davvero dubbi isolati, mai audit-only come criterio finale |
| U-UI-01 | Ponte MIG v1.1→post-audit | U-GOV-04 | G-MIG-0 | Riferimenti 89 task alla HEAD post-audit, no duplicati né status stale |
| U-UI-02 | Scorecard LocalOnly/Full e CAD handoff | T-MIG-P9-07 | G-MIG-9 | Parità reale e installer qualificati, handoff a CAD senza dichiarare collega-ready |
| U-CAD-01 | Scope storico Tauri | U-UI-02, U-OPS-02 | G-CADUI0 | Funzioni library esatte e permessi, no duplicazione norme Rust |
| U-CAD-02 | DTO storici / preview non forgiabile | U-CAD-01 | G-CADUI1 | Candidate handle backend, DTO versionati e negativi |
| U-CAD-03 | Preflight archive+key+namespace | U-CAD-02, U-OPS-05 | G-CADUI1 | Identità unica e HMAC verificati; nessun path arbitario JS |
| U-CAD-04 | Origini C1/omissioni/consenso | U-CAD-03 | G-CADUI1 | Origini, provenance, omissioni esatte, PROCEED non equivale Complete |
| U-CAD-05 | Preview e gating Unknown/Partial | U-CAD-04 | G-CADUI2 | Candidate mostrato esattamente, non approvabile se inammissibile |
| U-CAD-06 | Review esatta e CAS commit | U-CAD-05, U-OPS-04 | G-CADUI2 | Approvazione di un candidato verificato, rollback e ambiguità esplicite |
| U-CAD-07 | History read/diff e isolamento account | U-CAD-06 | G-CADUI2 | Query provenienza/snapshot senza dati bulk nel JS, account scoping |
| U-CAD-08 | End-to-end T1/T2/T3 cross-OS | U-CAD-07 | G-CADUI3 | 2 namespace, annullamenti L1, C1, report, errori, restart, no UI fittizia |
| U-CAD-09 | Gate CAD UI finale | U-CAD-08 | G-CADUI3 | CI Windows/Linux applicabili, disclosure capability, readiness interna per funzione |
| U-PERF-01 | Volumi rappresentativi da fonte anonima | U-GOV-02 | G-PERF | Carichi ragionevoli con disclosure ipotesi, non 500k come default |
| U-PERF-02 | Profiling end-to-end/React bridge | U-PERF-01, T-MIG-P7-08 | G-PERF | Costi attribuiti, regressioni motivate e DEC-17 non riaperta senza prova |
| U-REL-01 | Scope distribuzione interna, no freeze clinico | U-UI-02, U-CAD-09, U-NORM-04, U-OPS-08 | G-REL | Versione e capability effettive, nessun P0/P1; installer per colleghi autorizzati |
| U-REL-02 | NSIS per-machine e update/uninstall conservativi | U-REL-01, U-OPS-06, U-REL-03 | G-REL | Install offline Windows11 clean-room 2 account, preserva DB e backup, art SHA |
| U-REL-03 | Power loss / durability hardening | U-OPS-01, U-FIX-01 | G-DUR | Fault injection/kill/reopen/manifest/SQLite/report e matrice OS, limite hardware esplicito |

### Note sulle dipendenze

- Gli ID U-* originali e le 89 T-MIG-* **non sono rinumerati**. Le task `U-NORM-03/04/05`, `U-OPS-01..09`, `U-FIX-03`, `U-REL-01..03` sono **sostanzialmente rispecificate** da questa ratifica; i prompt atomici preesistenti devono essere aggiornati prima dell'esecuzione.
- `U-OPS-02/03/04` possono essere disegnate prima di MIG-P4 ma integrate nella GUI quando il bridge esiste; i gate della task devono controllare la parte implementata, non assumere codice futuro. Split di task se indispensabile senza riutilizzare ID originali; modificare dependency map in modo coerente, non segnare DONE per una UI inesistente.
- `U-NORM-03/04` è un percorso normativo **indipendente** dalla creazione della GUI; l'eventuale bridge SIOC parte solo dopo G-MIG-4 e la qualifica del servizio. `G-REL` richiede la capacità usabile promessa.
- Il vecchio G-REAL permane come gate di **conformità alle autorizzazioni aziendali per dati lavorativi**, non come motivo per dichiarare la GUI inutilizzabile o esigere un programma enterprise. L'autore non dispone tramite questa ratifica di autorizzazioni legali/aziendali non accertate.

## 10. Task T-MIG originali preservati per indice

Tutte le **89 task** conservano i propri identificativi, dipendenze tecniche originali e acceptance, salvo i delta espliciti del §6 e i gate della release interna. I prompt T-MIG canonici restano fonte più dettagliata, ma nessuna istruzione pregressa può ripristinare D11 audit-only o bloccare gli export shared a prescindere dalla conferma.

## Task di migrazione — indice preservato (89)

| ID | Task originale | Prerequisiti | Gate |
|---|---|---|---|
| T-MIG-P0-01 | Fotografia Git e baseline post-WIP08 | — | G-MIG-0 |
| T-MIG-P0-02 | Riconciliazione dello stato e dei riferimenti storici | T-MIG-P0-01 | G-MIG-0 |
| T-MIG-P0-03 | ADR di sostituzione mirata N-24 | T-MIG-P0-02 | G-MIG-0 |
| T-MIG-P0-04 | Perimetro LocalOnly+Full, parità e capability future | T-MIG-P0-03 | G-MIG-0 |
| T-MIG-P0-05 | Decisione OS minimi e distribuzione offline | T-MIG-P0-03 | G-MIG-0 |
| T-MIG-P0-06 | Policy PHI e threat model da ratificare | T-MIG-P0-03 | G-MIG-0 |
| T-MIG-P0-07 | Criteri di parità UX/visiva e budget prestazionale | T-MIG-P0-04, T-MIG-P0-05 | G-MIG-0 |
| T-MIG-P0-08 | Gate governance e autorizzazione all’avvio | T-MIG-P0-01, T-MIG-P0-02, T-MIG-P0-03, T-MIG-P0-04, T-MIG-P0-05, T-MIG-P0-06, T-MIG-P0-07 | G-MIG-0 |
| T-MIG-P1-01 | Spike build minimale Tauri su Linux | T-MIG-P0-08 | G-MIG-1 |
| T-MIG-P1-02 | Spike Windows clean install e WebView2 | T-MIG-P0-08 | G-MIG-1 |
| T-MIG-P1-03 | Qualifica runtime CachyOS e dipendenze | T-MIG-P1-01 | G-MIG-1 |
| T-MIG-P1-04 | POC zero-network, capability e CSP | T-MIG-P1-01, T-MIG-P1-02 | G-MIG-1 |
| T-MIG-P1-05 | Profilo WebView, font e comportamento DPI | T-MIG-P1-01, T-MIG-P1-02 | G-MIG-1 |
| T-MIG-P1-06 | Decisione GO/NO-GO piattaforme | T-MIG-P1-02, T-MIG-P1-03, T-MIG-P1-04, T-MIG-P1-05, T-MIG-P1-07 | G-MIG-1 |
| T-MIG-P1-07 | Spike anticipato installer NSIS offline per-machine su Windows 11 | T-MIG-P1-02, T-MIG-P1-04 | G-MIG-1 |
| T-MIG-P2-01 | Contratto architetturale e ownership processo | T-MIG-P1-06 | G-MIG-2 |
| T-MIG-P2-02 | Envelope JSON DTO versionati e error taxonomy | T-MIG-P2-01 | G-MIG-2 |
| T-MIG-P2-03 | Run identity, ownership e macchina stati | T-MIG-P2-02 | G-MIG-2 |
| T-MIG-P2-04 | Progresso bounded e terminale affidabile | T-MIG-P2-03 | G-MIG-2 |
| T-MIG-P2-05 | Contratto risultati: paging, offset e filtri | T-MIG-P2-02, T-MIG-P2-03 | G-MIG-2 |
| T-MIG-P2-06 | Contratto dettaglio e domini catturati | T-MIG-P2-05 | G-MIG-2 |
| T-MIG-P2-07 | Contratto export e conferma path/overwrite | T-MIG-P2-03 | G-MIG-2 |
| T-MIG-P2-08 | Capability, CSP, path e log policy | T-MIG-P2-02, T-MIG-P2-07 | G-MIG-2 |
| T-MIG-P2-09 | Contratto di parity test/fixture sintetica | T-MIG-P2-03, T-MIG-P2-04, T-MIG-P2-05, T-MIG-P2-06, T-MIG-P2-07, T-MIG-P2-08 | G-MIG-2 |
| T-MIG-P2-10 | Ratifica cross-contract e gate di design | T-MIG-P2-01, T-MIG-P2-02, T-MIG-P2-03, T-MIG-P2-04, T-MIG-P2-05, T-MIG-P2-06, T-MIG-P2-07, T-MIG-P2-08, T-MIG-P2-09, T-MIG-P2-11 | G-MIG-2 |
| T-MIG-P2-11 | Contratto Full multi-tracciato: input, run e report coerenti | T-MIG-P2-03, T-MIG-P2-05, T-MIG-P2-08 | G-MIG-2 |
| T-MIG-P3-01 | Scaffold Tauri affiancato non predefinito | T-MIG-P2-10 | G-MIG-3 |
| T-MIG-P3-02 | Pin frontend e lockfile/supply-chain | T-MIG-P3-01 | G-MIG-3 |
| T-MIG-P3-03 | Copia autorizzata token/font/asset | T-MIG-P3-01 | G-MIG-3 |
| T-MIG-P3-04 | Rust app state e dispatcher senza business logic | T-MIG-P3-01, T-MIG-P3-02 | G-MIG-3 |
| T-MIG-P3-05 | Client TS tipizzato e verifica schema bridge | T-MIG-P3-02, T-MIG-P3-04 | G-MIG-3 |
| T-MIG-P3-06 | CI scaffold e verifica regressione legacy | T-MIG-P3-01, T-MIG-P3-02, T-MIG-P3-03, T-MIG-P3-04, T-MIG-P3-05 | G-MIG-3 |
| T-MIG-P4-01 | API catalogo bundle e selezione | T-MIG-P3-06 | G-MIG-4 |
| T-MIG-P4-02 | API conferma path e preflight input | T-MIG-P4-01 | G-MIG-4 |
| T-MIG-P4-03 | Preparazione run e capture immutabile | T-MIG-P4-01, T-MIG-P4-02 | G-MIG-4 |
| T-MIG-P4-04 | Worker nativo e start LocalOnly | T-MIG-P4-03 | G-MIG-4 |
| T-MIG-P4-05 | Canale progresso e annullamento bounded | T-MIG-P4-04 | G-MIG-4 |
| T-MIG-P4-06 | Gestione esito, report handle e cleanup | T-MIG-P4-04, T-MIG-P4-05 | G-MIG-4 |
| T-MIG-P4-07 | Riutilizzo indice JSONL, senza bulk transfer | T-MIG-P4-06 | G-MIG-4 |
| T-MIG-P4-08 | Query filtrate e pagina risultati in Rust | T-MIG-P4-07 | G-MIG-4 |
| T-MIG-P4-09 | Dettagli, field guide e reference snapshot | T-MIG-P4-08 | G-MIG-4 |
| T-MIG-P4-10 | Export, About e test bridge integrati | T-MIG-P4-06, T-MIG-P4-08, T-MIG-P4-09, T-MIG-P4-12 | G-MIG-4 |
| T-MIG-P4-11 | Rust adapter Full multi-tracciato su ValidationService esistente | T-MIG-P4-06, T-MIG-P2-11 | G-MIG-4 |
| T-MIG-P4-12 | Progresso, cancel e cleanup affidabili per Full | T-MIG-P4-11, T-MIG-P4-05 | G-MIG-4 |
| T-MIG-P5-01 | Shell React responsive e tema | T-MIG-P4-10 | G-MIG-5 |
| T-MIG-P5-02 | Flow/schema/trace state machine UI | T-MIG-P5-01 | G-MIG-5 |
| T-MIG-P5-03 | Conferma file e picker sicuro | T-MIG-P5-02 | G-MIG-5 |
| T-MIG-P5-04 | Selezione formati di report | T-MIG-P5-02 | G-MIG-5 |
| T-MIG-P5-05 | Run screen e progress throttled | T-MIG-P5-03, T-MIG-P5-04 | G-MIG-5 |
| T-MIG-P5-06 | Cancel, stato terminale e old report | T-MIG-P5-05 | G-MIG-5 |
| T-MIG-P5-07 | Informazioni/provenienza e licenze | T-MIG-P5-01, T-MIG-P5-06 | G-MIG-5 |
| T-MIG-P5-08 | Gate parità configurazione/worker/UX | T-MIG-P5-01, T-MIG-P5-02, T-MIG-P5-03, T-MIG-P5-04, T-MIG-P5-05, T-MIG-P5-06, T-MIG-P5-07, T-MIG-P5-10 | G-MIG-5 |
| T-MIG-P5-09 | React: selezione Full con input esplicito per tracciato | T-MIG-P5-03, T-MIG-P4-12 | G-MIG-5 |
| T-MIG-P5-10 | React: monitor Full e stati terminali multi-pass | T-MIG-P5-09, T-MIG-P5-06 | G-MIG-5 |
| T-MIG-P6-01 | DataGrid DOM virtualizzata paginata | T-MIG-P5-08 | G-MIG-6 |
| T-MIG-P6-02 | Cinque filtri paritari e query stale safe | T-MIG-P6-01 | G-MIG-6 |
| T-MIG-P6-03 | Dettaglio diagnostico e selezione offset | T-MIG-P6-01, T-MIG-P6-02 | G-MIG-6 |
| T-MIG-P6-04 | Field guide e posizione/schema | T-MIG-P6-03 | G-MIG-6 |
| T-MIG-P6-05 | Dominio ammesso statico e range | T-MIG-P6-04 | G-MIG-6 |
| T-MIG-P6-06 | Dominio reference/evidenza provider | T-MIG-P6-05 | G-MIG-6 |
| T-MIG-P6-07 | Stati di risultati, malformed e partial | T-MIG-P6-01, T-MIG-P6-03, T-MIG-P6-06 | G-MIG-6 |
| T-MIG-P6-08 | Export UI con receipt e overwrite | T-MIG-P6-07 | G-MIG-6 |
| T-MIG-P6-09 | Verifica numerica e golden report | T-MIG-P6-02, T-MIG-P6-03, T-MIG-P6-04, T-MIG-P6-05, T-MIG-P6-06, T-MIG-P6-07, T-MIG-P6-08 | G-MIG-6 |
| T-MIG-P6-10 | React Full: risultati cross-trace, detail ed export canonico | T-MIG-P6-09, T-MIG-P5-10 | G-MIG-6 |
| T-MIG-P7-01 | Negative tests IPC e payload fuzz | T-MIG-P6-09 | G-MIG-7 |
| T-MIG-P7-02 | XSS, content injection e CSP offline | T-MIG-P7-01 | G-MIG-7 |
| T-MIG-P7-03 | Privacy DOM/log/temp/cache e crash recovery | T-MIG-P7-01, T-MIG-P7-02 | G-MIG-7 |
| T-MIG-P7-04 | Benchmark 300k/1M e profiling end-to-end | T-MIG-P7-01 | G-MIG-7 |
| T-MIG-P7-05 | Race/run lifecycle/cleanup/responsiveness | T-MIG-P7-01 | G-MIG-7 |
| T-MIG-P7-06 | A11y, keyboard, DPI e localizzazione | T-MIG-P7-02 | G-MIG-7 |
| T-MIG-P7-07 | Test regressione core/CLI/PERSIST | T-MIG-P7-03, T-MIG-P7-04, T-MIG-P7-05 | G-MIG-7 |
| T-MIG-P7-08 | Gate sicurezza/performance e release candidate | T-MIG-P7-01, T-MIG-P7-02, T-MIG-P7-03, T-MIG-P7-04, T-MIG-P7-05, T-MIG-P7-06, T-MIG-P7-07, T-MIG-P7-09 | G-MIG-7 |
| T-MIG-P7-09 | Qualifica Full end-to-end Linux/Windows con fixture sintetiche | T-MIG-P7-03, T-MIG-P7-04, T-MIG-P7-05, T-MIG-P6-10 | G-MIG-7 |
| T-MIG-P8-01 | CI dual Rust + TypeScript senza abbassare gate | T-MIG-P7-08 | G-MIG-8 |
| T-MIG-P8-02 | Audit licenze/SBOM/lock/asset | T-MIG-P8-01 | G-MIG-8 |
| T-MIG-P8-03 | Verifier artefatti UI Linux distinto da CLI | T-MIG-P8-01, T-MIG-P8-02 | G-MIG-8 |
| T-MIG-P8-04 | Windows NSIS offline e installazione WebView2 | T-MIG-P8-01, T-MIG-P8-02 | G-MIG-8 |
| T-MIG-P8-05 | UI CachyOS: build, smoke e runtime locale | T-MIG-P8-01, T-MIG-P8-03 | G-MIG-8 |
| T-MIG-P8-06 | Immutabilità, checksum e provenienza CI | T-MIG-P8-03, T-MIG-P8-04, T-MIG-P8-05 | G-MIG-8 |
| T-MIG-P8-07 | Smoke multi-OS estrazione e casi negativi | T-MIG-P8-04, T-MIG-P8-05, T-MIG-P8-06 | G-MIG-8 |
| T-MIG-P8-08 | Gate CI/release/artifact e policy CLI intatta | T-MIG-P8-01, T-MIG-P8-02, T-MIG-P8-03, T-MIG-P8-04, T-MIG-P8-05, T-MIG-P8-06, T-MIG-P8-07 | G-MIG-8 |
| T-MIG-P9-01 | Scorecard parità firmata | T-MIG-P8-08 | G-MIG-9 |
| T-MIG-P9-02 | Piano rollback e coesistenza verificato | T-MIG-P9-01 | G-MIG-9 |
| T-MIG-P9-03 | Cutover release controllato UI | T-MIG-P9-01, T-MIG-P9-02, T-MIG-P9-08 | G-MIG-9 |
| T-MIG-P9-04 | Rimozione Slint in fase separata | T-MIG-P9-03 | G-MIG-9 |
| T-MIG-P9-05 | Aggiornamento ARCH/DET/PLAN/ADR e status | T-MIG-P9-03, T-MIG-P9-04 | G-MIG-9 |
| T-MIG-P9-06 | Release finale e prove cross-OS post cleanup | T-MIG-P9-04, T-MIG-P9-05 | G-MIG-9 |
| T-MIG-P9-07 | Gate finale e handoff maintenance | T-MIG-P9-01, T-MIG-P9-02, T-MIG-P9-03, T-MIG-P9-04, T-MIG-P9-05, T-MIG-P9-06, T-MIG-P9-08 | G-MIG-9 |
| T-MIG-P9-08 | Signoff autonomo Full e controllo visibilità capability | T-MIG-P9-01, T-MIG-P7-09 | G-MIG-9 |

## 11. Stati, evidence e anti-bypass

- Stato distinto `UNIFIED_IMPLEMENTATION_STATUS.md`: `NOT_STARTED`, `IN_PROGRESS`, `WAITING_GATE`, `BLOCKED`, `DONE_UNVERIFIED`, `DONE`. `DONE` solo con precondizioni, test pertinenti e CI sullo SHA reale; no PASS dedotto dall'audit storico.
- Ogni task produce `docs/verification/unified-post-m9/<ID>.md`, lista file/delta, test positivi/negativi, hash/commit, CI (se richiesta), stato, blocker circoscritto e prossima READY.
- L'autore ha già ratificato D01–D15: **non richiedere una nuova ratifica generale**. Richiedere soltanto decisioni veramente non specificate che impattino norma, incompatibilità persistente o sicurezza; proporre alternative circoscritte senza bloccare task indipendenti.
- Stop su bug P0/P1 non risolti pertinenti alla release, errori di correctness, leak sensibili, violazioni di isolamento account, commit non atomico, regressione JSONL/golden, power-loss test software fallito, profilo cross-run non coerente, downgrade/silent data loss o falso pass nativo.
- Test e prove con **dati sintetici**; nessun upload cloud/CDN, telemetria, auto-update o dati grezzi nel renderer. Unità di commit e report separati: nessuna finta transazione globale JSONL+SQLite.
- Se nell'attività D11 compare una reale ambiguità su rettifiche/reinvii, non inventare eccezioni e non produrre false conformità; segnalarla per scelta umana motivata, continuando l'implementazione della casistica certa.
- Non cambiare policy N-19/N-20/CLI/core per portare Tauri su sistemi non approvati. Nessuna release Linux GUI ufficiale. Nessuna modifica al CSS/UI Slint legacy oltre quanto necessario a preservare coesistenza e rollback.
- **Criterio conclusivo**: `G-REL` può essere dichiarato qualificato solo dopo installer Windows11 realmente provato, UI LocalOnly+Full+CAD storico, enforcement SIOC cross-run realmente collaudato, backup/restore e fault-injection positivi, export con conferma shared, CI nativa e report capacità/onboarding per colleghi. `G-REAL` resta distinto per il trattamento di dati lavorativi personali quando applicabile. Un gate non eseguibile non diventa pass: annotare l'eventuale limite senza ingannare l'utente.
