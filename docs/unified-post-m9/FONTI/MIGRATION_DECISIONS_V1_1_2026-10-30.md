# Lintra — Registro DEC/CNF/RSK consolidato v1.1

**Data:** 2026-10-30 — Decisioni di prodotto approvate dall'autore nella conversazione, **non** prove di implementazione, approvazione dei singoli gate o ADR già committate nella repo. Non modificare automaticamente gli stati di `IMPLEMENTATION_STATUS.md`. Per elenco delle fonti e criteri di ratifica vedere `MIGRATION_REQUIREMENTS_TRACEABILITY.md`.

## Decisioni

| ID | Decisione approvata / boundary | Stato prodotto | Ratifica in repository | Gate |
|---|---|---|---|---|
| DEC-MIG-01 | React+TS+Tauri2 sostituisce Slint con coesistenza e rollback fino al cutover; core/CLI immutati | USER_APPROVED | ADR-MIG-001 da recepire in ARCH/DET | 0/9 |
| DEC-MIG-02 | GUI Windows **solo Win11 x86_64**, NSIS `offlineInstaller`, `perMachine`; GUI Linux **solo CachyOS personale** build locale; CLI e core N-19/20/23 invariati | USER_APPROVED | ADR-MIG-002 da recepire | 0/1/8 |
| DEC-MIG-03 | Local-only/offline: no rete, CDN, telemetry, auto-update; backend possiede PHI, no storage web persistente di dati sanitari | USER_APPROVED | ADR-MIG-003 e threat model | 0/2/7 |
| DEC-MIG-04 | IPC tipizzato, Channel bounded, Rust Job Manager; **una** validazione simultanea per istanza; filtri/paging Rust; ownership e cleanup backend | USER_APPROVED | Contratto dettagliato a G2 (nessuna firma tecnica implicitamente approvata) | 0/2/4 |
| DEC-MIG-05 | Redesign moderno e responsive **durante** la migrazione; parità di comportamenti, non pixel della Slint legacy | USER_APPROVED | ADR-MIG-001 e UX acceptance | 0/5/7 |
| DEC-MIG-06 | Baseline unica dopo PERSIST-08 o suo isolamento; CI completa e SHA identificata | **OPEN_TECHNICAL** | dossier Git + CI, non decisione a intuito | 0 |
| DEC-MIG-07 | Migrazione 1:1 + **UI Full multi-tracciato obbligatoria**, e altre capacità core Rust solo se qualificate; CAD storico e approval non disponibili senza gate separati | USER_APPROVED | ADR-MIG-001 + traccia task Full | 0/2/4/5/6/7/9 |
| DEC-MIG-08 | React, TypeScript strict, Vite, Tailwind CSS, shadcn/ui, TanStack Table/Virtual; versioni/license lock da verificare, DataGrid stress test | USER_APPROVED | ADR-MIG-003 + lock/supply chain | 2/3/7 |
| DEC-MIG-09 | Migrazione e test **solo con fixture sintetiche**; qualificazione per dati sanitari reali separata, non sbloccata da cutover UI | USER_APPROVED | ADR-MIG-004 + gate real data distinto | 0/7/9 |

