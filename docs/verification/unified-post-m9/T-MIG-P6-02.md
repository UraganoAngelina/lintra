# T-MIG-P6-02 — Cinque filtri paritari e query stale safe

Data 24–25 novembre 2026. Stato: **IN_REVIEW** (non DONE). Prerequisito
T-MIG-P6-01 DONE (`integration` `d0efe88`). Branch `work/T-MIG-P6-02` da
`d0efe88`, worktree `/home/alberto/Desktop/LINTRA/lintra-p6-02`.

Fonti: prompt atomico `MIGRAZIONE_89/T-MIG-P6-02.md`; MIGRATION_PLAN v1.1
riga P6-02 («Risultati esattamente uguali a query Rust e nessun sort/FTS
extra») e matrice di parità («5 filtri | ResultsFilter Rust | casefold/trim
automatico, selection stale»); IPC v1 `PAGING.md` (Filter DTO, query
immutabile, candidato tardivo); UI Slint `ui/ui/results.slint` e
`ui/src/results/controller.rs` (cinque campi, Invio o «Applica filtri»,
filtri azzerati a ogni nuovo report, candidato fallito senza righe vecchie).
Il `TASK_PROMPTS/T-MIG-P6-02.md` originale non è nel kit.

## Preflight

Il backend era già completo da T-MIG-P4-08: `results_query` con Filter DTO
esatto (code, field, record, fiscal_code, line, trace_ref), scansione filtrata
su worker Rust, pubblicazione atomica del candidato, `failed` per il proprio
`query_ref`, test Rust su filtri esatti/AND/case-sensitive/senza trim
(`filtered_queries_are_exact_and_combinable_and_built_off_the_main_thread`) e
sulla sostituzione (`candidates_publish_atomically_and_failures_never_replace_the_current_query`).
Il renderer apriva solo la query senza filtri. La task è quindi lato React.

## Cosa cambia

**Barra filtri** (`Results.tsx`, `FilterBar`): i cinque filtri Slint con
etichetta visibile (Codice, ID campo, ID record, Codice fiscale, Riga) tra
l’intestazione della card e la griglia, al posto della casella di ricerca
della reference: Lintra non ha ricerca libera e la task la esclude. Invio in
un campo o «Applica filtri» apre una nuova query Rust; «Azzera» svuota i campi
e, se c’erano filtri applicati, torna alla query completa. I valori partono
come digitati: niente trim, maiuscole/minuscole intatte. Riga segue la
grammatica di `ResultsFilter::parse_line` (cifre ASCII, u64 positivo, zeri
iniziali ammessi): un valore diverso è segnalato sotto il campo
(`role="alert"`, `aria-invalid`) e nessuna query parte; Rust resta comunque il
giudice e rifiuta da solo. Una riga di aiuto dice che la corrispondenza è
esatta su tutti i filtri compilati. Nessun ordinamento, ricerca libera,
chip di severità o filtro per tracciato (sesto filtro del DTO, non presente in
Slint).

**Stati con filtri**: «Applicazione dei filtri…» mentre Rust costruisce
l’indice; «Non è stato possibile applicare i filtri: nessuna riga è mostrata.»
con Riprova se il candidato fallisce; «Nessuna diagnostica corrisponde ai
filtri.» se il risultato è vuoto (distinto da «Il report non contiene
diagnostiche.»); nell’intestazione «N corrispondenze su M diagnostiche» con i
totali di Rust. Riepilogo e grafici restano quelli dell’intero report.

**Query stale safe** (`useResults`, `openQuery`): i filtri applicati sono
legati al RunRef del report; un nuovo report riparte senza filtri, come in
Slint. Ogni applicazione apre una nuova query e spegne la precedente: la
risposta di `results_query`, lo stato in polling, la prima pagina o un errore
che arrivano dopo il cambio vengono scartati (`live()` a ogni passo, timer di
polling cancellato). Mentre i filtri si applicano non si vedono righe: lo
stato passa a `building`, quindi nessuna riga vecchia appare sotto filtri
nuovi. Una pagina tardiva della query precedente non entra più nella cache
delle due pagine (prima poteva sostituire la cache della query nuova, senza
effetti visibili perché `cached()` confronta già il `query_ref`). L’apertura
della query è estratta in `openQuery` per poterla provare senza DOM.

**Stile** (`style.css`): `.grid-filters` a griglia `auto-fit` (minimo
9,5 rem per campo) che va a capo nelle finestre strette; sotto i campi un
piè di riga a tutta larghezza con aiuto e azioni. Prima le azioni stavano in
una cella di una colonna: i due bottoni non ci entravano e, allineati a
destra, debordavano sul campo Riga (visto nella prima cattura nativa).
Errore e aiuto usano i token esistenti.

**Rust** (`results.rs`, solo test): il DTO con le cinque chiavi vuote, che il
renderer ora manda sempre, è la stessa query non filtrata (pronta subito
dall’indice di pubblicazione, nessuna riga decodificata, nessun candidato).

