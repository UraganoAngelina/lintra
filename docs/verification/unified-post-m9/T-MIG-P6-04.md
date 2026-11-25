# T-MIG-P6-04 — Field guide e posizione/schema

Data 25 novembre 2026. Stato: **IN_REVIEW** (non DONE). Prerequisito
T-MIG-P6-03 DONE (`integration` `61ebb06`). Branch `work/T-MIG-P6-04` da
`61ebb06`, worktree `/home/alberto/Desktop/LINTRA/lintra-p6-04`.

Fonti: prompt atomico `MIGRAZIONE_89/T-MIG-P6-04.md`; MIGRATION_PLAN v1.1
riga P6-04 («Test snapshot e oracle su schemi sintetici; nessuna regola
ricostruita in TS») e matrice di parità §10.2 («Domain/field guide |
compiled schema + captured reference | off-by-one, rounding >u128,
reference cambiata dopo run»); IPC v1 `DETAIL.md` (guide e copertura del
campo dal `CompiledFlowSchema` catturato, `field_position` byte 1-based
inclusivi); Slint `ui/src/results/mod.rs` (posizione «start–end») e
`ui/ui/results.slint` («Campo / posizione in byte»); riferimento UI
`lintra_tauri_ui.png` (riquadro «Regola applicata» nel pannello). Il
`TASK_PROMPTS/T-MIG-P6-04.md` originale non è nel kit.

## Preflight

Il provider Rust (T-MIG-P4-09) produceva già, per una diagnostica di campo,
la componente `guide` (`lintra::field_format_guide` sullo schema compilato
catturato con il report) e `field_position` (start/end come testo u64,
`one_based_inclusive_bytes`); il client IPC le valida. Il test Rust
`selected_detail_matches_the_canonical_diagnostic_and_captured_schema` è
l'oracolo: per ogni diagnostica dello schema sintetico confronta la guida
letta via `results_text` con `field_format_guide` e la posizione con quella
canonica. Il pannello di P6-03 non mostrava né guida né posizione. La task
è lato renderer, più uno snapshot Rust.

## Cosa cambia

**Pannello** (`Results.tsx`, `DetailPanel`):
- «Posizione in byte» tra le coppie, nel formato Slint `start–end` con i
  numeri di Rust raggruppati come testo (nessun `Number`, nessuna lunghezza
  calcolata: la guida di Rust la dice già); «—» per le diagnostiche senza
  campo.
- «Guida al campo»: riquadro con la barra d'accento, come «Regola
  applicata» del riferimento, con il testo di `field_format_guide`.
  L'anteprima oltre 256 byte è marcata e si legge per intero con «Leggi il
  testo completo» (finestre di `results_text`, P6-03). Nessuna guida per le
  diagnostiche senza campo.
- Nessun termine dello schema nel renderer: guida, tipo, allineamento,
  riempimento e codifica sono testo di Rust.

**Stile** (`style.css`): `.detail-guide`.

**Rust** (`results/detail.rs`, solo test):
`field_guides_and_positions_of_the_synthetic_schema_are_pinned` fissa il
testo di guida e la posizione di F1 e F2 dello schema sintetico incorporato,
letti via `results_select`/`results_text` come fa il pannello. L'oracolo
esistente prova che sono uguali a `field_format_guide`; lo snapshot rende
visibile e soggetto a review ogni cambio di formulazione.

**Test Node** (`tests/field-guide.test.mjs`, nuovo): snapshot del markup
del riquadro per una guida completa e per le due guide sintetiche reali
(anteprima di Rust troncata a 256 byte su confine UTF-8, marcata, con lettura
completa); posizione u64 massima resa senza perdita; diagnostica senza campo
senza posizione né guida; assenza del lessico della guida nei sorgenti del
renderer. In `detail-panel.test.mjs` il controllo «nessuna posizione né
guida», che era il confine di P6-03, diventa «nessun dominio» (P6-05/06).

**Catture native** (`native_capture.py`): input sintetico `fields.txt`
(`002B  `, `00XA  `: F1 non ammesso, F2 non ammesso, F1 non numerico) e
quattro scenari: dettaglio con posizione e guida, guida completa a 1366 e
640, contesto del record con valori.

## Cosa NON cambia

Core (`field_format_guide` e motore), CLI, golden JSONL, provider e
contratto IPC v1, codice runtime Rust del desktop, export, SQLite,
dipendenze. Domini e fonti (`source_count`, `results_sources`,
`results_domain`) restano a T-MIG-P6-05/06.

## Acceptance

