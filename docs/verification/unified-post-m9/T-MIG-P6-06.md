# T-MIG-P6-06 — Dominio reference/evidenza provider

Data 25 novembre 2026. Stato: **DONE**. Review Codex round 1 APPROVED su
`dfadf53` senza rilievi (`lintra-review-handoffs/T-MIG-P6-06-CODEX-REVIEW.md`).
Integrazione autorizzata dall'autore per fast-forward di `integration`, senza
push. CI remota e Windows/WebView2 NOT_RUN, al checkpoint del blocco P6;
G-MIG-6 NOT_VERIFIED.
Prerequisito
T-MIG-P6-05 DONE (`integration` `1ad7140`). Branch `work/T-MIG-P6-06` da
`1ad7140`, worktree `/home/alberto/Desktop/LINTRA/lintra-p6-06`.

Fonti: prompt atomico `MIGRAZIONE_89/T-MIG-P6-06.md`; MIGRATION_PLAN v1.1
riga P6-06 («Reference snapshot consistent; no “valid” when
missing/unverified data») e matrice §10.2 («reference cambiata dopo run»);
IPC v1 `DETAIL.md` (reference da `PreparedReferenceEvidence` catturata,
digest dall'evidenza nativa, tabella degli stati, lettura solo per Verified
con digest uguale, mismatch errore tecnico, worker Rust); provider
`results/detail.rs` (`domain`, `domain_state`); client `ipc.ts`
(`resultsDomain`, `resultsDomainState`); Slint `ui/src/results/controller.rs`
(lettura reference su thread, testi «integrità non verificata…», «sorgente
non disponibile…», «Il dataset verificato non contiene codici.», SHA-256
mostrato). Il `TASK_PROMPTS/T-MIG-P6-06.md` originale non è nel kit.

## Preflight

Il provider (T-MIG-P4-09) legge il dataset reference solo dall'evidenza
catturata con il run: `not_captured` se manca, `degraded`/`unavailable`
dallo stato dell'evidenza, lettura solo per Verified con digest osservato
uguale all'atteso (altrimenti errore tecnico), su worker con risposta
`loading` + `read_ref` da interrogare con `results_domain_state`; poi
`available`/`available_empty`, `unsupported` o `cancelled`. Test Rust
esistenti: `domains_page_declarations_and_the_verified_reference_snapshot`,
`unverified_reference_evidence_is_never_an_empty_valid_domain`. Il renderer
di P6-05 elencava le fonti reference senza consultarle. La task è lato
renderer; nessuna modifica Rust.

## Cosa cambia

**Lettura** (`Results.tsx`, `readDomain`): `results_domain`, e se Rust
risponde `loading` interroga `results_domain_state` ogni 200 ms finché la
pagina o lo stato è pubblicato. Ogni risposta passa da `live()`: una lettura
lasciata (altra fonte, altra pagina di fonti, altra selezione) smette di
interrogare e non arriva alla vista. Durante la lettura: «Lettura della
pagina dal dataset usato nell'analisi…». Tutte le fonti si consultano ora
allo stesso modo, reference comprese.

