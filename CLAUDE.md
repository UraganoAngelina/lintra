# Lintra — Istruzioni per Claude Code

> Claude Code è lo **sviluppatore principale** di Lintra; Codex è il **revisore
> indipendente**. Il workflow AIProto e il precedente protocollo
> `.ai-bridge`/lane/`agroute` sono dismessi dal 18 novembre 2026: niente card,
> runtime, ledger o gate di protocollo.
>
> Per contratto tecnico, ratifiche e fonti canoniche vale `AGENTS.md`
> **sezioni 0, 2, 3** (sezione 5 = registro delle ratifiche, da consultare in
> modo mirato). La sezione 1 di `AGENTS.md` descrive il ruolo di Codex come
> revisore e il formato del suo rapporto.

## 1. Prima di sviluppare una task

1. Verificare stato Git, branch corrente, worktree e modifiche locali; non
   sovrascrivere né eliminare lavoro esistente.
2. Leggere in modo mirato: prompt atomico in
   `docs/unified-post-m9/PROMPT_ATOMICI/`, `UNIFIED_IMPLEMENTATION_STATUS.md`,
   `docs/unified-post-m9/MASTER_PLAN_UNIFICATO.md`, dependency graph, ratifiche
   pertinenti e dossier delle dipendenze in `docs/verification/unified-post-m9/`.
   Non presumere che due copie dello stesso documento siano equivalenti.
3. Verificare che le dipendenze siano davvero concluse e che la task non sia già
   implementata, anche parzialmente (codice e storia Git prevalgono su uno status
   non aggiornato: segnalare la discrepanza).
4. Ricostruire scope, file coinvolti, test richiesti e criteri di accettazione;
   preparare un piano breve e verificabile.
5. Lavorare su un branch dedicato creato da `integration`; una task alla volta,
   senza anticipare task successive.

Fermarsi e chiedere all'autore soltanto per: conflitto reale fra specifiche
autoritative, scelta architetturale non approvata, modifica necessaria fuori
scope, intervento su dati o contratti critici non autorizzato, push/merge/azioni
distruttive, gate obbligatorio non verificabile.

## 2. Durante lo sviluppo

- Implementare end-to-end entro lo scope; riutilizzare astrazioni esistenti;
  nessun refactoring opportunistico né dipendenze nuove senza motivazione
  tecnica verificata.
- Preservare gli invarianti di `AGENTS.md` §2: motore = COME / schema = COSA,
  DSL/compiler, determinismo, streaming, audit, PASS1/PASS2,
  `raw`/`logical`/`MissingValuePolicy`, errori tecnici vs normativi, storia e
  annullamenti, SQLite/CAS, privacy/HMAC/ACL, cancellazione, ordinamento
  diagnostici, IPC v1 (`schemaVersion = 1`, 1 MiB, profondità 32, niente bulk).
- Rust: errori espliciti, niente `panic`/`unwrap`/`expect` ingiustificati nei
  percorsi runtime, risorse e cancellazione controllate.
- React/Tauri: TypeScript strict, IPC minimo e tipizzato, nessuna elaborazione
  massiva o regola normativa nel frontend, job lifecycle governato dal backend.
- Non inventare regole normative, value set o comportamenti di storicizzazione.
- Documentare struct, funzioni, condizioni, invarianti e ragioni di correttezza
  del codice scritto o modificato.
- Test per percorso nominale, errori e input malformati, casi limite,
  regressioni, invarianti, concorrenza e limiti di memoria/prestazioni. Mai
  indebolire test o acceptance per ottenere il verde.

## 3. Verifiche e CI

1. Leggere le workflow in `.github/workflows/` applicabili e ricavarne i
   comandi ufficiali (fmt, clippy, test Rust, test nativi Tauri, Node, build).
2. Eseguire localmente tutto ciò che è replicabile; correggere i fallimenti
   introdotti dalla task.
3. Riportare esiti come `PASS`, `FAIL`, `NOT_RUN` o `BLOCKED`, con comandi ed
   exit code reali. Non dichiarare verde la CI remota sulla base dei test
   locali; CI remota, prove Windows e VM sono evidenze distinte.
4. Push o PR per ottenere CI remota solo con autorizzazione esplicita.

## 4. Commit e consegna

1. Riesaminare il diff: niente modifiche non pertinenti, segreti o artefatti
   temporanei.
2. Aggiornare il dossier `docs/verification/unified-post-m9/<TASK_ID>.md` senza
   dichiarare `DONE` prima di review e integrazione.
3. Commit atomico con trailer `Task: <TASK_ID>`. Mai `--no-verify`,
   force-push, merge su `integration` o push senza autorizzazione.
4. Scrivere l'handoff fuori dal repository:
   `/home/alberto/Desktop/LINTRA/lintra-review-handoffs/<TASK_ID>.md` con ID e
   descrizione, branch e SHA, base di `integration`, documenti consultati,
   criteri soddisfatti con evidenze, file modificati e motivazioni, test e
   comandi, CI locale e remota distinte, limiti e rischi, percorso del dossier,
   stato finale. `READY_FOR_CODEX_REVIEW` solo se tutti i gate pre-review sono
   soddisfatti e non ci sono modifiche non committate.

## 5. Rework dopo la review di Codex

Leggere il rapporto completo
(`lintra-review-handoffs/<TASK_ID>.review-<N>.md`), verificare ogni rilievo
contro le specifiche, correggere quelli fondati, motivare tecnicamente quelli
contestati, rieseguire test e verifiche, nuovo commit, aggiornare handoff e
richiedere nuova review. Il lavoro non è concluso prima di una review positiva e
dell'integrazione autorizzata.

## 6. Comunicazione

Risultati tecnici concreti, supportati da file, output o log reali. Non
dichiarare completato ciò che non è stato verificato.
