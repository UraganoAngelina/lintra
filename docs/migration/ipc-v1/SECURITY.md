# Sicurezza del contratto IPC — P2-08

15 novembre2026. Attuazione del threat model **già USER_APPROVED** in
[ADR-MIG-003](../../adr/ADR-MIG-003.md), senza nuova policy di prodotto.
P2-02 e P2-07 DONE; ratifica envelope distinta da G-MIG-2 ancora aperto.
Le permission del prodotto futuro non vengono dichiarate già qualificate.

## Autorità e ordine dei controlli

Il dispatcher custom sarà opt-in nell’AppManifest Tauri e nella capability
della sola WebView locale posseduta. Avere invoke_handler non basta: gli
handler custom non elencati possono eludere l’ACL prevista. Nessun core:default,
shell, fs, SQL, rete, updater, evento globale o comando generico di sistema.
Il trasporto raw unico non concede autorità a tutte le operation: registry Rust
chiuso e DTO esatto per operazione, seguito da controlli purpose/stato/owner.
Nomi/firme finali sono congelati dal gate P2-10 e implementati dopo quel gate.

Prima di ogni effetto: contesto nativo ammesso (origine esatta, finestra e
generazione di creazione), raw bytes,1MiB, UTF-8/depth32/JSON/versione, registry,
DTO specifico senza extra/coercizioni, handle e stato, limiti di concorrenza,
preflight dei servizi Rust. Un ID/label/origin dichiarato nel payload non
identifica il chiamante. Anche le chiamate dirette prive di UI devono fallire.
Le risposte subiscono lo stesso cap intero prima della pubblicazione.

Handle opachi sono scope-binding di oggetti nativi, non prove dell’intenzione
umana basate sul solo nonce. Path solo dal broker nativo e mai in errori/console.
Export usa conferme native per l’esatta selezione e autorizzazioni Rust del
servizio esistente. Nei89task MIG non si espongono candidate approval, PERSIST,
consensi storici C1, chiavi o token interni. Il consenso export non equivale
a PROCEED/Complete o approvazione di uno storico. API CAD restano nei task U-CAD.

## Minacce e prove obbligatorie

| Minaccia ADR-MIG-003 | Contratto/controllo | Evidenza corrente e verifica implementativa |
|---|---|---|
| XSS da record/metadata | Solo text node, niente HTML/URL/asset attivi | P1-04 nativo invariato; sentinelle prodotto P7-02 |
| Invoke abusivo | AppManifest/capability minimi più autorizzazione Rust | Audit ACL generata; DTO negativi P2; direct invoke reale P3/P7 |
| Path/link/scope swap | Broker e preflight/verify_unchanged nativi | Export60casi; fault/link e tutti gli input Full P5/P7 |
| Altro account | Root/cache/spool privati per-account, ACL OS | Prove prodotto due account P8; nessuna falsa garanzia da nonce |
| Bulk/flood | Byte/depth hard, pagine128/64, coalescing, lane dati/control | P2-02/04/05/06; stress bridge/DOM P4/P6/P7 |
| Esiti vecchi/race | Owner/run/query/selection, terminale autorevole | Lifecycle53/progress49 e stale paging/detail/export; thread reali P4/P7 |
| Approval non autorizzata | Nessuna operation storica nelle89MIG | Allowlist e test unknown/extra; CAS/preview nei task U-CAD |
| PHI in persistenza web/log | Solo finestra richiesta, niente cache/history nascosta | Projection P2; sentinelle log/cache/temp/DOM P7-03 |
| Exfiltration | CSP locale, navigation/popup/risorse nativi negati | P1-04 riusabile per spike identico; offline prodotto P7/P8 |
| Report corrotto/parziale | Writer Rust, receipt per-file e technical separato | Export60casi; parity/fault/durabilità nativi ai gate esistenti |
| Dipendenze alterate | Lock/provenienza e confine core/adapter | P2-01 audit dipendenze; verifica toolchain P3/P8 |
| Crash/chiave/backup | Recovery, key loader OS e restore originali | U-OPS/G-DUR invariati; nessun secure erase o power-loss universale |

Nessuna nuova suite sistematica introdotta: riusare una prova richiede input
identici e scope pertinente. Un test del modello non qualifica native IPC,
worker, DOM o piattaforme. Errore tecnico/protocollo non diventa un risultato
normativo valido/invalido; Unknown/Partial restano dimensioni del core.

## Audit permission/CSP effettivamente disponibili

audit_permissions.py legge anche i manifest **generati dalla build locale**:
gen/schemas/{acl-manifests,capabilities}.json, non solo il file dichiarativo.
Unica capability probe, local:true, windows:[probe], nessuna remote, due sole
permission esplicite risolte a runtime_probe/render_ack. Le altre permission
core presenti nel catalogo generato non sono automaticamente concesse.
AppManifest senza default/permission-set, devtools disabilitati, CSP default-src
none e connect-src limitato a ipc: e http://ipc.localhost della versione pinnata.

Il controllo confronta byte per byte main/guard/build/config/capability/manifest/
lock con608ccdc e verifica digest e risultati delle ricevute native P1-04
Linux/Win11 già archiviate: invoke vietati, payload errati, text-only,12eventi
CSP,0TCP/0HTTP. Non avvia WebView/VM/CI. Conferma lo spike esistente, non
un’ACL di prodotto che ancora non esiste. Quando P3 aggiunge gli handler dovrà
produrre i propri manifest effettivi e negativi, senza ereditare automaticamente
questo PASS. Nessuna permission aggiunta per far passare l’audit.

## Log, memoria e limiti reali

Error DTO solo category/code, nessun path/testo libero/stack/SQL/raw record.
Metriche solo conteggi e stati statici: niente identificatori di record, filtro,
nomi sensibili o dump dello stato React. No localStorage/sessionStorage/IndexedDB,
service worker, console PHI o cache applicative. Anche una pagina corrente
può contenere dati riservati: text-only non significa anonimizzazione.
Una pagina contesto/domino e finestre di testo richieste, nessun prefetch globale.

Nessuna telemetria/CDN/cloud o profilazione personale. I limiti applicativi non
provano un limite prima delle allocazioni Tauri/WebView né neutralizzano OS/admin
compromessi o dump runtime. CSP non prova da sola zero traffico del runtime/OS;
restano i guard e le prove native del piano. Nessuna cifratura custom o nuova
autenticazione. G-MIG-2 e G-REL sono checkpoint distinti ancora non superati.
