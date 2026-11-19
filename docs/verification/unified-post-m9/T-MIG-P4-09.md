# T-MIG-P4-09 — Dettagli, field guide e reference snapshot

19 novembre 2026. Autore Claude Code, revisore Codex. Baseline `integration`
`56627a1` (P4-08 e P4-11 DONE), branch `work/T-MIG-P4-09`.
Stato: **DONE** (19 novembre 2026): review Codex round 2 APPROVED su
`7d5cdbc`, integrazione autorizzata dall'autore. CI remota e prove Windows non
eseguite, rinviate al checkpoint del blocco P4. G-MIG-4 resta aperto.

Specifica: `docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P4-09.md`.
Acceptance: valori e diagnostici corrispondono al report originale e alla
fonte Rust; nessun cast JS a number non sicuro.
Contratto: `docs/migration/ipc-v1/DETAIL.md` e la reference
`detail_reference.py`/`verify_detail.py` (P2-06, IPC v1, ratifica 14/11/2026).
Prerequisito: P4-08 DONE (`6925385`).

## Rework 1 — rilievi Codex P409-01 e P409-02 (MAJOR)

Rapporto: `lintra-review-handoffs/T-MIG-P4-09-CODEX-REVIEW.md` (fuori repo).
Entrambi i rilievi sono fondati e corretti.

**P409-01.** Oltre 64 fonti, `results_select` falliva con errore tecnico, e
il client rifiutava più di 64 fonti. Correzione:
- tolto il tetto `MAX_SOURCES`;
- la selezione riporta `source_count` (u64 testuale) invece della lista;
- le fonti si consultano con la nuova operazione `results_sources`
  (`selection_ref`, offset u64, limit 1..64), a pagine;
- i `source_ref` sono stabili per la selezione;
- nessun troncamento e nessun allentamento di 1 MiB o di altri limiti.

**P409-02.** Della regola condizionale era conservato solo il RuleId; la
guardia `rule.when` era omessa. Correzione:
- nuova API pubblica di sola lettura `lintra::rule_condition_guide`
  (`src/engine/field_guide.rs`), che restituisce lo stesso testo controllato
  `condition_text` che il motore rende per `{condition}` nei diagnostici;
- nessuna valutazione, nessuna nuova semantica;
- ogni fonte condizionale della pagina riceve un componente `condition`
  (handle e preview), leggibile per intero con `results_text`;
- una nuova pagina di fonti invalida gli handle di guardia precedenti;
- se un letterale non è rappresentabile come testo, `condition` è null ma
  la fonte resta `conditional: true`;
- test di libreria in `tests/dsl_suite.rs`: la guida coincide col testo dei
  diagnostici golden e cambia se cambia solo la guardia.

Regressione nativa
`sources_are_paged_without_a_count_limit_and_carry_native_guards`:
- 73 fonti: la selezione riesce e le pagine sono 64 + 9;
- la fonte 10 (COND-07) ha guardia `F1 = "9"`;
- la guardia Unicode lunga ha preview incompleta e testo esatto a chunk;
- il dominio di una fonte della seconda pagina si apre;
- offset oltre il totale e limit 65 sono rifiutati;
- due bundle che differiscono solo nella guardia producono fonti identiche
  salvo la guida (`F1 = "1"` / `F1 = "2"`).

