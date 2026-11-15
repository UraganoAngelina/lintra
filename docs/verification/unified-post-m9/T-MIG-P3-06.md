# T-MIG-P3-06 — Integrazione scaffold e regressione legacy

15 novembre 2026. **IN_PROGRESS — G-MIG-3 NOT_VERIFIED**.
P3-01/02/03/04/05 DONE nel rispettivo perimetro; gate tecnico aggregato ancora
aperto fino agli esiti effettivi Linux/Windows e regressione core+Slint.

Lo scaffold rimane in un workspace Cargo indipendente, con propri manifest,
lock, dipendenze frontend e capability. Nessun diff da 45fce38 su core, CLI,
Slint, test/golden, manifest/lock principali, `.cargo` o CI legacy. La permission
generata dell'adapter concede solo il dispatcher raw alla finestra locale
`main`; nessuna capability shell/fs/SQL o validazione collegata in P3.

Il workflow `desktop.yml`, condizionato ai path dell'adapter, integra i test
TS/asset, lint statico TypeScript, typecheck/build e test/fmt/Clippy/build Rust
su Windows Server e Ubuntu. Lo smoke nativo richiede exit 0 e l'ack del renderer
effettivo; Ubuntu usa Xvfb/X11 e rendering software, distinti dal Wayland del
PC personale e dal Win11 della VM. Nessun allentamento di CSP o sandbox WebKit.
Il piccolo harness Python possiede e attende il processo, applica timeout e
controlla marker/exit code senza acquisire schermate o usare dati reali.

Linux locale: 16 test Node, 11 Rust, lint/typecheck/build frontend, fmt/Clippy e
build release PASS. Smoke release React/raw IPC/Rust Wayland PASS, exit 0,
SHA binario `adb9dfe68df15b1903a7f0868a88c1bf0e6534813446777b37d5969d71031e98`.
La regressione `cargo test --workspace --locked --offline -j 2` è in corso.
La nuova CI desktop e la CI ufficiale core/Slint saranno attribuite allo SHA
runtime pubblicato; le prove precedenti non qualificano il nuovo adapter.

Review Ponytail sul confine IPC: letto il percorso renderer → API Tauri
installata → permission generata → origine nativa → codec → stato → risposta.
Corretto il producer Rust per applicare il limite durante la serializzazione,
anziché dopo l'allocazione completa. Coperti limiti, duplicati, Unicode,
versioni, shape, categoria errori, owner/chiusura e ordine handshake. Nessun
servizio normativo, storia, SQLite/CAS o chiave HMAC entra nel renderer.

G-MIG-4..9 e G-REL restano aperti. P4-01 richiede G-MIG-3 effettivo: nessun
adapter applicativo anticipato. Niente VM o nuova qualifica installer in P3.
