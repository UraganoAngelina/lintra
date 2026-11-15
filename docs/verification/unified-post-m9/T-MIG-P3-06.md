# T-MIG-P3-06 — Integrazione scaffold e regressione legacy

15 novembre 2026. **DONE — G-MIG-3 PASS**, 15 novembre 2026; perimetro e prova finale sotto.
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
La regressione `cargo test --workspace --locked --offline -j 2` è PASS: 1110 esiti PASS riportati (inclusi helper subprocess), 9 ignored previsti, 129 riepiloghi suite/doc suite, exit 0. Log originale compresso in `T-MIG-P3-06-legacy-local.log.gz`.
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

Prima pubblicazione runtime f482918; CI desktop43757790267: frontend Windows
15/16 PASS, fallimento asset CRLF, non fallimento del wire. Log originale
conservato in `T-MIG-P3-06-windows-first.log.gz`. Linux dello stesso run ancora
in corso. La correzione dei soli attributi checkout è verificata localmente
con `core.autocrlf=true`; nessun DTO/decoder/guard o expected è cambiato.
La CI ufficiale43757791728 è partita su f482918: non qualifica automaticamente
il checkout corretto. Il gate richiede gli esiti pertinenti del nuovo SHA.

Il filtro path del workflow desktop include ora `.gitattributes`: una modifica
alla preservazione dei byte deve attivare le stesse prove dell'adapter.

Checkout corretto pubblicato su ce344d4. CI desktop43758529844 e ufficiale
core/Slint43758531087 in corso sullo stesso SHA. La run legacy43757791728
è stata annullata automaticamente dalla concurrency già prevista nel workflow
al dispatch della nuova run; nessuna cancellazione manuale è stata eseguita.
Il primo desktop Linux43757790267 è SUCCESS; il workflow complessivo resta FAIL
per l'asset Windows. Nessun errore passato viene cancellato o trasformato in PASS.

CI desktop43758529844 **SUCCESS**, Linux e Windows sullo SHA ce344d4:
16 test Node per OS, 11 test Rust Linux/12 Windows, fmt/Clippy/build release e
smoke React/raw IPC/Rust entrambi PASS. Log completo compresso in
`T-MIG-P3-06-desktop-ci.log.gz`; SHA binari e metadati/step effettivi nel receipt.
I log conservano i warning del bus accessibilità Xvfb assente e del cleanup
WebView2 Server (class unregister 1412), con ack e process exit 0; nessuna
qualifica a11y, Win11 desktop o installer viene dedotta da questi smoke.
Resta soltanto la CI ufficiale core/Slint43758531087 per G-MIG-3: nessun PASS
aggregato prima del suo completamento e nessun avvio applicativo P4 anticipato.

Ultima lettura ordinaria CI ufficiale: ancora IN_PROGRESS; Linux a “Test all
features”, Windows a “Test”, senza failure riportate fino a questa lettura.
Non è un esito finale. Handoff **WAITING_GATE**, 52/128 DONE; grafo verificato:
nessuna nuova task indipendente con tutti i prerequisiti DONE. Tutti i processi
locali posseduti sono terminati/attesi. Next P4-01 dopo G-MIG-3 PASS.

## Chiusura sul perimetro approvato G-MIG-3

Ripresa dell'autore: proseguire lo sviluppo e mostrare soltanto blocchi reali.
Il piano sorgente §10 assegna a G-MIG-3 QG-1/2/3: Slint+CLI verdi, scaffold,
frontend lint/typecheck/build e CI smoke Linux+Windows. L'acceptance P3-06
richiede cargo test core+Slint e smoke nativo. L'attesa dell'intera pipeline
ufficiale, che comprende anche artefatti release, snapshot cross-OS e confronti
ulteriori, era un checkpoint più ampio di questa fase. Nessun requisito, test,
workflow o criterio del piano viene modificato per correggere tale attesa.

Verificati da API ufficiale su ce344d4, Linux e Windows MSVC: formatting,
Clippy, test workspace, Clippy/test all features, test core senza bundle e
confine workspace/core tutti SUCCESS, con singoli step originali nel receipt.
CI desktop43758529844 già SUCCESS su entrambi gli OS, inclusi smoke nativi.
Prove locali native e tutti i prerequisiti P3 già PASS. **G-MIG-3 PASS e
P3-06 DONE**, 53/128. La CI ufficiale43758531087 è ancora IN_PROGRESS nelle
ulteriori prove: non viene dichiarata interamente SUCCESS. Quelle prove e i
gate futuri restano obbligatori nei rispettivi checkpoint. AUTO_CONTINUE P4-01.
