# T-MIG-P4-08 — Query filtrate e pagina risultati in Rust

18 novembre 2026. Autore Claude Code, revisore Codex. Baseline `integration`
`42aa890`. Stato: **IN_REVIEW** — verifiche locali PASS, review Codex,
CI remota e prove Windows non ancora eseguite. G-MIG-4 resta aperto.

Specifica: `docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P4-08.md`
(accettazione: parità filtri Slint, nessuna query da JS su 1M errori, limite
payload rispettato). Contratto: `docs/migration/ipc-v1/PAGING.md` e
`paging_limits.json` (IPC v1, ratifica 14/11/2026). Prerequisito P4-07 DONE
(`3a38ad2`). [Ricevuta comandi e digest](T-MIG-P4-08-checks.json).

## Cosa cambia

**Operazioni IPC.** Tre nuove operazioni registrate nel dispatcher nativo,
con DTO esatti decodificati prima di qualsiasi I/O e dopo `app_info`:

| Operazione | Richiesta | Risposta |
|---|---|---|
| `results_query` | `run` (RunRef), `filters` (Filter DTO) | `run`, `query_ref`, `state` `ready`/`building`, `total_matches`, `total_diagnostics` (testo u64 o `null`) |
| `results_query_state` | `query_ref` | stessa forma, `state` `building`/`ready`/`failed` |
| `results_page` | `query_ref`, `offset` (u64 canonico testuale), `limit` (intero JSON 1..128) | Page DTO di PAGING.md |

I nomi delle operazioni non erano fissati dal contratto; seguono il prefisso
per dominio già usato da `validation_*` e `catalog_*`. Lo stato `building`
deriva dal requisito ratificato di costruire il candidato fuori dall'event
loop: il comando Tauri sincrono gira sul main thread, quindi una scansione O(n)
non può avvenire dentro la chiamata.

**Filter DTO.** Chiavi ammesse `schemaVersion`, `code`, `field`, `record`,
`fiscal_code`, `line`, `trace_ref`; testo esatto, case-sensitive, senza trim;
vuoto/assente = nessuna restrizione. `line` riusa `ReportFilter::parse_line`
(Slint): intero positivo, zeri iniziali accettati, `null`/vuoto senza filtro.
`trace_ref` è l'handle opaco `trace_{i}` dello scope catturato, lo stesso
schema già emesso dal progresso nativo; un handle sconosciuto è
`IPC_STALE_CONTEXT`.

**Esecuzione (`apps/lintra-desktop/src-tauri/src/results.rs`).**
- Query senza filtri: riusa l'indice verificato alla pubblicazione del report
  (P4-07), pronta subito, nessuna riscansione.
- Query filtrata: un worker Rust riapre lo spool privato posseduto dal
  `Report` e lo scansiona con `JsonlReportIndex::scan`, che ricontrolla LF,
  diagnostica, scope, conteggio e istogramma. Il candidato viene pubblicato
  solo quando una chiamata successiva lo trova terminato ed è ancora
  l'intenzione corrente. Fallimento, cancellazione o sostituzione non
  sostituiscono la query corrente; il ref fallito risponde `failed`.
  Una nuova query annulla e attende (join) il candidato superato.
- Pagina: legge solo le righe richieste dall'indice (cache FIFO 128 della
  libreria), ricontrolla ogni riga contro il filtro immutabile (spool cambiato
  ⇒ errore tecnico), conia un `diagnostic_ref` casuale per riga, serializza
  l'intero frame entro 1 MiB **prima** di registrare token e `page_generation`.
  Registro limitato alle ultime 2 pagine (P4-09 lo userà per il dettaglio).
  `offset == total_matches` ⇒ pagina vuota; oltre ⇒ `IPC_INVALID_MESSAGE`,
  nessun clamp.
- Row DTO: `diagnostic_ref`, `ordinal`, `line`, `trace_ref`, preview di
  `code` e nome campo (massimo 256 byte UTF-8 su confine di carattere,
  `complete` esplicito). Mai `actual`, `raw_hex`, contesto, identità record,
  messaggio o percorsi.
- Un nuovo report pubblicato rende stale tutte le query del run precedente;
  la chiusura della finestra annulla il candidato in costruzione.

**Report (`jobs.rs`).** `Report` conserva la summary catturata, espone
`publish` (unico costruttore, usato dal worker di validazione), `open`,
`traces`, `with_index`. L'indice condiviso è in `Mutex` solo per la mutabilità
interna della cache attraverso `Arc`; tutte le chiamate passano dal gate IPC
serializzato. Ordine di drop invariato: reader prima della `TempDir`.

