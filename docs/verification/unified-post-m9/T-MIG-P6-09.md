# T-MIG-P6-09 — Verifica numerica e golden report

Data 27 novembre 2026. Stato: **DONE**. Review Codex round 1 APPROVED su
`8784db0` senza rilievi (`lintra-review-handoffs/T-MIG-P6-09-CODEX-REVIEW.md`).
Integrazione autorizzata dall'autore per fast-forward di `integration`,
senza push. G-MIG-6 e G-REL NOT_VERIFIED: UI Full (P6-10), Windows/WebView2
e CI remota dopo P6-10. Prerequisiti
T-MIG-P6-02..P6-08 DONE (`integration` `107b207`). Branch
`work/T-MIG-P6-09` da `107b207`, worktree
`/home/alberto/Desktop/LINTRA/lintra-p6-09`.

Fonti: prompt atomico `MIGRAZIONE_89/T-MIG-P6-09.md`; MIGRATION_PLAN v1.1
riga P6-09 («G-MIG-6 con matrix completa risultati/export e riferimento al
SHA, assenti feature future») e G-MIG-6 (QG-4/QG-6/QG-7: filtri, dettaglio,
domini, export completi LocalOnly+Full, goldens e comparazione per SHA);
`MIGRATION_SCOPE_PARITY.md`; IPC v1 `PAGING.md` (`results_overview`) ed
`EXPORT.md`; `tests/golden/README.md`; `tests/sioc_referential.rs`; dossier
P6-01..P6-08; `.github/workflows/desktop.yml`. Il `TASK_PROMPTS/T-MIG-P6-09.md`
originale non è nel kit.

## Decisioni dell'autore (27 novembre 2026)

- **Scope del gate.** G-MIG-6 comprende la UI Full, che arriva con
  T-MIG-P6-10, a valle di questa task. P6-09 verifica la matrice LocalOnly
  più la parità Full del backend desktop (worker, risultati, dettaglio,
  export via IPC). **G-MIG-6 resta NOT_VERIFIED** e si chiude dopo P6-10.
- **CI remota e Windows/WebView2.** Una sola volta per tutto il blocco P6,
  dopo P6-10 (LocalOnly e Full insieme). Qui restano NOT_RUN.

## Preflight

- P6-02..P6-08 DONE e integrate; nessuna implementazione precedente di P6-09
  in Git.
- I test desktop esistenti confrontano il worker con una chiamata diretta al
  servizio su fixture create nel test (`jobs.rs`), e risultati/filtri/
  overview su report sintetici scritti a mano (`results.rs`). Nessuno
  confrontava il percorso desktop con i **golden committati** (QG-4).

## Cosa cambia

**Test nuovo** `src-tauri/src/exports/golden_parity.rs` (dichiarato in
`exports.rs`, solo `cfg(test)`). Ogni scenario golden eseguibile dal desktop
passa per il worker di produzione (`Jobs`), le operazioni IPC
`results_overview`, `results_query`/`_state`/`_page`, `results_select`/
`results_text` e il bridge di export (`report_about`, `export_prepare`,
`export_execute`/`_state`). Il confronto è con l'oracolo committato, non con
un'altra esecuzione dello stesso codice:

| Verifica | Oracolo |
| --- | --- |
| Spool pubblicato ed export JSONL | `expected.errors.jsonl`, byte per byte, in ordine |
| Summary del report | `expected.summary.json`, tolti `started_at` ed `elapsed`; per l'adapter `golden-test` anche `validator_version`, e `schema_hash` = `schema_fingerprint(raw)` (l'adapter single-reader hasha i byte del file, il servizio il fingerprint canonico) |
| Sidecar dell'export | lo stesso summary più le tre chiavi `bundle_*` dell'audit del report |
| CSV | writer canonico sullo stesso report |
| Overview | `records_processed`, `valid_records`, `invalid_records`, `total_errors`, `by_code` (ordine e conteggi) dal summary; `by_trace` contato dalle righe dell'oracolo, zeri inclusi |
| Paging non filtrato (2 righe per pagina) | linea, tracciato, codice e messaggio di ogni riga dell'oracolo, in ordine; anteprima incompleta = prefisso esatto |
| Filtri esatti | per ogni codice, linea, tracciato e campo presenti: esattamente le righe dell'oracolo che li portano, in ordine, con `total_diagnostics` invariato |
| Dettaglio di ogni riga | linea, codice, messaggio ed `expected` completi |

Scenari: LocalOnly **12** (synthetic 6, semantic 2, dsl 2, derived 2,
tracciato T1); Full **3** (SIOC referential `valid`, `missing`,
`cardinality`, cinque tracciati, parametro tipizzato
`organisation_namespace = 1` come `tests/sioc_referential.rs`).
Esclusi `sioc-cross-run` e `cad-history`: storia SQLite e snapshot cross-run
sono fuori dallo scope MIG-P6 (U-NORM-04, U-CAD-*).

Per riusarli, gli helper di test di `exports::tests` (`Dialogs`,
`published`, `about`, `prepare`, `execute`, `canonical`) diventano
`pub(super)`; nessun cambio di comportamento.

**Difetto trovato e corretto** (`src-tauri/src/results.rs`, overview):
`PAGING.md` vuole `by_code` «nell'ordine del summary», cioè per simbolo come
lo scrive il summary JSON; l'overview iterava `errors_by_code` nell'ordine di
dichiarazione dell'enum `ValidationCode`. Nel golden semantic invalid usciva
`E_INVALID_TYPE, E_INVALID_DOMAIN, E_OUT_OF_RANGE, E_INVALID_DATE` invece di
`E_INVALID_DATE, E_INVALID_DOMAIN, E_INVALID_TYPE, E_OUT_OF_RANGE`; nel SIOC
`missing` i due codici erano invertiti. Conteggi giusti, ordine no; il grafico
di `Charts.tsx` spareggia i conteggi uguali proprio su quest'ordine («ties by
code, as Rust sorts them»). Ora l'overview ordina per simbolo. Il test
esistente non lo vedeva: i suoi due codici sono già in ordine alfabetico.

