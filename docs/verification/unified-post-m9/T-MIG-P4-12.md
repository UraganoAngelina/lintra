# T-MIG-P4-12 — Progresso, cancel e cleanup affidabili per Full

19 novembre 2026. Autore Claude Code, revisore Codex. Baseline `integration`
`02f5000` (P4-09 DONE; P4-11 e P4-05 DONE), branch `work/T-MIG-P4-12`,
worktree `../lintra-p4-12`.
Stato: **DONE** (19 novembre 2026): review Codex APPROVED su `cab4a04`,
integrazione autorizzata dall'autore. CI remota e prove Windows non eseguite,
rinviate al checkpoint del blocco P4. G-MIG-4 resta aperto.

Specifica: `docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P4-12.md`.
Acceptance: dimostrare con test di integrazione nessun report falso, nessun
run zombie, zero trasferimento bulk e stato finale coerente.
Contratti: `docs/migration/ipc-v1/FULL.md` (ownership e ciclo di vita),
`PROGRESS.md`, `LIFECYCLE.md`. Fonte: `FONTI/MIGRATION_PLAN_V1_1_2026-10-30.md`
riga 281.

## Prerequisiti e punto di partenza

- P4-11 (adapter Full) e P4-05 DONE, come da `UNIFIED_IMPLEMENTATION_STATUS.md`.
- Il worker Full era già posseduto: un `CancellationToken` e un thread per
  bundle, terminale solo dopo il join, drop con cancel e join. I test Full di
  P4-11, però, chiamano `execute` direttamente: non passano dal thread del
  `Jobs`, dallo stato IPC, dal progresso, dall'ack o dalla pulizia degli
  spool. Il delta di questa task sta lì.
- Sviluppata da `56627a1` e ribasata su `02f5000` dopo l'integrazione di
  P4-09 (entrambe toccano `jobs.rs`). Non ci sono stati conflitti testuali.
  La fusione di `execute` è stata verificata a mano: la cattura di schema e
  reference di P4-09 e la radice degli spool di P4-12 convivono in
  `execute_in`.

## Cosa cambia

- `src-tauri/src/progress.rs`: un tracciato del bundle il cui PASS1 è già
  iniziato non può ripartire. Prima un `TraceStarted` ripetuto (o il ritorno a
  un tracciato concluso) era accettato e faceva regredire il campione a un
  tracciato finito. Ora è un evento incoerente: sigilla solo il canale di
  progresso (PROGRESS.md) e non tocca l'esito della validazione. Il test
  unitario esistente è esteso al caso Full S, T, S.
- `src-tauri/src/jobs.rs`:
  - `execute_in(…, spool_root)`: lo spool di ogni run viene creato in una
    radice esplicita. In produzione il valore è sempre `None` (directory
    temporanea di sistema, comportamento invariato); `execute` resta solo
    per i test unitari;
  - due punti d'osservazione compilati solo nei test (`#[cfg(test)]`):
    - una sonda che il worker chiama dopo che ogni evento di progresso è
      stato registrato e il lock rilasciato, per fermare il worker in un
      punto esatto;
    - una radice privata degli spool.
  Il binario di produzione non contiene nessuno dei due.
- `src-tauri/src/jobs/full_lifecycle.rs` (nuovo): tre test di integrazione
  sul worker reale del `Jobs`, attraverso le stesse richieste IPC del
  renderer (`validation_state`, `validation_progress`, `validation_cancel`,
  ack, start).

## Cosa NON cambia

Restano invariati:
- motore, servizio, PASS1/PASS2, writer, golden, CLI e libreria;
- DTO e operazioni IPC (nessuna nuova operazione o campo);
- lifecycle, terminale, ack e dettaglio/query P4-08/P4-09;
- manifest, lockfile e dipendenze.

Nessuna regola normativa nel client; nessuna modifica TypeScript.

Fuori scope:
- export Full con protezione di tutti gli input e About (P4-10);
- UI React (P5).

## Acceptance

