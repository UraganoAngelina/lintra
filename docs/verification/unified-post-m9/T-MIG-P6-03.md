# T-MIG-P6-03 — Dettaglio diagnostico e selezione offset

Data 25 novembre 2026. Stato: **DONE**. Review Codex round 1 APPROVED su
`e9fa7cc` senza rilievi (`lintra-review-handoffs/T-MIG-P6-03-CODEX-REVIEW.md`).
Integrazione autorizzata dall'autore per fast-forward di `integration`, senza
push. CI remota e Windows/WebView2 NOT_RUN, al checkpoint del blocco P6;
G-MIG-6 NOT_VERIFIED.
Prerequisiti
T-MIG-P6-01 e T-MIG-P6-02 DONE (`integration` `860e88b`). Branch
`work/T-MIG-P6-03` da `860e88b`, worktree `/home/alberto/Desktop/LINTRA/lintra-p6-03`.

Fonti: prompt atomico `MIGRAZIONE_89/T-MIG-P6-03.md`; MIGRATION_PLAN v1.1
riga P6-03 («Valore/descrizione provenienti da Rust, nessun
dangerouslySetInnerHTML»), righe P6-04/05/06 per il confine di scope e
matrice di parità (5 filtri: «selection stale»); IPC v1 `DETAIL.md`
(selezione, testi, contesto) e `PAGING.md` («Shape della pagina e
selezione»); UI Slint `ui/ui/results.slint` (pannello «Dettaglio
diagnostico», tasti della tabella) e `ui/src/results/controller.rs`
(`select`, `selected_offset`); riferimento UI
`lintra_ui_references/lintra_tauri_ui.png` («Dettaglio controllo» a destra
della tabella). Il `TASK_PROMPTS/T-MIG-P6-03.md` originale non è nel kit.

## Preflight

Il backend era già completo da T-MIG-P4-09: `results_select`,
`results_text`, `results_context` nel provider Rust (`results/detail.rs`)
e nel client tipizzato (`ipc.ts`: `resultsSelect`, `resultsText`,
`resultsContext`, con DTO esatti), compresi i test sulla selezione che
sopravvive all'espulsione della pagina e cade con una nuova query
(`selection_is_pinned_across_eviction_and_dropped_by_a_new_query`). Il
renderer non selezionava righe né mostrava dettagli. La task è lato React;
nessuna modifica Rust.

Selezione offset. In Slint `selected_offset` è il byte offset JSONL, e dopo
una nuova query la riga viene riselezionata se è ancora presente. Il
contratto IPC v1 ratificato (14 novembre, `PAGING.md`/`DETAIL.md`) dice
invece che la selezione sopravvive all'espulsione della pagina ma non a un
cambio di query, e non espone offset fisici al renderer. Si applica il
contratto: la selezione è identificata dall'ordinale nella query (l'offset
logico di `PAGING.md`) e decade con una nuova query o un nuovo report.

## Cosa cambia

**Selezione** (`Results.tsx`, `ResultsGrid`): clic su una riga, oppure dal
riquadro della tabella (che riceve il fuoco) frecce su/giù, Home/End e
PagSu/PagGiù, come i tasti della tabella Slint. Si muovono entro la pagina
mostrata: le pagine si cambiano con il paginatore. La riga scelta ha
`aria-selected`, sfondo di selezione e la barra d'accento del riferimento;
la tabella diventa `role="grid"` (sola lettura) solo quando la selezione è
attiva. La selezione è `{query_ref, ordinal, diagnostic_ref}`: l'evidenza
resta sulla riga quando si torna alla sua pagina.

**Dettaglio** (`openDetail`, `useDetail`): una selezione chiede a Rust
`results_select` con il `diagnostic_ref` della pagina corrente. Come in
`openQuery` (P6-02), ogni risposta passa da `live()`: il dettaglio di una
riga lasciata non sostituisce quello appena scelto. Una nuova query (filtri,
nuovo report, riprova) svuota selezione e dettaglio e scarta le risposte in
corso; già nel render che precede l'effetto, una selezione di un'altra
query non è mostrata.