**Catture native** (`native_capture.py`): sei scenari filtri; gli scenari
«pagina 2» e «scorrimento» seguono il nuovo ordine di Tab (sette fermate in
più: cinque campi, Azzera, Applica filtri). Il testo digitato ammette ora
maiuscole e `_`. Su Hyprland con layout `us,it` la keysym `minus` viene
risolta sul layout italiano (tasto fisico di `/`) mentre la finestra legge
`us`, quindi Shift+`minus` arrivava come `?`; lo script manda il keycode
fisico `code:20` (prova su una `Gtk.Entry`: `minus` → `/`, Shift+`code:20`
→ `_`). Due passate di cattura con questo difetto sono state scartate.

## Cosa NON cambia

Core, CLI, golden JSONL, contratto IPC v1 (nessuna chiave o operazione nuova),
semantica dei filtri e ordine canonico in Rust, detail P4-09, export, SQLite,
dipendenze. Nessun ordinamento, filtro o conteggio calcolato nel renderer.

## Acceptance

| Requisito | Prova |
| --- | --- |
| Risultati uguali alla query Rust | il renderer mostra solo pagine e totali di `results_query`/`results_page`; Rust: test filtri esatti/AND esistente + DTO vuoto; `filters.test.mjs`: DTO inviato identico a quanto digitato (spazi e zeri iniziali compresi) |
| Nessun sort/FTS extra | `FilterBar` senza ricerca, ordinamento o chip (`filters.test.mjs`); nessun codice di ordinamento o filtro sulle righe |
| No casefold/trim automatico | `filters.test.mjs` (spazio come restrizione, DTO esatto); nativo `results-filter-code-nomatch` (codice minuscolo: nessuna corrispondenza) |
| Selezione/query stale | `filters.test.mjs`: risposte tardive di query, stato, prima pagina ed errore scartate; prova di mutazione su ciascun controllo `live()` (vedi Test) |
| Parità Slint | cinque campi ed etichette, Invio/Applica, Riga validata prima dell’invio, filtri azzerati per un nuovo report, nessuna riga dopo un candidato fallito |

## Test

| Verifica | Esito |
| --- | --- |
| `npm run lint` / `npm test` / `npm run build` | PASS / 144 PASS (6 nuovi in `filters.test.mjs`) / PASS (dopo il fix del piè di riga) |
| Prove di mutazione su `openQuery` | rimuovendo a turno il controllo `live()` su risposta della query, stato, prima pagina ed errore, `filters.test.mjs` fallisce (1 FAIL ciascuna); con il codice reale PASS. Togliere solo `clearTimeout` non fa fallire: è pulizia, la garanzia è `live()` |
| comandi Linux di `desktop.yml`: `cargo fmt -- --check` / `cargo test --locked --release --target x86_64-unknown-linux-gnu --features tauri/custom-protocol -j 2` / `cargo clippy … --all-targets … -- -D warnings` / `cargo build …` | PASS (0) / 85 passed, 0 failed, 1 ignored (0) / PASS (0) / PASS (0) |
| `native_smoke.py` sul binario del target CI (sessione Hyprland reale: `xvfb-run` non installato) | PASS / 0, `bebb2a483d6ca232a4cc495e8d6f9b1c9e10250cbff9067f15ac9ef228854e0e` |
| `npm run tauri -- build --no-bundle` + `native_smoke.py` | PASS / 0, binario `f647e688bcdaf51a11145ae3206b799fc64ee4dfbd14e70707626bd88fe58bdf` |
| `python3 tests/native_capture.py <bin> <out>` | PASS 57/57 + chiusura durante il run (exit 0, 0,02 s, nessuno spool residuo) su `f647e688…8bdf` ([receipt](T-MIG-P6-02-native/receipt.json)); 51 scenari precedenti + 6 filtri |
| workspace `ci.yml` | NOT_RUN: core, CLI e crate `lintra` non toccati |
| `verify_paging.py` | NOT_RUN: contratto e reference invariati |
| Windows/WebView2 e CI remota | NOT_RUN (al checkpoint del blocco P6) |

Esiti dei sei scenari filtri (`T-MIG-P6-02-native/`): Riga 42 → «1
corrispondenza su 100.000 diagnostiche» e la sola riga 42 (1366×768 e
640×480); `E_RECORD_LENGTH` → «100.000 corrispondenze su 100.000» con le
righe; `e_record_length` → «0 corrispondenze» (nessun casefold); Riga 0 →
campo in errore e nessuna query (l’intestazione resta «Righe 1–128 di
100.000»; il messaggio sotto il campo è fuori inquadratura e non è
verificato da un test DOM: `filters.test.mjs` copre solo la grammatica di
`validLine`); Azzera → di nuovo «Righe 1–128 di 100.000».

## Rischi residui

- I filtri sono applicati da Rust con una scansione O(n) per query (debito
  dichiarato nel piano §10); su report molto grandi «Applicazione dei
  filtri…» dura quanto la scansione. Applicare di nuovo durante la scansione
  sostituisce il candidato, che Rust annulla.
- Riepilogo e grafici non seguono i filtri: descrivono l’intero report.
  L’intestazione della griglia dà il conteggio filtrato.
- Filtro per tracciato (presente nel DTO), dettaglio e selezione sono fuori
  da questa task (P6-03).

Prossima task tecnicamente disponibile dopo l’integrazione: T-MIG-P6-03.
