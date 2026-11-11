# ADR-MIG-003 — Confini di sicurezza del nuovo adapter locale

Owner: autore Lintra; decisioni D02–D10/DEC-MIG-03/04/08 già approvate in chat
il10 novembre2026 e recepite in U-GOV-03 `2cd4bdf`.
Trascrizione11 novembre2026, T-MIG-P0-06.
Stato: **USER_APPROVED / REPOSITORY_RATIFIED**; implementazione e test WebView
**NOT_VERIFIED**. Fonte: [ratifica](../unified-post-m9/RATIFICATION_2026-11-10.md)
e [master](../unified-post-m9/MASTER_PLAN_UNIFICATO.md).

## Modello di fiducia e responsabilità

Software locale per autore/colleghi autorizzati. L'account OS controlla i propri
dati; amministratore/OS compromesso e osservazione fisica dello schermo non
sono neutralizzati da HMAC o da una WebView. Nessun nuovo login, password,
server, cifratura, key rotation o programma enterprise viene introdotto.
Input, nomi file, metadati schema/reference e diagnostici sono potenzialmente
ostili per il renderer, anche quando il servizio Rust li ha correttamente
validati rispetto al tracciato. Una stringa valida non è HTML autorizzato.

Rust possiede file, dati, key provider, validazione, indice/query, report e
SQLite. Il renderer vede soltanto metadata/pagine/dettagli richiesti, con
payload limitati. Nessun file intero, JSONL intero, indice o array globale di
errori viene trasferito in JS. Chiavi/segreti e token protetti interni non
entrano in IPC, console, screenshot o storage web.

I comandi custom richiedono **controlli Rust effettivi** di autorizzazione,
purpose, stato e parametri: un file capabilities o una CSP da soli non provano
che un handler sia sicuro. Niente shell generica, SQL, lettura/scrittura arbitraria
di file o comando di approval sganciato dalla preview. Selezione nativa e
binding della richiesta devono delimitare gli oggetti sui quali l'utente ha
espresso intenzione; percorso, schema/versione e oggetto attivo vanno verificati
prima dell'I/O. Firme, identificatori e limiti wire concreti si chiudono inP2.

Una validazione attiva per istanza, stato/run/report posseduti dal Job Manager.
Paging, filtri e domìni restano Rust; il protocollo bounded applica limiti hard,
backpressure/coalescing del progresso e terminali affidabili. Nessun limite
può troncare la validazione, il JSONL canonico o l'audit per simulare successo.
L'override del budget soft non supera questi limiti di correttezza. I numeri
concreti non vengono inventati qui; sono specificati e testati nelle task IPC.

## Threat model e verifiche obbligatorie

