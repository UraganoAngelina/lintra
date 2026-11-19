# T-MIG-P4-10 — Export, About e test bridge integrati

19 novembre 2026. Autore Claude Code, revisore Codex. Baseline `integration`
`ec6a9e0` (P4-06, P4-08, P4-09 e P4-12 DONE), branch `work/T-MIG-P4-10`.
Stato: **IN_REVIEW**. Le verifiche locali sono PASS; mancano la review Codex,
la CI remota e le prove Windows. G-MIG-4 resta aperto fino al checkpoint
nativo del blocco P4.

Specifica: `docs/unified-post-m9/PROMPT_ATOMICI/MIGRAZIONE_89/T-MIG-P4-10.md`.
Acceptance: bridge LocalOnly+Full completo, export e provenance; test di
integrazione su synthetic.
Contratti: `docs/migration/ipc-v1/EXPORT.md` (P2-07), `FULL.md` (protezione di
tutti gli input), `docs/EXPORT_DESTINATIONS.md` (D04). Fonte:
`FONTI/MIGRATION_PLAN_V1_1_2026-10-30.md` riga 279. La schermata
Informazioni con licenze e attribuzioni è P5-07: qui c'è la provenienza
lato backend.

## Decisioni dell'autore (19/11/2026)

1. Motore di export **spostato nella libreria** e condiviso da Slint e
   desktop, come P4-07 fece con l'indice JSONL. Coerente con G-MIG-4
   ("adapter Rust usa library").
2. Abilitata la feature `Win32_UI_WindowsAndMessaging` della dipendenza
   esistente `windows =0.62.2`, per le conferme native su Windows. Nessun
   crate nuovo; `Cargo.lock` invariato.

## Cosa cambia

- **Libreria**:
  - `src/report/export.rs` (nuovo) contiene il motore di export estratto da
    `ui/src/export.rs`, con stessi tipi, messaggi, preflight, conferme D04,
    writer canonici, `AtomicReportFile` e ricevuta parziale;
  - unica differenza di comportamento: `ExportSource.protected_inputs` è un
    elenco, quindi un bundle Full protegge **tutti** i suoi input. Ogni input
    è confrontato sia come scritto sia canonicalizzato;
  - nuove API pubbliche (`src/lib.rs`): `ExportSource`,
    `export_captured_report`, `export_captured_report_with_authorization`,
    `export_destinations` (preflight strutturale per la preparazione), più i
    tipi `ExportFormat`, `ExportError`, `ExportFailure`, `ExportReceipt` e
    `summary_path`.
- **UI Slint**: `ui/src/export.rs` è ridotto a wrapper con le stesse firme
  pubbliche (`export_report*` su `AnalysisReport`, un solo input) più
  `ExportJob`; `ExportFormat::preferred` diventa `preferred_format`. Test
  Slint invariati e PASS.
- **Desktop** — `src-tauri/src/exports.rs` (nuovo), cinque operazioni:
  - `report_about {run}`: provenienza del report pubblicato (`app_version`,
    `validator_version`, flow/versione/hash dello schema, modo, stato,
    completezza, truncated, exportable, numero di tracciati e dataset,
    provenienza del bundle) e il `report_ref` opaco di EXPORT.md;
  - `export_prepare {report_ref, format}`: dialogo di salvataggio nativo; poi
    preflight strutturale e protezione, rifiutati con
    `rejected/destination_not_allowed` senza chiedere nulla; poi
    classificazione D04 delle due destinazioni; poi conferme native
    **distinte** per destinazione remota/indeterminata e per sovrascrittura
    (anche del solo sidecar). Il risultato è un piano `ready` con
    `export_ref`, oppure `cancelled` con motivo;
  - `export_execute {export_ref}`: consuma il piano prima di qualsiasi I/O
    (nessun replay) e avvia un job posseduto. Il motore ripete preflight e
    `verify_unchanged` prima dello staging e della pubblicazione;
  - `export_state` e `export_cancel`: `writing` fino al join, poi la
    ricevuta ExportResult (ruoli pubblicati, `cleanup_failed`,
    `durability_confirmed`, conteggi D04, errore tecnico versionato); cancel
    cooperativo.
- **Desktop, contorno**:
  - `native_picker.rs`: dialogo di salvataggio e conferma native su GTK e su
    Windows (`IFileSaveDialog` senza `FOS_OVERWRITEPROMPT`, `MessageBoxW`
    con Annulla come default);
  - `state.rs`: trait `Native` (picker, salvataggio, conferma) al posto della
    sola closure del picker. Le closure esistenti restano valide e senza
    capacità di export;
  - `main.rs`: `WindowNative`, dove ogni risposta ottenuta durante la
    chiusura è stale;
  - `jobs.rs`: `Exports` vive accanto all'ultimo report e viene cancellato e
    joinato prima di esso; accessor `spool`, `private_root`, `inputs`,
    `audit` del `Report`.
- `src/ipc.ts`: client strict con i conteggi D04 (somma 2), le ricevute
  (successo solo con entrambi i file, durabili, senza cleanup né errore) e la
  provenienza.
