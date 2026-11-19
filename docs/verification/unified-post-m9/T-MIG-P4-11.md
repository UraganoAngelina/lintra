# T-MIG-P4-11 — Rust adapter Full multi-tracciato su ValidationService esistente

18 novembre 2026, rework 19 novembre 2026. Autore Claude Code (sviluppatore),
revisore Codex. Baseline: `integration` @ `ab679c5c9daf08edc5d963a5bc21c32cb8343186`
(P4-08, approvata da Codex il 19/11; consegna originale su `42aa890`),
branch `work/T-MIG-P4-11`, worktree `../lintra-p4-11`.
Stato: **DONE** (19 novembre 2026), integrazione autorizzata dall'autore.
Il rework 1 non ha avuto una seconda review Codex: l'autore vi ha rinunciato
esplicitamente. CI remota e Windows sono rinviate al checkpoint del blocco P4.

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

Test di mutazione (consegna originale su `42aa890`, non ripetuto nel rework): ripristinando il vecchio gate (`mode != LocalOnly ||`) il
test bridge fallisce (`InvalidState` alla `validation_start` Full, exit 101);
il sorgente è stato ripristinato con SHA identico al backup.

## Rework 1 — rilievo Codex P411-01 (MAJOR)

Rapporto: `lintra-review-handoffs/T-MIG-P4-11-CODEX-REVIEW.md` (fuori repo).
Rilievo fondato: sulla combinazione con `ab679c5` i test Full in `jobs.rs`
leggevano `report._directory` e tre volte `report._index`, campi rinominati da
P4-08 in `directory` e `index: Mutex<JsonlReportIndex>` (quattro E0609).

Correzione: branch ribasato su `ab679c5` (`git rebase`, nessun conflitto
testuale; nessun merge su `integration`). Nel solo test
`full_worker_matches_direct_service_outcome_summary_and_canonical_jsonl` lo
spool si legge da `report.directory` e l'indice tramite l'accessor di
produzione `Report::with_index` (lock del Mutex, report condiviso in `Arc`).
Le quattro asserzioni restano identiche: `total` e `row_count` uguali a
`total_errors`, `decoded_rows() == 0`, JSONL uguale al servizio diretto. Il
delta di produzione (gate di start in `configuration.rs`) non cambia.

## Verifiche locali (Linux CachyOS, rustc 1.99.1, Node v26.13.0)

Sorgenti finali del rework 1, ribasati su `ab679c5`. Sequenza ufficiale del job Linux di `.github/workflows/desktop.yml`, eseguita
nella directory `apps/lintra-desktop` sui sorgenti finali:

| Comando | Esito |
|---|---|
| `npm ci --ignore-scripts --no-audit --no-fund` | PASS / 0 |
| `npm run lint` | PASS / 0 |
| `npm test` | 32 PASS, 0 fail / 0 (31 su `ab679c5` + 1 nuovo) |
| `npm run build` | PASS / 0 |
| `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` | PASS / 0 |
| `cargo test --manifest-path src-tauri/Cargo.toml --locked --release --target x86_64-unknown-linux-gnu --features tauri/custom-protocol -j 2` | 47 PASS, 1 ignored GTK preesistente / 0 (43 su `ab679c5`, inclusi i 9 test `results`, + 4 nuovi) |
| `cargo clippy … --all-targets … -- -D warnings` (stessi flag) | PASS / 0 |
| `cargo build … --release` (stessi flag) | PASS / 0 |
| `python3 tests/native_smoke.py src-tauri/target/x86_64-unknown-linux-gnu/release/lintra-desktop` | PASS / 0, `LINTRA_DESKTOP_READY` una volta; binario `d86af7f52f9bec9486d46c5480ea4e522e912095f905c662b454dbee8a4f82c3` |
| `git diff --check` | PASS / 0 |

Consegna originale (su `42aa890`): il primo `cargo test` era fallito
(exit 101) per `dist/` assente, poi ripetuto nell'ordine ufficiale. Lo smoke è stato eseguito sul display reale della
sessione (XWayland, `GDK_BACKEND=x11`, `LIBGL_ALWAYS_SOFTWARE=1`) perché
`xvfb-run` non è installato: non equivale allo step Xvfb della CI.

