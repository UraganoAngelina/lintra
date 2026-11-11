# T-MIG-P1-01 — Spike runtime Linux

11 novembre 2026. **DONE per lo spike minimo Linux personale**. Prerequisito P0-08 DONE,
G-MIG-0 PASS governance su 925ef47. Nessun G-MIG-1 implicito.

Implementato `spikes/tauri-runtime`, workspace autonomo React / TypeScript strict
/ Vite / Tauri 2. Include soltanto font IBM Plex esistenti e geometria Lintra
già presente in Slint. Nessuna libreria UI o framework alternativo aggiunto.
Versioni dirette fissate; lockfile separati; workspace, lockfile, binari e
comportamenti core/CLI/Slint non modificati. Non è il frontend applicativo P3.

Il README dello spike spiega funzioni, struct, condizioni, necessità delle
dipendenze e procedura. Due soli comandi sintetici con ACL esplicita e controllo
Rust della finestra. Risposta costante reale, poi acknowledgment numerico dopo
render React e font pronti; `--smoke-test` esce solo dopo tale acknowledgment.
Non viene invocato né simulato il validatore, non vengono aperti file o archivi
e non viene acquisito alcun dato clinico.

Il preflight ha rilevato WebKitGTK 4.1 mancante. La transazione minima proposta
è documentata nel README; `sudo -n` si è fermato sulla password del sistema.
L'autore ha eseguito l'installazione nel proprio terminale e risposto
«Installazione completata». Verificate le cinque versioni installate e le
librerie pkg-config in `T-MIG-P1-01-context.json`. Non è stata richiesta o
registrata la password. Non è stato eseguito un upgrade generale dell'host.

Verifiche iniziali: build frontend PASS con 17 moduli e asset locali; TypeScript
strict, `skipLibCheck=false`; fmt Rust PASS; detector UI zero rilievi; diff check
PASS. Il primo typecheck segnalava due dichiarazioni mancanti: risolte con
ESNext.Disposable e tipi Vite client già inclusi negli strumenti approvati,
senza ignorare errori o aggiungere runtime/polyfill. La build Rust era ancora in corso nel preflight; esiti finali sotto.

Reference aggiuntive dell'autore ricevute in chat e consultate con vision,
con hash in context.json; brief UX aggiornato con l'uso previsto. Cartella
esterna preservata. I concept di altri framework sono riferimenti di layout,
non scelte tecniche né dati/regole da trasferire nel prodotto.

## Qualifica nativa effettiva

Codice applicativo nel commit `996947e48517dc61caea9c9eafaaf2220e04766f`,
invariato dopo le prove. Versioni: Rust 1.99.1, Node 26.13.0, npm 12.2.2,
React/DOM 19.3.1, TypeScript 7.0.2, Vite 8.3.8, Tauri Rust 2.12.5, build 2.7.4,
CLI/API 2.12.4. Host CachyOS rolling / Wayland / Hyprland 0.57.2.
WebKitGTK/JSC 2.54.2, GTK 3.24.52, libsoup 3.8.1.

Prima build nativa PASS in 9m48s. Build registrata successiva lock/offline,
incluso frontend strict: [build-v1](T-MIG-P1-01-build-v1.json), exit 0.
[Clippy all-targets](T-MIG-P1-01-clippy-v1.json), stessa feature
custom-protocol, `-D warnings`: PASS. Rust fmt e diff check PASS.

[Smoke Wayland reale](T-MIG-P1-01-wayland-v1.json): render React/IPC Rust
verificati, viewport 873×1138 imposta dal tiling host, exit 0, durata 2,195s.
[Capture nativa](T-MIG-P1-01-capture-v2.json): stessa identità binaria,
apertura, focus e chiusura effettivi della sola finestra sintetica, exit 0.
[PNG osservato con vision](T-MIG-P1-01-native-window.png), SHA-256
`554bcc62c9f7d8f29700362ff10a2a110db4c1b9ac74230db5421246a81f52fb`.
Testo leggibile, font/asset locali, nessun contenuto di altre applicazioni.
I log gzip e gli inventari nei JSON conservano input/digest; nessuna mutazione
durante le singole prove. I nuovi harness sono qualificati dall'inventario
working tree, senza attribuirli falsamente al commit iniziale dell'applicazione.

Binario 21.886.344 byte, SHA-256
`c53fd4ba3d5c7a1409dd4a62a9fc2a5c46cdfd0f271e0159fe7bd0e1c71e6883`.
Profilo **debug**, feature `tauri/custom-protocol` attiva: asset inclusi,
nessun dev server. [Inventario runtime](T-MIG-P1-01-native-runtime.json):
144 righe ldd, nessuna libreria mancante, feature e versioni effettive.
Le librerie OS sono dinamiche; nessuna portabilità universale o sostituzione
N-20 core/CLI. Profilo release, rete/capability negative complete, DPI e
parità applicativa rimangono alle rispettive task. Non è il frontend P3.

Il primo harness di cattura usava la sintassi Hyprland precedente:
fallimento conservato in capture-v1; corretto alla API Lua documentata di
0.57, senza cambiare il binario, quindi capture-v2 PASS. I due harness
foreground riproducibili sono nello spike; usano stdlib Python e, per la sola
capture personale, Hyprland/grim già installati, senza nuove dipendenze prodotto.
Il context.json conserva lo stato iniziale NOT_RUN come evidenza storica;
questi risultati ne documentano il superamento, senza riscrivere il preflight.

## Non regressione e handoff

Tutti i 520 input di produzione qualificati cb70bee sono byte-identici;
consuntivo originale 140/140 e fonti normative preservati. Nessuna nuova CI
core necessaria per questo spike isolato Linux; la CI antecedente non è una
prova dei nuovi sorgenti. Evidenze native pertinenti eseguite sul PC personale.
Windows NOT_RUN; niente PASS NSIS/offline, non-admin/due account, release o
distribuzione ai colleghi. **G-MIG-1 NOT_VERIFIED**. Nessun gate finale o
permesso sui dati implicito. Slint e rollback conservati.

Next READY **T-MIG-P1-02**; P1-03 ha ora il prerequisito soddisfatto.
P1-04/05 e GO/NO-GO complessivo rispettano le proprie dipendenze. AUTO_CONTINUE.