| Criterio | Evidenza |
|---|---|
| Progresso Full affidabile | `full_progress_is_one_bounded_sample_per_trace_and_pass`: bundle valido S→T da 2 000 record per tracciato. Il worker si ferma a ogni evento e il test legge il campione con `validation_progress`: identità di tracciato `trace_0` poi `trace_1` con ripartenza da 0; PASS2 senza tracciato; contatori e `sequence` monotoni; DTO con esattamente le chiavi di PROGRESS.md. Con il worker fermo a `RunCompleted` lo stato è `finalizing`, l'outcome è null e il cancel risponde `too_late`: il terminale arriva solo dopo il join. |
| Nessun report falso | `full_cancel_through_ipc_in_each_pass_publishes_nothing_and_leaves_nothing`: cancel via IPC con il worker fermo all'avvio di S, all'avvio di T, all'avvio di PASS2 e durante PASS2. Il cancel è idempotente (`requested`, poi `already_requested`); lo stato è `cancelling` senza outcome; il terminale è `cancelled`/`partial` con `report_available: false`, e `last_report` resta assente. Nel test di cleanup, un input rimosso a metà run dà `technical_failure`, mai un esito completed. |
| Nessun run zombie | Dopo ogni terminale il thread del job risulta joinato. Alla chiusura (`cancel_on_close` più drop del manager con un run in corso) il canale della sonda si disconnette, cioè il worker è terminato con il manager. Il terminale resta in attesa fino all'ack: prima dell'ack un nuovo start è `InvalidState`, dopo l'ack il run successivo completa. |
| Cleanup degli spool | Radice privata: esattamente uno spool durante il run, nessuno dopo un cancel o un errore tecnico. Un report più nuovo sostituisce il precedente e ne cancella lo spool; una query sul run superato è `StaleContext`. Dopo la chiusura con un run in corso non resta nessuno spool. |
| Zero trasferimento bulk | In tutti i test ogni frame di stato, progresso e cancel resta sotto 1 KiB e non contiene contenuto degli input (marcatore `Q7` assente). Il numero di frame dipende dalle richieste, non dai 4 000 record. |
| Stato finale coerente | `superseded_closed_and_failed_full_runs_release_every_spool`: dopo un run fallito il report consultabile resta quello del run precedente, con la sua identità; dopo un run completato `last_report` è il nuovo run. |

Test di mutazione: se un nuovo report non sostituisce più il precedente in
`refresh`, il test sugli spool superati fallisce. Il sorgente è stato
ripristinato con SHA identico (`8471e461…61ae`).

## Verifiche locali (Linux CachyOS, rustc 1.99.1, Node v26.13.0)

Sequenza Linux ufficiale di `.github/workflows/desktop.yml` in
`apps/lintra-desktop`, sui sorgenti finali ribasati su `02f5000`:

| Comando | Esito |
|---|---|
| `npm ci --ignore-scripts --no-audit --no-fund` | PASS / 0 |
| `npm run lint` | PASS / 0 |
| `npm test` | 38 PASS, 0 fail / 0 |
| `npm run build` | PASS / 0 |
| `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` | PASS / 0 |
| `cargo test … --release --target x86_64-unknown-linux-gnu --features tauri/custom-protocol -j 2` | 58 PASS, 1 ignored GTK preesistente / 0 (55 + 3 nuovi) |
| `cargo clippy … --all-targets … -- -D warnings` | PASS / 0 |
| `cargo build … --release` | PASS / 0 |
| `python3 tests/native_smoke.py …/release/lintra-desktop` | PASS / 0, binario `297d8929…09d7`; display reale, non Xvfb |
| `git diff --check integration` | PASS / 0 |

Il primo gate sull'albero ribasato ha avuto fmt FAIL/1 e Clippy FAIL/101:
- `execute` risultava codice morto in produzione: ora è `#[cfg(test)]`;
- `type_complexity` nel test: ora c'è l'alias `Hold`.

