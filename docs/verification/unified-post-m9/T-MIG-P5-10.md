# T-MIG-P5-10 — React: monitor Full e stati terminali multi-pass

Data 23 novembre 2026. Stato: **DONE**. Review Codex round 1 APPROVED su
`98bdf80` (un MINOR documentale, P510-01, corretto in questa chiusura);
integrazione autorizzata dall'autore per fast-forward di `integration`, senza
push. CI remota NOT_RUN. Prerequisiti T-MIG-P5-09 e T-MIG-P5-06
DONE (`integration` `76f709d`). Branch `work/T-MIG-P5-10` da `76f709d`,
worktree `/home/alberto/Desktop/LINTRA/lintra-p5-10`.

**Scope esteso autorizzato dall'autore** (23 novembre 2026): una modifica
Rust minima all'adapter desktop (`progress.rs`, `configuration.rs`) perché il
renderer possa nominare il tracciato in corso senza decodificare un handle
opaco. Forma del DTO, operazioni IPC v1 e core invariati.

Fonti: prompt atomico («Confronto semantico col servizio Full; nessuna
presentazione di conformity falsa o percentuali inventate»); MIGRATION_PLAN
v1.1 righe P5-10 e G-MIG-5; Master Plan §6 punto 4; IPC v1 `PROGRESS.md`
(campione unico, `trace_records` per il tracciato nominato, PASS2 con
`trace_ref` null e `cross_records` propri, nessun totale/percentuale/ETA,
riconciliazione con l'esito finale), `FULL.md` (PASS1 per trace in ordine
schema, PASS2, un solo esito per il bundle, VALID+Partial non è conformità),
`LIFECYCLE.md`; dossier P4-11, P4-12, P5-05, P5-06, P5-09. Il
`TASK_PROMPTS/T-MIG-P5-10.md` originale non è nel kit.

## Preflight

- P5-09 integrata (`ebef511`, chiusura `76f709d`), P5-06 DONE.
- Prima di questa task, dopo un run Full il pannello mostrava i contatori di
  un tracciato singolo: in PASS2 «Record del tracciato 0», fase con le parole
  LocalOnly, nessun tracciato nominato.
- Il `trace_ref` del campione era `trace_{i}`, con `i` indice nel bundle dei
  soli tracciati assegnati: non coincide con gli handle dei descrittori
  (`<istanza>_config_<n>_trace_<indice schema>`) e il renderer non ha
  un'operazione per risolverlo. Decodificarlo, o contare i tracciati osservati
  con letture ogni 250 ms (un tracciato breve può non comparire mai in un
  campione), avrebbe potuto dare il nome sbagliato.

## Cosa cambia

Rust (adapter desktop, scope esteso):
- `configuration.rs`: `PreparedIntent.trace_refs`, gli handle di piano dei
  tracciati dello scope nello stesso ordine (LocalOnly: il tracciato scelto;
  Full: i tracciati con un file, in ordine di schema), calcolati in `capture`.
- `progress.rs`: `Progress::new(scope, refs)`; il `trace_ref` del campione è
  l'handle di piano del tracciato in corso. Una lista di lunghezza diversa
  dallo scope sigilla solo il canale di progresso come errore tecnico (come le
  altre incoerenze di PROGRESS.md); esito e run non cambiano.
- `jobs.rs`, `jobs/full_lifecycle.rs`, `results/detail.rs`: costruzione nei
  test e asserzioni aggiornate al nuovo valore dell'handle.

React:
- `src/run.ts`: `RunContext` (modalità e tracciati del bundle) congelato
  all'avvio e conservato in `starting`/`active`/`terminal`; `phaseText` con
  le parole Full («Controlli sui singoli tracciati», «Controlli tra
  tracciati»); `sampleTrace` risolve l'handle solo nel contesto del run (un
  handle estraneo non riceve un nome); `counterRows` mostra i contatori come
  li definisce il servizio: record del tracciato nominato solo mentre un
  tracciato è in corso, record tra tracciati da PASS2; `lastPhaseText`, solo
  per un terminale Full senza report di questo run, dichiara l'ultima fase
  comunicata dal servizio come osservazione, non come causa.
- `phaseTrace`: il nome del tracciato accompagna la fase solo in PASS1.
  Trovato nella prova nativa: dopo un annullamento in PASS1 Rust passa a
  `finalizing_report` lasciando S nel campione, e il terminale diceva
  «Scrittura del report, tracciato S». Ora dice «Scrittura del report»; il
  contatore «Record del tracciato S» resta, perché è l'ultimo conteggio reale
  di S. Regressione in `monitor.test.mjs`.
- `src/RunPanel.tsx`: avvio con il contesto (`runTraces`), fase con il nome
  del tracciato, contatori da `counterRows`, ultima fase nel terminale.
- `src/analysis.ts`: `runTraces` (tracciati del bundle in ordine di schema).
- LocalOnly: parole e contatori P5-05 invariati.
- Test: `tests/monitor.test.mjs` (8); test Rust: nuovo
  `handles_that_do_not_match_the_scope_seal_only_progress`, asserzioni su
  handle in `progress.rs`, `full_lifecycle.rs` e nel test reale di
  preparazione Full.
