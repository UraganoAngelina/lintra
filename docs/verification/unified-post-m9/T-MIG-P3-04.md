# T-MIG-P3-04 — Stato Rust e dispatcher

15 novembre 2026. Prerequisiti P3-01 e P3-02 verificati. Dispatcher separato
dal core: nessuna dipendenza applicativa Tauri nel workspace principale,
nessun servizio di validazione/persistenza collegato.

`AppState` possiede la chiusura della finestra e l'handshake iniziale. La
permission generata concede soltanto `lintra_ipc` alla finestra locale `main`;
il dispatcher ricontrolla label, URL nativo e guard WebView2 prima del parsing.
Un mutex try-lock ammette un solo parsing/risposta; nessuna coda applicativa.
`app_info` espone solo versione e indisponibilità della validazione;
`client_ready` richiede un precedente `app_info` valido. Payload esatti:
soltanto `schemaVersion: 1`. Chiusura, lock occupato, origine o operazioni
illecite producono errori bounded; nessuna stringa diagnostica libera.

`ipc.rs` applica il contratto ratificato: body raw, UTF-8 strict, lunghezza
intera 1 MiB, profondità 32 prima della deserializzazione, duplicati anche
escapati rifiutati, interi safe JS e versioni esplicite. La risposta usa un
writer che interrompe la serializzazione al limite senza pubblicare prefissi;
gli errori distinguono protocollo da tecnica e non simulano esiti normativi.
Il buffer framework può essere già allocato: non è un limite globale anti-flood.

Smoke shell nativo CachyOS/Wayland PASS: documento locale caricato e processo
terminato con exit 0, marker `LINTRA_DESKTOP_SHELL_READY`. Directory XDG
temporanee private, nessuna capture o dato reale. Il tentativo nel sandbox
non accedeva a GTK (exit 101); ripetizione con accesso al compositor PASS.
Receipt: [controlli](T-MIG-P3-04-checks.json). Handshake React/Rust e decoder TS
seguono in P3-05; G-MIG-3 e Windows sul nuovo delta attendono P3-06.
