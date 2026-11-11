# Runtime GUI personale CachyOS

T-MIG-P1-03, 11 novembre 2026. Profilo locale qualificato per lo spike minimo
React/Tauri: **GO personale**, non una release Linux GUI ufficiale e non il GO
aggregato G-MIG-1. La policy GUI-only è già ratificata in ADR-MIG-002/D14;
N-19/N-20/N-23 del core/CLI restano invariati.

La distribuzione personale testata consiste nell'eseguibile con gli asset
React/CSS/font incorporati, su questo ambiente con i pacchetti OS completi
registrati. Nessun Node, npm, Git, Cargo o server di sviluppo serve all'avvio
dell'eseguibile. La build richiede invece la toolchain documentata nello spike.
Non si consegna una raccolta manuale di sole `.so`, AppImage o pacchetto deb.

## Ambiente e inventario

CachyOS rolling x86_64 / Hyprland 0.57.2 / Wayland. GTK 3.24.52,
WebKitGTK/JSC 2.54.2, libsoup 3.8.1, GLib 2.90.1. Runtime Tauri 2.12.5;
profilo provato debug con `custom-protocol` e devtools disabilitati.
L'installazione iniziale di WebKitGTK e quattro dipendenze obbligatorie è stata
eseguita dall'autore dopo preflight; nessuna dipendenza aggiuntiva nascosta.

Il [manifest completo](../verification/unified-post-m9/T-MIG-P1-03-runtime-profile.json)
registra 151 ELF con dimensioni e SHA-256 e 99 pacchetti OS con versioni/licenze
dichiarate dal package manager. Include la chiusura ldd del probe, dei processi
WebKit Web/Network/GPU, injected bundle e helper bubblewrap/xdg-dbus-proxy.
Nessuna libreria mancante; nessun RPATH/RUNPATH al checkout o all'host di build.

Pacchetti interi e relative risorse GTK/WebKit devono restare installati.
Driver, plugin multimediali/accessibilità e integrazione desktop caricati
dinamicamente appartengono al profilo OS personale: l'inventario ELF non
autorizza estrazione selettiva di una presunta distribuzione universale.
Non si supportano per analogia altre distro, ABI o altri host. Dopo aggiornamenti
rolling di ABI/WebKit/GTK/driver, ripetere build/smoke e registrare un profilo
nuovo. Il manifest è una fotografia, non un vincolo a disabilitare aggiornamenti OS.

## Riproduzione e prove

Dalla radice repository, dopo la build P1-01:

```sh
python3 scripts/tauri_runtime_inventory.py spikes/tauri-runtime/src-tauri/target/debug/lintra-runtime-spike /tmp/lintra-runtime-profile.json
python3 scripts/tauri_runtime_relocated_smoke.py spikes/tauri-runtime/src-tauri/target/debug/lintra-runtime-spike
python3 scripts/tauri_runtime_missing_library.py spikes/tauri-runtime/src-tauri/target/debug/lintra-runtime-spike
```

Gli script Linux di qualifica sono in `scripts/`, separati dagli input del
binario Windows: una loro modifica non deve rilanciare la build NSIS. Non
sono dipendenze o programmi avviati dal prodotto. Usano solo stdlib Python e
strumenti OS già presenti; nessun nuovo package npm/Cargo o framework GUI.

La prova positiva copia il binario byte-identico in una cartella privata fuori
dal checkout e cambia CWD. Il render/IPC Wayland riesce con asset incorporati,
directory WebView private temporanee e uscita zero. Non dipende da path di
build per leggere font o sorgenti al momento dell'avvio.

La negativa monta la root host in sola lettura nel namespace privato bubblewrap
e rende inaccessibile solo lì la libreria WebKit. Il loader segnala esattamente
`libwebkit2gtk-4.1.so.0`, esce 127 e non avvia il probe: nessun fallback fittizio.
L'hash della libreria host prima/dopo è invariato. Non si rinominano o cancellano
librerie dell'utente e non si cambiano i permessi del sistema.

## Limiti residui

P1-03 chiude l'inventario e la riproducibilità locale del runtime minimo con
prova positiva/negativa. Non qualifica il futuro frontend applicativo, IPC del
validatore, root dati privata Tauri, Full/CAD, release o distribuzione colleghi.
CSP/rete negative complete e profilo WebView/font/DPI restano P1-04/05 su entrambe
le piattaforme; G-MIG-1 è bloccato dalla prova Win11 indisponibile. Nessun
ambiente Windows è simulato per superare il gate.
