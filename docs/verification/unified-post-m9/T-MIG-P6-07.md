# T-MIG-P6-07 — Stati di risultati, malformed e partial

Data 26 novembre 2026. Stato: **IN_REVIEW** (non DONE). Prerequisiti
T-MIG-P6-01, T-MIG-P6-03, T-MIG-P6-06 DONE (`integration` `132be9f`).
Branch `work/T-MIG-P6-07` da `132be9f`, worktree
`/home/alberto/Desktop/LINTRA/lintra-p6-07`.

Fonti: prompt atomico `MIGRAZIONE_89/T-MIG-P6-07.md`; MIGRATION_PLAN v1.1
riga P6-07 («Non presentare conformità globale per LocalOnly o dati
incompleti; error states accessibili») e matrice §10.2 («Full UI mancante
oppure falsa global conformity»); IPC v1 `FULL.md` («VALID+Partial con
Unknown non dimostra conformità completa; una pagina senza diagnostici non
prova VALID»), `LIFECYCLE.md` (nessun boolean globale «conforme»),
`PAGING.md` (malformed/truncated/scope/count mismatch = errore tecnico, mai
pagina vuota); Slint `ui/src/results/controller.rs` (`completed_status`,
«Verifiche: complete nell'ambito richiesto / parziali», «Controlli locali»)
e `ui/src/worker_controller.rs` («Nessuna violazione nell'ambito
richiesto»). Il `TASK_PROMPTS/T-MIG-P6-07.md` originale non è nel kit.

## Preflight

Già presenti prima della task: esito a parole da `report_about`
(`reportStatusText`, nessun «conforme»), avvisi dei grafici per report
incompleto, completezza in lettura o non leggibile (P6-01, rilievo
P601-01); errori tecnici di query, pagina, conteggi, dettaglio, contesto e
dominio come `role="alert"` con «Riprova» dove previsto. Malformed,
truncated e count mismatch del report sono rifiutati da Rust (`PAGING.md`)
e arrivano al renderer come errore tecnico, già reso come avviso.

Lacune trovate nel riepilogo:
1. un report LocalOnly completo senza violazioni aveva il tono di successo
   (verde) come un Full completo: presentazione di conformità globale;
2. con `report_about` non leggibile il riepilogo restava su «In lettura…»
   e «—», senza dire che l'esito non c'è;
3. un report senza diagnostiche diceva solo «Il report non contiene
   diagnostiche.» anche se parziale o LocalOnly.

## Cosa cambia

`Results.tsx` (`Summary`, `emptyText`, `incomplete`):
- tono di successo solo per Full completo senza violazioni; LocalOnly
  completo resta neutro, parziale/troncato/annullato in avviso, violazioni
  in errore;
- nota dell'Esito con l'ambito delle verifiche, parole Slint: «verifiche
  complete nell'ambito richiesto» o «verifiche parziali»;
- `report_about` fallito: «Esito non disponibile» e nota `role="alert"`
  «Non è stato possibile leggere l'esito del report: nessun esito è
  mostrato.»; «Ambito non disponibile» al posto di «In lettura…»;
- report vuoto: LocalOnly «… nell'ambito richiesto: controlli locali sul
  tracciato.»; incompleto «… ma alcuni controlli non hanno un esito
  conclusivo: non è una conformità completa.»; esito non letto: solo
  l'assenza di diagnostiche;
- la scala dei grafici usa lo stesso `incomplete`.

Test: `tests/states.test.mjs` (nuovo, 4 test): tono solo Full completo;
parziale/troncato/annullato × LocalOnly/Full; esito non leggibile come
alert, distinto da «in lettura»; errori tecnici con «Riprova» e conteggi
non letti come «—». `tests/grid.test.mjs`: due attese aggiornate al nuovo
testo (nota dell'Esito, report vuoto LocalOnly). Prove di mutazione: verde
anche per LocalOnly, esito fallito nascosto, report vuoto che ignora la
parzialità: ciascuna 1 FAIL; codice reale 4/4.

## Cosa NON cambia

Core, provider e DTO (`report_about`, `results_overview`), CLI, golden,
contratto IPC v1, Rust del desktop, export, SQLite, dipendenze, banco di
cattura (gli scenari esistenti `results-valid` e `results-grid` coprono
LocalOnly completo valido e con violazioni).

## Acceptance

| Requisito | Prova |
| --- | --- |
| Nessuna conformità globale per LocalOnly | tono neutro e ambito dichiarato (test, mutazione); cattura `results-valid` |
| Nessuna conformità per dati incompleti | parziale/troncato/annullato in avviso con «verifiche parziali», report vuoto qualificato (test, mutazione) |
| Error states accessibili | esito non leggibile `role="alert"`; query/pagina/conteggi `role="alert"` con testo e «Riprova» (test) |

## Test

| Verifica | Esito |
| --- | --- |
| `npm run lint` / `npm test` / `npm run build` | PASS / 174 PASS (4 nuovi) / PASS |
| Prove di mutazione su `Summary`/`emptyText` | 3 mutazioni, ciascuna 1 FAIL; codice reale 4/4 |
| comandi Linux di `desktop.yml`: `cargo fmt -- --check` / `cargo test --locked --release --target x86_64-unknown-linux-gnu --features tauri/custom-protocol -j 2` / `cargo clippy … --all-targets … -- -D warnings` / `cargo build …` | PASS (0) / 86 passed, 0 failed, 1 ignored (0) / PASS (0) / PASS (0) |
| `native_smoke.py` sul binario del target CI (sessione Hyprland reale: `xvfb-run` non installato) | PASS / 0, `bc086a4467a10767a4c74d0721edb07d3a13f46e12adebf8aa85c0f45a67bd34` |
| `npm run tauri -- build --no-bundle` + `native_smoke.py` | PASS / 0, binario `3039d265f58402e91763b782b4e1b63aafed2fb054826bbaaf168098c2ea84cd` |
| `python3 tests/native_capture.py <bin> <out>` | PASS 71/71 + chiusura durante il run (exit 0, 0,04 s, nessuno spool residuo) su `3039d265…84cd` ([receipt](T-MIG-P6-07-native/receipt.json)); `results-valid`: Esito LocalOnly in tono neutro, nota «SYNTHETIC · v1 · verifiche complete nell'ambito richiesto» |
| `npm ci --ignore-scripts` | NOT_RUN in locale: EBADENGINE (node 26.13.1/npm 12.2.3 contro i pin) |
| workspace `ci.yml` | NOT_RUN: core e provider non toccati |
| Windows/WebView2 e CI remota | NOT_RUN (al checkpoint del blocco P6) |

## Rischi residui

- Gli stati parziale, troncato e annullato con report pubblicato non hanno
  una cattura nativa: il banco sintetico non li produce; provati da test
  Node e dai test Rust esistenti sull'esito.
- Il grafico «Record senza violazioni» (P6-01) resta verde al 100% per un
  LocalOnly senza violazioni: è una quota di record nei controlli eseguiti
  (etichetta accessibile «… nei controlli eseguiti»), non un esito del
  flusso; non toccato in questa task.
- «Esito non disponibile» non offre «Riprova»: `report_about` si rilegge
  tornando alla sezione, come già in Informazioni.

Prossima task tecnicamente disponibile dopo l'integrazione: T-MIG-P6-08.
