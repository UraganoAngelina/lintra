# Riconciliazione corrente della migrazione — 11 novembre2026

T-MIG-P0-02. Baseline di codice qualificata:
`cb70bee19f2d470c3ac2e98828343b32e23b3f0a`,
[CI42990775714](https://github.com/UraganoAngelina/lintra/actions/runs/42990775714),
sette job richiesti success. HEAD di ingresso22a6b55 aggiunge solo documenti:
520 input eseguibili/configurazione identici. Il preflight30 ottobre e la
matrice U-GOV-02 sono fotografie storiche, non nuove task da rieseguire.

| Riferimento / discrepanza | Riscontro attuale | Outcome e destinazione |
| --- | --- | --- |
| HEADfe5ada8 e WIP PERSIST08 | Baselinecb70bee qualificata; PERSIST08 e successive DONE nel consuntivo140/140 | AGGIORNATO: usare T-MIG-P0-01, nessun WIP da isolare |
| 117/140 e M9 pendente | Consuntivo140/140 byte-identico a U-GOV-01 | ACCETTATO: conservare originale, stato post-M9 separato |
| CIDelta budget/reference pendenti | U-FIX-01..05 DONE; G-HARD PASS; ACL Windows3 pass nativi | AGGIORNATO: log sucb70bee, non ereditare CI antenata per codice diverso |
| SIOC audit-only come unica policy | Nuovo cross-run.1/capability formato3 applica D11; profili legacy invariati | AGGIORNATO: audit-only resta storico per vecchi profili |
| PK cross-run sempre vietata su1–3 | Addendum-A01 ammette ricoperture identiche e rifiuta mismatch; carenza di prova Unknown | AGGIORNATO: sorgente normativa ratificata e oracle nativi |
| PK UDO a soli tre campi | Quattro campi:4.1/4.7/4.8/4.19.1, tutti conservati | AGGIORNATO: intra-invio e fra nuovi invii, scarto e violazione nella seconda passata |
| Decisione SIOC ancora da chiedere | U-NORM-05 DONE, DEC-SIOC-HIST-02-A01 recepita | AGGIORNATO: nessuna nuova domanda ordinaria |
| SIOC UI e monthly Full già utilizzabili | Backend03 DONE;04 WAITING_INTEGRATION per frontend e contesto amministrativo qualificato | ACCETTATO COME RESIDUO: dopo G-MIG-4, niente falso G-SIOC1/G-REL |
| CAD ancora non implementato | Servizio Rust CAD83/C1/L1 e SQLite qualificati | ACCETTATO: riuso backend, CAD UI aggiuntiva U-CAD-01..09 ancora da realizzare |
| Slint offre Full/storico | analysis.rs prepara Trace/LocalOnly | ACCETTATO: parità Slint più Full obbligatorio nella nuova UI; nessuna schermata simulata |
| Stack UI ancora proposta | D01–D15 e DEC-MIG già approvate dall'autore | ACCETTATO: formalizzare ADR inP0-03, senza nuova ratifica globale |
| N-24 continua a imporre esclusivamente Slint | Override autore autorizza React/TS strict/Vite/Tailwind/shadcn/TanStack/Tauri2 | DA AGGIORNARE: ADR/ARCH/DET mirati inP0-03 prima di G-MIG-0 |
| Core riceve dipendenze GUI | Workspace attuale root+ui; Rust/CLI distinti dagli adapter | ACCETTATO: eccezioni GUI solo adapter, lock e boundary da qualificare |
| Root per-account già collegata | Backend private root/key verificato; adapter Tauri assente | ACCETTATO COME RESIDUO: U-OPS-03 e MIG-P2-08/P7-03, nessun DONE di provisioning |
| Cifratura/retention/password custom da decidere | File locali/ACL/HMAC/chiave comune e nessun TTL già ratificati | AGGIORNATO: U-OPS-05/07, backup/restore ancora da qualificare |
| Shared export da bloccare | D07 locale libero; shared/UNC/remoto con warning/consenso | ACCETTATO: U-OPS-04 e MIG export, niente denylist generale |
| Budget software hard aggirabile o taglio record | Soft warning/Procedi comunque; hard safety non aggirabile | ACCETTATO: backend pronto, UI integra inP5-05 senza hash/record troncati |
| DEC-14/parallelismo automatico | DEC-17 finale: produzione sequenziale | ACCETTATO: volumi realistici decisionali;300k/1M stress, niente soglie inventate |
| Linux GUI ufficiale / Windows generico | GUI Win11 x86_64 NSIS offline/perMachine; CachyOS personale | DA AGGIORNARE: ADR OS inP0-05; core/CLI mantengono policy originale |
| CIWindows2025 prova installer Win11 | Prova nativa backend reale; nessuna clean-room/WebView2/installer qualificata | ACCETTATO COME RESIDUO: P1/P8/P9 con evidenze native specifiche |
| ADR dati reali/enterprise blocca GUI | Solo autorizzazioni aziendali applicabili ai dati specifici; fixture sempre sintetiche | AGGIORNATO: G-REAL distinto, nessuna infrastruttura enterprise imposta |
| MM9 equivale a distribuzione | Release interna richiede MIG9+CADUI3+SIOC1+OPS2+DUR e signoff previsto | ACCETTATO: niente tag/release o distribuzione implicita |

Non emerge un blocco di decisione **perP0-02**. Le righe DA AGGIORNARE sono
instradate alle taskP0 autorizzate; i residui tecnici futuri conservano i propri
gate e non diventano PASS per questa riconciliazione. Le89 task e i39ID U-* non
cambiano; nessun arco originale rimosso. La GUI resta da implementare dopo
G-MIG-0. Le fonti FONTI/kit ricevute non vengono riscritte.

Fonti correnti: [ratifica](RATIFICATION_2026-11-10.md),
[master](MASTER_PLAN_UNIFICATO.md),
[addendum SIOC](../normative/DEC-SIOC-HIST-02-A01.md),
[baseline qualificata](../verification/unified-post-m9/T-MIG-P0-01.md),
[stato autorevole](../../UNIFIED_IMPLEMENTATION_STATUS.md).
