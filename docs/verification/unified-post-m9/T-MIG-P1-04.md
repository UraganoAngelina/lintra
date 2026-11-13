# T-MIG-P1-04 — CSP, capability e navigazioni native

13 novembre 2026. **IN_PROGRESS**, G-MIG-1 aggregato NOT_VERIFIED.
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
i guard. Windows e CI del delta ancora NOT_RUN. Nessun DONE, release o gate aggregato anticipato.

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
