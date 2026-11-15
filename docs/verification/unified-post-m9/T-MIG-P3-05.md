# T-MIG-P3-05 — Client TypeScript e schema del bridge

15 novembre 2026. **DONE** dopo P3-04: 14 test Node mirati, 11 test Rust,
typecheck strict, build frontend e handshake nativo Wayland PASS.
[Receipt](T-MIG-P3-05-checks.json). Windows sul delta attende P3-06.

`ipc.ts` è il client delle sole operazioni di bootstrap. `encodeRequest`
costruisce byte UTF-8 con envelope/DTO v1 e un request ID UUID. `exchange`
chiama l'unico comando pubblico Tauri e riceve `unknown`: nessun cast del
trasporto a un DTO fidato. `decodeReply` richiede ArrayBuffer entro 1 MiB,
UTF-8 strict senza rimozione del BOM, profondità 32 prima del parser JSON,
chiavi uniche anche escapate, interi safe senza float e Unicode valido.
Controlla poi versione, shape esatta, correlazione e taxonomy. `appInfo` e
`clientReady` controllano i propri DTO esatti prima di esporre tipi al renderer.

Gli otto vettori JSON condivisi confrontano il vero dispatcher Rust con
risposte attese indipendenti, decodificate anche dal client TS. Includono
unknown fields, version mismatch, duplicati e stato non valido. Le prove TS
aggiungono byte/depth ai limiti, correlazioni errate, categorie incoerenti,
surrogate isolate e capability non previste. I mock della API Tauri installata
provano comando e body reali; lo smoke nativo prova il trasporto WebKitGTK
effettivo. Nessun mock viene presentato come prova nativa.

React carica le informazioni una volta, aspetta commit, font e due frame di
rendering prima dell'ack. Gli effect ignorano completamenti dopo unmount;
loading e fallimento sono annunciati con status/alert. Errori e fallback usano
solo testi statici con causa e riapertura dell'app: nessun dato/stack/path libero,
nessun fallback di protocollo o stato normativo inventato. Nessuna norma viene
duplicata in TS. Il servizio di validazione rimane assente in questa fase.

I test Node girano nello stesso processo (`--test-isolation=none`), con cleanup
dei mock dopo ogni caso; questo rende visibili i 14 esiti anche nell'ambiente
locale, dove il runner isolato riportava soltanto l'esito del file contenitore.
Il lint frontend usa le regole statiche TypeScript già installate (unused e
fallthrough), senza aggiungere ESLint o una dipendenza per questa shell.
G-MIG-3 ancora aperto; segue P3-03 e poi integrazione P3-06.

Integrazione su ce344d4: CI desktop43758529844 SUCCESS Linux/Windows,
16 test Node per OS, 11 test Rust Linux/12 Windows e smoke React/raw IPC/Rust
nativo su entrambi PASS. Receipt aggregato in `T-MIG-P3-06-checks.json`.
G-MIG-3 attende la CI ufficiale core/Slint, separata da questa prova desktop.
