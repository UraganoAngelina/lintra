# T-MIG-P5-02 — Flow/schema/trace state machine UI

Data 20 novembre 2026. Stato: **DONE**. Review Codex APPROVED senza rilievi
su `375e42b`; integrazione autorizzata dall'autore per fast-forward di
`integration` e `work/p1-foundations`, senza push. CI remota NOT_RUN. Prerequisito T-MIG-P5-01 DONE (integrato in
`integration` `ad974a5`). Branch `work/T-MIG-P5-02` da `integration`
`ad974a5`, worktree dedicato `/home/alberto/Desktop/LINTRA/lintra-p5-02`.

Fonti: prompt atomico; MIGRATION_PLAN v1.1 riga P5-02 («Parità dei test
configuration schema_binding; no norm logic TS»); preflight Slint→Tauri
(`AnalysisSelection::{select_flow,select_version,prepare}`, test
`analysis_configuration`/`schema_binding`, rischio «stato dipendente e
invalidazione»); dossier T-MIG-P4-01/02/03 (catalogo, selezione e piano in
Rust: nuova selezione invalida i riferimenti vecchi, nessuna versione
automatica); `MIGRATION_TAURI_BRIEF.md` (form flusso/versione, stati e testi).

## Cosa cambia

- `src/analysis.ts` (nuovo): macchina a stati pura flusso → versione esplicita
  → schema acquisito da Rust → tracciato → bozza di piano LocalOnly creata da
  Rust. Scegliere un genitore azzera ogni scelta dipendente, come
  `select_flow`/`select_version` in Slint. Valori non presenti nel catalogo o
  nei descrittori Rust sono ignorati; ogni richiesta ha un token e le risposte
  tardive di una scelta sostituita sono scartate. Controller con API iniettata:
  catalogo e tracciati letti a pagine (64 righe Rust) con provenienza costante
  tra pagine e numero di tracciati uguale a `trace_count`; lo schema mostrato
  è quello restituito da `configuration_select` e una risposta per un altro
  flusso/versione è un errore tecnico, mai una sostituzione silenziosa
  (parità `schema_binding`). Messaggi operatore senza dettagli IPC.
- `src/AnalysisForm.tsx` (nuovo): `useAnalysisSelection` mantiene lo stato per
  tutta la vita della finestra (sopravvive al cambio di sezione) ed esegue le
  richieste Rust; `AnalysisForm` presentazionale con tre `<select>` nativi ed
  etichette permanenti, nessuna preselezione, campo attivo solo dopo il
  genitore, stato `role=status`/`alert` che dice cosa manca, riepilogo dello
  schema acquisito (flusso, versione, UTF-8, impronta, origine del bundle,
  tracciato).
- `src/AppShell.tsx`, `src/main.tsx`: il form compare nella sezione Analisi
  quando il servizio è pronto; il catalogo si legge una volta.
- `src/style.css`: campi in griglia responsive (tre in riga sul desktop, a
  capo sotto ~44 rem), solo token.
- `tests/analysis.test.mjs` (nuovo, 10 test); `tests/tsx.mjs` (nuovo): il
  caricamento dei componenti TSX di P5-01 diventa un helper condiviso;
  `tests/shell.test.mjs` lo usa senza cambiare asserzioni.
- `tests/native_capture.py`: due scenari di selezione da tastiera sul catalogo
  Rust reale, con passo `wait` per il giro Rust.

## Cosa NON cambia

Rust/src-tauri, IPC, DTO, CSP, core/CLI/Slint, golden e JSONL. Nessuna
dipendenza nuova. Nessuna regola normativa in TypeScript: il modello non
interpreta campi, vincoli o domini (verificato anche da un test statico sugli
import e sui pattern). Fuori scope e non anticipati: scelta del file e
anteprima (P5-03), formati di report (P5-04), avvio/progress (P5-05),
modalità Full e input per tracciato (P5-09), parametri del piano.

## Parità con i test Slint

| Contratto Slint | Prova React/TS |
| --- | --- |
| `catalog_requires_explicit_version_and_preserves_declared_trace_selection` | flussi una volta in ordine di catalogo, nessun flusso/versione preselezionato, flusso azzera versione/tracciato, versione azzera tracciato/piano, tracciati dai descrittori Rust |
| `invalid_selection_encoding_and_missing_file_never_prepare_a_request` | valori sconosciuti o fuori ordine non cambiano lo stato; il piano è creato solo da Rust per configurazione e tracciato acquisiti; risposta per altra configurazione/modalità rifiutata. Encoding fisso UTF-8 (unico ammesso dal selettore); file in P5-03 |
| `mismatched_service_schema_cannot_publish_a_report_with_stale_guidance` | schema mostrato = schema acquisito da Rust; risposta per altro flusso/versione o con numero di tracciati diverso → errore tecnico; risposte tardive scartate |

I test Slint di riferimento restano verdi: `cargo test -p lintra-ui
--all-features --locked --test analysis_configuration --test schema_binding`
PASS 8/8 e 1/1.

## Test

| Verifica | Esito |
| --- | --- |
| `npm run lint` | PASS rc 0 |
| `npm test` (50 preesistenti + 10 `tests/analysis.test.mjs`) | PASS 60/60 |
| `npm run build` (typecheck strict + Vite) | PASS rc 0 |
| Test Slint di riferimento | PASS 8/8 + 1/1 |
| Build release Linux + `tests/native_smoke.py` | PASS, `NATIVE_DESKTOP_SMOKE_PASS`, binario `94585f79…` |
| `python3 tests/native_capture.py <bin> <out>` | PASS 12/12 ([receipt](T-MIG-P5-02-native/receipt.json)) |
| src-tauri fmt/test/Clippy | NOT_RUN: nessun file Rust modificato; verdi su `ad974a5` |
| `npm ci` | NOT_RUN valido: engine pin node 26.13.0, locale 26.13.1 |
| Windows | NOT_RUN: l'acceptance non richiede prove Windows; nessun codice nativo cambiato; i test Node girano nella CI desktop Windows al prossimo run |

Prova nativa (CachyOS/Hyprland, binario `94585f79`): sul catalogo incorporato
reale, Tab×6 raggiunge Flusso, ↓ sceglie `SYNTHETIC`, la versione si attiva
senza preselezione; ↓ sceglie `v1` e Rust acquisisce lo schema (impronta
`99f89afe8552edb6`, schemi incorporati `synthetic-embedded-v1`); ↓ sceglie
il tracciato `T1` e il servizio accetta la selezione. A 640×480 i campi vanno
a capo e il riepilogo resta raggiungibile scorrendo la pagina. Le altre 10
capture ripetono gli scenari della shell P5-01 sul nuovo binario.

## Rischi residui

- Il catalogo incorporato ha un solo flusso/versione: più flussi e versioni
  sono provati con risposte Rust simulate nei test, non nativamente.
- Il piano LocalOnly creato non ha ancora input: la scelta file è P5-03.
- CI remota sul delta NOT_RUN; G-MIG-5 aperto.

Prossima tecnicamente disponibile dopo l'integrazione: T-MIG-P5-03 o
T-MIG-P5-04 (entrambe dipendono da P5-02).
