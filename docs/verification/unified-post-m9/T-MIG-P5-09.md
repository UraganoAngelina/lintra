# T-MIG-P5-09 — React: selezione Full con input esplicito per tracciato

Data 23 novembre 2026. Stato: **IN_REVIEW** (READY_FOR_CODEX_REVIEW), non DONE
prima della review e dell'integrazione. Prerequisiti T-MIG-P5-03 e T-MIG-P4-12
DONE (`integration` `95ed944`). Branch `work/T-MIG-P5-09` da `95ed944`,
worktree `/home/alberto/Desktop/LINTRA/lintra-p5-09`.

Fonti: prompt atomico («La UI invia solo richieste legittime e tipizzate;
blocca input incoerente; nessun latest implicito, nessuna inferenza normativa
lato React»); MIGRATION_PLAN v1.1 riga P5-09; Master Plan §6 punto 4 e GAP-10
(Full multi-tracciato con parità Rust/CLI, niente logica normativa JS); IPC v1
`docs/migration/ipc-v1/FULL.md` (`plan_create` Full con `trace_ref` null,
`plan_input_choose`, `plan_parameter_set` con `value.kind` text/integer in
testo esatto, `plan_prepare` come preflight autorevole); dossier T-MIG-P4-11 e
T-MIG-P4-12 (adapter e lifecycle Full in Rust). Il `TASK_PROMPTS/T-MIG-P5-09.md`
originale non è nel kit. Slint non ha una selezione Full: nessuna parità
legacy da preservare oltre al flusso LocalOnly.

## Preflight

- P5-03 e P4-12 DONE in `UNIFIED_IMPLEMENTATION_STATUS.md` e in Git.
- Il backend era già completo per Full (P4-11/P4-12): `configuration.rs`
  accetta `plan_create` Full, sceglie l'input per qualunque tracciato,
  convalida i parametri col core (`VerifiedRunContext`) e in `plan_prepare`
  usa `preflight_flow_inputs`; `jobs.rs` rifiuta un avvio in modalità diversa
  da quella preparata. I decoder TypeScript (`ipc.ts`) avevano già
  `planCreate(…, "full", null)`, `planParameterSet`, `planPrepare`.
- Mancava solo il renderer: `analysis.ts` creava sempre un piano LocalOnly,
  scartava i descrittori dei parametri, e `run.ts` preparava e avviava sempre
  `local_only`. Nessuna implementazione parziale di Full nel frontend.

## Cosa cambia

- `src/analysis.ts`:
  - `mode` esplicito (`local_only` di default, come finora; nessuna versione
    scelta in automatico in nessuna modalità). Cambiare modalità scarta piano,
    file e valori, non lo schema acquisito.
  - Il piano pronto ha `inputs` per `trace_ref` (uno in LocalOnly, tutti i
    tracciati dello schema in Full) e `parameters` per `parameter_ref` (solo
    Full). In Full il piano si chiede appena Rust ha acquisito lo schema, con
    `trace_ref` null; `select_trace` è ignorato.
  - `loadConfiguration` conserva i descrittori dei parametri (id e tipo
    dichiarato da Rust) e verifica anche `parameter_count`.
  - Un dialogo nativo o una richiesta di parametro alla volta (`mutating`):
    ognuno cambia `plan_ref` in Rust.
  - Parametri: il testo digitato resta locale; all'uscita dal campo o con
    Invio si invia una sola richiesta, col tipo dichiarato. Un intero che non
    rispetta la sintassi di `IntegerLiteral` (`-` opzionale e cifre) viene
    bloccato e non inviato (`integerSyntax`). Range e valori ammessi li
    controlla solo Rust; se rifiuta, resta valido il valore applicato prima.
  - `startablePlan`: LocalOnly come prima; Full con almeno un file assegnato
    (conteggio di Rust), nessuna richiesta in corso e ogni valore digitato
    uguale a quello che Rust tiene. Quali tracciati e parametri siano
    richiesti lo dice Rust in `plan_prepare`.
  - `createPlan(…, mode, traceRef)` rifiuta localmente mode/trace incoerenti
    e una risposta in un'altra modalità; `setParameter` accetta solo un nuovo
    piano Full.
- `src/ipc.ts`: `integerText` e controllo in `planParameterSet`: un intero
  malformato non lascia il renderer (`IPC_INVALID_MESSAGE` locale). È la
  stessa sintassi di `integerSyntax`; un test verifica che coincidano.
- `src/AnalysisForm.tsx`: gruppo radio nativo «Tipo di analisi» (una fermata
  di Tab, frecce per cambiare); in Full, al posto del tracciato singolo,
  «File dei tracciati» con un pulsante per tracciato (`aria-label` col nome
  del tracciato) e «Parametri del flusso» con etichetta, tipo e messaggio
  collegato al campo (`aria-describedby`, `aria-invalid`, `role="alert"` per
  il blocco o il rifiuto). Nessun tracciato è etichettato come obbligatorio o
  facoltativo. Il riepilogo mostra modalità e «File assegnati N di M».
- `src/run.ts`, `src/RunPanel.tsx`: l'avvio porta la modalità del piano
  (`planPrepare` deve rispondere la stessa, poi `validationStart(plan, mode)`;
  la risincronizzazione dopo una risposta persa si aggancia solo a un run della
  stessa modalità). Rifiuto di un piano Full: messaggio dedicato
  `START_FAILED_FULL`, che elenca le cause possibili senza indovinare quale.
- `src/style.css`: stili del gruppo modalità, dei file per tracciato e dei
  campi parametro con i token esistenti.