## Cosa NON cambia

Core, CLI, golden committati, writer, contratto IPC v1, renderer, client
`ipc.ts`, SQLite, dipendenze, workflow.

Banco di catture non ripetuto: l'unico effetto visibile del fix è l'ordine
dei codici a pari conteggio nel grafico. La fixture `fields.txt` del banco
produce `E_INVALID_DOMAIN` 2 ed `E_INVALID_TYPE` 1 (CLI, stessi argomenti di
P6-08), senza pareggi: le catture P6-08 restano valide.

## Matrice G-MIG-6 (LocalOnly + backend Full) sullo SHA consegnato

| Criterio | LocalOnly | Full (backend desktop) |
| --- | --- | --- |
| Golden byte-identici e ordine (QG-4) | spool ed export JSONL = 12 golden (`golden_parity`) | spool ed export JSONL = 3 golden SIOC (`golden_parity`) |
| Comparazione core sullo stesso albero | `cargo test --test golden` 13 PASS | `cargo test --test sioc_referential` 8 PASS |
| Conteggi (overview, totali, per codice e tracciato) | golden (`golden_parity`); `overview_counts_come_from_the_verified_report_and_its_scope` | golden, cinque tracciati con zeri (`golden_parity`) |
| Paging Rust bounded, ordine canonico | golden; `unfiltered_query_reuses_publication_index_and_pages_are_bounded_and_ordered`; `one_million_diagnostics_stay_in_rust_and_pages_stay_bounded` (QG-6) | golden (`golden_parity`) |
| Cinque filtri esatti (P6-02) | golden; `filtered_queries_are_exact_and_combinable_and_built_off_the_main_thread` | golden, filtro per tracciato su cinque tracciati |
| Dettaglio (P6-03) | golden; `selected_detail_matches_the_canonical_diagnostic_and_captured_schema` | golden (`golden_parity`) |
| Field guide e posizione (P6-04) | `field_guides_and_positions_of_the_synthetic_schema_are_pinned` | — (UI Full: P6-10) |
| Domini statici e reference (P6-05/06) | `domains_page_declarations_and_the_verified_reference_snapshot`; `unverified_reference_evidence_is_never_an_empty_valid_domain` | — (P6-10) |
| Stati, partial, cancelled, troncato (P6-07) | `truncated_service_output_is_partial_and_has_no_published_report`; test Node della matrice esiti | `full_cancel_in_each_pass_and_input_failure_never_publish_or_complete`; `truncated_full_bundle_is_partial_without_published_report` |
| Export canonico e receipt (P6-08) | golden JSONL/CSV + sidecar; `local_only_and_full_reports_export_canonical_files_through_the_bridge` (XLSX: firma ZIP e sidecar) | idem |
| Assenza di feature future | nessuna operazione IPC, schermata o capacità nuova; storia/cross-run non toccati | idem |
| UI React Full, Windows/WebView2, CI remota | — | NOT_VERIFIED: P6-10 e checkpoint del blocco |

## Test

| Verifica | Esito |
| --- | --- |
| `cargo fmt -- --check` (desktop) | PASS (0) |
| `cargo test --locked --release --target x86_64-unknown-linux-gnu --features tauri/custom-protocol -j 2` | PASS (0): 88 passed, 1 ignored (2 nuovi) |
| `cargo clippy … --all-targets … -- -D warnings` | PASS (0) |
| `cargo build …` | PASS (0) |
| Mutazioni (solo `golden_parity`) | overview senza ordinamento: 2 FAIL; linea della riga sfalsata di 1: 2 FAIL; filtro per tracciato che ignora l'handle: 1 FAIL (Full; il LocalOnly ha un solo tracciato); codice reale 2/2 |
| `npm run lint` / `npm test` / `npm run build` | PASS / 185 PASS / PASS (frontend invariato) |
| `native_smoke.py` sul binario Linux (sessione reale, `xvfb-run` assente) | PASS (0), SHA-256 `d296ff03d2be1d40e7db803987a6f116a7d32bc73949fa46d592015d7c10960a` |
| `cargo test --locked --test golden --test sioc_referential` (workspace) | PASS (0): 13 + 8 |
| `native_capture.py` | NOT_RUN: nessun effetto visibile sulle catture (vedi sopra) |
| `npm ci` con i pin, workspace `ci.yml` completo | NOT_RUN: node/npm locali diversi dai pin; core non toccato |
| CI remota, Windows/WebView2 | NOT_RUN: dopo P6-10, decisione dell'autore |

## Rischi residui

- XLSX: il contenuto del workbook non è confrontato con i golden; serve
  aprire lo ZIP (`zip` non è dipendenza del crate desktop, non aggiunta).
  Il writer è quello canonico del core, invariato.
- Il summary dell'adapter `golden-test` differisce per definizione da
  quello del servizio in `validator_version` e `schema_hash`: il test li
  normalizza in modo esplicito, senza mascherare altri campi.
- G-MIG-6 non è dichiarato: mancano la UI Full (P6-10), Windows/WebView2 e
  CI remota sullo stesso SHA.

Prossima task tecnicamente disponibile: T-MIG-P6-10
(dipende anche da T-MIG-P5-10, DONE).