- **Test**:
  - 3 test di integrazione nativi in `exports.rs`;
  - 1 test di libreria in `src/report/export.rs`;
  - `tests/exports.test.mjs` (3 test Node).

## Cosa NON cambia

Restano invariati:
- motore di validazione, writer canonici JSONL/CSV/XLSX e il loro output,
  CLI, golden e persistenza;
- policy D04 e `ReportDestinationAssessment`;
- DTO e operazioni esistenti di lifecycle, progress, query, dettaglio;
- test Slint;
- `Cargo.lock` e dipendenze (solo una feature in più del crate `windows`).

Nessun path, nome file, flag di conferma o contenuto di report nell'IPC.
Nessun comando `write_file` e nessuna regola nel client.

## Acceptance

| Criterio | Evidenza |
|---|---|
| Bridge LocalOnly+Full completo con export | `local_only_and_full_reports_export_canonical_files_through_the_bridge`: report LocalOnly e Full prodotti dal worker reale, esportati via bridge in JSONL, CSV e XLSX. JSONL e CSV sono byte-identici al replay canonico diretto; per l'XLSX si verifica la firma del workbook; il summary è identico a `write_run_summary_with_bundle`. Ruoli `report`+`summary`, durabilità confermata, 2 file locali e nessuna conferma richiesta. Nessun frame contiene path di input, spool o destinazione. |
| Provenance | Stesso test: `report_about` dà modo, numero di tracciati, hash dello schema, provenienza del bundle e versione dell'eseguibile coerenti con summary e audit catturati; il `report_ref` è stabile finché il report resta pubblicato. |
| Protezione di tutti gli input Full e conferme native distinte | `full_inputs_are_protected_and_overwrite_is_a_separate_native_choice`: entrambi gli input del bundle e un file nella directory privata sono rifiutati prima di qualsiasi conferma, e gli input restano intatti. Selezione nativa annullata → `cancelled`. Sostituzione del solo sidecar → una conferma di sovrascrittura; rifiutata, il file resta intatto; accettata, il summary viene sostituito con quello canonico. Test di libreria `every_bundle_input_and_the_private_spool_are_protected_destinations`, anche con percorso alias. |
| Handle legati al report e piani consumati | `export_handles_are_bound_to_the_published_report_and_consumed_once`: nessun About o export senza report; `report_ref` sconosciuto stale; un nuovo `prepare` sostituisce il piano; il piano eseguito non si rigioca; cancel dopo il terminale `already_terminal`; un nuovo run pubblicato rende stale il vecchio `report_ref`; DTO con campi extra (path, overwrite) o formati ignoti rifiutati. |
| Client strict | `tests/exports.test.mjs`: provenienza, piani, ricevute e stati con forme e incoerenze rifiutate. |

Mutazione: togliendo gli input protetti da `prepare`, il test di protezione
fallisce. Il sorgente è stato ripristinato con SHA identico (`6ea27293…f9f8`).

## Verifiche locali (Linux CachyOS, rustc 1.99.1, Node v26.13.0)

| Comando | Esito |
|---|---|
| `npm ci` / `npm run lint` / `npm test` / `npm run build` (in `apps/lintra-desktop`) | 0 / 0 / 41 PASS / 0 |
| `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` | PASS / 0 |
| `cargo test … --release --target x86_64-unknown-linux-gnu --features tauri/custom-protocol -j 2` | 61 PASS, 1 ignored GTK preesistente / 0 (58 + 3 nuovi) |
| `cargo clippy … --all-targets … -- -D warnings` | PASS / 0 |
| `cargo build … --release` | PASS / 0 |
| `python3 tests/native_smoke.py …/release/lintra-desktop` | PASS / 0, binario `3d5f2290…6c46`; display reale, non Xvfb |
| `cargo fmt --all -- --check` | PASS / 0 |
| `cargo clippy --workspace --all-targets [--all-features] --locked -j 2 -- -D warnings` | PASS / 0, in entrambe le configurazioni |
| `cargo test --workspace [--all-features] --locked -j 2` | 1.114 PASS, 0 fail, 9 ignored / 0, in entrambe (inclusi i test Slint `exports`) |
| `cargo test -p lintra-ui --all-features --locked --test exports directory_sync_failure_after_rename_reports_the_published_file -- --exact --ignored` (step Linux di `ci.yml`) | 1 PASS / 0 |
| `git diff --check` | PASS / 0 |

Log fuori repo: `lintra-review-handoffs/T-MIG-P4-10.gate.log`, SHA-256
`0bc94b1cd11c8bb04d88c8bd9b08d41217e107ac2536d61e7bb9c4a57bcc190d`.

**BLOCKED in locale:** `cargo check --target x86_64-pc-windows-gnu` del
codice Windows nuovo (`save`, `confirm`). La build di `libsqlite3-sys` per
Windows richiede `x86_64-w64-mingw32-gcc`, non installato. Il codice Windows
nuovo non è quindi compilato localmente: la verifica è rinviata al job
Windows di `desktop.yml` al checkpoint P4.