| Requisito | Prova |
| --- | --- |
| Test snapshot su schemi sintetici | Rust: `field_guides_and_positions_of_the_synthetic_schema_are_pinned`; Node: `field-guide.test.mjs` (markup del riquadro sui testi reali) |
| Oracle su schemi sintetici | Rust: `selected_detail_matches_the_canonical_diagnostic_and_captured_schema` (guida = `field_format_guide`, posizione = diagnostica canonica), invariato e PASS |
| Nessuna regola ricostruita in TS | guida e posizione mostrate come le manda Rust; test sul lessico assente nel renderer; nessuna lunghezza o conversione numerica |
| Parità Slint | posizione `start–end`, nome del campo, guida dallo schema del report |
| Off-by-one / >u128 (matrice) | posizione u64 massima resa esatta (test); posizioni 1–3 e 4–6 nello snapshot Rust e nelle catture |

## Test

| Verifica | Esito |
| --- | --- |
| `npm run lint` / `npm test` / `npm run build` | PASS / 158 PASS (5 nuovi in `field-guide.test.mjs`) / PASS |
| `cargo test … field_guides_and_positions` | PASS (prima esecuzione con valori segnaposto: FAIL atteso, da cui i testi reali) |
| comandi Linux di `desktop.yml`: `cargo fmt -- --check` / `cargo test --locked --release --target x86_64-unknown-linux-gnu --features tauri/custom-protocol -j 2` / `cargo clippy … --all-targets … -- -D warnings` / `cargo build …` | PASS (0) / 86 passed, 0 failed, 1 ignored (0) / PASS (0) / PASS (0) |
| `native_smoke.py` sul binario del target CI (sessione Hyprland reale: `xvfb-run` non installato) | PASS / 0, `d3a1b438e667f023af9531bf93c195cefab48441effd3f574e152f5569548184` |
| `npm run tauri -- build --no-bundle` + `native_smoke.py` | PASS / 0, binario `c557112b0eec8a80ea2cd299ed0f828fb3fe21007f6e759988ba47e3bc262121` |
| `python3 tests/native_capture.py <bin> <out>` | PASS 65/65 + chiusura durante il run (exit 0, 0,02 s, nessuno spool residuo) su `c557112b…2121` ([receipt](T-MIG-P6-04-native/receipt.json)); 61 scenari precedenti + 4 guida |

Prima esecuzione di `cargo test` completa: 85 PASS, 1 FAIL
(`one_million_diagnostics_stay_in_rust_and_pages_stay_bounded`, `No space
left on device` in scrittura del JSONL di prova). Causa d'ambiente: `/tmp`
(tmpfs 7,7 GB) pieno di 144 spool `lintra-desktop-run-*` lasciati dalle
campagne di cattura di oggi, che chiudono ogni scenario con SIGTERM.
Rimossi (nessun processo attivo, solo dati sintetici), il test e la suite
passano. Il comportamento degli spool al SIGTERM non è cambiato da questa
task.
| workspace `ci.yml`, `verify_detail.py` | NOT_RUN: core, contratto e provider non toccati |
| Windows/WebView2 e CI remota | NOT_RUN (al checkpoint del blocco P6) |

Esiti degli scenari (`T-MIG-P6-04-native/`): `results-guide` E_INVALID_DOMAIN
su F1, «Posizione in byte 1–3», guida marcata «…» con «Leggi il testo
completo»; `results-guide-full` (1366 e 640) guida intera fino a «Fine del
testo.»; `results-guide-context` «Nessun elemento di contesto.» (risposta di
Rust per questa diagnostica). Un'anteprima prima della campagna aveva
mostrato che il primo «Leggi il testo completo» del pannello è quello del
messaggio (anch'esso oltre 256 byte): ordine dei Tab corretto.

## Rischi residui

- Il messaggio delle diagnostiche di dominio contiene già la guida (testo
  del motore): nel pannello la stessa guida compare nel messaggio e nel
  riquadro. È il testo di Rust; non viene accorciato né deduplicato nel
  renderer.
- Le guide sintetiche superano l'anteprima di 256 byte: il testo intero
  richiede un clic.
- Lo script di cattura termina l'app con SIGTERM e lascia uno spool per
  ogni analisi in corso in `/tmp`; su campagne ripetute riempie il tmpfs.
  È un comportamento preesistente del banco di prova (non dell'uscita
  normale, coperta da «chiusura durante il run»); fuori scope qui.

Prossima task tecnicamente disponibile dopo l'integrazione: T-MIG-P6-05.
