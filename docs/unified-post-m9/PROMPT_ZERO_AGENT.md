# PROMPT ZERO AGENTE — LINTRA POST-M9 UNIFICATO · RATIFICA 10/11/2026
## Da inviare a Codex per avviare l'esecuzione sequenziale delle task

Sei **l'unico agente operativo** di Lintra per il workstream **hardening post-M9 + SIOC cross-run + React/Tauri 2 + CAD storico + distribuzione interna**. L'autore ha **già espresso tutte le scelte D01–D15 il 10 novembre 2026**. **Non** chiedere un'altra ratifica globale, non trattare il vecchio `DECISION_PACK_UNICO.md` come proposta ancora pendente: le parti discordanti sono superate dal nuovo `MASTER_PLAN_UNIFICATO.md` v1.0. La ratifica in chat va **recepita formalmente in repository** durante `U-GOV-03` con fonte, data e tracciabilità; nessun gate è automaticamente PASS.

## 1. Leggi fonti e risolvi baseline

Ordine delle letture:
1. `MASTER_PLAN_UNIFICATO.md` v1.0 (questa ratifica e i 17 gap con 39 task U-*); `PROMPT_ZERO_AGENT.md`.
2. `FONTI/AUDIT_POST_M9.md`, `FONTI/GAP_REGISTER_POST_M9.md`, `FONTI/ROADMAP_POST_M9.md`.
3. `FONTI/PREFLIGHT_SLINT_TAURI2_2026-10-30.md` **come fotografia storica**, NON come HEAD corrente.
4. `FONTI/MIGRATION_PLAN_V1_1_2026-10-30.md`, `FONTI/MIGRATION_DECISIONS_V1_1_2026-10-30.md`, prompt canonici MIG se presenti: mantenere gli **89 ID** e sovrapporre i delta ratificati.
5. `AGENTS.md`, `MASTER_AGENT_PROMPT.md`, ARCH/DET/ADR/normativa, `IMPLEMENTATION_STATUS.md` e test **effettivi** del repository.

Primo passo **U-GOV-01**: `git status`, branch/HEAD, conferma inclusione commit audit `af3bcccb6246a7ff00463c595f4c780a9f5c5195`, CI `42789338814`, fonti/sorgenti e working tree. Non modificare checkout adiacenti o file dell'autore. Poi `U-GOV-02` e `U-GOV-03` per recepire la presente ratifica e correggere i prompt atomici legacy discordanti; `U-GOV-04` aggiorna il dependency graph. Il vecchio M9 resta **140/140 DONE**.

## 2. Decisioni vincolanti già prese (NON ripresentare)

- **Utenti:** solo autore e colleghi aziendali autorizzati; **software realmente pronto all'uso interno**, non demo. Non serve infrastruttura enterprise o certificazione clinica. **Non presumere però autorizzazioni aziendali su dati personali non documentate**: usare sempre sintetici in test/CI; solo dati effettivi per cui l'organizzazione consente il trattamento.
- **Dati:** filesystem locale per account (Windows `%LOCALAPPDATA%/Lintra`), senza server, cloud, password custom, nuova cifratura applicativa o purge/retention automatica. Preservare ACL/ownership OS, HMAC, key bundle e file temporanei privati; password/cartella protetta OS-native solo facoltativa.
- **Export:** sempre possibile se il path è scrivibile, locale senza nuova conferma di privacy; shared/UNC/mount remoti riconosciuti con avviso e consenso dell'operatore, poi salvare; mantenere conferma overwrite, receipt e JSONL/CSV/XLSX canonici. Non introdurre denylist generale delle shared folder.
- **SIOC D11:** **PK cross-run non ripetibili** è controllo normativo approvato da portare a **collaudo end-to-end**. La vecchia `DEC-SIOC-HIST-01` audit-only è superseded **prospetticamente** dalla nuova decisione versionata; non inventare eccezioni per rettifiche/reinvii, distinguere identità conservate negli snapshot da nuove trasmissioni, isolare namespace. Se manca prova sulla casistica contestata, blocca solo quel punto e richiedi una decisione mirata; **non tornare audit-only per default**.
- **SIOC D12:** dataset attualmente disponibili sono definitivi. Nessun nuovo sourcing e nessuna norma inventata; preservare perimetro progettato e gli eventuali deferred dichiarati. Implementazione nuova soltanto per D11 o difetti reali.
- **Budget D09:** **soft** preflight → avvisa e consenti `Procedi comunque`, senza taglio record/hash; i limiti **hard** di correttezza/safety non sono aggirabili, producono errore esplicito.
- **Recovery D15:** rafforzare realmente atomicità/durabilità controllabile con test di kill/fault-injection/reopen e recovery SQLite/report; nessuna promessa fisica universale senza hardware di prova. Backup-B verificato prima di upgrade irreversibile, ripristino isolato, chiave comune invariata.
- **Migrazione:** React+TS strict+Vite+Tailwind+shadcn/TanStack+Tauri 2; Rust Job Manager, IPC typed/bounded; una validazione attiva per istanza. Core/CLI e normative invariati salvo D11. LocalOnly + Full multi-tracciato + **CAD storico** da consegnare nella roadmap; nessuna logica normativa in JS.
- **Distribuzione colleghi D14:** Windows 11 x86_64 NSIS offline/perMachine, WebView2 offline installer, non-admin run, due account, update manuale, uninstall che conserva DB/report; Linux solo build/smoke personale CachyOS. Redesign responsive, Slint mantenuto fino a parità e rollback; no release Linux GUI ufficiale.
- **Performance:** no parallelismo automatico o DEC-14 inventata; benchmark 300k/1M come stress, volumi realistici come misura decisionale.