**NOT_RUN:**
- step Windows di `ci.yml` (`shared_export_cli` e `shared_exports
  --ignored`, con la fixture SMB della CI);
- CI remota `desktop.yml`/`ci.yml`;
- prove Windows/VM;
- dialoghi nativi reali (GTK e Windows) con interazione: i test usano
  dialoghi sintetici iniettati.

## Sorgenti verificati

| File | SHA-256 |
|---|---|
| `src/report/export.rs` | `4a6fed4ef95f9d7a2aedc4ec6a24b9e68000fb120d48bee0e8fb52cebd06c6e7` |
| `src/report/mod.rs` | `feb7a47b7feca2a885bf4f3aa13fdfec3756e65aed2d1da72b1116a88cbca37a` |
| `src/lib.rs` | `ed411b56f272d84f78c32bb8d6a19bc73094a87b931640b6b2831b988e5c1c33` |
| `ui/src/export.rs` | `8de8e7ef51cab6f952c6af677afafb0c7ca6cf1510758d57364117eb06b15206` |
| `ui/src/export_controller.rs` | `ee28f467c2090a405ea61549d4ab6e42c10beecd123314ef989590e667868f39` |
| `apps/lintra-desktop/src-tauri/Cargo.toml` | `0a0d257da488e8f327857a4ae2462b6241edf87d4a8553217810187cad322168` |
| `apps/lintra-desktop/src-tauri/src/exports.rs` | `6ea27293708418bc637100fa7afbde6ba74d260b15d66c8208558ca8c83bf9f8` |
| `apps/lintra-desktop/src-tauri/src/jobs.rs` | `9748f59034c952a9b08a73cc85a40299c0e499055b45a8c8f68536fcef48a3f2` |
| `apps/lintra-desktop/src-tauri/src/state.rs` | `ad94e6dddfde6e2f3695fa6f79781376f8d4a705814e8d181302bda4626dc407` |
| `apps/lintra-desktop/src-tauri/src/configuration.rs` | `d3b9f345e19ce98e57357dab58b67bf83a3b1ed1b50125134cd324c8650d1086` |
| `apps/lintra-desktop/src-tauri/src/main.rs` | `b94e8d8547da1a17445cba61c1fae1dd5b955ca4a5651c2a77c11bcf69683477` |
| `apps/lintra-desktop/src-tauri/src/native_picker.rs` | `56a9fef6b71b1932e3422bd751886d4ed2be0950b9c133a5da1f3732c944c489` |
| `apps/lintra-desktop/src-tauri/src/results.rs` | `5a7eff7a947ccb3bf884e35126ba1723e42b71187aec16437a157ff6673f541e` |
| `apps/lintra-desktop/src/ipc.ts` | `8eac2921106e612bea935a10ff265a2366cde4af57bc3d071504e95a8d5ba7ab` |
| `apps/lintra-desktop/tests/exports.test.mjs` | `1faa4e720c222b2b781f5f08291aaf9f151adbe0c2bf8bb907093daa1bb6c283` |

## Scelte da confermare in review

1. **`report_ref` emesso da `report_about`.** EXPORT.md non dice chi emette
   il `report_ref`: lo restituisce la stessa operazione di provenienza.
   Resta stabile finché il report è pubblicato; un nuovo run lo rende stale.
2. **`execute` asincrono.** Restituisce `writing` e la ricevuta arriva con
   `export_state`, perché un XLSX grande non può bloccare il thread GUI.
   Aggiunto `export_cancel`: cancel cooperativo, e i file già rinominati
   restano nella ricevuta.
3. **Stato `rejected/destination_not_allowed`.** Copre una destinazione
   protetta o strutturalmente non valida, rifiutata senza chiedere conferme
   e senza esporre path o dettagli.
4. **Insieme report + summary.** Il dialogo di salvataggio nomina il sidecar
   nel titolo; le conferme di destinazione condivisa e di sovrascrittura
   elencano entrambi i file. Per una destinazione locale nuova non c'è
   conferma aggiuntiva (D04).
5. **Estensione mancante.** Al nome scelto senza estensione viene aggiunta
   quella del formato; un'estensione diversa viene rifiutata dal preflight.

## Rischi residui

- Codice Windows nuovo non compilato né eseguito localmente (vedi BLOCKED).
- Destinazioni remote/indeterminate e la relativa conferma verificate solo
  via policy di libreria e test Slint: il test desktop usa destinazioni
  locali, e la qualifica SMB resta allo step Windows della CI.
- Cancel di un export a metà scrittura: coperto dai test del motore
  (adattatori annullabili), non da un test desktop che fermi il job.
- I limiti TOCTOU del servizio esistente restano quelli dichiarati da
  EXPORT.md.
- G-MIG-4: tutte le task P4 sono ora implementate; il gate aggregato
  richiede il checkpoint CI nativo Linux/Windows del blocco.

Prossimo passo: checkpoint G-MIG-4 (CI nativa del blocco P4), poi
T-MIG-P5-01.