**Pannello «Dettaglio controllo»** (`DetailPanel`): a destra della tabella
da 1200 px di finestra, sotto la tabella nelle finestre più strette (come
il layout stretto Slint); resta in vista mentre la pagina scorre. Mostra,
come li ha catturati Rust: codice anomalia, messaggio, riga, tracciato (nome
da `results_overview`), campo, ID record, valore letto, valore atteso.
Stati: nessuna selezione («Seleziona una riga…»), lettura (`role=status`),
errore con Riprova (`role=alert`).

- Testi: sempre nodi di testo React, mai HTML. Un'anteprima con
  `complete: false` è marcata «…» e offre «Leggi il testo completo»:
  `results_text` a finestre di 16 KiB, una alla volta, con «Parte
  successiva» e «Dall'inizio» (`DETAIL.md`: nessun accumulo nascosto).
- Valore letto: testo esatto in un riquadro monospaziato con bordo
  tratteggiato e `white-space: pre-wrap`, così spazi iniziali, finali e
  doppi restano visibili senza alterare il testo (Slint li sostituiva con
  `␠` nella UI). `local_report_only_record_scope`: «Il valore copre
  l'intero record: si consulta solo nel report locale.»; `not_present`:
  «Nessun valore letto.».
- Contesto del record: letto solo a richiesta («Mostra il contesto»),
  pagine da 64 elementi, una alla volta («Elementi successivi», «Primi
  elementi»); i valori di contesto a intero record restano nel report
  locale.
- Codice fiscale ed episodio, che il DTO consente per la sola riga
  selezionata, non sono mostrati: Slint mostrava solo l'ID record
  (minimizzazione, `DETAIL.md`).

**Stile** (`style.css`): selezione della riga, layout a due colonne, pannello
con le misure del riferimento (codice grande in rosso, coppie
etichetta/valore). Le coppie annullano la griglia globale `dl` a due
colonne (vista nella prima anteprima nativa: «Valore letto» e «Valore
atteso» affiancati con etichette spezzate).

**Catture native** (`native_capture.py`): quattro scenari di dettaglio. Lo
scenario esistente `results-grid-scrolled` (Tab fino alla tabella, poi End)
ora seleziona l'ultima riga della pagina, oltre a scorrere.

## Cosa NON cambia

Core, CLI, golden JSONL, provider e contratto IPC v1 (nessuna chiave o
operazione nuova), Rust del desktop, export, SQLite, dipendenze. Field
guide e posizione in byte (`field_position`, componente `guide`) restano a
T-MIG-P6-04; fonti e domini (`source_count`, `results_sources`,
`results_domain`) a T-MIG-P6-05/06.

## Acceptance

| Requisito | Prova |
| --- | --- |
| Valore/descrizione da Rust | il pannello mostra solo componenti di `results_select`, pagine di `results_context` e finestre di `results_text`; `detail-panel.test.mjs` (valori, nome del tracciato dalla mappa Rust, anteprime incomplete marcate) |
| Nessun `dangerouslySetInnerHTML` | test sui sorgenti `src/` (nessun `dangerouslySetInnerHTML`, `innerHTML`, `outerHTML`, `insertAdjacentHTML`); testi ostili (`<img onerror>`, `<b>`, `<i>`) resi come testo escapato |
| Selezione offset | ordinale della query; evidenza `aria-selected` (test); selezione fissata da Rust attraverso le pagine, azzerata con una nuova query (contratto P4-09 + `useDetail`) |
| Stale safe | `openDetail`: risposte tardive (successo ed errore) scartate; mutazione: togliere ciascuno dei due `live()` fa fallire il test |
| Parità Slint | stessi campi del pannello Slint (codice, messaggio, tracciato, riga/ID record, campo, valore letto, atteso, contesto); stessi tasti della tabella |

