# Inventario locale di dati, report e temporanei

Verificato sul codice post-M9 con U-FIX-03/04, novembre 2026. Le colonne
«corrente» descrivono sorgenti effettivi; la root Tauri è un requisito da
integrare, non una directory già creata o una prova multi-account superata.

| Superficie | Percorso corrente / proprietario | Contenuto e durata effettivi | Destinazione Tauri / qualifica residua |
|---|---|---|---|
| Input selezionati | Path esplicito CLI/Slint, file letto da Rust | Byte originali esterni, nessuna copia raw persistente implicita | Selezione backend, nessun file intero in JS |
| Schema/reference | Bundle esplicito, adiacente all'eseguibile o embedded; Rust registry/preparer | Cataloghi e schema, hash dei byte realmente caricati, snapshot immutabile | Statici sotto app/bundle controllato, controlli tree U-FIX-04 |
| SQLite | `--archive` esplicito; `<root>/lintra.db`, adapter Rust | Snapshot/provenienza/PK protette/metadati/fatti, persistenti | `%LOCALAPPDATA%/Lintra/archives` per account; U-OPS-03 e bridge storico |
| Rollback journal | `<archive>/lintra.db-journal`, SQLite DELETE/EXTRA | Temporaneo transazionale/recovery; non cancellarlo manualmente | Stessa root privata; fault/reopen in G-DUR |
| SQLite temporanei | Gestione SQLite/OS, `temp_store=FILE` | Lavorazione interna, senza promessa di cancellazione fisica | Controlli OS/root e recovery, non finta transazione con i report |
| Chiave comune | CLI `--key-file` + Key ID; convenzione privata Slint `hmac.key` presso eseguibile | 32 byte originali, provider in Rust con redazione/zeroize; nessuna rotazione | Copia per account/private bundle con continuità e custodia U-OPS-05; mai artefatto CI operativo |
| Spool CLI | ErrorSpool: NamedTempFile OS se JSONL non richiesto; altrimenti AtomicReportFile | JSONL canonico completo, può contenere valori diagnostici sensibili; close/cleanup espliciti | Spool privato della root account, senza nomi paziente; UI/core cleanup verificato |
| Spool Slint | TempDir `lintra-analysis-*`, worker Rust; errors.jsonl e `.part` dentro la directory | Report owned dal risultato; resta vivo mentre risultati/export ne hanno ownership | `%LOCALAPPDATA%/Lintra/spool`, ownership Rust Job Manager e no cache persistente JS |
| Indici/cache risultati | Rust JsonlModel: offset su file e cache128 righe | Memoria dell'istanza, può contenere dettagli selezionati; nessun archivio globale di tutti gli errori | Paging/query Rust bounded e virtualizzazione DOM, MIG-P2/P4/P6 |
| XLSX temporanei | UI passa la directory privata del report a with_temp_dir; CLI usa temp OS del backend | Worksheet/spool necessari a constant-memory; RAII/cleanup backend | Directory privata per job, verifiche U-OPS-03/MIG-P7-03; nessuna promessa di secure erase |
| Report esportati | Path scelto dall'operatore; JSONL/CSV/XLSX e summary separati | Possono contenere dati reali autorizzati; nessuna TTL o redazione silenziosa | Default report per account; locale scrivibile libero, shared con warning/consenso U-OPS-04 |
| Report `.part` | Stessa directory della destinazione, create_new; AtomicReportFile Rust | Staging, no follow/truncate di part preesistenti; pulizia/parzialità esplicite | Eredita il confine della destinazione; non confonderlo con spool privato interno |
| Export receipt | UI ExportReceipt/ExportFailure Rust, elenco file pubblicati | Stato di singoli output, nessuna transazione globale inventata | DTO bounded e gestione parziale MIG export |
| Preflight budget | Oggetto Rust con request/hash schema/path/fingerprint; audit separato | Memoria del preflight; nel log solo contatori e scelta | Handle opaco Rust, Annulla/Procedi comunque in React, nessuna serializzazione degli oggetti interni |
| Log tecnici | Eventi fissi CLI/worker, errori tipizzati; niente log automatico di tutti i diagnostici | Non sono il report normativo; chiavi/record non stampati dal preflight | Nessuna PHI in console/devtools/log; prove sentinella/XSS/DOM in MIG-P7 |
| Configurazione | CLI flag/path e selezioni in memoria Slint; budget JSON esplicito | Nessun nuovo server o store di credenziali; metadati/nome file devono restare privi di identità paziente | `%LOCALAPPDATA%/Lintra/config`; niente web storage persistente di dati sanitari |
| WebView/cache/browser storage | **Assenti dal prodotto corrente:** nessun progetto Tauri/React ancora creato | Nessuna prova WebView corrente da dedurre dall'assenza | Profilo cache per account, CSP/offline e verifiche localStorage/IndexedDB/cache/DOM/crash nei gate MIG |
| Backup/copie | Procedura chiusa controllata esistente; nessun wizard/backup live prodotto | Copie persistenti gestite manualmente, key bundle separato | BACKUP-B verificato pre-upgrade e restore isolato U-OPS-06/08 |

Root target Windows: `%LOCALAPPDATA%/Lintra`; Linux solo personale:
`$XDG_DATA_HOME/Lintra`, fallback `$HOME/.local/share/Lintra`. App e asset statici
Windows in Program Files; nessun database mutabile presso l'eseguibile.
Questi percorsi target richiedono provisioning e ACL nativi: non sono ancora
il comportamento automaticamente attestato di Slint o della CLI.

I report normativi e il riepilogo umano CLI sono output intenzionali: possono
mostrare diagnostici o basename selezionati dall'operatore. Non sono log tecnici
anonimizzati. Non mettere identificativi paziente nei nomi di file, submission
ID o metadati; HMAC non bonifica testo libero e non cifra il report.

Nessun file operativo, archivio reale o chiave dell'autore è stato aperto per
questo inventario. Verifiche e sentinelle restano sintetiche. Non si afferma
assenza universale di PHI da ogni output o da una WebView non ancora presente.
Il mapping guida U-OPS-03/04/05/06/08 e MIG; nessun gate nativo anticipato.

Fonti di codice: src/cli/{args,bundle,error_spool,command}.rs,
src/persistence/sqlite/mod.rs, src/persistence/file_security*,
src/persistence/key_file.rs, src/report/{atomic,xlsx}.rs,
ui/src/{worker,export}.rs e ui/src/results/mod.rs,
src/application/resource_budget.rs e src/cli/resource_budget.rs.
