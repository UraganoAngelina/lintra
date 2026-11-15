# Lintra desktop

Adapter React/Tauri affiancato a CLI e Slint; P4 in sviluppo, non release utenti.
Manifest e lock separati; preparazione collegata alla library, worker e
persistenza non ancora disponibili nella UI.

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
aggiuntivo. `npm test` esegue i decoder IPC e verifica asset/licenze locali.

L'unico comando `lintra_ipc` riceve e restituisce byte raw UTF-8: envelope e DTO
v1, massimo 1 MiB completo e profondità 32. La capability vale soltanto per la
finestra locale `main`, con ricontrollo Rust dell'origine. Il registry comprende
`app_info`, `client_ready`, catalogo e selezione esplicita, pagine dei
descrittori, `plan_create`, `plan_input_choose`, `plan_parameter_set` e
`plan_prepare`. Il selettore nativo conserva PathBuf in Rust; il client riceve
solo riferimenti opachi e contatori. Parametri tipizzati restano testo esatto,
anche per interi oltre il limite JavaScript. La preparazione congela schema,
mode e binding mediante i validator della library: non avvia record validation.
Nessuna operazione SQL o storia. La shell React resta quella P3 fino alle
viste P5; le API P4 sono provate attraverso i dispatcher reali.
CSP e guard WebView2 qualificati in P1-04 restano applicati.

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