**Evidenza e stati** (`DomainValues`, `Evidence`):
- `available`: «Dataset verificato usato nell'analisi · SHA-256 <digest>»
  (il digest di Rust, uguale all'atteso per contratto) e i valori come
  letterali di Rust; `available_empty` alla prima pagina «Il dataset
  verificato non contiene codici.», alle successive «Nessun valore in
  questa pagina del dominio.».
- `not_captured`, `degraded`, `unavailable`, `unsupported`, `cancelled`:
  avviso (`role="note"`) con le parole di Slint, nessun valore, nessuna
  dicitura «verificato»; i digest presenti sono mostrati come «SHA-256
  atteso … · osservato …» (o «non disponibile»), mai come verifica.

**Stile** (`style.css`): riga dell'evidenza e avviso.

**Test** (`tests/reference.test.mjs`, nuovo): polling fino alla pagina con
le richieste esatte (`results_domain`, poi `results_domain_state` con il
`read_ref`); lettura lasciata che smette di interrogare; errore; evidenza
verificata con digest e valori; dataset vuoto; ciascuno dei cinque stati
senza valori né «verificato», digest atteso/osservato. Prove di mutazione:
togliere il `live()` del polling, mostrare «verificato» per ogni stato o
ignorare lo stato fanno fallire i test. In `domain.test.mjs` il controllo
di confine di P6-05 («reference senza consultazione») diventa «reference
consultabile».

**Catture native** (`native_capture.py`): bundle esterno LocalOnly
(`reference_bundle`): schema SYNTHETIC incorporato con F2 vincolato al
dataset `union-A` (chiavi `AA`, `ZZ`) e ai propri valori ammessi, digest
reale (`verified_reference`) o errato (`degraded_reference`). Il parametro
`full` degli scenari accetta ora anche un costruttore di bundle. Tre
scenari: evidenza verificata a 1366 e 640, evidenza degradata.

## Cosa NON cambia

Core e provider (lettura reference, digest, stati), CLI, golden, contratto
IPC v1, Rust del desktop, export, SQLite, dipendenze.

## Acceptance

| Requisito | Prova |
| --- | --- |
| Reference snapshot consistent | valori solo dall'evidenza del run letta da Rust; digest dall'evidenza, mostrato com'è; test e cattura `results-reference` |
| No “valid” when missing/unverified | cinque stati senza valori né «verificato» (test, mutazioni); cattura `results-reference-degraded` (digest atteso/osservato e avviso) |
| Lettura stale safe | `readDomain` con `live()`; test e mutazione |

## Test

| Verifica | Esito |
| --- | --- |
| `npm run lint` / `npm test` / `npm run build` | PASS / 170 PASS (4 nuovi in `reference.test.mjs`) / PASS |
| Prove di mutazione su `readDomain`/`DomainValues` | 3 mutazioni, ciascuna 1 FAIL; codice reale 4/4 |
| comandi Linux di `desktop.yml`: `cargo fmt -- --check` / `cargo test --locked --release --target x86_64-unknown-linux-gnu --features tauri/custom-protocol -j 2` / `cargo clippy … --all-targets … -- -D warnings` / `cargo build …` | PASS (0) / 86 passed, 0 failed, 1 ignored (0) / PASS (0) / PASS (0) |
| `npm ci --ignore-scripts` | NOT_RUN in locale: EBADENGINE (node 26.13.1/npm 12.2.3 contro i pin 26.13.0/12.2.2); `node_modules` già allineato al lock |
| `native_smoke.py` sul binario del target CI (sessione Hyprland reale: `xvfb-run` non installato) | PASS / 0, `c404c1b9b8c89905fdfb95ff43cf44c80be1f4b6e09582b228ddc601f34b623c` |
| `npm run tauri -- build --no-bundle` + `native_smoke.py` | PASS / 0, binario `ac9f787c2e050d827295dd53bcf23f56c97f9a0ca1d878964dc3940eecca5f46` |
| `python3 tests/native_capture.py <bin> <out>` | PASS 71/71 + chiusura durante il run (exit 0, 0,02 s, nessuno spool residuo) su `ac9f787c…5f46` ([receipt](T-MIG-P6-06-native/receipt.json)); 68 scenari precedenti + 3 reference. Una campagna precedente, interrotta prima del receipt, è stata scartata e ripetuta per intero |
| workspace `ci.yml`, `verify_detail.py` | NOT_RUN: core, contratto e provider non toccati |
| Windows/WebView2 e CI remota | NOT_RUN (al checkpoint del blocco P6) |

## Rischi residui

- Gli stati `not_captured`, `unavailable`, `unsupported` e `cancelled` sono
  provati da test Node e test Rust esistenti, non in nativo (il banco
  produce solo evidenza verificata e degradata).
- Con evidenza degradata il controllo reference non produce violazioni: la
  fonte è raggiungibile solo da un'altra diagnostica dello stesso campo.
- Il polling non ha un limite di tempo: dura quanto la lettura del worker
  Rust, interrotta cambiando fonte o selezione.

Prossima task tecnicamente disponibile: T-MIG-P6-07.
