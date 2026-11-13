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


## P1-04: origine, CSP e prova di attacco nativa

La finestra `probe` ha `create: false`: il setup Rust la costruisce esplicitamente
con i guard già collegati, prima del primo documento. `local_asset_origin`
confronta schema/host/credenziali/porta, non un prefisso URL: ammette soltanto
`tauri://localhost` e `http://tauri.localhost`, le origini della versione pinnata.
`authorize` controlla anche l’URL effettivo oltre alla label. `on_navigation`
nega le altre origini, `on_new_window` nega ogni nuova finestra. I contatori
atomici osservano i callback e non trasportano URL o dati arbitrari nei log.

`--security-test` abilita una fixture sintetica tramite initialization script
nativo. La modalità ordinaria conserva UI e acknowledgment precedenti. Non
vengono aggiunti comandi/capability: il report opzionale di `render_ack` ha
campi booleani e un conteggio limitato, rifiuta campi sconosciuti e viene
accettato soltanto nella modalità di qualificazione. `SecurityReport::passed`
richiede tutti i controlli; Rust richiede inoltre callback reali di rifiuto
navigazione/popup. Protocollo, tipi, overflow e dimensioni errate sono testati;
la risposta errata per un motivo diverso da quello atteso non soddisfa il test.

`src/security_probe.ts` verifica il testo ostile reso letteralmente da React,
script inline/remoti, CSS, font, immagini, frame, object, fetch, WebSocket,
EventSource, beacon e form, con eventi CSP effettivi. Una connessione fallita,
un’API assente o un errore CORS non contano come rifiuto CSP. Verifica anche
invoke lecito, comandi fs/shell/window/webview assenti o non autorizzati e
payload invalidi. Le prove usano esclusivamente il collettore loopback locale,
mai endpoint Internet o dati clinici.

Il browser può bloccare un popup scriptato prima del callback Rust. Per
verificare anche quel callback, una fixture button riceve un evento OS trusted:
il renderer la focalizza e invia un handshake incompleto, rifiutato da Rust con
un marcatore fisso di readiness. Il harness dirige Enter soltanto alla finestra
del PID figlio verificato (Hyprland `send_shortcut` sul CachyOS personale;
foreground PID verificato e SendKeys nel desktop standard Windows). Un click
JS non trusted non soddisfa il controllo. Nessuna configurazione host cambia.

`tools/native_security.cjs` possiede GUI e server temporanei nel medesimo
processo foreground e li conclude prima di terminare. Prima verifica che il
collettore `127.0.0.1:49183` risponda; durante la prova richiede **zero
connessioni e richieste**, ACK unici, exit 0 e contatori Rust positivi.
Mantiene output e hash del binario in JSON senza sovrascrivere prove esistenti.
I percorsi XDG Linux sono privati ed effimeri. Windows usa il profilo reale
dell’account standard, con sessione desktop verificata dal launcher.

```sh
npm run build
cargo build --manifest-path src-tauri/Cargo.toml --locked --offline --features tauri/custom-protocol -j 2
node tools/native_security.cjs src-tauri/target/debug/lintra-runtime-spike /tmp/lintra-security-result.json
```

Questa è una verifica selettiva dello spike P1-04, non un nuovo gate ricorrente.
Il collettore osserva le richieste delle prove, non tutto il traffico dell’OS o
telemetria del redistributable Microsoft. Non qualifica l’intero futuro prodotto,
PHI/cache/dump, Job Manager, persistenza, native file broker o release. G-MIG-1
richiede ancora DPI/installer e la decisione di fase previste dal piano.


### Confine risorse WebView2 rilevato sulla VM

Su WebView2 157, un `NavigationStarting` cancellato manteneva la pagina locale
ma il collettore osservava comunque un GET del caso `/navigation`. Gli eventi
CSP e il solo rifiuto del cambio pagina non dimostravano quindi zero richieste.
`src-tauri/src/windows_request_guard.rs` aggiunge un filter WebResourceRequested
su tutti i contesti: ammette soltanto le origini asset e IPC interne esatte,
restituisce una risposta locale403 senza contenuto per gli altri URI e non
registra URL o payload. I callback ricevono oggetti COM validi dal thread Tauri;
le copie dell’environment sono possedute, il buffer URI viene liberato dal
medesimo helper CoTaskMem della Wry pinnata. Errori di installazione/risposta
falliscono esplicitamente e impediscono un ACK di successo.

Per questa necessità concreta sono dichiarati direttamente `webview2-com
0.39.1` e `windows0.62.2`, **gli stessi pacchetti già presenti nel lockfile e
nella Wry selezionata**: nessuna versione o famiglia di dipendenze nuova,
nessuna dipendenza GUI nella library/CLI. Un terzo test verifica il confine
asset/IPC e gli origin lookalike. Il modulo SDK resta isolato con cfg Windows;
il protocollo e le decisioni cliniche del core non sono coinvolti.

Il driver registra separatamente connessioni, byte e il solo nome noto del
caso sintetico HTTP. Un renderer che segnala rifiuti ma invia richieste resta
FAIL. Il timeout del helper OS è20s e la finestra di input trusted30s, per
accogliere il primo caricamento PowerShell della VM senza cambiare le policy.
La prova continua a richiedere zero connessioni/richieste, non un mero exit0.

## P1-05: profilo WebView, font e scala nativa

La modalità opt-in `--profile-test` misura il documento dopo caricamento dei
font e confronta DPR, viewport e geometria con scala/dimensioni lette dal
backend. Rifiuta misure incoerenti e font non caricati; non cambia lo stile
ordinario e non promette pixel parity fra motori. È esclusiva rispetto alla
modalità security. Esecuzione mirata sul desktop Linux già configurato:

```sh
node tools/native_profile.cjs src-tauri/target/debug/lintra-runtime-spike /tmp/lintra-profile-result.json
```

Il harness verifica anche che il profilo WebView sia realmente presente nella
root per-account risolta da Tauri; Linux usa XDG temporanee private. Il profilo
dello spike usa `it.lintra.runtime-spike`: non implementa ancora la root dati
`Lintra` del futuro prodotto. Su Windows il launcher
`qualification/windows-profile-desktop.ps1` richiede la console reale con
token non elevato, confronta `GetDpiForWindow`, acquisisce la sola finestra e
la chiude. SSH prepara/raccoglie la prova; sessione0 non soddisfa il launcher.
I risultati, gli hash e i limiti delle scale effettivamente osservate sono nel
[dossier P1-05](../../docs/verification/unified-post-m9/T-MIG-P1-05.md).
Questi strumenti sono specifici dello spike, senza nuovi gate Windows ricorrenti.
