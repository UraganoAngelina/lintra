# PROTOCOLLO DI COLLABORAZIONE — Progetto [Lintra]

## 0. Artefatti condivisi

All'inizio di ogni sessione leggi, e alla fine aggiorna, i file in `.ai_bridge/`:

- `PROJECT_CONTEXT.md` — visione, stack, vincoli non negoziabili, standard di qualità.
- `CURRENT_STATE.md` — cosa è fatto, cosa è in corso, cosa è bloccato.
- `HANDOFF.md` — branch, commit, file toccati, validazioni, problemi aperti, prossime azioni.
- `DECISIONS.md` — log append-only delle decisioni architetturali.
- `LEDGER.md` — stato dei task: ID, agente, branch, lane, stato, modello/effort usato.

## 1. Classificazione dei task (obbligatoria prima di qualsiasi routing)

Ogni task atomico deve essere classificato in una delle tre lane:

- **Lane 1 — Routine**: spec congelata, I/O definiti, test di accettazione già
  scritti o derivabili meccanicamente. Nessun dominio critico.
- **Lane 2 — Implementazione non banale**: multi-file, invarianti da rispettare,
  edge case elencati ma non dettagliati. Nessun dominio critico.
- **Lane 3 — Critico**: autenticazione, autorizzazione, pagamenti, concorrenza,
  migrazioni dati, policy di sicurezza, parser/regex critiche, integrazioni fragili.
  **Marcare sempre con `[LANE-3]`** nel titolo del task e nel `LEDGER.md`.

La classificazione va scritta in `LEDGER.md` prima di procedere.

## 2. Routing

### Per Lane 1 e Lane 2 — AUTOMATICO

Prima di delegare un task di Lane 1 o 2, esegui in quest'ordine:

1. `agroute usage` — ispeziona la quota residua.
2. `agroute candidates --host claude --output /tmp/TASK-candidates.json --json`
   — emetti le route candidate per l'host corrente.
3. Risolvi il piano con `agroute route --task "<descrizione>"`.
4. Delega con lo strumento nativo dell'host, usando modello ed effort indicati
   dal piano.
5. Registra l'esito e l'uso misurato in `LEDGER.md`.

Non scegliere modello ed effort a mano per Lane 1 e 2: lascia che sia `agroute`
a farlo in base alla difficoltà e alla quota.

### Per Lane 3 — MANUALE, MAI AUTOMATICO

Se il task contiene `[LANE-3]`:

1. **Non invocare `agroute`.** Il routing automatico è vietato.
2. **Fermati e chiedi conferma all'utente**:
   *"Task [LANE-3] rilevato. Specificare manualmente modello ed effort prima di procedere."*
3. Procedi solo dopo che l'utente ha fornito modello ed effort espliciti.
4. Esegui con i flag manuali:
   - Implementazione: `codex exec --model <modello> --effort <effort>`
   - Review indipendente: `claude --model <modello> --effort <effort>`

## 3. Isolamento e parallelismo

- Ogni agente lavora nel proprio git worktree o branch isolato:
  `agent/claude/<task-id>` e `agent/codex/<task-id>`.
- **Mai due agenti nella stessa working directory contemporaneamente.**
- I conflitti si risolvono nell'**integration worktree** (`integration/<task-id>`),
  mai nei worktree degli agenti.
- Il merge su `main` avviene solo dopo che l'integration worktree è verde
  (test, lint, type check).

## 4. Review incrociata (obbligatoria prima del merge)

- **Lane 1**: review leggera dell'altro agente (effort `medium`) + quality gate CI.
- **Lane 2**: review dell'altro agente (effort `high`) + quality gate CI.
- **Lane 3**: review **cross-model obbligatoria e manuale**:
  1. Codex implementa (effort Extra High).
  2. Claude revisiona in modo indipendente (effort max), cercando edge case non
     coperti, violazioni dei vincoli, problemi di scalabilità, assunzioni implicite.
  3. Se concordano → merge. Se disaccordo minore → iterazione.
     Se disaccordo maggiore → **escalation all'utente**.

## 5. Handoff strutturato

Quando passi il testimone a un altro agente, scrivi in `HANDOFF.md` e comunica
nel prompt di delegazione:

- Contesto del task.
- Fase corrente (es. "implementazione completata, pronta per review").
- Cosa è stato completato.
- Cosa deve essere verificato.
- Livello di confidenza (basso/medio/alto).
- Branch e commit di riferimento.

## 6. Quality gate (indipendenti dagli agenti)

Prima di ogni merge devono passare: linting, type checking, test suite,
coverage minima concordata, analisi statica. Questi gate non sostituiscono la
review, ma impediscono che errori banali passino perché entrambi gli agenti li
hanno ignorati.

## 7. Divieti espliciti

- **Divieto di routing automatico per Lane 3**: nessun agente può invocare
  `agroute` su un task marcato `[LANE-3]`.
- **Divieto di review "compiacente"**: nessun agente può revisionare il proprio
  codice su Lane 3.
- **Divieto di force-push** su branch condivisi.
- **Divieto di merge senza quality gate verde.**