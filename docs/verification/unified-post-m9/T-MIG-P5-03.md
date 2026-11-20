# T-MIG-P5-03 — Conferma file e picker sicuro

Data 20 novembre 2026. Stato: **IN REVIEW**, non DONE prima della review Codex
e dell'integrazione. Prerequisito T-MIG-P5-02 DONE (integrato in
`integration` `cdf8373`). Branch `work/T-MIG-P5-03` da `cdf8373`, worktree
dedicato `/home/alberto/Desktop/LINTRA/lintra-p5-03`.

Fonti: prompt atomico; MIGRATION_PLAN v1.1 riga P5-03 («Il frontend non legge
byte fixed-width e non presenta sample come file completo»); preflight
Slint→Tauri (Conferma file: `InputPreview` ≤64 KiB, «nessun invio contenuto
input al frontend»); dossier T-MIG-P4-02 (broker file: dialogo nativo GTK /
IFileOpenDialog posseduto da Rust, path solo nel piano, client con soli
disposizione/nuovo plan_ref/contatori, annullamento senza modifica, errori
che non sostituiscono il binding); `MIGRATION_TAURI_BRIEF.md`.

## Cosa cambia

- `src/analysis.ts`: il piano accettato da Rust porta uno stato di input
  (`chosen` + attività: nessuna, dialogo aperto con token, annullato,
  fallito). Un solo dialogo per volta; risposte tardive scartate. Un file
  accettato aggiorna `plan_ref` e `assigned_input_count` con i valori Rust;
  annullamento ed errore conservano la scelta precedente, come il binding
  Rust. Cambiare tracciato, versione o flusso crea un altro piano e quindi
  azzera l'input. Il controller `chooseInput` accetta solo una risposta per
  il tracciato del piano, con nuovo `plan_ref` e anteprima.
- `src/AnalysisForm.tsx`: sezione «File del tracciato» con un pulsante
  («Scegli il file» / «Scegli un altro file», disabilitato a dialogo aperto),
  stati annunciati e anteprima del campione: dimensione del file, byte del
  campione e fine riga «nel campione»; se il campione non arriva alla fine,
  la nota dice che i conteggi riguardano solo i primi N byte e non l'intero
  file. Nessun nome, path o byte: Rust non li invia.
- `src/style.css`: stile del pulsante (azione primaria) e della nota.
- `tests/input.test.mjs` (nuovo, 8 test); `tests/analysis.test.mjs`: due
  asserzioni aggiornate perché, con P5-03, dopo il piano la scelta mancante è
  il file (contratto esteso, non indebolito).
- `tests/native_capture.py`: scenari con il vero dialogo nativo
  (`dialog:<path>`, `dialog-cancel`) su file sintetici creati e rimossi dallo
  harness; Ctrl+L per il campo posizione GTK4, slash del tastierino (uguale
  su ogni layout), conferma ripetuta finché il dialogo si chiude.

## Cosa NON cambia

Rust/src-tauri (broker, `InputPreview`, IPC, DTO), CSP, core/CLI/Slint, golden
e JSONL. Nessuna dipendenza nuova, nessun plugin fs/dialog. Fuori scope e non
anticipati: formati di report (P5-04), avvio (P5-05), Full (P5-09), parametri.

## Acceptance

| Requisito | Prova |
| --- | --- |
| Il frontend non legge byte | nessun `<input type="file">`, FileReader, Blob, `arrayBuffer`, readFile, plugin fs/dialog in `src/` (test statico); la richiesta IPC contiene solo `plan_ref` e `trace_ref`; una risposta con `path`, `name` o campi extra nell'anteprima è rifiutata |
| Il campione non è presentato come file completo | testo «nel campione» per ogni conteggio; con `reached_eof=false` la nota «solo i primi N byte: i conteggi non descrivono l'intero file»; nessuna dicitura di righe/record del file; prova nativa con file di 140.000 byte (campione 65.536) e di 15 byte (intero) |

## Test

| Verifica | Esito |
| --- | --- |
| `npm run lint` | PASS rc 0 |
| `npm test` (60 + 8 `tests/input.test.mjs`) | PASS 68/68 |
| `npm run build` | PASS rc 0 |
| Build release Linux + `tests/native_smoke.py` | PASS, binario `1dcefc74…` |
| `python3 tests/native_capture.py <bin> <out>` | PASS 15/15, tre esecuzioni consecutive rc 0 ([receipt](T-MIG-P5-03-native/receipt.json)) |
| src-tauri fmt/test/Clippy | NOT_RUN: nessun file Rust modificato; verdi su `cdf8373` |
| `npm ci` | NOT_RUN valido: engine pin node 26.13.0, locale 26.13.1 |
| Windows (IFileOpenDialog) | NOT_RUN: non richiesto dall'acceptance; nessun codice nativo cambiato |

Prova nativa (CachyOS/Hyprland, dialogo GTK4 reale aperto da Rust): selezione
da tastiera, Tab+Invio sul pulsante, path digitato nel campo posizione del
dialogo del processo posseduto. File di 140.000 byte: «Dimensione del file
140.000 byte», «Campione letto 65.536 byte», «LF 9362» nel campione e nota
del campione parziale. File di 15 byte con LF e CRLF: «LF 1 · CRLF 1» e
«Il campione comprende l'intero file». Esc: «Scelta annullata: nessun file
assegnato», piano invariato. Le tre esecuzioni danno immagini identiche, salvo
la barra di scorrimento a comparsa in una cattura (stesso contenuto). I primi
tentativi dello harness sono falliti per problemi dello harness (nome del
tasto `slash`, layout italiano, ricerca GTK4, race sulla chiusura del dialogo),
corretti senza modificare l'app.

## Rischi residui

- Errore di lettura e BOM arrivano come `IPC_TECHNICAL_FAILURE` senza causa
  distinta: il messaggio cita entrambe le possibilità. Distinguerle richiede
  un nuovo codice nel contratto IPC (decisione fuori scope).
- Nome del file non mostrato (scelta P4-02: niente path al renderer); il
  riepilogo «Pronta» del brief avrà file/schema/ambito effettivi con P5-05.
- Dialogo Windows non provato in questa task; CI remota NOT_RUN; G-MIG-5 aperto.

Prossima tecnicamente disponibile dopo l'integrazione: T-MIG-P5-04.
