# GOV-2026-10-26-01 — Revisione della governance operativa

26 ottobre2026. Approvazione esplicita dell'autore: GLOBAL WORKFLOW POLICY
MIGRATION. Questa revisione è operativa, non modifica architettura o norme.

## Policy permanente e priorità

La sezione GLOBAL AGENT EXECUTION POLICY di AGENTS.md governa tutte le task:
un solo agente principale; nessun Superpowers/subagente/delega; commit/push/CI
autorizzati; avanzamento automatico dopo gate e handoff; stop solo su decisioni
normative/architetturali/contrattuali/sicurezza non ratificate o nessuna READY.
I bug ordinari e i problemi tecnici si risolvono autonomamente. Il precedente
checkpoint dopo14-05 è superato; DEC-18 e le altre ratifiche restano valide nel
merito. Nessun gate o criterio di accettazione è rimosso o reso presunto.

Il limite storico di interrogazioni CI non richiede più una conferma per log,
retry motivati o evidenze finali necessarie: valgono autorizzazione permanente,
polling sobrio (obiettivo tre letture ordinarie per run, intervalli≥60s), nessuna
pipeline duplicata e stato reale su guasti di rete. Sicurezza/TLS invariati.

## File operativi e differenze

| File | Revisione |
|---|---|
| AGENTS.md | Policy corrente in testa; cronologia precedente conservata separatamente, ratifiche sostanziali valide |
| MASTER_AGENT_PROMPT.md | Copia operativa versionata del master del kit; protocollo sequenziale/autonomo e fonti operative correnti; bootstrap marcato storico |
| docs/MASTER_IMPLEMENTATION_PLAN.md | Solo §14: single-agent, autorizzazione Git/CI, continuità, blocchi reali e recovery senza consenso ordinario |
| TASK_DEPENDENCY_MAP.md | Sostituita sola regola di stop; grafo/dipendenze e gate invariati |
| ATOMIC_TASKS_INDEX.md e TASK_PROMPTS/ | Versioni operative delle51 task future/parziali; cambiate solo clausole stop/handoff, nessuna riscrittura dei contratti |
| templates/ATOMIC_TASK.md | Ciclo completo con gate e next READY automatica |
| templates/HANDOFF.md | Task conclusa, gate, next READY, blocker e stato AUTO_CONTINUE/WAITING_GATE/BLOCKED/NO_READY_TASK |
| templates/SESSION_RECOVERY.md | Ripresa selettiva, nessuna duplicazione, continuazione immediata su task pronte |
| README.md e IMPLEMENTATION_STATUS.md | Punti d'ingresso e stato della nuova prosecuzione |
| migration-manifest.json e verify_policy.py | Provenienza esatta e verifica di invarianti/diff operativi |

I master originali e i140 prompt erano nel kit adiacente fuori Git. Il kit resta
intatto: non si modifica un archivio verificato per farlo apparire retroattivamente
migrato. I prompt già DONE, docs/bootstrap, vecchi piani Superpowers e dossier/
ratifiche/handoff storici restano invariati. Le loro clausole di workflow non sono
operative, mentre norme/contratti/evidenze continuano a essere consultabili.
I riferimenti legacy SOURCE_DOCUMENTS/<file> nei prompt si risolvono in docs/<file>
come specificato dal master corrente. Nessun file di pazienti è copiato nel repo.

## Verifiche effettive

`python3 docs/governance/verify_policy.py`: PASS,51 prompt. Il controllo legge
il manifest e inverte le sole sostituzioni operative: il digest del risultato
coincide con ciascun prompt originale, quindi obiettivi, prerequisiti, test,
accettazione e vincoli non sono cambiati. Verifica inoltre i digest del piano
prima/dopo§14, invarianti del master, corpo del grafo, ARCH/DET e kit se presente.
Scansiona le istruzioni operative per stop generici e override inattesi; distingue
il registro storico in AGENTS dalla parte corrente. Nessuna mutazione durante
l'audit, nessuna dipendenza runtime o pacchetto aggiunto.

Ricerca iniziale con rg --files --hidden --no-ignore: nessun AGENTS.override.md
o AGENTS annidato nel repository, nessuna istruzione aggiuntiva nei parent
applicabili o configurazione locale .agents/.codex in conflitto. La ricerca finale
classifica gli stop residui nei dossier e nel kit come storici; i gate umani
normativi e di sicurezza restano reali. git diff --check: PASS.

Nessun Rust modificato per la migrazione; non si esegue una CI runtime duplicata
per sole istruzioni documentali. La successiva task applica i propri gate esatti.

## Handoff della migrazione

- Task/revisione conclusa: GOV-2026-10-26-01; non aggiunge ID alle140 task di piano.
- Gate: audit operativo/integrità PASS; git diff --check PASS.
- HEAD iniziale:9cf5eab, work/p1-foundations, working tree inizialmente pulito.
- Stato preesistente:14-05 DONE/M7, CI40298536590 Linux/MSVC success su8a9ffd8.
- Next READY:T-P15-01, prerequisitoT-P8-06 DONE/CI39747270986; preflight tecnico
  completo subito dopo questa revisione, prima di modificare configurazioni.
- Blocker: nessuno per la governance. Rinvii normativi e altri gate restano.
- Prosecuzione: AUTO_CONTINUE, nessuno stop dopo questo handoff.
