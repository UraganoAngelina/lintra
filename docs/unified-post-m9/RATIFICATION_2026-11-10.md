# ADR-POST-M9-001 — Recepimento della ratifica D01–D15

Autore della decisione: autore del progetto, messaggio utente del **10 novembre
2026, Europe/Rome**, titolo «PROMPT ZERO AGENTE — LINTRA POST-M9 UNIFICATO ·
RATIFICA 10/11/2026». Recepimento eseguito da Codex in U-GOV-03; nessuna firma
digitale, identità legale o autorizzazione aziendale aggiuntiva viene attestata.
Il client non fornisce un ID permanente del messaggio: non ne viene inventato uno.

Stato: **USER_APPROVED / REPOSITORY_RATIFIED / GATES_NOT_VERIFIED**.
Le quindici decisioni integrali sono recepite verbatim nella tabella §1 del
[master v1.0](MASTER_PLAN_UNIFICATO.md), con digest e provenienza nel
[manifest](SOURCE_MANIFEST.json). Il [mandato operativo](PROMPT_ZERO_AGENT.md)
ne conserva i vincoli e la sequenza. Il Decision Pack importato è una fonte
storica: la sua dicitura «recepimento pendente» descrive il momento di consegna,
superato da questo atto. Non è una richiesta di approvazione ancora aperta.

## Effetti approvati

| ID | Effetto operativo recepito | Task / prove richieste |
|---|---|---|
| D01 | LocalOnly, Full e CAD storico nella roadmap; nessun bypass library | 89 MIG, U-UI, U-CAD; G-MIG-9 + G-CADUI3 |
| D02 | Software usabile internamente da autore/colleghi autorizzati; test sintetici; dati personali soltanto con autorizzazioni aziendali effettive | U-OPS-01/09; G-REAL circoscritto |
| D03 | Filesystem locale per account, ACL/ownership OS, temporanei privati; nessun server, nuova cifratura o password custom | U-OPS-02/03 |
| D04 | Export locale su path scrivibile; shared/UNC/remoto con avviso e consenso, poi salva; overwrite e receipt preservati | U-OPS-04, MIG export |
| D05 | Nessuna retention, TTL o purge automatici; gestione operatore/azienda | U-OPS-07 |
| D06 | Chiave comune esterna 32 byte e key ID invariati; recupero documentato, nessuna rotazione automatica | U-OPS-05 |
| D07 | BACKUP-B consistente e verificato prima di upgrade irreversibili; stop se invalido; restore isolato | U-OPS-06/08 |
| D08 | JSONL/CSV canonici byte-fedeli, XLSX celle testo; nessuna sanificazione CSV silenziosa | U-FIX-02 |
| D09 | Budget soft misurati con «Procedi comunque» legato alla richiesta; hard non aggirabili; EOF/hash/audit/cancel invariati | U-FIX-03, MIG-P5-05 |
| D10 | Reference tree locale controllata, permessi/link verificati; errore preciso se non attendibile | U-FIX-04 |
| D11 | PK SIOC cross-run non ripetibili, nuova versione e collaudo; snapshot conservati distinti da nuove trasmissioni | DEC-SIOC-HIST-02, U-NORM-01/02/05 → 03/04 |
| D12 | Dataset ricevuti definitivi; controlli progettati e deferred trasparenti; nuovo normativo solo D11 o bug provati | U-NORM-01/02 |
| D13 | Misure realistiche; DEC-17 D sequenziale; nessuna soglia DEC-14 o parallelismo inventati | U-PERF-01/02 |
| D14 | Windows11 x86_64 NSIS offline/perMachine, WebView2 offline, run non-admin, due account, update manuale, uninstall conservativo; Linux personale CachyOS | MIG-P1/P8/P9, U-REL-01/02 |
| D15 | Hardening di atomicità/durabilità/recovery con kill/fault/reopen; nessuna garanzia fisica universale | U-REL-03, G-DUR |

## Delta sui contratti

La scelta React + TypeScript strict + Vite + Tailwind/shadcn + TanStack + Tauri 2
è approvata. N-24/ARCH §39D/DET Addendum B mantengono Slint come prodotto legacy
fino a parità e rollback, ma non vietano più il nuovo adapter dopo G-MIG-0.
Le eccezioni WebView/installer sono GUI-only: N-19/N-20/N-23 e la policy CLI
rimangono applicabili al core/CLI. Niente rete, CDN, telemetria o updater nel
prodotto. Rust possiede job, dati, indici, query, validation e storage; IPC typed
e bounded; una validazione attiva per istanza. Nessuna normativa in JavaScript.
Gli ADR MIG specifici e i contratti tecnici si completano nelle task P0/P2.

[DEC-SIOC-HIST-02](../normative/DEC-SIOC-HIST-02.md) supera prospetticamente la
policy audit-only nei nuovi profili; non modifica i vecchi schemi o golden.
HMAC1, codec legacy, Unknown/Partial, CAS, phase-major, max_errors e cancellazione
restano vincolanti. Il consuntivo M9 rimane 140/140 DONE.

## Gate e distribuzione

Nessun gate è PASS per la sola ratifica. G-REL richiede G-MIG-9, G-CADUI3,
G-SIOC1, G-OPS2, G-DUR, installer Windows11 effettivamente provato e assenza
di P0/P1 pertinenti. Il cutover MM9 non autorizza già la distribuzione.
Nessun tag/release/pubblicazione creato da questo atto; resta il signoff previsto.
G-REAL riguarda solo gli eventuali dati lavorativi personali e non ritarda
GUI, fixture sintetiche e package tecnico. Nessun programma enterprise aggiunto.

Esecuzione: un agente, una task per volta, nessuna delega/background. Stato
separato in UNIFIED_IMPLEMENTATION_STATUS; dossier per ogni ID. Continuare le
task indipendenti READY se una prova nativa o un caso normativo resta bloccato.
