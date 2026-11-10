# LINTRA — RATIFICA UNICA D01–D15 · 10 novembre 2026

**STATO: APPROVATA DALL'AUTORE IN CONVERSAZIONE; RECEPIMENTO IN REPOSITORY PENDENTE.**

Questo documento **sostituisce** la proposta Decision Pack con default A del precedente kit. Le scelte non vanno richieste di nuovo. Fonte e semantica vincolante: `MASTER_PLAN_UNIFICATO.md` v1.0 §1. Alcune alternative A originali sono state **modificate o respinte**, in particolare D02–05, D09, D11, D12, D14, D15.

| ID | Ratifica sintetica |
|---|---|
| D01 | A: LocalOnly + Full + CAD storico nella stessa roadmap, gate separati e rilascio interno solo dopo qualifica. |
| D02 | Uso **interno reale** da autore e colleghi autorizzati, non clinico o vendita; test sintetici; dati lavorativi secondo autorizzazioni aziendali. |
| D03 | File locali per-account, OS ACL, nessun nuovo cipher/password custom/retention. |
| D04 | Export consentiti su path locale accessibile; condivisi con avviso e conferma, poi salvataggio. |
| D05 | Nessun purge o durata automatica; gestione operatore/azienda. |
| D06 | A: chiave comune invariata, recovery, no rotazione automatica. |
| D07 | A: BACKUP-B manuale coerente, verificato prima degli upgrade irreversibili e restore isolato. |
| D08 | A: CSV canonico invariato, XLSX per spreadsheet. |
| D09 | A estesa: superamento budget **soft** avvisa e propone «Procedi comunque»; **hard** non aggirabili. |
| D10 | A: reference tree controllata, no garanzia TOCTOU ostile inventata. |
| D11 | **A RESPINTA**: la non ripetizione PK SIOC **cross-run è vincolo normativo da implementare e collaudare**, superando audit-only prospetticamente. |
| D12 | Dataset ricevuti definitivi; controlli progettati invariati, niente sourcing o completezza inventata. |
| D13 | A: profiling realistico, nessun parallelismo automatico. |
| D14 | Applicazione **distribuibile ai colleghi** con installer Windows offline e flussi realmente usabili; non solo technical preview. |
| D15 | Hardening power-loss, fault-injection, recovery e trasparenza su limite hardware non provato. |

**Decisioni già approvate non riaperte:** DEC-MIG-01/02/03/04/05/07/08/09, BACKUP-B, C1/L1, phase-major, DEC-17 D, HMAC e storage versionato. `DEC-MIG-06` è solo verifica SHA/CI. La direttiva D11 va tracciata con decisione successiva a `DEC-SIOC-HIST-01` senza modificare la storia; la fonte normativa va verificata nel repository durante U-NORM-05 e le ambiguità di rettifica trattate esplicitamente.

**Limiti:** la ratifica tecnica non concede automaticamente permessi aziendali di trattare dati di pazienti o altre informazioni personali. Non inventare un programma enterprise. Nessuna interfaccia, schema, formato o data pubblicazione sono dichiarati qualificati solo per la ratifica.
