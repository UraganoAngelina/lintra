# Lintra desktop

Adapter React/Tauri affiancato a CLI e Slint; P4 in sviluppo, non release utenti.
Manifest e lock separati; preparazione e worker LocalOnly collegati alla library.
Shell P5-01: sezioni Analisi/Risultati/Informazioni e tema Sistema/Chiaro/Scuro
(solo di sessione). P5-02: selezione flusso, versione esplicita e tracciato
sul catalogo Rust, con piano LocalOnly creato dal servizio. P5-03: file
scelto nel dialogo nativo aperto da Rust, con anteprima del solo campione
(mai byte o path nel renderer). P5-04: formati del report XLSX/JSONL/CSV
come in Slint (XLSX e JSONL di default), solo intento: nessun writer JS.
P5-05: «Avvia analisi», progresso letto da Rust ogni 250 ms (una richiesta
alla volta, avanzamento indeterminato) ed esito; form bloccato durante il run.
Budget di risorse soft D09: file `resource-budget.json` nella directory di
configurazione per-utente (vedi `docs/RESOURCE_BUDGETS.md`), avviso con
Annulla / Procedi comunque deciso in Rust. P5-06: «Interrompi analisi»
finché Rust può fermare il lavoro; l'esito annullato dichiara che non c'è un
report nuovo e che un eventuale report precedente riguarda l'analisi
precedente.

```sh
npm ci --ignore-scripts --no-audit --no-fund
npm run lint
npm test
npm run build
npm run tauri -- build --no-bundle -- --locked -j 2
```

Node e npm ammessi sono in `engines`/`.nvmrc`, con engine-strict e lifecycle
scripts disabilitati in `.npmrc`. Il lint usa le regole statiche TypeScript
unused/fallthrough; `build` include anche il typecheck strict. Nessun ESLint
aggiuntivo. `npm test` esegue i decoder IPC, verifica asset/licenze locali e la shell
(markup semantico, tema, contrasti). `tests/native_capture.py` (Hyprland
personale) e `tests/windows_capture.ps1` (sessione console Windows) producono
capture e prove di tastiera native; non sono gate CI.

L'unico comando `lintra_ipc` riceve e restituisce byte raw UTF-8: envelope e DTO
v1, massimo 1 MiB completo e profondità 32. La capability vale soltanto per la
finestra locale `main`, con ricontrollo Rust dell'origine. Il registry comprende
`app_info`, `client_ready`, catalogo e selezione esplicita, pagine dei
descrittori, `plan_create`, `plan_input_choose`, `plan_parameter_set` e
`plan_prepare`. Il registry del worker comprende `validation_start`,
`validation_state`, `validation_progress`, `validation_cancel` e
`validation_acknowledge`: un thread posseduto, un solo campione progresso,
terminale conservato fino all’ack e report temporanei legati al RunRef.
La chiusura richiede cancel e attende il thread prima del cleanup. Full
resta rifiutato fino a P4-11. Il selettore nativo conserva PathBuf in Rust; il client riceve
solo riferimenti opachi e contatori. Parametri tipizzati restano testo esatto,
anche per interi oltre il limite JavaScript. La preparazione congela schema,
mode e binding mediante i validator della library: non avvia record validation.
Nessuna operazione SQL o storia. Le API P4 sono provate attraverso i
dispatcher reali; la shell P5-01 non le richiama ancora oltre al bootstrap.
CSP e guard WebView2 qualificati in P1-04 restano applicati.
Su Windows, all'avvio, la finestra viene ridotta e spostata dentro la work
area del monitor (anche a 200%); su Wayland decide il compositor.

`--shell-smoke-test` verifica caricamento del documento e chiusura; `--smoke-test`
aspetta anche l'ack React/IPC dopo il rendering. Entrambi terminano con exit 0
e un marker fisso soltanto sul percorso riuscito. Un harness deve imporre un
timeout e isolare le directory dati della WebView. Non sono prove di release.

Font e licenza OFL sono copie byte-identiche dei file già forniti per Slint.
Provenienza e fonti dei token in `assets-provenance.json`; palette blu/cyan
ratificata, contrasti di testo mantenuti. La baseline WebView è distinta da
Slint. Il nome nell'header è una semplice etichetta: gli artwork separati del
logo book saranno integrati nella UI di prodotto, senza ricostruire il marchio
con un font o distribuire l'intera tavola come logo. Nessuna risorsa remota.