| Minaccia | Controllo richiesto | Evidenza prevista |
| --- | --- | --- |
| HTML/script da record, metadati o note | Rendering text-only; niente innerHTML/URL/asset attivi da valori; CSP locale e navigazioni controllate | P1-04, P2-08, P7-02: sentinelle XSS, zero execution/remote request |
| Invoke custom abusivo / privilege escalation | Allowlist minima e autorizzazione Rust per comando; niente shell/fs generici; payload/stato verificati | P2-08 eP7: chiamate dirette invalide, non solo pulsanti disabilitati |
| Path swap, link o key source non sicuro | Broker/binding e controlli OS/file effettivi, key loader per handle; niente auto-repair/fallback | U-OPS-03, P2-08/P7-03; regressioni root/key preservate |
| Accesso da altro account | Root privata per-account e cache/spool isolati; Program Files senza DB mutabili o chiave comune pubblica | Win11 due account/non-admin inP8; fixture a due directory non basta |
| Bulk IPC o saturazione eventi | Streaming Rust, pagine/bytes bounded, lifecycle e terminale affidabile | P2/P4/P6/P7: richieste sovradimensionate, slow consumer, nessun blocco UI |
| Eventi/report vecchi dopo nuovo run | Identità/stato autorevoli Rust, reject handle non più valido e correlazione eventi | P4/P7: cancel/restart/race/eventi tardivi, niente risultati della run sbagliata |
| Approval o modifica indebita dello storico | Solo servizio Rust reale, preview/review e CAS, sessioni approvate immutabili | U-CAD/U-NORM-04, nessuna approval anticipata nelle89 MIG |
| PHI in log/cache/DOM persistente | Niente console tecnica di record; nessuna persistenza web PHI, profilo runtime privato e minimizzazione della pagina corrente | P7-03: sentinelle sintetiche in log/temp/cache/dump e DOM |
| Exfiltration da rete/font/CDN/updater | Asset locali, nessun endpoint esterno o updater, origini interne minime realmente verificate | P1/P7: osservazione release offline, policy effective e fallimenti espliciti |
| Report incompleto o copia pubblicata erroneamente | Writer canonici Rust, staging/receipt, stato parziale distinto; shared con avviso/consenso | MIG export, U-OPS-04 eG-DUR; niente transazione globale report+DB inventata |
| Dipendenze/payload alterati | Versioni/lock e licenze/provenienza verificate; hash e build native qualificate | P2/P3/P8/P9, nessuna dipendenza GUI nel core/CLI |
| Crash, perdita chiave o backup incompleto | Temporanei privati, recovery reale, chiave originale, BACKUP-B e restore isolato | U-OPS-05/06/08, U-REL-03; nessun secure erase o power-loss universale promesso |

## Contenimento dei dati e runtime offline

Dati personali non vanno in localStorage, sessionStorage, IndexedDB, service
worker/cache applicative del renderer o log di console/devtools. Una pagina
DOM corrente può mostrare i diagnostici richiesti dall'operatore; non conserva
l'intero input o una cronologia nascosta dei dettagli. Nessuna serializzazione
massiva dello stato React per debug. Produzione senza devtools/logging di dati;
le limitazioni reali del runtime e degli eventuali crash dump vanno documentate
e provate con sentinelle sintetiche, senza proclamare cancellazione fisica.

CSP/permission effettive includono soltanto origini asset/IPC interne necessarie
alla versione Tauri pinnata; niente wildcard o policy permissiva introdotta per
far funzionare una pagina. Config concreta e test negativi sonoP1/P2/P7, non
un header inventato da questo ADR. Il server Vite locale è solo uno strumento
di sviluppo con sintetici; la release non richiede quel server né una rete.
Export filesystem shared esplicitamente scelto resta possibile dopo warning e
consenso D04; ciò non autorizza HTTP applicativo, telemetry o cloud sync.

Font/asset e componenti frontend sono locali e versionati. Lockfiles e verifiche
licenze/supply chain precedono l'adozione effettiva; nessuna versione è dichiarata
sicura solo perché nominata nel piano. Il componente WebView non eredita un
PASS dai vecchi test Slint o dalla sola compilazione Rust.

I report normativi possono contenere valori diagnostici intenzionali e non
sono log anonimizzati. HMAC protegge identità/integrità, non cifra report/DB.
Persistenza legacy, ACL/ownership, codec, key bundle e chiave originale restano
invariati. Pulizia dei temporanei posseduti dal job non è retention/purge dei
dati; [gestione manuale](../MANUAL_DATA_MANAGEMENT.md) e
[custodia](../KEY_CUSTODY_RECOVERY.md) restano applicabili.

## Approvazione e limiti di questo recepimento

Questo ADR recepisce il livello di policy già approvato. Il contratto tecnico
wire/lifecycle, CSP/permissions e prove di attacco devono essere concreti e
verificati ai gate previsti. Nessun G-MIG-2/7 o G-REL passa per questo documento;
nessuna feature normativa o approval nasce dal renderer. Se una garanzia non è
ottenibile sul runtime qualificato, si registra il punto materiale senza
abbassare il controllo o inventare un'eccezione normativa.
