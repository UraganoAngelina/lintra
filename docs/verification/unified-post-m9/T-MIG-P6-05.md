# T-MIG-P6-05 — Dominio ammesso statico e range

Data 25 novembre 2026. Stato: **IN_REVIEW** (non DONE). Prerequisito
T-MIG-P6-04 DONE (`integration` `315b641`). Branch `work/T-MIG-P6-05` da
`315b641`, worktree `/home/alberto/Desktop/LINTRA/lintra-p6-05`.

Fonti: prompt atomico `MIGRAZIONE_89/T-MIG-P6-05.md`; MIGRATION_PLAN v1.1
riga P6-05 («Parità field_guide; JSON string per big integers; UI non
calcola domini»), riga P6-06 per il confine (reference) e matrice §10.2
(«Domain/field guide … off-by-one, rounding >u128»); IPC v1 `DETAIL.md`
(«Domini, provenienza e disponibilità»: una fonte alla volta, kind
esplicito, dichiarazioni condizionali con RuleId e guardia,
`declared_domain_page` max 64, IntegerLiteral oltre u128, regex
`not_enumerable`); Slint `ui/src/results/controller.rs` (`show_domain_page`:
«Valori dichiarati · dichiarazione N · campo / regola condizionale X»,
«Fallback autorizzati», «Primi valori»/«Valori successivi», testo per i
vincoli non enumerabili). Il `TASK_PROMPTS/T-MIG-P6-05.md` originale non è
nel kit.

## Preflight

Provider e client già completi da T-MIG-P4-09: `results_sources` (pagine
di 64 fonti con kind, regola, guardia, dataset e `comparison` =
`constraint_comparison_guide`) e `results_domain` (per le fonti statiche
risposta immediata da `declared_domain_page`, cursore nativo opaco;
regex/non enumerabili `not_enumerable`). Test Rust esistenti:
`domains_page_declarations_and_the_verified_reference_snapshot`,
`sources_are_paged_without_a_count_limit_and_carry_native_guards`. Il
pannello di P6-04 non mostrava i domini. La task è lato renderer; nessuna
modifica Rust.

## Cosa cambia

**«Dominio ammesso»** (`Results.tsx`, `DomainSection`, `DomainSourceItem`,
`DomainValues`), solo per le diagnostiche di campo, su richiesta («Mostra
il dominio», come il contesto):
- Dichiarazioni del campo nell'ordine di Rust, a pagine di 64 («Dichiarazioni
  successive», «Prime dichiarazioni»). Per ognuna: tipo con le parole di
  Slint («Valori dichiarati», «Intervallo dichiarato», «Fallback
  autorizzati», «Espressione regolare», «Vincolo non enumerabile»),
  numero, origine («campo» oppure «regola condizionale <RuleId>» con la
  guardia «Si applica quando: …», mai come vincolo incondizionato) e il
  testo di confronto di Rust.
- Valori di una fonte alla volta: «Consulta i valori», pagine di al massimo
  64 letterali di Rust (`domain_literal`: virgolette, `␠`, zeri iniziali),
  «Valori successivi» con il cursore di Rust, «Primi valori». Interi di
  range come testo: nessun parsing, nessun calcolo, nessun ordinamento o
  filtro nel renderer. Un letterale oltre l'anteprima si legge per intero.
- Nessun elenco inventato: `not_enumerable` → «Nessun elenco finito: il
  vincolo non è enumerabile…»; `available_empty` → «Nessun valore in
  questa pagina del dominio.»; errore con Riprova.
- Fonti reference: elencate con dataset e versione, senza consultazione
  dei valori (T-MIG-P6-06: stati di evidenza e digest).
- Fuoco: il bottone premuto sparisce con la risposta, quindi il fuoco passa
  al titolo «Dominio ammesso» (dichiarazioni lette) o alla pagina di valori
  (valori letti); senza questo il fuoco finiva sul `body`.

**Stile** (`style.css`): blocchi delle dichiarazioni e lista dei valori.

**Test** (`tests/domain.test.mjs`, nuovo): resa delle dichiarazioni (tipo,
origine, confronto), regola condizionale con guardia, letterali esatti
compreso un intero u128::MAX + 1, anteprima incompleta leggibile per
intero, paginazione a cursore, `not_enumerable`/vuoto/errore, reference
senza valori, sezione solo per diagnostiche di campo, nessun parsing o
ordinamento nel codice dei domini. In `detail-panel.test.mjs` il controllo
di confine «nessun dominio» diventa «dominio solo su richiesta, nessun
valore prima».