Dopo le correzioni ho rieseguito l'intero gate. Log fuori repo:
`lintra-review-handoffs/T-MIG-P4-12.gate.log`, SHA-256
`d155d7f05296016e049a763141a3e2dd50b8b1b093d2df16670965261d96be05`.

Suite workspace di `ci.yml`: **NOT_RUN**, perché il delta non tocca `src/`,
`ui/` o `tests/`; la base `02f5000` ha la suite workspace PASS registrata per
P4-09. CI remota e prove Windows/VM: **NOT_RUN**.

## Sorgenti verificati

| File | SHA-256 |
|---|---|
| `apps/lintra-desktop/src-tauri/src/jobs.rs` | `8471e461c149d7baa40bc726a20b672e61f96cef3c8f8092149d8f996c4f61ae` |
| `apps/lintra-desktop/src-tauri/src/jobs/full_lifecycle.rs` | `6d6ec8bf03a3e7e099401dde9eb510858cfb42488555a2d594642b1a891ab0b8` |
| `apps/lintra-desktop/src-tauri/src/progress.rs` | `a489957fb82a3c455fca53c3369b3a8d5582ad0943eec814e2158209ba970194` |

## Scelte da confermare in review

1. **Punti d'osservazione solo nei test.** La sonda e la radice degli spool
   esistono solo con `#[cfg(test)]`. In produzione il worker è identico,
   salvo la chiamata a `execute_in(…, None)`.
2. **Ripartenza di un tracciato come evento incoerente.** Il progresso
   rifiuta un secondo `TraceStarted` per lo stesso tracciato. Il servizio
   legge ogni tracciato una sola volta in PASS1 (`flow_execution.rs`), quindi
   il caso indica un producer incoerente; sigilla solo il progresso, non
   l'esito.
3. **Errore tecnico a metà run.** È prodotto rimuovendo l'input T mentre il
   worker è fermo all'avvio di S. Si basa sul fatto che il servizio apre T
   all'inizio del suo pass, comportamento verificato dal test.

## Rischi residui

- Il join alla chiusura avviene sul thread che chiude, con cancellazione
  cooperativa tra record: una singola lettura I/O bloccata non ha una
  latenza massima garantita. Non misurato su Windows.
- I test usano una radice degli spool privata. In produzione gli spool
  stanno nella directory temporanea di sistema: un kill del processo può
  lasciarli lì, e PROGRESS.md non promette recupero dopo un kill.
- G-MIG-4 resta aperto: servono P4-10 e la CI nativa Linux/Windows.

Prossima task tecnicamente disponibile: T-MIG-P4-10 (dipende da P4-06,
P4-08, P4-09 e P4-12).

## Chiusura

- Review Codex su `cab4a041d2a2f5063ab67c150487682c448da7e6`: **APPROVED**,
  nessun rilievo BLOCKER/MAJOR/MINOR (rapporto
  `lintra-review-handoffs/T-MIG-P4-12-CODEX-REVIEW.md`, fuori repo).
- Repliche indipendenti di Codex, tutte exit 0:
  - 58 test nativi PASS (1 ignored GTK), inclusi i 3 nuovi test Full;
  - 38 Node PASS;
  - lint, build, fmt, Clippy e `git diff --check`.
  I digest dei sorgenti coincidono, e quello del binario coincide con il log
  dello smoke. Build di produzione, smoke, mutazione e workspace non sono
  stati rieseguiti dal revisore.
- Integrazione autorizzata dall'autore il 19/11/2026: fast-forward di
  `integration` da `02f5000` a `cab4a04`, senza merge; si aggiunge questo
  solo commit documentale. Nessun push.
- **Non eseguito:** CI remota e prove Windows/MSVC/VM, rinviate al checkpoint
  del blocco P4. G-MIG-4 e G-REL aperti.
- Rischi residui accettati in review:
  - cancel cooperativo tra record, senza latenza massima per I/O bloccato;
  - spool di sistema che possono sopravvivere a un kill del processo.