## Sotto-decisioni DEC-MIG-02 (tutte USER_APPROVED)
- Windows installer: NSIS `.exe` completamente offline, MSVC CI, `webviewInstallMode=offlineInstaller`, `nsis.installMode=perMachine` (**Program Files**, elevazione all'installazione, esecuzione ordinaria non elevata, multi-account).
- Windows GUI baseline: **Windows 11 x86_64 soltanto**; Windows 10/Server fuori supporto GUI, **nessuna modifica alla baseline CLI**.
- Update: **manuale e offline**, installer nuova versione; nessun updater/network service.
- Uninstall: rimuove l'app, non distrugge database, snapshot/config/report persistenti; wipe solo operazione distinta ed esplicita.
- Account: dati default per-account in `%LOCALAPPDATA%\Lintra`; condivisione futura solo predisposta in design, **non abilitata**. Program Files non contiene archivi mutabili.
- Linux: build/smoke personali sul PC CachyOS + Hyprland/Wayland + Caelestia; non produrre release Linux GUI ufficiale né promettere AppImage/.deb universalmente portable; non estendere per analogia supporto ad altri OS.

## Sotto-decisione persistenza — BACKUP-B
- Backup manuali consistenti e verificati, ripristino controllato; **prima di qualunque upgrade SQLite potenzialmente irreversibile** backup obbligatorio verificato, altrimenti blocco dell'upgrade; restore con compatibilità, integrità, prevenzione overwrite e dependency HMAC/keys.
- **Non è task implicita di questa migrazione UI**: da inserire nel workstream PERSIST/real-data tramite change request e test propri. Non dichiarare che il software già offre backup.

## Conflitti CNF-MIG — stato documentale e gates
| ID | Origine | Risoluzione approvata/proposta | Stato residuo |
|---|---|---|---|
| CNF-MIG-01 | N-24 prescrive Slint | ADR-MIG-001 sostitutivo per UI React/Tauri, Slint conservato sino a cutover | Product resolved, repository ADR/ARCH/DET pending |
| CNF-MIG-02 | N-19/20/23, QG5 vs WebView e distro | Eccezione **GUI-only**: Windows11 NSIS offline, CachyOS personale; CLI/core CI/release contratti invariati | Product resolved; native proof pending |
| CNF-MIG-03 | QG7 pixel RGB Slint vs WebView | Nuovo visual acceptance cross-OS e UX responsive; screenshot per-OS, test semantici/a11y | Policy resolved; budget/prove pending |
| CNF-MIG-04 | Documenti storici contro implementazione SQLite/status | Reconcile stato e dossier senza riscrivere passato; traccia evidence basata su SHA | Technical/documental open |
| CNF-MIG-05 | Clap nel package condiviso e build.rs legge Git vs frasi N02 | Documentare package-vs-module e provenance build, non fare refactor non richiesto | Technical/documental open |
| CNF-MIG-06 | WIP PERSIST-08 drift durante audit | Baseline post-WIP8 o isolamento esplicito + full CI SHA | **Blocking G0** |

## Rischi / mitigazioni (continuano gli ID precedenti)
| ID | Rischio | Mitigazione / evidenza |
|---|---|---|
| RSK-MIG-01 | 300k/1M anomalie → RAM e filtri scan O(n) | Backend Rust query/page + TanStack virtual, p95/RSS reale |
| RSK-MIG-02 | Late events, incorrect completeness/outcome | Job Manager state machine, runId/reportId, terminal reliable |
| RSK-MIG-03 | PHI in IPC/DOM/log/cache / XSS | local-only, CSP, allowlist custom, redaction, no persistent web caches |
| RSK-MIG-04 | Doppia pubblicazione export+summary | Receipt parziale esplicita, niente transazione globale inventata |
| RSK-MIG-05 | CI su SHA antenato | CI stessa SHA e build nativa ripetibile |
| RSK-MIG-06 | SIOC/CAD normativi incompleti | Fixture sintetiche + capability disclosure, nessuna certificazione implicita |
| RSK-MIG-07 | SQLite contesa/recovery/snapshot O(n) | Regression suite invariata, no nuove approval UI |
| RSK-MIG-08 | PHI temporanei / crash e restore | Protocollo storage/real-data separato; no secure erase presunto |
| RSK-MIG-09 | Installer per-machine potrebbe contaminare utenti / cancellare dati | Windows11 due account, ACL per-user, uninstall/reinstall sentinel e upgrades |
| RSK-MIG-10 | Full UI aumenta surface ma core/profile non completi | Test Full nativi sintetici, contratto g2, scorecard cutover e capability gating |

**Procedura di ratifica:** l'agente può redigere/committare ADR solo in task autorizzata, con owner approvazione e SHA; non può dare `G-MIG-n APPROVED` senza prove e approvazione quando obbligatoria. Le scelte `USER_APPROVED` possono essere recepite fedelmente senza riaprire domande di prodotto; le decisioni tecniche non ancora determinate vanno proposte con opzioni, non inventate.