**Catture native** (`native_capture.py`): tre scenari su `fields.txt`
(dichiarazioni di F1, valori di F2 a 1366 e 640).

**Banco di cattura, modifica autorizzata** (commit separato `4b2b62f`,
autorizzazione dell'autore del 25 novembre 2026): ogni scenario di
`native_capture.py` dà all'app un `TMPDIR` privato nella propria directory
temporanea, rimossa a fine scenario, come già faceva lo scenario di
chiusura durante il run. Prima gli spool delle analisi aperte al SIGTERM di
fine scenario restavano nel `/tmp` di sistema (tmpfs, quindi RAM): in P6-04
hanno riempito `/tmp` (`StorageFull` in `cargo test`); in P6-05 la prima
campagna completa è stata fermata da Claude Code per memoria esaurita (66
immagini, nessun receipt: scartata e cancellata). Verifica: durante uno
scenario lo spool nasce in `/tmp/lintra-desktop-capture-…/tmp/`, a fine
scenario non ne resta nessuno; a fine campagna 0 spool nel `/tmp` di
sistema, `/tmp` invariato al 42%.

## Cosa NON cambia

Core (`declared_domain_page`, `constraint_comparison_guide`), CLI, golden,
provider e contratto IPC v1, Rust del desktop, export, SQLite, dipendenze.

## Acceptance

| Requisito | Prova |
| --- | --- |
| Parità field_guide | confronto di ogni fonte = `constraint_comparison_guide` di Rust mostrato com'è; tipi e origini con le parole di Slint; guardia delle regole condizionali (test) |
| JSON string per big integers | valori di range come testo Rust; u128::MAX + 1 resa esatta (test); il client IPC li tratta già come stringhe |
| UI non calcola domini | valori solo da `results_domain`, cursore di Rust; test sul codice dei domini senza `parseInt`/`Number(`/ordinamenti/filtri |
| Nessun elenco inventato | `not_enumerable` e `available_empty` espliciti (test) |

## Test

| Verifica | Esito |
| --- | --- |
| `npm run lint` / `npm test` / `npm run build` | PASS / 166 PASS (8 nuovi in `domain.test.mjs`) / PASS |
| comandi Linux di `desktop.yml`: `cargo fmt -- --check` / `cargo test --locked --release --target x86_64-unknown-linux-gnu --features tauri/custom-protocol -j 2` / `cargo clippy … --all-targets … -- -D warnings` / `cargo build …` | PASS (0) / 86 passed, 0 failed, 1 ignored (0) / PASS (0) / PASS (0) |
| `native_smoke.py` sul binario del target CI (sessione Hyprland reale: `xvfb-run` non installato) | PASS / 0, `af82a1148640cd2a4f81957b387e4c5472279387f086b7cf961ad395671f310e` |
| `npm run tauri -- build --no-bundle` + `native_smoke.py` | PASS / 0, binario `2526895a21622766dcedacbabce8e0d02ec7f5c390731f2c9020dd253c03c801` |
| `python3 tests/native_capture.py <bin> <out>` | PASS 68/68 + chiusura durante il run (exit 0, 0,02 s, nessuno spool residuo) su `2526895a…c801` ([receipt](T-MIG-P6-05-native/receipt.json)), con il banco corretto; 65 scenari precedenti + 3 dominio. Prima campagna interrotta per memoria (vedi sopra) |
| workspace `ci.yml`, `verify_detail.py` | NOT_RUN: core, contratto e provider non toccati |
| Windows/WebView2 e CI remota | NOT_RUN (al checkpoint del blocco P6) |

Esiti degli scenari (`T-MIG-P6-05-native/`): `results-domain` F1, una
dichiarazione «Valori dichiarati · dichiarazione 1 · campo» con il testo di
confronto e «Consulta i valori», fuoco sul titolo «Dominio ammesso»;
`results-domain-values` (1366 e 640) F2, valori `"A"` `"É"` come letterali
di Rust, fuoco sulla pagina di valori.

## Rischi residui

- Il testo di confronto di una fonte arriva solo come anteprima (nessun
  handle nel DTO): oltre 256 byte resta marcato «…» senza lettura completa.
- Lo schema sintetico delle catture non ha range né regex: questi rami sono
  provati dai test Node e dai test Rust esistenti, non in nativo.
- «Mostra il contesto» di P6-03 ha lo stesso difetto di fuoco corretto qui
  per il dominio (il bottone sparisce con la risposta); non toccato in
  questa task.

Prossima task tecnicamente disponibile dopo l'integrazione: T-MIG-P6-06.
