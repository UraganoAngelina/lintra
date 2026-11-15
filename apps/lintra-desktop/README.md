# Lintra desktop

Adapter React/Tauri affiancato a CLI e Slint; scaffold P3, non release utenti.
Manifest e lock separati; nessuna validazione o persistenza collegata.

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
finestra locale `main`, con ricontrollo Rust dell'origine. Sono registrate solo
`app_info` e `client_ready`; nessuna operazione file, SQL, validazione o storia.
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