**Libreria (`src/report/index.rs`).** `ReportFilter` acquisisce
`trace: Option<TraceId>` (default `None`) e `matches` diventa pubblico per la
riverifica della riga. Slint imposta `trace: None` (`ui/src/results/controller.rs`):
i cinque filtri e il loro comportamento restano identici. Il test esistente
`ui/tests/results.rs` riceve solo il nuovo campo nel costrutto esaustivo.

**Client TypeScript (`apps/lintra-desktop/src/ipc.ts`).** `resultsQuery`,
`resultsQueryState`, `resultsPage` con validazione strict: chiavi esatte,
run/query_ref coerenti, contatori u64 testuali, ordinali consecutivi,
`next_offset`/`has_more`/totali coerenti, preview entro 256 byte. Nessun
filtro, ordinamento o indice lato JS.

## Cosa NON cambia

Motore, DSL, writer JSONL, golden, storage, CLI, test di accettazione,
manifest e lockfile. Nessuna dipendenza nuova (`getrandom` già presente).
Dettaglio/field guide (P4-09), export (P4-10), Full (P4-11/12) e UI React
(P6) non sono implementati. Il limite `i32::MAX` di offset della libreria
resta un errore tecnico esplicito, non un troncamento.

## Verifiche

Tutti i comandi su Linux locale, sorgenti del working tree sopra `42aa890`.

| Comando | Esito |
|---|---|
| `cargo test … --features tauri/custom-protocol -j 2 results::` (release) | 9 PASS / exit 0, 4,97 s |
| `cargo test --manifest-path apps/lintra-desktop/src-tauri/Cargo.toml --locked --offline --release --features tauri/custom-protocol -j 2` | 43 PASS, 1 ignored (GTK preesistente) / 0 |
| `cargo clippy` nativo `--release --all-targets … -D warnings` | PASS / 0 |
| `cargo test --workspace --locked --offline -j 2` | 1.112 PASS, 9 ignored / 0 |
| `cargo test --workspace --all-features --locked --offline -j 2` | 1.112 PASS, 9 ignored / 0 |
| `cargo clippy --workspace --all-targets [--all-features] … -D warnings` | PASS / 0 entrambi |
| `cargo fmt --all -- --check` e fmt del manifest Tauri | PASS / 0 |
| `npm --prefix apps/lintra-desktop run lint` / `test` / `run build` | PASS / 0; 31 test Node |
| `tests/native_smoke.py` sul binario release (Wayland) | `NATIVE_DESKTOP_SMOKE_PASS`, exit 0 |

Primo giro: fmt, un `write!` con newline (Clippy) e il costrutto esaustivo in
`ui/tests/results.rs` fallivano; corretti e tutti i gate rieseguiti. Un primo
tentativo dei test si era bloccato per un difetto del solo test (candidato
simulato che ignorava il token mentre il sender restava vivo); corretto
facendo osservare la cancellazione come la scansione reale.

**Copertura dei test nuovi.** Decodifica esatta e rifiuti (chiavi extra,
tipi, `line` 0/negativa/spazi/overflow/numero, `trace_ref` malformato,
offset non canonico, limit 0/129/stringa/bool/null); query vuota pronta senza
decode; pagine 128/128/44 ordinate, senza duplicati né buchi, registro ≤ 2
pagine, chiavi esatte e assenza di campi/percorsi vietati; filtri esatti,
case-sensitive, AND, `trace_ref`, `line` con zeri; candidato in costruzione
non paginabile, fallimento e successo deterministici con worker trattenuto,
sostituzione con join, cancel su chiusura, corruzione reale dello spool ⇒
`failed` con query precedente intatta; riga non più coerente con il filtro ⇒
errore tecnico senza avanzare la generazione; binding a run e reset su nuovo
report con due validazioni reali; preview UTF-8 e pagina peggiore (128 righe
di caratteri di controllo) entro 1 MiB; **1.000.000 diagnostiche sintetiche**:
pagina di coda, 192 decode totali, cache ≤ 128, offset 8 MB, query filtrata
su worker con 1 corrispondenza.

**Non eseguito:** CI remota, prove Windows/MSVC, VM. Rinviati al checkpoint
del blocco P4 come per P4-01..07.

## Rischi residui

- La scansione filtrata resta O(n) per query (debito già dichiarato in
  MIGRATION_PLAN). Su 1M righe in release i test completano in pochi secondi;
  nessun benchmark p50/p95 formale (G-PERF).
- Il join di un candidato superato avviene sul main thread; è breve perché la
  scansione osserva il token fra le righe, ma non è misurato su Windows.
- I nomi di operazione e lo stato `building`/`failed` sono scelte di questa
  task sul contratto ratificato: da confermare in review.

Prossima task tecnicamente disponibile dopo integrazione: T-MIG-P4-09.
