**OVERRIDE RATIFICATO 10/11/2026 — PREVALE SU QUALSIASI DESCRIZIONE STALE IN QUESTA SCHEDA**

Le scelte D01–D15 sono APPROVATE e non vanno riproposte. Usa `MASTER_PLAN_UNIFICATO.md` v1.0 §1/§6 e `PROMPT_ZERO_AGENT.md` per i delta. M9 originario = 140/140 DONE; preflight 30/10 è storico. Software interno reale per colleghi; dati/test CI sintetici; nessun nuovo cloud/crypto/retention. SIOC cross-run da collaudare, non audit-only. Export shared consentito dopo avviso/conferma. Release utenti solo dopo gate aggregato G-REL.

**Nota specifica per T-MIG-P0-04:** Preservare il contratto originario e applicare solo le variazioni esplicitate dal Master Plan unificato; G-MIG non equivale automaticamente alla release interna finale.

---

# T-MIG-P0-04 — Perimetro LocalOnly+Full, parità e capability future

**Percorso:** MIG-P0: Governance e baseline  
**Fonte canonica:** `FONTI/MIGRATION_PLAN_V1_1_2026-10-30.md`, riga task T-MIG-P0-04; se `TASK_PROMPTS/T-MIG-P0-04.md` originale è nel kit preesistente, leggilo e preserva i vincoli più specifici.  
**Prerequisiti:** T-MIG-P0-03  
**Gate fase:** G-MIG-0: ADR recepite, N-24/OS/eccezioni UI-only, baseline post-audit Git/CI, scope Full; non iniziare implementazione Tauri prima del gate.

## Mandato atomico
Esegui esclusivamente **Perimetro LocalOnly+Full, parità e capability future** come task della migrazione React + TypeScript strict + Tauri 2, adattando le vecchie evidenze del 30 ottobre alla baseline post-M9 del 10 novembre. Prima di agire rileggi il codice e controlla che lo stato registrato non sia ormai superato. Non dedurre che T-PERSIST-08 sia WIP: è stato concluso, come CAD12 e P18-04.

## Accettazione obbligatoria della task
Matrice approvata LocalOnly + Full (obbligatoria) e capability ancora non qualificate; niente Full fittizio né UI approval.

## Procedura
1. Preflight dei prerequisiti sul repository corrente e identificazione dei file/contratti interessati.
2. Ricostruzione dei test/regressioni esistenti e verifica prima della modifica.
3. Delta minimo limitato a questa task; l'eventuale decisione non ancora ratificata produce BLOCKED, non comportamento arbitrario.
4. Prove locali negative e positive di funzionalità e non regressione proporzionate al rischio, con evidenze SHA.
5. Gate specifico della fase, commit e CI solo se necessario e autorizzato; non attribuire alla task un gate non ancora eseguito.
6. Dossier `docs/verification/unified-post-m9/T-MIG-P0-04.md` con cosa è cambiato, cosa NON è cambiato, test, rischi residui e prossima task READY.

## Vincoli invarianti
- Single-agent sequenziale; leggere AGENTS.md, ratifiche, schema/ADR e stato sul commit effettivo. Non creare subagenti.
- Nessuna regola clinica/DSL in React; Rust mantiene parsing, validation, projection, reference, spool, query/paging, report e SQLite.
- CLI, golden JSONL byte-identici, namespace, Unknown/Partial/Cancelled/technical error, CAS e immutabilità non si indeboliscono.
- Usa esclusivamente dati sintetici; niente PHI in console, storage DOM persistente, telemetria, CDN, update automatico o comunicazioni web.
- Le decisioni di prodotto MIG e D01–D15 sono già ratificate; il Decision Pack non è pendente. Recepire la ratifica repository del 10 novembre; chiedere soltanto decisioni sostanziali non coperte.
- Non dichiarare DONE senza risultati locali e nativi pertinenti sullo SHA giusto; se un gate è indisponibile usa BLOCKED/DONE_UNVERIFIED.

