# T-MIG-P4-09 — Dettagli, field guide e reference snapshot

19 novembre 2026. Autore Claude Code, revisore Codex. Baseline `integration`
`56627a1` (P4-08 e P4-11 DONE), branch `work/T-MIG-P4-09`.
Stato: **IN_REVIEW**. Le verifiche locali sono PASS; mancano la review Codex,
la CI remota e le prove Windows. G-MIG-4 resta aperto.

Specifica: `docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P4-09.md`.
Acceptance: valori e diagnostici corrispondono al report originale e alla
fonte Rust; nessun cast JS a number non sicuro.
Contratto: `docs/migration/ipc-v1/DETAIL.md` e la reference
`detail_reference.py`/`verify_detail.py` (P2-06, IPC v1, ratifica 14/11/2026).
Prerequisito: P4-08 DONE (`6925385`).

## Cosa cambia

- `src-tauri/src/jobs.rs`:
  - il `Report` pubblicato cattura in `Captured` lo `CompiledFlowSchema`
    risolto dal registry del run (hash già verificato contro la summary);
  - cattura anche la `PreparedReferenceEvidence` passata al servizio, tramite
    `CapturingReferences`, lo stesso schema del worker Slint;
  - l'evidenza è conservata solo se coincide con l'audit della summary
    (dataset, digest, availability, chiave dichiarata); altrimenti non è
    catturata.
- `src-tauri/src/results/detail.rs` (nuovo), cinque operazioni:
  - `results_select`: risolve un `diagnostic_ref` delle due pagine trattenute
    tramite la posizione di riga registrata, rilegge la diagnostica canonica
    dallo spool e la fissa. La selezione sopravvive all'espulsione della
    pagina; non sopravvive a una nuova query, a un nuovo report o a una nuova
    selezione;
  - `results_text`: chunk UTF-8 esatti, 1..16384 byte;
  - `results_context`: pagine da 64 elementi al massimo;
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
- `src/ipc.ts`: client strict per le cinque operazioni. Coordinate, offset e
  righe restano stringhe decimali (confronti con `BigInt`); visibilità,
  stati, digest e handle sono validati.
- Test: 7 nuovi test nativi in `detail.rs`; `tests/detail.test.mjs` (5).

### Contenuto della selezione

Il DTO contiene:
- riferimenti `selection_ref`, `query_ref`, `diagnostic_ref`, `run`;
- `line` u64 testuale, `trace_ref`;
- `field_position` in byte, 1-based inclusivi;
- `actual_visibility`;
- componenti con handle e preview: code, message, expected, field_name,
  guide, actual, sioc_record_id, fiscal_code, episode_id.

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
   `conditional: true` e RuleId;
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
- libreria `lintra`, UI Slint e workspace (`src/`, `ui/`, `tests/`);
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
| DTO esatti | `detail_dtos_are_exact_and_reject_noncanonical_values` e `tests/detail.test.mjs`. |
| Niente cast number JS unsafe | Linea, coordinate, offset e totali sono stringhe u64; i limiti numerici (16384, 64) sono interi piccoli verificati; test con linea `18446744073709551615`. |

Test di mutazione: forzando `visible = true` in `select`, il test sui valori
dell'intero record fallisce. Il sorgente è stato ripristinato con SHA
identico (`d64295a2…2eb3`, prima della correzione Clippy).

## Verifiche locali (Linux CachyOS, rustc 1.99.1, Node v26.13.0)

Sequenza ufficiale del job Linux di `.github/workflows/desktop.yml` in
`apps/lintra-desktop`, sui sorgenti finali:

| Comando | Esito |
|---|---|
| `npm ci --ignore-scripts --no-audit --no-fund` | PASS / 0 |
| `npm run lint` | PASS / 0 |
| `npm test` | 37 PASS, 0 fail / 0 (32 + 5 nuovi) |
| `npm run build` | PASS / 0 |
| `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` | PASS / 0 |
| `cargo test … --release --target x86_64-unknown-linux-gnu --features tauri/custom-protocol -j 2` | 54 PASS, 1 ignored GTK preesistente / 0 (47 + 7 nuovi) |
| `cargo clippy … --all-targets … -- -D warnings` | PASS / 0 |
| `cargo build … --release` | PASS / 0 |
| `python3 tests/native_smoke.py …/release/lintra-desktop` | PASS / 0, binario `22630021…21b1`; display reale, non Xvfb |
| `git diff --check` | PASS / 0 |

Il primo giro del gate ha avuto Clippy FAIL/101 (`question_mark` in
`detail.rs`). Dopo la correzione ho rieseguito l'intero gate. Log fuori
repo: `lintra-review-handoffs/T-MIG-P4-09.gate.log`, SHA-256
`abc8a4a6afce3b7f984f7f4297e00a11ac49f41171a26dc4332cb283c811c273`.

`ci.yml` (workspace): **NOT_RUN** per assenza di delta in `src/`, `ui/` e
`tests/`. CI remota e prove Windows/VM: **NOT_RUN**.

## Sorgenti verificati

| File | SHA-256 |
|---|---|
| `apps/lintra-desktop/src-tauri/src/configuration.rs` | `1de234004427c9aee48a030a4d2fdbf90f5345d1e6fe75f3fbd2f0038de28409` |
| `apps/lintra-desktop/src-tauri/src/jobs.rs` | `e995da6a13c7a1b1ae3539b133a50066ccfbc0108412daa2a7ba27bf723d7970` |
| `apps/lintra-desktop/src-tauri/src/results.rs` | `d6caaed7023a6a8924073df16a70c0ce7447b5ed1c21ac2a1bac4c8b29a24f5b` |
| `apps/lintra-desktop/src-tauri/src/results/detail.rs` | `40f56611ed4f18fb63642f9dce9c6f1769f4ba8f3751f95e9b2eb578eaa4c55b` |
| `apps/lintra-desktop/src-tauri/src/state.rs` | `a67116788889fc648531707f00d90169fbb6cdd67aa7a6cd6de7ba85c10ad9bd` |
| `apps/lintra-desktop/src/ipc.ts` | `a1a7e847be8fc537c684ed8a4ce9535e902e22e83f5a4d97f631096878819cf8` |
| `apps/lintra-desktop/tests/detail.test.mjs` | `e0d80fe75f5c8299146937c9558a02211d5a6efac3d436040fe76fad128cb36f` |

## Scelte da confermare in review

1. Nomi delle cinque operazioni e stato `loading`/`ready` con `read_ref`
   per la lettura reference asincrona: DETAIL.md non li fissa; lo schema è
   lo stesso polling di P4-08.
2. Campi aggiuntivi nel DTO di selezione: `trace_ref` e `domain_sources`
   (con `source_ref`, kind, conditional, RuleId, dataset id/versione e guida
   di confronto). DETAIL.md lascia la scelta della fonte al nativo, senza
   fissare come la si elenca.
3. La condizione di una regola condizionale è rappresentata dal solo RuleId,
   come nella vista Slint: non esiste un renderer nativo del `CompiledExpr`,
   e non ne invento uno.
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
