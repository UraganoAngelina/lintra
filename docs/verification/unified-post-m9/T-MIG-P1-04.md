# T-MIG-P1-04 — CSP, capability e navigazioni native

13 novembre 2026. **BLOCKED — zero-network Windows**, G-MIG-1 aggregato NOT_VERIFIED.
Prerequisiti P1-01/P1-02 DONE; ingresso documentale 7f165e5. P1-02 resta
qualificata sull’artefatto bdca1cf; questa task modifica il runtime dello spike
e richiede proprie prove native/CI, senza attribuirsi la CI del codice vecchio.

Preflight: CSP locale già restrittiva e due comandi custom opt-in via
AppManifest/capability `probe`. Mancava però una decisione nativa esplicita
sulle navigazioni e sulle nuove finestre: la CSP non costituisce da sola una
policy di navigazione top-level. ADR-MIG-003 richiede quel confine reale.

Delta nello spike isolato: finestra costruita con callback prima del primo
documento, sole origini asset Tauri pinnate, nessuna nuova finestra, controllo
di label e origine prima dell’handler. Nuova modalità sintetica
`--security-test`, stessi due comandi e stesso allowlist, report wire a forma
fissa/bounded senza path, URL, record o mappe libere. Frontend di produzione,
core/CLI/Slint, norme, golden e persistenza non modificati.

La prova deve osservare eventi CSP reali per script/inline/style/font/image/
frame/object/connect/form, invoke leciti/negati, payload invalidi e contatori
nativi dei guard. Collettore loopback raggiungibile prima del lancio e zero
connessioni/richieste durante la prova; rete irraggiungibile da sola non è PASS.
Processi GUI e server sono posseduti e conclusi dal harness in foreground,
senza servizio permanente né dipendenza npm nuova.

Esiti iniziali: frontend TS strict/Vite PASS; due test Rust sui veri confini
origin/protocollo/dimensioni/deserializzazione PASS; build e Clippy isolati
Linux PASS. Smoke ordinario e security native Wayland PASS: 12 violazioni CSP, guard
navigazione e popup esercitati con evento OS trusted, zero connessioni e
richieste al collettore raggiungibile. Primo tentativo script-only mantenuto
FAIL perché il browser bloccava il popup prima del callback Rust; handshake
e input OS alla sola finestra figlia hanno risolto la prova senza indebolire
i guard. Gli esiti Windows successivi e la CI sono riportati sotto. Nessun DONE, release o gate aggregato anticipato.

Finding ereditato e distinto da P1-02: il redistributable Microsoft WebView2
ha eseguito un processo `/ping`, bloccato dal firewall guest nella prova offline.
Questo non è traffico applicativo dimostrato, né una prova di zero tentativi
del software Microsoft. P1-04 deve distinguere policy del renderer/adapter,
runtime browser e installazione/OS; non promettere un isolamento universale
dell’intero sistema dalla sola CSP.


Prova Linux finale: `linux-acl-reasons.json`, binario SHA256
`5d153a7afac5b6e8e9e87b2cf8f04137324169211a4cd84955f59a3f1ac50254`,
12 eventi CSP, un rifiuto nativo di popup trusted e uno di navigazione,
zero connessioni/richieste al collettore raggiungibile, exit 0. Gli invoke
negativi devono contenere `not allowed` emesso dall’ACL Tauri: un errore I/O
su file/programmi inesistenti non soddisfa la prova. Output corrente e tentativi
precedenti distinti in `T-MIG-P1-04-native/`.

VM: stable disponibile1.100.0; aggiunta affiancata1.99.1 MSVC fissata dal repo,
nessuna sostituzione/aggiornamento di stable o Node. Sorgenti locali non
committati e `dist` web neutrale compilato con Node26.13.0 trasferiti tramite
allowlist di28 file e manifest SHA; niente target/node_modules/credenziali.
Compilazione solo dello spike con Cargo1.99.1, lockfile e target MSVC, non
suite core né installer VM. Frontend non ricompilato con versioni Node diverse.
CI spike estesa soltanto ai due test puri origin/wire nel job già esistente;
nessun nuovo gate o matrice obbligatoria generato dalla disponibilità della VM.


## Esiti Windows e correzione del confine HTTP

La VM è stata usata perché WebView2 e gli eventi nativi di Windows non sono
riprodotti da WebKitGTK. Account `DESKTOP-RLLAD2M\lintra-agent`, token standard,
sessione console desktop1 verificata, task pianificata Interactive/Limited.
Lo smoke ordinario ha prodotto il vero ACK900×600 ed exit0. L'input popup è
stato inviato soltanto dopo aver verificato il PID della finestra in foreground;
non è stato dedotto un PASS grafico dal semplice lancio SSH.

Il primo tentativo ha evidenziato un timeout troppo breve del helper PowerShell;
20s per il helper e30s per l'input trusted risolvono questo errore della prova.
Il tentativo successivo supera i18 controlli del renderer,12 eventi CSP, ACL
con motivo `not allowed`, deserializzazione e guard nativi, ma invia realmente
un **GET `/navigation`** al collettore. La sola cancellazione NavigationStarting
non realizza dunque il confine HTTP richiesto sul runtime provato. L'esito è
FAIL e rimane conservato, anche se il processo Rust termina con0.