## Test

| Verifica | Esito |
| --- | --- |
| `npm run lint` / `npm test` / `npm run build` | PASS / 153 PASS (9 nuovi in `detail-panel.test.mjs`) / PASS |
| Prove di mutazione su `openDetail` | togliendo il `live()` del successo o dell'errore il test delle risposte tardive fallisce (8/9); codice reale 9/9. La prima versione della prova non rilevava la mutazione del successo: la fixture tardiva era invalida per `resultsSelect` (`available` senza `actual`) e il test passava dal ramo di errore; fixture corretta |
| comandi Linux di `desktop.yml`: `cargo fmt -- --check` / `cargo test --locked --release --target x86_64-unknown-linux-gnu --features tauri/custom-protocol -j 2` / `cargo clippy … --all-targets … -- -D warnings` / `cargo build …` | PASS (0) / 85 passed, 0 failed, 1 ignored (0) / PASS (0) / PASS (0) |
| `native_smoke.py` sul binario del target CI (sessione Hyprland reale: `xvfb-run` non installato) | PASS / 0, `85eb92e3305108f0c94db5843512817ddc14abaacbfd80bf34adea2cff5a1500` |
| `npm run tauri -- build --no-bundle` + `native_smoke.py` | PASS / 0, binario `b825b31a78ecac758ba1bfc97e1ffc60c6d8999fbf37fd119f94b270bafe06af` |
| `python3 tests/native_capture.py <bin> <out>` | PASS 61/61 + chiusura durante il run (exit 0, 0,02 s, nessuno spool residuo) su `b825b31a…06af` ([receipt](T-MIG-P6-03-native/receipt.json)); 57 scenari precedenti + 4 dettaglio |
| workspace `ci.yml`, `verify_paging.py`, `verify_detail.py` | NOT_RUN: core, contratto e provider non toccati |
| Windows/WebView2 e CI remota | NOT_RUN (al checkpoint del blocco P6) |

Esiti degli scenari (`T-MIG-P6-03-native/`): `results-detail-1366x768`
riga 1 selezionata, pannello a destra con E_RECORD_LENGTH, messaggio,
Riga 1, T1, valore letto «4», atteso «6 byte»; `results-detail-keys`
tre volte giù → riga 3 nel pannello; `results-grid-scrolled` End → riga
128 selezionata e nel pannello; `results-detail-context` Riga 42 →
contesto letto, «Nessun elemento di contesto.»; `results-detail-640x480`
pannello sotto la tabella, «Mostra il contesto» a fuoco; dopo un filtro
(`results-filter-code`) il pannello torna a «Seleziona una riga…». Prima
della campagna, un'anteprima dei soli scenari di dettaglio ha mostrato le
coppie «Valore letto»/«Valore atteso» affiancate (griglia `dl` globale) e
il pannello fuori inquadratura a 640: corretti prima della campagna.

## Rischi residui

- Le interazioni (clic, tasti, lettura dei testi e del contesto) non hanno
  test DOM: nei test Node non c'è un DOM. Sono coperte dalle catture native
  e, per la logica di stato, da `openDetail`.
- Il report sintetico delle catture ha solo E_RECORD_LENGTH: campo, ID
  record e contesto sono vuoti, e nessuna anteprima è incompleta. I rami
  «Leggi il testo completo», contesto con valori e intero record sono
  provati solo dai test Node di rendering.
- I tasti muovono la selezione entro la pagina; il passaggio alla pagina
  successiva resta al paginatore (in Slint la lista era unica).
- «Riprova» del dettaglio riusa il `diagnostic_ref` della pagina: se nel
  frattempo la pagina è stata espulsa, Rust risponde stale e il pannello
  resta in errore finché si seleziona di nuovo la riga.

Prossima task tecnicamente disponibile: T-MIG-P6-04.