## 3. Esecuzione atomica e priorità

1. Esegui `U-GOV-01..04` e recepisci ratifica, baseline e rispecificazione dei prompt incoerenti.
2. Proteggi regressioni SQLite; avanza U-FIX e U-OPS mirate **senza bloccare** la migrazione per procedure enterprise non richieste.
3. Per D11 esegui `U-NORM-01/02/05 → U-NORM-03/04` con nuova capability normativa/versionamento, fixture sintetiche e CI. Non usare vecchi `U-NORM-05` che prescrivono audit-only.
4. Esegui `U-UI-01`, poi le **89 task `T-MIG-P0..P9`** rispettando G-MIG-0..9; cambia solo scope e acceptance nei punti definiti dal nuovo master. Mantieni tutte le prove native Windows11 clean-room/WebView2 e CachyOS personale.
5. Dopo `U-UI-02`, realizza `U-CAD-01..09` per namespace, C1/L1, preview/review/approve CAS, T1/T2/T3 e consultazione storico tramite la nuova UI. Non attivare l'approval partendo da CLI audit-only con un flag artificiale.
6. Esegui `U-REL-03` e `G-DUR`: test di interruzione e crash recovery, senza riscrivere il repository SQLite che ha già garanzie.
7. `U-REL-01/02` può chiudere **G-REL** soltanto dopo **G-MIG-9 + G-CADUI3 + G-SIOC1 + G-OPS2 + G-DUR** e installer testato. Distribuisci ai colleghi soltanto artefatto Windows qualificato e con autorizzazione di pubblicazione prevista dal piano; non creare tag o release arbitraria.
8. `G-REAL` è separato e limitato alle autorizzazioni aziendali eventualmente necessarie per specifici dati lavorativi; non è una scusa per rimandare GUI, test sintetici o package interno funzionante.

**Sicurezza e qualità:** non toccare normative non autorizzate, HMAC1/format legacy, writer JSONL, `Unknown`, `Partial`, CAS/commit, phase-major, max_errors, cancellation per ottenere test verdi. Non introdurre dati sanitari reali in test, log o screenshot. Non cambiare golden legacy per nascondere regressioni. Non usare rete, telemetria, updater o CDN nel prodotto. Un solo agente, **una task alla volta**, nessun subagente o programma asincrono/delegato.

## 4. Stato, gate e handoff

Aggiorna `UNIFIED_IMPLEMENTATION_STATUS.md` **separato** dal consuntivo 140/140, con ID, prerequisiti, file modificati, SHA, test locali, CI per OS, gate, blocchi e `next READY`. Ogni task produce dossier `docs/verification/unified-post-m9/<ID>.md`. `DONE` richiede evidenze. Non riutilizzare CI su SHA precedente come prova di un delta nuovo.

Se una task dipende da decisione non coperta (es. eccezioni normative SIOC rettifica/riuso o modifica chiavi/codec), fornisci **una proposta circoscritta** e passa alla successiva task indipendente READY. Non chiedere conferma su D01–D15, stack, OS, file locale, export shared, backup-B, vincolo SIOC o necessità di hardening power-loss.

**Criterio finale:** Lintra Tauri 2 Windows offline per i colleghi, LocalOnly + Full + CAD storico verificati; SIOC PK cross-run normativo in collaudo, UI capace di mostrarne l'esito; backup/restore e recovery testati; export locale e shared con warning; nessun P0/P1 aperto riferito alla distribuzione. Qualità e limiti documentati, senza proclamare certificazione clinica o garanzia totale contro ogni power loss.

**Comincia ora da U-GOV-01. Non fermarti a una nuova specifica senza implementazione, salvo blocker materiale.**
