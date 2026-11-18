# T-MIG-P4-11 — Rust adapter Full multi-tracciato su ValidationService esistente

18 novembre 2026. Autore Claude Code (sviluppatore), revisore Codex.
Baseline: `integration` @ `42aa890febf2ef05c15d93a0072ca7037583dd7b`,
branch `work/T-MIG-P4-11`, worktree `../lintra-p4-11`.
Stato: **IN_REVIEW** (in attesa della review Codex). Non DONE: mancano review,
integrazione autorizzata e CI remota/Windows.

Prompt: `docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P4-11.md`.
Contratto: `docs/migration/ipc-v1/FULL.md` e `full-cases.json` (P2-11, ratificati
in G-MIG-2 il 15/11). Fonte: `FONTI/MIGRATION_PLAN_V1_1_2026-10-30.md` riga 280.

## Prerequisiti

- T-MIG-P4-06 DONE (`5e3acdb`, CI 44065663299) e T-MIG-P2-11 DONE (G-MIG-2),
  come da `UNIFIED_IMPLEMENTATION_STATUS.md`.
- Preflight del codice: la preparazione Full (P4-03) già cattura
  `ValidationScope::Flow` con `ValidationMode::Full`, input nell'ordine dello
  schema, `VerifiedRunContext` e `preflight_flow_inputs`. Il worker P4-04..07 è
  già generico sullo scope (`validate_with_typed_parameters`, `Progress` su
  Flow, spool atomico, indice P4-07). L'unico blocco era il gate di start in
  `Configuration::job`, che rifiutava esplicitamente ogni `mode != local_only`
  ("Full execution is integrated in P4-11").

## Cosa è cambiato

- `apps/lintra-desktop/src-tauri/src/configuration.rs`: lo start accetta Full
  quando il piano preparato è Full. La mode richiesta deve coincidere con quella
  catturata da `plan_prepare`: Full→LocalOnly e LocalOnly→Full restano
  `InvalidState`, prima di creare un worker (nessun downgrade silenzioso).
  Nuovo test bridge end-to-end.
- `apps/lintra-desktop/src-tauri/src/jobs.rs`: soltanto test. Fixture Full
  sintetica sullo schema condiviso `tests/support/full_schema.rs` (S→T, join
  KEY, confronto DATA); helper per l'invocazione diretta del servizio.
- `apps/lintra-desktop/tests/jobs.test.mjs`: DTO esatto di `validation_start`
  con `mode: "full"`, snapshot Full Partial, rifiuto delle mode non canoniche.

## Cosa NON è cambiato

Motore, DSL, compiler, PASS1/PASS2, servizio, writer JSONL, golden, CLI,
indice/cache P4-07, DTO e operazioni IPC (`validation_start` accettava già
`full` nel decoder e nel client TS), worker, progresso, cancellazione,
manifest e lockfile. Nessuna dipendenza nuova, nessun test di acceptance
modificato. Nessuna API PERSIST/approve/C1. UI prodotto, query/paging (P4-08),
dettagli (P4-09), export Full con protezione multi-input (P4-10/12) restano
fuori scope. `full-cases.json` mantiene `native_Full_adapter: NOT_IMPLEMENTED`:
è un artefatto di design P2-11 coperto da receipt e digest; l'aggiornamento
spetta alla chiusura autorizzata, non a questo delta.

## Acceptance

| Criterio | Evidenza |
|---|---|
| Test end-to-end bridge Full con fixture sintetica | `configuration::tests::full_bridge_runs_prepared_flow_once_and_matches_direct_service_outcome`: catalogo → selezione → `plan_create` Full → due `plan_input_choose` nativi → `plan_prepare` → `validation_start` Full → terminale, su tre bundle (valido, cross invalido, Unknown/Partial). |
| Confronto outcome con invocazione diretta | Stesso test: status e completeness del DTO terminale uguali a `ValidationService::validate_with_typed_parameters` sull'intent catturato. |
| Confronto summary e JSONL canonici | `jobs::tests::full_worker_matches_direct_service_outcome_summary_and_canonical_jsonl`: `ValidationSummary` uguale per `PartialEq` (normalizzati solo `started_at`/`elapsed`), JSONL dello spool uguale riga per riga (rimossi solo `started_at`/`elapsed`), su quattro bundle: valido, `E_CROSS_FIELD`, `E_RELATED_RECORD_MISSING`, Unknown con `E_RECORD_LENGTH` → Partial. Scope `Flow` [S, T], indice con `total`/`row_count` = `total_errors`, nessuna decodifica anticipata, nessun path nel DTO. |
| Un run per bundle, no downgrade | Il bridge rifiuta `local_only` su piano Full con `InvalidState` e lo stato resta `idle`; il run Full successivo ha `run_id` 1 e `mode: "full"`. Il test preesistente rifiuta Full su piano LocalOnly. |
| Cancel in ogni pass, errori tecnici, troncamento | `full_cancel_in_each_pass_and_input_failure_never_publish_or_complete`: cancel su `PhaseStarted(LocalValidation)` e `PhaseStarted(CrossRecordValidation)` → Cancelled/Partial senza report; input T rimosso → `technical_failure` senza `last_report`. `truncated_full_bundle_is_partial_without_published_report`: `max_errors = 1` → truncated, Partial, nessun report. |
| Progresso Full valido | Il campione finale di una run Full non è `failed`: fase `cross_record_validation`/`finalizing_report`, stessa `RunRef`. |

