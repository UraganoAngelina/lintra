# ADR-MIG-001 — Sostituzione mirata N-24 per il nuovo adapter desktop

Decision owner: **autore del progetto Lintra**. Decisione ricevuta in chat
il10 novembre2026 (D01–D15 e mandato unificato; DEC-MIG-01/04/05/07/08 già
approvate). Recepimento: Codex,11 novembre2026, T-MIG-P0-03.
Stato: **USER_APPROVED / REPOSITORY_RATIFIED**, gate tecnici **NOT_VERIFIED**.

Autorità: [ADR-POST-M9-001](../unified-post-m9/RATIFICATION_2026-11-10.md),
recepito in U-GOV-03 `2cd4bdf`, e
[master unificato](../unified-post-m9/MASTER_PLAN_UNIFICATO.md) §§1/6/7.
Non si inventano una nuova firma, un ID chat o una nuova approvazione tecnica.
La ratifica dell'autore è già acquisita; questo ADR la rende consultabile nel
repository. L'approvazione del prodotto non rende PASS i gate MIG.

## Decisione e ambito sostituito

Il nuovo eseguibile desktop usa **React, TypeScript strict, Vite, Tailwind CSS,
shadcn/ui, TanStack Table/Virtual e Tauri2**. Questa scelta sostituisce la
prescrizione esclusiva Slint di N-24/ARCH§39D/DET Addendum B **per il nuovo
adapter**, dopo il gate G-MIG-0. Non sostituisce la library, la CLI o i contratti
normativi; il package condiviso non viene rifatto per spostare clap.

Slint resta operativo e verificato durante la migrazione. Cutover e ritiro
seguono parità LocalOnly+Full, rollback e gate previsti; i test/contratti Slint
restano applicabili al prodotto legacy finché presente. L'adozione React non
riapre le140 task M9 e non cancella le loro evidenze.

Il nuovo adapter è separato dal core/CLI: dipendenze GUI/WebView e frontend
appartengono all'adapter. Rust conserva parsing, compile dello schema, regole,
reference, validazione, proiezioni, namespace, indici/query, report e SQLite.
React presenta lo stato e raccoglie richieste: non implementa regole cliniche,
confronti PK, storia CAD/SIOC o classificazioni normative in TypeScript.
I controlli D11 approvati restano nell'implementazione Rust qualificata.

Un Rust Job Manager possiede file, spool e risultati, con **una validazione
attiva per istanza**. IPC typed e bounded; risultati paginati/virtualizzati,
progresso separato dal dettaglio JSONL. Cancelled, Partial, Unknown, errori
tecnici e normativi conservano il proprio significato. Le firme e i limiti
concreti, stati run/report e ownership saranno verificati nei contratti MIG-P2;
questo ADR non finge una firma già implementata né inventa soglie numeriche.

## Parità, scope e sicurezza

LocalOnly e Full multi-tracciato sono entrambi obbligatori. Non basta riprodurre
la Slint attuale a file singolo. CAD storico è nello stesso master attraverso
U-CAD-01..09 dopo U-UI-02; SIOC normativo e il suo contesto applicativo vengono
collegati al servizio Rust dopo G-MIG-4. Nessun audit-only con flag artificiale
rende disponibile una approval. Nessuna capability è presentata pronta senza
il proprio gate e il perimetro dichiarato dal profilo.

Redesign moderno e responsive durante la migrazione; parità di comportamenti,
stati, diagnostici ed export. Token di design condivisi e font/asset locali.
Le immagini della nuova WebView si qualificano per OS insieme ad accessibilità
e keyboard/focus; non si pretende identità RGB fra motori WebView differenti.
Il vecchio confronto RGB Slint continua a valere per la vecchia UI. Brief,
viewport, prove e budget prestazionali saranno definiti inP0-07 e qualificati
nelle task previste, senza soglie DEC-14 o parallelismo automatico.

Offline/local-only: niente rete applicativa, CDN, telemetry, updater o server.
Dati/file posseduti da Rust, niente storage web persistente di dati sanitari o
key material nel renderer. Nessuna nuova cifratura/password custom/retention;
ACL/ownership e chiave comune esterna32B invariati. Export locale consentito,
shared con warning/consenso; receipt e writer canonici preservati. Threat model
e supply chain/versioni/licenze saranno recepiti inP0-06 e nei gate tecnici.

GUI Windows11 x86_64 con NSIS offline/perMachine e WebView2 offline; Linux GUI
solo build/smoke personale CachyOS. Le eccezioni N-19/N-20/N-23/QG-5/QG-7 sono
limitate all'adapter; policy core/CLI invariata. Dettagli OS e prove inP0-05/ADR
relativo; nessuna promessa di portabilità Linux GUI universale. Windows2025 CI
non equivale alla clean-room Windows11 o a due account/non-admin.

## Verifica e condizioni di efficacia

P0-03 chiude il recepimento della decisione N-24, non G-MIG-0. Tauri non viene
implementato prima diP0-08; gli spike, il bridge, la sicurezza e la distribuzione
mantengono tutti i gate originari. MM9/cutover non è già distribuzione ai colleghi:
G-REL richiede MIG9+CADUI3+SIOC1+OPS2+DUR, installer qualificato e signoff.
Non è concessa pubblicazione/tag/release arbitraria.

Source of truth corrente per questo delta: questo ADR e master unificato;
ARCH§39D/DET Addendum B conservano i dettagli legacy dove non sostituiti.
La voce N-24 del master originale rinvia qui, con applicabilità esplicita.
Gli ADR di OS/security e i contratti MIG successivi non possono spostare
norme nel renderer, indebolire CAS/immutabilità o abbassare i gate.

## Testo N-24 precedente conservato come storia

«UI: Slint, eseguibile separato, renderer software di riferimento, font
incorporati, token di design unici, attribuzione Slint in “Informazioni”, test
visivi in CI; client di ValidationService, nessuna logica normativa».
I dettagli applicabili a Slint restano in ARCH§39D/DET Addendum B. I principi
separazione, token/font locali e client del servizio Rust restano nella nuova
UI; renderer Slint, attribuzione specifica e pixel equality non si trasferiscono
come vincoli tecnici impliciti alla WebView. Le attribuzioni delle dipendenze
adottate devono essere realmente verificate/versionate, non dedotte da questo ADR.
