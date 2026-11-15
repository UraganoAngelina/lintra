# Lintra desktop

Adapter React/Tauri affiancato a CLI e Slint; scaffold P3, non release utenti.
Manifest/lock separati. Nessuna validazione o persistenza collegata in P3-01.
Versioni copiate dallo spike qualificato, senza nuovi pacchetti registry.

```sh
npm ci --ignore-scripts --no-audit --no-fund
npm run build
npm run tauri -- build --no-bundle -- --locked -j 2
```

Asset/IPC interni soltanto, navigazioni e popup negati, guard WebView2
riusato da P1-04. Nessuna capability custom ancora concessa. Il workspace
principale conserva i comandi di build/test della CLI e della Slint.