Test di mutazione: ripristinando il vecchio gate (`mode != LocalOnly ||`) il
test bridge fallisce (`InvalidState` alla `validation_start` Full, exit 101);
il sorgente è stato ripristinato con SHA identico al backup.

## Verifiche locali (Linux CachyOS, rustc 1.99.1, Node v26.13.0)

Sequenza ufficiale del job Linux di `.github/workflows/desktop.yml`, eseguita
nella directory `apps/lintra-desktop` sui sorgenti finali:

| Comando | Esito |
|---|---|
| `npm ci --ignore-scripts --no-audit --no-fund` | PASS / 0 |
| `npm run lint` | PASS / 0 |
| `npm test` | 29 PASS, 0 fail / 0 (28 preesistenti + 1 nuovo) |
| `npm run build` | PASS / 0 |
| `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` | PASS / 0 |
| `cargo test --manifest-path src-tauri/Cargo.toml --locked --release --target x86_64-unknown-linux-gnu --features tauri/custom-protocol -j 2` | 38 PASS, 1 ignored GTK preesistente / 0 (34 preesistenti + 4 nuovi) |
| `cargo clippy … --all-targets … -- -D warnings` (stessi flag) | PASS / 0 |
| `cargo build … --release` (stessi flag) | PASS / 0 |
| `python3 tests/native_smoke.py src-tauri/target/x86_64-unknown-linux-gnu/release/lintra-desktop` | PASS / 0, `LINTRA_DESKTOP_READY` una volta; binario `9ff7abe4…147ac` |
| `git diff --check` | PASS / 0 |

Il primo tentativo di `cargo test` è fallito (exit 101) perché `dist/` non
esisteva: la CI esegue `npm run build` prima dei test Rust; ripetuto
nell'ordine ufficiale. Lo smoke è stato eseguito sul display reale della
sessione (XWayland, `GDK_BACKEND=x11`, `LIBGL_ALWAYS_SOFTWARE=1`) perché
`xvfb-run` non è installato: non equivale allo step Xvfb della CI.

Log (scratchpad di sessione, non artefatti CI): `p411-gate.log` SHA-256
`9c4ffc92f9ad85a652e08257df4856aa3f5298862cf8b544f7a67662d834a228`,
`p411-smoke.log`, `p411-mutation.log`.

`ci.yml` (workspace `lintra` + `ui`) non è applicabile: il crate desktop è
escluso dal workspace e il delta non tocca `src/`, `ui/` o `tests/`;
suite workspace **NOT_RUN** per assenza di delta. CI remota `desktop.yml`
Linux/Windows: **NOT_RUN** (richiede push autorizzato). Prove Windows/VM: NOT_RUN.

## Sorgenti verificati

| File | SHA-256 |
|---|---|
| `apps/lintra-desktop/src-tauri/src/configuration.rs` | `3416006587e7e181cee068fedfe37274c7eaea8523696578a2f51204698553b7` |
| `apps/lintra-desktop/src-tauri/src/jobs.rs` | `d77b279ee0919ef2be0bba66a6e547fcc4891ab3e26530c8a73c64c7ece1843f` |
| `apps/lintra-desktop/tests/jobs.test.mjs` | `7fd24a759dc881994277de30c41d2b1ba547f982dfd3ae4c9af7d07e6a404bf8` |

## Rischi residui e prossimi passi

- Durante lo sviluppo `integration` è avanzato a `ab679c5` (P4-08, in review).
  P4-08 rinomina in `jobs.rs` i campi `Report::_index`/`_directory` usati da
  questi test: al riallineamento sarà necessario un adattamento meccanico dei
  soli test. Il delta di produzione (gate di start) non si sovrappone.
- Export Full con protezione di tutti gli input del bundle e About restano in
  P4-10/P4-12: il report Full è consultabile ma non esportabile da Tauri.
- G-MIG-4 aggregato resta aperto: servono P4-08..12 e la CI nativa Linux/Windows.
- Prossima task tecnicamente pronta dopo l'integrazione: T-MIG-P4-12
  (dipende da P4-11 e P4-05).
