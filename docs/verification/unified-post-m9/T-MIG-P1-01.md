# T-MIG-P1-01 — Spike runtime Linux

11 novembre 2026. Stato iniziale: **IN_PROGRESS**. Prerequisito P0-08 DONE,
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

Verifiche fin qui: build frontend PASS con 17 moduli e asset locali; TypeScript
strict, `skipLibCheck=false`; fmt Rust PASS; detector UI zero rilievi; diff check
PASS. Il primo typecheck segnalava due dichiarazioni mancanti: risolte con
ESNext.Disposable e tipi Vite client già inclusi negli strumenti approvati,
senza ignorare errori o aggiungere runtime/polyfill. Build Rust nativa in corso.

Reference aggiuntive dell'autore ricevute in chat e consultate con vision,
con hash in context.json; brief UX aggiornato con l'uso previsto. Cartella
esterna preservata. I concept di altri framework sono riferimenti di layout,
non scelte tecniche né dati/regole da trasferire nel prodotto.

Mancano ancora eseguibile nativo, apertura finestra e smoke IPC/render reali.
Non attribuire la CI core cb70bee a questi nuovi sorgenti. DPI, rete/capability
negative complete e clean-room Windows restano prove distinte P1-02..07;
nessun PASS NSIS, installazione, due account o distribuzione da questo spike.

Prossimo passo: terminare build nativa, eseguire su CachyOS/Wayland e archiviare
log/versioni/hash dell'eseguibile e capture della sola finestra sintetica.
Next indipendente READY: P1-02; P1-03 soltanto dopo P1-01 effettivamente DONE.