- Test: `tests/full.test.mjs` (12 nuovi); adattati alla nuova forma dello
  stato (`plan.inputs`, firma di `createPlan`) `analysis`, `input`, `run`,
  `cancel`, senza togliere asserzioni; aggiunti i casi Full e il controllo di
  `parameter_count` in `analysis.test.mjs`.
- `tests/native_capture.py`: la sequenza di selezione attraversa il nuovo
  gruppo radio (una Tab in più; `shift-Tab` da «Avvia» a Informazioni da 9 a
  10); cinque scenari Full su un bundle esterno sintetico (schema golden con
  tracciati S e T in join, come `tests/support/full_schema.rs`, più i
  parametri `unit` testo a/b e `year` intero) passato con
  `LINTRA_SCHEMA_DIR`/`LINTRA_REFERENCE_DATA_DIR` nell'ambiente privato dello
  scenario; gli altri scenari tolgono esplicitamente le due variabili.
- README desktop.

## Cosa NON cambia

Rust di produzione (core, Tauri, IPC v1, DTO), golden, CLI, lockfile e
dipendenze. Nessuna regola normativa, value set o default di parametro nel
renderer. LocalOnly: stesso flusso e stesse parole; solo il gruppo radio in
più. Monitor Full per tracciato e pass e stati terminali multi-pass: T-MIG-P5-10.

## Acceptance

| Requisito | Prova |
| --- | --- |
| Solo richieste legittime e tipizzate | test: `plan_create` Full con `trace_ref` null e mai con mode/trace incoerenti (nessuna chiamata); `plan_parameter_set` con payload esatto, tipo dichiarato e testo esatto (`-0042`, mai `Number`); intero malformato rifiutato in `ipc.ts` senza invio; avvio Full preparato e avviato come Full, risposta `local_only` rifiutata |
| Blocca input incoerente | test: `12a`, `+5`, `1.0`, `1e3`, ` 7`, `-` bloccati senza token né richiesta; avvio bloccato con dialogo aperto, valore in verifica, valore non applicato o rifiutato; un solo dialogo per volta fra tracciati; tracciato sconosciuto ignorato. Nativo: `20x6` segnalato sul campo, `Avvia` rifiutato da Rust con un solo file |
| Nessun latest implicito | test: nessuna versione scelta in Full; modalità di default invariata (LocalOnly) e cambiata solo da evento esplicito; valori sconosciuti ignorati |
| Nessuna inferenza normativa in React | test: nessun tracciato marcato richiesto/facoltativo nel markup; nessun `allowed_values`/`min`/`max`/`Number(`/`parseInt` nei moduli; il modulo del modello importa solo tipi da `./ipc` e non usa regex (test P5-02 invariato). Nativo: la mancanza di T è scoperta da Rust in `plan_prepare` |

Mutazioni: con il blocco della sintassi intera disattivato falliscono 2 test;
con `validationStart` forzato a `local_only` fallisce 1 test. Sorgenti
ripristinati (SHA-256 `analysis.ts` `bd4313ee…af81a0`, `run.ts`
`777cda6a…d39f`).

## Test

| Verifica | Esito |
| --- | --- |
| `npm run lint` / `npm test` / `npm run build` | PASS / 116 PASS (104 + 12) / PASS |
| desktop `cargo fmt … -- --check` | PASS / 0 |
| desktop `cargo test … --release --target x86_64-unknown-linux-gnu --features tauri/custom-protocol -j 2` | PASS / 0: 82, 0 fail, 1 ignored (nessun file Rust modificato) |
| desktop `cargo clippy … --release --all-targets … -- -D warnings` | PASS / 0 |
| `npm run tauri -- build --no-bundle -- --locked -j 2` + `native_smoke.py` | PASS, binario `0adb5779…df17fee` |
| `python3 tests/native_capture.py <bin> <out>` | PASS 39/39, rc 0 ([receipt](T-MIG-P5-09-native/receipt.json)): 34 scenari precedenti con la nuova sequenza di Tab + 5 Full |
| `git diff --check integration` | PASS |
| `npm ci` | NOT_RUN: engine pin node 26.13.0, locale 26.13.1 (come P5-07) |
| workspace core | NOT_RUN: nessun file core modificato |
| CI remota / Windows / WebView2 | NOT_RUN |

Prova nativa (Hyprland, controllata a vista), bundle esterno sintetico S/T:
- `full-selection-*`: scelta «Flusso completo (Full)» da tastiera; nessun
  campo Tracciato; un pulsante per S e uno per T, «Nessun file assegnato»;
- `full-parameter-blocked`: S e T assegnati nel dialogo GTK reale, `unit=a`
  «Valore applicato dal servizio», `year=20x6` bordo d'errore e messaggio
  sotto il campo, «Avvia» disabilitato;
- `full-prepare-refused`: solo S, nessun parametro: «Avvia» manda il piano,
  Rust lo rifiuta in `plan_prepare`, messaggio Full; nessun run;
- `full-run-completed`: S, T, `unit=a`, `year=2026`: run Full completato,
  «Nessuna violazione nei controlli eseguiti», «Report disponibile».

## Rischi residui

- Dopo un run Full il pannello mostra i contatori pensati per un tracciato
  (l'ultimo campione Rust, in PASS2 «Record del tracciato 0»): il monitor Full
  per tracciato e pass spetta a T-MIG-P5-10.
- Con un parametro in verifica, un secondo valore confermato in quel momento
  resta «non ancora applicato» finché l'operatore non conferma di nuovo
  (Invio o uscita dal campo); l'avvio resta bloccato e il motivo è scritto.
- Rust non dice quale tracciato o parametro manca in `plan_prepare`: il
  messaggio elenca le cause possibili.
- CI remota e Windows/WebView2 NOT_RUN; G-MIG-5 aperto.