- `tests/native_capture.py`: quattro scenari del monitor Full.

## Cosa NON cambia

Forma e chiavi del DTO di progresso, operazioni e versioni IPC v1, core,
servizio, PASS1/PASS2, esiti, golden, CLI, lockfile, dipendenze, workflow.
`outcomeText` (mappatura degli esiti Rust) era già conforme e resta
invariata: la task la verifica sull'intera matrice. Nessuno storico per
tracciato: l'ultimo conteggio visto di un tracciato concluso può essere
inferiore al reale, quindi non si mostrano conteggi per tracciato concluso né
totali Full. Risultati, dettaglio ed export Full: T-MIG-P6-10.

## Acceptance

| Requisito | Prova |
| --- | --- |
| Confronto semantico col servizio Full | il campione è mostrato come lo definisce PROGRESS.md (tracciato nominato dall'handle Rust, PASS2 senza tracciato e con i propri record, monotonia della sequence); esiti: matrice completa status × completezza × troncamento × report (24 casi) con la sola combinazione valida+completa+non troncata come «Nessuna violazione nei controlli eseguiti», annullato e parziale mai assorbiti nel successo, report dichiarato solo se Rust lo dichiara. Nativo: run Full completato, con violazioni, annullato in PASS1 |
| Nessuna conformity falsa | nessun «conforme/valido» in nessun esito; VALID+Partial «Controlli incompleti»; troncato «Analisi parziale» |
| Nessuna percentuale inventata | barra indeterminata; nessun `%`, ETA, «totale», «di N» nel monitor; nessun conteggio per tracciato concluso |

Mutazioni: riga del tracciato mostrata anche in PASS2 → 2 FAIL; handle
estraneo nominato col primo tracciato → 1 FAIL. Sorgente ripristinato
(SHA-256 `run.ts` `54989c47…573e6d`).

## Test

| Verifica | Esito |
| --- | --- |
| `npm run lint` / `npm test` / `npm run build` | PASS / 124 PASS (116 + 8) / PASS |
| desktop `cargo fmt … -- --check` | PASS / 0 |
| desktop `cargo test … --locked --release --target x86_64-unknown-linux-gnu --features tauri/custom-protocol -j 2` | PASS / 0: 83 (82 + 1), 0 fail, 1 ignored |
| desktop `cargo clippy … --release --all-targets … -- -D warnings` | PASS / 0 |
| `npm run tauri -- build --no-bundle -- --locked -j 2` + `native_smoke.py` | PASS / 0, binario `835d526b2fed4f25007b0d13aaf544e632a82ad2f4ba51a29d44a6fc8f8548d4` |
| `python3 tests/native_capture.py <bin> <out>` | PASS 43/43, rc 0 ([receipt](T-MIG-P5-10-native/receipt.json)): 39 precedenti + 4 del monitor Full |
| `git diff --check integration` | PASS |
| `npm ci` | NOT_RUN: engine pin node 26.13.0, locale 26.13.1 |
| workspace core | NOT_RUN: nessun file core modificato |
| CI remota / Windows / WebView2 | NOT_RUN |

Prova nativa (Hyprland, controllata a vista), bundle esterno sintetico S/T,
S da 245 MB:
- `full-monitor-pass1` e `full-monitor-later`: «Controlli sui singoli
  tracciati: tracciato S», «Tracciati nell'analisi S, T», «Record del
  tracciato S» crescente (722.944, poi 2.584.576 nello stesso tipo di run),
  barra indeterminata, nessuna percentuale;
- `full-run-cancelled`: «Interrompi analisi» in PASS1: «Analisi annullata»,
  nessun report, «Ultima fase comunicata dal servizio: Scrittura del report.»,
  ultimo conteggio di S;
- `full-run-invalid`: T con DATA diverso: «Violazioni rilevate», report
  disponibile, «Record nei controlli tra tracciati 2»;
- `full-run-completed` (scenario P5-09): «Nessuna violazione nei controlli
  eseguiti», «Record nei controlli tra tracciati 2», senza la vecchia riga
  «Record del tracciato 0».

Un primo giro nativo era fallito perché la sessione era bloccata (lock screen
sopra la finestra): rieseguito dopo lo sblocco, nessuna modifica al codice
per quella causa.

## Rischi residui

- PROGRESS.md dichiara `trace_ref` opaco di 1..64 caratteri: l'handle di piano
  è lungo 47 caratteri più le cifre di generazione e indice (circa 49 in
  pratica). `validationProgress` applica il limite specifico di 64
  (`ipc.ts:436`): supererebbe 64 solo con 18 o più cifre di generazione e
  indice in tutto, e in quel caso il client rifiuterebbe il campione mentre
  l'esito finale, letto da `validation_state`, resterebbe indipendente.
  Correzione P510-01 (review Codex round 1): la versione in review diceva per
  errore «il decoder accetta fino a 128» e «17 cifre».
- Il campione resta letto ogni 250 ms: un tracciato o PASS2 molto brevi
  possono non comparire mai nel monitor; l'esito finale resta quello di Rust.
- CI remota e Windows/WebView2 NOT_RUN; G-MIG-5 aperto.