Log del rework 1 (non artefatto CI), copiato fuori repo accanto all'handoff:
`lintra-review-handoffs/T-MIG-P4-11.r1-gate.log`, SHA-256 riportato
nell'handoff. I log della consegna originale (`p411-gate.log`,
`p411-smoke.log`, `p411-mutation.log`) erano solo nello scratchpad di sessione
e non sono più reperibili: valgono come dichiarazione storica, non evidenza.

`ci.yml` (workspace `lintra` + `ui`) non è applicabile: il crate desktop è
escluso dal workspace e il delta non tocca `src/`, `ui/` o `tests/`;
suite workspace **NOT_RUN** per assenza di delta. CI remota `desktop.yml`
Linux/Windows: **NOT_RUN** (richiede push autorizzato). Prove Windows/VM: NOT_RUN.

## Sorgenti verificati

| File | SHA-256 |
|---|---|
| `apps/lintra-desktop/src-tauri/src/configuration.rs` | `7b40370730c7a7479df30faa931a45b7de12dba65e53d0e926f6ff4d53a648c4` |
| `apps/lintra-desktop/src-tauri/src/jobs.rs` | `8c70bbf4fc4d9e1ded7e9f8af198e1a4548752634c7251a8d0931c3993665716` |
| `apps/lintra-desktop/tests/jobs.test.mjs` | `7fd24a759dc881994277de30c41d2b1ba547f982dfd3ae4c9af7d07e6a404bf8` |

## Rischi residui e prossimi passi

- Riallineamento con P4-08 risolto nel rework 1 (P411-01). Se `integration`
  avanza di nuovo prima dell'integrazione, le evidenze vanno rivalutate sul delta.
- Export Full con protezione di tutti gli input del bundle e About restano in
  P4-10/P4-12: il report Full è consultabile ma non esportabile da Tauri.
- G-MIG-4 aggregato resta aperto: servono P4-08..12 e la CI nativa Linux/Windows.
- Prossima task tecnicamente pronta dopo l'integrazione: T-MIG-P4-12
  (dipende da P4-11 e P4-05).

## Chiusura

- Review Codex su `caab33d`: **CHANGES_REQUESTED**, con il solo rilievo
  P411-01 (MAJOR). Nessun altro rilievo sul delta di produzione, che Codex ha
  replicato con esito positivo.
- Il rework 1 corregge P411-01 (vedi sopra). L'autore ha autorizzato il
  19/11/2026 l'integrazione **senza una seconda review Codex** del rework.
  Il rework tocca solo test (9+/7−); questo è registrato come deroga
  dell'autore, non come approvazione Codex.
- Integrazione lineare su `integration`: P4-08 (`ab679c5`) e la sua chiusura
  documentale (`6925385`), poi P4-11 ribasato su `6925385` (`f97cb7c` +
  `1f33be8`, contenuto identico a `9456931`/`8b84dec`) e fast-forward.
  Rispetto a `8b84dec` cambiano solo file documentali di P4-08.
- Gate di merge locale sull'albero integrato (`1f33be8`), tutto exit 0:
  - sequenza Linux di `desktop.yml`, 9 comandi: 32 Node PASS, 47 nativi PASS
    + 1 ignored GTK, Clippy, build release, smoke `NATIVE_DESKTOP_SMOKE_PASS`
    sul display reale (non Xvfb);
  - workspace di `ci.yml`: fmt, Clippy con e senza `--all-features`, test
    1.112 PASS / 0 fail / 9 ignored in entrambe le configurazioni.

  Log fuori repo: `lintra-review-handoffs/T-MIG-P4-11.merge-gate.log`, SHA-256
  `f1a903b223795d4ab0bda0db860fbaf550f59a9c70aa78976ad125aebd64cd3c`.
- **Non eseguiti:**
  - gli step speciali di `ci.yml` (test `--ignored`, `--no-default-features`,
    bench smoke, script Python e xtask);
  - CI remota e prove Windows/MSVC/VM. La qualifica nativa resta al
    checkpoint del blocco P4.

  G-MIG-4 e G-REL restano aperti.
