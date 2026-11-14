# T-MIG-P1-04 — CSP, capability e navigazioni native

14 novembre 2026. **DONE — prove native e CI del delta PASS**;
G-MIG-1 aggregato NOT_VERIFIED. La cronologia del13 novembre resta sotto.
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

## Ripresa 14 novembre 2026: connessioni del documento

Baseline c33cdb8, working tree iniziale pulita; P1-05 e33/128 DONE conservati.
Il nuovo workflow è recepito in AGENTS.md. Installazioni e ricevuta delle skill
sono esclusivamente utente, fuori dal repository; nessuna milestone o suite
di regressione introdotta per il setup.

Il delta candidato usa `WebviewWindowBuilder::on_web_resource_request`, API
verificata nel sorgente della Tauri2.12.5 pinnata. Prima del primo documento
aggiunge alle risposte asset `Connection-Allowlist` con i soli pattern
`http://tauri.localhost/*` e `http://ipc.localhost/*`, intercettati da Wry.
`restrict_document_connections` sostituisce eventuali policy precedenti,
senza duplicarle o cambiare la CSP. Il callback è compilato solo Windows;
nessuna nuova dipendenza, capability, norma o modifica core/CLI/Slint.
Il checkpoint successivo estende soltanto il DTO privato della sonda.
La [documentazione Chromium](https://developer.chrome.com/blog/connection-allowlist-announcement)
descrive il contenimento prima delle connessioni, compresi canali non coperti
dalla CSP. Il supporto nel WebView2 concreto richiede comunque prova nativa.

Livello A: cinque test Rust Linux PASS, incluso il nuovo test di sostituzione
della policy permissiva, idempotenza e conservazione CSP; Clippy locale
`--all-targets -- -D warnings` PASS. MSVC1.99.1: cinque test, Clippy e build
release custom-protocol PASS. Nessuna suite workspace o golden rieseguita.
Frontend e harness sono riutilizzati dal codice qualificato P1-05; il manifest
di39file identifica sorgenti/font/dist, inclusi i delta locali. Lo staging
iniziale ha rifiutato i due font ereditati perché assenti dalla nuova allowlist:
errore conservato, corretto ammettendo solo quei nomi esatti, prima dell’overlay.

La prima prova desktop standard/sessione1 osserva sul binario SHA256
`1aeb5cc0274dd44c5187085defe9ad1c1373d646a9ae0341851b34fdddfb75a0`
(9037824byte) ACK ordinario900×600/exit0, guard popup e navigazione attivati,
**0TCP/0HTTP**, ma nessun report finale: timeout/SIGTERM, harness1 e launcher
FAIL. Il solo successo del contenimento di rete non chiude l’acceptance.

Diagnosi separata sullo stesso binario, senza ricompilarlo: CDP limitato a
`127.0.0.1:9227` tramite variabile del solo processo figlio, secondo il
[metodo Microsoft per il debugging](https://learn.microsoft.com/en-us/microsoft-edge/webview2/how-to/debug-visual-studio-code).
Non è una qualificazione e non viene usata per attribuire un PASS alla rete.
La pagina osservata dopo l’attacco è `chrome-error://chromewebdata/`, titolo
sintetico127.0.0.1. L’invoke riceve `Origin header is not a valid URL`/HTTP500:
il confine IPC rifiuta l’origine non valida, mentre il documento locale è stato
sostituito. Non è un motivo per consentire quell’origine o indebolire l’ACL.
Disabilitare la built-in error page non preserva il documento: Microsoft
[documenta una pagina vuota](https://learn.microsoft.com/en-us/microsoft-edge/webview2/reference/win32/icorewebview2settings),
quindi non viene adottato come correzione.

Il tentativo successivo aggiunge nel solo adapter SDK un NavigationStarting
che verifica ancora `local_asset_origin`, imposta Cancel e invoca `Stop()` sul
sender COM del medesimo evento. URI malformati vengono negati; argomenti nulli
ed errori SDK impostano failure ed exit1. Nessun riferimento COM preso in
prestito esce dal callback. L’esito del tentativo viene registrato sotto dopo
il collaudo; non è dichiarato risolutivo dalla sola compilazione. Test puri e
frontend invariati vengono riutilizzati; si ricompila solo il delta Windows.

Le strumentazioni temporanee JS/Node della diagnosi sono state rimosse dai
sorgenti; il frontend precedente è stato ripristinato byte per byte dal suo
archivio identificato. La prova successiva rifiuta un listener CDP ancora
attivo e conserva separatamente gli output del primo fallimento.


## Blocco integrato: contenimento, recovery e checkpoint sintetico

Il tentativo SDK `Stop()` descritto sopra e la variante accodata sul thread
Tauri sono stati **scartati**: build/Clippy passavano, ma la prova non produceva
il report finale e l’input si bloccava. `windows_request_guard.rs` è tornato
esattamente alla versione qualificata precedente, senza quei callback COM.

`DocumentState::loaded` conserva l’entry asset attendibile, riconosce un
Finished estraneo e accoda il ritorno a quell’URL tramite l’executor Tauri.
Non trasferisce riferimenti COM e non naviga mantenendo il mutex acquisito.
Origine di errore, target diverso, recovery ricorsiva e overflow non acquistano
autorità: il processo fallisce. Lo spike ricarica la pagina; non conserva stato
React arbitrario o lavori del prodotto futuro.

Per permettere alla prova di attraversare un reload reale, `checkpoint` salva
in Rust un solo `SecurityReport` completo dei controlli CSP/ACL già eseguiti,
con i due dinieghi nativi ancora falsi. Forma e dimensioni restano bounded,
nessun dato di lavoro entra in memoria DOM persistente o nei log. `resume`
verifica l’URL esatto e deriva i due flag da contatori distinti dei target
sintetici canonici `/popup` e `/navigation`. Un popup non sostituisce la seconda
prova. `render_ack` non emette successo al checkpoint e confronta il risultato
finale con la copia Rust. Query miste o fuori dalla modalità security sono
rifiutate; il normale protocollo1 continua a serializzare gli stessi campi.
Il renderer ricontrolla sentinella testuale e IPC dopo il reload.

Il driver usa `windowsHide` per il helper PowerShell: la sua console non deve
sottrarre il focus. Prima di Enter resta obbligatoria la verifica del PID
foreground. Un tentativo precedente con focus errato è conservato come FAIL,
con driver/launcher1 e terminazione dell’albero GUI posseduto. Timeout e
fallimento del taskkill non possono produrre PASS; output/pipe e collettore
sono chiusi senza lasciare la prova appesa.

Verifiche A/B sul blocco corrente:

- TS strict/Vite,9 test Rust Linux, Clippy all-targets `-D warnings`: PASS.
  Test su confini origin/wire, allowlist header, recovery/overflow, URL esatto,
  controlli incompleti, query illegali e distinzione dei due attacchi.
- Release MSVC1.99.1 e Clippy all-targets: PASS. I9 test Windows correnti
  sono demandati alla CI ufficiale; i5 test della prima variante non li sostituiscono.
- Wayland: binario080a26ee4af83f829d13544ee18734b56a93d00b889e4b2149aaeb83d2c8bf87,
  tutti18 flag veri,12 eventi CSP,1popup/1navigazione,0TCP/0HTTP,exit0.
- Windows11 build26300/WebView2 157.0.4341.46, console1 standard:
  binario976167f807ac6a2ed2cc8514513f79148df928266189c47194f687bcea78f5b1,
  9061888byte; smoke ordinario900×600/exit0 riutilizzato sullo stesso binario.
  Prova finale:18 flag veri,12 eventi CSP,1popup/1navigazione,1recovery,
  **0TCP/0HTTP, processo/driver/task/launcher0**. Nessun CDP attivo nella prova.
  Il warning WebView2 di uscita `Failed to unregister class ... Error = 1412`
  è preservato nello stderr; non ha impedito ACK né uscita0.
- Driver negativo con processo non GUI e con spawn EACCES: exit1/PASS false.
  Il fallimento reale del focus ha verificato anche la chiusura dell’albero
  Windows; non è stata aggiunta una seconda prova artificiale equivalente.

Ponytail-review circoscritta a main.rs, renderer, chiamanti IPC e driver:
nessun finding bloccante sul carico della singola finestra sintetica. Nessuna
modifica a SDK/lockfile/core/CLI/DSL/SQLite o riduzione dei predicati del test.
Header supportato dal runtime provato, non garanzia per WebView2 precedenti
che lo ignorano o per l’intero traffico OS/redistributable. I gate applicativi
PHI/cache/job/persistenza restano nelle task previste.

Evidenze con hash, sorgenti esatti e tentativi negativi separati:
[manifest14 novembre](T-MIG-P1-04-native/2026-11-14/manifest.json).
Il manifest runtime identifica gli input; il nuovo hash separato del driver
registra la correzione `windowsHide`, successiva alla build ma senza delta del
binario o del frontend. La CI precedente non viene attribuita a questo stato.


Ripristino14 novembre PASS: snapshot `LINTRA-P1-04-BEFORE-DESKTOP-20261113`
con UUID/dischi/stato spento controllati, revert/start0, marker base presente,
marker/directory/task del tentativo assenti, nessun autologon o password
plaintext Winlogon. SSH admin e standard verificati; richiesto shutdown pulito.
Questi controlli non qualificano un nuovo installer P1-07. La CI del blocco è
[43595367523](https://github.com/UraganoAngelina/lintra/actions/runs/43595367523)
sul commit608ccdc1d79c8495085ade5888e8df99f97c958f: **SUCCESS** (TS strict,
9test MSVC, fmt/Clippy, release e NSIS). Log archiviati separati dalle prove
native. VM finale `shut off`. **T-MIG-P1-04 DONE**,34/128 (26,6%).
Prossima READY **T-MIG-P1-07**; P1-06 e G-MIG-1 attendono ancora l’installer.
