# ADR-PERSIST-002 — scelta del backend persistente locale

Stato: PROPOSTA, da approvare dall’autore. Data:2026-10-28.
Task: T-PERSIST-05, preflight dopo PERSIST03/04 DONE e CI40658672940 positiva.
Nessun backend o nuovo driver è stato aggiunto.

## Decisione proposta

Adottare SQLite incorporato e collegato staticamente, dietro i port generici
esistenti. Nessun server, installazione SQLite sul PC dell’analista o dipendenza
runtime dalla SQLite di sistema. La proposta richiede esplicitamente l’eccezione
alla preferenza pure-Rust di N-20: il motore SQLite è C. Binding/versioni/feature,
licenze e checksum saranno qualificati prima di modificare Cargo, come DEC-15.

La motivazione è ridurre il protocollo transazionale e gli indici disco che
Lintra dovrebbe implementare e verificare autonomamente per il filesystem.
Questa è una valutazione progettuale, non un benchmark di superiorità SQLite.
Non approva norme SIOC/CAD, cifratura di payload futuri, retention, backend
remoto o attivazione della modalità storica nel servizio attuale.

## Confronto

| Aspetto | Filesystem con snapshot/manifest atomici | SQLite incorporato |
|---|---|---|
| Build Linux/MSVC | std/serde già qualificati; resta pure-Rust | Richiede build C statica e verifica packaging sui due runner; eccezione N-20 esplicita |
| Atomicità/restart | Protocollo Lintra per staging, flush, pubblicazione, recovery e crash tra file | Journal/transazione del motore; Lintra verifica comunque CAS, errori e restart |
| Query identità storiche | Serve indice derivato su disco o RAM; scansioni ripetute non sono una soluzione scalabile | Indice database sullo scope/identità canonici, senza caricare tutte le versioni |
| Memoria | Dipende dall’indice scelto; non si può riusare il fake conservando tutti gli snapshot in RAM | Query puntuali e cache configurabile; il port load_snapshot resta O(n) perché restituisce un oggetto posseduto |
| Backup | Copia di un insieme coerente di file, fissato dal manifest e dal lock | API backup oppure copia soltanto a database chiuso e verificato |
| Corruzione/versioni | Envelope e parser versionati, digest e recovery progettati in Lintra | Versione storage/logica esplicita e controlli del database più validazione delle DTO |
| Dipendenze | Minime; maggior codice storage di cui siamo responsabili | Nuova catena da qualificare; minore protocollo filesystem proprietario |

Non sono stati eseguiti benchmark comparativi di backend. DEC-14 e soglie di
prodotto restano aperte; la scelta non introduce numeri prestazionali fittizi.
I test sintetici fino a centinaia di migliaia di identità dovranno misurare
memoria, costo lookup e commit sul backend scelto, non sul fake03.

## Perimetro proposto dell’implementazione05

- Adapter dietro PersistentContextRepository/PersistenceTransaction: nessun SQL
  o nome di flusso nell’engine, nessun if SIOC/CAD e nessuna regola clinica.
- Formato storage versionato distinto dallo schema normativo. Conservare
  snapshot/revisioni, SourceSubmission, ultime identità osservate e audit di
  protezione; omettere una chiave da un nuovo snapshot non cancella lo storico.
- Staging privato in memoria; soltanto commit entra nella transazione disco,
  verifica nuovamente la generazione e pubblica insieme indice, snapshot e audit.
  Drop/abort non pubblicano dati. Nessuna approvazione clinica implicita.
- Primo assetto da qualificare: rollback journal e sincronizzazione completa,
  niente journal disabilitato, WAL attivato per comodità o DB su share di rete.
  Le opzioni precise devono essere provate sui runner, non soltanto documentate.
- Solo identità/proiezioni protette compatibili con04 e metadata necessari.
  Chiavi esterne tramite provider configurato, nessun segreto nel DB, log o repo.
  Nessun nuovo campo raw/CF/record recuperabile nel codec di default.
- Validare ogni dato recuperato tramite costruttori/contratti di dominio; errore
  di corruzione, versione non supportata, chiave o lock resta tecnico e redatto.
- Root e namespace isolati, path non costruiti da identificativi paziente,
  confini symlink/permessi/ACL qualificati prima di qualsiasi uso reale.
- Test veri tra processi per restart, contesa e terminazione durante commit;
  file corrotto, mismatch generazione, isolamento e nessun successo fittizio.
  Errori dopo un punto di pubblicazione non possono essere dichiarati rollback
  senza verifica: un’eventuale ambiguità contrattuale va risolta esplicitamente
  prima di introdurre nuove semantiche di commit o recovery.

Backup/restore completo, lifecycle delle approvazioni e servizio storico restano
alle rispettive task;05 non ne dichiara automaticamente gli acceptance gate.
Nessun archivio clinico reale viene aperto per scegliere o testare il backend.

## Alternativa pronta se si preferisce pure-Rust

Scegliere filesystem locale con snapshot immutabili, manifest generazionale,
lock cross-process e indice derivato ricostruibile. Il progetto dovrà prima
precisare il protocollo di pubblicazione/recovery e misurarne gli indici; non
basta sostituire una write con rename per dichiarare crash safety. Nessun
cambio delle API/garanzie di atomicità sarà implicito nella scelta filesystem.

## Fonti primarie verificate

Rust1.99.1 offre try_lock e sync_all; la rename ha dettagli dipendenti da OS e
filesystem. Queste primitive sono componenti utilizzabili, non una prova del
protocollo multi-file di Lintra. [File](https://doc.rust-lang.org/std/fs/struct.File.html#method.try_lock),
[rename](https://doc.rust-lang.org/std/fs/fn.rename.html).

SQLite documenta il proprio journal/commit e le assunzioni sullo storage;
il comportamento dell’adapter deve essere comunque verificato. [Atomic commit](https://sqlite.org/atomiccommit.html).
Il motore incorporabile deriva dall’amalgamation C: da qui l’eccezione pure-Rust
richiesta, non una dipendenza C aggiunta silenziosamente. [Compilazione](https://sqlite.org/howtocompile.html).
La copia ordinaria di file attivi non sostituisce un backup coerente: sono
documentate API dedicate e gestione del journal. [Backup](https://sqlite.org/backup.html),
[WAL](https://sqlite.org/wal.html).

## Approvazione richiesta

Fonte: TASK_PROMPTS/T-PERSIST-05.md, «implementa il backend approvato»;
ADR-PERSIST-001 §1 rinvia la scelta a una decisione tecnica; N-20 richiede una
decisione esplicita per dipendenze non pure-Rust. Le autorizzazioni permanenti
Git/CI non scelgono automaticamente un database o questa eccezione.

Scelta A, consigliata: SQLite incorporato/statico, con eccezione C circoscritta
al motore e qualificazione dipendenze prima del manifest. Scelta B: filesystem
pure-Rust, con protocollo atomicità/indici da qualificare prima dell’uso reale.
Fino alla scelta, nessuna implementazione05 o nuova dipendenza è autorizzata
implicitamente. UI05 e le attività indipendenti proseguono con i propri gate.