La correzione in `windows_request_guard.rs` registra WebResourceRequested per
tutti i contesti e sorgenti prima che gli invoke siano ammessi. Consente
esclusivamente gli origin asset/IPC interni esatti, senza credenziali o porte
aggiuntive. Per ogni altra URI produce localmente403 con corpo vuoto e no-store;
URI non valide ed errori SDK sono fail-closed. I contatori di readiness/failure
impediscono ACK positivi senza filtro attivo. Controller ed environment COM sono
usati sul thread proprietario fornito da Tauri; il callback possiede i riferimenti,
la memoria URI è liberata con il medesimo helper della Wry pinnata. L'unsafe è
circoscritto a questo adapter Windows, senza estenderlo al core.

Sono dichiarate direttamente le versioni **già transitive** webview2-com0.39.1 e
windows0.62.2, senza nuove versioni o famiglie nel lockfile. Tre test Rust Linux e
MSVC, build release MSVC e Clippy `-D warnings` PASS. Il manifest v3 identifica29
sorgenti/asset, comprese modifiche non committate e `dist` generato dal Node host
fissato; ogni hash è verificato prima/dopo l'overlay controllato sul guest. Nessun
`target`, node_modules, segreto o dato sanitario è stato trasferito nei sorgenti.
Il `source_head`7f165e5 del manifest è la base della raccolta con delta locali,
non il commit finale: l'identità degli input è la lista SHA completa.

Binario Windows corretto: SHA256
`121363a6800410d3635ec4c908f1a69b2a5a4b59b09a37395117a6adb07c81a1`,
8984064byte, Rust1.99.1 MSVC, Win11 build26300, WebView2 157.0.4341.46.
La prova desktop dopo la correzione osserva tutti i18 flag veri,12 eventi CSP,
un rifiuto di navigazione, uno di popup e uno di risorsa; ACK e processo exit0,
**zero richieste HTTP ma3 connessioni TCP, tutte con0 byte letti/scritti**.
Il driver e il launcher restituiscono correttamente1/FAIL: il criterio zero
connessioni non è stato allentato. I PID delle connessioni sono registrati.
Questi risultati non provano l'assenza universale di traffico OS/runtime.

**Blocco ristretto:** resta da ottenere un contenimento riproducibile delle
connessioni Windows prima del trasporto con API supportate, oppure sottoporre
una decisione materiale sul runtime/contenimento senza inventare eccezioni.
La causa speculativa è un'ipotesi compatibile con connessioni prive di dati,
non una classificazione certa ricavata dal solo netstat. Non si impiegano flag
Chromium instabili, proxy fittizi o firewall della prova per simulare il PASS.
Microsoft stessa sconsiglia i browser flag WebView2 in produzione:
[documentazione ufficiale](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/webview-features-flags).
La policy NetworkPredictionOptions di Edge browser non è per questo dimostrata
applicabile a WebView2. Nessuna modifica dei controlli dell'host.

Linux aggiornato: `linux-resource-allowlist.json`, SHA binario
`d4315866c0d6d3550e9b053bb8ee65441f50b4afa9ab50c23c934be4a9c9e05e`,
18 flag veri/12 eventi CSP, guard navigazione e popup positivi,0TCP/0HTTP,exit0.
Non è stato ripetuto un test Windows ordinario per abitudine: queste prove
rispondono al malfunzionamento specifico del confine nativo.

CI43437870531 **SUCCESS su337697df9c1f8ff9c25e8a82234acb8873bc7f44**:
unit test, Clippy, release MSVC e NSIS. Questa CI precede il filtro SDK e **non
qualifica quel delta**; la nuova CI deve essere letta sul commit della correzione.
I suoi log non attestano la prova desktop, che rimane un'evidenza distinta.

Il checkpoint `LINTRA-P1-04-BEFORE-DESKTOP-20261113` è stato creato a VM spenta
per preservare toolchain/cache e sorgenti v2 prima di password e automazioni.
La correzione v3 e tutte le prove sono raccolte sull'host prima del ripristino;
il ripristino deve eliminare marker successivo, directory credenziali DPAPI,
Autologon amministrativo e task, preservando gli accessi standard/admin. Il
checkpoint non conserva il binario v3, la cui copia esatta è in target ignorato.
L'esito effettivo del ripristino è aggiunto soltanto dopo averlo verificato.

P1-02 resta DONE sulle proprie prove. P1-04 non è DONE e G-MIG-1 non passa.
P1-05 ha prerequisiti indipendenti già soddisfatti e può essere avviata in
sequenza dopo l'handoff fedele di questo blocco; P1-07/06 attendono i loro
prerequisiti. Nessun nuovo gate o obbligo ricorrente Windows.


Ripristino **PASS**: UUID/disco e stato spento verificati, snapshot revert/start
riusciti, marker base presente e delta assente,0 task P1-04, directory DPAPI e
Autologon assenti, nessuna password plaintext Winlogon. SSH admin elevato e
standard non elevato funzionanti;1.99.1 affiancata e stable1.100.0 conservate,
sorgente/cache v2 presente. Il checkpoint è quindi provato ripristinabile.
Indice delle evidenze con SHA: [manifest](T-MIG-P1-04-native/manifest.json).
