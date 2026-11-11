# Spike runtime React / Tauri 2 — T-MIG-P1-01

Prova tecnica isolata, successiva a G-MIG-0. Non è il frontend di produzione e
non modifica il workspace core/CLI/Slint, i suoi lockfile o le regole normative.
Usa soltanto i componenti dello stack già approvato. La UI applicativa completa,
i componenti shadcn/TanStack e il bridge di validazione appartengono alle fasi
successive e non vengono anticipati da questo controllo del runtime.

## Dipendenze e necessità

- React / React DOM: rendering effettivo nel WebView, senza framework alternativi.
- TypeScript strict e relativi tipi React: verifica del renderer in compilazione.
- Vite: produce JS/CSS/font locali; nessun server o CDN serve all'eseguibile.
- Tauri Rust / build / CLI / API: host nativo, asset inclusi e IPC effettivo.
- Serde / serde_json: payload sintetico tipizzato e acknowledgment verificabile;
  stesse versioni dirette del repository, nessuna nuova famiglia crittografica.
- Nessun plugin filesystem, shell, HTTP, updater, libreria icone o UI aggiuntiva.
  Le dipendenze transitive necessarie degli strumenti sono nei lockfile.

Sul solo host CachyOS il preflight ha rilevato GTK 3.24.52 e l'assenza di
WebKitGTK 4.1. Il runtime Linux di Tauri richiede quest'ultimo secondo le
[istruzioni ufficiali](https://v2.tauri.app/start/prerequisites/).
La transazione proposta dal package manager locale è:

| Pacchetto OS | Versione proposta | Necessità |
|---|---|---|
| webkit2gtk-4.1 | 2.54.2-1 | WebView Tauri Linux |
| enchant | 2.8.21-1.1 | Dipendenza obbligatoria WebKitGTK |
| libmanette | 0.2.13-2.1 | Dipendenza obbligatoria WebKitGTK |
| woff2 | 1.0.2-7.1 | Dipendenza obbligatoria WebKitGTK |
| xdg-dbus-proxy | 0.1.9-1.1 | Dipendenza obbligatoria WebKitGTK |

È una dipendenza di piattaforma prevista dalla scelta Tauri, non una nuova
libreria del prodotto. L'installazione richiede il permesso di sistema; non
è stato richiesto un aggiornamento generale dell'host. La lista effettiva e gli
esiti vanno registrati nel dossier, senza presumere il successo dal preflight.

## Build e prova

Dal presente percorso:

```sh
npm ci --ignore-scripts --no-audit --no-fund
npm run tauri -- build --no-bundle --debug -- --locked --offline -j 2
python3 tools/native_smoke.py src-tauri/target/debug/lintra-runtime-spike
```

La modalità normale apre la finestra; `--smoke-test` esce con codice zero solo
dopo un render React, font locali pronti, due animation frame e una seconda
chiamata Rust che registra `LINTRA_RUNTIME_RENDER_ACK` con dimensioni numeriche.
Un timeout, un crash o una pagina senza acknowledgment non costituiscono PASS.
Questo segnale non dimostra confronto pixel, DPI, zero traffico OS o gate Windows.

La prova effettuata usa il profilo debug con la feature `tauri/custom-protocol`
attiva: tutti gli asset sono inclusi nell'eseguibile, senza dev server. Non è
una qualifica release. Su Hyprland 0.57 la sola finestra sintetica può essere
acquisita e chiusa dal programma foreground seguente, usando `hyprctl` e `grim`
già installati sull'host (nessuna dipendenza del prodotto):

```sh
python3 tools/native_capture.py src-tauri/target/debug/lintra-runtime-spike /tmp/lintra-probe-window.png
```

I due harness isolano cache/config/dati WebView in cartelle temporanee private.
La cattura filtra per PID figlio, verifica il titolo e il focus e ritaglia solo
la finestra. Nessun titolo di altre applicazioni viene registrato. L'API Lua
segue la [documentazione Hyprland 0.57](https://wiki.hypr.land/0.57.0/configuring/core/dispatchers/).
L'icona riprende la geometria di `ui/ui/main.slint`; il PNG è generato con il
comando `tauri icon src-tauri/icons/mark.svg --output src-tauri/icons --png 128`.

## Codice e confini

`RuntimeProbe` distingue attesa, risposta costante verificata ed errore, rende
solo testo React e permette il cambio tema senza dati persistenti. L'effetto
di render non acquisisce file o contenuti del desktop. I due font IBM Plex
provengono dagli asset locali già presenti nel repository.

`runtime_probe` verifica la finestra chiamante e restituisce un marcatore fisso.
`render_ack` accetta un DTO con campi esclusivamente numerici, rifiuta campi
extra, protocollo diverso o dimensioni fuori intervallo e stampa soltanto tale
DTO sintetico. `ProbeOptions` abilita l'uscita automatica solo per il flag CLI
esplicito; non è un comando di chiusura generico esposto al renderer.

`build.rs` usa `AppManifest::commands` per rendere applicabile l'ACL anche ai
custom commands. La capability locale concede solo questi due comandi alla
finestra `probe`, oltre al controllo Rust della label. La CSP nega le fonti
remote e abilita solo asset locali e il protocollo IPC Tauri. La qualifica
negativa completa delle permission e della rete rimane T-MIG-P1-04.

## Reference visive fornite dall'autore

L'11 novembre 2026 l'autore ha indicato la cartella esterna in sola lettura
`/home/alberto/Desktop/LINTRA/lintra_ui_references`, richiedendo esclusivamente
React/Tauri o componenti già approvati, con necessità documentata per ulteriori
dipendenze. Sono stati esaminati mediante vision il concept Tauri, la galleria
comparativa e i concept Dioxus/GPUI come riferimenti di layout. Nessuno degli
altri framework viene introdotto. I duplicati `(1)` hanno hash identici.

Le immagini suggeriscono navigazione laterale, risultati tabellari e inspector
contestuale; non attestano runtime, regole, conteggi o conformità effettivi.
La palette e i font Copper Ledger già approvati restano la fonte applicativa.
Il brief di migrazione registra le modalità di adozione per la futura UI.

Versioni Tauri fissate consultando le
[release ufficiali](https://github.com/tauri-apps/tauri/releases).
Configurazione e ACL seguono la documentazione ufficiale di
[configurazione](https://v2.tauri.app/reference/config/),
[custom commands](https://v2.tauri.app/develop/calling-rust/) e
[capabilities](https://v2.tauri.app/security/capabilities/).