Mutazione: sopprimendo la guida, falliscono due test. Il sorgente è stato
ripristinato con SHA identico (`d514f335…104a`, prima dell'alias Clippy).

## Cosa cambia

- `src-tauri/src/jobs.rs`:
  - il `Report` pubblicato cattura in `Captured` lo `CompiledFlowSchema`
    risolto dal registry del run (hash già verificato contro la summary);
  - cattura anche la `PreparedReferenceEvidence` passata al servizio, tramite
    `CapturingReferences`, lo stesso schema del worker Slint;
  - l'evidenza è conservata solo se coincide con l'audit della summary
    (dataset, digest, availability, chiave dichiarata); altrimenti non è
    catturata.
- `src-tauri/src/results/detail.rs` (nuovo), sei operazioni (dopo il rework 1):
  - `results_select`: risolve un `diagnostic_ref` delle due pagine trattenute
    tramite la posizione di riga registrata, rilegge la diagnostica canonica
    dallo spool e la fissa. La selezione sopravvive all'espulsione della
    pagina; non sopravvive a una nuova query, a un nuovo report o a una nuova
    selezione;
  - `results_text`: chunk UTF-8 esatti, 1..16384 byte;
  - `results_context`: pagine da 64 elementi al massimo;
  - `results_sources`: pagine da 64 fonti al massimo, con guardia nativa per
    le fonti condizionali (rework 1);
  - `results_domain`: una fonte alla volta, con cursore opaco unico;
  - `results_domain_state`: polling della lettura reference, che gira su un
    worker.
- `src-tauri/src/results.rs`:
  - il registro pagine passa da token a (token, riga);
  - la selezione vive dentro la `Query` corrente, quindi viene invalidata
    con lei;
  - cancel alla chiusura;
  - dispatch delle nuove operazioni.
- `state.rs`, `configuration.rs`: instradamento attraverso il gate
  serializzato, dopo `app_info`.
- `src/engine/field_guide.rs`, `src/lib.rs`: `rule_condition_guide`
  pubblica, che riusa `condition_text` (rework 1).
- `src/ipc.ts`: client strict per le sei operazioni. Coordinate, offset e
  righe restano stringhe decimali (confronti con `BigInt`); visibilità,
  stati, digest e handle sono validati.
- Test: 8 nuovi test nativi in `detail.rs`; `tests/detail.test.mjs` (6);
  un test di libreria in `tests/dsl_suite.rs`.

### Contenuto della selezione

Il DTO contiene:
- riferimenti `selection_ref`, `query_ref`, `diagnostic_ref`, `run`;
- `line` u64 testuale, `trace_ref`;
- `field_position` in byte, 1-based inclusivi;
- `actual_visibility`;
- componenti con handle e preview: code, message, expected, field_name,
  guide, actual, sioc_record_id, fiscal_code, episode_id;
- `source_count`: numero delle fonti di dominio, consultabili a pagine con
  `results_sources` (rework 1).

`guide` è `field_format_guide` sul campo compilato del trace della
diagnostica, quindi vale anche per Full. Mai `raw_hex`, mappa
`technical_ids`, path o riga fisica.

`actual` è esposto solo se trace, FieldId e range coincidono con lo schema
compilato e il campo è un sottoinsieme proprio del record; altrimenti
`local_report_only_record_scope`. Fa eccezione il conteggio ASCII di
`E_RECORD_LENGTH` senza campo, che è metadato. Il contesto segue la stessa
regola di copertura.

### Fonti di dominio

Seguono l'ordine della vista Slint:
1. vincoli del campo;
2. vincoli che le regole locali asseriscono sul campo, con
   `conditional: true`, RuleId e la guardia `rule.when` come componente
   `condition` nativo (rework 1);
3. per `reference_data`: prima i fallback (`fallback`), poi un `reference`
   per dataset.

Comportamento per tipo:
- `allowed_values`, `range` e `fallback` usano `declared_domain_page`
  (pagine da 64, `IntegerLiteral` esatto);
- `regex` e gli altri vincoli danno `not_enumerable`, con `entries` null;
- `reference` usa `PreparedReferenceEvidence::domain_page` sul worker, con i
  letterali di `domain_literal`;
- gli stati espliciti sono `not_captured`, `degraded`, `unavailable`,
  `unsupported`, `cancelled`, `available` e `available_empty`;
- un digest Verified che non coincide è un errore tecnico.

## Cosa NON cambia

Restano invariati:
- motore, DSL, compiler, servizio, writer JSONL, golden e CLI;
- UI Slint e semantica della libreria. L'unico delta di libreria è la
  funzione pubblica di sola lettura `rule_condition_guide` del rework 1, con
  il suo test;
- query/paging P4-08 (stessi DTO e limiti), lifecycle e progress;
- manifest, lockfile e dipendenze.

Nessuna regola normativa nel client.

Fuori scope:
- export e About (P4-10);
- progresso, cancel e cleanup Full (P4-12);
- UI React di consultazione.

## Acceptance

| Criterio | Evidenza |
|---|---|
| Valori e diagnostici = report originale e fonte Rust | `selected_detail_matches_the_canonical_diagnostic_and_captured_schema`: run reale del worker di produzione. Per ogni riga la selezione viene confrontata con il `ValidationError` riletto dall'indice canonico: code, message, expected, nome campo, actual, linea, posizione. Ogni testo è ricostruito con chunk da 7 byte. La guida è uguale a `field_format_guide` dello schema compilato catturato. Nessun `raw_hex`, `technical_ids` o path. |
| Snapshot reference del run | `domains_page_declarations_and_the_verified_reference_snapshot`: il dataset viene modificato su disco dopo il run, ma la pagina mostra `"AA"` con digest atteso = effettivo = digest del caso. Fallback `"ZZ"`, regola condizionale COND-F2 con `"A"`, range 0..200 in 4 pagine senza buchi, regex `not_enumerable`. Cursori, valori e `read_ref` superati sono stale. |
| Evidenza non verificata | `unverified_reference_evidence_is_never_an_empty_valid_domain`: `degraded`, `entries` null, nessun worker avviato. |
| Valori fuori copertura, contesto, testi | `whole_record_values_unknown_context_and_texts_stay_bounded`: range diverso dallo schema e campo che copre l'intero record → valore trattenuto; contesto di trace sconosciuto trattenuto; messaggio Unicode da circa 60 KB ricostruito esatto; confini UTF-8, EOF e offset fuori range; pagine di contesto 64/64/12; conteggio `E_RECORD_LENGTH` esposto. |
| Pin e invalidazione | `selection_is_pinned_across_eviction_and_dropped_by_a_new_query`. |
| Contratto della pagina provider | `provider_pages_violating_the_native_contract_are_technical_failures`: 65 valori o pagina vuota con continuazione → errore tecnico; `unsupported`, `cancelled` e `available_empty` distinti; `read_ref` mai riutilizzabile. |
| Fonti senza tetto e guardia nativa (rework 1) | `sources_are_paged_without_a_count_limit_and_carry_native_guards`; `tests/dsl_suite.rs::rule_condition_guide_matches_the_golden_condition_text`. |
| DTO esatti | `detail_dtos_are_exact_and_reject_noncanonical_values` e `tests/detail.test.mjs`. |
| Niente cast number JS unsafe | Linea, coordinate, offset e totali sono stringhe u64; i limiti numerici (16384, 64) sono interi piccoli verificati; test con linea `18446744073709551615`. |

Test di mutazione: forzando `visible = true` in `select`, il test sui valori
dell'intero record fallisce. Il sorgente è stato ripristinato con SHA
identico (`d64295a2…2eb3`, prima della correzione Clippy).

## Verifiche locali (Linux CachyOS, rustc 1.99.1, Node v26.13.0)

Rework 1, sui sorgenti finali. Sequenza Linux di `desktop.yml` in
`apps/lintra-desktop`, poi workspace di `ci.yml` (la libreria ora cambia):

| Comando | Esito |
|---|---|
| `npm ci --ignore-scripts --no-audit --no-fund` | PASS / 0 |
| `npm run lint` | PASS / 0 |
| `npm test` | 38 PASS, 0 fail / 0 |
| `npm run build` | PASS / 0 |
| `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` | PASS / 0 |
| `cargo test … --release --target x86_64-unknown-linux-gnu --features tauri/custom-protocol -j 2` | 55 PASS, 1 ignored GTK preesistente / 0 |
| `cargo clippy … --all-targets … -- -D warnings` | PASS / 0 |
| `cargo build … --release` | PASS / 0 |
| `python3 tests/native_smoke.py …/release/lintra-desktop` | PASS / 0, binario `61021e4b…9020`; display reale, non Xvfb |
| `cargo fmt --all -- --check` | PASS / 0 |
| `cargo clippy --workspace --all-targets --locked -j 2 -- -D warnings` | PASS / 0 |
| `cargo test --workspace --locked -j 2` | 1.113 PASS, 0 fail, 9 ignored / 0 |
| `cargo clippy --workspace --all-targets --all-features --locked -j 2 -- -D warnings` | PASS / 0 |
| `cargo test --workspace --all-features --locked -j 2` | 1.113 PASS, 0 fail, 9 ignored / 0 |
| `git diff --check` | PASS / 0 |

Il primo giro del rework ha avuto Clippy desktop FAIL/101
(`type_complexity`): l'ho corretto con l'alias `RuleRef` e ho rieseguito
l'intero gate. Log fuori repo:
`lintra-review-handoffs/T-MIG-P4-09.r1-gate.log`, SHA-256
`f613fae1af00d08c2ab68f40d166ec8c5c4b001ad65a4b909513c41e0cbce669`. Il log
della consegna originale (`T-MIG-P4-09.gate.log`, `abc8a4a6…c273`) resta
evidenza storica.

Non eseguiti: gli step speciali di `ci.yml` (test `--ignored`,
`--no-default-features`, bench, script, xtask), CI remota e prove
Windows/VM.

## Sorgenti verificati

| File | SHA-256 |
|---|---|
| `apps/lintra-desktop/src-tauri/src/configuration.rs` | `1de234004427c9aee48a030a4d2fdbf90f5345d1e6fe75f3fbd2f0038de28409` |
| `apps/lintra-desktop/src-tauri/src/jobs.rs` | `e995da6a13c7a1b1ae3539b133a50066ccfbc0108412daa2a7ba27bf723d7970` |
| `apps/lintra-desktop/src-tauri/src/results.rs` | `39b7e8e5be3e28454fdfe1eea17fce42218ef5e10ddd6efbdc46b49249689abc` |
| `apps/lintra-desktop/src-tauri/src/results/detail.rs` | `a9e6142020447eba91e458a7626545f401a6727ce3cc8e5ddb671c1fc5470776` |
| `apps/lintra-desktop/src-tauri/src/state.rs` | `0cf8b0c0909836dbf7fae76340f19f43d9b0cccbcaede203a6d8a10421db3171` |
| `apps/lintra-desktop/src/ipc.ts` | `d424019f29840fa94e89b018b502e29e8ec71bcb08cfa57a96a0b60848e0c32f` |
| `apps/lintra-desktop/tests/detail.test.mjs` | `54e2f5211cb2751d15bc1d4920f2cb4f822f1c1bd022e24c1f2943d6d7f325b3` |
| `src/engine/field_guide.rs` | `33611633371ba66438e0cf179c115a934b466a8f0f652d05626aa4e0d58909a3` |
| `src/lib.rs` | `0642e5b82e0c8b64cb7c9cbf8f491b8745d35c1fa90dd11b9a41b26743054d2c` |
| `tests/dsl_suite.rs` | `b3b4712ef0601cfc6bf503e6a88f2125bf33880f1d4cc16be1b2d551a83da9c1` |

## Scelte da confermare in review

1. Nomi delle sei operazioni e stato `loading`/`ready` con `read_ref`
   per la lettura reference asincrona: DETAIL.md non li fissa; lo schema è
   lo stesso polling di P4-08.
2. Campi aggiuntivi: `trace_ref` e `source_count` nella selezione; nelle
   pagine `results_sources`: `source_ref`, kind, conditional, RuleId,
   `condition`, dataset id/versione e guida di confronto. DETAIL.md lascia la
   scelta della fonte al nativo, senza fissare come la si elenca.
3. (Rework 1) La guardia è il testo `condition_text` del motore, esposto da
   `rule_condition_guide`. Una guardia non rappresentabile come testo dà
   `condition: null` con `conditional: true`.
4. Per diagnostiche senza campo (o con campo assente dallo schema) l'elenco
   delle fonti è vuoto. Lo stato `record_scope` della reference non è quindi
   raggiungibile tramite IPC.
5. Lo stato `cancelled` si produce solo alla chiusura dell'applicazione
   (`cancel_on_close`): non c'è un'operazione di cancel dedicata. Una
   lettura sostituita viene cancellata, attesa e scartata.

## Rischi residui

- La lettura reference è O(N log 64) per pagina sul worker; il join di una
  lettura superata avviene sul thread della chiamata (cancel cooperativo tra
  valori), come per i candidati P4-08. Non misurato su Windows.
- Il DTO di selezione è verificato contro il limite di 1 MiB in
  codifica: un caso patologico (più di 64 fonti) è un errore tecnico, non un
  taglio.
- La minimizzazione non è una garanzia PHI-free: valori di campo e messaggi
  restano dati riservati lato DOM, come dichiarato da DETAIL.md.
- G-MIG-4 resta aperto: servono P4-10, P4-12 e la CI nativa Linux/Windows.

Prossima task tecnicamente disponibile: T-MIG-P4-12 (P4-10 dipende anche da
P4-12).

## Chiusura

- Review Codex round 1 su `5027ee4`: **CHANGES_REQUESTED** per P409-01 e
  P409-02 (MAJOR), corretti nel rework 1 `7d5cdbc`.
- Review Codex round 2 su `7d5cdbcabf8505ebcd1826ac064ec97419f7fab9`:
  **APPROVED**, nessun rilievo aperto (rapporti
  `lintra-review-handoffs/T-MIG-P4-09-CODEX-REVIEW.md` e `-REVIEW-2.md`, fuori
  repo).
- Repliche indipendenti di Codex, tutte exit 0:
  - 55 test nativi PASS (1 ignored GTK) e 38 Node PASS;
  - Clippy, lint, build e fmt;
  - suite DSL (5 PASS);
  - riproduzione con 2, 64 e 65 fonti e con guardie distinte.
  I dieci digest sorgente coincidono. Workspace completo, smoke e mutazioni
  sono stati verificati da Codex sul log locale dell'autore, non rieseguiti.
- Integrazione autorizzata dall'autore il 19/11/2026: fast-forward di
  `integration` da `56627a1` a `7d5cdbc`, senza merge; si aggiunge questo
  solo commit documentale.
- **Non eseguito:** CI remota, prove Windows/MSVC/VM e step speciali di
  `ci.yml`, rinviati al checkpoint del blocco P4. G-MIG-4 e G-REL aperti.
- Rischi residui accettati in review:
  - lettura reference O(N log 64) sul worker;
  - join cooperativo di una lettura superata, non misurato su Windows;
  - test di dettaglio prevalentemente su run LocalOnly.
